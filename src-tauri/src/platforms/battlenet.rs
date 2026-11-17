//! Battle.net.
//! - installed games: uninstall entries published by Blizzard whose UninstallString has
//!   `--uid=<uid>` (the Battle.net app itself is skipped)
//! - launch: Battle.net.exe --exec="launch_uid <uid>"
use std::path::PathBuf;

use crate::error::{msg, Result};
use crate::library::models::{Game, PlatformId};
use crate::sys;

use super::{Platform, Progress};

pub struct BattleNet;

fn client_dir() -> Option<PathBuf> {
    sys::uninstall_entries()
        .into_iter()
        .find(|u| u.name == "Battle.net")
        .map(|u| PathBuf::from(u.location))
        .or_else(|| Some(PathBuf::from(r"C:\Program Files (x86)\Battle.net")))
        .filter(|d| d.join("Battle.net.exe").is_file())
}

fn uid(uninstall: &str) -> Option<String> {
    let start = uninstall.find("--uid=")? + 6;
    let rest = &uninstall[start..];
    let end = rest.find([' ', '"']).unwrap_or(rest.len());
    Some(rest[..end].to_string()).filter(|s| !s.is_empty())
}

impl Platform for BattleNet {
    fn id(&self) -> PlatformId {
        PlatformId::Battlenet
    }
    fn name(&self) -> &'static str {
        "Battle.net"
    }
    fn install_path(&self) -> Option<String> {
        client_dir().map(|d| d.to_string_lossy().to_string())
    }
    fn scan_games(&self) -> Result<Vec<Game>> {
        let mut games = Vec::new();
        for u in sys::uninstall_entries() {
            if !u.publisher.contains("Blizzard") || u.name == "Battle.net" {
                continue;
            }
            let Some(id) = uid(&u.uninstall) else { continue };
            let dir = Some(u.location).filter(|l| !l.is_empty());
            games.push(Game::installed(PlatformId::Battlenet, &id, &u.name, &id, dir));
        }
        Ok(games)
    }
    fn launch(&self, game: &Game, progress: Progress) -> Result<()> {
        let Some(dir) = client_dir() else { return msg("Battle.net غير مثبت.") };
        progress("تشغيل اللعبة عبر Battle.net…");
        let exec = format!("--exec=launch_uid {}", game.app_id);
        sys::spawn(&dir.join("Battle.net.exe"), &[&exec])?;
        Ok(())
    }
}
