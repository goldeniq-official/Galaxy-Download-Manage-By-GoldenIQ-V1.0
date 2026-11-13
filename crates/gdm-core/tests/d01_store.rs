// SPDX-License-Identifier: RPL-1.5
//! D-01 (corrupt database -> backup + sidecar recovery), D-02 (newer schema -> read-only), backups, untrusted
//! sidecars, and the 3-level limiter (L-01).
mod common;
use common::*;
use gdm_core::limiter::{Bucket, JobLimit};
use gdm_core::sidecar::{self, Sidecar};
use gdm_core::store::{self, Store};
use gdm_core::{seg, Engine, Fail, NewJob, Unverified};
use gdm_fixture::{expected_sha256, Fixture};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[tokio::test(flavor = "multi_thread")]
async fn d01_truncated_database_restored_and_jobs_recovered() {
    let a = Fixture::start("127.0.0.1:0").await;
    let root = tmpdir("d01");
    let (db, dl) = (root.join("data").join("gdm.db"), root.join("Downloads"));
    std::fs::create_dir_all(&dl).unwrap();
    let (done_id, part_id);
    {
        let e = Engine::open(&db, quick()).unwrap();
        done_id = e.add(NewJob { url: a.url("/p/300000/50/0/a.bin"), dir: dl.clone(), typed: true, ..Default::default() }).unwrap();
        e.run(&done_id, Unverified::Ask, None).await.unwrap();
        e.store.lock().unwrap().backup().unwrap(); // the daily backup ran after job 1
        part_id =
            e.add(NewJob { url: a.url(&format!("/p/{}/51/1500000/b.bin", 48 * MIB)), dir: dl.clone(), typed: true, ..Default::default() }).unwrap();
        run_until_checkpoint(&e, &part_id, Unverified::Ask, Duration::from_secs(20)).await;
        assert!(e.job(&part_id).unwrap().done > 0, "job 2 checkpointed some bytes");
    } // engine closed: WAL checkpointed into the main file
    tokio::time::sleep(Duration::from_millis(500)).await;
    let len = std::fs::metadata(&db).unwrap().len();
    std::fs::OpenOptions::new().write(true).open(&db).unwrap().set_len(len / 2).unwrap(); // cut in half
    let e = Engine::open(&db, quick()).unwrap();
    assert!(e.report.quarantined.is_some() && e.report.restored_from.is_some(), "{:?}", e.report);
    assert_eq!(e.job(&done_id).unwrap().state, "complete", "job from the backup is back");
    let j2 = e.job(&part_id).expect("job 2 recovered from its sidecar");
    assert_eq!(j2.state, "paused");
    // W-02: the sidecar carries no "user typed this private address" intent: refused until confirmed again
    let r = e.run(&part_id, Unverified::Ask, None).await;
    assert!(matches!(r, Err(Fail::Forbidden(_))), "{r:?}");
    e.allow_private(&part_id, "127.0.0.1").unwrap();
    let o = e.run(&part_id, Unverified::Ask, None).await.unwrap();
    assert_eq!(o.sha256, expected_sha256(51, 48 * MIB), "recovered job resumes to a correct file");
    assert!(
        std::fs::read_dir(db.parent().unwrap()).unwrap().flatten().any(|f| f.file_name().to_string_lossy().contains(".corrupt-")),
        "corrupt db kept aside"
    );
}

#[test]
fn d02_newer_schema_opens_read_only() {
    let root = tmpdir("d02");
    let db = root.join("gdm.db");
    drop(Store::open(&db).unwrap());
    rusqlite_set_version(&db, store::SCHEMA + 5);
    let (st, rep) = Store::open(&db).unwrap();
    assert!(rep.read_only && st.read_only);
    let e = Engine::open(&db, quick()).unwrap();
    let r = tokio::runtime::Runtime::new().unwrap().block_on(e.run("x", Unverified::Ask, None));
    assert!(matches!(r, Err(Fail::Io(m)) if m.contains("read-only")));
}

fn rusqlite_set_version(db: &std::path::Path, v: i64) {
    let (st, _) = Store::open(db).unwrap();
    st.conn.pragma_update(None, "user_version", v).unwrap();
}

#[test]
fn backups_are_checked_and_seven_kept() {
    let root = tmpdir("bk");
    let (st, rep) = Store::open(&root.join("gdm.db")).unwrap();
    assert!(rep.backup_made.is_some(), "first open makes the first backup");
    for _ in 0..9 {
        st.backup().unwrap();
    }
    let b = store::backups(&st.path);
    assert_eq!(b.len(), store::KEEP_BACKUPS);
    assert!(st.backup_if_due().unwrap().is_none(), "not due again within 24 h");
    assert!(!std::fs::read_dir(root.join("backups")).unwrap().flatten().any(|f| f.file_name().to_string_lossy().ends_with(".tmp")));
}

fn good() -> Sidecar {
    Sidecar {
        v: 1,
        job: "abc-1".into(),
        url: "https://h.example/f.bin".into(),
        final_url: "https://cdn.example/f.bin".into(),
        name: "f.bin".into(),
        validator: Some("\"e\"".into()),
        total: 100,
        epoch: 0,
        fid: "0123abcd:00000000000000ff".into(),
        segs: vec![[0, 50, 50], [50, 100, 70]],
    }
}

