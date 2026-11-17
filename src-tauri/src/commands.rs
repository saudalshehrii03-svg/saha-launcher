use std::collections::HashMap;

use base64::Engine;
use tauri::{AppHandle, Emitter};

use crate::art::{self, Art};
use crate::error::{msg, Error, Result};
use crate::library::{self, models::Library};
use crate::mods::{self, nexus, InstalledMod};
use crate::online::{self, Details, StoreItem};
use crate::settings::{self, Settings};
use crate::tracker;

#[derive(serde::Serialize)]
pub struct AppInfo {
    name: &'static str,
    version: &'static str,
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo { name: "Saha", version: env!("CARGO_PKG_VERSION") }
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Result<T> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| Error::Msg(e.to_string()))
}

#[tauri::command]
pub async fn scan_library() -> Result<Library> {
    blocking(library::scan).await
}

/// Look up covers for games that have none yet (network). Returns the full art map.
#[tauri::command]
pub async fn fetch_art() -> Result<HashMap<String, Art>> {
    blocking(|| art::fetch_missing(&library::scan().games)).await
}

/// Refresh every owned Steam game (Web API). Returns the number of accounts synced.
#[tauri::command]
pub async fn sync_owned() -> Result<usize> {
    blocking(|| {
        let accounts: Vec<_> = library::scan().accounts.into_iter().filter(|a| a.platform == library::models::PlatformId::Steam).collect();
        online::sync_owned(&accounts)
    })
    .await?
}

#[tauri::command]
pub async fn game_details(title: String, steam_app_id: Option<u64>) -> Result<Details> {
    blocking(move || online::details(&title, steam_app_id)).await
}

#[tauri::command]
pub async fn store_featured() -> Result<HashMap<String, Vec<StoreItem>>> {
    blocking(online::featured).await
}

#[tauri::command]
pub async fn store_search(term: String) -> Result<Vec<StoreItem>> {
    blocking(move || online::search(&term)).await
}

#[derive(Clone, serde::Serialize)]
struct Progress {
    key: String,
    message: String,
}

/// Launch a game; may switch Steam accounts first. Emits `launch-progress`,
/// then tracks the session (`game-state` events).
#[tauri::command]
pub async fn launch_game(app: AppHandle, key: String) -> Result<()> {
    blocking(move || {
        let progress = |m: &str| {
            let _ = app.emit("launch-progress", Progress { key: key.clone(), message: m.into() });
        };
        let game = library::launch(&key, &progress)?;
        if game.installed {
            tracker::watch(app.clone(), game);
        }
        Ok(())
    })
    .await?
}

#[tauri::command]
pub fn get_settings() -> Settings {
    settings::load()
}

#[tauri::command]
pub fn save_settings(value: Settings) -> Result<()> {
    settings::save(&value)
}

/// Save an uploaded image (base64, GIF/WebP animations kept as-is) into the media dir.
/// Returns its absolute path (shown through the asset protocol).
#[tauri::command]
pub fn save_media(name: String, data: String) -> Result<String> {
    let Some(dir) = settings::dir().map(|d| d.join("media")) else { return msg("مجلد الإعدادات غير جاهز.") };
    std::fs::create_dir_all(&dir)?;
    let bytes = base64::engine::general_purpose::STANDARD.decode(data.split(',').next_back().unwrap_or(""))
        .map_err(|e| Error::Msg(e.to_string()))?;
    let ext = std::path::Path::new(&name).extension().and_then(|e| e.to_str()).unwrap_or("png").to_lowercase();
    if !["png", "jpg", "jpeg", "gif", "webp", "avif", "apng"].contains(&ext.as_str()) {
        return msg("نوع الصورة غير مدعوم.");
    }
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    let path = dir.join(format!("{stamp}.{ext}"));
    std::fs::write(&path, bytes)?;
    Ok(path.to_string_lossy().to_string())
}

// ---------- mods ----------

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModSupport {
    domain: Option<String>,
    /// Nexus game name the mods belong to (shown so a wrong match is obvious).
    nexus_name: Option<String>,
    blocked: Option<&'static str>,
    warning: Option<&'static str>,
}

#[tauri::command]
pub async fn mod_support(title: String) -> Result<ModSupport> {
    blocking(move || {
        let found = nexus::game_domain(&title);
        ModSupport {
            blocked: mods::blocked(&title),
            warning: mods::warning(&title),
            nexus_name: found.as_ref().map(|f| f.1.clone()),
            domain: found.map(|f| f.0),
        }
    })
    .await
}

#[tauri::command]
pub async fn mods_browse(domain: String, term: String, sort: String) -> Result<Vec<nexus::ModInfo>> {
    blocking(move || nexus::browse(&domain, &term, &sort)).await?
}

#[tauri::command]
pub async fn mod_page(domain: String, mod_id: u64) -> Result<nexus::ModInfo> {
    blocking(move || nexus::mod_page(&domain, mod_id)).await?
}

#[tauri::command]
pub fn mods_installed() -> Vec<InstalledMod> {
    mods::list()
}

/// Install from an nxm:// link (free accounts) or directly by mod id (premium).
#[tauri::command]
pub async fn mod_install(app: AppHandle, game_key: String, link: Option<String>, domain: Option<String>, mod_id: Option<u64>) -> Result<InstalledMod> {
    blocking(move || {
        let progress = |m: &str| {
            let _ = app.emit("mod-progress", Progress { key: game_key.clone(), message: m.into() });
        };
        let file = match (link, domain, mod_id) {
            (Some(l), _, _) => nexus::parse_nxm(&l).ok_or_else(|| Error::Msg("رابط nxm غير صالح.".into()))?,
            (None, Some(d), Some(id)) => nexus::FileRef { file_id: nexus::main_file(&d, id)?, domain: d, mod_id: id, key: None, expires: None },
            _ => return msg("بيانات المود ناقصة."),
        };
        let lib = library::scan();
        let Some(game) = lib.games.iter().find(|g| g.key == game_key) else { return msg("اللعبة غير موجودة.") };
        mods::install(game, &file, &progress)
    })
    .await?
}

