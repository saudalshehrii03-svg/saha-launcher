//! Online data (all keyless except owned games):
//! - owned Steam games per account: Steam Web API GetOwnedGames (user's API key), cached in owned.json
//! - game page: Steam appdetails, current players, awards (Wikidata), cheapest trusted deals (CheapShark)
//! - store browsing: Steam featured categories + search
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{msg, Result};
use crate::library::models::Account;
use crate::settings;

pub fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout(Duration::from_secs(10)).user_agent("Saha-Launcher/0.2").build()
}

fn get_json(a: &ureq::Agent, url: &str, query: &[(&str, &str)]) -> Option<Value> {
    let mut r = a.get(url);
    for (k, v) in query {
        r = r.query(k, v);
    }
    r.call().ok()?.into_json().ok()
}

// ---------- owned Steam games ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Owned {
    pub app_id: u64,
    pub name: String,
    pub playtime: u64,
    pub last_played: u64,
}

pub const OWNED_FILE: &str = "owned.json";

/// Account id64 -> owned games (cache).
pub fn owned_cached() -> HashMap<String, Vec<Owned>> {
    settings::read_json(OWNED_FILE)
}

/// Refresh owned games for every Steam account. Returns how many accounts answered.
pub fn sync_owned(accounts: &[Account]) -> Result<usize> {
    let key = settings::load().steam_api_key.trim().to_string();
    if key.is_empty() {
        return msg("أضف مفتاح Steam Web API من الإعدادات عشان تظهر كل ألعابك.");
    }
    let a = agent();
    let mut all = owned_cached();
    let mut ok = 0;
    for acc in accounts {
        let Some(v) = get_json(
            &a,
            "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/",
            &[("key", &key), ("steamid", &acc.id), ("include_appinfo", "1"), ("include_played_free_games", "1")],
        ) else {
            continue;
        };
        let Some(games) = v["response"]["games"].as_array() else { continue }; // private profile
        all.insert(
            acc.id.clone(),
            games
                .iter()
                .filter_map(|g| {
                    Some(Owned {
                        app_id: g["appid"].as_u64()?,
                        name: g["name"].as_str()?.to_string(),
                        playtime: g["playtime_forever"].as_u64().unwrap_or(0),
                        last_played: g["rtime_last_played"].as_u64().unwrap_or(0),
                    })
                })
                .collect(),
        );
        ok += 1;
    }
    settings::write_json(OWNED_FILE, &all)?;
    if ok == 0 {
        return msg("ستيم ما رجّع ألعاب. تأكد من المفتاح، وإن «تفاصيل الألعاب» في خصوصية كل حساب عامة.");
    }
    Ok(ok)
}

