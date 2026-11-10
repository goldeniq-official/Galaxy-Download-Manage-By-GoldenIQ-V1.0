// SPDX-License-Identifier: RPL-1.5
//! G7 proxy through GDM's policy relay (council G7, W-02 + W-05): the upstream proxy only ever receives a
//! checked IP (CONNECT ip:port / SOCKS5 with an IP), NTLM runs its 3 legs on ONE socket, private destinations
//! are refused unless the user asked for them. One test function: the relay is process-wide.
mod common;
use common::*;
use gdm_core::relay::{self, ProxyAuth, Relay, Route};
use gdm_core::Unverified;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Default)]
struct Seen {
    /// (connection number, CONNECT authority, Proxy-Authorization present)
    connects: Vec<(usize, String, bool)>,
    socks: Vec<String>,
}

async fn head(s: &mut TcpStream) -> Option<String> {
    let mut b = Vec::new();
    while !b.ends_with(b"\r\n\r\n") {
        let mut c = [0u8; 1];
        if s.read(&mut c).await.ok()? == 0 {
            return None;
        }
        b.push(c[0]);
    }
    Some(String::from_utf8_lossy(&b).to_string())
}

/// an HTTP CONNECT proxy: "none" | "basic" (user:pw) | "ntlm" (real SSPI server side, current user)
async fn http_proxy(mode: &'static str, seen: Arc<Mutex<Seen>>) -> String {
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = l.local_addr().unwrap().to_string();
    tokio::spawn(async move {
        let mut n = 0;
        loop {
            let (mut s, _) = l.accept().await.unwrap();
            n += 1;
            let seen = seen.clone();
            let conn = n;
            tokio::spawn(async move {
                let mut srv = (mode == "ntlm").then(|| gdm_auth::TestServer::new("NTLM").unwrap());
                loop {
                    let Some(h) = head(&mut s).await else { return };
                    let first = h.lines().next().unwrap_or("").to_string();
                    let authority = first.split(' ').nth(1).unwrap_or("").to_string();
                    let auth = h.lines().find_map(|l| l.strip_prefix("Proxy-Authorization: ")).map(String::from);
                    seen.lock().unwrap().connects.push((conn, authority.clone(), auth.is_some()));
                    let ok = match mode {
                        "basic" => auth.as_deref() == Some(&relay::basic("alice", "pw")),
                        "ntlm" => match auth.as_deref().and_then(|a| a.strip_prefix("NTLM ")) {
                            None => {
                                s.write_all(b"HTTP/1.1 407 Auth\r\nProxy-Authenticate: NTLM\r\nContent-Length: 0\r\n\r\n").await.unwrap();
                                continue;
                            }
                            Some(t) => match srv.as_mut().unwrap().step(&gdm_auth::unb64(t).unwrap()).unwrap() {
                                gdm_auth::Step::Continue(tok) => {
                                    let r =
                                        format!("HTTP/1.1 407 Auth\r\nProxy-Authenticate: NTLM {}\r\nContent-Length: 0\r\n\r\n", gdm_auth::b64(&tok));
                                    s.write_all(r.as_bytes()).await.unwrap();
                                    continue;
                                }
                                gdm_auth::Step::Done(_) => true,
                            },
                        },
                        _ => true,
                    };
                    if !ok {
                        s.write_all(b"HTTP/1.1 407 Auth\r\nProxy-Authenticate: Basic realm=\"p\"\r\nContent-Length: 0\r\n\r\n").await.unwrap();
                        continue;
                    }
                    let Ok(mut up) = TcpStream::connect(&authority).await else { return };
                    s.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await.unwrap();
                    let _ = tokio::io::copy_bidirectional(&mut s, &mut up).await;
                    return;
                }
            });
        }
    });
    addr
}

