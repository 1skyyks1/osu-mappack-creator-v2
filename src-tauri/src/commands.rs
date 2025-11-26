use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use chrono::Local;
use serde::Serialize;
use serde_json;
use tauri::{Window, Emitter};
use tauri_plugin_dialog::DialogExt;
use regex::Regex;
use zip::write::FileOptions;
use zip::CompressionMethod;

const INDEX_FILE_NAME: &str = "beatmap_index.json";
const DELETE_OSU_TEMPLATE: &str = include_str!("../resources/delete.osu");
const DELETE_BG_BYTES: &[u8] = include_bytes!("../resources/delete.jpg");
const DELETE_AUDIO_BYTES: &[u8] = include_bytes!("../resources/delete.mp3");
const LOG_FILE_NAME: &str = "osu-mappack-creator-v2.log";

fn log_file_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(LOG_FILE_NAME);
    path
}

fn persist_log(level: &str, message: &str) -> Result<(), String> {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tag = level.to_uppercase();
    let line = format!("[{}][{}] {}\n", timestamp, tag, message);
    let log_path = log_file_path();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|err| format!("Failed to open log file {}: {}", log_path.display(), err))?;
    file.write_all(line.as_bytes())
        .map_err(|err| format!("Failed to write log file {}: {}", log_path.display(), err))?;
    Ok(())
}

fn log_debug(message: &str) {
    if let Err(err) = persist_log("DEBUG", message) {
        eprintln!("Failed to persist log entry: {}", err);
    }
}

fn log_error(message: &str) {
    if let Err(err) = persist_log("ERROR", message) {
        eprintln!("Failed to persist log entry: {}", err);
    }
}

#[derive(Serialize, serde::Deserialize)]
pub struct Beatmap {
    pub id: String,
    pub path: String,
}

#[derive(Serialize)]
pub struct BeatmapPage {
    pub beatmaps: Vec<Beatmap>,
    pub total_dirs: usize,
    pub total_pages: usize,
}

#[derive(Serialize)]
pub struct BeatmapIndexResult {
    pub beatmaps: Vec<Beatmap>,
    pub index_path: String,
    pub total: usize,
    pub duration_ms: u128,
}

#[derive(Serialize, serde::Deserialize, Clone, Default)]
pub struct BeatmapMetadata {
    pub title: String,
    pub artist: String,
    pub creator: String,
    pub version: String,
    pub audio_filename: String,
    pub hp_drain_rate: f32,
    pub overall_difficulty: f32,
    pub background_file: String,
}

#[derive(Serialize)]
pub struct BeatmapDetail {
    pub id: String,
    pub path: String,
    pub metadata: BeatmapMetadata,
}

#[derive(Serialize, Clone)]
pub struct IndexProgressPayload {
    pub folder: String,
    pub scanned: usize,
}

#[derive(serde::Deserialize)]
pub struct PackBeatmapInput {
    pub path: String,
    pub new_version: String,
    pub hp_drain_rate: f32,
    pub overall_difficulty: f32,
}

#[derive(Serialize)]
pub struct PackCreationResult {
    pub osz_path: String,
    pub folder_path: String,
    pub file_count: usize,
}

#[tauri::command]
pub fn load_cached_index(osu_path: String) -> Result<BeatmapIndexResult, String> {
    let songs_dir = Path::new(&osu_path);
    let index_path = songs_dir.join(INDEX_FILE_NAME);

    if !index_path.exists() {
        return Err("Index file not found".into());
    }

    let content = fs::read_to_string(&index_path).map_err(|e| e.to_string())?;
    let beatmaps: Vec<Beatmap> = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    Ok(BeatmapIndexResult {
        index_path: index_path.to_string_lossy().to_string(),
        total: beatmaps.len(),
        duration_ms: 0,
        beatmaps,
    })
}

#[tauri::command]
pub async fn select_osu_path(app_handle: tauri::AppHandle) -> Option<String> {

    let (tx, rx) = std::sync::mpsc::channel::<Option<String>>();

    app_handle
        .dialog()
        .file()
        .set_title("Select osu! Songs Folder")
        .pick_folder(move |folder| {
            let path = folder.map(|p| p.to_string());
            tx.send(path).ok();
        });

    rx.recv().unwrap_or(None)
}

