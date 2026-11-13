// SPDX-License-Identifier: RPL-1.5
//! G7 F08: sync queue conditional checks (Q-05) and queue order/claims in the store.
mod common;
use common::*;
use gdm_core::Unverified;

#[tokio::test(flavor = "multi_thread")]
async fn q05_sync_304_reads_nothing_and_changed_replaces_only_own_file() {
    let fx = gdm_fixture::Fixture::start("127.0.0.1:0").await;
    let t = engine("sync", quick());
    let url = fx.url("/file?size=300000&seed=7");
    let (id, r) = t.get(&url).await;
    let o = r.unwrap();
    assert_eq!(o.name, "file");
    fx.clear_log();
    // unchanged: 304, the request carried If-None-Match and Range 0-0, the job stays complete
    assert!(!t.engine.sync_check(&id).await.unwrap());
    let log = fx.log();
    assert_eq!(log.len(), 1);
    let h = &log[0].headers;
    assert!(h.iter().any(|(k, v)| k == "if-none-match" && v == "\"s7-300000\""), "{h:?}");
    assert!(h.iter().any(|(k, v)| k == "range" && v == "bytes=0-0"));
    assert_eq!(t.engine.job(&id).unwrap().state, "complete");

    // the server's file changed (our recorded validator no longer matches): changed => re-download in place
    t.engine.store.lock().unwrap().conn.execute("UPDATE jobs SET validator = '\"old\"' WHERE id = ?1", [&id]).unwrap();
    assert!(t.engine.sync_check(&id).await.unwrap());
    assert_eq!(t.engine.job(&id).unwrap().state, "queued");
    let o2 = t.engine.run(&id, Unverified::Ask, None).await.unwrap();
    assert_eq!(o2.name, "file", "replaced its own published file, no counter name");
    assert_eq!(t.files(), vec!["file".to_string()]);
    assert_eq!(sha_file(&t.dl.join("file")), gdm_fixture::expected_sha256(7, 300000));

    // someone else's file took the name: a sync re-download never overwrites it (counter name instead)
    std::fs::remove_file(t.dl.join("file")).unwrap();
    std::fs::write(t.dl.join("file"), b"the user's own file").unwrap();
    t.engine.store.lock().unwrap().conn.execute("UPDATE jobs SET validator = '\"old\"' WHERE id = ?1", [&id]).unwrap();
    assert!(t.engine.sync_check(&id).await.unwrap());
    let o3 = t.engine.run(&id, Unverified::Ask, None).await.unwrap();
    assert_ne!(o3.name, "file");
    assert_eq!(std::fs::read(t.dl.join("file")).unwrap(), b"the user's own file");
}

#[test]
fn queue_order_retry_and_claims() {
    let t = engine("qorder", quick());
    let ids: Vec<String> = (0..3).map(|i| t.engine.add(t.job(&format!("http://127.0.0.1:9/f{i}.bin"))).unwrap()).collect();
    let mut st = t.engine.store.lock().unwrap();
    st.set_queue(&ids, Some("main")).unwrap();
    assert_eq!(st.queue_jobs("main").unwrap(), ids);
    assert_eq!(st.requeue_failed(&ids[0]).unwrap(), 1);
    assert_eq!(st.queue_jobs("main").unwrap(), vec![ids[1].clone(), ids[2].clone(), ids[0].clone()], "a failed file goes to the end");
    st.set_queue(&ids[1..2], None).unwrap();
    assert_eq!(st.queue_jobs("main").unwrap().len(), 2);
    assert!(st.claim("main", "2026-11-10").unwrap());
    assert!(!st.claim("main", "2026-11-10").unwrap(), "Q-03: one occurrence starts once");
    assert_eq!(st.last_claim("main").unwrap().as_deref(), Some("2026-11-10"));
}
