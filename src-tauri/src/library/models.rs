//! Shared data models. Mirrored in `src/lib/types.ts`.
use serde::{Deserialize, Serialize};

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

#[allow(dead_code)] // used from phase 1
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub platform: PlatformId,
    /// Platform-side id, e.g. SteamID64.
    pub id: String,
    pub display_name: String,
    /// Login name used by the switcher. Passwords are never stored.
    pub login_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    /// Stable key, e.g. "steam:1938090".
    pub key: String,
    pub title: String,
    pub platform: PlatformId,
    pub app_id: String,
    /// Account that owns the game; launching switches to it first.
    pub owner_account_id: Option<String>,
    pub installed: bool,
    pub install_dir: Option<String>,
    pub size_bytes: Option<u64>,
    pub playtime_minutes: Option<u64>,
    pub last_played: Option<String>,
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
