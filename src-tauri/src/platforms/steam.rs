//! Steam.
//! - install path: HKCU\Software\Valve\Steam\SteamPath / SteamExe
//! - accounts: config/loginusers.vdf; signed-in one: HKCU\...\ActiveProcess\ActiveUser
//! - installed games: steamapps/libraryfolders.vdf + appmanifest_*.acf (LastOwner = owner)
//! - playtime / last played: userdata/<id32>/config/localconfig.vdf per account
//! - launch: switch to the right account (accounts::steam_switch), then steam://rungameid/<appid>
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::accounts::{rules, steam_switch};
use crate::error::{msg, Result};
use crate::library::models::{Account, Game, PlatformId};
use crate::settings;
use crate::sys::{self, Hive};
use crate::vdf;

use super::{Platform, Progress};

pub const REG: &str = r"Software\Valve\Steam";
const ID64_BASE: u64 = 76_561_197_960_265_728;

/// Not games: redistributables, runtimes, tools.
const SKIP_APPS: &[&str] = &["228980", "1070560", "1391110", "1628350", "1493710", "250820"];

pub struct Steam;

pub struct Install {
    pub root: PathBuf,
    pub exe: PathBuf,
}

pub fn install() -> Option<Install> {
    let root = PathBuf::from(sys::reg_str(Hive::CurrentUser, REG, "SteamPath")?);
    if !root.is_dir() {
        return None;
    }
    let exe = sys::reg_str(Hive::CurrentUser, REG, "SteamExe")
        .map(PathBuf::from)
        .filter(|p| p.is_file())
        .unwrap_or_else(|| root.join("steam.exe"));
    Some(Install { root, exe })
}

pub fn id32(id64: &str) -> Option<u32> {
    id64.parse::<u64>().ok()?.checked_sub(ID64_BASE).map(|v| v as u32)
}

/// SteamID32 of the signed-in account, if Steam is running and signed in.
pub fn active_user32() -> Option<u32> {
    sys::reg_u32(Hive::CurrentUser, &format!(r"{REG}\ActiveProcess"), "ActiveUser").filter(|v| *v != 0)
}

pub fn read_accounts(inst: &Install) -> Vec<Account> {
    let active = active_user32();
    let Some(root) = vdf::read(&inst.root.join("config/loginusers.vdf")) else {
        return Vec::new();
    };
    let Some(users) = root.get("users") else {
        return Vec::new();
    };
    users
        .entries()
        .iter()
        .map(|(id, u)| Account {
            platform: PlatformId::Steam,
            id: id.clone(),
            display_name: u.str("PersonaName").or(u.str("AccountName")).unwrap_or(id).to_string(),
            login_name: u.str("AccountName").map(str::to_string),
            active: active.is_some() && id32(id) == active,
        })
        .collect()
}

fn libraries(inst: &Install) -> Vec<PathBuf> {
    let mut libs = vec![inst.root.clone()];
    if let Some(v) = vdf::read(&inst.root.join("steamapps/libraryfolders.vdf")) {
        if let Some(f) = v.get("libraryfolders") {
            for (_, lib) in f.entries() {
                if let Some(p) = lib.str("path") {
                    let p = PathBuf::from(p);
                    if !libs.iter().any(|l| same_path(l, &p)) {
                        libs.push(p);
                    }
                }
            }
        }
    }
    libs
}

fn same_path(a: &std::path::Path, b: &std::path::Path) -> bool {
    let n = |p: &std::path::Path| p.to_string_lossy().replace('\\', "/").trim_end_matches('/').to_lowercase();
    n(a) == n(b)
}

