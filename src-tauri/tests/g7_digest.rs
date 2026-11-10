// SPDX-License-Identifier: RPL-1.5
//! G7 Auth row: a site login on a server that wants Digest (RFC 7616). Basic first gets 401; GDM reads the
//! challenge, switches the job to Digest and finishes; every request carries its own nonce count.
use gdm_app::app::{lock, App};
use std::time::Duration;

#[tokio::test(flavor = "multi_thread")]
async fn digest_site_login_after_401() {
    let fx = gdm_fixture::Fixture::start("127.0.0.1:0").await;
    let d = std::env::temp_dir().join(format!("gdm-dig-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let a = App::open(&d.join("data")).unwrap();
    lock(&a.settings).download_dir = d.join("dl").display().to_string();
    let l = a.login_set(&fx.url("/"), "alice", "wonderland").unwrap();
    let size = 6u64 << 20;
    let id = a.add_and_start(&fx.url(&format!("/digest?size={size}&seed=91&trate=4000000")), Some("d.bin".into()), None, vec![], true, true).unwrap();
    for _ in 0..300 {
        if a.engine.job(&id).unwrap().state == "complete" {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let j = a.engine.job(&id).unwrap();
    assert_eq!(j.state, "complete", "{:?}", j.error_code);
    assert_eq!(j.sha256.as_deref(), Some(gdm_fixture::expected_sha256(91, size).as_str()));
    let auths: Vec<String> = fx.log().iter().filter_map(|r| r.headers.iter().find(|(k, _)| k == "authorization").map(|(_, v)| v.clone())).collect();
    assert!(auths.iter().any(|v| v.starts_with("Basic ")), "Basic tried first");
    let ncs: Vec<&str> =
        auths.iter().filter(|v| v.starts_with("Digest ")).filter_map(|v| v.split("nc=").nth(1)).map(|s| s.split(',').next().unwrap_or("")).collect();
    assert!(ncs.len() >= 2, "Digest requests: {ncs:?}");
    let mut uniq = ncs.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(uniq.len(), ncs.len(), "every request has its own nonce count: {ncs:?}");
    gdm_app::creds::delete(&l.id);
    let _ = std::fs::remove_dir_all(d);
}
