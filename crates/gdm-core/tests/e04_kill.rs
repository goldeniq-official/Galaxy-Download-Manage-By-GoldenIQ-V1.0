// SPDX-License-Identifier: RPL-1.5
//! E-04: crash at every step (deterministic failpoints), then seeded TerminateProcess rounds against the
//! gdm-get child. Oracle after every kill: every byte the database AND the sidecar call committed is already
//! correct on disk, committed progress never goes backwards, and every finished file has the right sha256.
//! GDM_KILL_ROUNDS (default 20; the G5 evidence run uses 1000). TerminateProcess is a process crash, not a
//! power cut (OS caches survive).
mod common;
use common::*;
use gdm_core::sidecar;
use gdm_core::store::Store;
use gdm_fixture::{expected_sha256, fill, Fixture};
use std::os::windows::fs::FileExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const EXE: &str = env!("CARGO_BIN_EXE_gdm-get");

fn child(db: &Path, url: &str, dir: &Path, fp: Option<&str>) -> Command {
    let mut c = Command::new(EXE);
    c.arg(db).arg(url).arg(dir).stdout(Stdio::piped()).stderr(Stdio::null());
    match fp {
        Some(f) => {
            c.env("GDM_FAILPOINT", f);
        }
        None => {
            c.env_remove("GDM_FAILPOINT");
        }
    }
    c
}

/// bytes in [a, d) of every segment on disk == the fixture's bytes
fn ranges_ok(part: &Path, seed: u64, segs: &[[u64; 3]]) -> Result<u64, String> {
    let Ok(f) = std::fs::File::open(part) else {
        return if segs.iter().all(|s| s[2] == s[0]) { Ok(0) } else { Err("committed bytes but no .part".into()) };
    };
    let (mut buf, mut want) = (vec![0u8; MIB as usize], vec![0u8; MIB as usize]);
    let mut total = 0;
    for &[a, _, d] in segs {
        let mut p = a;
        while p < d {
            let n = ((d - p) as usize).min(buf.len());
            let got = f.seek_read(&mut buf[..n], p).map_err(|e| e.to_string())?;
            fill(seed, p, &mut want[..n]);
            if got != n || buf[..n] != want[..n] {
                return Err(format!("bad bytes in [{p}, {}) of segment starting {a}", p + n as u64));
            }
            p += n as u64;
        }
        total += d - a;
    }
    Ok(total)
}

/// (db committed, sidecar committed) after a crash, both checked against the disk
fn oracle(db: &Path, dir: &Path, url: &str, seed: u64) -> Result<(u64, u64), String> {
    let (st, _) = Store::open(db).map_err(|e| format!("db: {e}"))?;
    let id = st.job_ids().unwrap().into_iter().find(|id| st.job(id).unwrap().is_some_and(|j| j.url == url));
    let Some(id) = id else { return Ok((0, 0)) };
    let j = st.job(&id).unwrap().unwrap();
    let mut part = dir.join(format!("{}.part", j.name));
    if j.state == "finalizing" && !part.exists() {
        part = dir.join(&j.name);
    } // crashed after the final rename
    if j.state == "complete" {
        return Ok((u64::MAX, u64::MAX));
    }
    let db_c = ranges_ok(&part, seed, &st.segments(&id).unwrap()).map_err(|e| format!("DB claims: {e}"))?;
    let sc = std::fs::read(dir.join(sidecar::leaf(&j.name))).ok().and_then(|r| sidecar::decode(&r));
    let sc_c = match sc {
        Some(s) => ranges_ok(&part, seed, &s.segs).map_err(|e| format!("sidecar claims: {e}"))?,
        None => 0,
    };
    Ok((db_c, sc_c))
}

fn ok_line(out: &std::process::Output) -> Option<(String, String)> {
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    let v: Vec<&str> = s.split_whitespace().collect();
    (v.first() == Some(&"OK")).then(|| (v[1].to_string(), v[8..].join(" ")))
}

