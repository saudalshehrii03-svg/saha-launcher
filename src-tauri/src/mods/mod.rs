//! Mods: browse (Nexus), one-click install, enable/disable, clean delete.
//!
//! Layout (app config dir):
//!   mods/<game>/<mod>-<file>/files/...   extracted mod (staging, kept while installed)
//!   mods/<game>/<mod>-<file>/backup/...  game files we overwrote (restored on disable)
//!   mods.json                            registry of installed mods
//! Enabling copies staging -> game folder; disabling removes exactly those files and
//! restores the backups, so the game is back to its original state.
//! Online-only games are blocked to avoid bans.
pub mod nexus;

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{msg, Error, Result};
use crate::library::models::Game;
use crate::online::agent;
use crate::settings;

const FILE: &str = "mods.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledMod {
    pub id: String,
    pub game_key: String,
    pub domain: String,
    pub mod_id: u64,
    pub file_id: u64,
    pub name: String,
    pub version: String,
    pub picture: Option<String>,
    pub enabled: bool,
    /// Where the files were deployed (absolute).
    pub target: String,
    /// Deployed files, relative to `target`.
    pub files: Vec<String>,
    /// Files that existed before and were backed up, relative to `target`.
    pub backups: Vec<String>,
    pub installed_at: u64,
}

/// Titles where mods risk an account ban (online-only / anti-cheat).
pub fn blocked(title: &str) -> Option<&'static str> {
    let t = title.to_lowercase();
    const ONLINE: &[&str] = &[
        "call of duty", "warzone", "counter-strike", "fortnite", "rocket league", "valorant", "apex legends", "pubg",
        "roblox", "overwatch", "rainbow six", "the finals", "marvel rivals",
    ];
    ONLINE.iter().any(|o| t.contains(o)).then_some("لعبة أونلاين: المودات ممنوعة فيها عشان ما ينحظر حسابك.")
}

pub fn warning(title: &str) -> Option<&'static str> {
    let t = title.to_lowercase();
    (t.contains("grand theft auto") || t.contains("red dead")).then_some("المودات للعب الفردي فقط. لا تدخل الأونلاين والمودات مفعّلة.")
}

pub fn list() -> Vec<InstalledMod> {
    settings::read_json(FILE)
}

fn save(all: &[InstalledMod]) -> Result<()> {
    settings::write_json(FILE, &all)
}

fn root() -> Result<PathBuf> {
    settings::dir().map(|d| d.join("mods")).ok_or_else(|| Error::Msg("مجلد الإعدادات غير جاهز.".into()))
}

fn safe(s: &str) -> String {
    s.chars().map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' }).collect()
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

// ---------- archives ----------

fn extract(archive: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    let name = archive.to_string_lossy().to_lowercase();
    if name.ends_with(".zip") {
        let mut z = zip::ZipArchive::new(fs::File::open(archive)?).map_err(|e| Error::Msg(format!("zip: {e}")))?;
        for i in 0..z.len() {
            let mut f = z.by_index(i).map_err(|e| Error::Msg(format!("zip: {e}")))?;
            let Some(rel) = f.enclosed_name() else { continue }; // blocks ../ paths
            let out = dest.join(rel);
            if f.is_dir() {
                fs::create_dir_all(&out)?;
            } else {
                if let Some(p) = out.parent() {
                    fs::create_dir_all(p)?;
                }
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;
                fs::write(out, buf)?;
            }
        }
        Ok(())
    } else if name.ends_with(".7z") {
        sevenz_rust::decompress_file(archive, dest).map_err(|e| Error::Msg(format!("7z: {e}")))
    } else if name.ends_with(".rar") {
        msg("ملفات RAR غير مدعومة بعد. اختر نسخة zip أو 7z من صفحة المود إن وجدت.")
    } else {
        // Single file (e.g. .asi / .dll / .pak): keep as is.
        let file = archive.file_name().ok_or_else(|| Error::Msg("اسم ملف غير صالح".into()))?;
        fs::copy(archive, dest.join(file))?;
        Ok(())
    }
}

fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) {
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, base, out);
            } else if let Ok(rel) = p.strip_prefix(base) {
                out.push(rel.to_string_lossy().to_string());
            }
        }
    }
}

fn entries(dir: &Path) -> Vec<(String, bool)> {
    fs::read_dir(dir)
        .map(|rd| rd.flatten().map(|e| (e.file_name().to_string_lossy().to_string(), e.path().is_dir())).collect())
        .unwrap_or_default()
}