/// appid -> (playtime minutes summed over accounts, latest last-played).
fn play_stats(inst: &Install, accounts: &[Account]) -> HashMap<String, (u64, u64)> {
    let mut out: HashMap<String, (u64, u64)> = HashMap::new();
    for a in accounts {
        let Some(id) = id32(&a.id) else { continue };
        let path = inst.root.join(format!("userdata/{id}/config/localconfig.vdf"));
        let Some(v) = vdf::read(&path) else { continue };
        let Some(apps) = v.path(&["UserLocalConfigStore", "Software", "Valve", "Steam", "apps"]) else {
            continue;
        };
        for (appid, app) in apps.entries() {
            let num = |k: &str| app.str(k).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
            let e = out.entry(appid.clone()).or_default();
            e.0 += num("Playtime");
            e.1 = e.1.max(num("LastPlayed"));
        }
    }
    out
}

pub fn scan(inst: &Install, accounts: &[Account]) -> Vec<Game> {
    let settings = settings::load();
    let stats = play_stats(inst, accounts);
    let mut games: Vec<Game> = Vec::new();
    for lib in libraries(inst) {
        let Ok(dir) = std::fs::read_dir(lib.join("steamapps")) else { continue };
        for entry in dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !(name.starts_with("appmanifest_") && name.ends_with(".acf")) {
                continue;
            }
            let Some(root) = vdf::read(&entry.path()) else { continue };
            let Some(m) = root.get("AppState") else { continue };
            let (Some(appid), Some(title)) = (m.str("appid"), m.str("name")) else { continue };
            if SKIP_APPS.contains(&appid) || title.contains("Redistributable") || title.starts_with("Proton") {
                continue;
            }
            if games.iter().any(|g| g.app_id == appid) {
                continue;
            }
            let flags: u32 = m.str("StateFlags").and_then(|s| s.parse().ok()).unwrap_or(0);
            let owner = m
                .str("LastOwner")
                .filter(|o| accounts.iter().any(|a| a.id == *o))
                .map(str::to_string);
            let stat = stats.get(appid).copied().unwrap_or_default();
            let game = Game {
                key: format!("steam:{appid}"),
                title: title.to_string(),
                platform: PlatformId::Steam,
                app_id: appid.to_string(),
                owner_account_id: owner,
                launch_account_id: None,
                guarded: rules::is_cod(title),
                installed: flags & 4 != 0,
                install_dir: m
                    .str("installdir")
                    .map(|d| lib.join("steamapps/common").join(d).to_string_lossy().to_string()),
                size_bytes: m.str("SizeOnDisk").and_then(|s| s.parse().ok()),
                playtime_minutes: (stat.0 > 0).then_some(stat.0),
                last_played: (stat.1 > 0).then_some(stat.1),
            };
            games.push(game);
        }
    }

    // Owned but not installed (Steam Web API cache).
    let owned = crate::online::owned_cached();
    for acc in accounts {
        for o in owned.get(&acc.id).into_iter().flatten() {
            let appid = o.app_id.to_string();
            if let Some(g) = games.iter_mut().find(|g| g.app_id == appid) {
                g.playtime_minutes = g.playtime_minutes.max(Some(o.playtime).filter(|p| *p > 0));
                g.last_played = g.last_played.max(Some(o.last_played).filter(|p| *p > 0));
                continue;
            }
            if SKIP_APPS.contains(&appid.as_str()) {
                continue;
            }
            games.push(Game {
                key: format!("steam:{appid}"),
                title: o.name.clone(),
                platform: PlatformId::Steam,
                app_id: appid,
                owner_account_id: Some(acc.id.clone()),
                launch_account_id: None,
                guarded: rules::is_cod(&o.name),
                installed: false,
                install_dir: None,
                size_bytes: None,
                playtime_minutes: Some(o.playtime).filter(|p| *p > 0),
                last_played: Some(o.last_played).filter(|p| *p > 0),
            });
        }
    }

    for g in &mut games {
        g.launch_account_id = rules::resolve(g, &settings, accounts).ok().flatten().map(|a| a.id.clone());
    }
    games
}

