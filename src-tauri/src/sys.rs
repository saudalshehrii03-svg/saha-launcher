//! Thin OS helpers (registry, processes, URLs). Windows only; other targets get
//! inert stubs so `cargo check` works anywhere.
use std::path::Path;
use std::process::Command;

#[derive(Clone, Copy)]
pub enum Hive {
    CurrentUser,
    LocalMachine,
}

/// Command that never flashes a console window.
pub fn hidden(program: impl AsRef<std::ffi::OsStr>) -> Command {
    #[allow(unused_mut)]
    let mut c = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    c
}

/// Is a process with this image name (e.g. "steam.exe") running?
pub fn process_running(image: &str) -> bool {
    if !cfg!(windows) {
        return false;
    }
    hidden("tasklist")
        .args(["/FI", &format!("IMAGENAME eq {image}"), "/NH"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_lowercase().contains(&image.to_lowercase()))
        .unwrap_or(false)
}

/// Open a URL / protocol link with its registered handler.
pub fn open_url(url: &str) -> std::io::Result<()> {
    hidden("rundll32").args(["url.dll,FileProtocolHandler", url]).spawn().map(|_| ())
}

pub fn spawn(exe: &Path, args: &[&str]) -> std::io::Result<()> {
    let mut c = Command::new(exe);
    c.args(args);
    if let Some(dir) = exe.parent() {
        c.current_dir(dir);
    }
    c.spawn().map(|_| ())
}

/// Hides the windows of given processes (Steam's client and web helper) while a
/// launch is in progress, and can bring them back if the launch stalls.
#[derive(Default)]
pub struct WindowHider {
    #[cfg_attr(not(windows), allow(dead_code))]
    hidden: Vec<isize>,
}

#[cfg(windows)]
impl WindowHider {
    /// Hide every visible top-level window owned by one of `images`
    /// (except sign-in / Steam Guard prompts, which the user must see).
    pub fn sweep(&mut self, images: &[&str]) {
        use windows_sys::Win32::Foundation::{HWND, LPARAM};
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible, ShowWindow, SW_HIDE,
        };

        unsafe extern "system" fn collect(hwnd: HWND, lp: LPARAM) -> i32 {
            let v = &mut *(lp as *mut Vec<HWND>);
            if IsWindowVisible(hwnd) != 0 {
                v.push(hwnd);
            }
            1
        }

        let mut windows: Vec<HWND> = Vec::new();
        unsafe { EnumWindows(Some(collect), &mut windows as *mut _ as LPARAM) };
        for hwnd in windows {
            let mut pid = 0u32;
            unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
            let Some(image) = process_path(pid) else { continue };
            if !images.iter().any(|i| image.ends_with(&format!("\\{i}"))) {
                continue;
            }
            let mut title = [0u16; 256];
            let n = unsafe { GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32) };
            let title = String::from_utf16_lossy(&title[..n.max(0) as usize]).to_lowercase();
            if title.contains("sign in") || title.contains("steam guard") || title.contains("login") {
                continue;
            }
            unsafe { ShowWindow(hwnd, SW_HIDE) };
            if !self.hidden.contains(&(hwnd as isize)) {
                self.hidden.push(hwnd as isize);
            }
        }
    }

    /// Show again whatever we hid that still exists.
    pub fn restore(&self) {
        use windows_sys::Win32::UI::WindowsAndMessaging::{IsWindow, ShowWindow, SW_SHOWNOACTIVATE};
        for &h in &self.hidden {
            let hwnd = h as windows_sys::Win32::Foundation::HWND;
            unsafe {
                if IsWindow(hwnd) != 0 {
                    ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                }
            }
        }
    }
}

/// Full lower-case exe path of a process.
#[cfg(windows)]
pub fn process_path(pid: u32) -> Option<String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() {
            return None;
        }
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut len);
        CloseHandle(h);
        (ok != 0).then(|| String::from_utf16_lossy(&buf[..len as usize]).to_lowercase())
    }
}

