// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

mod commands;
mod state;

use commands::sync::*;
use commands::translate::*;
use state::{AppSyncState, AppTranslateState, SyncState, TranslateState};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .manage(Mutex::new(SyncState::default()) as AppSyncState)
        .manage(Mutex::new(TranslateState::default()) as AppTranslateState)
        .invoke_handler(tauri::generate_handler![
            // Comandi traduzione
            set_api_config,
            load_srt_for_translate,
            start_translation,
            cancel_translation,
            // Comandi sincronizzazione
            sync_load_srt,
            sync_set_video,
            sync_get_status,
            sync_get_subtitles,
            sync_get_subtitles_range,
            sync_get_subtitle,
            sync_find_subtitle_at_time,
            sync_find_nearest_subtitle,
            sync_add_anchor,
            sync_remove_anchor,
            sync_get_anchors,
            sync_suggest_next,
            sync_set_strategy,
            sync_save_file,
            sync_save_session,
            sync_load_session,
            sync_reset,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
