//! GOG.
//! - installed games: HKLM\SOFTWARE\WOW6432Node\GOG.com\Games\<id> (gameName, path, exe, workingDir)
//! - launch: GOG Galaxy (`/command=runGame`) when installed, otherwise the game exe directly (DRM-free)
use std::path::PathBuf;

use crate::error::{msg, Result};
use crate::library::models::{Game, PlatformId};
use crate::sys::{self, Hive};

use super::{launch_in_background, Platform, Progress};

const REG: &str = r"SOFTWARE\WOW6432Node\GOG.com";

pub struct Gog;

fn galaxy_exe() -> Option<PathBuf> {
    sys::reg_str(Hive::LocalMachine, &format!(r"{REG}\GalaxyClient\paths"), "client")
        .map(|d| PathBuf::from(d).join("GalaxyClient.exe"))
        .filter(|p| p.is_file())
}

fn game_reg(id: &str, name: &str) -> Option<String> {
    sys::reg_str(Hive::LocalMachine, &format!(r"{REG}\Games\{id}"), name)
}

impl Platform for Gog {
    fn id(&self) -> PlatformId {
        PlatformId::Gog
    }
    fn name(&self) -> &'static str {
        "GOG"
    }
    fn install_path(&self) -> Option<String> {
        galaxy_exe()
            .and_then(|p| p.parent().map(|d| d.to_string_lossy().to_string()))
            .or_else(|| (!sys::reg_subkeys(Hive::LocalMachine, &format!(r"{REG}\Games")).is_empty()).then(|| "GOG.com".into()))
    }
    fn scan_games(&self) -> Result<Vec<Game>> {
        let mut games = Vec::new();
        for id in sys::reg_subkeys(Hive::LocalMachine, &format!(r"{REG}\Games")) {
            let (Some(name), Some(path)) = (game_reg(&id, "gameName"), game_reg(&id, "path")) else { continue };
            if !PathBuf::from(&path).is_dir() {
                continue;
            }
            games.push(Game::installed(PlatformId::Gog, &id, &name, &id, Some(path)));
        }
        Ok(games)
    }
    fn launch(&self, game: &Game, progress: Progress) -> Result<()> {
        let dir = game.install_dir.clone().unwrap_or_default();
        if let Some(galaxy) = galaxy_exe() {
            progress("تشغيل اللعبة عبر GOG Galaxy في الخلفية…");
            let id = format!("/gameId={}", game.app_id);
            let path = format!("/path={dir}");
            let start = || -> Result<()> {
                sys::spawn(&galaxy, &["/command=runGame", &id, &path])?;
                Ok(())
            };
            return launch_in_background(game, &["galaxyclient.exe", "galaxyclient helper.exe"], false, progress, &start);
        }
        let Some(exe) = game_reg(&game.app_id, "exe").map(PathBuf::from).filter(|p| p.is_file()) else {
            return msg("ما لقيت ملف تشغيل اللعبة.");
        };
        progress("تشغيل اللعبة…");
        sys::spawn(&exe, &[])?;
        Ok(())
    }
}
