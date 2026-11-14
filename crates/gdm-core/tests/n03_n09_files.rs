// SPDX-License-Identifier: RPL-1.5
//! N-02 (name on disk), N-03 (junction races), N-04 (no silent overwrite), N-05 (MOTW + AV), N-09 (FAT/SMB).
mod common;
use common::*;
use gdm_core::finalize::Protection;
use gdm_core::{Fail, NewJob, Unverified};
use gdm_fixture::{expected_sha256, Fixture};
#[cfg(windows)]
use std::path::Path;
#[cfg(windows)]
use std::process::Command;

async fn fx() -> Fixture {
    Fixture::start("127.0.0.1:0").await
}

#[cfg(windows)]
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

#[cfg(windows)]
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

#[cfg(windows)]
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

#[cfg(windows)]
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
    assert!(n1 != "setup.exe" && n2 != "setup.exe" && n1 != n2, "{n1} {n2}");
    // Linux names are case-sensitive: SETUP.EXE is a different file there, not a clash
    if !cfg!(target_os = "linux") {
        assert_ne!(n1.to_lowercase(), n2.to_lowercase(), "two different names");
        assert!(!n2.eq_ignore_ascii_case("setup.exe"), "{n1} {n2}");
    }
    assert_eq!(std::fs::read(t.dl.join("setup.exe")).unwrap(), b"user's own file", "existing file never overwritten");
}

#[cfg(windows)]
#[tokio::test(flavor = "multi_thread")]
async fn n05_mark_of_the_web_zone3_without_query_secrets() {
    let a = fx().await;
    let t = engine("n05", quick());
    let url = a.url("/file?size=20000&seed=5&token=SECRET");
    let id =
        t.engine.add(NewJob { name: Some("tool.exe".into()), referrer: Some("http://ref.example/page?sid=SECRET".into()), ..t.job(&url) }).unwrap();
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
    if std::fs::write(&p, eicar()).is_err() {
        return true;
    }
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if std::fs::read(&p).is_err() {
            return true;
        }
    }
    let _ = std::fs::remove_file(&p);
    false
}

#[tokio::test(flavor = "multi_thread")]
async fn n05_eicar_blocked_never_complete() {
    if !av_active() {
        eprintln!("SKIP: no real-time antivirus acted on EICAR here (N-05 AV leg needs one)");
        return;
    }
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
        let Ok(dir) = std::env::var(var) else {
            eprintln!("SKIP {var}: not set (needs admin to create the volume/share)");
            continue;
        };
        ran = true;
        let a = fx().await;
        let t = engine("n09", quick());
        let id = t
            .engine
            .add(NewJob {
                url: a.url("/file?size=30000&seed=9"),
                dir: dir.clone().into(),
                typed: true,
                name: Some(format!("n09-{}.exe", std::process::id())),
                ..Default::default()
            })
            .unwrap();
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
    if !ran {
        eprintln!("N-09 real-volume matrix not run here");
    }
}

