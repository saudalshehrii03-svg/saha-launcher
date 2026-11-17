//! Detect mods the user installed by hand (not through Saha), for any game.
//!
//! Recognizes common loaders and mod folders: Script Hook V/RDR2, ASI loaders and
//! .asi plugins, Lenny's Mod Loader (lml/), scripts/, ReShade, BepInEx, MelonLoader,
//! Cyberpunk (archive/pc/mod, CET mods, RED4ext, redscript), Witcher 3 (Mods/),
//! Hollow Knight (Managed/Mods), Elden Ring / Sekiro (ModEngine2), generic mods/ folders.
//!
//! Disabling never deletes: the mod's files move to `<game>/.saha-disabled/<same path>`
//! (same drive, instant) and move back when enabled. State lives in external_mods.json.
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{msg, Result};
use crate::settings;

const FILE: &str = "external_mods.json";
const STASH: &str = ".saha-disabled";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalMod {
    /// Stable id, e.g. "lml:RTEA" or "file:dinput8.dll".
    pub id: String,
    pub name: String,
    /// "loader" | "mod" | "graphics"
    pub kind: String,
    /// Paths relative to the game folder.
    pub paths: Vec<String>,
    pub enabled: bool,
}

/// game key -> mods currently disabled by Saha (their files are in the stash).
fn disabled_state() -> HashMap<String, Vec<ExternalMod>> {
    settings::read_json(FILE)
}

fn save_state(s: &HashMap<String, Vec<ExternalMod>>) -> Result<()> {
    settings::write_json(FILE, s)
}

fn entries(dir: &Path) -> Vec<(String, bool)> {
    fs::read_dir(dir)
        .map(|rd| rd.flatten().map(|e| (e.file_name().to_string_lossy().to_string(), e.path().is_dir())).collect())
        .unwrap_or_default()
}

fn find_ci(dir: &Path, name: &str) -> Option<String> {
    entries(dir).into_iter().map(|(n, _)| n).find(|n| n.eq_ignore_ascii_case(name))
}

/// Path segments resolved case-insensitively ("archive/pc/mod").
fn resolve(base: &Path, rel: &str) -> Option<PathBuf> {
    let mut p = base.to_path_buf();
    for seg in rel.split('/') {
        p = p.join(find_ci(&p, seg)?);
    }
    Some(p)
}

fn rel(base: &Path, p: &Path) -> String {
    p.strip_prefix(base).unwrap_or(p).to_string_lossy().replace('\\', "/")
}

fn stem(n: &str) -> &str {
    n.rsplit_once('.').map(|(s, _)| s).unwrap_or(n)
}

const LOADERS: &[(&str, &str, &str)] = &[
    ("ScriptHookRDR2.dll", "Script Hook RDR2", "loader"),
    ("ScriptHookV.dll", "Script Hook V", "loader"),
    ("ScriptHookVDotNet.asi", "Script Hook V .NET", "loader"),
    ("ScriptHookVDotNet2.dll", "Script Hook V .NET", "loader"),
    ("ScriptHookVDotNet3.dll", "Script Hook V .NET", "loader"),
    ("dinput8.dll", "ASI Loader / محمّل مودات (dinput8)", "loader"),
    ("version.dll", "محمّل مودات (version.dll)", "loader"),
    ("winmm.dll", "محمّل مودات (winmm.dll)", "loader"),
    ("ModManager.Core.dll", "Lenny's Mod Loader", "loader"),
    ("modengine2_launcher.exe", "Mod Engine 2", "loader"),
];

