//! Automatic Steam account switching.
//!
//! Why: Steam allows one signed-in account at a time. The user owns games on two
//! accounts (Call of Duty on a second account because the main one is banned
//! there), so Family Sharing is not an option: the game must run on the account
//! that owns it.
//!
//! Flow when the wanted account is not the signed-in one:
//! 1. Shut Steam down cleanly (`steam.exe -shutdown`) and wait for exit.
//! 2. Set HKCU\Software\Valve\Steam\AutoLoginUser = <AccountName>
//!    and mark that user MostRecent/AutoLogin in config/loginusers.vdf.
//! 3. Start Steam; it signs in with the token it already remembers.
//! 4. Wait until ActiveUser really is that account (caller launches the game after).
//!
//! Requirements: each account was signed in once with "Remember me".
//! Passwords are never read or stored.
use std::time::{Duration, Instant};

use crate::error::{msg, Result};
use crate::library::models::Account;
use crate::platforms::steam::{active_user32, id32, Install, REG};
use crate::platforms::Progress;
use crate::sys;
use crate::vdf::{self, Vdf};

fn wait_until(timeout: Duration, mut done: impl FnMut() -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if done() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(700));
    }
    done()
}

fn steam_running() -> bool {
    sys::process_running("steam.exe")
}

/// Make `acc` the signed-in Steam account (no-op if it already is).
pub fn ensure_active(inst: &Install, acc: &Account, progress: Progress) -> Result<()> {
    let Some(want) = id32(&acc.id) else { return msg("معرّف حساب ستيم غير صالح.") };
    if active_user32() == Some(want) && steam_running() {
        return Ok(());
    }
    let Some(login) = acc.login_name.as_deref() else {
        return msg(format!("ما لقيت اسم دخول الحساب {}.", acc.display_name));
    };

    if steam_running() {
        progress(&format!("إغلاق ستيم للتبديل إلى {}…", acc.display_name));
        sys::spawn(&inst.exe, &["-shutdown"])?;
        if !wait_until(Duration::from_secs(45), || !steam_running()) {
            return msg("ستيم ما قفل. اقفله يدوياً وجرّب مرة ثانية.");
        }
        std::thread::sleep(Duration::from_secs(1));
    }

    sys::reg_set_str(REG, "AutoLoginUser", login)?;
    sys::reg_set_u32(REG, "RememberPassword", 1)?;
    // Clear the stale value so we only accept a fresh sign-in below.
    let _ = sys::reg_set_u32(&format!(r"{REG}\ActiveProcess"), "ActiveUser", 0);
    mark_most_recent(inst, &acc.id)?;

    progress(&format!("تشغيل ستيم على حساب {}…", acc.display_name));
    sys::spawn(&inst.exe, &["-silent"])?; // tray only, no main window
    if !wait_until(Duration::from_secs(120), || active_user32() == Some(want)) {
        return msg(format!(
            "ستيم ما دخل على حساب {} تلقائياً. سجّل دخوله مرة من ستيم مع «تذكرني» ثم جرّب.",
            acc.display_name
        ));
    }
    progress("انتظار جاهزية ستيم…");
    std::thread::sleep(Duration::from_secs(4));
    Ok(())
}

/// Mark `id64` as the account Steam signs into next. Keeps a one-time backup.
fn mark_most_recent(inst: &Install, id64: &str) -> Result<()> {
    let path = inst.root.join("config/loginusers.vdf");
    let src = std::fs::read_to_string(&path)?;
    let backup = path.with_extension("vdf.saha.bak");
    if !backup.exists() {
        let _ = std::fs::write(&backup, &src);
    }
    let mut root = vdf::parse(&src);
    let Some(users) = root.get_mut("users").and_then(Vdf::entries_mut) else {
        return msg("ملف حسابات ستيم (loginusers.vdf) غير مفهوم.");
    };
    for (id, user) in users.iter_mut() {
        let on = if id == id64 { "1" } else { "0" };
        user.set_str("MostRecent", on);
        user.set_str("AutoLogin", on);
        if id == id64 {
            user.set_str("RememberPassword", "1");
            user.set_str("WantsOfflineMode", "0");
        }
    }
    std::fs::write(&path, vdf::to_string(&root))?;
    Ok(())
}
