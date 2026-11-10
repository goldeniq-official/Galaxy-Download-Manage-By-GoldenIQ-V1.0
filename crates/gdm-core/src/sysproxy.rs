// SPDX-License-Identifier: RPL-1.5
//! Windows proxy settings (IDM "Get System" / PAC, §04 Proxy row): the user's Internet Options (manual proxy,
//! bypass list, auto-detect/WPAD, PAC URL) read through WinHTTP, and PAC evaluation per destination by
//! WinHttpGetProxyForUrl (Windows runs the script, GDM never does). The result is routes for the policy relay:
//! a PAC answer is routing input only, never permission to skip the destination checks (council G7).
use crate::relay::{ProxyAuth, Route, Router};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::Foundation::{GlobalFree, HGLOBAL};
use windows::Win32::Networking::WinHttp::*;

fn take(p: PWSTR) -> Option<String> {
    if p.is_null() {
        return None;
    }
    let s = unsafe { p.to_string().ok() };
    unsafe {
        let _ = GlobalFree(Some(HGLOBAL(p.0 as _)));
    }
    s.filter(|s| !s.trim().is_empty())
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct IeConfig {
    pub auto_detect: bool,
    pub pac_url: Option<String>,
    pub proxy: Option<String>,
    pub bypass: Vec<String>,
}

/// the current user's Internet Options proxy settings
pub fn ie_config() -> IeConfig {
    let mut c = WINHTTP_CURRENT_USER_IE_PROXY_CONFIG::default();
    if unsafe { WinHttpGetIEProxyConfigForCurrentUser(&mut c) }.is_err() {
        return IeConfig::default();
    }
    IeConfig {
        auto_detect: c.fAutoDetect.as_bool(),
        pac_url: take(c.lpszAutoConfigUrl),
        proxy: take(c.lpszProxy),
        bypass: take(c.lpszProxyBypass).map(|b| b.split([';', ' ']).filter(|x| !x.is_empty()).map(String::from).collect()).unwrap_or_default(),
    }
}

/// "PROXY a:1; SOCKS b:2; DIRECT" or WinHTTP's "a:1;b:2" / "http=a:1;https=b:2" => routes for `scheme`
pub fn parse_list(list: &str, scheme: &str, auth: &ProxyAuth) -> Vec<Route> {
    let mut out = vec![];
    for item in list.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        let up = item.to_ascii_uppercase();
        let (kind, rest) = if let Some(r) = up.strip_prefix("PROXY ").map(|_| &item[6..]) {
            ("http", r)
        } else if let Some(r) = up.strip_prefix("SOCKS5 ").map(|_| &item[7..]) {
            ("socks", r)
        } else if let Some(r) = up.strip_prefix("SOCKS ").map(|_| &item[6..]) {
            ("socks", r)
        } else if up == "DIRECT" {
            out.push(Route::Direct);
            continue;
        } else if let Some((k, v)) = item.split_once('=') {
            // per protocol: http=..., https=..., socks=...
            match k.trim().to_ascii_lowercase().as_str() {
                s if s == scheme => ("http", v),
                "socks" => ("socks", v),
                _ => continue,
            }
        } else {
            ("http", item)
        };
        let hp = crate::relay::hostport(rest.trim(), if kind == "socks" { 1080 } else { 80 });
        out.push(if kind == "socks" { Route::Socks5(hp, None) } else { Route::Http(hp, auth.clone()) });
    }
    out
}

