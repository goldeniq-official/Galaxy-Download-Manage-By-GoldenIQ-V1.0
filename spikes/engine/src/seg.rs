// SPDX-License-Identifier: RPL-1.5
//! S5 segmenter (§09): probe, 2 connections, ramp +1 per 2 s while speed rises >= 10 %, split the largest
//! remaining half (>= 1 MiB), one actor owns the segment map, leases carry a generation and a split waits
//! for the worker's boundary ack, .part preallocated, 1 MiB buffers, offsets committed only after
//! sync_data (8 MiB or 2 s). Sidecar `<part>.gdm` is untrusted on load.
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::os::windows::fs::FileExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub const MIB: u64 = 1 << 20;

#[derive(Clone)]
pub struct Config {
    pub cap: usize,           // default 8 (per-host profile can raise to MAX_CONNS)
    pub fixed: Option<usize>, // benchmark config (b): fixed connection count, no ramp
    pub min_split: u64,
    pub buf: usize,
    pub flush_bytes: u64,
    pub flush_every: Duration,
    pub read_timeout: Duration,
    pub min_speed: (u64, Duration), // trickle watchdog: < bytes/s over the window => reconnect (like curl --speed-limit)
    pub retries: u32,
    pub unknown_size_cap: u64,
    pub headers: Vec<(String, String)>,
}
pub const MAX_CONNS: usize = 32;

