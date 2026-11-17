//! Saha launcher backend.
//!
//! Modules:
//! - `library`   shared models (Game, Account, Platform) and the merged library
//! - `platforms` one scanner/launcher per store (Steam, Epic, Rockstar, ...)
//! - `accounts`  multi-account handling (automatic Steam account switching)
//! - `mods`      mod sources (mod.io, Thunderstore, Nexus) and per-game install rules
//! - `commands`  Tauri commands exposed to the frontend

mod accounts;
mod commands;
mod error;
mod library;
mod mods;
mod platforms;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::list_platforms,
            commands::scan_library,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Saha");
}
