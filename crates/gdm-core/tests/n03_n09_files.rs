// SPDX-License-Identifier: RPL-1.5
//! N-02 (name on disk), N-03 (junction races), N-04 (no silent overwrite), N-05 (MOTW + AV), N-09 (FAT/SMB).
mod common;
use common::*;
use gdm_core::finalize::Protection;
use gdm_core::{Fail, NewJob, Unverified};
use gdm_fixture::{expected_sha256, Fixture};
use std::path::Path;
use std::process::Command;

async fn fx() -> Fixture { Fixture::start("127.0.0.1:0").await }

fn junction(link: &Path, target: &Path) {
    let st = Command::new("cmd").args(["/c", "mklink", "/J"]).arg(link).arg(target).output().unwrap();
    assert!(st.status.success(), "mklink /J: {}", String::from_utf8_lossy(&st.stderr));
}

#[tokio::test(flavor = "multi_thread")]
async fn n02_rtlo_name_on_disk() {
    let a = fx().await;
    let t = engine("n02", quick());
    let id = t.engine.add(NewJob { name: Some("invoice\u{202E}fdp.exe".into()), ..t.job(&a.url("/file?size=1000&seed=1")) }).unwrap();
    let o = t.engine.run(&id, Unverified::Ask, None).await.unwrap();
    assert_eq!(o.name, "invoicefdp.exe");
    assert!(t.files().iter().all(|f| !f.contains('\u{202E}')));
    assert!(gdm_core::name::dangerous(&o.name) && gdm_core::name::real_extension(&o.name) == "exe");
}

#[tokio::test(flavor = "multi_thread")]
async fn n03_destination_is_a_junction_refused() {
    let root = tmpdir("n03a");
    let real = root.join("real");
    std::fs::create_dir_all(&real).unwrap();
    junction(&root.join("link"), &real);
    let t = engine("n03a-e", quick());
    let r = t.engine.add(NewJob { url: "http://127.0.0.1:1/x".into(), dir: root.join("link"), typed: true, ..Default::default() });
    assert!(matches!(r, Err(Fail::Reparse(_))), "{r:?}");
    // an ancestor that is a junction: the path no longer resolves to itself
    std::fs::create_dir_all(real.join("sub")).unwrap();
    let r = t.engine.add(NewJob { url: "http://127.0.0.1:1/x".into(), dir: root.join("link").join("sub"), typed: true, ..Default::default() });
    assert!(matches!(r, Err(Fail::Reparse(_))), "ancestor junction: {r:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn n03_junction_planted_at_part_name_job_fails_target_untouched() {
    let a = fx().await;
    let t = engine("n03b", quick());
    let victim_dir = tmpdir("n03b-victim");
    std::fs::write(victim_dir.join("thesis.docx"), b"precious").unwrap();
    let id = t.engine.add(t.job(&a.url("/file?size=100000&seed=2"))).unwrap();
    let name = t.engine.job(&id).unwrap().name;
    junction(&t.dl.join(format!("{name}.part")), &victim_dir); // attacker plants it between add and run
    let r = t.engine.run(&id, Unverified::Ask, None).await;
    assert!(r.is_err(), "job fails: {r:?}");
    assert_eq!(std::fs::read(victim_dir.join("thesis.docx")).unwrap(), b"precious");
    assert_eq!(std::fs::read_dir(&victim_dir).unwrap().count(), 1, "nothing written through the junction");
}

#[tokio::test(flavor = "multi_thread")]
async fn n03_destination_cannot_be_swapped_while_downloading() {
    let a = fx().await;
    let t = engine("n03c", quick());
    let id = t.engine.add(t.job(&a.url(&format!("/file?size={}&seed=3&rate=4000000", 24 * MIB)))).unwrap();
    let dl = t.dl.clone();
    let swap = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(700)).await;
        // the attacker tries to move the folder away and put a junction in its place
        std::fs::rename(&dl, dl.with_file_name("moved")).is_err()
    });
    let o = t.engine.run(&id, Unverified::Ask, None).await.unwrap();
    assert!(swap.await.unwrap(), "renaming the held destination folder must fail");
    assert_eq!(o.sha256, expected_sha256(3, 24 * MIB));
    assert!(o.path.starts_with(&t.dl));
}

