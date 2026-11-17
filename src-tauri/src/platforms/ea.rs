//! EA app.
//! - installed games: uninstall entries published by Electronic Arts whose folder has
//!   `__Installer\installerdata.xml`; the offer id is its first <contentID>
//! - launch: origin2://game/launch?offerIds=<id>&autoDownload=1 (handled by the EA app)
use std::path::PathBuf;

use crate::error::{msg, Result};
use crate::library::models::{Game, PlatformId};
use crate::sys::{self, Hive};

use super::{Platform, Progress};

pub struct Ea;

fn content_id(dir: &str) -> Option<String> {
    let xml = std::fs::read_to_string(PathBuf::from(dir).join(r"__Installer\installerdata.xml")).ok()?;
    let start = xml.find("<contentID>")? + "<contentID>".len();
    let end = xml[start..].find("</contentID>")? + start;
    Some(xml[start..end].trim().to_string()).filter(|s| !s.is_empty())
}

impl Platform for Ea {
    fn id(&self) -> PlatformId {
        PlatformId::Ea
    }
    fn name(&self) -> &'static str {
        "EA app"
    }
    fn install_path(&self) -> Option<String> {
        sys::reg_str(Hive::LocalMachine, r"SOFTWARE\Electronic Arts\EA Desktop", "InstallLocation")
            .or_else(|| Some(r"C:\Program Files\Electronic Arts\EA Desktop".into()))
            .filter(|d| PathBuf::from(d).is_dir())
    }
    fn scan_games(&self) -> Result<Vec<Game>> {
        let mut games = Vec::new();
        for u in sys::uninstall_entries() {
            if !u.publisher.contains("Electronic Arts") || u.location.is_empty() || u.name.starts_with("EA app") {
                continue;
            }
            let Some(id) = content_id(&u.location) else { continue };
            if games.iter().any(|g: &Game| g.app_id == id) {
                continue;
            }
            games.push(Game::installed(PlatformId::Ea, &id, &u.name, &id, Some(u.location)));
        }
        Ok(games)
    }
    fn launch(&self, game: &Game, progress: Progress) -> Result<()> {
        if self.install_path().is_none() {
            return msg("EA app غير مثبت.");
        }
        progress("تشغيل اللعبة عبر EA app…");
        sys::open_url(&format!("origin2://game/launch?offerIds={}&autoDownload=1", game.app_id))?;
        Ok(())
    }
}
