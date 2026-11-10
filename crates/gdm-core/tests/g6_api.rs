// SPDX-License-Identifier: RPL-1.5
//! G6 UI API: live progress, pause (drop the run) + resume, live connection count (P1 #3), remove (P1 #28),
//! FTS search over 10 000 rows incl. Khmer (P1 #20, D-03).
mod common;
use common::*;
use gdm_core::{seg::Live, Unverified};
use gdm_fixture::{expected_sha256, Fixture};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[tokio::test(flavor = "multi_thread")]
async fn live_progress_pause_resume_hash_ok() {
    let a = Fixture::start("127.0.0.1:0").await;
    let t = engine("g6p", quick());
    let (size, seed) = (32 * MIB, 61);
    let id = t.engine.add(t.job(&a.url(&format!("/file?size={size}&seed={seed}&rate=8000000")))).unwrap();
    let live = Arc::new(Live::default());
    let run = t.engine.run_live(&id, Unverified::Ask, None, Some(live.clone()));
    // pause = drop the future after some progress
    let _ = tokio::time::timeout(Duration::from_millis(1500), run).await;
    let seen = live.written.load(Ordering::Relaxed);
    assert!(seen > 0 && seen < size, "live progress visible while running: {seen}");
    assert_eq!(live.total.load(Ordering::Relaxed), size);
    assert!(!live.segs.lock().unwrap().is_empty());
    assert!(!t.engine.is_running(&id), "dropping the run releases the job");
    t.engine.stopped(&id).unwrap();
    assert_eq!(t.engine.job(&id).unwrap().state, "paused");
    let o = t.engine.run_live(&id, Unverified::Ask, None, Some(live.clone())).await.unwrap();
    assert_eq!(o.sha256, expected_sha256(seed, size), "resume after pause: correct bytes");
}

