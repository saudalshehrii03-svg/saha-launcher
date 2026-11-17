//! One module per store. Each implements `Platform`.
use crate::error::Result;
use crate::library::models::{Account, Game, PlatformId};

pub mod battlenet;
pub mod ea;
pub mod epic;
pub mod gog;
pub mod rockstar;
pub mod steam;
pub mod ubisoft;
pub mod xbox;

/// Progress messages (Arabic) shown while launching.
pub type Progress<'a> = &'a dyn Fn(&str);

pub trait Platform: Send + Sync {
    fn id(&self) -> PlatformId;
    fn name(&self) -> &'static str;
    /// Where the store is installed; `None` = not detected.
    fn install_path(&self) -> Option<String>;
    /// Accounts known on this machine (Steam can have several).
    fn accounts(&self) -> Vec<Account> {
        Vec::new()
    }
    /// Games found for every known account.
    fn scan_games(&self) -> Result<Vec<Game>>;
    /// Launch a game. Must make sure the right account is active first.
    fn launch(&self, game: &Game, progress: Progress) -> Result<()>;
}

pub fn all() -> Vec<Box<dyn Platform>> {
    vec![
        Box::new(steam::Steam),
        Box::new(epic::Epic),
        Box::new(rockstar::Rockstar),
        Box::new(gog::Gog),
        Box::new(ea::Ea),
        Box::new(ubisoft::Ubisoft),
        Box::new(battlenet::BattleNet),
        Box::new(xbox::Xbox),
    ]
}

/// The platform a game key ("steam:123") belongs to.
pub fn for_key(key: &str) -> Option<Box<dyn Platform>> {
    let prefix = key.split(':').next()?;
    all().into_iter().find(|p| p.id().slug() == prefix)
}