/// PAC / WPAD for one destination URL (Windows evaluates the script)
pub fn pac_routes(pac_url: Option<&str>, auto_detect: bool, dest: &str, auth: &ProxyAuth) -> Option<Vec<Route>> {
    unsafe {
        let s = WinHttpOpen(&HSTRING::from("GDM"), WINHTTP_ACCESS_TYPE_NO_PROXY, PCWSTR::null(), PCWSTR::null(), 0);
        if s.is_null() {
            return None;
        }
        let pac = pac_url.map(HSTRING::from);
        let mut opt = WINHTTP_AUTOPROXY_OPTIONS::default();
        if let Some(p) = &pac {
            opt.dwFlags |= WINHTTP_AUTOPROXY_CONFIG_URL;
            opt.lpszAutoConfigUrl = PCWSTR(p.as_ptr());
        }
        if auto_detect {
            opt.dwFlags |= WINHTTP_AUTOPROXY_AUTO_DETECT;
            opt.dwAutoDetectFlags = WINHTTP_AUTO_DETECT_TYPE_DHCP | WINHTTP_AUTO_DETECT_TYPE_DNS_A;
        }
        let mut info = WINHTTP_PROXY_INFO::default();
        let r = WinHttpGetProxyForUrl(s, &HSTRING::from(dest), &mut opt, &mut info);
        let _ = WinHttpCloseHandle(s);
        r.ok()?;
        let list = take(info.lpszProxy);
        let _ = take(info.lpszProxyBypass);
        let scheme = dest.split(':').next().unwrap_or("http");
        Some(match list {
            Some(l) if info.dwAccessType == WINHTTP_ACCESS_TYPE_NAMED_PROXY => parse_list(&l, scheme, auth),
            _ => vec![Route::Direct],
        })
    }
}

/// the router for "use the system proxy": manual proxy + bypass, or PAC/WPAD per host (cached 5 minutes);
/// a PAC that cannot be fetched or run => DIRECT (like browsers), recorded in the relay's routes
pub fn system_router(auth: ProxyAuth) -> Router {
    let cfg = ie_config();
    let cache: Mutex<HashMap<String, (Instant, Vec<Route>)>> = Mutex::new(HashMap::new());
    Arc::new(move |u: &url::Url| {
        let host = u.host_str().unwrap_or("");
        if crate::relay::bypassed(&cfg.bypass, host) {
            return vec![Route::Direct];
        }
        if cfg.pac_url.is_some() || cfg.auto_detect {
            // a tunnel shows only scheme://host:port/ (council G7: per-URL PAC rules on the path cannot apply)
            let key = format!("{}://{}/", u.scheme(), u.host_str().unwrap_or(""));
            if let Some((t, r)) = cache.lock().unwrap_or_else(|p| p.into_inner()).get(&key) {
                if t.elapsed() < Duration::from_secs(300) {
                    return r.clone();
                }
            }
            let r = pac_routes(cfg.pac_url.as_deref(), cfg.auto_detect, u.as_str(), &auth).unwrap_or_else(|| vec![Route::Direct]);
            cache.lock().unwrap_or_else(|p| p.into_inner()).insert(key, (Instant::now(), r.clone()));
            return r;
        }
        match &cfg.proxy {
            Some(p) => parse_list(p, u.scheme(), &auth),
            None => vec![Route::Direct],
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn proxy_lists() {
        let a = ProxyAuth::None;
        assert_eq!(
            parse_list("PROXY p1:3128; SOCKS s:1080; DIRECT", "https", &a),
            vec![Route::Http("p1:3128".into(), a.clone()), Route::Socks5("s:1080".into(), None), Route::Direct]
        );
        assert_eq!(parse_list("http=h:8080;https=s:8443", "https", &a), vec![Route::Http("s:8443".into(), a.clone())]);
        assert_eq!(parse_list("corp:8080", "http", &a), vec![Route::Http("corp:8080".into(), a.clone())]);
        assert_eq!(parse_list("PROXY corp", "http", &a), vec![Route::Http("corp:80".into(), a.clone())]);
        assert!(crate::relay::bypassed(&["<local>".into(), "*.corp.local".into()], "intranet"));
        assert!(
            crate::relay::bypassed(&["*.corp.local".into()], "a.corp.local")
                && !crate::relay::bypassed(&["*.corp.local".into()], "corp.local.evil.com")
        );
    }
}
