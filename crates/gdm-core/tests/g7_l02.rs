// SPDX-License-Identifier: RPL-1.5
//! G7 F11 L-02 in its own test binary: the connection gauge is per process, other tests must not share it.
mod common;
use common::*;
use gdm_core::Unverified;
use std::sync::atomic::Ordering;
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

/// L-02 card test: 10 jobs that each want 8 connections stay within the total cap (48 by default; 12 here to
/// make the cap bind with a small fixture)
#[tokio::test(flavor = "multi_thread")]
async fn l02_total_cap_across_jobs() {
    let fx = gdm_fixture::Fixture::start("127.0.0.1:0").await;
    let t = Arc::new(engine("l02", quick()));
    t.engine.conns.lock().unwrap().set(12, vec![]);
    let urls = (0..10).map(|i| fx.url(&format!("/p/{}/{}/2000000/f{i}.bin", 24 * MIB, 100 + i))).collect();
    run_all(&t, urls).await;
    // GDM's own connections (each worker holds a slot of the total cap). The fixture counts server-side
    // sockets, which outlive a dropped response for a moment (netstat in G7 evidence: <= cap + 2 pairs)
    let peak = gdm_core::seg::PEAK_CONNS.load(Ordering::SeqCst);
    eprintln!("L-02: peak GDM connections {peak}, fixture in-flight {}, fixture sockets {}", fx.peak_inflight(), fx.peak_conns());
    assert!(peak <= 12, "peak {peak} connections > total cap 12");
    assert!(peak >= 10, "every job ran (peak {peak})");
    assert_eq!(gdm_core::TOTAL_CONNS, 48, "default total cap (card L-02)");
}