impl Platform for Steam {
    fn id(&self) -> PlatformId {
        PlatformId::Steam
    }
    fn name(&self) -> &'static str {
        "Steam"
    }
    fn install_path(&self) -> Option<String> {
        install().map(|i| i.root.to_string_lossy().to_string())
    }
    fn accounts(&self) -> Vec<Account> {
        install().map(|i| read_accounts(&i)).unwrap_or_default()
    }
    fn scan_games(&self) -> Result<Vec<Game>> {
        Ok(install().map(|i| scan(&i, &read_accounts(&i))).unwrap_or_default())
    }
    /// Steam windows (store pop-ups, account switch, "preparing to launch") stay
    /// hidden until the game is running; if it never starts they come back.
    fn launch(&self, game: &Game, progress: Progress) -> Result<()> {
        if !game.installed {
            return self.open_install(game, progress);
        }
        const RUNNING: u8 = 0;
        const STARTED: u8 = 1; // keep hiding late pop-ups for a while, then stop
        const FAILED: u8 = 2;
        let state = Arc::new(AtomicU8::new(RUNNING));
        let s = state.clone();
        let hider = std::thread::spawn(move || {
            let mut h = sys::WindowHider::default();
            let mut started_at: Option<Instant> = None;
            loop {
                match s.load(Ordering::Relaxed) {
                    FAILED => return h,
                    STARTED => {
                        if started_at.get_or_insert_with(Instant::now).elapsed() > Duration::from_secs(25) {
                            return h;
                        }
                    }
                    _ => {}
                }
                h.sweep(&["steam.exe", "steamwebhelper.exe"]);
                std::thread::sleep(Duration::from_millis(200));
            }
        });

        match self.launch_hidden(game, progress) {
            Ok(true) => {
                state.store(STARTED, Ordering::Relaxed);
                progress("اللعبة شغّالة");
                Ok(())
            }
            other => {
                state.store(FAILED, Ordering::Relaxed);
                if let Ok(h) = hider.join() {
                    h.restore();
                }
                match other {
                    Ok(_) => msg("ستيم ما أكّد تشغيل اللعبة خلال دقيقة. أظهرت نوافذه لو كان يطلب منك شي."),
                    Err(e) => Err(e),
                }
            }
        }
    }
}

impl Steam {
    /// Not installed: switch to the owning account, then open Steam's install dialog (visible).
    fn open_install(&self, game: &Game, progress: Progress) -> Result<()> {
        let Some(inst) = install() else { return msg("ستيم غير مثبت على الجهاز.") };
        let accounts = read_accounts(&inst);
        if let Some(acc) = rules::resolve(game, &settings::load(), &accounts)? {
            steam_switch::ensure_active(&inst, acc, progress)?;
        }
        progress("فتح نافذة التثبيت في ستيم…");
        sys::spawn(&inst.exe, &[&format!("steam://install/{}", game.app_id)])?;
        Ok(())
    }

    /// Ok(true) = game confirmed running.
    fn launch_hidden(&self, game: &Game, progress: Progress) -> Result<bool> {
        let Some(inst) = install() else { return msg("ستيم غير مثبت على الجهاز.") };
        let accounts = read_accounts(&inst);
        let settings = settings::load();
        let target = rules::resolve(game, &settings, &accounts)?;

        if let Some(acc) = target {
            steam_switch::ensure_active(&inst, acc, progress)?;
        }

        // Final guard right before launch: re-read who is actually signed in.
        if rules::is_cod(&game.title) {
            let active = active_user32();
            let ok = target.is_some_and(|t| id32(&t.id) == active)
                && !settings.cod_banned_accounts.iter().any(|b| id32(b) == active);
            if !ok {
                return msg("أُلغي التشغيل: الحساب المسجّل في ستيم ليس الحساب الآمن لـ Call of Duty.");
            }
        }

        progress("تشغيل اللعبة…");
        sys::spawn(&inst.exe, &["-silent", &format!("steam://rungameid/{}", game.app_id)])?;

        // Steam writes the running game's id here once it has started.
        let want: u32 = game.app_id.parse().unwrap_or(0);
        let deadline = Instant::now() + Duration::from_secs(60);
        while Instant::now() < deadline {
            if sys::reg_u32(Hive::CurrentUser, REG, "RunningAppID") == Some(want) {
                return Ok(true);
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        Ok(false)
    }
}
