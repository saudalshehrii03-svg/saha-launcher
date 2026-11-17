//! Official covers (portrait 600x900) and hero banners for every game. Keyless sources:
//! 1. Steam IStoreBrowseService/GetItems: exact asset URLs (new games use hashed paths,
//!    so the old fixed `library_600x900.jpg` URL is missing for them).
//!    Used for Steam games and for any other game also sold on Steam (found by title).
//! 2. Microsoft Store catalog (Xbox games, by Store ID): Poster / BoxArt / hero art.
//! 3. SteamGridDB, when the user added an API key in settings.
//!
//! Results (including misses) are cached in art.json; entries from older versions are retried.
use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::library::models::{Game, PlatformId};
use crate::settings;

const FILE: &str = "art.json";
/// Bump to retry every cached miss after the lookup logic improves.
const VERSION: u32 = 2;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Art {
    pub cover: Option<String>,
    pub hero: Option<String>,
    /// SteamGridDB was already asked (so a new key can retry older misses).
    #[serde(default)]
    pub grid_tried: bool,
    #[serde(default)]
    pub v: u32,
}

pub fn cached() -> HashMap<String, Art> {
    settings::read_json(FILE)
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout(Duration::from_secs(10)).user_agent("Saha-Launcher").build()
}

fn norm(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase()
}

/// Games not found by title search (delisted etc.) -> Steam app id.
const KNOWN_STEAM: &[(&str, u64)] = &[("epic:Sugar", 252950)];

/// Steam asset URLs for many app ids in one call.
fn steam_assets(a: &ureq::Agent, ids: &[u64]) -> HashMap<u64, Art> {
    let mut out = HashMap::new();
    for chunk in ids.chunks(50) {
        let input = json!({
            "ids": chunk.iter().map(|id| json!({ "appid": id })).collect::<Vec<_>>(),
            "context": { "language": "english", "country_code": "US" },
            "data_request": { "include_assets": true }
        });
        let Some(v) = a
            .get("https://api.steampowered.com/IStoreBrowseService/GetItems/v1/")
            .query("input_json", &input.to_string())
            .call()
            .ok()
            .and_then(|r| r.into_json::<Value>().ok())
        else {
            continue;
        };
        for item in v["response"]["store_items"].as_array().into_iter().flatten() {
            let (Some(id), Some(fmt)) = (item["appid"].as_u64(), item["assets"]["asset_url_format"].as_str()) else { continue };
            let url = |key: &str| {
                item["assets"][key]
                    .as_str()
                    .map(|f| format!("https://shared.akamai.steamstatic.com/store_item_assets/{}", fmt.replace("${FILENAME}", f)))
            };
            let cover = url("library_capsule_2x").or_else(|| url("library_capsule"));
            let hero = url("library_hero").or_else(|| url("page_background"));
            if cover.is_some() || hero.is_some() {
                out.insert(id, Art { cover, hero, grid_tried: false, v: VERSION });
            }
        }
    }
    out
}

/// Steam app id by title; tries the title without "®/™", "Demo", "Beta", "Enhanced", "Launcher".
fn steam_search(a: &ureq::Agent, title: &str) -> Option<u64> {
    let clean: String = title.chars().filter(|c| !matches!(c, '®' | '™' | '©')).collect();
    let mut tries = vec![clean.clone()];
    let mut short = clean.clone();
    for w in [" Demo", " Beta", " Enhanced", " Launcher", " Remastered"] {
        short = short.replace(w, "");
    }
    if short != clean {
        tries.push(short.trim().to_string());
    }
    for term in tries {
        let Some(v) = a
            .get("https://store.steampowered.com/api/storesearch/")
            .query("term", &term)
            .query("l", "english")
            .query("cc", "US")
            .call()
            .ok()
            .and_then(|r| r.into_json::<Value>().ok())
        else {
            continue;
        };
        let want = norm(&term);
        let hit = v["items"].as_array().into_iter().flatten().find_map(|i| {
            let n = norm(i["name"].as_str()?);
            (n == want || (want.len() >= 6 && (n.starts_with(&want) || want.starts_with(&n)))).then(|| i["id"].as_u64()).flatten()
        });
        if hit.is_some() {
            return hit;
        }
    }
    None
}

