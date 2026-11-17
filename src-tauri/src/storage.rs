//! Storage report: free/total space per drive, size of every installed game,
//! and what Saha itself uses (mods + media). Folder sizes are computed when the
//! platform doesn't report them (cached for the session).
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use serde::Serialize;

use crate::library;
use crate::settings;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Drive {
    pub root: String,
    pub total: u64,
    pub free: u64,
    /// Bytes used by games on this drive.
    pub games: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSize {
    pub key: String,
    pub title: String,
    pub platform: crate::library::models::PlatformId,
    pub bytes: u64,
    pub dir: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub drives: Vec<Drive>,
    pub games: Vec<GameSize>,
    pub mods_bytes: u64,
    pub media_bytes: u64,
}

pub fn dir_size(p: &Path) -> u64 {
    let Ok(rd) = std::fs::read_dir(p) else { return 0 };
    rd.flatten()
        .map(|e| match e.file_type() {
            Ok(t) if t.is_dir() => dir_size(&e.path()),
            Ok(t) if t.is_file() => e.metadata().map(|m| m.len()).unwrap_or(0),
            _ => 0, // skip symlinks/junctions
        })
        .sum()
}

fn cache() -> &'static Mutex<HashMap<String, u64>> {
    static C: OnceLock<Mutex<HashMap<String, u64>>> = OnceLock::new();
    C.get_or_init(Default::default)
}

#[cfg(windows)]
fn disk_space(root: &str) -> Option<(u64, u64)> {
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    let wide: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();
    let (mut free, mut total, mut _all_free) = (0u64, 0u64, 0u64);
    let ok = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut free, &mut total, &mut _all_free) };
    (ok != 0).then_some((total, free))
}

#[cfg(not(windows))]
fn disk_space(_: &str) -> Option<(u64, u64)> {
    None
}

fn drive_of(dir: &str) -> Option<String> {
    let d = dir.replace('/', "\\");
    let b = d.as_bytes();
    (b.len() >= 2 && b[1] == b':').then(|| format!("{}:\\", (b[0] as char).to_ascii_uppercase()))
}

pub fn report() -> Report {
    let lib = library::scan();
    let installed: Vec<_> = lib.games.into_iter().filter(|g| g.installed && g.install_dir.is_some()).collect();

    // Size each game: platform value, else cached, else walk the folder (in parallel).
    let games: Vec<GameSize> = std::thread::scope(|s| {
        let handles: Vec<_> = installed
            .iter()
            .map(|g| {
                s.spawn(move || {
                    let dir = g.install_dir.clone().unwrap_or_default();
                    let bytes = g.size_bytes.filter(|b| *b > 0).unwrap_or_else(|| {
                        if let Some(b) = cache().lock().ok().and_then(|c| c.get(&dir).copied()) {
                            return b;
                        }
                        let b = dir_size(Path::new(&dir));
                        if let Ok(mut c) = cache().lock() {
                            c.insert(dir.clone(), b);
                        }
                        b
                    });
                    GameSize { key: g.key.clone(), title: g.title.clone(), platform: g.platform, bytes, dir }
                })
            })
            .collect();
        handles.into_iter().filter_map(|h| h.join().ok()).collect()
    });
    let mut games = games;
    games.sort_by_key(|g| std::cmp::Reverse(g.bytes));

    let mut roots: Vec<String> = games.iter().filter_map(|g| drive_of(&g.dir)).collect();
    roots.push("C:\\".into());
    roots.sort();
    roots.dedup();
    let drives = roots
        .into_iter()
        .filter_map(|r| {
            let (total, free) = disk_space(&r)?;
            let used: u64 = games.iter().filter(|g| drive_of(&g.dir).as_deref() == Some(r.as_str())).map(|g| g.bytes).sum();
            Some(Drive { root: r, total, free, games: used })
        })
        .collect();

    let (mods_bytes, media_bytes) = settings::dir()
        .map(|d| (dir_size(&d.join("mods")), dir_size(&d.join("media"))))
        .unwrap_or((0, 0));
    Report { drives, games, mods_bytes, media_bytes }
}
