//! Ubisoft Connect.
//! - installed games: HKLM\SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs\<id>\InstallDir
//!   title from the "Uplay Install <id>" uninstall entry, else the folder name
//! - launch: uplay://launch/<id>/0
use std::path::PathBuf;

use crate::error::Result;
use crate::library::models::{Game, PlatformId};
use crate::sys::{self, Hive};

use super::{launch_in_background, Platform, Progress};

const REG: &str = r"SOFTWARE\WOW6432Node\Ubisoft\Launcher";

pub struct Ubisoft;

impl Platform for Ubisoft {
    fn id(&self) -> PlatformId {
        PlatformId::Ubisoft
    }
    fn name(&self) -> &'static str {
        "Ubisoft Connect"
    }
    fn install_path(&self) -> Option<String> {
        sys::reg_str(Hive::LocalMachine, REG, "InstallDir").filter(|d| PathBuf::from(d).is_dir())
    }
    fn scan_games(&self) -> Result<Vec<Game>> {
        let titles: Vec<(String, String)> = sys::uninstall_entries()
            .into_iter()
            .filter(|u| u.publisher.contains("Ubisoft"))
            .map(|u| (u.location.to_lowercase().trim_end_matches(['\\', '/']).to_string(), u.name))
            .collect();
        let mut games = Vec::new();
        for id in sys::reg_subkeys(Hive::LocalMachine, &format!(r"{REG}\Installs")) {
            let Some(dir) = sys::reg_str(Hive::LocalMachine, &format!(r"{REG}\Installs\{id}"), "InstallDir") else {
                continue;
            };
            let path = PathBuf::from(&dir);
            if !path.is_dir() {
                continue;
            }
            let norm = dir.to_lowercase().replace('/', "\\").trim_end_matches('\\').to_string();
            let title = titles
                .iter()
                .find(|(loc, _)| *loc == norm)
                .map(|(_, n)| n.clone())
                .or_else(|| path.file_name().map(|f| f.to_string_lossy().to_string()))
                .unwrap_or_else(|| id.clone());
            games.push(Game::installed(PlatformId::Ubisoft, &id, &title, &id, Some(dir)));
        }
        Ok(games)
    }
    fn launch(&self, game: &Game, progress: Progress) -> Result<()> {
        progress("تشغيل اللعبة عبر Ubisoft Connect في الخلفية…");
        let url = format!("uplay://launch/{}/0", game.app_id);
        let start = || -> Result<()> {
            sys::open_url(&url)?;
            Ok(())
        };
        launch_in_background(game, &["upc.exe", "ubisoftconnect.exe", "uplaywebcore.exe"], true, progress, &start)
    }
}