fn exists_ci(dir: &Path, name: &str) -> bool {
    entries(dir).iter().any(|(n, _)| n.eq_ignore_ascii_case(name))
}

/// Pick (source folder inside the mod, target folder in the game).
/// Strips wrapper folders, then matches the mod's top level against the game's folders.
fn plan(staging: &Path, game_dir: &Path, title: &str) -> (PathBuf, PathBuf) {
    let mut src = staging.to_path_buf();
    for _ in 0..4 {
        let e = entries(&src);
        if e.iter().any(|(n, _)| exists_ci(game_dir, n)) {
            return (src, game_dir.to_path_buf());
        }
        match e.as_slice() {
            [(only, true)] => src = src.join(only),
            _ => break,
        }
    }
    let t = title.to_lowercase();
    let e = entries(&src);
    let all_ext = |ext: &str| !e.is_empty() && e.iter().all(|(n, d)| !d && n.to_lowercase().ends_with(ext));
    let sub = if t.contains("cyberpunk") && all_ext(".archive") {
        "archive/pc/mod"
    } else if t.contains("witcher 3") {
        if e.iter().any(|(n, _)| n.to_lowercase().starts_with("mod")) { "Mods" } else { "" }
    } else if t.contains("hollow knight") {
        "hollow_knight_Data/Managed/Mods"
    } else if t.contains("stardew") {
        "Mods"
    } else if t.contains("skyrim") || t.contains("fallout") || t.contains("oblivion") || t.contains("starfield") {
        "Data"
    } else if t.contains("elden ring") || t.contains("sekiro") {
        "mods"
    } else if t.contains("baldur") {
        "Mods"
    } else {
        ""
    };
    (src, if sub.is_empty() { game_dir.to_path_buf() } else { game_dir.join(sub) })
}

// ---------- deploy / undeploy ----------

fn mod_dir(m: &InstalledMod) -> Result<PathBuf> {
    Ok(root()?.join(safe(&m.game_key)).join(&m.id))
}

fn deploy(m: &mut InstalledMod) -> Result<()> {
    let dir = mod_dir(m)?;
    let files_dir = dir.join("files");
    let target = PathBuf::from(&m.target);
    m.backups.clear();
    for rel in &m.files {
        let dst = target.join(rel);
        if dst.exists() {
            let bak = dir.join("backup").join(rel);
            if let Some(p) = bak.parent() {
                fs::create_dir_all(p)?;
            }
            fs::copy(&dst, &bak)?;
            m.backups.push(rel.clone());
        }
        if let Some(p) = dst.parent() {
            fs::create_dir_all(p)?;
        }
        fs::copy(files_dir.join(rel), &dst)?;
    }
    m.enabled = true;
    Ok(())
}

fn undeploy(m: &mut InstalledMod) -> Result<()> {
    let dir = mod_dir(m)?;
    let target = PathBuf::from(&m.target);
    for rel in &m.files {
        let dst = target.join(rel);
        if m.backups.contains(rel) {
            fs::copy(dir.join("backup").join(rel), &dst)?;
        } else {
            let _ = fs::remove_file(&dst);
            // remove now-empty folders we created
            let mut p = dst.parent();
            while let Some(d) = p {
                if d == target || fs::remove_dir(d).is_err() {
                    break;
                }
                p = d.parent();
            }
        }
    }
    let _ = fs::remove_dir_all(dir.join("backup"));
    m.backups.clear();
    m.enabled = false;
    Ok(())
}

/// Disable mods first (other mods may have backed up our files) — keep it simple:
/// only allow toggling when no other enabled mod touches the same files.
fn conflicts(all: &[InstalledMod], m: &InstalledMod) -> Option<String> {
    all.iter()
        .filter(|o| o.id != m.id && o.game_key == m.game_key && o.enabled && o.target == m.target)
        .find(|o| o.files.iter().any(|f| m.files.contains(f)))
        .map(|o| o.name.clone())
}