/// Exe paths (lower-case) of every running process we can query.
#[cfg(windows)]
pub fn running_exe_paths() -> Vec<String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };
    let mut out = Vec::new();
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap.is_null() || snap as isize == -1 {
            return out;
        }
        let mut e: PROCESSENTRY32W = std::mem::zeroed();
        e.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut ok = Process32FirstW(snap, &mut e);
        while ok != 0 {
            if let Some(p) = process_path(e.th32ProcessID) {
                out.push(p);
            }
            ok = Process32NextW(snap, &mut e);
        }
        CloseHandle(snap);
    }
    out
}

#[cfg(not(windows))]
pub fn running_exe_paths() -> Vec<String> {
    Vec::new()
}

/// One "Programs and Features" entry.
pub struct Uninstall {
    pub name: String,
    pub publisher: String,
    pub location: String,
    pub uninstall: String,
}

pub fn uninstall_entries() -> Vec<Uninstall> {
    const ROOTS: &[(Hive, &str)] = &[
        (Hive::LocalMachine, r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
        (Hive::LocalMachine, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
        (Hive::CurrentUser, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
    ];
    let mut out = Vec::new();
    for &(hive, root) in ROOTS {
        for key in reg_subkeys(hive, root) {
            let path = format!(r"{root}\{key}");
            let get = |n: &str| reg_str(hive, &path, n).unwrap_or_default();
            let name = get("DisplayName");
            if name.is_empty() {
                continue;
            }
            out.push(Uninstall {
                name,
                publisher: get("Publisher"),
                location: get("InstallLocation"),
                uninstall: get("UninstallString"),
            });
        }
    }
    out
}

#[cfg(not(windows))]
impl WindowHider {
    pub fn sweep(&mut self, _: &[&str]) {}
    pub fn restore(&self) {}
}

#[cfg(windows)]
mod reg {
    use super::Hive;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    fn root(h: Hive) -> RegKey {
        RegKey::predef(match h {
            Hive::CurrentUser => HKEY_CURRENT_USER,
            Hive::LocalMachine => HKEY_LOCAL_MACHINE,
        })
    }
    pub fn get_str(h: Hive, path: &str, name: &str) -> Option<String> {
        root(h).open_subkey_with_flags(path, KEY_READ).ok()?.get_value(name).ok()
    }
    pub fn get_u32(h: Hive, path: &str, name: &str) -> Option<u32> {
        root(h).open_subkey_with_flags(path, KEY_READ).ok()?.get_value(name).ok()
    }
    pub fn subkeys(h: Hive, path: &str) -> Vec<String> {
        root(h)
            .open_subkey_with_flags(path, KEY_READ)
            .map(|k| k.enum_keys().flatten().collect())
            .unwrap_or_default()
    }
    pub fn set_str(path: &str, name: &str, val: &str) -> std::io::Result<()> {
        root(Hive::CurrentUser).open_subkey_with_flags(path, KEY_WRITE)?.set_value(name, &val.to_string())
    }
    pub fn set_u32(path: &str, name: &str, val: u32) -> std::io::Result<()> {
        root(Hive::CurrentUser).open_subkey_with_flags(path, KEY_WRITE)?.set_value(name, &val)
    }
}

#[cfg(not(windows))]
mod reg {
    use super::Hive;
    pub fn get_str(_: Hive, _: &str, _: &str) -> Option<String> {
        None
    }
    pub fn get_u32(_: Hive, _: &str, _: &str) -> Option<u32> {
        None
    }
    pub fn subkeys(_: Hive, _: &str) -> Vec<String> {
        Vec::new()
    }
    pub fn set_str(_: &str, _: &str, _: &str) -> std::io::Result<()> {
        Err(std::io::Error::other("registry is Windows only"))
    }
    pub fn set_u32(_: &str, _: &str, _: u32) -> std::io::Result<()> {
        Err(std::io::Error::other("registry is Windows only"))
    }
}

pub use reg::{
    get_str as reg_str, get_u32 as reg_u32, set_str as reg_set_str, set_u32 as reg_set_u32, subkeys as reg_subkeys,
};