#[tokio::test(flavor = "multi_thread")]
async fn n04_two_jobs_same_name_and_existing_file() {
    let a = fx().await;
    let t = engine("n04", quick());
    std::fs::write(t.dl.join("setup.exe"), b"user's own file").unwrap();
    let url = a.url("/file?size=50000&seed=4&n=setup.exe");
    let j1 = t.engine.add(NewJob { name: Some("setup.exe".into()), ..t.job(&url) }).unwrap();
    let j2 = t.engine.add(NewJob { name: Some("SETUP.EXE".into()), ..t.job(&url) }).unwrap();
    let (r1, r2) = tokio::join!(t.engine.run(&j1, Unverified::Ask, None), t.engine.run(&j2, Unverified::Ask, None));
    let (n1, n2) = (r1.unwrap().name, r2.unwrap().name);
    assert_ne!(n1.to_lowercase(), n2.to_lowercase(), "two different names");
    assert!(n1 != "setup.exe" && !n2.eq_ignore_ascii_case("setup.exe"), "{n1} {n2}");
    assert_eq!(std::fs::read(t.dl.join("setup.exe")).unwrap(), b"user's own file", "existing file never overwritten");
}

#[tokio::test(flavor = "multi_thread")]
async fn n05_mark_of_the_web_zone3_without_query_secrets() {
    let a = fx().await;
    let t = engine("n05", quick());
    let url = a.url("/file?size=20000&seed=5&token=SECRET");
    let id = t.engine.add(NewJob { name: Some("tool.exe".into()), referrer: Some("http://ref.example/page?sid=SECRET".into()), ..t.job(&url) }).unwrap();
    let o = t.engine.run(&id, Unverified::Ask, None).await.unwrap();
    assert_eq!(o.protection, Protection::Protected);
    let z = std::fs::read_to_string(format!("{}:Zone.Identifier", o.path.display())).expect("Zone.Identifier ADS");
    assert!(z.contains("ZoneId=3"), "{z}");
    assert!(!z.contains("SECRET"), "HostUrl/ReferrerUrl without query: {z}");
    assert_eq!(t.engine.job(&id).unwrap().state, "complete");
}

fn eicar() -> Vec<u8> {
    // stored reversed so this test binary itself is not flagged
    "*H+H$!ELIF-TSET-SURIVITNA-DRADNATS-RACIE$}7)CC7)^P(45XZP\\4[PA@%P!O5X".chars().rev().collect::<String>().into_bytes()
}

/// is a real-time antivirus acting on this machine? (write EICAR, see whether it survives)
fn av_active() -> bool {
    let d = tmpdir("avprobe");
    let p = d.join("probe.com");
    if std::fs::write(&p, eicar()).is_err() { return true; }
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if std::fs::read(&p).is_err() { return true; }
    }
    let _ = std::fs::remove_file(&p);
    false
}

#[tokio::test(flavor = "multi_thread")]
async fn n05_eicar_blocked_never_complete() {
    if !av_active() { eprintln!("SKIP: no real-time antivirus acted on EICAR here (N-05 AV leg needs one)"); return; }
    let a = fx().await;
    a.add_blob("eicar.com", eicar());
    let t = engine("n05av", quick());
    let id = t.engine.add(t.job(&a.url("/static/eicar.com"))).unwrap();
    let r = t.engine.run(&id, Unverified::Ask, None).await;
    assert!(matches!(r, Err(Fail::Blocked(_)) | Err(Fail::Io(_))), "{r:?}");
    let j = t.engine.job(&id).unwrap();
    assert!(j.state == "blocked" || j.state == "failed", "{}", j.state);
    assert!(!t.dl.join("eicar.com").exists(), "never a final name");
}

/// N-09 matrix on real volumes: CI creates them (GDM_TEST_FAT_DIR = exFAT VHD, GDM_TEST_SMB_DIR = \\localhost share)
#[tokio::test(flavor = "multi_thread")]
async fn n09_unprotected_targets() {
    let mut ran = false;
    for var in ["GDM_TEST_FAT_DIR", "GDM_TEST_SMB_DIR"] {
        let Ok(dir) = std::env::var(var) else { eprintln!("SKIP {var}: not set (needs admin to create the volume/share)"); continue };
        ran = true;
        let a = fx().await;
        let t = engine("n09", quick());
        let id = t.engine.add(NewJob { url: a.url("/file?size=30000&seed=9"), dir: dir.clone().into(), typed: true, name: Some(format!("n09-{}.exe", std::process::id())), ..Default::default() }).unwrap();
        let r = t.engine.run(&id, Unverified::Ask, None).await;
        match var {
            "GDM_TEST_FAT_DIR" => {
                let o = r.unwrap();
                assert!(matches!(o.protection, Protection::Unprotected(_)), "{var}: {:?}", o.protection);
                assert_eq!(t.engine.job(&id).unwrap().state, "unprotected");
            }
            _ => {
                // SMB to an NTFS share may carry ADS: either outcome is labelled, never silently "protected" without a zone
                let o = r.unwrap();
                let zone = std::fs::read_to_string(format!("{}:Zone.Identifier", o.path.display())).ok();
                assert_eq!(matches!(o.protection, Protection::Protected), zone.is_some_and(|z| z.contains("ZoneId=3")), "{var}");
            }
        }
    }
    if !ran { eprintln!("N-09 real-volume matrix not run here"); }
}
