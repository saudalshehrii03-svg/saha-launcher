pub mod models;

use crate::error::{msg, Result};
use crate::platforms::{self, Progress};
use crate::{art, tracker};
use models::{Game, Library, PlatformId, PlatformStatus};

/// Scan every detected platform (in parallel) into one merged library.
pub fn scan() -> Library {
    let results: Vec<_> = std::thread::scope(|s| {
        let handles: Vec<_> = platforms::all()
            .into_iter()
            .map(|p| {
                s.spawn(move || {
                    let path = p.install_path();
                    let games = if path.is_some() { p.scan_games().unwrap_or_default() } else { Vec::new() };
                    let status = PlatformStatus {
                        id: p.id(),
                        name: p.name().into(),
                        detected: path.is_some() || !games.is_empty(),
                        install_path: path,
                        game_count: games.len(),
                    };
                    (status, p.accounts(), games)
                })
            })
            .collect();
        handles.into_iter().filter_map(|h| h.join().ok()).collect()
    });

    let mut lib = Library { platforms: Vec::new(), accounts: Vec::new(), games: Vec::new(), art: art::cached() };
    for (status, accounts, games) in results {
        lib.platforms.push(status);
        lib.accounts.extend(accounts);
        lib.games.extend(games);
    }

    // Saha's own tracking: playtime for platforms that don't report it, last played for all.
    let played = tracker::load();
    for g in &mut lib.games {
        if let Some(e) = played.get(&g.key) {
            g.last_played = g.last_played.max(Some(e.last_played));
            if g.platform != PlatformId::Steam {
                g.playtime_minutes = Some(e.minutes).filter(|m| *m > 0);
            }
        }
    }
    lib.games.sort_by(|a, b| b.last_played.cmp(&a.last_played).then_with(|| a.title.cmp(&b.title)));
    lib
}

/// Launch by key; returns the game so it can be tracked.
pub fn launch(key: &str, progress: Progress) -> Result<Game> {
    let Some(p) = platforms::for_key(key) else { return msg("منصة اللعبة غير معروفة.") };
    let games = p.scan_games()?;
    let Some(g) = games.into_iter().find(|g| g.key == key) else {
        return msg("اللعبة غير موجودة. أعد فحص المكتبة.");
    };
    p.launch(&g, progress)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    /// Read-only scan of this machine: `cargo test scan_local -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn scan_local() {
        let lib = super::scan();
        for p in &lib.platforms {
            println!("{:?} detected={} games={} {:?}", p.id, p.detected, p.game_count, p.install_path);
        }
        for a in &lib.accounts {
            println!("account {} {} active={}", a.id, a.display_name, a.active);
        }
        for g in &lib.games {
            println!("{} | {} | {} | launch={:?} guarded={}", g.key, g.title, g.app_id, g.launch_account_id, g.guarded);
        }
    }

    /// Network: `cargo test art_lookup -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn art_lookup() {
        let lib = super::scan();
        let games: Vec<_> = lib.games.into_iter().filter(|g| g.platform != super::PlatformId::Steam).collect();
        for (k, a) in super::art::fetch_missing(&games) {
            println!("{k} -> {:?}", a.cover);
        }
    }
}
