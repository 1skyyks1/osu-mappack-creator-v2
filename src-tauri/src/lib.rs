mod commands;
use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            select_osu_path,
            select_output_folder,
            load_osu_files_page,
            modify_osu_file,
            search_beatmaps_by_id,
            build_beatmap_index,
            load_cached_index,
            load_beatmap_details,
            create_pack,
            write_app_log
        ])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