/// Scan one game folder (and its "Game" subfolder, used by FromSoftware titles).
fn scan_dir(base: &Path, game_dir: &Path, out: &mut Vec<ExternalMod>) {
    let mut push = |id: String, name: String, kind: &str, paths: Vec<PathBuf>| {
        let paths: Vec<String> = paths.iter().map(|p| rel(game_dir, p)).collect();
        // A file belongs to one mod only (first match wins: loaders, then ReShade, then plugins).
        let taken = out.iter().any(|m| m.paths.iter().any(|p| paths.contains(p)));
        if !paths.is_empty() && !taken && !out.iter().any(|m| m.id == id) {
            out.push(ExternalMod { id, name, kind: kind.into(), paths, enabled: true });
        }
    };
    let top = entries(base);
    let has = |n: &str| top.iter().any(|(e, _)| e.eq_ignore_ascii_case(n));

    // Loaders / frameworks (single files, with companions).
    for (file, name, kind) in LOADERS {
        if let Some(f) = find_ci(base, file) {
            let mut paths = vec![base.join(&f)];
            if file.eq_ignore_ascii_case("ModManager.Core.dll") {
                for extra in ["ModManager.NativeInterop.dll", "NLog.dll", "lml.ini"] {
                    if let Some(x) = find_ci(base, extra) {
                        paths.push(base.join(x));
                    }
                }
            }
            if file.eq_ignore_ascii_case("modengine2_launcher.exe") {
                for (n, _) in &top {
                    if n.to_lowercase().starts_with("config_") && n.ends_with(".toml") || n.eq_ignore_ascii_case("modengine2") {
                        paths.push(base.join(n));
                    }
                }
            }
            push(format!("file:{}", rel(game_dir, &base.join(&f))), name.to_string(), kind, paths);
        }
    }

    // ReShade (graphics): proxy dll + ReShade.ini + shader folders.
    if has("ReShade.ini") {
        let mut paths = Vec::new();
        for (n, _) in &top {
            let l = n.to_lowercase();
            if ["dxgi.dll", "d3d11.dll", "d3d12.dll", "d3d9.dll", "opengl32.dll", "reshade.ini", "reshade.asi", "reshadepreset.ini", "reshade-shaders"].contains(&l.as_str()) {
                paths.push(base.join(n));
            }
        }
        push("reshade".into(), "ReShade (فلاتر الرسوميات)".into(), "graphics", paths);
    }

    // .asi plugins (+ same-name .ini).
    for (n, dir) in &top {
        if !dir && n.to_lowercase().ends_with(".asi") && !n.eq_ignore_ascii_case("ScriptHookVDotNet.asi") {
            let mut paths = vec![base.join(n)];
            for (m, _) in &top {
                if m != n && stem(m).eq_ignore_ascii_case(stem(n)) && m.to_lowercase().ends_with(".ini") {
                    paths.push(base.join(m));
                }
            }
            push(format!("asi:{}", rel(game_dir, &base.join(n))), stem(n).to_string(), "mod", paths);
        }
    }

    // Folders where each child is one mod.
    let per_child: &[(&str, &str)] = &[
        ("lml", "mod"),
        ("scripts", "mod"),
        ("BepInEx/plugins", "mod"),
        ("Mods", "mod"),
        ("mods", "mod"),
        ("mod", "mod"),
        ("archive/pc/mod", "mod"),
        ("bin/x64/plugins/cyber_engine_tweaks/mods", "mod"),
        ("red4ext/plugins", "mod"),
        ("r6/scripts", "mod"),
        ("r6/tweaks", "mod"),
        ("hollow_knight_Data/Managed/Mods", "mod"),
    ];
    let skip_child = |n: &str| {
        let l = n.to_lowercase();
        l.ends_with(".log") || l.ends_with(".xml") || l.ends_with(".dat") || l.starts_with('.') || l == "disabled"
    };
    // Game data folder names: a mods/ folder holding these is one override mod, not many.
    const DATA_DIRS: &[&str] = &[
        "font", "menu", "msg", "parts", "chr", "map", "sfx", "sound", "action", "event", "param", "script", "obj", "mtd",
        "shader", "other", "cutscene", "movie", "facegen", "material", "update", "x64", "common", "regulation.bin",
    ];
    for (folder, kind) in per_child {
        let Some(dir) = resolve(base, folder) else { continue };
        if !dir.is_dir() {
            continue;
        }
        let kids: Vec<String> = entries(&dir).into_iter().map(|(n, _)| n).filter(|n| !skip_child(n)).collect();
        let is_data = kids.iter().any(|n| DATA_DIRS.contains(&n.to_lowercase().as_str()));
        if is_data && folder.eq_ignore_ascii_case("mods") || is_data && *folder == "mod" {
            push(format!("dir:{}", rel(game_dir, &dir)), format!("مودات مجلد {}", rel(game_dir, &dir)), kind, vec![dir.clone()]);
            continue;
        }
        // Group a folder and same-name files (e.g. RemoveVignette/ + RemoveVignette.dll) into one mod.
        let mut groups: Vec<(String, Vec<PathBuf>)> = Vec::new();
        for n in kids {
            let key = stem(&n).to_lowercase();
            match groups.iter_mut().find(|(k, _)| *k == key) {
                Some((_, v)) => v.push(dir.join(&n)),
                None => groups.push((key, vec![dir.join(&n)])),
            }
        }
        for (_, paths) in groups {
            let first = paths[0].file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
            push(format!("dir:{}", rel(game_dir, &paths[0])), stem(&first).replace(['_', '-'], " "), kind, paths);
        }
    }

    // BepInEx / MelonLoader frameworks themselves.
    for (folder, name) in [("BepInEx", "BepInEx"), ("MelonLoader", "MelonLoader")] {
        if let Some(f) = find_ci(base, folder) {
            push(format!("file:{}", rel(game_dir, &base.join(&f))), name.into(), "loader", vec![base.join(f)]);
        }
    }
}

