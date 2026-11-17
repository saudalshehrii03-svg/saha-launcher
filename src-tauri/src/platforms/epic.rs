//! Epic Games.
//! - installed games: %ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\*.item (JSON)
//! - launch: com.epicgames.launcher://apps/<ns>%3A<itemId>%3A<AppName>?action=launch&silent=true
use std::path::PathBuf;

use serde::Deserialize;

use crate::error::Result;
use crate::library::models::{Game, PlatformId};
use crate::sys;

use super::{Platform, Progress};

pub struct Epic;

fn manifests_dir() -> PathBuf {
    let pd = std::env::var("ProgramData").unwrap_or_else(|_| r"C:\ProgramData".into());
    PathBuf::from(pd).join(r"Epic\EpicGamesLauncher\Data\Manifests")
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Item {
    display_name: String,
    app_name: String,
    catalog_namespace: String,
    catalog_item_id: String,
    install_location: String,
    #[serde(default)]
    install_size: Option<u64>,
    #[serde(default)]
    main_game_app_name: String,
    #[serde(default)]
    app_categories: Vec<String>,
    #[serde(default, rename = "bIsIncompleteInstall")]
    incomplete: bool,
}

impl Platform for Epic {
    fn id(&self) -> PlatformId {
        PlatformId::Epic
    }
    fn name(&self) -> &'static str {
        "Epic Games"
    }
    fn install_path(&self) -> Option<String> {
        let d = manifests_dir();
        d.is_dir().then(|| d.to_string_lossy().to_string())
    }
    fn scan_games(&self) -> Result<Vec<Game>> {
        let Ok(dir) = std::fs::read_dir(manifests_dir()) else { return Ok(Vec::new()) };
        let mut games = Vec::new();
        for e in dir.flatten() {
            if e.path().extension().is_none_or(|x| x != "item") {
                continue;
            }
            let Ok(s) = std::fs::read_to_string(e.path()) else { continue };
            let Ok(it) = serde_json::from_str::<Item>(&s) else { continue };
            let is_dlc = !it.main_game_app_name.is_empty() && it.main_game_app_name != it.app_name;
            if is_dlc || !it.app_categories.iter().any(|c| c == "games") {
                continue;
            }
            games.push(Game {
                key: format!("epic:{}", it.app_name),
                title: it.display_name,
                platform: PlatformId::Epic,
                // Launch id: namespace:itemId:appName
                app_id: format!("{}:{}:{}", it.catalog_namespace, it.catalog_item_id, it.app_name),
                owner_account_id: None,
                launch_account_id: None,
                guarded: false,
                installed: !it.incomplete && PathBuf::from(&it.install_location).is_dir(),
                install_dir: Some(it.install_location),
                size_bytes: it.install_size,
                playtime_minutes: None,
                last_played: None,
                store_id: None,
            });
        }
        Ok(games)
    }
    fn launch(&self, game: &Game, progress: Progress) -> Result<()> {
        progress("تشغيل اللعبة عبر Epic…");
        let id = game.app_id.replace(':', "%3A");
        sys::open_url(&format!("com.epicgames.launcher://apps/{id}?action=launch&silent=true"))?;
        Ok(())
    }
}
