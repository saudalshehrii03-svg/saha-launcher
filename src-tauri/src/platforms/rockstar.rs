//! Rockstar Games Launcher.
//! Plan (phase 1):
//! - installed games: registry HKLM\SOFTWARE\WOW6432Node\Rockstar Games\<title> (InstallFolder)
//! - launch: through the Rockstar launcher (or the Steam/Epic copy when the game is owned there)
use crate::error::{Error, Result};
use crate::library::models::{Account, Game, PlatformId, PlatformStatus};

use super::Platform;

pub struct Rockstar;

impl Platform for Rockstar {
    fn status(&self) -> PlatformStatus {
        PlatformStatus {
            id: PlatformId::Rockstar,
            name: "Rockstar Games".into(),
            detected: false,
            install_path: None,
            game_count: 0,
        }
    }
    fn accounts(&self) -> Result<Vec<Account>> {
        Ok(Vec::new())
    }
    fn scan_games(&self) -> Result<Vec<Game>> {
        Ok(Vec::new())
    }
    fn launch(&self, _game: &Game) -> Result<()> {
        Err(Error::NotImplemented("rockstar launch"))
    }
}
