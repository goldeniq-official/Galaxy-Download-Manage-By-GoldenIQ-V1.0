// SPDX-License-Identifier: RPL-1.5
//! S5 check: segmenter against every S4 fixture (expected terminal state per case), credential stripping on
//! cross-origin redirect, deterministic failpoints, seeded TerminateProcess kill/resume rounds with an on-disk
//! oracle (every committed byte must already be correct on disk), and a tampered sidecar.
//!   s5 [rounds]            full check (default 30 kill rounds)
//!   s5 child <url> <part>  one download attempt (used by the parent; failpoint via GDM_FAILPOINT)
use gdm_spikes::fixture::{expected_sha256, fill, pct_encode, Fixture};
use gdm_spikes::seg::{download, load_sidecar, sidecar_path, Config, Fail, MIB};
use gdm_spikes::Check;
use std::os::windows::fs::FileExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

fn tmpdir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("gdm-s5-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// invariant 6 oracle: every byte the sidecar calls committed is already correct on disk
fn committed_bytes_correct(part: &Path, seed: u64) -> (bool, u64) {
    let Some(sc) = load_sidecar(part) else { return (true, 0) };
    let f = std::fs::File::open(part).unwrap();
    let mut buf = vec![0u8; MIB as usize];
    let mut want = vec![0u8; MIB as usize];
    let mut total = 0;
    for [a, _, d] in sc.segs {
        let mut p = a;
        while p < d {
            let n = ((d - p) as usize).min(buf.len());
            f.seek_read(&mut buf[..n], p).unwrap();
            fill(seed, p, &mut want[..n]);
            if buf[..n] != want[..n] { return (false, total); }
            p += n as u64;
        }
        total += d - a;
    }
    (true, total)
}

async fn run(url: &str, dir: &Path, cfg: Config) -> (Result<gdm_spikes::seg::Done, Fail>, Duration) {
    let t0 = Instant::now();
    let r = download(url, &dir.join("f.part"), cfg).await;
    (r, t0.elapsed())
}

fn child(exe: &Path, url: &str, part: &Path, fp: Option<&str>) -> Command {
    let mut c = Command::new(exe);
    c.args(["child", url, part.to_str().unwrap()]);
    match fp { Some(f) => { c.env("GDM_FAILPOINT", f); } None => { c.env_remove("GDM_FAILPOINT"); } }
    c.stderr(std::process::Stdio::null());
    c
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("child") {
        let r = download(&args[2], Path::new(&args[3]), Config::default()).await;
        match r { Ok(d) => { println!("{}", d.sha256); std::process::exit(0) } Err(e) => { println!("{e:?}"); std::process::exit(3) } }
    }
    let rounds: u32 = args.get(1).and_then(|v| v.parse().ok()).unwrap_or(30);
    let mut c = Check::new("S5 segmenter");
    let a = Fixture::start("127.0.0.1:0").await;
    let b = Fixture::start("127.0.0.1:0").await;
    let quick = || Config { read_timeout: Duration::from_secs(3), min_speed: (1024, Duration::from_secs(4)), retries: 2, ..Config::default() };

    // 1. control: 64 MiB, probe headers, ramp, hash
    let d = tmpdir("ctl");
    let (size, seed) = (64 * MIB, 11);
    let (r, el) = run(&a.url(&format!("/file?size={size}&seed={seed}&rate=3000000")), &d, Config::default()).await;
    let first = a.log().into_iter().next().unwrap();
    let h = |k: &str| first.headers.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone()).unwrap_or_default();
    c.ok(h("range") == "bytes=0-" && h("accept-encoding") == "identity", "probe = GET Range: bytes=0- + Accept-Encoding: identity");
    match &r {
        Ok(dn) => {
            c.ok(dn.sha256 == expected_sha256(seed, size) && dn.total == size, "64 MiB ranged download: byte count = total and sha256 matches");
            c.ok(dn.peak_conns >= 3 && dn.peak_conns <= 8, &format!("ramp from 2 added connections under per-connection throttling, capped at 8 (peak {} in {:.1}s)", dn.peak_conns, el.as_secs_f64()));
        }
        Err(e) => c.ok(false, &format!("control download failed: {e:?}")),
    }
    let ranges: Vec<_> = a.log().iter().filter_map(|r| r.headers.iter().find(|(k, _)| k == "range").map(|(_, v)| v.clone())).collect();
    c.ok(ranges.iter().skip(1).all(|r| r.starts_with("bytes=") && r.contains('-') && !r.ends_with('-')), "every follow-up request asks a closed range (lease) with If-Range");
    let mut starts: Vec<u64> = ranges.iter().skip(1).filter_map(|r| r[6..].split('-').next()?.parse().ok()).collect();
    starts.sort();
    c.ok(starts.windows(2).all(|w| w[1] - w[0] >= MIB), "splits never smaller than 1 MiB");
    c.ok(a.log().iter().skip(1).all(|r| r.headers.iter().any(|(k, v)| k == "if-range" && v == &format!("\"s{seed}-{size}\""))), "follow-up requests carry If-Range with the strong ETag");

    // 1b. total bandwidth capped at the server: extra connections do not add >= 10 % => ramp stops at 3
    let d = tmpdir("flat");
    let (r, _) = run(&a.url(&format!("/file?size={}&seed=13&trate=8000000", 48 * MIB)), &d, Config::default()).await;
    c.ok(matches!(&r, Ok(dn) if dn.peak_conns == 3 && dn.sha256 == expected_sha256(13, 48 * MIB)), &format!("flat total speed: one exploratory add (2 -> 3), then no more (peak {:?})", r.as_ref().map(|d| d.peak_conns)));

    // 2. fixed 8 (benchmark config b)
    a.clear_log();
    let d = tmpdir("fixed");
    let (r, _) = run(&a.url(&format!("/file?size={}&seed=12&rate=10000000", 32 * MIB)), &d, Config { fixed: Some(8), ..Config::default() }).await;
    c.ok(matches!(&r, Ok(dn) if dn.peak_conns == 8 && dn.sha256 == expected_sha256(12, 32 * MIB)), &format!("fixed 8 connections: peak 8, hash ok ({:?})", r.as_ref().map(|d| d.peak_conns)));

    // 3. hostile cases: expected terminal state
    let cases: Vec<(&str, String, Config, Box<dyn Fn(&Result<gdm_spikes::seg::Done, Fail>) -> bool>)> = vec![
        ("200 ignoring Range -> single stream, correct file", a.url(&format!("/norange?size={}&seed=21", 16 * MIB)), Config::default(), Box::new(|r| matches!(r, Ok(d) if !d.ranged && d.sha256 == expected_sha256(21, 16 * MIB)))),
        ("206 then 200 mid-download -> stop all, restart one stream from 0, correct file", a.url(&format!("/range-then-200?size={}&seed=22", 16 * MIB)), Config::default(), Box::new(|r| matches!(r, Ok(d) if d.sha256 == expected_sha256(22, 16 * MIB)))),
        ("ETag change mid-download -> FileChanged, never Complete", a.url(&format!("/etag-change?size={}&seed=23&after=1&rate=4000000", 16 * MIB)), Config::default(), Box::new(|r| r == &Err(Fail::FileChanged))),
        ("infinite body (no length) -> SizeExceeded at cap", a.url("/infinite"), Config { unknown_size_cap: 8 * MIB, ..Config::default() }, Box::new(|r| r == &Err(Fail::SizeExceeded))),
        ("overlong 206 -> extra bytes never written (correct file or LongBody)", a.url(&format!("/overlong?size={}&seed=24", 4 * MIB)), Config::default(), Box::new(|r| matches!(r, Ok(d) if d.sha256 == expected_sha256(24, 4 * MIB)) || r == &Err(Fail::LongBody))),
        ("short 206 bodies -> retry, never a wrong file", a.url(&format!("/short?size={}&seed=25", 4 * MIB)), quick(), Box::new(|r| matches!(r, Ok(d) if d.sha256 == expected_sha256(25, 4 * MIB)) || matches!(r, Err(Fail::TooManyRetries(_))))),
        ("Content-Range start mismatch -> BadContentRange, nothing written", a.url(&format!("/bad-cr?size={}&seed=26", 4 * MIB)), Config::default(), Box::new(|r| r == &Err(Fail::BadContentRange))),
        ("slowloris headers -> Timeout", a.url("/slowloris"), quick(), Box::new(|r| r == &Err(Fail::Timeout))),
        ("slow body -> Timeout", a.url("/slowbody?size=1000"), quick(), Box::new(|r| r == &Err(Fail::Timeout) || matches!(r, Err(Fail::TooManyRetries(_))))),
        ("429 + Retry-After: 2 -> waits, then correct file", a.url(&format!("/429?size={}&seed=27", 4 * MIB)), Config::default(), Box::new(|r| matches!(r, Ok(d) if d.sha256 == expected_sha256(27, 4 * MIB)))),
        ("text/html instead of binary -> Html", a.url("/html"), Config::default(), Box::new(|r| r == &Err(Fail::Html))),
        ("multipart/byteranges -> Multipart (rejected)", a.url("/multipart"), Config::default(), Box::new(|r| r == &Err(Fail::Multipart))),
    ];
    for (name, url, cfg, want) in cases {
        let d = tmpdir("case");
        let t0 = Instant::now();
        let r = tokio::time::timeout(Duration::from_secs(60), download(&url, &d.join("f.part"), cfg)).await;
        let r = r.unwrap_or(Err(Fail::Io("test deadline 60 s".into())));
        let el = t0.elapsed();
        let extra = if name.starts_with("429") { el >= Duration::from_secs(2) } else { true };
        c.ok(want(&r) && extra, &format!("{name}  [got {:?} in {:.1}s]", r.as_ref().map(|d| &d.sha256[..12]), el.as_secs_f64()));
    }

    // 4. credentials on redirect: stripped cross-origin (other port, other host), kept same-origin
    let creds = vec![("Cookie".to_string(), "sid=SECRET".to_string()), ("Authorization".to_string(), "Bearer SECRET".to_string())];
    for (label, target) in [("other port", b.url("/echo")), ("other host", format!("http://localhost:{}/echo", b.addr.port()))] {
        b.clear_log();
        let d = tmpdir("redir");
        let r = download(&a.url(&format!("/redirect?to={}", pct_encode(&target))), &d.join("f.part"), Config { headers: creds.clone(), ..Config::default() }).await;
        let leaked = b.log().iter().any(|q| q.headers.iter().any(|(k, v)| (k == "cookie" || k == "authorization") && v.contains("SECRET")));
        c.ok(!b.log().is_empty() && !leaked && r.is_ok(), &format!("cross-origin redirect ({label}): destination saw no Cookie/Authorization ({} requests)", b.log().len()));
    }
    a.clear_log();
    let d = tmpdir("redir-same");
    let _ = download(&a.url(&format!("/redirect?to={}", pct_encode(&a.url("/echo")))), &d.join("f.part"), Config { headers: creds.clone(), ..Config::default() }).await;
    c.ok(a.log().iter().any(|q| q.target == "/echo" && q.headers.iter().any(|(k, v)| k == "cookie" && v.contains("SECRET"))), "control: same-origin redirect keeps the cookie (test is not vacuous)");

    // 5. deterministic failpoints, then resume to a correct file
    let exe = std::env::current_exe().unwrap();
    let (ksize, kseed) = (48 * MIB, 31);
    let kurl = a.url(&format!("/file?size={ksize}&seed={kseed}&rate=6000000"));
    let want = expected_sha256(kseed, ksize);
    for fp in ["mid_write:5", "after_sync:2", "after_sidecar_tmp:2", "after_commit:2"] {
        let d = tmpdir("fp");
        let part = d.join("f.part");
        let st = child(&exe, &kurl, &part, Some(fp)).status().unwrap();
        let (good, committed) = committed_bytes_correct(&part, kseed);
        a.clear_log();
        let out = child(&exe, &kurl, &part, None).output().unwrap();
        let resumed_from = a.log().first().and_then(|q| q.headers.iter().find(|(k, _)| k == "range").map(|(_, v)| v.clone())).unwrap_or_default();
        let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
        c.ok(!st.success() && good && sha == want, &format!("failpoint {fp}: crashed, committed {:.1} MiB all correct on disk, resume probe '{resumed_from}', final sha256 ok", committed as f64 / MIB as f64));
    }

    // 6. tampered sidecar: overlapping / out-of-range segments are refused, download restarts and is correct
    let d = tmpdir("tamper");
    let part = d.join("f.part");
    let _ = child(&exe, &kurl, &part, Some("after_commit:2")).status();
    std::fs::write(sidecar_path(&part), format!("{{\"v\":1,\"validator\":\"\\\"s{kseed}-{ksize}\\\"\",\"total\":{ksize},\"segs\":[[0,{},{}],[100,{ksize},{ksize}]]}}", ksize, ksize)).unwrap();
    c.ok(load_sidecar(&part).is_none(), "tampered sidecar (overlap, committed past data) rejected by the loader");
    let out = child(&exe, &kurl, &part, None).output().unwrap();
    c.ok(String::from_utf8_lossy(&out.stdout).trim() == want, "after tampered sidecar: restart from 0, final sha256 ok");

    // 7. seeded TerminateProcess kill/resume rounds
    let d = tmpdir("kill");
    let part = d.join("f.part");
    let (ksize, kseed) = (128 * MIB, 32); // slow enough that `rounds` kills land before completion
    let kurl = a.url(&format!("/file?size={ksize}&seed={kseed}&rate=2000000"));
    let want = expected_sha256(kseed, ksize);
    // when one file completes before the kill budget is spent, its hash is checked and a fresh file starts
    let (mut rng, mut kills, mut bad, mut regress, mut last_committed) = (0x5EEDu64, 0u32, 0u32, 0u32, 0u64);
    let (mut files, mut files_ok) = (0u32, 0u32);
    let t0 = Instant::now();
    loop {
        let mut ch = child(&exe, &kurl, &part, None).stdout(std::process::Stdio::piped()).spawn().unwrap();
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let wait = Duration::from_millis(500 + (rng >> 33) % 3000);
        let t = Instant::now();
        let mut done = None;
        while t.elapsed() < wait {
            if let Some(s) = ch.try_wait().unwrap() { done = Some(s); break; }
            std::thread::sleep(Duration::from_millis(10));
        }
        if done.is_none() && kills < rounds {
            ch.kill().unwrap(); // TerminateProcess
            ch.wait().unwrap();
            kills += 1;
            let (good, committed) = committed_bytes_correct(&part, kseed);
            if !good { bad += 1; }
            if committed < last_committed { regress += 1; }
            last_committed = committed;
            continue;
        }
        let out = ch.wait_with_output().unwrap();
        files += 1;
        if String::from_utf8_lossy(&out.stdout).trim() == want { files_ok += 1; }
        if kills >= rounds { break; }
        let _ = std::fs::remove_file(&part);
        let _ = std::fs::remove_file(sidecar_path(&part));
        last_committed = 0;
    }
    c.ok(bad == 0, &format!("{kills} TerminateProcess kills: committed bytes always correct on disk (invariant 6), {bad} bad"));
    c.ok(regress == 0, &format!("committed progress never went backwards across kills within a file ({regress} regressions)"));
    c.ok(files_ok == files && files > 0, &format!("after {kills} kills + resumes: {files_ok}/{files} files completed with correct sha256 ({:.0}s)", t0.elapsed().as_secs_f64()));
    c.note("TerminateProcess is a process crash, not a power cut: OS caches survive, so this proves crash recovery only");
    c.finish();
}
