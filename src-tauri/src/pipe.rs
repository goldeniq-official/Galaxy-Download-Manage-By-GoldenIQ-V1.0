// SPDX-License-Identifier: RPL-1.5
//! Named pipe server for gdm-bridge (ADR-2: no localhost port). B-02: FILE_FLAG_FIRST_PIPE_INSTANCE (a squatter
//! that came first makes GDM refuse to serve and say so), protected DACL user + SYSTEM, remote clients rejected,
//! name carries the user SID. One request per connection, message mode, 256 KiB max.
use crate::app::{lock, App};
use gdm_bridge::{Envelope, Msg, MAX_MSG};
use serde_json::{json, Value};
use std::sync::Arc;
use windows::core::HSTRING;
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_MORE_DATA, ERROR_PIPE_CONNECTED, HANDLE};
use windows::Win32::Security::Authorization::{ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1};
use windows::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};
use windows::Win32::Storage::FileSystem::{FlushFileBuffers, ReadFile, WriteFile, FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_DUPLEX};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_MESSAGE, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_MESSAGE, PIPE_WAIT,
};

fn create(name: &str) -> Result<HANDLE, u32> {
    let sid = gdm_bridge::user_sid().ok_or(0u32)?;
    unsafe {
        let mut sd = PSECURITY_DESCRIPTOR::default();
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            &HSTRING::from(format!("D:P(A;;GA;;;{sid})(A;;GA;;;SY)")),
            SDDL_REVISION_1,
            &mut sd,
            None,
        )
        .map_err(|_| 1u32)?;
        // the descriptor lives as long as the process (one pipe, created once)
        let sa = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: sd.0,
            bInheritHandle: false.into(),
        };
        let h = CreateNamedPipeW(
            &HSTRING::from(name),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
            PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
            1,
            MAX_MSG,
            MAX_MSG,
            0,
            Some(&sa),
        );
        if h.is_invalid() {
            Err(GetLastError().0)
        } else {
            Ok(h)
        }
    }
}

/// start the server thread; Err = the pipe name is already taken (another GDM, or a squatter: B-02)
pub fn serve(app: Arc<App>, rt: tauri::async_runtime::RuntimeHandle) -> Result<(), String> {
    let h = create(&gdm_bridge::pipe_name()).map_err(|e| format!("pipe-taken:{e}"))?;
    let h = h.0 as isize;
    std::thread::spawn(move || {
        let h = HANDLE(h as _);
        let mut buf = vec![0u8; MAX_MSG as usize];
        loop {
            unsafe {
                if let Err(e) = ConnectNamedPipe(h, None) {
                    if e.code() != ERROR_PIPE_CONNECTED.to_hresult() {
                        continue;
                    }
                }
                let mut n = 0u32;
                let reply = match ReadFile(h, Some(&mut buf), Some(&mut n), None) {
                    Err(e) if e.code() == ERROR_MORE_DATA.to_hresult() => json!({"ok": false, "error": "too-large"}),
                    Err(_) => {
                        let _ = DisconnectNamedPipe(h);
                        continue;
                    }
                    Ok(()) => match serde_json::from_slice::<Envelope>(&buf[..n as usize]) {
                        // the engine validates again: the bridge is not a security boundary (B-08)
                        Ok(env) if env.msg.validate().is_ok() && (!matches!(env.msg, Msg::Cli { .. }) || client_is_gdm(h)) => {
                            let (a, m, o) = (app.clone(), env.msg, env.origin);
                            let reply = rt.block_on(async move { handle(&a, &o, m).await });
                            // B-09 crash-window test only: hold the reply after the job exists (never set in use)
                            if let Some(ms) = std::env::var("GDM_TEST_REPLY_DELAY_MS").ok().and_then(|v| v.parse::<u64>().ok()) {
                                std::thread::sleep(std::time::Duration::from_millis(ms.min(10_000)));
                            }
                            reply
                        }
                        _ => json!({"ok": false, "error": "rejected"}),
                    },
                };
                let b = serde_json::to_vec(&reply).unwrap_or_default();
                let _ = WriteFile(h, Some(&b), None, None);
                let _ = FlushFileBuffers(h);
                let _ = DisconnectNamedPipe(h);
            }
        }
        #[allow(unreachable_code)]
        unsafe {
            let _ = CloseHandle(h);
        }
    });
    Ok(())
}

