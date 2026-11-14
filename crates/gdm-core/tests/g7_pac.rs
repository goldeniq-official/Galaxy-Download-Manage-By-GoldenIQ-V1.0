// SPDX-License-Identifier: RPL-1.5
//! G7 proxy row: a PAC script evaluated by Windows (WinHttpGetProxyForUrl), per destination host.
use gdm_core::relay::{ProxyAuth, Route};
#[cfg(windows)]
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// macOS / Linux have no system PAC evaluator and GDM never runs the script: every destination is blocked
#[cfg(unix)]
#[test]
fn pac_blocked_fail_closed_on_unix() {
    let r = gdm_core::sysproxy::pac_routes(Some("http://127.0.0.1:1/proxy.pac"), true, "https://files.example.com/", &ProxyAuth::None);
    assert!(matches!(r.as_deref(), Some([Route::Block(_)])), "{r:?}");
}

#[cfg(windows)]
#[tokio::test(flavor = "multi_thread")]
async fn pac_script_routes_per_host() {
    let pac = r#"function FindProxyForURL(url, host) {
        if (dnsDomainIs(host, ".intranet.test") || host == "intranet.test") return "DIRECT";
        if (shExpMatch(host, "*.socks.test")) return "SOCKS 127.0.0.1:1080";
        return "PROXY 127.0.0.1:3128; DIRECT";
    }"#;
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = l.local_addr().unwrap().port();
    tokio::spawn(async move {
        loop {
            let (mut s, _) = l.accept().await.unwrap();
            let mut b = [0u8; 4096];
            let _ = s.read(&mut b).await;
            let r = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/x-ns-proxy-autoconfig\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{pac}",
                pac.len()
            );
            let _ = s.write_all(r.as_bytes()).await;
        }
    });
    let url = format!("http://127.0.0.1:{port}/proxy.pac");
    let a = ProxyAuth::None;
    let go = |dest: &'static str| {
        let (u, a) = (url.clone(), a.clone());
        tokio::task::spawn_blocking(move || gdm_core::sysproxy::pac_routes(Some(&u), false, dest, &a))
    };
    let r = go("https://files.example.com/").await.unwrap();
    // WinHTTP returns the proxies of a "PROXY a; DIRECT" answer and drops the DIRECT fallback
    assert_eq!(r, Some(vec![Route::Http("127.0.0.1:3128".into(), ProxyAuth::None)]), "{r:?}");
    assert_eq!(go("http://a.intranet.test/").await.unwrap(), Some(vec![Route::Direct]));
    let s = go("https://x.socks.test/").await.unwrap();
    assert!(s.is_some(), "PAC answered");
    eprintln!("WinHTTP for a SOCKS PAC answer: {s:?}");

    // fail closed (council G7): the script names SOCKS, so the DIRECT that WinHTTP returns for it is refused
    let r = gdm_core::sysproxy::pac_router(Some(url.clone()), false, ProxyAuth::None);
    let socks_route = tokio::task::spawn_blocking(move || r(&url::Url::parse("https://x.socks.test/").unwrap())).await.unwrap();
    assert!(matches!(socks_route.as_slice(), [Route::Block(_)]), "{socks_route:?}");
}
