//! Automatic Steam account switching (phase 1).
//!
//! Why: Steam allows one signed-in account at a time. The user owns games on two
//! accounts (e.g. Call of Duty on a second account because the main one is banned
//! there), so Family Sharing is not an option: the game must run on the account
//! that owns it.
//!
//! Flow when launching a game whose owner is not the active account:
//! 1. Shut Steam down cleanly (`steam.exe -shutdown`) and wait for exit.
//! 2. Set HKCU\Software\Valve\Steam\AutoLoginUser = <AccountName>
//!    and mark that user MostRecent in config/loginusers.vdf.
//! 3. Start Steam; it signs in with the token it already remembers.
//! 4. Launch steam://rungameid/<appid>.
//!
//! Requirements: each account was signed in once with "Remember me".
//! Passwords are never read or stored. Steam Guard may still prompt sometimes.
use crate::error::{Error, Result};

#[allow(dead_code)]
pub fn switch_to(_account_name: &str) -> Result<()> {
    Err(Error::NotImplemented("steam account switch"))
}
