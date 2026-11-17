use crate::error::Result;
use crate::library::models::{Game, PlatformStatus};
use crate::platforms;

#[derive(serde::Serialize)]
pub struct AppInfo {
    name: &'static str,
    version: &'static str,
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo { name: "Saha", version: env!("CARGO_PKG_VERSION") }
}

#[tauri::command]
pub fn list_platforms() -> Vec<PlatformStatus> {
    platforms::all().iter().map(|p| p.status()).collect()
}

#[tauri::command]
pub fn scan_library() -> Result<Vec<Game>> {
    let mut games = Vec::new();
    for p in platforms::all() {
        if p.status().detected {
            games.extend(p.scan_games()?);
        }
    }
    Ok(games)
}
