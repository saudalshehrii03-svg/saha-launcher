//! Saha's own play tracking: works for every platform (Epic, Rockstar, Battle.net...
//! don't expose playtime locally). After a launch it waits for the game process,
//! minimizes Saha while it runs, then records the session and brings Saha back.
//!
//! "Running" = Steam's RunningAppID for Steam games, otherwise any process whose
//! exe lives inside the game's install folder.
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::library::models::{Game, PlatformId};
use crate::platforms::steam;
use crate::settings;
use crate::sys::{self, Hive};

const FILE: &str = "playtime.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub minutes: u64,
    pub last_played: u64,
}

pub fn load() -> HashMap<String, Entry> {
    settings::read_json(FILE)
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn record(key: &str, minutes: u64) {
    let mut all = load();
    let e = all.entry(key.to_string()).or_default();
    e.minutes += minutes;
    e.last_played = now();
    let _ = settings::write_json(FILE, &all);
}

pub fn is_running(game: &Game) -> bool {
    if game.platform == PlatformId::Steam {
        let want: u32 = game.app_id.parse().unwrap_or(0);
        return sys::reg_u32(Hive::CurrentUser, steam::REG, "RunningAppID") == Some(want);
    }
    let Some(dir) = game.install_dir.as_deref().filter(|d| d.len() > 3) else { return false };
    let dir = format!("{}\\", dir.to_lowercase().replace('/', "\\").trim_end_matches('\\'));
    sys::running_exe_paths().iter().any(|p| p.starts_with(&dir))
}

#[derive(Clone, Serialize)]
struct GameState {
    key: String,
    running: bool,
}

/// Track one launched game in the background.
pub fn watch(app: AppHandle, game: Game) {
    std::thread::spawn(move || {
        let wait = Instant::now();
        while !is_running(&game) {
            if wait.elapsed() > Duration::from_secs(180) {
                return; // never saw it start
            }
            std::thread::sleep(Duration::from_secs(2));
        }
        let start = Instant::now();
        let _ = app.emit("game-state", GameState { key: game.key.clone(), running: true });
        let window = app.get_webview_window("main");
        let minimized = settings::load().minimize_on_play;
        if minimized {
            if let Some(w) = &window {
                let _ = w.minimize();
            }
        }

        // Two misses in a row = closed (launchers hop between processes).
        let mut misses = 0;
        while misses < 2 {
            std::thread::sleep(Duration::from_secs(5));
            misses = if is_running(&game) { 0 } else { misses + 1 };
        }

        let secs = start.elapsed().as_secs().saturating_sub(10);
        record(&game.key, (secs + 30) / 60);
        let _ = app.emit("game-state", GameState { key: game.key.clone(), running: false });
        if minimized {
            if let Some(w) = &window {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }
    });
}
