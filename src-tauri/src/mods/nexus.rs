//! Nexus Mods.
//! - browse / mod page: public GraphQL v2 (no key)
//! - download: v1 REST with the user's API key. Free accounts must click
//!   "Mod Manager Download" on the site; that opens nxm://... which Saha receives
//!   (key + expires in the link). Premium accounts can download directly.
use serde::Serialize;
use serde_json::{json, Value};

use crate::error::{msg, Result};
use crate::online::agent;
use crate::settings;

const GQL: &str = "https://api.nexusmods.com/v2/graphql";
const MOD_FIELDS: &str =
    "modId name summary pictureUrl thumbnailUrl endorsements downloads author version updatedAt adultContent game { domainName id }";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModInfo {
    pub mod_id: u64,
    pub domain: String,
    pub name: String,
    pub summary: String,
    pub picture: Option<String>,
    pub thumbnail: Option<String>,
    pub endorsements: u64,
    pub downloads: u64,
    pub author: String,
    pub version: String,
    pub updated_at: String,
    /// Mod page only.
    pub description: String,
    pub images: Vec<String>,
    pub url: String,
}

fn gql(query: &str, variables: Value) -> Result<Value> {
    let v: Value = agent()
        .post(GQL)
        .send_json(json!({ "query": query, "variables": variables }))
        .map_err(|e| crate::error::Error::Msg(format!("Nexus Mods ما رد: {e}")))?
        .into_json()?;
    Ok(v["data"].clone())
}

fn parse(n: &Value) -> Option<ModInfo> {
    let domain = n["game"]["domainName"].as_str()?.to_string();
    let mod_id = n["modId"].as_u64()?;
    let description = n["description"].as_str().unwrap_or("").to_string();
    Some(ModInfo {
        url: format!("https://www.nexusmods.com/{domain}/mods/{mod_id}"),
        images: bbcode_images(&description),
        description: bbcode_to_text(&description),
        mod_id,
        domain,
        name: n["name"].as_str()?.to_string(),
        summary: n["summary"].as_str().unwrap_or("").to_string(),
        picture: n["pictureUrl"].as_str().map(str::to_string),
        thumbnail: n["thumbnailUrl"].as_str().map(str::to_string),
        endorsements: n["endorsements"].as_u64().unwrap_or(0),
        downloads: n["downloads"].as_u64().unwrap_or(0),
        author: n["author"].as_str().unwrap_or("").to_string(),
        version: n["version"].as_str().unwrap_or("").to_string(),
        updated_at: n["updatedAt"].as_str().unwrap_or("").to_string(),
    })
}

fn bbcode_images(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(i) = rest.find("[img]") {
        rest = &rest[i + 5..];
        if let Some(j) = rest.find("[/img]") {
            let url = rest[..j].trim();
            if url.starts_with("http") && !out.iter().any(|u| u == url) {
                out.push(url.to_string());
            }
            rest = &rest[j..];
        }
    }
    out.truncate(12);
    out
}

/// Strip BBCode/HTML tags, keep text and line breaks.
fn bbcode_to_text(s: &str) -> String {
    let s = s.replace("<br />", "\n").replace("<br>", "\n");
    let mut out = String::new();
    let mut skip_img = false;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '[' || c == '<' {
            let close = if c == '[' { ']' } else { '>' };
            if let Some(len) = chars[i..].iter().position(|&x| x == close) {
                let tag: String = chars[i + 1..i + len].iter().collect::<String>().to_lowercase();
                if tag == "img" {
                    skip_img = true;
                } else if tag == "/img" {
                    skip_img = false;
                }
                i += len + 1;
                continue;
            }
        }
        if !skip_img {
            out.push(c);
        }
        i += 1;
    }
    let text = out.replace("&nbsp;", " ").replace("&amp;", "&").replace("&quot;", "\"");
    // collapse 3+ blank lines
    let mut res = String::new();
    let mut blank = 0;
    for line in text.lines() {
        if line.trim().is_empty() {
            blank += 1;
            if blank > 1 {
                continue;
            }
        } else {
            blank = 0;
        }
        res.push_str(line.trim_end());
        res.push('\n');
    }
    res.trim().chars().take(6000).collect()
}