/// N-10 / N-03: the .part was replaced by another file between sessions: never resumed into, never truncated
#[tokio::test(flavor = "multi_thread")]
async fn n10_replaced_part_is_not_reused() {
    let a = fx().await;
    let t = engine("n10", quick());
    let id = t.engine.add(t.job(&a.url(&format!("/file?size={}&seed=11&rate=1500000", 48 * MIB)))).unwrap();
    let _ = tokio::time::timeout(std::time::Duration::from_millis(2500), t.engine.run(&id, Unverified::Ask, None)).await;
    let part = t.dl.join(format!("{}.part", t.engine.job(&id).unwrap().name));
    // the cancelled run's writer thread closes its handle a moment later (longer when the disk is busy): wait for
    // it instead of a fixed 500 ms, which made this test flaky under load
    let mut tries = 0;
    while let Err(e) = std::fs::remove_file(&part) {
        tries += 1;
        assert!(tries < 200, "the .part stayed held: {e}");
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    std::fs::write(&part, b"someone else's data").unwrap();
    let r = t.engine.run(&id, Unverified::Ask, None).await;
    assert!(matches!(r, Err(Fail::NeedsDecision(_))), "{r:?}");
    assert_eq!(std::fs::read(&part).unwrap(), b"someone else's data", "untouched");
}

/// a hard link to a user file, planted at the .part name before the first run: never written through
#[tokio::test(flavor = "multi_thread")]
async fn n03_hard_link_at_part_name_never_written() {
    let a = fx().await;
    let t = engine("hl", quick());
    let victim = t.root.join("thesis.docx");
    std::fs::write(&victim, b"precious").unwrap();
    let id = t.engine.add(t.job(&a.url("/file?size=100000&seed=12"))).unwrap();
    let name = t.engine.job(&id).unwrap().name;
    std::fs::hard_link(&victim, t.dl.join(format!("{name}.part"))).unwrap();
    let r = t.engine.run(&id, Unverified::Ask, None).await;
    assert!(matches!(r, Err(Fail::Reparse(_)) | Err(Fail::NeedsDecision(_))), "{r:?}");
    assert_eq!(std::fs::read(&victim).unwrap(), b"precious");
}

/// a non-empty .part this job never created: not adopted, not truncated
#[tokio::test(flavor = "multi_thread")]
async fn n03_foreign_part_not_adopted() {
    let a = fx().await;
    let t = engine("foreign", quick());
    let id = t.engine.add(t.job(&a.url("/file?size=100000&seed=13"))).unwrap();
    let part = t.dl.join(format!("{}.part", t.engine.job(&id).unwrap().name));
    std::fs::write(&part, b"not ours").unwrap();
    assert!(matches!(t.engine.run(&id, Unverified::Ask, None).await, Err(Fail::NeedsDecision(_))));
    assert_eq!(std::fs::read(&part).unwrap(), b"not ours");
}

/// the folder's ancestors cannot be renamed away while a job holds the folder (the full path given to
/// IAttachmentExecute stays the same object)
#[cfg(windows)]
#[tokio::test(flavor = "multi_thread")]
async fn n03_ancestor_cannot_be_renamed_while_held() {
    let root = tmpdir("anc");
    let deep = root.join("a").join("b");
    std::fs::create_dir_all(&deep).unwrap();
    let held = gdm_core::fsx::Dir::open(&deep).unwrap();
    assert!(std::fs::rename(root.join("a"), root.join("a2")).is_err(), "ancestor rename refused while held");
    drop(held);
    assert!(std::fs::rename(root.join("a"), root.join("a2")).is_ok(), "control: allowed once released");
}

#[tokio::test(flavor = "multi_thread")]
async fn one_run_per_job() {
    let a = fx().await;
    let t = engine("dup", quick());
    let id = t.engine.add(t.job(&a.url(&format!("/file?size={}&seed=14&rate=4000000", 8 * MIB)))).unwrap();
    let (r1, r2) = tokio::join!(t.engine.run(&id, Unverified::Ask, None), async {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        t.engine.run(&id, Unverified::Ask, None).await
    });
    assert_eq!(r1.unwrap().sha256, expected_sha256(14, 8 * MIB));
    assert!(matches!(r2, Err(Fail::Io(m)) if m.contains("already running")));
}

#[test]
fn n05_only_internet_zone_counts_as_protected() {
    use gdm_core::finalize::zone_internet;
    assert!(zone_internet(Some("[ZoneTransfer]\r\nZoneId=3\r\nHostUrl=https://x/\r\n")));
    assert!(zone_internet(Some("[ZoneTransfer]\nZoneId=4\n")));
    assert!(!zone_internet(Some("[ZoneTransfer]\r\nZoneId=0\r\n")) && !zone_internet(Some("")) && !zone_internet(None));
    assert!(!zone_internet(Some("ZoneId=30")));
}
