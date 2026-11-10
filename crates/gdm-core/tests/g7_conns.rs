// SPDX-License-Identifier: RPL-1.5
//! G7 F11: L-02 total connection cap across jobs, per-server profile (IDM Exceptions), cap learned from 429.
mod common;
use common::*;
use gdm_core::Unverified;
use std::sync::Arc;

async fn run_all(t: &Arc<T>, urls: Vec<String>) {
    let ids: Vec<String> = urls.iter().map(|u| t.engine.add(t.job(u)).unwrap()).collect();
    let mut hs = vec![];
    for id in ids {
        let t2 = t.clone();
        hs.push(tokio::spawn(async move { t2.engine.run(&id, Unverified::Ask, None).await.map(|_| ()) }));
    }
    for h in hs {
        h.await.unwrap().unwrap();
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn per_server_profile_caps_one_job() {
    let fx = gdm_fixture::Fixture::start("127.0.0.1:0").await;
    let t = Arc::new(engine("host", quick()));
    t.engine.conns.lock().unwrap().set(48, vec![("127.0.0.1".into(), 2)]);
    run_all(&t, vec![fx.url(&format!("/p/{}/7/3000000/a.bin", 32 * MIB))]).await;
    assert!(fx.peak_inflight() <= 2 + 1, "profile 2, saw {}", fx.peak_inflight());
    assert!(gdm_core::host_match("*.example.com", "dl.example.com") && gdm_core::host_match("*.example.com", "example.com"));
    assert!(!gdm_core::host_match("*.example.com", "badexample.com") && !gdm_core::host_match("example.com", "x.example.com"));
}

/// a 429 halves the target (E-05) and later jobs to that host start from the learned cap
#[tokio::test(flavor = "multi_thread")]
async fn learns_cap_from_429() {
    let fx = gdm_fixture::Fixture::start("127.0.0.1:0").await;
    let t = Arc::new(engine("learn", quick()));
    let before = t.engine.conns.lock().unwrap().for_host("127.0.0.1", gdm_core::seg::Config::default()).cap;
    assert_eq!(before, 8);
    run_all(&t, vec![fx.url(&format!("/429?size={}&seed=5&rate=3000000", 16 * MIB))]).await;
    let after = t.engine.conns.lock().unwrap().for_host("127.0.0.1", gdm_core::seg::Config::default()).cap;
    assert!(after < before, "learned cap {after} after a 429");
}
