// SPDX-License-Identifier: RPL-1.5
//! E-01, E-02, E-03 and the rest of the §09 server-response table against the hostile fixture.
//! Expected terminal state per case: a correct file (oracle sha256) or the right error; never a wrong file
//! with a Complete state.
mod common;
use common::*;
use gdm_core::{Fail, Unverified};
use gdm_fixture::{expected_sha256, Fixture};
use std::time::{Duration, Instant};

async fn fx() -> Fixture { Fixture::start("127.0.0.1:0").await }

#[tokio::test(flavor = "multi_thread")]
async fn control_ranged_ramp_probe_headers() {
    let a = fx().await;
    let t = engine("ctl", gdm_core::seg::Config::default());
    let (size, seed) = (64 * MIB, 11);
    let (_, r) = t.get(&a.url(&format!("/file?size={size}&seed={seed}&rate=3000000"))).await;
    let o = r.unwrap();
    assert_eq!(o.sha256, expected_sha256(seed, size));
    assert!(o.transfer.ranged && (3..=8).contains(&o.transfer.peak_conns), "ramp from 2, cap 8: peak {}", o.transfer.peak_conns);
    let log = a.log();
    let h = |i: usize, k: &str| log[i].headers.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone()).unwrap_or_default();
    assert_eq!((h(0, "range").as_str(), h(0, "accept-encoding").as_str()), ("bytes=0-", "identity"), "probe");
    for i in 1..log.len() {
        assert!(h(i, "range").starts_with("bytes=") && !h(i, "range").ends_with('-'), "closed range lease");
        assert_eq!(h(i, "if-range"), format!("\"s{seed}-{size}\""), "If-Range with the strong ETag on every follow-up");
    }
    let mut starts: Vec<u64> = log.iter().skip(1).filter_map(|r| r.headers.iter().find(|(k, _)| k == "range").and_then(|(_, v)| v[6..].split('-').next()?.parse().ok())).collect();
    starts.sort();
    assert!(starts.windows(2).all(|w| w[1] - w[0] >= MIB), "splits never below 1 MiB");
    assert_eq!(t.files(), vec![format!("file")], "only the final file remains (no .part, no sidecar)");
}

#[tokio::test(flavor = "multi_thread")]
async fn e01_200_ignoring_range_single_stream() {
    let a = fx().await;
    let t = engine("e01a", quick());
    let (_, r) = t.get(&a.url(&format!("/norange?size={}&seed=21", 16 * MIB))).await;
    let o = r.unwrap();
    assert!(!o.transfer.ranged && o.sha256 == expected_sha256(21, 16 * MIB));
}

#[tokio::test(flavor = "multi_thread")]
async fn e01_206_then_200_mid_download_restarts_from_zero() {
    let a = fx().await;
    let t = engine("e01b", quick());
    let (_, r) = t.get(&a.url(&format!("/range-then-200?size={}&seed=22&rate=8000000", 16 * MIB))).await;
    let o = r.unwrap();
    assert_eq!(o.sha256, expected_sha256(22, 16 * MIB), "final hash correct after the 200 on a range");
    assert!(!o.transfer.ranged, "host marked no-range: one sequential stream");
}

#[tokio::test(flavor = "multi_thread")]
async fn e01_bad_content_range_and_multipart_rejected() {
    let a = fx().await;
    let t = engine("e01c", quick());
    assert_eq!(t.get(&a.url(&format!("/bad-cr?size={}&seed=26", 4 * MIB))).await.1.unwrap_err(), Fail::BadContentRange);
    assert_eq!(t.get(&a.url("/multipart")).await.1.unwrap_err(), Fail::Multipart);
    assert_eq!(t.get(&a.url("/html")).await.1.unwrap_err(), Fail::Html);
    assert!(t.files().iter().all(|f| f.ends_with(".part") || f.ends_with(".gdm")), "no final name for a refused file: {:?}", t.files());
}

#[tokio::test(flavor = "multi_thread")]
async fn e02_file_changed_mid_download_pauses_never_complete() {
    let a = fx().await;
    let t = engine("e02a", quick());
    let (id, r) = t.get(&a.url(&format!("/etag-change?size={}&seed=23&after=1&rate=4000000", 16 * MIB))).await;
    assert_eq!(r.unwrap_err(), Fail::FileChanged);
    let j = t.engine.job(&id).unwrap();
    assert_eq!(j.state, "paused");
    assert!(j.error_code.unwrap().starts_with("FileChanged"));
    assert!(t.files().iter().any(|f| f.ends_with(".part")), "old .part kept");
    assert!(!t.files().contains(&"etag-change".to_string()), "never a final file");
}

#[tokio::test(flavor = "multi_thread")]
async fn e02_changed_between_sessions_is_detected_on_resume() {
    // the job checkpoints under ETag A, the process "restarts", the server now serves ETag B (same size)
    let a = fx().await;
    let t = engine("e02b", quick());
    let url = a.url(&format!("/etag-change?size={}&seed=40&after=3&rate=2000000", 64 * MIB));
    let id = t.engine.add(t.job(&url)).unwrap();
    let r = tokio::time::timeout(Duration::from_millis(2500), t.engine.run(&id, Unverified::Ask, None)).await;
    assert!(r.is_err() || r.unwrap().is_err(), "first run interrupted");
    let r = t.engine.run(&id, Unverified::Ask, None).await;
    assert_eq!(r.unwrap_err(), Fail::FileChanged, "200 under If-Range / new validator on resume");
}

