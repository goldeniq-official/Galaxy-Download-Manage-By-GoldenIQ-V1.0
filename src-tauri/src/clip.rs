// SPDX-License-Identifier: RPL-1.5
//! Clipboard monitor (F10, I-01). Opt-in (off by default; IDM has it on). A message-only window gets
//! WM_CLIPBOARDUPDATE; while the clipboard is open, the privacy markers are checked BEFORE any text is read:
//! `ExcludeClipboardContentFromMonitorProcessing` present, or `CanIncludeInClipboardHistory` = 0 (password
//! managers set these) => nothing is read. Text is read only when the monitor is on, at most 8 KiB, and kept
//! only when the whole text is one http(s) URL whose path ends in a caught extension. Nothing is logged.
//! macOS / Linux (P4): not available yet. Password managers mark secrets differently there
//! (org.nspasteboard.ConcealedType, x-kde-passwordManagerHint) and a monitor that cannot honour every marker
//! would read passwords, so the monitor stays off rather than read what it must not.
use std::sync::atomic::AtomicU64;
#[cfg(windows)]
use std::sync::atomic::Ordering;
#[cfg(windows)]
use windows::core::w;
#[cfg(windows)]
use windows::Win32::Foundation::HGLOBAL;
#[cfg(windows)]
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, CloseClipboard, GetClipboardData, GetClipboardSequenceNumber, IsClipboardFormatAvailable, OpenClipboard,
    RegisterClipboardFormatW,
};
#[cfg(windows)]
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
#[cfg(windows)]
use windows::Win32::System::Ole::CF_UNICODETEXT;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::*;

pub const MAX_TEXT: usize = 8 * 1024;
/// times the text of the clipboard was actually read (tests check it stays 0 for excluded content)
pub static TEXT_READS: AtomicU64 = AtomicU64::new(0);

/// the whole text is one http(s) URL with a caught extension in its path (not in the query)
pub fn pick(text: &str, exts: &[String]) -> Option<String> {
    let t = text.trim();
    if t.len() > gdm_bridge::MAX_URL || t.contains(char::is_whitespace) {
        return None;
    }
    let u = url::Url::parse(t).ok()?;
    if !matches!(u.scheme(), "http" | "https") || !u.username().is_empty() || u.password().is_some() || u.host_str().is_none() {
        return None;
    }
    let leaf = u.path_segments()?.next_back()?.to_ascii_lowercase();
    let ext = gdm_core::name::type_extension(&leaf); // split volumes (.7z.001, .r00) count as their archive
    (!ext.is_empty() && exts.iter().any(|e| e.eq_ignore_ascii_case(&ext))).then(|| u.to_string())
}

/// §04 YouTube row: a copied video page (youtube.com / youtu.be, exact host or subdomain by parsed host)
pub fn pick_video(text: &str) -> Option<String> {
    let t = text.trim();
    if t.len() > gdm_bridge::MAX_URL || t.contains(char::is_whitespace) {
        return None;
    }
    let u = url::Url::parse(t).ok()?;
    let h = u.host_str()?.to_ascii_lowercase();
    let video = ["youtube.com", "youtu.be"].iter().any(|d| h == *d || h.ends_with(&format!(".{d}")));
    (matches!(u.scheme(), "http" | "https") && u.username().is_empty() && video && u.path().len() > 1).then(|| u.to_string())
}