#[tauri::command]
pub async fn mod_set_enabled(id: String, enabled: bool) -> Result<InstalledMod> {
    blocking(move || mods::set_enabled(&id, enabled)).await?
}

#[tauri::command]
pub async fn mod_remove(id: String) -> Result<()> {
    blocking(move || mods::remove(&id)).await?
}

#[tauri::command]
pub async fn storage_report() -> Result<crate::storage::Report> {
    blocking(crate::storage::report).await
}

/// Show a game folder in Explorer (existing folders only).
#[tauri::command]
pub fn open_folder(path: String) -> Result<()> {
    if !std::path::Path::new(&path).is_dir() {
        return msg("المجلد غير موجود.");
    }
    std::process::Command::new("explorer.exe").arg(&path).spawn()?;
    Ok(())
}

/// Open a web page in the default browser (store deals, Nexus pages).
#[tauri::command]
pub fn open_external(url: String) -> Result<()> {
    if !(url.starts_with("https://") || url.starts_with("steam://")) {
        return msg("رابط غير مسموح.");
    }
    crate::sys::open_url(&url)?;
    Ok(())
}

/// Arabic "what it does" + one explainer video for a mod page.
#[tauri::command]
pub async fn mod_explain(name: String, game: String, summary: String, description: String) -> Result<mods::explain::Explain> {
    blocking(move || mods::explain::explain(&name, &game, &summary, &description)).await
}

fn installed_dir(key: &str) -> Result<(String, String)> {
    let lib = library::scan();
    let Some(g) = lib.games.into_iter().find(|g| g.key == key && g.installed) else { return msg("اللعبة غير مثبتة.") };
    let Some(dir) = g.install_dir else { return msg("مسار اللعبة غير معروف.") };
    Ok((g.key, dir))
}

/// Mods the user installed by hand (detected automatically).
#[tauri::command]
pub async fn mods_external(game_key: String) -> Result<Vec<mods::detect::ExternalMod>> {
    blocking(move || {
        let (key, dir) = installed_dir(&game_key)?;
        Ok(mods::detect::detect(&key, &dir, &mods::saha_files(&key, &dir)))
    })
    .await?
}

#[tauri::command]
pub async fn mod_external_toggle(game_key: String, id: String, enabled: bool) -> Result<()> {
    blocking(move || {
        let (key, dir) = installed_dir(&game_key)?;
        mods::detect::set_enabled(&key, &dir, &id, enabled, &mods::saha_files(&key, &dir))
    })
    .await?
}

/// Per game: (enabled, total) hand-installed + Saha mods, for badges in the library.
#[tauri::command]
pub async fn mods_counts() -> Result<HashMap<String, (usize, usize)>> {
    blocking(|| {
        let lib = library::scan();
        let saha = mods::list();
        lib.games
            .iter()
            .filter(|g| g.installed)
            .filter_map(|g| {
                let dir = g.install_dir.as_deref()?;
                let ext = mods::detect::detect(&g.key, dir, &mods::saha_files(&g.key, dir));
                let own: Vec<_> = saha.iter().filter(|m| m.game_key == g.key).collect();
                let total = ext.len() + own.len();
                let on = ext.iter().filter(|m| m.enabled).count() + own.iter().filter(|m| m.enabled).count();
                (total > 0).then(|| (g.key.clone(), (on, total)))
            })
            .collect()
    })
    .await
}

/// Arabic description of a game (Claude when a key is set, else machine translation).
#[tauri::command]
pub async fn game_text(title: String, description: String) -> Result<mods::explain::GameText> {
    blocking(move || mods::explain::game_text(&title, &description)).await
}

#[tauri::command]
pub async fn ai_test_key(key: String) -> Result<()> {
    blocking(move || crate::ai::test_key(&key).map_err(Error::Msg)).await?
}

/// Open a Nexus Mods page in a window inside Saha. Free accounts must click
/// "Mod Manager Download" on the site (Nexus rule); that nxm:// link is caught here
/// and sent to the main window to install, so the user never leaves the app.
/// The window loads a remote site, so it gets no IPC access (capabilities cover "main" only).
#[tauri::command]
pub fn open_nexus(app: AppHandle, url: String) -> Result<()> {
    use tauri::Manager;
    if !url.starts_with("https://www.nexusmods.com/") {
        return msg("رابط غير مسموح.");
    }
    let parsed: tauri::Url = url.parse().map_err(|_| Error::Msg("رابط غير صالح.".into()))?;
    if let Some(w) = app.get_webview_window("nexus") {
        let _ = w.navigate(parsed);
        let _ = w.unminimize();
        let _ = w.set_focus();
        return Ok(());
    }
    let handle = app.clone();
    tauri::WebviewWindowBuilder::new(&app, "nexus", tauri::WebviewUrl::External(parsed))
        .title("Nexus Mods · ساحة")
        .inner_size(1180.0, 860.0)
        .on_navigation(move |u| {
            if u.scheme() == "nxm" {
                let _ = handle.emit_to("main", "nxm-link", u.to_string());
                if let Some(w) = handle.get_webview_window("nexus") {
                    let _ = w.close();
                }
                return false;
            }
            matches!(u.scheme(), "https" | "about")
        })
        .build()
        .map_err(|e| Error::Msg(e.to_string()))?;
    Ok(())
}
