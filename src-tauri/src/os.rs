// SPDX-License-Identifier: RPL-1.5
//! Windows shell actions without the shell plugin (X-02, CVE-2025-31477): every path comes from the database
//! (looked up by job id), every URL from the fixed table below. Nothing the WebView sends is ever executed.
use std::path::{Path, PathBuf};
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// links the UI may open, by key only
pub fn link(key: &str) -> Option<&'static str> {
    Some(match key {
        "store-chrome" => "https://chromewebstore.google.com/search/Galaxy%20Download%20Manager",
        "store-edge" => "https://microsoftedge.microsoft.com/addons/search/Galaxy%20Download%20Manager",
        "store-firefox" => "https://addons.mozilla.org/firefox/search/?q=Galaxy%20Download%20Manager",
        "github" => "https://github.com/goldeniq-official/galaxy-download-manager",
        "releases" => "https://github.com/goldeniq-official/galaxy-download-manager/releases",
        "license" => "https://github.com/goldeniq-official/galaxy-download-manager/blob/main/LICENSE",
        "privacy" => "https://github.com/goldeniq-official/galaxy-download-manager/blob/main/PRIVACY.md",
        "windows-security" => "windowsdefender://threat/",
        _ => return None,
    })
}

fn shell_open(target: &str, verb: &str) -> Result<(), String> {
    let r = unsafe { ShellExecuteW(None, &HSTRING::from(verb), &HSTRING::from(target), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL) };
    if r.0 as isize > 32 {
        Ok(())
    } else {
        Err(format!("shell {}", r.0 as isize))
    }
}

pub fn open_link(key: &str) -> Result<(), String> {
    shell_open(link(key).ok_or("unknown link")?, "open")
}

/// a job's source page (J5 refresh, "login needed"): from the database, http(s) only, never a file: or custom scheme
pub fn open_web(url: &str) -> Result<(), String> {
    let u = url::Url::parse(url).map_err(|_| "bad-url")?;
    if !matches!(u.scheme(), "http" | "https") {
        return Err("bad-url".into());
    }
    shell_open(u.as_str(), "open")
}

/// open a finished download with its associated program. Dangerous types are never opened directly (N-02):
/// the UI shows only [open folder] for them, and this refuses even if asked.
pub fn open_file(path: &Path, name: &str) -> Result<(), String> {
    if gdm_core::name::dangerous(name) {
        return Err("dangerous".into());
    }
    if !path.is_file() {
        return Err("missing".into());
    }
    shell_open(&path.display().to_string(), "open")
}

/// Explorer with the file selected (falls back to the folder)
pub fn open_folder(path: &Path) -> Result<(), String> {
    let mut c = std::process::Command::new("explorer.exe");
    if path.is_file() {
        c.arg(format!("/select,{}", path.display()));
    } else {
        c.arg(path.parent().unwrap_or(path));
    }
    c.spawn().map(|_| ()).map_err(|e| e.to_string())
}

/// browsers GDM knows (J2.2): (key, display name, exe candidates under the usual roots)
pub fn browsers() -> Vec<(&'static str, &'static str, bool)> {
    let roots: Vec<PathBuf> =
        ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"].iter().filter_map(|v| std::env::var_os(v).map(PathBuf::from)).collect();
    let has = |rel: &[&str]| roots.iter().any(|r| rel.iter().any(|p| r.join(p).is_file()));
    vec![
        ("chrome", "Chrome", has(&[r"Google\Chrome\Application\chrome.exe"])),
        ("edge", "Edge", has(&[r"Microsoft\Edge\Application\msedge.exe"])),
        ("firefox", "Firefox", has(&[r"Mozilla Firefox\firefox.exe"])),
        ("brave", "Brave", has(&[r"BraveSoftware\Brave-Browser\Application\brave.exe"])),
        ("vivaldi", "Vivaldi", has(&[r"Vivaldi\Application\vivaldi.exe"])),
        ("opera", "Opera", has(&[r"Programs\Opera\opera.exe", r"Opera\opera.exe"])),
    ]
}

/// volume checks for the download folder (onboarding step 3, plan-ux §4): filesystem and free bytes
pub fn folder_check(dir: &Path) -> (String, u64) {
    let fs = std::fs::File::open(dir).map(|f| gdm_core::fsx::fs_name(&f)).unwrap_or_default();
    (fs, gdm_core::fsx::free_space(dir))
}

/// the native folder picker (IFileOpenDialog, FOS_PICKFOLDERS) on its own STA thread
pub fn pick_folder(start: &Path) -> Option<PathBuf> {
    let start = start.to_path_buf();
    std::thread::spawn(move || unsafe {
        use windows::Win32::System::Com::*;
        use windows::Win32::UI::Shell::*;
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().ok()?;
        let r = (|| -> windows::core::Result<PathBuf> {
            let d: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
            d.SetOptions(d.GetOptions()? | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM)?;
            if let Ok(item) = SHCreateItemFromParsingName::<_, _, IShellItem>(&HSTRING::from(start.as_os_str()), None) {
                let _ = d.SetFolder(&item);
            }
            d.Show(None)?;
            let p = d.GetResult()?.GetDisplayName(SIGDN_FILESYSPATH)?;
            let s = p.to_string().unwrap_or_default();
            CoTaskMemFree(Some(p.0 as _));
            Ok(PathBuf::from(s))
        })();
        CoUninitialize();
        r.ok()
    })
    .join()
    .ok()
    .flatten()
}