/// Microsoft Store art by Store ID (Xbox / Game Pass games).
fn microsoft(a: &ureq::Agent, store_id: &str) -> Option<Art> {
    let v: Value = a
        .get("https://displaycatalog.mp.microsoft.com/v7.0/products")
        .query("bigIds", store_id)
        .query("market", "US")
        .query("languages", "en-us")
        .call()
        .ok()?
        .into_json()
        .ok()?;
    let images = v["Products"][0]["LocalizedProperties"][0]["Images"].as_array()?.clone();
    let find = |purposes: &[&str]| {
        purposes.iter().find_map(|p| {
            images
                .iter()
                .find(|i| i["ImagePurpose"].as_str() == Some(*p))
                .and_then(|i| i["Uri"].as_str())
                .map(|u| format!("https:{u}"))
        })
    };
    let cover = find(&["Poster", "BoxArt"]);
    let hero = find(&["SuperHeroArt", "TitledHeroArt", "BrandedKeyArt"]);
    (cover.is_some() || hero.is_some()).then_some(Art { cover, hero, grid_tried: false, v: VERSION })
}

fn from_grid(a: &ureq::Agent, key: &str, title: &str) -> Option<Art> {
    let auth = format!("Bearer {key}");
    let get = |url: &str| -> Option<Value> { a.get(url).set("Authorization", &auth).call().ok()?.into_json().ok() };
    let enc: String = title.chars().filter(|c| !matches!(c, '/' | '?' | '#' | '%')).collect();
    let search = get(&format!("https://www.steamgriddb.com/api/v2/search/autocomplete/{enc}"))?;
    let id = search["data"][0]["id"].as_u64()?;
    let first = |v: Option<Value>| v.and_then(|v| v["data"][0]["url"].as_str().map(str::to_string));
    let cover = first(get(&format!("https://www.steamgriddb.com/api/v2/grids/game/{id}?dimensions=600x900")));
    let hero = first(get(&format!("https://www.steamgriddb.com/api/v2/heroes/game/{id}")));
    (cover.is_some() || hero.is_some()).then_some(Art { cover, hero, grid_tried: true, v: VERSION })
}

/// Look up art for games that don't have it yet. Returns the whole cache.
pub fn fetch_missing(games: &[Game]) -> HashMap<String, Art> {
    let mut cache = cached();
    let key = settings::load().steamgriddb_key.trim().to_string();
    let a = agent();
    let needs = |g: &Game, cache: &HashMap<String, Art>| match cache.get(&g.key) {
        None => true,
        Some(k) if k.v < VERSION => true,
        Some(k) => k.cover.is_none() && !key.is_empty() && !k.grid_tried,
    };
    let todo: Vec<&Game> = games.iter().filter(|g| needs(g, &cache)).collect();
    if todo.is_empty() {
        return cache;
    }

    // Steam id for each game: its own, a known mapping, or a title search.
    let mut ids: HashMap<String, u64> = HashMap::new();
    for g in &todo {
        let known = KNOWN_STEAM.iter().find(|(k, _)| *k == g.key).map(|(_, id)| *id);
        let id = if g.platform == PlatformId::Steam { g.app_id.parse().ok() } else { known };
        let id = id.or_else(|| (g.platform != PlatformId::Steam && g.platform != PlatformId::Xbox).then(|| steam_search(&a, &g.title)).flatten());
        if let Some(id) = id {
            ids.insert(g.key.clone(), id);
        }
    }
    let mut unique: Vec<u64> = ids.values().copied().collect();
    unique.sort_unstable();
    unique.dedup();
    let assets = steam_assets(&a, &unique);

    for g in todo {
        let mut found = ids.get(&g.key).and_then(|id| assets.get(id)).cloned();
        if found.as_ref().is_none_or(|f| f.cover.is_none()) {
            if let Some(sid) = g.store_id.as_deref() {
                found = microsoft(&a, sid).or(found);
            }
        }
        if found.as_ref().is_none_or(|f| f.cover.is_none()) && !key.is_empty() {
            found = from_grid(&a, &key, &g.title).or(found);
        }
        let entry = found.unwrap_or(Art { cover: None, hero: None, grid_tried: !key.is_empty(), v: VERSION });
        cache.insert(g.key.clone(), entry);
    }
    let _ = settings::write_json(FILE, &cache);
    cache
}
