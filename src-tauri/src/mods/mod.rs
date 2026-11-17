//! Mods (phase 3).
//! - `sources`: where mods come from (mod.io, Thunderstore, Nexus via nxm:// links)
//! - `games`:   per-game install rules (target folders, loaders, load order)
//! Deployment plan: mods live in a staging folder and are linked into the game,
//! so disabling or removing a mod restores the original files exactly.
//! Online-only titles (GTA Online, Call of Duty, CS2, Fortnite) are flagged to avoid bans.
pub mod games;
pub mod sources;
