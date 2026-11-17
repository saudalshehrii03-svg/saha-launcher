//! "What does this mod do?" in Arabic + one explainer video. Keyless:
//! - Arabic: the mod's own summary + first paragraphs, machine-translated (Google translate endpoint)
//! - Video: first YouTube search result for "<mod> <game> mod"
use serde::Serialize;
use serde_json::Value;

use crate::online::agent;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub id: String,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Explain {
    /// Arabic summary of what the mod does (empty if translation failed).
    pub arabic: String,
    /// Written by Claude (true) or machine-translated (false).
    pub ai: bool,
    pub video: Option<Video>,
    /// YouTube search page, used when no single video was found.
    pub search_url: String,
}

fn translate(text: &str) -> Option<String> {
    let v: Value = agent()
        .get("https://translate.googleapis.com/translate_a/single")
        .query("client", "gtx")
        .query("sl", "auto")
        .query("tl", "ar")
        .query("dt", "t")
        .query("q", text)
        .call()
        .ok()?
        .into_json()
        .ok()?;
    let out: String = v[0].as_array()?.iter().filter_map(|s| s[0].as_str()).collect();
    Some(out.trim().to_string()).filter(|s| !s.is_empty())
}

/// Summary + the first meaningful paragraphs of the description, capped.
fn source_text(summary: &str, description: &str) -> String {
    let mut text = summary.trim().to_string();
    for para in description.split("\n\n").map(str::trim).filter(|p| p.len() > 40).take(3) {
        if text.len() + para.len() > 1400 {
            break;
        }
        text.push_str("\n\n");
        text.push_str(para);
    }
    text
}

fn enc(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            b' ' => "+".into(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn youtube(query: &str) -> Option<Video> {
    let html = agent()
        .get("https://www.youtube.com/results")
        .query("search_query", query)
        .set("Accept-Language", "en")
        .call()
        .ok()?
        .into_string()
        .ok()?;
    // First regular video result in the embedded JSON.
    let start = html.find("\"videoRenderer\":{\"videoId\":\"")? + 28;
    let id: String = html[start..].chars().take_while(|c| *c != '"').collect();
    let after = &html[start..];
    let title = after
        .find("\"title\":{\"runs\":[{\"text\":\"")
        .map(|i| after[i + 26..].chars().take_while(|c| *c != '"').collect::<String>())
        .unwrap_or_default()
        .replace("\\u0026", "&");
    (id.len() == 11).then(|| Video { url: format!("https://www.youtube.com/watch?v={id}"), id, title })
}

/// Same Arabic text for a game page (story/description).
#[derive(Debug, Clone, Serialize)]
pub struct GameText {
    pub arabic: String,
    pub ai: bool,
}

pub fn game_text(title: &str, description: &str) -> GameText {
    if let Some(t) = crate::ai::explain("game", title, title, description) {
        return GameText { arabic: t, ai: true };
    }
    GameText { arabic: translate(description).unwrap_or_default(), ai: false }
}

pub fn explain(name: &str, game: &str, summary: &str, description: &str) -> Explain {
    let query = format!("{name} {game} mod");
    let search_url = format!("https://www.youtube.com/results?search_query={}", enc(&query));
    let src = source_text(summary, description);
    std::thread::scope(|s| {
        let tr = s.spawn(|| {
            let full: String = format!("{summary}

{description}").chars().take(6000).collect();
            crate::ai::explain("mod", name, game, &full).map(|t| (t, true)).or_else(|| translate(&src).map(|t| (t, false)))
        });
        let yt = s.spawn(|| youtube(&query));
        let (arabic, ai) = tr.join().ok().flatten().unwrap_or_default();
        Explain {
            arabic,
            ai,
            video: yt.join().ok().flatten(),
            search_url,
        }
    })
}

#[cfg(test)]
mod tests {
    /// Network: `cargo test explain_live -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn explain_live() {
        let e = super::explain("Cyber Engine Tweaks", "Cyberpunk 2077", "Scripting framework for modders and quality of life fixes.", "");
        println!("ar={}\nvideo={:?}", e.arabic, e.video);
    }
}
