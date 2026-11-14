// SPDX-License-Identifier: RPL-1.5
//! The browsers GDM knows (J2.2) and where each gets the extension. One list for the app's onboarding and
//! Settings and for GDM Setup's last page.
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;

use crate::{CWS_ID, EDGE_ID, FIREFOX_IDS};

/// (key, display name, exe candidates under Program Files / Program Files (x86) / LocalAppData)
#[cfg(windows)]
const WIN: [(&str, &str, &[&str]); 6] = [
    ("chrome", "Chrome", &[r"Google\Chrome\Application\chrome.exe"]),
    ("edge", "Edge", &[r"Microsoft\Edge\Application\msedge.exe"]),
    ("firefox", "Firefox", &[r"Mozilla Firefox\firefox.exe"]),
    ("brave", "Brave", &[r"BraveSoftware\Brave-Browser\Application\brave.exe"]),
    ("vivaldi", "Vivaldi", &[r"Vivaldi\Application\vivaldi.exe"]),
    ("opera", "Opera", &[r"Programs\Opera\opera.exe", r"Opera\opera.exe"]),
];

#[cfg(windows)]
fn find(rel: &[&str]) -> Option<PathBuf> {
    ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"]
        .iter()
        .filter_map(|v| std::env::var_os(v).map(PathBuf::from))
        .flat_map(|r| rel.iter().map(move |p| r.join(p)))
        .find(|p| p.is_file())
}

/// (key, display name, installed)
#[cfg(windows)]
pub fn browsers() -> Vec<(&'static str, &'static str, bool)> {
    WIN.iter().map(|(k, n, rel)| (*k, *n, find(rel).is_some())).collect()
}

/// A Chromium browser whose store item is not live yet (owner 2026-11-14: release before paying the Chrome Web
/// Store fee): start the browser and show the extension folder GDM installs next to itself (`dir`, built with the
/// dev key = DEV_ID, which the bridge accepts). Returns the browser's extensions page for the UI to copy: browsers
/// ignore chrome:// pages given on their command line (checked 2026-11-14: Chromium opened a New Tab), so the user
/// pastes it, turns on Developer mode and drags the folder onto the page. None = use the store. Windows only:
/// macOS/Linux keep the store link.
// ponytail: an unpacked copy is not updated by the store; once CWS_ID / EDGE_ID are set the app should ask
// users to remove it (two copies would both catch downloads).
#[cfg(windows)]
pub fn add_unpacked(browser: &str, dir: &std::path::Path) -> Option<&'static str> {
    let page = match browser {
        "firefox" => return None,
        "edge" if !EDGE_ID.is_empty() => return None,
        "edge" => "edge://extensions",
        _ if !CWS_ID.is_empty() => return None,
        "opera" => "opera://extensions",
        _ => "chrome://extensions",
    };
    let exe = WIN.iter().find(|b| b.0 == browser).and_then(|b| find(b.2))?;
    if !dir.join("manifest.json").is_file() {
        return None;
    }
    use std::os::windows::process::CommandExt;
    let _ = std::process::Command::new(exe).spawn();
    // explorer reads its own command line: /select,"path" must not be quoted as one argument
    let _ = std::process::Command::new(explorer()).raw_arg(format!("/select,\"{}\"", dir.display())).spawn();
    Some(page)
}

/// %SystemRoot%\explorer.exe by full path: a bare name is searched in the program's own folder first, which is
/// user-writable (GDM's install folder, Downloads for GDM Setup), so a planted explorer.exe would run
#[cfg(windows)]
pub fn explorer() -> PathBuf {
    std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows")).join("explorer.exe")
}

#[cfg(unix)]
pub fn add_unpacked(_browser: &str, _dir: &std::path::Path) -> Option<&'static str> {
    None
}

/// (key, display name, installed): macOS applications / Linux commands on PATH
#[cfg(unix)]
pub fn browsers() -> Vec<(&'static str, &'static str, bool)> {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let app = |names: &[&str]| names.iter().any(|n| Path::new("/Applications").join(n).is_dir() || home.join("Applications").join(n).is_dir());
    let path: Vec<PathBuf> = std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default();
    let cmd = |names: &[&str]| names.iter().any(|n| path.iter().any(|d| d.join(n).is_file()));
    let mac = cfg!(target_vendor = "apple");
    let has = |a: &[&str], c: &[&str]| if mac { app(a) } else { cmd(c) };
    vec![
        ("chrome", "Chrome", has(&["Google Chrome.app"], &["google-chrome", "google-chrome-stable"])),
        ("edge", "Edge", has(&["Microsoft Edge.app"], &["microsoft-edge", "microsoft-edge-stable"])),
        ("firefox", "Firefox", has(&["Firefox.app"], &["firefox"])),
        ("brave", "Brave", has(&["Brave Browser.app"], &["brave-browser", "brave"])),
        ("vivaldi", "Vivaldi", has(&["Vivaldi.app"], &["vivaldi", "vivaldi-stable"])),
        ("opera", "Opera", has(&["Opera.app"], &["opera"])),
    ]
}

/// The store page for a store key ("store-chrome" / "store-edge" / "store-firefox"): GDM's own listing once its
/// id is set, the store's search for GDM until then. Brave, Vivaldi and Opera use the Chrome Web Store.
pub fn store_url(key: &str) -> Option<String> {
    Some(match key {
        "store-chrome" if !CWS_ID.is_empty() => format!("https://chromewebstore.google.com/detail/{CWS_ID}"),
        "store-edge" if !EDGE_ID.is_empty() => format!("https://microsoftedge.microsoft.com/addons/detail/{EDGE_ID}"),
        "store-firefox" => format!("https://addons.mozilla.org/firefox/addon/{}/", FIREFOX_IDS[0]),
        "store-chrome" => "https://chromewebstore.google.com/search/Galaxy%20Download%20Manager".into(),
        "store-edge" => "https://microsoftedge.microsoft.com/addons/search/Galaxy%20Download%20Manager".into(),
        _ => return None,
    })
}

/// the store key for a browser key
pub fn store_key(browser: &str) -> &'static str {
    match browser {
        "firefox" => "store-firefox",
        "edge" => "store-edge",
        _ => "store-chrome",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_browser_has_a_store() {
        for (k, _, _) in browsers() {
            assert!(store_url(store_key(k)).is_some_and(|u| u.starts_with("https://")), "{k}");
        }
        assert!(store_url("evil").is_none());
    }

    #[test]
    fn unpacked_only_for_chromium_with_the_bundled_folder() {
        let none = std::path::Path::new("no-such-dir");
        for b in ["firefox", "chrome", "edge", "brave", "evil"] {
            assert!(add_unpacked(b, none).is_none(), "{b}"); // no manifest.json there: nothing is launched
        }
    }
}