/// I-02: a CLI message counts only when the connected client process IS this gdm.exe (same image path), read
/// through the process handle; the origin string alone is never trusted
fn client_is_gdm(h: HANDLE) -> bool {
    let mut pid = 0;
    let ok = unsafe { windows::Win32::System::Pipes::GetNamedPipeClientProcessId(h, &mut pid).is_ok() };
    let me = std::env::current_exe().ok();
    ok && me.is_some_and(|me| gdm_bridge::image_of(pid).is_some_and(|img| gdm_bridge::norm(&img) == gdm_bridge::norm(&me)))
}

/// glob with `*` only, case-insensitive (C-01 exception list)
pub fn glob(pat: &str, s: &str) -> bool {
    let (p, s) = (pat.to_ascii_lowercase(), s.to_ascii_lowercase());
    let parts: Vec<&str> = p.split('*').collect();
    let mut pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        match s[pos..].find(part) {
            Some(k) if i > 0 || k == 0 => pos += k + part.len(),
            _ => return false,
        }
    }
    p.ends_with('*') || pos == s.len()
}

/// the browser's session for one job: its Cookie, plus its User-Agent (becomes the job's UA, never a header)
fn browser_creds(cookie: Option<String>, ua: Option<String>) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = cookie.filter(|c| !c.is_empty()).map(|c| ("Cookie".to_string(), c)).into_iter().collect();
    v.extend(ua.filter(|u| !u.trim().is_empty()).map(|u| ("User-Agent".to_string(), u)));
    v
}

/// "download with GDM" links: each link with its credentials; the browser's cookie belongs to the origin of
/// `urls[0]` and only links of that same origin get it (the UA is no secret: every link gets it)
fn link_creds(urls: Vec<String>, cookie: Option<String>, ua: Option<String>) -> Vec<(String, Vec<(String, String)>)> {
    let origin_of = |u: &str| url::Url::parse(u).ok().map(|u| u.origin());
    let home = urls.first().and_then(|u| origin_of(u));
    let (with, ua_only) = (browser_creds(cookie, ua.clone()), browser_creds(None, ua));
    urls.into_iter()
        .map(|u| {
            let c = if home.is_some() && origin_of(&u) == home { with.clone() } else { ua_only.clone() };
            (u, c)
        })
        .collect()
}