#[tauri::command]
pub async fn select_output_folder(app_handle: tauri::AppHandle) -> Option<String> {
    let (tx, rx) = std::sync::mpsc::channel::<Option<String>>();

    app_handle
        .dialog()
        .file()
        .set_title("Select output folder")
        .pick_folder(move |folder| {
            let path = folder.map(|p| p.to_string());
            tx.send(path).ok();
        });

    rx.recv().unwrap_or(None)
}

#[tauri::command]
pub fn write_app_log(level: String, message: String) -> Result<(), String> {
    persist_log(&level, &message)
}

#[tauri::command]
pub fn load_osu_files_page(osu_path: String, page: usize, page_size: usize) -> BeatmapPage {
    let mut beatmaps = vec![];
    let songs_dir = Path::new(&osu_path);

    if page_size == 0 {
        return BeatmapPage {
            beatmaps,
            total_dirs: 0,
            total_pages: 0,
        };
    }

    let all_folders = read_song_directories(songs_dir);
    let total = all_folders.len();

    let start = page.saturating_mul(page_size);
    if start >= total {
        return BeatmapPage {
            beatmaps,
            total_dirs: total,
            total_pages: (total + page_size - 1) / page_size,
        };
    }
    let end = (start + page_size).min(total);

    log_debug(&format!(
        "Reading osu folder page={} page_size={} start={} end={} total={}",
        page, page_size, start, end, total
    ));

    for folder in &all_folders[start..end] {
        beatmaps.extend(collect_folder_beatmaps(&folder.path, &folder.name));
    }

    BeatmapPage {
        beatmaps,
        total_dirs: total,
        total_pages: (total + page_size - 1) / page_size,
    }
}