/// what the clipboard holds right now, if the monitor may read it (None = excluded, empty, or not text)
#[cfg(windows)]
fn read_allowed() -> Option<String> {
    unsafe {
        let excl = RegisterClipboardFormatW(w!("ExcludeClipboardContentFromMonitorProcessing"));
        let hist = RegisterClipboardFormatW(w!("CanIncludeInClipboardHistory"));
        if IsClipboardFormatAvailable(excl).is_ok() {
            return None;
        }
        if IsClipboardFormatAvailable(hist).is_ok() {
            // a DWORD; 0 = "do not keep this" (privacy marker): treat like exclusion
            let h = GetClipboardData(hist).ok()?;
            let g = HGLOBAL(h.0);
            if GlobalSize(g) >= 4 {
                let p = GlobalLock(g) as *const u32;
                let v = if p.is_null() { 0 } else { *p };
                let _ = GlobalUnlock(g);
                if v == 0 {
                    return None;
                }
            }
        }
        if IsClipboardFormatAvailable(CF_UNICODETEXT.0 as u32).is_err() {
            return None;
        }
        let h = GetClipboardData(CF_UNICODETEXT.0 as u32).ok()?;
        let g = HGLOBAL(h.0);
        let size = GlobalSize(g);
        if size == 0 || size > MAX_TEXT * 2 + 2 {
            return None; // bounded before scanning for the terminator
        }
        let p = GlobalLock(g) as *const u16;
        if p.is_null() {
            return None;
        }
        TEXT_READS.fetch_add(1, Ordering::Relaxed);
        let s = std::slice::from_raw_parts(p, size / 2);
        let n = s.iter().position(|&c| c == 0).unwrap_or(s.len());
        let out = String::from_utf16_lossy(&s[..n]);
        let _ = GlobalUnlock(g);
        Some(out)
    }
}

/// (url, is a video page)
pub type OnUrl = Box<dyn Fn(String, bool) + Send>;
/// the caught extensions + whether video pages count (yt-dlp installed) when the monitor is on
pub type Enabled = Box<dyn Fn() -> Option<(Vec<String>, bool)> + Send>;

/// run the monitor on its own thread for the life of the process. `enabled()` returns the caught extensions
/// when the monitor is on (None = off: nothing is opened or read).
#[cfg(unix)]
pub fn spawn(_enabled: Enabled, _on_url: OnUrl) {}

