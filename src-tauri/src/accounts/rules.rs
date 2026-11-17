//! Which account a game launches on.
//!
//! Order: forced account from settings > owner from local files > whatever is signed in.
//! Safety rule (mandatory): Call of Duty never launches on an account marked
//! "banned in Call of Duty". Until at least one account is marked, CoD is refused.
use crate::error::{msg, Result};
use crate::library::models::{Account, Game};
use crate::settings::Settings;

pub fn is_cod(title: &str) -> bool {
    let t = title.to_lowercase();
    t.contains("call of duty") || t.contains("warzone")
}

/// `Ok(None)` = launch on whatever account is signed in (no switch).
pub fn resolve<'a>(game: &Game, settings: &Settings, accounts: &'a [Account]) -> Result<Option<&'a Account>> {
    let find = |id: &str| accounts.iter().find(|a| a.id == id);
    let pinned = settings.pinned_accounts.get(&game.key).and_then(|id| find(id));
    let owner = game.owner_account_id.as_deref().and_then(find);
    let target = pinned.or(owner);

    if !is_cod(&game.title) {
        return Ok(target);
    }

    let banned = &settings.cod_banned_accounts;
    if banned.is_empty() {
        return msg("قاعدة الأمان: حدّد الحساب المحظور في Call of Duty من صفحة «المنصات والحسابات» قبل التشغيل.");
    }
    if let Some(p) = pinned {
        if banned.contains(&p.id) {
            return msg(format!("الحساب المفروض لهذه اللعبة ({}) محظور في Call of Duty. غيّره من الإعدادات.", p.display_name));
        }
        return Ok(Some(p));
    }
    if let Some(o) = owner.filter(|o| !banned.contains(&o.id)) {
        return Ok(Some(o));
    }
    // Owner unknown or banned: use the only allowed account, otherwise ask.
    let mut allowed = accounts.iter().filter(|a| !banned.contains(&a.id));
    match (allowed.next(), allowed.next()) {
        (Some(a), None) => Ok(Some(a)),
        (None, _) => msg("كل حسابات ستيم معلّمة كمحظورة في Call of Duty، ما فيه حساب آمن للتشغيل."),
        _ => msg("فيه أكثر من حساب مسموح لـ Call of Duty. اختر الحساب من الإعدادات."),
    }
}
