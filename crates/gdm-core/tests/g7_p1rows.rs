// SPDX-License-Identifier: RPL-1.5
//! G7 rows moved from P1: #8 User-Agent (global + per site), #10 `.sha256` next to the file (same origin).
mod common;
use common::*;
use gdm_core::{Fail, Unverified};

#[tokio::test(flavor = "multi_thread")]
async fn p1_10_sha256_sidecar_checks_the_file() {
    let fx = gdm_fixture::Fixture::start("127.0.0.1:0").await;
    let t = engine("sha", quick());
    t.engine.hash_sidecar.store(true, std::sync::atomic::Ordering::Relaxed);
    let (id, ok) = t.get(&fx.url("/p/300000/5/0/good.iso")).await;
    let o = ok.unwrap();
    assert_eq!(t.engine.job(&id).unwrap().sha256_expected.as_deref(), Some(o.sha256.as_str()));
    assert!(fx.log().iter().any(|r| r.target.ends_with("good.iso.sha256")));
    let (_, bad) = t.get(&fx.url("/p/300000/6/0/bad.iso")).await;
    assert_eq!(bad.unwrap_err(), Fail::HashMismatch, "a published hash that does not match is never Complete");
    // no sidecar (404) => downloads as before
    let (_, none) = t.get(&fx.url("/file?size=1000&seed=7")).await;
    none.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn p1_08_user_agent_global_and_per_site() {
    let fx = gdm_fixture::Fixture::start("127.0.0.1:0").await;
    let t = engine("ua", quick());
    t.engine.hash_sidecar.store(false, std::sync::atomic::Ordering::Relaxed);
    let ua_of = |fx: &gdm_fixture::Fixture| {
        fx.log().iter().filter_map(|r| r.headers.iter().find(|(k, _)| k == "user-agent").map(|(_, v)| v.clone())).collect::<Vec<_>>()
    };
    t.get(&fx.url("/file?size=1000&seed=1")).await.1.unwrap();
    assert!(ua_of(&fx).iter().all(|u| u == gdm_core::net::DEFAULT_UA), "{:?}", ua_of(&fx));
    fx.clear_log();
    t.engine.conns.lock().unwrap().user_agent = "Global/1".into();
    t.get(&fx.url("/file?size=1000&seed=2")).await.1.unwrap();
    assert!(ua_of(&fx).iter().all(|u| u == "Global/1"));
    fx.clear_log();
    t.engine.conns.lock().unwrap().site_agents = vec![("127.0.0.1".into(), "Site/2".into())];
    let id = t.engine.add(t.job(&fx.url("/file?size=1000&seed=3"))).unwrap();
    t.engine.run(&id, Unverified::Ask, None).await.unwrap();
    assert!(ua_of(&fx).iter().all(|u| u == "Site/2"), "{:?}", ua_of(&fx));
}

#[test]
fn p1_10_sha256_file_formats() {
    let h = "a".repeat(64);
    assert_eq!(gdm_core::parse_sha256_file(&format!("{h}  x.iso\n{}  y.iso", "b".repeat(64)), "x.iso"), Some(h.clone()));
    assert_eq!(gdm_core::parse_sha256_file(&format!("{h} *dir/x.iso"), "x.iso"), Some(h.clone()));
    assert_eq!(gdm_core::parse_sha256_file(&h, "x.iso"), Some(h.clone()), "a bare hash");
    assert_eq!(gdm_core::parse_sha256_file(&format!("{h}  other.iso"), "x.iso"), None, "another file's hash");
    assert_eq!(gdm_core::parse_sha256_file("<html>not found</html>", "x.iso"), None);
}