// ---------- game page ----------

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Deal {
    pub store: String,
    /// "official" (the publisher/platform store) or "authorized" (licensed reseller).
    pub trust: String,
    pub price: String,
    pub retail: String,
    pub savings: u32,
    pub url: String,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Details {
    pub steam_app_id: Option<u64>,
    pub name: String,
    pub description: String,
    pub background: Option<String>,
    pub screenshots: Vec<String>,
    pub genres: Vec<String>,
    pub developers: Vec<String>,
    pub publishers: Vec<String>,
    pub release_date: String,
    pub metacritic: Option<u64>,
    pub price: Option<String>,
    pub current_players: Option<u64>,
    pub awards: Vec<String>,
    pub goty: bool,
    pub deals: Vec<Deal>,
}

fn strs(v: &Value, field: &str) -> Vec<String> {
    v.as_array()
        .map(|a| a.iter().filter_map(|x| (if field.is_empty() { x } else { &x[field] }).as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

/// Find a Steam app id by title (exact-ish match).
pub fn steam_search_id(a: &ureq::Agent, title: &str) -> Option<u64> {
    let norm = |s: &str| s.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase();
    let v = get_json(a, "https://store.steampowered.com/api/storesearch/", &[("term", title), ("l", "english"), ("cc", "US")])?;
    let want = norm(title);
    v["items"].as_array()?.iter().find_map(|i| {
        let n = norm(i["name"].as_str()?);
        (n == want || (want.len() >= 6 && (n.starts_with(&want) || want.starts_with(&n)))).then(|| i["id"].as_u64()).flatten()
    })
}

fn players(a: &ureq::Agent, id: u64) -> Option<u64> {
    let v = get_json(a, "https://api.steampowered.com/ISteamUserStats/GetNumberOfCurrentPlayers/v1/", &[("appid", &id.to_string())])?;
    v["response"]["player_count"].as_u64()
}

fn awards(a: &ureq::Agent, id: u64) -> Vec<String> {
    let q = format!(
        "SELECT DISTINCT ?awardLabel WHERE {{ ?g wdt:P1733 \"{id}\" . ?g wdt:P166 ?award . \
         SERVICE wikibase:label {{ bd:serviceParam wikibase:language \"ar,en\". }} }} LIMIT 40"
    );
    get_json(a, "https://query.wikidata.org/sparql", &[("query", &q), ("format", "json")])
        .and_then(|v| v["results"]["bindings"].as_array().cloned())
        .map(|b| b.iter().filter_map(|x| x["awardLabel"]["value"].as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

fn store_names(a: &ureq::Agent) -> &'static HashMap<String, String> {
    static STORES: OnceLock<HashMap<String, String>> = OnceLock::new();
    STORES.get_or_init(|| {
        get_json(a, "https://www.cheapshark.com/api/1.0/stores", &[])
            .and_then(|v| v.as_array().cloned())
            .map(|s| {
                s.iter()
                    .filter_map(|x| Some((x["storeID"].as_str()?.to_string(), x["storeName"].as_str()?.to_string())))
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// Only these stores are shown: platform stores and well-known licensed resellers.
/// (CheapShark already excludes grey-market key sites like G2A/Kinguin; this list is stricter.)
const TRUSTED: &[(&str, &str)] = &[
    ("Steam", "official"),
    ("GOG", "official"),
    ("Epic Games Store", "official"),
    ("Ubisoft Store", "official"),
    ("Uplay", "official"),
    ("Origin", "official"),
    ("EA", "official"),
    ("Blizzard Shop", "official"),
    ("Humble Store", "authorized"),
    ("Fanatical", "authorized"),
    ("GreenManGaming", "authorized"),
    ("GamersGate", "authorized"),
    ("Gamesplanet", "authorized"),
    ("WinGameStore", "authorized"),
    ("IndieGala Store", "authorized"),
    ("GameBillet", "authorized"),
];

fn trust(store: &str) -> Option<&'static str> {
    TRUSTED.iter().find(|(name, _)| store.starts_with(name)).map(|(_, t)| *t)
}

/// Cheapest offers from trusted stores only.
fn deals(a: &ureq::Agent, steam_id: Option<u64>, title: &str) -> Vec<Deal> {
    let found = match steam_id {
        Some(id) => get_json(a, "https://www.cheapshark.com/api/1.0/games", &[("steamAppID", &id.to_string())]),
        None => get_json(a, "https://www.cheapshark.com/api/1.0/games", &[("title", title), ("exact", "1"), ("limit", "1")]),
    };
    let Some(game_id) = found.as_ref().and_then(|v| v[0]["gameID"].as_str()) else { return Vec::new() };
    let Some(info) = get_json(a, "https://www.cheapshark.com/api/1.0/games", &[("id", game_id)]) else { return Vec::new() };
    let stores = store_names(a);
    let mut out: Vec<Deal> = info["deals"]
        .as_array()
        .map(|d| {
            d.iter()
                .filter_map(|x| {
                    let deal = x["dealID"].as_str()?;
                    let store = stores.get(x["storeID"].as_str()?)?.clone();
                    let trust = trust(&store)?.to_string();
                    Some(Deal {
                        store,
                        trust,
                        price: format!("${}", x["price"].as_str()?),
                        retail: format!("${}", x["retailPrice"].as_str().unwrap_or("")),
                        savings: x["savings"].as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0).round() as u32,
                        url: format!("https://www.cheapshark.com/redirect?dealID={deal}"),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    out.sort_by(|a, b| {
        let p = |d: &Deal| d.price.trim_start_matches('$').parse::<f64>().unwrap_or(f64::MAX);
        p(a).total_cmp(&p(b))
    });
    out.truncate(4);
    out
}

pub fn details(title: &str, steam_app_id: Option<u64>) -> Details {
    let a = agent();
    let id = steam_app_id.or_else(|| steam_search_id(&a, title));
    let mut d = Details { steam_app_id: id, name: title.to_string(), ..Default::default() };
    std::thread::scope(|s| {
        let app = s.spawn(|| {
            id.and_then(|id| {
                get_json(&a, "https://store.steampowered.com/api/appdetails", &[("appids", &id.to_string()), ("l", "arabic"), ("cc", "SA")])
                    .map(|v| v[id.to_string()]["data"].clone())
            })
        });
        let pl = s.spawn(|| id.and_then(|id| players(&a, id)));
        let aw = s.spawn(|| id.map(|id| awards(&a, id)).unwrap_or_default());
        let dl = s.spawn(|| deals(&a, id, title));
        if let Some(Some(v)) = app.join().ok().filter(|v| v.as_ref().is_some_and(|v| !v.is_null())) {
            d.name = v["name"].as_str().unwrap_or(title).to_string();
            d.description = v["short_description"].as_str().unwrap_or("").to_string();
            d.background = v["background_raw"].as_str().or(v["background"].as_str()).map(str::to_string);
            d.screenshots = strs(&v["screenshots"], "path_full").into_iter().take(8).collect();
            d.genres = strs(&v["genres"], "description");
            d.developers = strs(&v["developers"], "");
            d.publishers = strs(&v["publishers"], "");
            d.release_date = v["release_date"]["date"].as_str().unwrap_or("").to_string();
            d.metacritic = v["metacritic"]["score"].as_u64();
            d.price = v["price_overview"]["final_formatted"]
                .as_str()
                .map(str::to_string)
                .or_else(|| v["is_free"].as_bool().filter(|f| *f).map(|_| "مجانية".into()));
        }
        d.current_players = pl.join().ok().flatten();
        d.awards = aw.join().unwrap_or_default();
        d.deals = dl.join().unwrap_or_default();
    });
    d.goty = d.awards.iter().any(|x| {
        let l = x.to_lowercase();
        l.contains("game of the year") || x.contains("لعبة العام") || x.contains("لعبة السنة")
    });
    d
}

// ---------- store ----------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreItem {
    pub app_id: u64,
    pub name: String,
    pub image: String,
    pub price: Option<String>,
    pub discount: u64,
}

fn header(id: u64) -> String {
    format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{id}/header.jpg")
}

fn price_str(cents: Option<u64>, currency: &str) -> Option<String> {
    cents.map(|c| if c == 0 { "مجانية".into() } else { format!("{:.2} {currency}", c as f64 / 100.0) })
}

/// Sections: new_releases, top_sellers, specials, coming_soon.
pub fn featured() -> HashMap<String, Vec<StoreItem>> {
    let a = agent();
    let Some(v) = get_json(&a, "https://store.steampowered.com/api/featuredcategories", &[("cc", "SA"), ("l", "arabic")]) else {
        return HashMap::new();
    };
    ["new_releases", "top_sellers", "specials", "coming_soon"]
        .iter()
        .map(|sec| {
            let mut seen = Vec::new();
            let items = v[*sec]["items"]
                .as_array()
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|i| {
                            let id = i["id"].as_u64()?;
                            if seen.contains(&id) {
                                return None;
                            }
                            seen.push(id);
                            Some(StoreItem {
                                app_id: id,
                                name: i["name"].as_str()?.to_string(),
                                image: i["header_image"].as_str().map(str::to_string).unwrap_or_else(|| header(id)),
                                price: price_str(i["final_price"].as_u64(), i["currency"].as_str().unwrap_or("SAR")),
                                discount: i["discount_percent"].as_u64().unwrap_or(0),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            (sec.to_string(), items)
        })
        .collect()
}

pub fn search(term: &str) -> Vec<StoreItem> {
    let a = agent();
    get_json(&a, "https://store.steampowered.com/api/storesearch/", &[("term", term), ("l", "arabic"), ("cc", "SA")])
        .and_then(|v| v["items"].as_array().cloned())
        .map(|items| {
            items
                .iter()
                .filter_map(|i| {
                    let id = i["id"].as_u64()?;
                    Some(StoreItem {
                        app_id: id,
                        name: i["name"].as_str()?.to_string(),
                        image: header(id),
                        price: price_str(i["price"]["final"].as_u64(), i["price"]["currency"].as_str().unwrap_or("SAR")),
                        discount: 0,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    /// Network: `cargo test online_live -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn online_live() {
        let d = super::details("ELDEN RING", Some(1245620));
        println!("name={} players={:?} goty={} awards={} deals={:?} price={:?} genres={:?}", d.name, d.current_players, d.goty, d.awards.len(), d.deals.iter().map(|x| format!("{} {}", x.store, x.price)).collect::<Vec<_>>(), d.price, d.genres);
        let f = super::featured();
        println!("featured: {:?}", f.iter().map(|(k, v)| (k.clone(), v.len())).collect::<Vec<_>>());
        let dom = crate::mods::nexus::game_domain("Cyberpunk 2077®");
        let w3 = crate::mods::nexus::game_domain("The Witcher 3: Wild Hunt — Remastered");
        println!("domains: {dom:?} {w3:?}");
        let m = crate::mods::nexus::browse("cyberpunk2077", "", "endorsements").unwrap();
        println!("mods: {} first={:?}", m.len(), m.first().map(|x| &x.name));
        let p = crate::mods::nexus::mod_page("cyberpunk2077", 107).unwrap();
        println!("page images={} desc={}", p.images.len(), p.description.chars().take(80).collect::<String>());
    }
}