#[cfg(windows)]
pub fn spawn(enabled: Enabled, on_url: OnUrl) {
    std::thread::spawn(move || unsafe {
        let Ok(hwnd) =
            CreateWindowExW(Default::default(), w!("STATIC"), w!("GDM clipboard"), WINDOW_STYLE(0), 0, 0, 0, 0, Some(HWND_MESSAGE), None, None, None)
        else {
            return;
        };
        if AddClipboardFormatListener(hwnd).is_err() {
            return;
        }
        let mut last_seq = 0u32;
        let mut last_url = String::new();
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message != WM_CLIPBOARDUPDATE {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
                continue;
            }
            let Some((exts, video)) = enabled() else { continue };
            let seq = GetClipboardSequenceNumber();
            if seq == last_seq {
                continue;
            }
            last_seq = seq;
            // the clipboard can be busy for a moment (another program is writing it)
            let mut opened = false;
            for _ in 0..10 {
                if OpenClipboard(Some(hwnd)).is_ok() {
                    opened = true;
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(30));
            }
            if !opened {
                continue;
            }
            let text = read_allowed();
            let _ = CloseClipboard();
            let file = text.as_deref().and_then(|t| pick(t, &exts)).map(|u| (u, false));
            let page = if video { text.as_deref().and_then(pick_video).map(|u| (u, true)) } else { None };
            if let Some((u, is_video)) = file.or(page) {
                if u != last_url {
                    last_url = u.clone();
                    on_url(u, is_video);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn exts() -> Vec<String> {
        ["zip", "mp4", "exe"].map(String::from).to_vec()
    }

    #[test]
    fn i01_only_one_url_with_a_caught_extension() {
        assert_eq!(pick("https://e.com/a/file.ZIP", &exts()).as_deref(), Some("https://e.com/a/file.ZIP"));
        assert_eq!(pick("  http://e.com/v.mp4\r\n", &exts()).as_deref(), Some("http://e.com/v.mp4"));
        assert_eq!(pick("https://e.com/big.zip.001", &exts()).as_deref(), Some("https://e.com/big.zip.001"), "a split volume of a caught type");
        assert_eq!(pick("https://e.com/big.z01", &exts()).as_deref(), Some("https://e.com/big.z01"));
        for no in [
            "hunter2",
            "my password is https://e.com/a.zip",
            "https://e.com/page.html",
            "https://e.com/a.php?f=x.zip",
            "ftp://e.com/a.zip",
            "https://user:pw@e.com/a.zip",
            "https://e.com/a.zip https://e.com/b.zip",
            "123456",
        ] {
            assert!(pick(no, &exts()).is_none(), "{no}");
        }
        assert!(pick_video("https://www.youtube.com/watch?v=abc").is_some() && pick_video("https://youtu.be/abc").is_some());
        for no in ["https://youtube.com.evil.net/watch", "https://notyoutube.com/x", "https://youtu.be/", "hunter2"] {
            assert!(pick_video(no).is_none(), "{no}");
        }
    }

    #[cfg(windows)]
    unsafe fn put(text: &str, exclude: bool) {
        use windows::Win32::System::DataExchange::{EmptyClipboard, SetClipboardData};
        use windows::Win32::System::Memory::{GlobalAlloc, GMEM_MOVEABLE};
        for _ in 0..50 {
            if OpenClipboard(None).is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let _ = EmptyClipboard();
        let w: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let g = GlobalAlloc(GMEM_MOVEABLE, w.len() * 2).unwrap();
        let p = GlobalLock(g) as *mut u16;
        std::ptr::copy_nonoverlapping(w.as_ptr(), p, w.len());
        let _ = GlobalUnlock(g);
        let _ = SetClipboardData(CF_UNICODETEXT.0 as u32, Some(windows::Win32::Foundation::HANDLE(g.0)));
        if exclude {
            let f = RegisterClipboardFormatW(w!("ExcludeClipboardContentFromMonitorProcessing"));
            let g2 = GlobalAlloc(GMEM_MOVEABLE, 1).unwrap();
            let _ = SetClipboardData(f, Some(windows::Win32::Foundation::HANDLE(g2.0)));
        }
        let _ = CloseClipboard();
    }

    #[cfg(windows)]
    unsafe fn current() -> Option<String> {
        if OpenClipboard(None).is_err() {
            return None;
        }
        let r = (|| {
            let h = GetClipboardData(CF_UNICODETEXT.0 as u32).ok()?;
            let g = HGLOBAL(h.0);
            let p = GlobalLock(g) as *const u16;
            let s = std::slice::from_raw_parts(p, GlobalSize(g) / 2);
            let n = s.iter().position(|&c| c == 0).unwrap_or(s.len());
            let out = String::from_utf16_lossy(&s[..n]);
            let _ = GlobalUnlock(g);
            Some(out)
        })();
        let _ = CloseClipboard();
        r
    }

    /// I-01 on the real Windows clipboard: content marked "exclude from monitor processing" is never read
    /// (the text-read counter does not move); an ordinary URL copy is proposed once. The user's clipboard text is
    /// put back afterwards.
    #[test]
    #[cfg(windows)]
    fn i01_excluded_clipboard_is_never_read() {
        let saved = unsafe { current() };
        let (tx, rx) = std::sync::mpsc::channel::<String>();
        let tx = std::sync::Mutex::new(tx);
        spawn(Box::new(|| Some((["zip"].map(String::from).to_vec(), false))), Box::new(move |u, _| tx.lock().unwrap().send(u).unwrap()));
        std::thread::sleep(std::time::Duration::from_millis(300));
        let before = TEXT_READS.load(Ordering::Relaxed);
        unsafe { put("https://e.com/secret-vault-export.zip", true) };
        assert!(rx.recv_timeout(std::time::Duration::from_millis(1500)).is_err(), "excluded content proposed");
        assert_eq!(TEXT_READS.load(Ordering::Relaxed), before, "excluded content was read");
        unsafe { put("https://e.com/ok.zip", false) };
        assert_eq!(rx.recv_timeout(std::time::Duration::from_secs(3)).unwrap(), "https://e.com/ok.zip");
        unsafe { put(&saved.unwrap_or_default(), false) };
    }
}
