// SPDX-License-Identifier: RPL-1.5
//! G7 F06 refresh download address: R-01 (same size, other bytes => never resumed), R-02 (ticket in SQLite),
//! R-03 (an HTML page instead of the file).
mod common;
use common::*;
use gdm_core::Unverified;
use std::time::Duration;

async fn partial(t: &T, url: &str) -> String {
    let id = t.engine.add(t.job(url)).unwrap();
    let _ = tokio::time::timeout(Duration::from_millis(3000), t.engine.run(&id, Unverified::Ask, None)).await;
    let _ = t.engine.stopped(&id);
    assert!(t.engine.job(&id).unwrap().done > 0, "some progress committed");
    id
}

#[tokio::test(flavor = "multi_thread")]
async fn r01_same_size_other_bytes_is_not_resumed_same_file_is() {
    let fx = gdm_fixture::Fixture::start("127.0.0.1:0").await;
    let t = engine("r01", quick());
    let size = 24 * MIB;
    let id = partial(&t, &fx.url(&format!("/file?size={size}&seed=61&trate=3000000"))).await;
    // another file of the same size (different bytes and validator): not "strong", seam check fails
    let other = fx.url(&format!("/file?size={size}&seed=62"));
    let c = t.engine.refresh_check(&id, &other, &[]).await.unwrap();
    assert!(c.same_size && !c.strong, "{c:?}");
    assert_eq!(c.seam, Some(false), "R-01: the seam check sees other bytes");
    // the same file under a new address (path form, same ETag): strong => resume gives the right file
    let same = fx.url(&format!("/p/{size}/61/0/renamed.bin"));
    let c = t.engine.refresh_check(&id, &same, &[]).await.unwrap();
    assert!(c.strong && c.seam == Some(true), "{c:?}");
    t.engine.refresh_apply(&id, &same, vec![]).unwrap();
    let o = t.engine.run(&id, Unverified::Ask, None).await.unwrap();
    assert_eq!(o.sha256, gdm_fixture::expected_sha256(61, size));
    // R-03: a login page instead of the file
    let id2 = partial(&t, &fx.url(&format!("/file?size={size}&seed=63&trate=3000000"))).await;
    assert_eq!(t.engine.refresh_check(&id2, &fx.url("/html"), &[]).await.unwrap_err(), gdm_core::Fail::Html);
}

/// R-02: the ticket is in the database (survives the extension worker and a GDM restart), expires, matches
/// only its browser and site
#[test]
fn r02_ticket_persists_expires_and_binds() {
    let t = engine("r02", quick());
    let id = t.engine.add(t.job("http://127.0.0.1:9/x.bin")).unwrap();
    let db = t.root.join("data").join("gdm.db");
    t.engine.store.lock().unwrap().ticket_put(&id, "chrome", "example.com", 1_000).unwrap();
    drop(t.engine.store.lock().unwrap());
    let (st2, _) = gdm_core::store::Store::open(&db).unwrap();
    assert_eq!(st2.ticket_match("chrome", "example.com", 999).unwrap().as_deref(), Some(id.as_str()), "after a restart");
    assert!(st2.ticket_match("edge", "example.com", 999).unwrap().is_none(), "other browser");
    assert!(st2.ticket_match("chrome", "evil.com", 999).unwrap().is_none(), "other site");
    assert!(st2.ticket_match("chrome", "example.com", 1_000).unwrap().is_none(), "expired");
}
