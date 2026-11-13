// SPDX-License-Identifier: RPL-1.5
//! Windows proxy settings (IDM "Get System" / PAC, §04 Proxy row): the user's Internet Options (manual proxy,
//! bypass list, auto-detect/WPAD, PAC URL) read through WinHTTP, and PAC evaluation per destination by
//! WinHttpGetProxyForUrl (Windows runs the script, GDM never does). The result is routes for the policy relay:
//! a PAC answer is routing input only, never permission to skip the destination checks (council G7).
//! macOS / Linux: the conventional HTTPS_PROXY / HTTP_PROXY / ALL_PROXY / NO_PROXY variables; a PAC script is
//! not evaluated there (no system evaluator to delegate to), so a PAC setting blocks instead of going DIRECT.
use crate::relay::{ProxyAuth, Route, Router};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
#[cfg(windows)]
use windows::core::{HSTRING, PCWSTR, PWSTR};
#[cfg(windows)]
use windows::Win32::Foundation::{GlobalFree, HGLOBAL};
#[cfg(windows)]
use windows::Win32::Networking::WinHttp::*;

#[cfg(windows)]
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
    /// a configured proxy GDM cannot use safely (e.g. TLS to the proxy): block, never fall back to plaintext
    pub unsupported: Option<String>,
}

/// the current user's Internet Options proxy settings
#[cfg(windows)]
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
        unsupported: None,
    }
}

/// macOS / Linux: the proxy environment variables (lower case first, as curl reads them), as one list in the
/// WinHTTP "http=a:1;https=b:2;socks=c:3" form; NO_PROXY becomes the bypass list
#[cfg(unix)]
pub fn ie_config() -> IeConfig {
    env_config(&|k| std::env::var(k).ok())
}

#[cfg(unix)]
fn env_config(get: &dyn Fn(&str) -> Option<String>) -> IeConfig {
    let var = |k: &str| get(&k.to_ascii_lowercase()).or_else(|| get(&k.to_ascii_uppercase())).filter(|v| !v.trim().is_empty());
    // "http://user@host:3128/" -> ("http", "host:3128"): the scheme is kept (a TLS proxy is never spoken to in
    // plaintext); credentials never come from the environment (the Proxy settings own them)
    let parse = |v: String| {
        let v = v.trim().to_string();
        let (scheme, rest) = v.split_once("://").map(|(s, r)| (s.to_ascii_lowercase(), r.to_string())).unwrap_or(("http".into(), v));
        let rest = rest.rsplit_once('@').map(|(_, h)| h.to_string()).unwrap_or(rest);
        (scheme, rest.trim_end_matches('/').to_string())
    };
    let mut list: Vec<String> = vec![];
    let mut unsupported = None;
    let mut add = |list: &mut Vec<String>, for_scheme: &str, v: String| {
        let (scheme, hp) = parse(v);
        match scheme.as_str() {
            "http" => list.push(format!("{for_scheme}={hp}")),
            "socks" | "socks5" | "socks5h" => list.push(format!("socks={hp}")),
            s => unsupported = Some(format!("proxy scheme {s}:// is not supported: set the proxy in Settings")),
        }
    };
    if let Some(v) = var("http_proxy") {
        add(&mut list, "http", v);
    }
    if let Some(v) = var("https_proxy") {
        add(&mut list, "https", v);
    }
    if let Some(v) = var("all_proxy") {
        if !list.iter().any(|x| x.starts_with("http=")) {
            add(&mut list, "http", v.clone());
        }
        if !list.iter().any(|x| x.starts_with("https=") || x.starts_with("socks=")) {
            add(&mut list, "https", v);
        }
    }
    list.dedup();
    let bypass = var("no_proxy")
        .map(|b| {
            b.split(',')
                .map(|x| x.trim().trim_start_matches('.'))
                .filter(|x| !x.is_empty())
                .map(|x| if x == "*" { x.to_string() } else { format!("*.{x}") })
                .flat_map(|w| [w.clone(), w.trim_start_matches("*.").to_string()])
                .collect()
        })
        .unwrap_or_default();
    IeConfig { auto_detect: false, pac_url: None, proxy: (!list.is_empty()).then(|| list.join(";")), bypass, unsupported }
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

/// PAC / WPAD: no system evaluator on macOS / Linux, and GDM never runs the script itself: block (fail closed)
#[cfg(unix)]
pub fn pac_routes(_pac_url: Option<&str>, _auto_detect: bool, _dest: &str, _auth: &ProxyAuth) -> Option<Vec<Route>> {
    Some(vec![Route::Block("pac-unsupported: set the proxy in Settings".into())])
}

/// PAC / WPAD for one destination URL (Windows evaluates the script)
#[cfg(windows)]
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

/// the PAC script's own text (<= 1 MiB, 10 s), fetched once by GDM to see what Windows will not tell it
fn pac_text(url: &str) -> Option<String> {
    let u = url::Url::parse(url).ok()?;
    if !matches!(u.scheme(), "http" | "https") {
        return None;
    }
    // the PAC host is the user's own configuration: it may be on the LAN
    let policy = Arc::new(crate::net::Policy { allow_private: u.host_str().map(|h| vec![h.to_string()]).unwrap_or_default() });
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().ok()?;
        rt.block_on(async move {
            let c = crate::net::client(policy, false, Duration::from_secs(10)).ok()?;
            let mut r = tokio::time::timeout(Duration::from_secs(10), c.get(u).send()).await.ok()?.ok()?;
            let mut b = Vec::new();
            while let Some(ch) = r.chunk().await.ok()? {
                if b.len() + ch.len() > 1 << 20 {
                    return None;
                }
                b.extend_from_slice(&ch);
            }
            Some(String::from_utf8_lossy(&b).into_owned())
        })
    })
    .join()
    .ok()
    .flatten()
}

