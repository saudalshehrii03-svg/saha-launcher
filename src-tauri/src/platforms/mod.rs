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

/// Launch through a store launcher without showing it:
/// - `start` asks the launcher to run the game (URL / command line);
/// - every window of the launcher processes (`images`, lower-case exe names or path tails)
///   stays hidden until the game process is running, then for 20 s more (late pop-ups);
/// - when `resend` is set, the request is repeated while the launcher is still starting
///   (Epic ignores a launch request it receives during its own start-up);
/// - if the game never starts, the windows come back so the user can see what the launcher wants.
pub fn launch_in_background(game: &Game, images: &'static [&'static str], resend: bool, progress: Progress, start: &dyn Fn() -> Result<()>) -> Result<()> {
    use std::sync::atomic::{AtomicU8, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    const RUNNING: u8 = 0;
    const STARTED: u8 = 1;
    const FAILED: u8 = 2;
    let state = Arc::new(AtomicU8::new(RUNNING));
    let s = state.clone();
    let hider = std::thread::spawn(move || {
        let mut h = crate::sys::WindowHider::default();
        let mut started_at: Option<Instant> = None;
        loop {
            match s.load(Ordering::Relaxed) {
                FAILED => return h,
                STARTED if started_at.get_or_insert_with(Instant::now).elapsed() > Duration::from_secs(20) => return h,
                _ => {}
            }
            h.sweep(images);
            std::thread::sleep(Duration::from_millis(200));
        }
    });

    let run = || -> Result<bool> {
        start()?;
        let begin = Instant::now();
        let mut resent = 0;
        while begin.elapsed() < Duration::from_secs(120) {
            if crate::tracker::is_running(game) {
                return Ok(true);
            }
            let secs = begin.elapsed().as_secs();
            if resend && ((secs >= 20 && resent == 0) || (secs >= 50 && resent == 1)) {
                progress("اللانشر يجهّز… أعيد طلب التشغيل");
                let _ = start();
                resent += 1;
            }
            std::thread::sleep(Duration::from_secs(1));
        }
        Ok(false)
    };

    match run() {
        Ok(true) => {
            state.store(STARTED, Ordering::Relaxed);
            progress("اللعبة شغّالة");
            Ok(())
        }
        other => {
            state.store(FAILED, Ordering::Relaxed);
            if let Ok(h) = hider.join() {
                h.restore();
            }
            match other {
                Ok(_) => crate::error::msg(format!(
                    "{} ما شغّل اللعبة خلال دقيقتين. أظهرت نافذته، يمكن يطلب تسجيل دخول أو تحديث.",
                    game.platform.slug()
                )),
                Err(e) => Err(e),
            }
        }
    }
}