/// Nexus game (domain, name) for a library title. Strict: the Nexus name must equal the
/// title, its part before ":" / "-", or the title without edition words ("Demo", "Remastered"...).
/// A loose "contains" match once linked games to the wrong mods, so prefixes alone never count.
pub fn game_domain(title: &str) -> Option<(String, String)> {
    let clean: String = title.chars().filter(|c| !matches!(c, '®' | '™' | '©')).collect();
    let clean = clean.replace(" — ", " - ");
    let norm = |s: &str| s.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase();
    let mut accept = vec![norm(&clean)];
    if let Some(short) = clean.split([':', '-', '—']).next() {
        accept.push(norm(short));
    }
    let mut stripped = clean.to_lowercase();
    for w in ["demo", "beta", "remastered", "enhanced", "definitive edition", "game of the year edition", "goty edition", "complete edition", "launcher", "director's cut"] {
        stripped = stripped.replace(w, "");
    }
    accept.push(norm(&stripped));
    accept.retain(|a| a.len() >= 3);

    let q = "query($n: String!) { games(filter: { name: { value: $n, op: WILDCARD } }, count: 30) { nodes { domainName name } } }";
    let short = clean.split([':', '-', '—']).next()?.trim().to_string();
    let mut found: Vec<(String, String)> = Vec::new();
    let stripped_term = stripped.split_whitespace().collect::<Vec<_>>().join(" ");
    for term in [clean.trim().to_string(), short, stripped_term] {
        if term.len() < 3 {
            continue;
        }
        let Ok(d) = gql(q, json!({ "n": term })) else { continue };
        for g in d["games"]["nodes"].as_array().into_iter().flatten() {
            if let (Some(dom), Some(name)) = (g["domainName"].as_str(), g["name"].as_str()) {
                found.push((dom.to_string(), name.to_string()));
            }
        }
    }
    // Prefer the most specific accepted form (full title first).
    accept.iter().find_map(|a| found.iter().find(|(_, n)| norm(n) == *a).cloned())
}

/// Browse: sort = "endorsements" | "downloads" | "updatedAt" | "createdAt".
pub fn browse(domain: &str, term: &str, sort: &str) -> Result<Vec<ModInfo>> {
    let sort_key = match sort {
        "downloads" | "updatedAt" | "createdAt" => sort,
        _ => "endorsements",
    };
    let has_term = !term.trim().is_empty();
    let (decl, name_filter) =
        if has_term { (", $term: String!", ", name: { value: $term, op: WILDCARD }") } else { ("", "") };
    let q = format!(
        "query($d: String!{decl}) {{ mods(filter: {{ gameDomainName: {{ value: $d, op: EQUALS }}, adultContent: {{ value: false, op: EQUALS }}{name_filter} }}, \
         sort: {{ {sort_key}: {{ direction: DESC }} }}, count: 40) {{ nodes {{ {MOD_FIELDS} }} }} }}"
    );
    let vars = if has_term { json!({ "d": domain, "term": term.trim() }) } else { json!({ "d": domain }) };
    let d = gql(&q, vars)?;
    Ok(d["mods"]["nodes"].as_array().map(|a| a.iter().filter_map(parse).collect()).unwrap_or_default())
}

pub fn mod_page(domain: &str, mod_id: u64) -> Result<ModInfo> {
    // Filtering by modId needs the numeric game id; get it from any mod of the domain.
    let g = gql(
        "query($d: String!) { mods(filter: { gameDomainName: { value: $d, op: EQUALS } }, count: 1) { nodes { game { id } } } }",
        json!({ "d": domain }),
    )?;
    let game_id = g["mods"]["nodes"][0]["game"]["id"]
        .as_u64()
        .map(|v| v.to_string())
        .or_else(|| g["mods"]["nodes"][0]["game"]["id"].as_str().map(str::to_string))
        .ok_or_else(|| crate::error::Error::Msg("اللعبة غير موجودة في Nexus.".into()))?;
    let q = format!(
        "query($g: String!, $id: String!) {{ mods(filter: {{ gameId: {{ value: $g, op: EQUALS }}, modId: {{ value: $id, op: EQUALS }} }}, count: 1) \
         {{ nodes {{ {MOD_FIELDS} description }} }} }}"
    );
    let d = gql(&q, json!({ "g": game_id, "id": mod_id.to_string() }))?;
    d["mods"]["nodes"].get(0).and_then(parse).ok_or_else(|| crate::error::Error::Msg("المود غير موجود.".into()))
}

