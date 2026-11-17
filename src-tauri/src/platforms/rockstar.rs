//! Rockstar Games Launcher.
//! - installed games: HKLM\SOFTWARE\WOW6432Node\Rockstar Games\<title>\InstallFolder
//!   (skipped: keys with InstallFolderSteam/Epic = owned there, or folders without an .exe)
//! - launch: <Launcher>\Launcher.exe -launchTitleInFolder "<InstallFolder>"
use std::path::{Path, PathBuf};

use crate::error::{msg, Result};
use crate::library::models::{Game, PlatformId};
use crate::sys::{self, Hive};

use super::{Platform, Progress};

const REG: &str = r"SOFTWARE\WOW6432Node\Rockstar Games";
const NOT_GAMES: &[&str] = &["Launcher", "Rockstar Games Social Club", "Steam"];

pub struct Rockstar;

fn launcher_dir() -> Option<PathBuf> {
    sys::reg_str(Hive::LocalMachine, &format!(r"{REG}\Launcher"), "InstallFolder")
        .map(PathBuf::from)
        .filter(|p| p.join("Launcher.exe").is_file())
}

fn has_exe(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .map(|d| d.flatten().any(|e| e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("exe"))))
        .unwrap_or(false)
}

fn pretty_title(key: &str) -> String {
    match key {
        "GTAV Enhanced" | "GTA V Enhanced" => "Grand Theft Auto V Enhanced".into(),
        "GTAV" | "Grand Theft Auto V" => "Grand Theft Auto V".into(),
        "GTA IV" => "Grand Theft Auto IV".into(),
        _ => key.into(),
    }
}

impl Platform for Rockstar {
    fn id(&self) -> PlatformId {
        PlatformId::Rockstar
    }
    fn name(&self) -> &'static str {
        "Rockstar Games"
    }
    fn install_path(&self) -> Option<String> {
        launcher_dir().map(|p| p.to_string_lossy().to_string())
    }
    fn scan_games(&self) -> Result<Vec<Game>> {
        let mut games = Vec::new();
        for key in sys::reg_subkeys(Hive::LocalMachine, REG) {
            if NOT_GAMES.contains(&key.as_str()) {
                continue;
            }
            let Some(folder) = sys::reg_str(Hive::LocalMachine, &format!(r"{REG}\{key}"), "InstallFolder") else {
                continue;
            };
            let dir = PathBuf::from(&folder);
            let lower = folder.to_lowercase();
            // Copies owned on Steam/Epic are listed by those platforms instead.
            let elsewhere = ["InstallFolderSteam", "InstallFolderEpic"]
                .iter()
                .any(|v| sys::reg_str(Hive::LocalMachine, &format!(r"{REG}\{key}"), v).is_some());
            if elsewhere || !has_exe(&dir) || lower.contains("steamapps") || lower.contains("epic games") {
                continue;
            }
            games.push(Game {
                key: format!("rockstar:{key}"),
                title: pretty_title(&key),
                platform: PlatformId::Rockstar,
                app_id: key.clone(),
                owner_account_id: None,
                launch_account_id: None,
                guarded: false,
                installed: true,
                install_dir: Some(folder),
                size_bytes: None,
                playtime_minutes: None,
                last_played: None,
            });
        }
        Ok(games)
    }
    fn launch(&self, game: &Game, progress: Progress) -> Result<()> {
        let Some(dir) = game.install_dir.as_deref() else { return msg("مسار اللعبة غير معروف.") };
        let Some(launcher) = launcher_dir() else { return msg("Rockstar Games Launcher غير مثبت.") };
        progress("تشغيل اللعبة عبر Rockstar…");
        sys::spawn(&launcher.join("Launcher.exe"), &["-launchTitleInFolder", dir])?;
        Ok(())
    }
}