#[tauri::command]
pub fn modify_osu_file(path: String, new_title: String, new_artist: String) -> Result<(), String> {
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let updated = update_osu_file(&content, &new_title, &new_artist);
    fs::write(&path, updated).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn load_beatmap_details(paths: Vec<String>) -> Result<Vec<BeatmapDetail>, String> {
    let mut details = Vec::new();
    for path in paths {
        let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read beatmap {}: {}", path, e))?;
        let metadata = extract_metadata(&content);
        let id = Path::new(&path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        details.push(BeatmapDetail { id, path, metadata });
    }
    Ok(details)
}

#[tauri::command]
pub fn create_pack(
    pack_title: String,
    pack_artist: String,
    pack_creator: String,
    beatmaps: Vec<PackBeatmapInput>,
    output_dir: Option<String>,
    include_extra_files: Option<bool>,
) -> Result<PackCreationResult, String> {
    if pack_title.trim().is_empty() || pack_artist.trim().is_empty() || pack_creator.trim().is_empty() {
        return Err("Pack title, artist, and creator cannot be empty".into());
    }
    if beatmaps.is_empty() {
        return Err("At least one beatmap must be selected".into());
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();
    let pack_folder = std::env::temp_dir().join(format!("osu_pack_{}", timestamp));
    fs::create_dir_all(&pack_folder).map_err(|e| e.to_string())?;

    let mut written_files = 0usize;

    for (index, beatmap) in beatmaps.iter().enumerate() {
        let content = fs::read_to_string(&beatmap.path)
            .map_err(|e| format!("Failed to read beatmap {}: {}", beatmap.path, e))?;
        let metadata = extract_metadata(&content);
        let beatmap_dir = Path::new(&beatmap.path)
            .parent()
            .ok_or_else(|| format!("Unable to locate beatmap directory: {}", beatmap.path))?;

        let osu_name = format!("{}.osu", index + 1);
        let version_value = if beatmap.new_version.trim().is_empty() {
            if metadata.version.is_empty() {
                format!("{} - {}", metadata.artist, metadata.title).trim().to_string()
            } else {
                metadata.version.clone()
            }
        } else {
            beatmap.new_version.trim().to_string()
        };
        let audio_ext = extension_with_dot(&metadata.audio_filename);
        let audio_name = if audio_ext.is_empty() {
            format!("{}{}", index + 1, ".audio")
        } else {
            format!("{}{}", index + 1, audio_ext)
        };
        let bg_ext = extension_with_dot(&metadata.background_file);
        let bg_name = if metadata.background_file.is_empty() {
            String::new()
        } else if bg_ext.is_empty() {
            format!("{}{}", index + 1, ".bg")
        } else {
            format!("{}{}", index + 1, bg_ext)
        };

        let mut updated = content;
        updated = replace_line(&updated, "Title", &pack_title);
        updated = replace_line(&updated, "TitleUnicode", &pack_title);
        updated = replace_line(&updated, "Artist", &pack_artist);
        updated = replace_line(&updated, "ArtistUnicode", &pack_artist);
        updated = replace_line(&updated, "Creator", &pack_creator);
        updated = replace_line(&updated, "Version", &version_value);
        updated = replace_line(&updated, "AudioFilename", &audio_name);
        updated = replace_line(&updated, "HPDrainRate", &format!("{}", beatmap.hp_drain_rate));
        updated = replace_line(
            &updated,
            "OverallDifficulty",
            &format!("{}", beatmap.overall_difficulty),
        );
        updated = replace_line(&updated, "Source", "");
        updated = replace_line(&updated, "Tags", "");
        updated = replace_line(&updated, "BeatmapID", "0");
        updated = replace_line(&updated, "BeatmapSetID", "-1");
        if !bg_name.is_empty() {
            updated = replace_background_reference(&updated, &bg_name);
        }

        let osu_path = pack_folder.join(&osu_name);
        fs::write(&osu_path, updated).map_err(|e| format!("Failed to write file {}: {}", osu_name, e))?;
        written_files += 1;

        if !metadata.audio_filename.is_empty() {
            let src_audio = beatmap_dir.join(&metadata.audio_filename);
            if src_audio.exists() {
                let dst_audio = pack_folder.join(&audio_name);
                fs::copy(&src_audio, &dst_audio)
                    .map_err(|e| format!("Failed to copy audio {} -> {}: {}", src_audio.display(), dst_audio.display(), e))?;
                written_files += 1;
            }
        }

        if !metadata.background_file.is_empty() && !bg_name.is_empty() {
            let src_bg = beatmap_dir.join(&metadata.background_file);
            if src_bg.exists() {
                let dst_bg = pack_folder.join(&bg_name);
                fs::copy(&src_bg, &dst_bg)
                    .map_err(|e| format!("Failed to copy background {} -> {}: {}", src_bg.display(), dst_bg.display(), e))?;
                written_files += 1;
            }
        }
    }

    let sanitized_title = sanitize_file_name(&pack_title);
    let resolved_parent = output_dir
        .and_then(|dir| {
            let path = PathBuf::from(dir);
            if path.exists() || fs::create_dir_all(&path).is_ok() {
                Some(path)
            } else {
                None
            }
        })
        .or_else(|| pack_folder.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::temp_dir());
    let mut osz_file_name = format!("{}.osz", sanitized_title);
    let mut osz_path = resolved_parent.join(&osz_file_name);
    if osz_path.exists() {
        osz_file_name = format!("{}-{}.osz", sanitized_title, timestamp);
        osz_path = resolved_parent.join(&osz_file_name);
    }
    if include_extra_files.unwrap_or(false) {
        written_files += add_delete_files(&pack_folder, &pack_title, &pack_artist, &pack_creator)?;
    }

    create_osz_from_folder(&pack_folder, &osz_path)?;

    Ok(PackCreationResult {
        osz_path: osz_path.to_string_lossy().to_string(),
        folder_path: pack_folder.to_string_lossy().to_string(),
        file_count: written_files,
    })
}

fn update_osu_file(content: &str, title: &str, artist: &str) -> String {
    let mut updated = content.to_string();

    updated = Regex::new(r"(?m)^Title:[^\n]+")
        .unwrap()
        .replace(&updated, format!("Title:{}", title))
        .to_string();

    updated = Regex::new(r"(?m)^Artist:[^\n]+")
        .unwrap()
        .replace(&updated, format!("Artist:{}", artist))
        .to_string();

    updated
}

fn extract_metadata(content: &str) -> BeatmapMetadata {
    let mut metadata = BeatmapMetadata::default();
    let mut current_section = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed.trim_matches(&['[', ']'][..]).to_string();
            continue;
        }

        if let Some((key, value)) = trimmed.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            match (current_section.as_str(), key) {
                ("General", "AudioFilename") => metadata.audio_filename = value.to_string(),
                ("Metadata", "Title") => metadata.title = value.to_string(),
                ("Metadata", "Artist") => metadata.artist = value.to_string(),
                ("Metadata", "Creator") => metadata.creator = value.to_string(),
                ("Metadata", "Version") => metadata.version = value.to_string(),
                ("Difficulty", "HPDrainRate") => {
                    metadata.hp_drain_rate = value.parse().unwrap_or(metadata.hp_drain_rate)
                }
                ("Difficulty", "OverallDifficulty") => {
                    metadata.overall_difficulty = value.parse().unwrap_or(metadata.overall_difficulty)
                }
                _ => {}
            }
        } else if current_section == "Events" && metadata.background_file.is_empty() {
            if let Some(start) = trimmed.find('"') {
                if let Some(end_rel) = trimmed[start + 1..].find('"') {
                    let end = start + 1 + end_rel;
                    metadata.background_file = trimmed[start + 1..end].to_string();
                }
            }
        }
    }

    metadata
}

