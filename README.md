<div align="center">
<h2>osu-mappack-creator-v2</h2>

A Tauri + Vue 3 desktop app for creating osu! map packs.

**[English](README.md)**
·
**[简体中文](README_zh.md)**
</div>

## Key Features
- Select your osu! Songs directory, get beatmaps list with pagination & search.
- Multi-select beatmaps, edit pack title/artist/creator, and edit HP/OD for each beatmap.
- Automatically loads beatmap metadata and defaults new Version to `Artist - Title [Creator] (Version)`.
- One-click pack creation with optional `delete this` files.

## How to use
1. Launch the app and click **Select Folder** to point at your osu! `Songs` directory.
2. If beatmaps in your Songs directory changed, please click **Refresh**.
3. Select beatmaps, click **Next**, and adjust Version/HP/OD as needed.
4. Fill Pack Title / Artist / Creator (required), choose an output folder, decide whether to include delete files.
5. Click **Create Pack** and get `.osz` pack in output folder.

## Logging
All runtime errors and important events are written to `%TEMP%\osu-mappack-creator-v2.log`.

## Getting Started & Build
```bash
npm install
npx tauri dev
npx tauri build
```