/// a SOCKS5 proxy without login; records the address type it was asked for
async fn socks_proxy(seen: Arc<Mutex<Seen>>) -> String {
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = l.local_addr().unwrap().to_string();
    tokio::spawn(async move {
        loop {
            let (mut s, _) = l.accept().await.unwrap();
            let seen = seen.clone();
            tokio::spawn(async move {
                let mut g = [0u8; 3];
                s.read_exact(&mut g).await.unwrap();
                s.write_all(&[5, 0]).await.unwrap();
                let mut q = [0u8; 4];
                s.read_exact(&mut q).await.unwrap();
                let target = match q[3] {
                    1 => {
                        let mut a = [0u8; 6];
                        s.read_exact(&mut a).await.unwrap();
                        format!("{}.{}.{}.{}:{}", a[0], a[1], a[2], a[3], u16::from_be_bytes([a[4], a[5]]))
                    }
                    4 => {
                        // IPv6 address type: nothing listens on [::1] in this test => "host unreachable"
                        let mut a = [0u8; 18];
                        s.read_exact(&mut a).await.unwrap();
                        seen.lock().unwrap().socks.push("ipv6".into());
                        s.write_all(&[5, 4, 0, 1, 0, 0, 0, 0, 0, 0]).await.unwrap();
                        return;
                    }
                    t => {
                        seen.lock().unwrap().socks.push(format!("address type {t}"));
                        return;
                    }
                };
                seen.lock().unwrap().socks.push(target.clone());
                let mut up = TcpStream::connect(&target).await.unwrap();
                s.write_all(&[5, 0, 0, 1, 0, 0, 0, 0, 0, 0]).await.unwrap();
                let _ = tokio::io::copy_bidirectional(&mut s, &mut up).await;
            });
        }
    });
    addr
}

async fn tls_server(cert: rcgen::CertifiedKey<rcgen::KeyPair>) -> std::net::SocketAddr {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let provider = std::sync::Arc::new(rustls::crypto::ring::default_provider());
    let key = rustls::pki_types::PrivateKeyDer::Pkcs8(cert.signing_key.serialize_der().into());
    let cfg = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![cert.cert.der().clone()], key)
        .unwrap();
    let acc = tokio_rustls::TlsAcceptor::from(std::sync::Arc::new(cfg));
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = l.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((s, _)) = l.accept().await {
            let acc = acc.clone();
            tokio::spawn(async move {
                if let Ok(mut t) = acc.accept(s).await {
                    let mut b = [0u8; 4096];
                    let _ = t.read(&mut b).await;
                    let _ = t.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok").await;
                }
            });
        }
    });
    addr
}

fn cert(names: &[&str], expired: bool) -> rcgen::CertifiedKey<rcgen::KeyPair> {
    let mut p = rcgen::CertificateParams::new(names.iter().map(|s| s.to_string()).collect::<Vec<_>>()).unwrap();
    if expired {
        p.not_before = rcgen::date_time_ymd(2020, 1, 1);
        p.not_after = rcgen::date_time_ymd(2021, 1, 1);
    }
    let key = rcgen::KeyPair::generate().unwrap();
    let cert = p.self_signed(&key).unwrap();
    rcgen::CertifiedKey { cert, signing_key: key }
}

async fn via(route: Route, t: &T, url: &str) -> Result<gdm_core::Outcome, gdm_core::Fail> {
    let r = Relay::start(Arc::new(move |_| vec![route.clone()])).await.unwrap();
    relay::set_global(Some(r));
    let res = t.get(url).await.1;
    relay::set_global(None);
    res
}

