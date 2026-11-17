//! Epic Games.
//! Plan (phase 1):
//! - installed games: %ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\*.item (JSON)
//! - launch: com.epicgames.launcher://apps/<CatalogNamespace>%3A<CatalogItemId>%3A<AppName>?action=launch&silent=true
use crate::error::{Error, Result};
use crate::library::models::{Account, Game, PlatformId, PlatformStatus};

use super::Platform;

pub struct Epic;

impl Platform for Epic {
    fn status(&self) -> PlatformStatus {
        PlatformStatus {
            id: PlatformId::Epic,
            name: "Epic Games".into(),
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
        Err(Error::NotImplemented("epic launch"))
    }
}
