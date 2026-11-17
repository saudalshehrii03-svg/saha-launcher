//! User settings, stored as JSON in the app config dir. Mirrored in `src/lib/types.ts`.
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Theme {
    pub accent: String,
    pub accent2: String,
}

impl Default for Theme {
    fn default() -> Self {
        Theme { accent: "#ff2a4d".into(), accent2: "#8a0f24".into() }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Profile {
    pub name: String,
    pub bio: String,
    /// Paths of saved images (GIF/WebP animations allowed) in the media dir.
    pub avatar: String,
    pub header: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Game key -> account id that must be used for it ("forced account per game").
    pub pinned_accounts: HashMap<String, String>,
    /// Steam accounts banned in Call of Duty. CoD never launches on these.
    pub cod_banned_accounts: Vec<String>,
    pub favorites: Vec<String>,
    /// Games hidden from the library (tools, demos...).
    pub hidden: Vec<String>,
    /// Minimize Saha while a game runs, bring it back when the game closes.
    pub minimize_on_play: bool,
    pub theme: Theme,
    /// Optional SteamGridDB API key for covers of games not on Steam.
    pub steamgriddb_key: String,
    /// Steam Web API key: lists every owned game (not only installed ones).
    pub steam_api_key: String,
    /// Nexus Mods personal API key (installing mods).
    pub nexus_key: String,
    /// "glass" (glows) or "amoled" (pure black).
    pub mode: String,
    pub profile: Profile,
    /// Game key -> custom poster paths; several = slideshow.
    pub custom_covers: HashMap<String, Vec<String>>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            pinned_accounts: HashMap::new(),
            cod_banned_accounts: Vec::new(),
            favorites: Vec::new(),
            hidden: Vec::new(),
            minimize_on_play: true,
            theme: Theme::default(),
            steamgriddb_key: String::new(),
            steam_api_key: String::new(),
            nexus_key: String::new(),
            mode: "glass".into(),
            profile: Profile::default(),
            custom_covers: HashMap::new(),
        }
    }
}

static DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn init(dir: PathBuf) {
    let _ = DIR.set(dir);
}

/// App config dir (settings.json, playtime.json, art.json).
pub fn dir() -> Option<&'static PathBuf> {
    DIR.get()
}

pub fn read_json<T: serde::de::DeserializeOwned + Default>(name: &str) -> T {
    dir()
        .and_then(|d| std::fs::read_to_string(d.join(name)).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn write_json<T: Serialize>(name: &str, value: &T) -> Result<()> {
    if let Some(d) = dir() {
        std::fs::create_dir_all(d)?;
        std::fs::write(d.join(name), serde_json::to_string_pretty(value)?)?;
    }
    Ok(())
}

pub fn load() -> Settings {
    read_json("settings.json")
}

pub fn save(s: &Settings) -> Result<()> {
    write_json("settings.json", s)
}