#[tokio::test(flavor = "multi_thread")]
async fn w05_proxy_gets_only_checked_ips_ntlm_on_one_socket_private_refused() {
    let fx = gdm_fixture::Fixture::start("127.0.0.1:0").await;
    let t = engine("proxy", quick());
    let port = fx.addr.port();
    // the URL names a host ("localhost"): the proxy must receive 127.0.0.1, never the name
    let url = format!("http://localhost:{port}/file?size={}&seed=81", 6 * MIB);

    // 1. plain HTTP proxy, plain http download (tunnelled with CONNECT, origin-form inside)
    let seen = Arc::new(Mutex::new(Seen::default()));
    let p = http_proxy("none", seen.clone()).await;
    let o = via(Route::Http(p, ProxyAuth::None), &t, &url).await.unwrap();
    assert_eq!(o.sha256, gdm_fixture::expected_sha256(81, 6 * MIB));
    let c = seen.lock().unwrap().connects.clone();
    // only IP literals; localhost may be tried as [::1] first (nothing listens there) then 127.0.0.1
    assert!(!c.is_empty() && c.iter().all(|(_, a, _)| a == &format!("127.0.0.1:{port}") || a == &format!("[::1]:{port}")), "CONNECT targets: {c:?}");

    // 2. Basic proxy login
    let seen = Arc::new(Mutex::new(Seen::default()));
    let p = http_proxy("basic", seen.clone()).await;
    let url2 = format!("http://localhost:{port}/file?size={}&seed=82", MIB);
    let o = via(Route::Http(p, ProxyAuth::Basic(relay::basic("alice", "pw"))), &t, &url2).await.unwrap();
    assert_eq!(o.sha256, gdm_fixture::expected_sha256(82, MIB));

    // 3. NTLM with the current user's credentials (real SSPI on both sides): 3 CONNECTs on one socket
    let seen = Arc::new(Mutex::new(Seen::default()));
    let p = http_proxy("ntlm", seen.clone()).await;
    let url3 = format!("http://localhost:{port}/file?size={}&seed=83", MIB);
    let o = via(Route::Http(p, ProxyAuth::Sspi("NTLM".into())), &t, &url3).await.unwrap();
    assert_eq!(o.sha256, gdm_fixture::expected_sha256(83, MIB));
    let c = seen.lock().unwrap().connects.clone();
    let v4 = format!("127.0.0.1:{port}");
    let ok_conn = c.iter().filter(|(_, a, _)| a == &v4).map(|(n, _, _)| *n).next().unwrap();
    let legs = c.iter().filter(|(n, _, _)| *n == ok_conn).count();
    assert_eq!(legs, 3, "NTLM: 3 CONNECTs on one socket: {c:?}");

    // 4. SOCKS5: an IPv4 address, never a domain (address type 3)
    let seen = Arc::new(Mutex::new(Seen::default()));
    let p = socks_proxy(seen.clone()).await;
    let url4 = format!("http://localhost:{port}/file?size={}&seed=84", MIB);
    let o = via(Route::Socks5(p, None), &t, &url4).await;
    let o = o.unwrap_or_else(|e| panic!("socks: {e:?} seen {:?}", seen.lock().unwrap().socks));
    assert_eq!(o.sha256, gdm_fixture::expected_sha256(84, MIB));
    let socks = seen.lock().unwrap().socks.clone();
    assert!(socks.contains(&format!("127.0.0.1:{port}")) && socks.iter().all(|s| !s.contains("type 3")), "{socks:?}");

    // 5. W-02 through the proxy: a private destination the user did not ask for is refused by the relay
    let seen = Arc::new(Mutex::new(Seen::default()));
    let p = http_proxy("none", seen.clone()).await;
    let r = Relay::start(Arc::new(move |_| vec![Route::Http(p.clone(), ProxyAuth::None)])).await.unwrap();
    relay::set_global(Some(r));
    let id = t
        .engine
        .add(gdm_core::NewJob {
            url: format!("http://localhost:{port}/file?size=1000&seed=85"),
            dir: t.dl.clone(),
            typed: false,
            ..Default::default()
        })
        .unwrap();
    let res = t.engine.run(&id, Unverified::Ask, None).await;
    relay::set_global(None);
    assert!(res.is_err(), "private destination through the proxy must fail");
    assert!(seen.lock().unwrap().connects.is_empty(), "the upstream proxy was never asked");

    // 6. HTTPS through the relay and the proxy: TLS is end to end (the relay never terminates it), so a bad
    //    certificate still fails closed (W-03 through the tunnel)
    let seen = Arc::new(Mutex::new(Seen::default()));
    let p = http_proxy("none", seen.clone()).await;
    let tls = tls_server(cert(&["localhost"], false)).await;
    let r = Relay::start(Arc::new(move |_| vec![Route::Http(p.clone(), ProxyAuth::None)])).await.unwrap();
    relay::set_global(Some(r));
    let id = t
        .engine
        .add(gdm_core::NewJob { url: format!("https://localhost:{}/f.bin", tls.port()), dir: t.dl.clone(), typed: true, ..Default::default() })
        .unwrap();
    let res = t.engine.run(&id, Unverified::Ask, None).await;
    relay::set_global(None);
    assert!(matches!(res, Err(gdm_core::Fail::Tls(_))), "self-signed through the proxy: {res:?}");
    assert!(seen.lock().unwrap().connects.iter().any(|(_, a, _)| a == &format!("127.0.0.1:{}", tls.port())), "it did go through the proxy");
}
