//! One module per store. Each implements `Platform`.
//! Phase 1: steam, epic, rockstar. Phase 2: gog, ea, ubisoft, battlenet, xbox.
use crate::error::Result;
use crate::library::models::{Account, Game, PlatformStatus};

mod epic;
mod rockstar;
mod steam;

#[allow(dead_code)] // accounts/launch are wired up in phase 1
pub trait Platform: Send + Sync {
    /// Is the store installed, where, and how many games it has.
    fn status(&self) -> PlatformStatus;
    /// Accounts known on this machine (Steam can have several).
    fn accounts(&self) -> Result<Vec<Account>>;
    /// Games owned by every known account (installed or not).
    fn scan_games(&self) -> Result<Vec<Game>>;
    /// Launch a game. Must make sure the owning account is active first.
    fn launch(&self, game: &Game) -> Result<()>;
}

pub fn all() -> Vec<Box<dyn Platform>> {
    vec![Box::new(steam::Steam), Box::new(epic::Epic), Box::new(rockstar::Rockstar)]
}
