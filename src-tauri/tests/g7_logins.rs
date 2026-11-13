// SPDX-License-Identifier: RPL-1.5
//! G7 F16 P-01: a site login's password is only in Credential Manager (never in settings.json, the database,
//! its WAL or an export) and is sent only to the matching folder of its origin.
use gdm_app::app::{lock, App};
use std::time::Duration;

fn contains(hay: &[u8], needle: &str) -> bool {
    let u8n = needle.as_bytes();
    let u16n: Vec<u8> = needle.encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
    hay.windows(u8n.len()).any(|w| w == u8n) || hay.windows(u16n.len()).any(|w| w == u16n.as_slice())
}

#[tokio::test(flavor = "multi_thread")]
async fn p01_password_never_in_settings_db_or_export_and_sent_to_its_folder_only() {
    let fx = gdm_fixture::Fixture::start("127.0.0.1:0").await;
    let d = std::env::temp_dir().join(format!("gdm-p01-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let a = App::open(&d.join("data")).unwrap();
    lock(&a.settings).download_dir = d.join("dl").display().to_string();
    const PW: &str = "Zq9-GDM-p01-secret";
    let l = a.login_set(&fx.url("/p/"), "alice", PW).unwrap();
    // a job inside the folder gets Authorization; one outside does not
    let inside = a.add_and_start(&fx.url("/p/100000/3/0/in.bin"), None, None, vec![], true, true).unwrap();
    let outside = a.add_and_start(&fx.url("/file?size=1000&seed=4"), None, None, vec![], true, true).unwrap();
    for _ in 0..100 {
        if [&inside, &outside].iter().all(|i| a.engine.job(i).unwrap().state == "complete") {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let auth: Vec<(String, bool)> = fx.log().iter().map(|r| (r.target.clone(), r.headers.iter().any(|(k, _)| k == "authorization"))).collect();
    assert!(auth.iter().filter(|(t, _)| t.starts_with("/p/")).all(|(_, h)| *h), "{auth:?}");
    assert!(auth.iter().filter(|(t, _)| t.starts_with("/file")).all(|(_, h)| !*h), "{auth:?}");
    lock(&a.settings).categories[1].dir = r"D:\RememberedFolderXYZ".into();
    let export = a.export_json().unwrap();
    assert!(!contains(&export, "RememberedFolderXYZ"), "no folder paths in an export");
    drop(a);
    for f in ["settings.json", "gdm.db", "gdm.db-wal"] {
        if let Ok(b) = std::fs::read(d.join("data").join(f)) {
            assert!(!contains(&b, PW), "password found in {f}");
        }
    }
    assert!(!contains(&export, PW), "password in the export");
    assert_eq!(gdm_app::creds::load(&l.id).map(|x| x.1).as_deref(), Some(PW));
    gdm_app::creds::delete(&l.id);
    let _ = std::fs::remove_dir_all(d);
}