/// PAC/WPAD routes per host (cached 5 minutes). WinHTTP turns a PAC "SOCKS" answer into DIRECT (measured in
/// g7_pac): when the script names SOCKS at all, a DIRECT answer is refused instead (fail closed, council G7),
/// and the user is told to set the SOCKS proxy by hand. A script that cannot be read counts as naming SOCKS.
pub fn pac_router(pac_url: Option<String>, auto_detect: bool, auth: ProxyAuth) -> Router {
    let socks_in_pac = match &pac_url {
        Some(u) => pac_text(u).is_none_or(|t| t.to_ascii_uppercase().contains("SOCKS")),
        None => false, // WPAD: the script location is found by Windows only (documented limit)
    };
    let cache: Mutex<HashMap<String, (Instant, Vec<Route>)>> = Mutex::new(HashMap::new());
    Arc::new(move |u: &url::Url| {
        // a tunnel shows only scheme://host:port/ (council G7: per-URL PAC rules on the path cannot apply)
        let key = format!("{}://{}/", u.scheme(), u.host_str().unwrap_or(""));
        if let Some((t, r)) = cache.lock().unwrap_or_else(|p| p.into_inner()).get(&key) {
            if t.elapsed() < Duration::from_secs(300) {
                return r.clone();
            }
        }
        let mut r = pac_routes(pac_url.as_deref(), auto_detect, u.as_str(), &auth).unwrap_or_else(|| vec![Route::Direct]);
        if socks_in_pac {
            r = r
                .into_iter()
                .map(|x| if x == Route::Direct { Route::Block("pac-socks: set the SOCKS proxy in Settings".into()) } else { x })
                .collect();
        }
        cache.lock().unwrap_or_else(|p| p.into_inner()).insert(key, (Instant::now(), r.clone()));
        r
    })
}

/// the router for "use the system proxy": manual proxy + bypass, or PAC/WPAD per host
pub fn system_router(auth: ProxyAuth) -> Router {
    system_router_from(ie_config(), auth)
}

fn system_router_from(cfg: IeConfig, auth: ProxyAuth) -> Router {
    let pac = (cfg.pac_url.is_some() || cfg.auto_detect).then(|| pac_router(cfg.pac_url.clone(), cfg.auto_detect, auth.clone()));
    Arc::new(move |u: &url::Url| {
        if let Some(why) = &cfg.unsupported {
            return vec![Route::Block(why.clone())];
        }
        let host = u.host_str().unwrap_or("");
        if crate::relay::bypassed(&cfg.bypass, host) {
            return vec![Route::Direct];
        }
        if let Some(p) = &pac {
            return p(u);
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
    #[cfg(unix)]
    #[test]
    fn unix_env_proxy() {
        let env = |v: &[(&str, &str)]| {
            let m: HashMap<String, String> = v.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
            env_config(&move |k| m.get(k).cloned())
        };
        let c = env(&[("HTTPS_PROXY", "http://user:pw@corp:3128/"), ("no_proxy", "localhost,.corp.local")]);
        assert_eq!(c.proxy.as_deref(), Some("https=corp:3128"), "no credentials taken from the environment");
        assert!(crate::relay::bypassed(&c.bypass, "a.corp.local") && crate::relay::bypassed(&c.bypass, "localhost"));
        assert!(!crate::relay::bypassed(&c.bypass, "corp.local.evil.com"));
        assert_eq!(env(&[("all_proxy", "socks5://s:1080")]).proxy.as_deref(), Some("socks=s:1080"));
        let tls = env(&[("https_proxy", "https://p:443")]);
        assert!(tls.unsupported.is_some() && tls.proxy.is_none(), "a TLS proxy is refused, never used in plaintext");
        assert!(matches!(system_router_from(tls, ProxyAuth::None)(&url::Url::parse("https://e.com/").unwrap())[0], Route::Block(_)));
        assert_eq!(env(&[]).proxy, None);
        assert!(matches!(pac_routes(Some("http://wpad/x.pac"), false, "https://e.com/", &ProxyAuth::None).unwrap()[0], Route::Block(_)));
    }
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