fn api_key() -> Result<String> {
    let k = settings::load().nexus_key.trim().to_string();
    if k.is_empty() {
        return msg("أضف مفتاح Nexus Mods من الإعدادات (مجاني) عشان تقدر تثبّت المودات.");
    }
    Ok(k)
}

fn v1(url: &str) -> Result<Value> {
    let key = api_key()?;
    agent()
        .get(url)
        .set("apikey", &key)
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(403, _) => crate::error::Error::Msg("Nexus يطلب الضغط على «Mod Manager Download» من صفحة المود (الحساب المجاني).".into()),
            other => crate::error::Error::Msg(format!("Nexus: {other}")),
        })?
        .into_json()
        .map_err(Into::into)
}

#[derive(Debug, Clone)]
pub struct FileRef {
    pub domain: String,
    pub mod_id: u64,
    pub file_id: u64,
    pub key: Option<String>,
    pub expires: Option<String>,
}

/// Parse nxm://<domain>/mods/<mod>/files/<file>?key=..&expires=..
pub fn parse_nxm(link: &str) -> Option<FileRef> {
    let rest = link.strip_prefix("nxm://")?;
    let (path, query) = rest.split_once('?').unwrap_or((rest, ""));
    let parts: Vec<&str> = path.trim_end_matches('/').split('/').collect();
    if parts.len() < 5 || parts[1] != "mods" || parts[3] != "files" {
        return None;
    }
    let q = |k: &str| query.split('&').find_map(|kv| kv.strip_prefix(&format!("{k}=")).map(str::to_string));
    Some(FileRef {
        domain: parts[0].to_lowercase(),
        mod_id: parts[2].parse().ok()?,
        file_id: parts[4].parse().ok()?,
        key: q("key"),
        expires: q("expires"),
    })
}

/// Main file of a mod (premium direct-download path).
pub fn main_file(domain: &str, mod_id: u64) -> Result<u64> {
    let v = v1(&format!("https://api.nexusmods.com/v1/games/{domain}/mods/{mod_id}/files.json"))?;
    let files = v["files"].as_array().cloned().unwrap_or_default();
    files
        .iter()
        .rev()
        .find(|f| f["category_name"].as_str() == Some("MAIN"))
        .or(files.last())
        .and_then(|f| f["file_id"].as_u64())
        .ok_or_else(|| crate::error::Error::Msg("ما لقيت ملفات للمود.".into()))
}

/// (download url, file name, version)
pub fn download_link(f: &FileRef) -> Result<(String, String, String)> {
    let base = format!("https://api.nexusmods.com/v1/games/{}/mods/{}/files/{}", f.domain, f.mod_id, f.file_id);
    let info = v1(&format!("{base}.json"))?;
    let mut url = format!("{base}/download_link.json");
    if let (Some(k), Some(e)) = (&f.key, &f.expires) {
        url.push_str(&format!("?key={k}&expires={e}"));
    }
    let links = v1(&url)?;
    let uri = links[0]["URI"].as_str().ok_or_else(|| crate::error::Error::Msg("Nexus ما أعطى رابط تنزيل.".into()))?;
    Ok((
        uri.to_string(),
        info["file_name"].as_str().unwrap_or("mod.zip").to_string(),
        info["version"].as_str().unwrap_or("").to_string(),
    ))
}

#[cfg(test)]
mod tests {
    /// Network: `cargo test domains_local -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn domains_local() {
        for g in crate::library::scan().games {
            println!("{} => {:?}", g.title, super::game_domain(&g.title));
        }
    }
}