#[tokio::test(flavor = "multi_thread")]
async fn live_connection_count_is_followed() {
    let a = Fixture::start("127.0.0.1:0").await;
    let t = engine("g6c", quick());
    let (size, seed) = (64 * MIB, 62);
    let id = t.engine.add(t.job(&a.url(&format!("/file?size={size}&seed={seed}&rate=2000000")))).unwrap();
    let live = Arc::new(Live::default());
    live.want_conns.store(8, Ordering::Relaxed);
    let l2 = live.clone();
    let watch = tokio::spawn(async move {
        let t0 = Instant::now();
        let mut hit8 = None;
        while t0.elapsed() < Duration::from_secs(10) {
            if l2.conns.load(Ordering::Relaxed) >= 8 && hit8.is_none() {
                hit8 = Some(t0.elapsed());
                l2.want_conns.store(16, Ordering::Relaxed);
            }
            if hit8.is_some() && l2.conns.load(Ordering::Relaxed) >= 16 {
                return (hit8, Some(t0.elapsed()));
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        (hit8, None)
    });
    let run = t.engine.run_live(&id, Unverified::Ask, None, Some(live.clone()));
    let (o, w) = tokio::join!(run, watch);
    let (h8, h16) = w.unwrap();
    assert!(h8.is_some_and(|d| d < Duration::from_secs(2)), "8 connections within 2 s of asking: {h8:?}");
    let (h8, h16) = (h8.unwrap(), h16.expect("16 connections reached"));
    assert!(h16 - h8 < Duration::from_secs(2), "8 -> 16 within 2 s: {:?}", h16 - h8);
    assert_eq!(o.unwrap().sha256, expected_sha256(seed, size));
}

#[tokio::test(flavor = "multi_thread")]
async fn remove_deletes_own_part_and_recycles_finished_file() {
    let a = Fixture::start("127.0.0.1:0").await;
    let t = engine("g6r", quick());
    // unfinished: .part + sidecar go away, the row too
    let id = t.engine.add(t.job(&a.url(&format!("/file?size={}&seed=63&rate=4000000", 32 * MIB)))).unwrap();
    let _ = tokio::time::timeout(Duration::from_millis(3000), t.engine.run(&id, Unverified::Ask, None)).await;
    t.engine.stopped(&id).unwrap();
    assert!(t.files().iter().any(|f| f.ends_with(".part")));
    t.engine.remove(&id, false).unwrap();
    assert!(t.files().is_empty(), "nothing left: {:?}", t.files());
    assert!(t.engine.job(&id).is_err());
    // a .part this job did not create is never deleted
    let id2 = t.engine.add(t.job(&a.url("/file?size=1024&seed=64"))).unwrap();
    let name = t.engine.job(&id2).unwrap().name;
    std::fs::write(t.dl.join(format!("{name}.part")), b"user data").unwrap();
    t.engine.remove(&id2, false).unwrap();
    assert_eq!(std::fs::read(t.dl.join(format!("{name}.part"))).unwrap(), b"user data");
    std::fs::remove_file(t.dl.join(format!("{name}.part"))).unwrap();
    // finished + delete_file: off the disk (to the Recycle Bin), row gone
    let (id3, r) = t.get(&a.url("/file?size=4096&seed=65")).await;
    let o = r.unwrap();
    assert!(o.path.exists());
    t.engine.remove(&id3, true).unwrap();
    assert!(!o.path.exists());
    // finished, keep the file
    let (id4, r) = t.get(&a.url("/file?size=4096&seed=66")).await;
    let o = r.unwrap();
    t.engine.remove(&id4, false).unwrap();
    assert!(o.path.exists(), "remove from list keeps the file");
}

#[test]
fn fts_10k_rows_khmer_and_latin_under_100ms() {
    let t = engine("g6f", quick());
    {
        let mut st = t.engine.store.lock().unwrap();
        let dir_id = st.dir_id(&t.dl).unwrap();
        let tx = st.conn.transaction().unwrap();
        for i in 0..10_000 {
            let (name, url) = if i % 1000 == 7 {
                (format!("ឯកសារមេរៀនគណិតវិទ្យា-{i}.pdf"), format!("https://school.example.kh/lesson/{i}"))
            } else {
                (format!("setup-tool-{i}.zip"), format!("https://cdn.example.com/files/{i}/setup.zip"))
            };
            tx.execute(
                "INSERT INTO jobs(id, url, name, dir_id, state, created_at) VALUES (?1, ?2, ?3, ?4, 'complete', ?5)",
                rusqlite::params![format!("j{i}"), url, name, dir_id, i],
            )
            .unwrap();
        }
        tx.commit().unwrap();
    }
    let st = t.engine.store.lock().unwrap();
    let t0 = Instant::now();
    let km = st.list(Some("គណិតវិទ្យា"), 500).unwrap();
    let d_km = t0.elapsed();
    let t1 = Instant::now();
    let la = st.list(Some("tool-99"), 500).unwrap();
    let d_la = t1.elapsed();
    let t2 = Instant::now();
    let url = st.list(Some("school.example"), 500).unwrap();
    let d_url = t2.elapsed();
    let short = st.list(Some("ឯក"), 500).unwrap(); // < 3 characters: LIKE path
    assert_eq!(km.len(), 10, "Khmer substring without spaces");
    assert_eq!(la.len(), 111, "tool-99, tool-990..999, tool-9900..9999");
    assert_eq!(url.len(), 10);
    assert_eq!(short.len(), 10);
    assert_eq!(st.list(None, 50).unwrap()[0].id, "j9999", "newest first");
    assert_eq!(st.list(Some("\"; DROP TABLE jobs; --"), 50).unwrap().len(), 0, "query is data");
    assert_eq!(st.list(Some("100%"), 50).unwrap().len(), 0, "LIKE wildcards escaped");
    for d in [d_km, d_la, d_url] {
        assert!(d < Duration::from_millis(100), "search < 100 ms: {d:?}");
    }
    println!("fts 10k: km {d_km:?}, latin {d_la:?}, url {d_url:?}");
}
