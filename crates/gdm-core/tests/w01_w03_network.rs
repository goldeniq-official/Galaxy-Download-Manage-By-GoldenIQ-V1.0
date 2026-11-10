// SPDX-License-Identifier: RPL-1.5
//! W-01 (credentials across origins), W-02 (SSRF), W-03 (TLS verification).
mod common;
use common::*;
use gdm_core::net::{self, Creds, Policy};
use gdm_core::{Fail, NewJob, Unverified};
use gdm_fixture::{pct_encode, Fixture};
use std::net::SocketAddr;
use std::sync::Arc;
use url::Url;

async fn fx() -> Fixture {
    Fixture::start("127.0.0.1:0").await
}
const SECRET: [(&str, &str); 2] = [("Cookie", "sid=SECRET"), ("Authorization", "Bearer SECRET")];
fn creds() -> Vec<(String, String)> {
    SECRET.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}
fn saw_secret(f: &Fixture) -> bool {
    f.log().iter().any(|q| q.headers.iter().any(|(k, v)| (k == "cookie" || k == "authorization") && v.contains("SECRET")))
}

#[tokio::test(flavor = "multi_thread")]
async fn w01_cross_origin_redirect_drops_credentials() {
    let (a, b) = (fx().await, fx().await);
    let t = engine("w01", quick());
    let target = b.url(&format!("/file?size={}&seed=5", 2 * MIB));
    let id = t.engine.add(NewJob { creds: creds(), ..t.job(&a.url(&format!("/redirect?to={}", pct_encode(&target)))) }).unwrap();
    let o = t.engine.run(&id, Unverified::Ask, None).await.unwrap();
    assert_eq!(o.sha256, gdm_fixture::expected_sha256(5, 2 * MIB));
    assert!(!b.log().is_empty(), "the other origin (other port) was reached");
    assert!(!saw_secret(&b), "the other origin never saw Cookie/Authorization: {:?}", b.log().iter().map(|q| &q.headers).collect::<Vec<_>>());
    assert!(saw_secret(&a), "control: the job's own origin did get them (test is not vacuous)");
    // nothing secret is persisted
    let db = std::fs::read(t.root.join("data").join("gdm.db")).unwrap_or_default();
    let wal = std::fs::read(t.root.join("data").join("gdm.db-wal")).unwrap_or_default();
    assert!(!String::from_utf8_lossy(&db).contains("SECRET") && !String::from_utf8_lossy(&wal).contains("SECRET"));
}

#[tokio::test(flavor = "multi_thread")]
async fn w01_same_origin_segments_keep_credentials() {
    let a = fx().await;
    let t = engine("w01b", quick());
    let id = t.engine.add(NewJob { creds: creds(), ..t.job(&a.url(&format!("/file?size={}&seed=6&rate=4000000", 16 * MIB))) }).unwrap();
    t.engine.run(&id, Unverified::Ask, None).await.unwrap();
    let reqs = a.log();
    assert!(
        reqs.len() >= 2 && reqs.iter().all(|q| q.headers.iter().any(|(k, v)| k == "cookie" && v == "sid=SECRET")),
        "every segment request to the same origin carries the cookie"
    );
}

