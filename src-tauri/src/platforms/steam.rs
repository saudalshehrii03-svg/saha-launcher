//! Steam.
//! Plan (phase 1):
//! - install path: registry HKCU\Software\Valve\Steam\SteamPath
//! - accounts: config/loginusers.vdf (SteamID64, AccountName, PersonaName)
//! - installed games: steamapps/libraryfolders.vdf + appmanifest_*.acf per library
//! - owned games per account: Steam Web API IPlayerService/GetOwnedGames
//! - launch: if owner != active account, accounts::steam_switch first, then steam://rungameid/<appid>
use crate::error::{Error, Result};
use crate::library::models::{Account, Game, PlatformId, PlatformStatus};

use super::Platform;

pub struct Steam;

impl Platform for Steam {
    fn status(&self) -> PlatformStatus {
        PlatformStatus {
            id: PlatformId::Steam,
            name: "Steam".into(),
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
        Err(Error::NotImplemented("steam launch"))
    }
}
