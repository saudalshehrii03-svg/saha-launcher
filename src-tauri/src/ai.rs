//! Natural Saudi-Arabic explanations of mod and game descriptions, written by Claude.
//!
//! Raw HTTP to the Messages API (Rust has no official SDK). Needs the user's Anthropic
//! API key (settings.anthropic_key); without it callers fall back to machine translation.
//! Results are cached in ai_cache.json, so each mod/game is paid for once.
use std::collections::HashMap;
use std::time::Duration;

use serde_json::{json, Value};

use crate::settings;

const FILE: &str = "ai_cache.json";
const MODEL: &str = "claude-opus-5-5";

const SYSTEM_MOD: &str = "أنت تشرح مودات ألعاب الكمبيوتر للاعب سعودي. اكتب باللهجة السعودية البيضاء العامية \
المفهومة (مثل كلام الناس اليومي)، مو فصحى ولا ترجمة حرفية. خلّ أسماء الألعاب والمودات والمصطلحات التقنية \
(Script Hook، ReShade، DLL) بالإنجليزي.\n\
الشكل المطلوب بالضبط:\n\
سطر أو سطرين: وش يسوي المود وليش الناس تركّبه.\n\
بعدها «المميزات:» ونقاط قصيرة تبدأ بـ • (من ٢ إلى ٥ نقاط).\n\
بعدها «يحتاج:» واذكر المتطلبات إن وُجدت (محمّلات، مودات ثانية، نسخة معيّنة من اللعبة)، أو اكتب «ما يحتاج شي إضافي».\n\
وإذا فيه خطر حظر أو يخرب الحفظ أو ما يشتغل أونلاين، أضف سطر يبدأ بـ ⚠️.\n\
لا تستخدم عناوين ماركداون ولا نجوم. لا تتجاوز ١٢٠ كلمة. لا تخترع معلومات مو موجودة في الوصف.";

const SYSTEM_GAME: &str = "أنت تعرّف لاعب سعودي على لعبة كمبيوتر. اكتب باللهجة السعودية البيضاء العامية المفهومة، \
مو فصحى ولا ترجمة حرفية. خلّ اسم اللعبة وأسماء الشخصيات والأماكن بالإنجليزي.\n\
اكتب فقرة وحدة (٣ إلى ٥ جمل): وش قصة اللعبة أو فكرتها بدون حرق أحداث، ووش نوع اللعب فيها، وليش ممكن تعجبه.\n\
لا عناوين ولا نقاط ولا نجوم. لا تتجاوز ٨٠ كلمة. لا تخترع معلومات مو موجودة في الوصف.";

fn key() -> Option<String> {
    Some(settings::load().anthropic_key.trim().to_string()).filter(|k| !k.is_empty())
}

fn cache_key(kind: &str, name: &str, text: &str) -> String {
    // Cheap stable hash (FNV-1a) so a changed description is explained again.
    let mut h: u64 = 0xcbf29ce484222325;
    for b in text.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{kind}:{name}:{h:x}")
}

/// Arabic explanation, or None when there is no key / the call fails or is declined.
/// `kind` is "mod" or "game".
pub fn explain(kind: &str, name: &str, game: &str, text: &str) -> Option<String> {
    let api_key = key()?;
    let ck = cache_key(kind, name, text);
    let mut cache: HashMap<String, String> = settings::read_json(FILE);
    if let Some(hit) = cache.get(&ck) {
        return Some(hit.clone());
    }

    let system = if kind == "game" { SYSTEM_GAME } else { SYSTEM_MOD };
    let user = if kind == "game" {
        format!("اللعبة: {name}\n\nالوصف الرسمي:\n{text}")
    } else {
        format!("اللعبة: {game}\nاسم المود: {name}\n\nوصف المود من صفحته:\n{text}")
    };
    let body = json!({
        "model": MODEL,
        "max_tokens": 16000,
        // Short, well-defined writing task: low thinking effort is enough.
        "output_config": { "effort": "low" },
        // If a safety classifier declines, the API retries on the default fallback model.
        "fallbacks": "default",
        "system": system,
        "messages": [{ "role": "user", "content": user }],
    });

    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(90)).build();
    let resp: Value = agent
        .post("https://api.anthropic.com/v1/messages")
        .set("x-api-key", &api_key)
        .set("anthropic-version", "2026-10-21")
        .set("anthropic-beta", "server-side-fallback-2026-07-01")
        .set("content-type", "application/json")
        .send_json(body)
        .ok()?
        .into_json()
        .ok()?;
    if resp["stop_reason"].as_str() == Some("refusal") {
        return None;
    }
    let out: String = resp["content"]
        .as_array()?
        .iter()
        .filter(|b| b["type"].as_str() == Some("text"))
        .filter_map(|b| b["text"].as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let out = out.trim().to_string();
    if out.is_empty() {
        return None;
    }
    cache.insert(ck, out.clone());
    let _ = settings::write_json(FILE, &cache);
    Some(out)
}

/// Check a key with a tiny request. Ok(()) = works.
pub fn test_key(api_key: &str) -> Result<(), String> {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(60)).build();
    let body = json!({
        "model": MODEL,
        "max_tokens": 2000,
        "output_config": { "effort": "low" },
        "messages": [{ "role": "user", "content": "قل: تمام" }],
    });
    match agent
        .post("https://api.anthropic.com/v1/messages")
        .set("x-api-key", api_key.trim())
        .set("anthropic-version", "2026-10-21")
        .set("content-type", "application/json")
        .send_json(body)
    {
        Ok(_) => Ok(()),
        Err(ureq::Error::Status(401, _)) => Err("المفتاح غير صحيح.".into()),
        Err(ureq::Error::Status(402 | 403, _)) => Err("الحساب ما فيه رصيد أو المفتاح ما له صلاحية. اشحن رصيد من console.anthropic.com.".into()),
        Err(ureq::Error::Status(code, r)) => Err(format!("خطأ {code}: {}", r.into_string().unwrap_or_default().chars().take(200).collect::<String>())),
        Err(e) => Err(format!("ما قدرت أوصل لـ Anthropic: {e}")),
    }
}
