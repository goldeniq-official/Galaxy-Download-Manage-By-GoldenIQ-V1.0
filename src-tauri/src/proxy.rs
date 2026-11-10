// SPDX-License-Identifier: RPL-1.5
//! Applies the proxy setting: no proxy => direct as before; system or manual => every engine request goes
//! through gdm-core's policy relay with a router built from the setting (relay.rs, sysproxy.rs).
use crate::app::ProxySettings;
use gdm_core::relay::{self, ProxyAuth, Relay, Route, Router};
use std::sync::{Arc, Mutex};

static LAST: Mutex<Option<ProxySettings>> = Mutex::new(None);

pub fn router(p: &ProxySettings) -> Option<Router> {
    let auth = match p.auth.as_str() {
        "basic" => crate::creds::load("proxy").map(|(u, pw)| ProxyAuth::Basic(relay::basic(&u, &pw))).unwrap_or(ProxyAuth::None),
        "windows" => ProxyAuth::Sspi("Negotiate".into()),
        _ => ProxyAuth::None,
    };
    match p.mode.as_str() {
        "system" => Some(gdm_core::sysproxy::system_router(auth)),
        "manual" => {
            let (http, socks, bypass) = (p.http.trim().to_string(), p.socks.trim().to_string(), p.bypass.clone());
            if http.is_empty() && socks.is_empty() {
                return None;
            }
            Some(Arc::new(move |u: &url::Url| {
                if relay::bypassed(&bypass, u.host_str().unwrap_or("")) {
                    return vec![Route::Direct];
                }
                if !socks.is_empty() {
                    return vec![Route::Socks5(relay::hostport(&socks, 1080), None)];
                }
                vec![Route::Http(relay::hostport(&http, 80), auth.clone())]
            }))
        }
        _ => None,
    }
}

/// (re)start the relay when the setting changed; the relay itself is created on the async runtime
pub fn apply(p: &ProxySettings) {
    {
        let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
        if last.as_ref() == Some(p) {
            return;
        }
        *last = Some(p.clone());
    }
    match router(p) {
        None => relay::set_global(None),
        Some(r) => {
            if let Some(cur) = relay::global() {
                cur.set_router(r);
                return;
            }
            tauri::async_runtime::spawn(async move {
                if let Ok(rl) = Relay::start(r).await {
                    relay::set_global(Some(rl));
                }
            });
        }
    }
}