#[test]
fn sidecar_untrusted_shapes_rejected() {
    let ok = sidecar::encode(&good()).unwrap();
    assert_eq!(sidecar::decode(&ok), Some(good()));
    let bad = |f: &dyn Fn(&mut Sidecar)| {
        let mut s = good();
        f(&mut s);
        sidecar::decode(&serde_json::to_vec(&s).unwrap())
    };
    assert!(bad(&|s| s.segs = vec![[0, 60, 60], [50, 100, 70]]).is_none(), "overlap");
    assert!(bad(&|s| s.segs = vec![[0, 50, 50], [60, 100, 70]]).is_none(), "gap");
    assert!(bad(&|s| s.segs = vec![[0, 100, 101]]).is_none(), "committed past end");
    assert!(bad(&|s| s.segs = vec![[0, 50, 50], [50, 120, 70]]).is_none(), "past total");
    assert!(bad(&|s| s.name = "..\\evil.exe".into()).is_none(), "path in name");
    assert!(bad(&|s| s.url = "file:///C:/x".into()).is_none(), "scheme");
    assert!(bad(&|s| s.url = "https://h.example/f?token=x".into()).is_none(), "query (secrets) never in a sidecar");
    assert!(bad(&|s| s.job = "../x".into()).is_none(), "job id");
    assert!(bad(&|s| s.v = 2).is_none(), "version");
    assert!(bad(&|s| s.fid = "../x".into()).is_none(), "file id shape");
    assert!(sidecar::decode(&vec![b' '; sidecar::MAX + 1]).is_none(), "size cap before parsing");
    assert!(sidecar::decode(br#"{"v":1,"extra":1}"#).is_none(), "unknown fields");
    assert_eq!(sidecar::compact(&[[0, 10, 10], [10, 20, 20], [20, 30, 25], [30, 40, 30]]), vec![[0, 30, 25], [30, 40, 30]]);
}

/// L-01 (card test "limit 1 MB/s total, 5 jobs => +-5 %"): the bucket chain itself, 5 jobs x 3 connections
/// taking 16 KiB chunks for 10 s under one global 1 MB/s limit
#[tokio::test(flavor = "multi_thread")]
async fn l01_global_limit_five_jobs_three_connections_each() {
    let global = Bucket::new(1_000_000);
    let t0 = Instant::now();
    let dur = Duration::from_secs(10);
    let mut tasks = vec![];
    for _ in 0..5 {
        let lim = JobLimit { job: Bucket::new(0), queue: None, global: global.clone() };
        let n = Arc::new(std::sync::atomic::AtomicU64::new(0));
        for _ in 0..3 {
            let (lim, n) = (lim.clone(), n.clone());
            tasks.push(tokio::spawn(async move {
                while t0.elapsed() < dur {
                    lim.take(16 * 1024).await;
                    n.fetch_add(16 * 1024, std::sync::atomic::Ordering::Relaxed);
                }
                n
            }));
        }
    }
    let mut per_job = vec![];
    for (i, t) in tasks.into_iter().enumerate() {
        let n = t.await.unwrap();
        if i % 3 == 0 {
            per_job.push(n);
        }
    }
    let per_job: Vec<u64> = per_job.iter().map(|n| n.load(std::sync::atomic::Ordering::Relaxed)).collect();
    let total: u64 = per_job.iter().sum();
    let rate = total as f64 / t0.elapsed().as_secs_f64();
    assert!((rate - 1e6).abs() / 1e6 <= 0.05, "aggregate {rate:.0} B/s vs 1 000 000 (+-5 %)");
    let fair = total as f64 / 5.0;
    assert!(per_job.iter().all(|&j| (j as f64 - fair).abs() / fair <= 0.10), "each job within 10 % of a fair share: {per_job:?}");
}

/// engine level: 5 real downloads under one global limit never exceed it (the elapsed time also includes
/// probe, hash, MOTW and rename, so the lower bound is looser)
#[tokio::test(flavor = "multi_thread")]
async fn l01_engine_five_jobs_never_exceed_global() {
    let a = Fixture::start("127.0.0.1:0").await;
    let t = engine("l01", seg::Config::default());
    let global = Bucket::new(1_000_000);
    let size = 2 * MIB;
    let ids: Vec<String> = (0..5).map(|i| t.engine.add(t.job(&a.url(&format!("/file?size={size}&seed={}", 60 + i)))).unwrap()).collect();
    let t0 = Instant::now();
    let runs = ids.iter().map(|id| {
        let lim = Arc::new(JobLimit { job: Bucket::new(0), queue: None, global: global.clone() });
        let (e, id) = (&t.engine, id.clone());
        async move { e.run(&id, Unverified::Ask, Some(lim)).await }
    });
    let res = futures_util::future::join_all(runs).await;
    let rate = (5 * size) as f64 / t0.elapsed().as_secs_f64();
    assert!(res.iter().all(|r| r.is_ok()));
    // upper bound = the card (never above the limit); lower bound only catches a stalled limiter (elapsed also
    // holds probe, hash, MOTW and rename, and other tests may load the machine)
    assert!((500_000.0..=1_050_000.0).contains(&rate), "aggregate {rate:.0} B/s under a 1 000 000 B/s global limit");
}

/// a job limit under looser queue and global limits: the job limit wins
#[tokio::test(flavor = "multi_thread")]
async fn l01_job_limit_under_global() {
    let a = Fixture::start("127.0.0.1:0").await;
    let t = engine("l01b", seg::Config::default());
    let id = t.engine.add(t.job(&a.url(&format!("/file?size={}&seed=70", 3 * MIB)))).unwrap();
    let lim = Arc::new(JobLimit { job: Bucket::new(500_000), queue: Some(Bucket::new(2_000_000)), global: Bucket::new(4_000_000) });
    let t0 = Instant::now();
    t.engine.run(&id, Unverified::Ask, Some(lim)).await.unwrap();
    let rate = (3 * MIB) as f64 / t0.elapsed().as_secs_f64();
    assert!((350_000.0..=525_000.0).contains(&rate), "{rate:.0} B/s vs job limit 500 000");
}
