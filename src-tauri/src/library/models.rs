//! Shared data models. Mirrored in `src/lib/types.ts`.
use serde::{Deserialize, Serialize};

#[allow(dead_code)] // phase 2 platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlatformId {
    Steam,
    Epic,
    Rockstar,
    Gog,
    Ea,
    Ubisoft,
    Battlenet,
    Xbox,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub platform: PlatformId,
    /// Platform-side id, e.g. SteamID64.
    pub id: String,
    pub display_name: String,
    /// Login name used by the switcher. Passwords are never stored.
    pub login_name: Option<String>,
    /// Currently signed in on the platform client.
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    /// Stable key, e.g. "steam:1938090".
    pub key: String,
    pub title: String,
    pub platform: PlatformId,
    pub app_id: String,
    /// Account that owns the game (from local files).
    pub owner_account_id: Option<String>,
    /// Account the launcher will switch to before launching (after rules).
    pub launch_account_id: Option<String>,
    /// Protected by a safety rule (Call of Duty). Launch is refused until it resolves.
    pub guarded: bool,
    pub installed: bool,
    pub install_dir: Option<String>,
    pub size_bytes: Option<u64>,
    pub playtime_minutes: Option<u64>,
    /// Unix seconds.
    pub last_played: Option<u64>,
    /// Microsoft Store product id (Xbox games), used for official art.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store_id: Option<String>,
}

impl PlatformId {
    /// Key prefix, e.g. "battlenet" in "battlenet:auks".
    pub fn slug(self) -> String {
        serde_json::to_value(self).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default()
    }
}

impl Game {
    /// An installed game with no account/playtime info (most launchers).
    pub fn installed(platform: PlatformId, id: &str, title: &str, app_id: &str, dir: Option<String>) -> Game {
        Game {
            key: format!("{}:{id}", platform.slug()),
            title: title.to_string(),
            platform,
            app_id: app_id.to_string(),
            owner_account_id: None,
            launch_account_id: None,
            guarded: false,
            installed: true,
            install_dir: dir,
            size_bytes: None,
            playtime_minutes: None,
            last_played: None,
            store_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformStatus {
    pub id: PlatformId,
    pub name: String,
    pub detected: bool,
    pub install_path: Option<String>,
    pub game_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub platforms: Vec<PlatformStatus>,
    pub accounts: Vec<Account>,
    pub games: Vec<Game>,
    /// Game key -> cover/hero art (non-Steam games).
    pub art: std::collections::HashMap<String, crate::art::Art>,
}
