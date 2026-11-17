//! Saha launcher backend.
//!
//! Modules:
//! - `library`   shared models (Game, Account, Platform) and the merged library
//! - `platforms` one scanner/launcher per store (Steam, Epic, Rockstar, GOG, EA, Ubisoft, Battle.net, Xbox)
//! - `accounts`  account rules + automatic Steam account switching
//! - `mods`      Nexus browse, one-click install, enable/disable, clean delete
//! - `online`    owned Steam games, game page data, store browsing
//! - `settings`  user settings (accounts, favorites, theme, profile, keys)
//! - `tracker`   Saha's own playtime tracking + minimize while playing
//! - `art`       covers for non-Steam games (Steam store search, SteamGridDB)
//! - `commands`  Tauri commands exposed to the frontend

mod accounts;
mod ai;
mod art;
mod commands;
mod error;
mod library;
mod mods;
mod online;
mod platforms;
mod settings;
mod storage;
mod sys;
mod tracker;
mod vdf;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        // Must be first: a second launch (e.g. an nxm:// link) is forwarded to the running app.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if let Ok(dir) = app.path().app_config_dir() {
                settings::init(dir);
            }
            // Installed builds register nxm:// via the installer; dev builds register here.
            #[cfg(all(windows, debug_assertions))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let _ = app.deep_link().register_all();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::scan_library,
            commands::fetch_art,
            commands::sync_owned,
            commands::game_details,
            commands::store_featured,
            commands::store_search,
            commands::launch_game,
            commands::get_settings,
            commands::save_settings,
            commands::save_media,
            commands::mod_support,
            commands::mods_browse,
            commands::mod_page,
            commands::mods_installed,
            commands::mod_install,
            commands::mod_set_enabled,
            commands::mod_remove,
            commands::open_external,
            commands::storage_report,
            commands::mod_explain,
            commands::mods_external,
            commands::mod_external_toggle,
            commands::mods_counts,
            commands::game_text,
            commands::ai_test_key,
            commands::open_nexus,
            commands::open_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Saha");
}
