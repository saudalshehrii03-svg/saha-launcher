//! Official covers for games that are not Steam games (Steam games use their own art).
//! 1. Steam store search by title (no key needed): if the game is also on Steam, use its art.
//! 2. SteamGridDB, when the user added an API key in settings.
//!
//! Results (including misses) are cached in art.json so lookups happen once.
use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::library::models::{Game, PlatformId};
use crate::settings;

const FILE: &str = "art.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Art {
    pub cover: Option<String>,
    pub hero: Option<String>,
    /// SteamGridDB was already asked (so a new key can retry older misses).
    #[serde(default)]
    pub grid_tried: bool,
}

pub fn cached() -> HashMap<String, Art> {
    settings::read_json(FILE)
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout(Duration::from_secs(8)).user_agent("Saha-Launcher").build()
}

fn norm(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase()
}

fn exists(a: &ureq::Agent, url: &str) -> bool {
    a.head(url).call().map(|r| r.status() == 200).unwrap_or(false)
}

fn from_steam(a: &ureq::Agent, title: &str) -> Option<Art> {
    #[derive(Deserialize)]
    struct Item {
        id: u64,
        name: String,
    }
    #[derive(Deserialize)]
    struct Res {
        items: Vec<Item>,
    }
    let res: Res = a
        .get("https://store.steampowered.com/api/storesearch/")
        .query("term", title)
        .query("l", "english")
        .query("cc", "US")
        .call()
        .ok()?
        .into_json()
        .ok()?;
    let want = norm(title);
    let item = res.items.into_iter().find(|i| {
        let n = norm(&i.name);
        n == want || (want.len() >= 6 && (n.starts_with(&want) || want.starts_with(&n)))
    })?;
    let base = format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}", item.id);
    let cover = format!("{base}/library_600x900.jpg");
    if !exists(a, &cover) {
        return None;
    }
    let hero = Some(format!("{base}/library_hero.jpg")).filter(|h| exists(a, h));
    Some(Art { cover: Some(cover), hero, grid_tried: false })
}

fn from_grid(a: &ureq::Agent, key: &str, title: &str) -> Option<Art> {
    let auth = format!("Bearer {key}");
    let get = |url: &str| -> Option<serde_json::Value> {
        a.get(url).set("Authorization", &auth).call().ok()?.into_json().ok()
    };
    let enc: String = title.chars().filter(|c| !matches!(c, '/' | '?' | '#' | '%')).collect();
    let search = get(&format!("https://www.steamgriddb.com/api/v2/search/autocomplete/{enc}"))?;
    let id = search["data"][0]["id"].as_u64()?;
    let first = |v: Option<serde_json::Value>| v.and_then(|v| v["data"][0]["url"].as_str().map(str::to_string));
    let cover = first(get(&format!("https://www.steamgriddb.com/api/v2/grids/game/{id}?dimensions=600x900")));
    let hero = first(get(&format!("https://www.steamgriddb.com/api/v2/heroes/game/{id}")));
    (cover.is_some() || hero.is_some()).then_some(Art { cover, hero, grid_tried: true })
}

/// Look up art for non-Steam games that don't have it yet. Returns the whole cache.
pub fn fetch_missing(games: &[Game]) -> HashMap<String, Art> {
    let mut cache = cached();
    let key = settings::load().steamgriddb_key.trim().to_string();
    let a = agent();
    let mut changed = false;
    for g in games.iter().filter(|g| g.platform != PlatformId::Steam) {
        let known = cache.get(&g.key);
        if known.is_some_and(|k| k.cover.is_some() || key.is_empty() || k.grid_tried) {
            continue;
        }
        let tried_steam = known.is_some();
        let found = if tried_steam { None } else { from_steam(&a, &g.title) };
        let found = found.or_else(|| (!key.is_empty()).then(|| from_grid(&a, &key, &g.title)).flatten());
        let entry = found.unwrap_or(Art { cover: None, hero: None, grid_tried: !key.is_empty() });
        cache.insert(g.key.clone(), entry);
        changed = true;
    }
    if changed {
        let _ = settings::write_json(FILE, &cache);
    }
    cache
}