#[tokio::test(flavor = "multi_thread")]
async fn e02_no_validator_unverified_and_restart_asks() {
    let a = fx().await;
    let t = engine("e02c", quick());
    let url = a.url(&format!("/novalidator?size={}&seed=41", 4 * MIB));
    let (_, r) = t.get(&url).await;
    let o = r.unwrap();
    assert!(o.unverified, "no strong validator => 'unverified' stays on the result");
    assert_eq!(o.sha256, expected_sha256(41, 4 * MIB));
    // interrupted job without validator, then a new session: Ask => NeedsDecision; Restart => correct file
    let url = a.url(&format!("/novalidator?size={}&seed=42&rate=1500000", 48 * MIB));
    let id = t.engine.add(t.job(&url)).unwrap();
    let _ = tokio::time::timeout(Duration::from_millis(3000), t.engine.run(&id, Unverified::SameSession, None)).await;
    assert!(t.engine.job(&id).unwrap().done > 0, "progress was committed");
    assert!(matches!(t.engine.run(&id, Unverified::Ask, None).await, Err(Fail::NeedsDecision(_))));
    let o = t.engine.run(&id, Unverified::Restart, None).await.unwrap();
    assert_eq!(o.sha256, expected_sha256(42, 48 * MIB));
}

#[tokio::test(flavor = "multi_thread")]
async fn e02_416_on_resume_is_size_changed() {
    let a = fx().await;
    let t = engine("e02d", quick());
    // resume past the end: the fixture answers 416
    let url = a.url(&format!("/file?size={}&seed=43&rate=2000000", 64 * MIB));
    let id = t.engine.add(t.job(&url)).unwrap();
    let _ = tokio::time::timeout(Duration::from_millis(2500), t.engine.run(&id, Unverified::Ask, None)).await;
    let smaller = a.url(&format!("/file?size={}&seed=43", 1024));
    t.engine.store.lock().unwrap().conn.execute("UPDATE jobs SET url = ?1, final_url = ?1 WHERE id = ?2", [&smaller, &id]).unwrap();
    let r = t.engine.run(&id, Unverified::Ask, None).await;
    assert!(matches!(r, Err(Fail::SizeChanged) | Err(Fail::FileChanged)), "{r:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn e03_infinite_body_stops_at_cap_disk_not_filled() {
    let a = fx().await;
    let t = engine("e03a", gdm_core::seg::Config { unknown_cap: Some(8 * MIB), ..quick() });
    let (_, r) = t.get(&a.url("/infinite")).await;
    assert_eq!(r.unwrap_err(), Fail::SizeExceeded);
    let part = t.files().into_iter().find(|f| f.ends_with(".part")).unwrap();
    assert!(std::fs::metadata(t.dl.join(part)).unwrap().len() <= 8 * MIB, "never more than the cap on disk");
}

#[tokio::test(flavor = "multi_thread")]
async fn e03_overlong_body_extra_bytes_never_written() {
    let a = fx().await;
    let t = engine("e03b", quick());
    let (_, r) = t.get(&a.url(&format!("/overlong?size={}&seed=24", 4 * MIB))).await;
    match r {
        Ok(o) => assert_eq!(o.sha256, expected_sha256(24, 4 * MIB)),
        Err(e) => assert_eq!(e, Fail::LongBody),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn e03_declared_size_larger_than_free_space_refused_before_preallocation() {
    let a = fx().await;
    let t = engine("e03c", quick());
    let (_, r) = t.get(&a.url("/file?size=100000000000000&seed=1")).await; // 100 TB
    assert_eq!(r.unwrap_err(), Fail::DiskFull);
    let part = t.files().into_iter().find(|f| f.ends_with(".part")).unwrap();
    assert_eq!(std::fs::metadata(t.dl.join(part)).unwrap().len(), 0, "nothing preallocated");
}

#[tokio::test(flavor = "multi_thread")]
async fn short_bodies_retry_never_wrong_file() {
    let a = fx().await;
    let t = engine("short", quick());
    match t.get(&a.url(&format!("/short?size={}&seed=25", 4 * MIB))).await.1 {
        Ok(o) => assert_eq!(o.sha256, expected_sha256(25, 4 * MIB)),
        Err(e) => assert!(matches!(e, Fail::TooManyRetries(_)), "{e:?}"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn e07_slowloris_headers_time_out() {
    let a = fx().await;
    let t = engine("loris", quick());
    assert_eq!(t.get(&a.url("/slowloris")).await.1.unwrap_err(), Fail::Timeout);
}

#[tokio::test(flavor = "multi_thread")]
async fn e07_slow_body_times_out() {
    let a = fx().await;
    let t = engine("slowbody", quick());
    let r = t.get(&a.url("/slowbody?size=1000")).await.1.unwrap_err();
    assert!(matches!(r, Fail::Timeout | Fail::TooManyRetries(_)), "{r:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn e05_429_retry_after_honoured() {
    let a = fx().await;
    let t = engine("429", quick());
    let t0 = Instant::now();
    let o = t.get(&a.url(&format!("/429?size={}&seed=27", 4 * MIB))).await.1.unwrap();
    assert!(t0.elapsed() >= Duration::from_secs(2) && o.sha256 == expected_sha256(27, 4 * MIB), "Retry-After honoured");
}