#[tokio::test(flavor = "multi_thread")]
async fn e04_failpoints_then_resume_correct() {
    let a = Fixture::start("127.0.0.1:0").await;
    let (size, seed) = (48 * MIB, 31);
    let want = expected_sha256(seed, size);
    for (i, fp) in [
        "lease_revoke:1",
        "after_ack:1",
        "mid_write:5",
        "after_sync:2",
        "sidecar_tmp:2",
        "after_sidecar:2",
        "after_commit:2",
        "after_save:1",
        "before_rename:1",
        "after_rename:1",
        "after_part_created:1",
        "after_reset:1",
        "after_reset_checkpoint:1",
        "after_truncate:1",
        "after_final_checkpoint:1",
        "before_finalizing:1",
        "after_finish:1",
    ]
    .iter()
    .enumerate()
    {
        let root = tmpdir("fp");
        let (db, dir) = (root.join("gdm.db"), root.join("dl"));
        std::fs::create_dir_all(&dir).unwrap();
        let url = a.url(&format!("/file?size={size}&seed={seed}&rate=6000000&i={i}"));
        let st = child(&db, &url, &dir, Some(fp)).output().unwrap();
        assert!(!st.status.success(), "{fp}: the child must have crashed");
        oracle(&db, &dir, &url, seed).unwrap_or_else(|e| panic!("{fp}: {e}"));
        let out = child(&db, &url, &dir, None).output().unwrap();
        let (sha, path) = ok_line(&out).unwrap_or_else(|| panic!("{fp}: resume failed: {}", String::from_utf8_lossy(&out.stdout)));
        assert_eq!(sha, want, "{fp}");
        assert_eq!(sha_file(Path::new(&path)), want, "{fp}: file on disk");
        let finals: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        assert_eq!(finals.len(), 1, "{fp}: exactly one file, no duplicate, no leftovers: {finals:?}");
        let _ = std::fs::remove_dir_all(&root);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn e04_kill_rounds() {
    let rounds: u32 = std::env::var("GDM_KILL_ROUNDS").ok().and_then(|v| v.parse().ok()).unwrap_or(20);
    let a = Fixture::start("127.0.0.1:0").await;
    let root = tmpdir("kill");
    let (db, dir) = (root.join("gdm.db"), root.join("dl"));
    std::fs::create_dir_all(&dir).unwrap();
    let size = 128 * MIB;
    let (mut rng, mut kills, mut files, mut files_ok) = (0x5EEDu64, 0u32, 0u32, 0u32);
    let (mut bad, mut regress, mut last) = (vec![], 0u32, 0u64);
    let mut seed = 1000u64;
    let mut url = a.url(&format!("/file?size={size}&seed={seed}&rate=2000000"));
    let t0 = Instant::now();
    while kills < rounds {
        let mut ch = child(&db, &url, &dir, None).spawn().unwrap();
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let wait = Duration::from_millis(300 + (rng >> 33) % 2700);
        let t = Instant::now();
        let mut exited = false;
        while t.elapsed() < wait {
            if ch.try_wait().unwrap().is_some() {
                exited = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        if !exited {
            ch.kill().unwrap(); // TerminateProcess
            ch.wait().unwrap();
            kills += 1;
            match oracle(&db, &dir, &url, seed) {
                Ok((d, s)) => {
                    if d != u64::MAX && d < last {
                        regress += 1;
                    }
                    if d != u64::MAX {
                        last = d.max(s.min(d));
                    }
                }
                Err(e) => bad.push(format!("kill {kills}: {e}")),
            }
            continue;
        }
        let out = ch.wait_with_output().unwrap();
        files += 1;
        if let Some((sha, path)) = ok_line(&out) {
            if sha == expected_sha256(seed, size) && sha_file(Path::new(&path)) == sha {
                files_ok += 1;
            }
            let _ = std::fs::remove_file(path);
        } else {
            bad.push(format!("file {files} failed: {}", String::from_utf8_lossy(&out.stdout).trim()));
        }
        seed += 1;
        last = 0;
        url = a.url(&format!("/file?size={size}&seed={seed}&rate=2000000"));
    }
    // finish the last file
    let out = child(&db, &url, &dir, None).output().unwrap();
    files += 1;
    if ok_line(&out).is_some_and(|(sha, p)| sha == expected_sha256(seed, size) && sha_file(Path::new(&p)) == sha) {
        files_ok += 1;
    }
    println!("E-04 kill rounds: {kills} TerminateProcess kills, {} oracle failures, {regress} progress regressions, {files_ok}/{files} files with correct sha256 ({:.0} s)", bad.len(), t0.elapsed().as_secs_f64());
    assert!(bad.is_empty(), "{bad:?}");
    assert_eq!(regress, 0);
    assert_eq!(files_ok, files);
    let _ = std::fs::remove_dir_all(&root);
}

/// crash after the final rename, then the published file is changed before the next start: recovery must
/// not bless it (the hash recorded before the rename differs)
#[tokio::test(flavor = "multi_thread")]
async fn e04_published_file_changed_after_crash_is_not_completed() {
    let a = Fixture::start("127.0.0.1:0").await;
    let root = tmpdir("tamper-final");
    let (db, dir) = (root.join("gdm.db"), root.join("dl"));
    std::fs::create_dir_all(&dir).unwrap();
    let url = a.url(&format!("/file?size={}&seed=77", 4 * MIB));
    assert!(!child(&db, &url, &dir, Some("after_rename:1")).status().unwrap().success());
    let fin = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.path()).find(|p| !p.to_string_lossy().contains(".part")).unwrap();
    let f = std::fs::OpenOptions::new().write(true).open(&fin).unwrap();
    f.seek_write(b"XXXX", 1000).unwrap(); // same size, same file object, different bytes
    drop(f);
    let out = child(&db, &url, &dir, None).output().unwrap();
    let line = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(line.starts_with("FAIL NeedsDecision"), "{line}");
    let _ = std::fs::remove_dir_all(&root);
}

/// the rename happened, then recording "complete" failed and the run marked the job failed: the next run must
/// finish from the recorded intent, never download a second copy
#[tokio::test(flavor = "multi_thread")]
async fn e04_intent_survives_a_failed_state() {
    let a = Fixture::start("127.0.0.1:0").await;
    let root = tmpdir("intent-failed");
    let (db, dir) = (root.join("gdm.db"), root.join("dl"));
    std::fs::create_dir_all(&dir).unwrap();
    let url = a.url(&format!("/file?size={}&seed=78", 4 * MIB));
    assert!(!child(&db, &url, &dir, Some("after_rename:1")).status().unwrap().success());
    {
        let (st, _) = Store::open(&db).unwrap();
        st.conn.execute("UPDATE jobs SET state = 'failed', error_code = 'Io: simulated'", []).unwrap();
    }
    let out = child(&db, &url, &dir, None).output().unwrap();
    let (sha, _) = ok_line(&out).unwrap_or_else(|| panic!("{}", String::from_utf8_lossy(&out.stdout)));
    assert_eq!(sha, expected_sha256(78, 4 * MIB));
    let names: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    assert_eq!(names.len(), 1, "one file, no second download: {names:?}");
    let _ = std::fs::remove_dir_all(&root);
}
