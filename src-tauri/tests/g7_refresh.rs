// SPDX-License-Identifier: RPL-1.5
//! G7 F06 refresh flow in the app: ticket -> the next caught link from that site is a candidate (not a new
//! job) -> probe -> only a strong match may resume; other bytes => start over in a new job, the old .part stays.
use gdm_app::app::{lock, App};
use std::time::Duration;

#[tokio::test(flavor = "multi_thread")]
async fn r01_refresh_flow_candidate_probe_decide() {
    let fx = gdm_fixture::Fixture::start("127.0.0.1:0").await;
    let d = std::env::temp_dir().join(format!("gdm-rf-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let a = App::open(&d.join("data")).unwrap();
    lock(&a.settings).download_dir = d.join("dl").display().to_string();
    let size = 16u64 << 20;
    let page = fx.url("/page.html");
    let id = a
        .add_and_start(&fx.url(&format!("/file?size={size}&seed=71&trate=2500000")), Some("big.bin".into()), Some(page.clone()), vec![], true, true)
        .unwrap();
    for _ in 0..100 {
        if a.engine.job(&id).unwrap().done > 0 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    a.pause(&id).await;
    let j = a.engine.job(&id).unwrap();
    assert!(j.done > 0 && j.done < size, "partial: {} {:?} {:?}", j.done, j.state, j.error_code);
    // ticket as [Refresh address] makes it (without opening a browser in the test)
    lock(&a.engine.store).ticket_put(&id, "*", "127.0.0.1", gdm_core::store::now() + 600).unwrap();
    // a download from another site is not a candidate
    assert!(a.refresh_take("https://other.example/x.bin", Some("https://other.example/"), vec![]).is_none());
    // the same-size OTHER file from that page: candidate, not strong, seam fails => resume refused
    let other = fx.url(&format!("/file?size={size}&seed=72"));
    assert_eq!(a.refresh_take(&other, Some(&page), vec![]).as_deref(), Some(id.as_str()));
    let c = a.refresh_candidate(&id).await.unwrap().unwrap();
    let k = c.check.unwrap();
    assert!(!k.strong && k.seam == Some(false), "{k:?}");
    assert_eq!(a.refresh_decide(&id, "resume").await.unwrap_err(), "not-allowed", "R-01: no resume onto other bytes");
    // again, choose [start over]: a new job; the old job and its .part stay
    lock(&a.engine.store).ticket_put(&id, "*", "127.0.0.1", gdm_core::store::now() + 600).unwrap();
    a.refresh_take(&other, Some(&page), vec![]).unwrap();
    a.refresh_candidate(&id).await.unwrap();
    let nid = a.refresh_decide(&id, "restart").await.unwrap();
    assert_ne!(nid, id);
    let j = a.engine.job(&id).unwrap();
    assert!(j.dir.join("big.bin.part").exists(), "old .part kept");
    // the same file at a new address: strong => resume, correct bytes
    lock(&a.engine.store).ticket_put(&id, "*", "127.0.0.1", gdm_core::store::now() + 600).unwrap();
    a.refresh_take(&fx.url(&format!("/p/{size}/71/0/big.bin")), Some(&page), vec![]).unwrap();
    assert!(a.refresh_candidate(&id).await.unwrap().unwrap().check.unwrap().strong);
    a.refresh_decide(&id, "resume").await.unwrap();
    for _ in 0..200 {
        if a.engine.job(&id).unwrap().state == "complete" {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let j = a.engine.job(&id).unwrap();
    assert_eq!(j.state, "complete");
    assert_eq!(j.sha256.as_deref(), Some(gdm_fixture::expected_sha256(71, size).as_str()));
    a.pause(&nid).await;
    let _ = std::fs::remove_dir_all(d);
}
