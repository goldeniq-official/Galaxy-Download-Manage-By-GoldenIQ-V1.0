// SPDX-License-Identifier: RPL-1.5
//! The browsers GDM knows (J2.2) and where each gets the extension. One list for the app's onboarding and
//! Settings and for GDM Setup's last page.
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;

use crate::{CWS_ID, EDGE_ID, FIREFOX_IDS};

/// (key, display name, installed): Windows exe candidates under the usual roots
#[cfg(windows)]
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
}