impl Default for Config {
    fn default() -> Self {
        Config {
            cap: 8, fixed: None, min_split: MIB, buf: MIB as usize, flush_bytes: 8 * MIB,
            flush_every: Duration::from_secs(2), read_timeout: Duration::from_secs(20), min_speed: (1024, Duration::from_secs(30)), retries: 10,
            unknown_size_cap: 1 << 40, headers: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Fail {
    FileChanged,      // validator differs / 200 under If-Range
    SizeChanged,      // 416
    Html,             // text/html instead of binary (R-03)
    Multipart,        // multipart/byteranges never requested
    BadContentRange,  // invariant 5
    LongBody,         // more bytes than requested
    SizeExceeded,     // unknown length body grew past the cap
    Auth,             // 401/403
    Gone,             // 404/410
    Http(u16),
    Timeout,
    TooManyRetries(String),
    Io(String),
}

#[derive(Debug, PartialEq)]
pub struct Done {
    pub total: u64,
    pub sha256: String,
    pub peak_conns: usize,
    pub transfer: Duration,
    pub ranged: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sidecar {
    pub v: u32,
    pub validator: String,
    pub total: u64,
    pub segs: Vec<[u64; 3]>, // start, end(excl), committed
}

pub fn sidecar_path(part: &Path) -> PathBuf { PathBuf::from(format!("{}.gdm", part.display())) }

/// Untrusted: size limit, ordering, bounds. Anything odd => None (restart from 0).
pub fn load_sidecar(part: &Path) -> Option<Sidecar> {
    let raw = std::fs::read(sidecar_path(part)).ok()?;
    if raw.len() > 64 * 1024 { return None; }
    let sc: Sidecar = serde_json::from_slice(&raw).ok()?;
    if sc.v != 1 || sc.segs.is_empty() { return None; }
    let mut prev = 0;
    for s in &sc.segs {
        let [a, e, d] = *s;
        if a != prev || e <= a || e > sc.total || d < a || d > e { return None; }
        prev = e;
    }
    (prev == sc.total).then_some(sc)
}

fn failpoint(name: &str, n: u64) {
    if let Ok(fp) = std::env::var("GDM_FAILPOINT") {
        if let Some((k, v)) = fp.split_once(':') {
            if k == name && v.parse::<u64>().ok() == Some(n) {
                eprintln!("failpoint {fp}");
                std::process::abort();
            }
        }
    }
}

struct Seg { start: u64, end: u64, pos: u64, committed: u64, worker: Option<usize>, gen: u64, tries: u32, fail_pos: u64 }

enum Ctl { Shrink { gen: u64, end: u64 } }
enum Ev {
    Progress { wid: usize, pos: u64 },
    Ack { wid: usize, gen: u64, end: u64 },
    Nack { wid: usize },
    Exit { wid: usize, pos: u64, err: Option<WErr> },
}
#[derive(Debug, Clone)]
enum WErr { Fatal(Fail), NoRange, Retry(String), RetryAfter(u64), Backoff(u16) }

struct Shared { client: Client, url: String, validator: Option<String>, total: Option<u64>, file: Arc<File>, cfg: Config, rx_bytes: std::sync::atomic::AtomicU64 }

pub async fn download(url: &str, part: &Path, cfg: Config) -> Result<Done, Fail> {
    let t0 = Instant::now();
    let client = Client::builder()
        .read_timeout(cfg.read_timeout)
        .connect_timeout(Duration::from_secs(10))
        .pool_max_idle_per_host(MAX_CONNS)
        .build()
        .map_err(|e| Fail::Io(e.to_string()))?;
    let resume = load_sidecar(part);
    // probe: Range from the first missing byte, identity encoding, If-Range when resuming
    let first = resume.as_ref().and_then(|s| s.segs.iter().find(|x| x[2] < x[1]).map(|x| x[2])).unwrap_or(0);
    let mut tries = 0;
    let probe = loop {
        let mut rq = client.get(url).header("Range", format!("bytes={first}-")).header("Accept-Encoding", "identity");
        for (k, v) in &cfg.headers { rq = rq.header(k, v); }
        if let Some(s) = &resume { rq = rq.header("If-Range", &s.validator); }
        let r = rq.send().await.map_err(net_err)?;
        match classify(&r) {
            Ok(()) => break r,
            Err(WErr::Fatal(f)) => return Err(f),
            Err(e) if tries < cfg.retries => {
                tries += 1;
                let w = match e { WErr::RetryAfter(s) => s.min(60), _ => 1u64 << tries.min(6) };
                tokio::time::sleep(Duration::from_secs(w)).await;
            }
            Err(e) => return Err(Fail::TooManyRetries(format!("{e:?}"))),
        }
    };
    let ct = probe.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("").to_ascii_lowercase();
    if ct.starts_with("multipart/byteranges") { return Err(Fail::Multipart); }
    if ct.starts_with("text/html") { return Err(Fail::Html); }
    let etag = strong_etag(probe.headers());
    let validator = etag.clone().or_else(|| hv(probe.headers(), "last-modified"));
    let (ranged, total) = if probe.status() == StatusCode::PARTIAL_CONTENT {
        let (a, _, t) = content_range(probe.headers()).ok_or(Fail::BadContentRange)?;
        if a != first { return Err(Fail::BadContentRange); }
        (true, Some(t.ok_or(Fail::BadContentRange)?))
    } else {
        if resume.is_some() && validator.is_some() && validator != resume.as_ref().map(|s| s.validator.clone()) {
            return Err(Fail::FileChanged); // 200 under If-Range
        }
        (false, probe.content_length())
    };
    let mut segs: Vec<Seg> = match (&resume, ranged) {
        (Some(s), true) if Some(s.total) == total && Some(&s.validator) == validator.as_ref() =>
            s.segs.iter().map(|x| Seg { start: x[0], end: x[1], pos: x[2], committed: x[2], worker: None, gen: 0, tries: 0, fail_pos: 0 }).collect(),
        (Some(_), true) => return Err(Fail::FileChanged),
        _ => vec![Seg { start: 0, end: total.unwrap_or(cfg.unknown_size_cap), pos: 0, committed: 0, worker: None, gen: 0, tries: 0, fail_pos: 0 }],
    };
    let file = OpenOptions::new().read(true).write(true).create(true).truncate(resume.is_none() || !ranged).open(part).map_err(io)?;
    if let Some(t) = total { if file.metadata().map_err(io)?.len() != t { file.set_len(t).map_err(io)?; } } // preallocate
    let sh = Arc::new(Shared { client, url: url.into(), validator: validator.clone(), total, file: Arc::new(file), cfg: cfg.clone(), rx_bytes: Default::default() });
    let (ev_tx, mut ev_rx) = mpsc::unbounded_channel::<Ev>();
    let mut ctl: HashMap<usize, mpsc::UnboundedSender<Ctl>> = HashMap::new();
    let mut tasks: Vec<tokio::task::JoinHandle<()>> = vec![];
    let mut next_wid = 0usize;
    let mut ranged = ranged;
    let mut target = if !ranged { 1 } else { cfg.fixed.unwrap_or(2).min(MAX_CONNS) };
    let mut peak = 0usize;
    let mut pending_split: Option<(usize, u64, u64)> = None; // (seg idx, gen, mid)
    let (mut ramp_base, mut ramp_stopped, mut ramp_t, mut ramp_bytes) = (0.0f64, cfg.fixed.is_some() || !ranged, Instant::now(), 0u64);
    let (mut last_flush, mut commits) = (Instant::now(), 0u64);
    let mut probe = Some(probe);

    macro_rules! spawn_worker {
        ($i:expr, $resp:expr) => {{
            let wid = next_wid; next_wid += 1;
            let (tx, rx) = mpsc::unbounded_channel();
            ctl.insert(wid, tx);
            let s = &mut segs[$i];
            s.worker = Some(wid);
            s.gen += 1;
            let (a, e, g) = (s.pos, s.end, s.gen);
            tasks.push(tokio::spawn(worker(wid, g, a, e, sh.clone(), $resp, rx, ev_tx.clone(), ranged)));
        }};
    }

    // initial leases: probe connection keeps the first missing segment; the second starts at half
    if ranged && segs.len() == 1 && target >= 2 {
        let (a, e) = (segs[0].pos, segs[0].end);
        let mid = a + (e - a) / 2;
        if e - mid >= cfg.min_split { segs[0].end = mid; segs.push(Seg { start: mid, end: e, pos: mid, committed: mid, worker: None, gen: 0, tries: 0, fail_pos: 0 }); }
    }
    if let Some(i) = segs.iter().position(|s| s.pos < s.end) { spawn_worker!(i, probe.take()); }

    let mut tick = tokio::time::interval(Duration::from_millis(100));
    loop {
        // fill idle slots: unassigned segments first, then split the largest remaining half
        let active = segs.iter().filter(|s| s.worker.is_some()).count();
        if active < target {
            if let Some(i) = segs.iter().position(|s| s.worker.is_none() && s.pos < s.end) {
                spawn_worker!(i, None);
                continue;
            }
            if ranged && pending_split.is_none() {
                if let Some((i, s)) = segs.iter().enumerate().filter(|(_, s)| s.worker.is_some()).max_by_key(|(_, s)| s.end - s.pos) {
                    let mid = s.pos + (s.end - s.pos) / 2;
                    if s.end - mid >= cfg.min_split {
                        let w = s.worker.unwrap();
                        let g = s.gen + 1;
                        let _ = ctl[&w].send(Ctl::Shrink { gen: g, end: mid });
                        pending_split = Some((i, g, mid));
                    }
                }
            }
        }
        peak = peak.max(segs.iter().filter(|s| s.worker.is_some()).count());
        if segs.iter().all(|s| s.pos >= s.end && s.worker.is_none()) { break; }
        tokio::select! {
            ev = ev_rx.recv() => match ev.unwrap() {
                Ev::Progress { wid, pos } => {
                    if let Some(s) = segs.iter_mut().find(|s| s.worker == Some(wid)) { s.pos = pos; }
                }
                Ev::Ack { wid, gen, end } => {
                    if let Some((i, g, mid)) = pending_split.take() {
                        if segs[i].worker == Some(wid) && g == gen && end == mid {
                            let old_end = segs[i].end;
                            segs[i].end = mid;
                            segs[i].gen = gen;
                            segs.insert(i + 1, Seg { start: mid, end: old_end, pos: mid, committed: mid, worker: None, gen: 0, tries: 0, fail_pos: 0 });
                        }
                    }
                }
                Ev::Nack { .. } => { pending_split = None; }
                Ev::Exit { wid, pos, err } => {
                    ctl.remove(&wid);
                    let Some(i) = segs.iter().position(|s| s.worker == Some(wid)) else { continue };
                    if pos > segs[i].pos { segs[i].pos = pos; }
                    segs[i].worker = None;
                    if let Some((pi, _, _)) = pending_split { if pi == i { pending_split = None; } }
                    if !ranged && total.is_none() && err.is_none() { segs[i].end = segs[i].pos; }
                    match err {
                        None => {}
                        Some(WErr::Fatal(f)) => { tasks.iter().for_each(|t| t.abort()); return Err(f); }
                        Some(WErr::NoRange) => {
                            // E-01: stop every worker, mark host no-range, restart one sequential stream from 0
                            tasks.iter().for_each(|t| t.abort());
                            ctl.clear();
                            pending_split = None;
                            ranged = false;
                            ramp_stopped = true;
                            target = 1;
                            segs = vec![Seg { start: 0, end: total.unwrap_or(cfg.unknown_size_cap), pos: 0, committed: 0, worker: None, gen: 0, tries: 0, fail_pos: 0 }];
                        }
                        Some(e) => {
                            // a retry budget is per stall: only >= 1 MiB of progress since the last failure refills it
                            if segs[i].pos >= segs[i].fail_pos + MIB { segs[i].tries = 0; }
                            segs[i].fail_pos = segs[i].pos;
                            segs[i].tries += 1;
                            if segs[i].tries > cfg.retries { tasks.iter().for_each(|t| t.abort()); return Err(match e { WErr::Retry(m) if m.contains("timed out") || m.contains("timeout") => Fail::Timeout, e => Fail::TooManyRetries(format!("{e:?}")) }); }
                            let wait = match e { WErr::RetryAfter(s) => { target = (target / 2).max(1); s.min(60) } WErr::Backoff(_) => 1u64 << segs[i].tries.min(6), _ => 0 };
                            if !ranged { segs[i].pos = 0; } // no resume without ranges
                            if wait > 0 { tokio::time::sleep(Duration::from_secs(wait)).await; }
                        }
                    }
                }
            },
            _ = tick.tick() => {
                // ramp: +1 every 2 s while total speed rose >= 10 % since the last add
                // measured on received bytes (not 1 MiB-quantised writes); the clock starts at the first byte
                let rx = sh.rx_bytes.load(std::sync::atomic::Ordering::Relaxed);
                if ramp_bytes == 0 && rx == 0 { ramp_t = Instant::now(); }
                if !ramp_stopped && ramp_t.elapsed() >= Duration::from_secs(2) {
                    let speed = (rx - ramp_bytes) as f64 / ramp_t.elapsed().as_secs_f64();
                    if ramp_base == 0.0 || speed >= ramp_base * 1.10 {
                        if target < cfg.cap.min(MAX_CONNS) { target += 1; } else { ramp_stopped = true; }
                        ramp_base = speed;
                    } else { ramp_stopped = true; }
                    ramp_t = Instant::now();
                    ramp_bytes = rx;
                }
                // commit: sync data first, then write the offsets that were on disk before the sync
                let unflushed: u64 = segs.iter().map(|s| s.pos - s.committed).sum();
                if ranged && (unflushed >= cfg.flush_bytes || (unflushed > 0 && last_flush.elapsed() >= cfg.flush_every)) {
                    let snap: Vec<u64> = segs.iter().map(|s| s.pos).collect();
                    commits += 1;
                    let f = sh.file.clone();
                    tokio::task::spawn_blocking(move || f.sync_data()).await.unwrap().map_err(io)?;
                    failpoint("after_sync", commits);
                    write_sidecar(part, &Sidecar { v: 1, validator: validator.clone().unwrap_or_default(), total: total.unwrap_or(0), segs: segs.iter().zip(&snap).map(|(s, p)| [s.start, s.end, *p]).collect() }, commits).map_err(io)?;
                    for (s, p) in segs.iter_mut().zip(snap) { s.committed = p; }
                    failpoint("after_commit", commits);
                    last_flush = Instant::now();
                }
            }
        }
    }
    let transfer = t0.elapsed();
    let f = sh.file.clone();
    tokio::task::spawn_blocking(move || f.sync_data()).await.unwrap().map_err(io)?;
    let got: u64 = segs.iter().map(|s| s.end - s.start).sum();
    let len = sh.file.metadata().map_err(io)?.len();
    if total.is_some_and(|t| t != got || t != len) { return Err(Fail::Io(format!("byte count {got}/{len} != total {total:?}"))); }
    if total.is_none() { sh.file.set_len(got).map_err(io)?; }
    let f2 = sh.file.clone();
    let sha256 = tokio::task::spawn_blocking(move || sha_handle(&f2)).await.unwrap().map_err(io)?;
    Ok(Done { total: got, sha256, peak_conns: peak, transfer, ranged })
}

fn write_sidecar(part: &Path, sc: &Sidecar, n: u64) -> std::io::Result<()> {
    let p = sidecar_path(part);
    let tmp = PathBuf::from(format!("{}.tmp", p.display()));
    let f = File::create(&tmp)?;
    serde_json::to_writer(&f, sc)?;
    f.sync_all()?;
    drop(f);
    failpoint("after_sidecar_tmp", n);
    std::fs::rename(&tmp, &p) // MoveFileEx(REPLACE_EXISTING): old or new, never half
}

/// hash through the already-open handle (no path re-resolution between check and use)
pub fn sha_handle(f: &File) -> std::io::Result<String> {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 4 << 20];
    let mut off = 0u64;
    loop {
        let n = f.seek_read(&mut buf, off)?;
        if n == 0 { break; }
        h.update(&buf[..n]);
        off += n as u64;
    }
    Ok(crate::fixture::hex(&h.finalize()))
}

#[allow(clippy::too_many_arguments)]
async fn worker(wid: usize, mut gen: u64, start: u64, mut end: u64, sh: Arc<Shared>, pre: Option<Response>, mut ctl: mpsc::UnboundedReceiver<Ctl>, ev: mpsc::UnboundedSender<Ev>, ranged: bool) {
    let mut pos = start;
    let r = run_worker(wid, &mut gen, &mut pos, &mut end, &sh, pre, &mut ctl, &ev, ranged).await;
    let _ = ev.send(Ev::Exit { wid, pos, err: r.err() });
}

#[allow(clippy::too_many_arguments)]
async fn run_worker(wid: usize, gen: &mut u64, pos: &mut u64, end: &mut u64, sh: &Shared, pre: Option<Response>, ctl: &mut mpsc::UnboundedReceiver<Ctl>, ev: &mpsc::UnboundedSender<Ev>, ranged: bool) -> Result<(), WErr> {
    let start = *pos;
    let (resp, req_end) = match pre {
        Some(r) => (r, sh.total.unwrap_or(u64::MAX)),
        None => {
            let mut rq = sh.client.get(&sh.url).header("Accept-Encoding", "identity");
            if ranged {
                rq = rq.header("Range", format!("bytes={}-{}", start, *end - 1));
                if let Some(v) = &sh.validator { rq = rq.header("If-Range", v); }
            }
            for (k, v) in &sh.cfg.headers { rq = rq.header(k, v); }
            let r = rq.send().await.map_err(|e| WErr::Retry(e.to_string()))?;
            classify(&r)?;
            if ranged {
                if r.status() == StatusCode::OK {
                    // 200 to a range request: changed file (validator differs) or server ignores Range
                    let et = strong_etag(r.headers()).or_else(|| hv(r.headers(), "last-modified"));
                    return Err(if et.is_some() && et != sh.validator { WErr::Fatal(Fail::FileChanged) } else { WErr::NoRange });
                }
                let (a, b, t) = content_range(r.headers()).ok_or(WErr::Fatal(Fail::BadContentRange))?;
                if a != start || b.is_some_and(|b| b + 1 > *end) || t != sh.total { return Err(WErr::Fatal(Fail::BadContentRange)); }
                if strong_etag(r.headers()).is_some_and(|e| Some(e) != sh.validator) { return Err(WErr::Fatal(Fail::FileChanged)); }
            }
            let ct = r.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("");
            if ct.starts_with("multipart/") { return Err(WErr::Fatal(Fail::Multipart)); }
            (r, if ranged { *end } else { sh.total.unwrap_or(u64::MAX) })
        }
    };
    let mut body = resp.bytes_stream();
    let mut buf: Vec<u8> = Vec::with_capacity(sh.cfg.buf);
    let mut n_writes = 0u64;
    let (mut win_t, mut win_b) = (tokio::time::Instant::now(), 0u64);
    let write = |buf: &mut Vec<u8>, pos: &mut u64, n_writes: &mut u64| -> Result<(), WErr> {
        if buf.is_empty() { return Ok(()); }
        // ponytail: synchronous positional write inside the async task (1 MiB to the page cache); move to a writer thread if profiling shows stalls
        sh.file.seek_write(buf, *pos).map_err(|e| WErr::Fatal(Fail::Io(e.to_string())))?;
        *n_writes += 1;
        failpoint("mid_write", *n_writes); // bytes written, progress not yet reported
        *pos += buf.len() as u64;
        buf.clear();
        ev.send(Ev::Progress { wid, pos: *pos }).ok();
        Ok(())
    };
    loop {
        let cur = *pos + buf.len() as u64;
        if cur >= *end {
            write(&mut buf, pos, &mut n_writes)?;
            return if sh.total.is_none() { Err(WErr::Fatal(Fail::SizeExceeded)) } else { Ok(()) };
        }
        let (min_bps, win) = sh.cfg.min_speed;
        tokio::select! {
            biased;
            _ = tokio::time::sleep_until(win_t + win) => {
                if win_b < min_bps * win.as_secs() { write(&mut buf, pos, &mut n_writes)?; return Err(WErr::Retry("timed out: below minimum speed".into())); }
                win_t = tokio::time::Instant::now();
                win_b = 0;
            }
            c = ctl.recv() => match c {
                Some(Ctl::Shrink { gen: g, end: e }) => {
                    // boundary ack: everything already written lies below e, and every later write is clamped to e
                    if cur >= e { ev.send(Ev::Nack { wid }).ok(); } else { *end = e; *gen = g; ev.send(Ev::Ack { wid, gen: g, end: e }).ok(); }
                }
                None => return Ok(()),
            },
            chunk = body.next() => match chunk {
                None => {
                    write(&mut buf, pos, &mut n_writes)?;
                    if sh.total.is_some() && *pos < *end { return Err(WErr::Retry("short body".into())); }
                    if sh.total.is_none() { *end = *pos; }
                    return Ok(());
                }
                Some(Err(e)) => { write(&mut buf, pos, &mut n_writes)?; return Err(WErr::Retry(format!("{e} {:?}", std::error::Error::source(&e)))); }
                Some(Ok(b)) => {
                    win_b += b.len() as u64;
                    sh.rx_bytes.fetch_add(b.len() as u64, std::sync::atomic::Ordering::Relaxed);
                    let cur = *pos + buf.len() as u64;
                    if cur + b.len() as u64 > req_end { // server sent more than it promised / we asked for
                        let keep = (req_end.saturating_sub(cur)) as usize;
                        buf.extend_from_slice(&b[..keep.min(b.len())]);
                        let clamp = (*end - *pos) as usize;
                        buf.truncate(clamp);
                        write(&mut buf, pos, &mut n_writes)?;
                        if *pos >= *end { return Ok(()); }
                        return Err(WErr::Fatal(if sh.total.is_none() { Fail::SizeExceeded } else { Fail::LongBody }));
                    }
                    let room = (*end - cur) as usize;
                    buf.extend_from_slice(&b[..room.min(b.len())]);
                    if buf.len() >= sh.cfg.buf || *pos + buf.len() as u64 >= *end { write(&mut buf, pos, &mut n_writes)?; }
                }
            }
        }
    }
}

fn classify(r: &Response) -> Result<(), WErr> {
    let s = r.status().as_u16();
    match s {
        200 | 206 => Ok(()),
        429 | 503 => Err(WErr::RetryAfter(hv(r.headers(), "retry-after").and_then(|v| v.parse().ok()).unwrap_or(5))),
        401 | 403 => Err(WErr::Fatal(Fail::Auth)),
        404 | 410 => Err(WErr::Fatal(Fail::Gone)),
        416 => Err(WErr::Fatal(Fail::SizeChanged)),
        500..=599 => Err(WErr::Backoff(s)),
        _ => Err(WErr::Fatal(Fail::Http(s))),
    }
}

fn hv(h: &HeaderMap, k: &str) -> Option<String> { h.get(k).and_then(|v: &HeaderValue| v.to_str().ok()).map(String::from) }
fn strong_etag(h: &HeaderMap) -> Option<String> { hv(h, "etag").filter(|e| !e.starts_with("W/")) }

/// "bytes a-b/total" -> (a, Some(b), Some(total)); total "*" -> None
fn content_range(h: &HeaderMap) -> Option<(u64, Option<u64>, Option<u64>)> {
    let v = hv(h, "content-range")?;
    let r = v.strip_prefix("bytes ")?;
    let (ab, t) = r.split_once('/')?;
    let (a, b) = ab.split_once('-')?;
    let (a, b): (u64, u64) = (a.parse().ok()?, b.parse().ok()?);
    if b < a { return None; }
    Some((a, Some(b), if t == "*" { None } else { Some(t.parse().ok()?) }))
}

fn net_err(e: reqwest::Error) -> Fail { if e.is_timeout() { Fail::Timeout } else { Fail::Io(e.to_string()) } }
fn io(e: std::io::Error) -> Fail { Fail::Io(e.to_string()) }