fn replace_line(content: &str, key: &str, value: &str) -> String {
    let pattern = Regex::new(&format!(r"(?m)^{}:[^\n]*", regex::escape(key))).unwrap();
    if pattern.is_match(content) {
        pattern
            .replace(content, format!("{}:{}", key, value))
            .to_string()
    } else {
        let mut result = content.to_string();
        if !result.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(&format!("{}:{}\n", key, value));
        result
    }
}

fn replace_background_reference(content: &str, new_bg: &str) -> String {
    if new_bg.is_empty() {
        return content.to_string();
    }

    let mut result = String::with_capacity(content.len());
    let mut in_events = false;
    let mut replaced = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_events = trimmed == "[Events]";
        }

        if in_events && !replaced && trimmed.contains('"') && !trimmed.starts_with("//") {
            if let Some(start) = line.find('"') {
                if let Some(end_rel) = line[start + 1..].find('"') {
                    let end = start + 1 + end_rel;
                    let mut new_line = line.to_string();
                    new_line.replace_range(start + 1..end, new_bg);
                    result.push_str(&new_line);
                    result.push('\n');
                    replaced = true;
                    continue;
                }
            }
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

fn sanitize_file_name(input: &str) -> String {
    let invalid = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut sanitized: String = input
        .chars()
        .map(|c| if c.is_control() || invalid.contains(&c) { '_' } else { c })
        .collect();
    sanitized = sanitized.trim_matches([' ', '.']).to_string();
    if sanitized.is_empty() {
        sanitized = "osu_pack".to_string();
    }
    sanitized
}

fn add_delete_files(
    pack_folder: &Path,
    pack_title: &str,
    pack_artist: &str,
    pack_creator: &str,
) -> Result<usize, String> {
    let mut updated = DELETE_OSU_TEMPLATE.to_string();
    updated = replace_line(&updated, "Title", pack_title);
    updated = replace_line(&updated, "TitleUnicode", pack_title);
    updated = replace_line(&updated, "Artist", pack_artist);
    updated = replace_line(&updated, "ArtistUnicode", pack_artist);
    updated = replace_line(&updated, "Creator", pack_creator);

    let delete_osu_path = pack_folder.join("delete.osu");
    fs::write(&delete_osu_path, updated).map_err(|e| format!("Failed to write delete.osu: {}", e))?;

    let delete_bg_path = pack_folder.join("delete.jpg");
    fs::write(&delete_bg_path, DELETE_BG_BYTES)
        .map_err(|e| format!("Failed to write delete.jpg: {}", e))?;

    let delete_audio_path = pack_folder.join("delete.mp3");
    fs::write(&delete_audio_path, DELETE_AUDIO_BYTES)
        .map_err(|e| format!("Failed to write delete.mp3: {}", e))?;

    Ok(3)
}

fn extension_with_dot(file_name: &str) -> String {
    if file_name.trim().is_empty() {
        return String::new();
    }
    Path::new(file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!(".{}", ext))
        .unwrap_or_default()
}

fn create_osz_from_folder(folder: &Path, osz_path: &Path) -> Result<(), String> {
    let file = fs::File::create(osz_path).map_err(|e| format!("Unable to create package file: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in fs::read_dir(folder).map_err(|e| format!("Failed to read folder: {}", e))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            let name = entry
                .file_name()
                .to_string_lossy()
                .to_string();
            zip.start_file(name, options)
                .map_err(|e| format!("Failed to write zip entry: {}", e))?;
            let mut src = fs::File::open(&path).map_err(|e| e.to_string())?;
            std::io::copy(&mut src, &mut zip).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn collect_folder_beatmaps(folder: &Path, folder_name: &str) -> Vec<Beatmap> {
    let mut beatmaps = Vec::new();

    let inner = match fs::read_dir(folder) {
        Ok(it) => it,
        Err(e) => {
            log_error(&format!("Failed to read folder {}: {}", folder.display(), e));
            return beatmaps;
        }
    };

    for file_entry in inner.filter_map(Result::ok) {
        let file_path = file_entry.path();
        let is_osu = file_path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.eq_ignore_ascii_case("osu"))
            .unwrap_or(false);

        if !is_osu {
            continue;
        }

        if let Some(file_stem) = file_path.file_stem().and_then(|s| s.to_str()) {
            beatmaps.push(Beatmap {
                id: file_stem.to_string(),
                path: file_path.to_string_lossy().to_string(),
            });
        } else {
            log_debug(&format!(
                "Skipped file in folder {} because file_stem is missing",
                folder_name
            ));
        }
    }

    beatmaps
}

struct FolderEntry {
    path: PathBuf,
    name: String,
}

fn read_song_directories(songs_dir: &Path) -> Vec<FolderEntry> {
    let mut entries: Vec<_> = match fs::read_dir(songs_dir) {
        Ok(e) => e.filter_map(Result::ok).collect(),
        Err(e) => {
            log_error(&format!("Failed to read songs dir {}: {}", songs_dir.display(), e));
            return vec![];
        }
    };

    entries.retain(|entry| entry.path().is_dir());
    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    entries.into_iter().filter_map(|entry| {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        Some(FolderEntry { path, name })
    }).collect()
}

fn collect_all_beatmaps(songs_dir: &Path) -> Vec<Beatmap> {
    let folders = read_song_directories(songs_dir);
    let mut all = Vec::new();
    for folder in folders {
        all.extend(collect_folder_beatmaps(&folder.path, &folder.name));
    }
    all
}

fn emit_progress(window: &Window, folder: &str, scanned: usize) {
    if let Err(err) = window.emit(
        "index-progress",
        IndexProgressPayload {
            folder: folder.to_string(),
            scanned,
        },
    ) {
        log_error(&format!("Failed to emit progress event: {}", err));
    }
}

#[tauri::command]
pub fn search_beatmaps_by_id(osu_path: String, keyword: String) -> Vec<Beatmap> {
    let trimmed = keyword.trim().to_lowercase();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let songs_dir = Path::new(&osu_path);
    collect_all_beatmaps(songs_dir)
        .into_iter()
        .filter(|beatmap| beatmap.id.to_lowercase().contains(&trimmed))
        .collect()
}

#[tauri::command]
pub fn build_beatmap_index(window: Window, osu_path: String) -> Result<BeatmapIndexResult, String> {
    let songs_dir = Path::new(&osu_path);
    if !songs_dir.exists() {
        return Err("Provided path does not exist".into());
    }

    let started = Instant::now();
    let folders = read_song_directories(songs_dir);
    let mut beatmaps = Vec::new();
    let mut scanned = 0usize;

    for folder in folders {
        emit_progress(&window, &folder.name, scanned);
        let folder_maps = collect_folder_beatmaps(&folder.path, &folder.name);
        scanned += folder_maps.len();
        beatmaps.extend(folder_maps);
        emit_progress(&window, &folder.name, scanned);
    }

    emit_progress(&window, "Finished", scanned);
    let index_path = songs_dir.join(INDEX_FILE_NAME);

    let json = serde_json::to_string_pretty(&beatmaps).map_err(|e| e.to_string())?;
    fs::write(&index_path, json).map_err(|e| e.to_string())?;

    let duration_ms = started.elapsed().as_millis();
    Ok(BeatmapIndexResult {
        index_path: index_path.to_string_lossy().to_string(),
        total: beatmaps.len(),
        duration_ms,
        beatmaps,
    })
}