#[test]
fn w01_creds_scoped_to_scheme_host_port() {
    let c = Creds { origin: Some(net::origin(&Url::parse("https://drive.example.com/a").unwrap())), headers: creds(), digest: None };
    assert_eq!(c.for_url(&Url::parse("https://drive.example.com/other?x").unwrap()).len(), 2);
    for other in ["https://cdn.attacker.net/a", "http://drive.example.com/a", "https://drive.example.com:8443/a", "https://sub.drive.example.com/a"] {
        assert!(c.for_url(&Url::parse(other).unwrap()).is_empty(), "{other}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn w02_redirect_to_loopback_is_stopped() {
    // the user typed "localhost": that host may be private; a redirect to the literal 127.0.0.1 may not
    let (a, b) = (fx().await, fx().await);
    let t = engine("w02", quick());
    let typed = format!("http://localhost:{}/redirect?to={}", a.addr.port(), pct_encode(&b.url("/echo")));
    let id = t.engine.add(NewJob { url: typed, dir: t.dl.clone(), typed: true, ..Default::default() }).unwrap();
    let r = t.engine.run(&id, Unverified::Ask, None).await;
    assert!(matches!(r, Err(Fail::Forbidden(_))), "{r:?}");
    assert!(b.log().is_empty(), "loopback target never contacted");
    assert!(!a.log().is_empty(), "control: the typed host itself was allowed");
}

#[tokio::test(flavor = "multi_thread")]
async fn w02_untyped_private_url_needs_confirmation() {
    let a = fx().await;
    let t = engine("w02b", quick());
    let id = t.engine.add(NewJob { typed: false, ..t.job(&a.url("/echo")) }).unwrap();
    assert!(matches!(t.engine.run(&id, Unverified::Ask, None).await, Err(Fail::Forbidden(_))));
    assert!(a.log().is_empty(), "private address never contacted without intent");
}

#[test]
fn w02_address_classes_and_mixed_answers() {
    let private = [
        "127.0.0.1",
        "10.1.2.3",
        "172.16.0.1",
        "192.168.1.1",
        "169.254.169.254",
        "100.64.0.1",
        "0.0.0.0",
        "224.0.0.1",
        "255.255.255.255",
        "::1",
        "::",
        "fc00::1",
        "fd12::1",
        "fe80::1",
        "ff02::1",
        "::ffff:127.0.0.1",
        "::ffff:192.168.0.1",
        "64:ff9b::7f00:1",
        "::127.0.0.1",
    ];
    for p in private {
        assert!(!net::public(p.parse().unwrap()), "{p} must not be public");
    }
    for p in ["1.1.1.1", "93.184.216.34", "2606:4700::1111", "::ffff:8.8.8.8"] {
        assert!(net::public(p.parse().unwrap()), "{p}");
    }
    let pol = Policy::default();
    let mixed: Vec<SocketAddr> = vec!["93.184.216.34:443".parse().unwrap(), "127.0.0.1:443".parse().unwrap()];
    assert!(pol.check("rebind.example", &mixed).is_err(), "a mixed public/private DNS answer is refused as a whole");
    assert!(pol.check("ok.example", &mixed[..1]).is_ok());
}

#[test]
fn w02_redirect_rules() {
    let pol = Policy::default();
    let u = |s: &str| Url::parse(s).unwrap();
    assert!(net::redirect_ok(&[u("https://a.example/")], &u("http://a.example/x"), &pol).is_err(), "https -> http");
    assert!(net::redirect_ok(&[u("https://a.example/")], &u("https://b.example/x"), &pol).is_ok());
    assert!(net::redirect_ok(&[u("http://a.example/")], &u("http://169.254.169.254/latest"), &pol).is_err(), "metadata IP literal");
    assert!(net::redirect_ok(&[u("http://a.example/")], &u("file:///C:/Windows/win.ini"), &pol).is_err(), "scheme");
    assert!(net::redirect_ok(&[u("http://a.example/")], &u("http://user:pw@b.example/"), &pol).is_err(), "userinfo");
    let ten: Vec<Url> = (0..10).map(|i| u(&format!("https://h{i}.example/"))).collect();
    assert!(net::redirect_ok(&ten, &u("https://h10.example/"), &pol).is_err(), "more than 10 hops");
}

#[test]
fn w04_crlf_in_credentials_refused() {
    let t = engine("w04", quick());
    let r = t.engine.add(NewJob { creds: vec![("Cookie".into(), "x\r\nX-Evil: 1".into())], ..t.job("http://127.0.0.1:1/x") });
    assert!(matches!(r, Err(Fail::Forbidden(_))));
    // userinfo in the URL is never stored
    let r = t.engine.add(t.job("http://user:pw@127.0.0.1:1/x"));
    assert!(matches!(r, Err(Fail::Forbidden(_))));
    // W-01: any header other than Cookie/Authorization could follow a cross-origin redirect: refused
    let r = t.engine.add(NewJob { creds: vec![("X-Api-Key".into(), "k".into())], ..t.job("http://127.0.0.1:1/x") });
    assert!(matches!(r, Err(Fail::Forbidden(_))));
}

// ---------------- W-03: local TLS servers with bad certificates ----------------

async fn tls_server(cert: rcgen::CertifiedKey<rcgen::KeyPair>) -> SocketAddr {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let key = rustls::pki_types::PrivateKeyDer::Pkcs8(cert.signing_key.serialize_der().into());
    let cfg = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![cert.cert.der().clone()], key)
        .unwrap();
    let acc = tokio_rustls::TlsAcceptor::from(Arc::new(cfg));
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

#[tokio::test(flavor = "multi_thread")]
async fn w03_self_signed_expired_wrong_host_all_fail_closed() {
    let t = engine("w03", quick());
    for (label, c) in
        [("self-signed", cert(&["localhost"], false)), ("expired", cert(&["localhost"], true)), ("wrong host", cert(&["other.example"], false))]
    {
        let addr = tls_server(c).await;
        let id = t
            .engine
            .add(NewJob { url: format!("https://localhost:{}/f.bin", addr.port()), dir: t.dl.clone(), typed: true, ..Default::default() })
            .unwrap();
        let r = t.engine.run(&id, Unverified::Ask, None).await;
        assert!(matches!(r, Err(Fail::Tls(_))), "{label}: {r:?}");
        assert_eq!(t.engine.job(&id).unwrap().state, "failed", "{label}: never retried into success");
    }
}

#[test]
fn w03_no_tls_bypass_in_source() {
    // the CI grep rule, also as a test: nothing in the core may disable certificate checks
    let mut dirs = vec![std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
    while let Some(d) = dirs.pop() {
        for e in std::fs::read_dir(d).unwrap().flatten() {
            if e.path().is_dir() {
                dirs.push(e.path());
                continue;
            }
            let s = std::fs::read_to_string(e.path()).unwrap();
            // split literals: this file itself must pass the CI grep for the same words
            for bad in [
                concat!("danger_", "accept_invalid"),
                concat!("dangerous", "()"),
                concat!("NoCertificate", "Verification"),
                concat!("webpki", "_roots"),
            ] {
                assert!(!s.contains(bad), "{} contains {bad}", e.path().display());
            }
        }
    }
}

/// online (CI sets GDM_NET_TESTS=1): badssl.com expired / wrong host / self-signed / untrusted root fail;
/// a valid site succeeds (positive control: the platform verifier trusts the Windows store)
#[tokio::test(flavor = "multi_thread")]
async fn w03_badssl_online() {
    if std::env::var("GDM_NET_TESTS").is_err() {
        eprintln!("SKIP: set GDM_NET_TESTS=1 (network)");
        return;
    }
    let t = engine("w03net", quick());
    for host in ["expired.badssl.com", "wrong.host.badssl.com", "self-signed.badssl.com", "untrusted-root.badssl.com"] {
        let id = t.engine.add(NewJob { url: format!("https://{host}/"), dir: t.dl.clone(), ..Default::default() }).unwrap();
        let r = t.engine.run(&id, Unverified::Ask, None).await;
        assert!(matches!(r, Err(Fail::Tls(_))), "{host}: {r:?}");
    }
    // positive control: a valid certificate over HTTPS, offered h2 + http/1.1 (the non-segmented client)
    let c = net::client(Arc::new(Policy::default()), false, std::time::Duration::from_secs(20)).unwrap();
    let r = c.get("https://badssl.com/").send().await.unwrap();
    assert!(r.status().is_success(), "valid certificate accepted ({:?})", r.version());
    let r = c.get("https://www.google.com/").send().await.unwrap();
    assert_eq!(r.version(), reqwest::Version::HTTP_2, "HTTP/2 negotiated by ALPN on the non-segmented client");
}