pub async fn handle(app: &Arc<App>, _origin: &str, m: Msg) -> Value {
    match m {
        Msg::Hello { browser, .. } => {
            lock(&app.browsers_seen).insert(browser.chars().take(32).collect(), gdm_core::store::now());
            let s = lock(&app.settings).clone();
            json!({"ok": true, "ver": env!("CARGO_PKG_VERSION"), "catch": {
                "enabled": s.catch_enabled, "exts": s.catch_exts, "min_kb": s.catch_min_kb, "skip": s.catch_skip,
                "force": s.force_key, "bypass": s.bypass_key, "video": s.video_button }})
        }
        Msg::Status {} => {
            let rows = app.rows(None).unwrap_or_default();
            let recent: Vec<Value> =
                rows.iter().take(5).map(|r| json!({"name": r.row.name, "state": r.row.state, "done": r.row.done, "size": r.row.size})).collect();
            json!({"ok": true, "running": app.running(), "recent": recent})
        }
        Msg::Handoff { hid, url, referrer, name, cookie, ua, .. } => {
            let creds = browser_creds(cookie, ua);
            // F06: the new link a refresh ticket waits for (checked before the catch rules: the user asked for it)
            if let Some(job) = app.refresh_take(&url, referrer.as_deref(), creds.clone()) {
                crate::show_main(app, true);
                return json!({"ok": true, "id": job, "started": false, "refresh": true});
            }
            let s = lock(&app.settings).clone();
            // the engine applies its own policy too (B-04): catching off or an excepted site => browser keeps it
            if !s.catch_enabled || s.catch_skip.iter().any(|p| glob(p, url.split("://").nth(1).unwrap_or(""))) {
                return json!({"ok": false, "error": "not-caught"});
            }
            // admission bound (council G6): a hijacked extension cannot pile up unbounded jobs or mappings
            {
                let mut m = lock(&app.handoff_jobs);
                m.retain(|_, (_, at)| at.elapsed() < std::time::Duration::from_secs(600)); // a dup comes within seconds
                if m.contains_key(&hid) || m.len() >= 1000 || lock(&app.pending).len() >= 200 {
                    return json!({"ok": false, "error": "busy"}); // reused id or too many open proposals
                }
            }
            let start = app.handoff_allowed();
            match app.add_and_start(&url, name, referrer, creds, false, start) {
                Ok(id) => {
                    lock(&app.handoff_jobs).insert(hid, (id.clone(), std::time::Instant::now()));
                    if s.show_info || !start {
                        lock(&app.pending).push(id.clone());
                        crate::show_main(app, true);
                    }
                    json!({"ok": true, "id": id, "started": start})
                }
                Err(e) => json!({"ok": false, "error": e}),
            }
        }
        Msg::Dup { hid } => {
            let id = lock(&app.handoff_jobs).remove(&hid);
            if let Some((id, _)) = id {
                let _ = app.info_cancel(&id).await;
            }
            json!({"ok": true})
        }
        Msg::Cli { cmd, url, dir, name, add_only, silent, quit_after } => {
            crate::cli::serve(app, &cmd, url, dir, name, add_only, silent, quit_after).await
        }
        Msg::Video { page } => {
            // never starts anything: the app shows "download this video?" and the quality list on request
            app.push_toast("video-link", "", &page);
            crate::show_main(app, true);
            json!({"ok": true})
        }
        Msg::Links { urls, referrer, cookie, ua } => {
            if lock(&app.pending).len() + urls.len() > 600 {
                return json!({"ok": false, "error": "busy"});
            }
            let mut ids = vec![];
            for (u, creds) in link_creds(urls, cookie, ua) {
                if let Ok(id) = app.add_and_start(&u, None, referrer.clone(), creds, false, false) {
                    ids.push(id);
                }
            }
            lock(&app.pending).extend(ids.iter().cloned());
            crate::show_main(app, true);
            json!({"ok": true, "count": ids.len()})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// IDM parity: link-menu links carry the browser session, but the cookie only reaches links of its own origin;
    /// every link (and a caught download) keeps the browser's UA
    #[test]
    fn links_cookie_stays_with_its_origin() {
        let urls = [
            "https://site.example/a.zip",
            "https://cdn.other.example/b.zip",
            "http://site.example/c.zip",
            "https://site.example:8443/d",
            "https://site.example/e",
        ];
        let got = link_creds(urls.iter().map(|s| s.to_string()).collect(), Some("sid=SECRET".into()), Some("Mozilla/5.0 test".into()));
        let has = |i: usize, k: &str| got[i].1.iter().any(|(kk, _)| kk == k);
        assert_eq!((0..5).filter(|&i| has(i, "Cookie")).collect::<Vec<_>>(), vec![0, 4], "same scheme+host+port only: {got:?}");
        assert!((0..5).all(|i| has(i, "User-Agent")));
        assert_eq!(browser_creds(Some(String::new()), Some(" ".into())), vec![], "empty values are dropped");
        assert_eq!(link_creds(vec!["not a url".into(), "https://x.example/".into()], Some("c=1".into()), None)[1].1, vec![], "no origin, no cookie");
    }

    #[test]
    fn c01_exception_glob() {
        assert!(glob("*.reddit.com/compression-dictionary*", "www.reddit.com/compression-dictionary.dat?x=1"));
        assert!(!glob("*.reddit.com/compression-dictionary*", "www.reddit.com/r/rust/a.zip"));
        assert!(glob("*.tonec.com*", "dl.tonec.com/idm.exe"));
        assert!(!glob("example.com*", "evil.example.com/a"));
        assert!(glob("example.com*", "example.com/a"));
    }
}