/// All hand-installed mods of a game: active ones found on disk + ones Saha disabled.
pub fn detect(game_key: &str, game_dir: &str, saha_files: &[String]) -> Vec<ExternalMod> {
    let dir = PathBuf::from(game_dir);
    let mut out = Vec::new();
    scan_dir(&dir, &dir, &mut out);
    if let Some(g) = find_ci(&dir, "Game") {
        scan_dir(&dir.join(g), &dir, &mut out);
    }
    // Hide what Saha itself installed (shown in its own list).
    out.retain(|m| !m.paths.iter().all(|p| saha_files.iter().any(|s| s.eq_ignore_ascii_case(p))));
    for m in disabled_state().get(game_key).into_iter().flatten() {
        if !out.iter().any(|o| o.id == m.id) {
            out.push(ExternalMod { enabled: false, ..m.clone() });
        }
    }
    out.sort_by(|a, b| (a.kind != "loader").cmp(&(b.kind != "loader")).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    out
}

fn move_path(from: &Path, to: &Path) -> Result<()> {
    if let Some(p) = to.parent() {
        fs::create_dir_all(p)?;
    }
    fs::rename(from, to).map_err(|e| crate::error::Error::Msg(format!("ما قدرت أنقل الملف (يمكن اللعبة شغّالة؟): {e}")))
}

/// Enable / disable a hand-installed mod by moving its files to/from the stash.
pub fn set_enabled(game_key: &str, game_dir: &str, id: &str, enabled: bool, saha_files: &[String]) -> Result<()> {
    let dir = PathBuf::from(game_dir);
    let stash = dir.join(STASH);
    let mut state = disabled_state();
    let list = state.entry(game_key.to_string()).or_default();
    if enabled {
        let Some(i) = list.iter().position(|m| m.id == id) else { return msg("المود غير موجود في المعطّلة.") };
        let m = list[i].clone();
        for p in &m.paths {
            let src = stash.join(p);
            if src.exists() {
                move_path(&src, &dir.join(p))?;
            }
        }
        list.remove(i);
    } else {
        let Some(m) = detect(game_key, game_dir, saha_files).into_iter().find(|m| m.id == id && m.enabled) else {
            return msg("المود غير موجود.");
        };
        for p in &m.paths {
            let src = dir.join(p);
            if src.exists() {
                move_path(&src, &stash.join(p))?;
            }
        }
        list.push(ExternalMod { enabled: false, ..m });
    }
    save_state(&state)
}

#[cfg(test)]
mod tests {
    /// Read-only: `cargo test detect_local -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn detect_local() {
        let lib = crate::library::scan();
        for g in lib.games.iter().filter(|g| g.installed) {
            let Some(dir) = &g.install_dir else { continue };
            let found = super::detect(&g.key, dir, &[]);
            if !found.is_empty() {
                println!("== {}", g.title);
                for m in found {
                    println!("  [{}] {} -> {:?}", m.kind, m.name, m.paths);
                }
            }
        }
    }
}