pub fn set_enabled(id: &str, enabled: bool) -> Result<InstalledMod> {
    let mut all = list();
    let idx = all.iter().position(|m| m.id == id).ok_or_else(|| Error::Msg("المود غير موجود.".into()))?;
    let mut m = all[idx].clone();
    if enabled && !m.enabled {
        if let Some(other) = conflicts(&all, &m) {
            return msg(format!("يتعارض مع «{other}» (نفس الملفات). عطّله أول."));
        }
        deploy(&mut m)?;
    } else if !enabled && m.enabled {
        if let Some(other) = conflicts(&all, &m) {
            return msg(format!("عطّل «{other}» أول، لأنه ركّب فوق ملفات هذا المود."));
        }
        undeploy(&mut m)?;
    }
    all[idx] = m.clone();
    save(&all)?;
    Ok(m)
}

pub fn remove(id: &str) -> Result<()> {
    let mut all = list();
    let Some(idx) = all.iter().position(|m| m.id == id) else { return Ok(()) };
    if all[idx].enabled {
        set_enabled(id, false)?;
        all = list();
    }
    let m = all.remove(idx);
    let _ = fs::remove_dir_all(mod_dir(&m)?);
    save(&all)
}

/// Download a Nexus file and install it into `game`.
pub fn install(game: &Game, f: &nexus::FileRef, progress: &dyn Fn(&str)) -> Result<InstalledMod> {
    if let Some(why) = blocked(&game.title) {
        return msg(why);
    }
    let Some(game_dir) = game.install_dir.as_deref().filter(|_| game.installed).map(PathBuf::from) else {
        return msg("ثبّت اللعبة أول عشان تقدر تركّب مودات.");
    };
    let id = format!("{}-{}", f.mod_id, f.file_id);
    if list().iter().any(|m| m.id == id && m.game_key == game.key) {
        return msg("المود مثبت من قبل.");
    }
    progress("جلب رابط التنزيل من Nexus…");
    let (url, file_name, version) = nexus::download_link(f)?;
    let info = nexus::mod_page(&f.domain, f.mod_id).ok();

    let dir = root()?.join(safe(&game.key)).join(&id);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;
    let base = Path::new(&file_name).file_name().map(|f| f.to_os_string()).unwrap_or_else(|| "mod.zip".into());
    let archive = dir.join(base);

    progress(&format!("تنزيل {file_name}…"));
    let resp = agent().get(&url).call().map_err(|e| Error::Msg(format!("فشل التنزيل: {e}")))?;
    let total: u64 = resp.header("Content-Length").and_then(|s| s.parse().ok()).unwrap_or(0);
    let mut reader = resp.into_reader();
    let mut out = fs::File::create(&archive)?;
    let mut buf = vec![0u8; 1 << 16];
    let (mut done, mut last) = (0u64, 0u64);
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        out.write_all(&buf[..n])?;
        done += n as u64;
        if total > 0 && done * 100 / total >= last + 10 {
            last = done * 100 / total;
            progress(&format!("تنزيل… {last}%"));
        }
    }
    drop(out);

    progress("فك الضغط…");
    let staging_raw = dir.join("raw");
    extract(&archive, &staging_raw)?;
    let _ = fs::remove_file(&archive);

    let (src, target) = plan(&staging_raw, &game_dir, &game.title);
    let files_dir = dir.join("files");
    fs::rename(&src, &files_dir).or_else(|_| copy_dir(&src, &files_dir))?;
    let _ = fs::remove_dir_all(&staging_raw);
    let mut files = Vec::new();
    walk(&files_dir, &files_dir, &mut files);
    if files.is_empty() {
        return msg("الأرشيف فاضي.");
    }

    let mut m = InstalledMod {
        id,
        game_key: game.key.clone(),
        domain: f.domain.clone(),
        mod_id: f.mod_id,
        file_id: f.file_id,
        name: info.as_ref().map(|i| i.name.clone()).unwrap_or(file_name),
        version: if version.is_empty() { info.as_ref().map(|i| i.version.clone()).unwrap_or_default() } else { version },
        picture: info.and_then(|i| i.thumbnail),
        enabled: false,
        target: target.to_string_lossy().to_string(),
        files,
        backups: Vec::new(),
        installed_at: now(),
    };
    let mut all = list();
    if conflicts(&all, &m).is_none() {
        progress("تركيب الملفات في مجلد اللعبة…");
        deploy(&mut m)?;
    }
    all.push(m.clone());
    save(&all)?;
    Ok(m)
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::create_dir_all(to)?;
    for e in fs::read_dir(from)?.flatten() {
        let p = e.path();
        if p.is_dir() {
            copy_dir(&p, &to.join(e.file_name()))?;
        } else {
            fs::copy(&p, to.join(e.file_name()))?;
        }
    }
    Ok(())
}
