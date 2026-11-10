// SPDX-License-Identifier: RPL-1.5
//! Dynamic segmentation (§09). One actor owns the segment map. Network workers hold write leases with a
//! generation; a split shrinks a lease and waits for the worker's boundary ack. Workers never touch the disk:
//! they hand chunks to ONE writer thread through a bounded queue, and the writer reports what it has written.
//! A checkpoint snapshots the written positions, runs FlushFileBuffers, and only then persists them, so every
//! persisted offset is <= flushed data (E-04). A restart from 0 (E-01) starts a new epoch: the old workers are
//! joined first and the writer drops every queued chunk of an older epoch.
use crate::http::{self, Step};
use crate::limiter::JobLimit;
use crate::net::{self, Creds};
use crate::{failpoint, Fail};
use futures_util::StreamExt;
use reqwest::{Client, Response, StatusCode};
use std::collections::HashMap;
use std::fs::File;
use std::os::windows::fs::FileExt;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use url::Url;

pub const MIB: u64 = 1 << 20;
pub const MAX_CONNS: usize = 32;

/// connections (workers) alive in this process, and the peak (L-02 evidence; never used for decisions)
pub static LIVE_CONNS: AtomicUsize = AtomicUsize::new(0);
pub static PEAK_CONNS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug)]
pub struct Config {
    /// connection cap for one job: default 8, max 32 (per-host profile)
    pub cap: usize,
    /// benchmark config (b): fixed connection count, no ramp
    pub fixed: Option<usize>,
    pub min_split: u64,
    /// worker buffer = writer chunk size
    pub buf: usize,
    /// writer queue depth in chunks (bounded memory: depth * buf)
    pub queue: usize,
    pub flush_bytes: u64,
    pub flush_every: Duration,
    pub read_timeout: Duration,
    /// slow-connection watchdog (E-07): fewer than .0 bytes/s over .1 => reconnect
    pub min_speed: (u64, Duration),
    pub retries: u32,
    /// unknown length: stop here (E-03); None = min(20 GB, 90 % of free space minus the 1 GB reserve)
    pub unknown_cap: Option<u64>,
    /// connection slots shared with other jobs (L-02 total cap, per-server limit, F11): every connection after
    /// the first holds one permit of each; the first always runs so a job never waits forever
    pub slots: Vec<Arc<tokio::sync::Semaphore>>,
    /// a 429/503 halved the connection target: the new target is written here so later jobs to the same host
    /// start at it (F11 "learn from 429")
    pub learned: Option<Arc<AtomicUsize>>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            cap: 8,
            fixed: None,
            min_split: MIB,
            buf: MIB as usize,
            queue: 16,
            flush_bytes: 8 * MIB,
            flush_every: Duration::from_secs(2),
            read_timeout: Duration::from_secs(60),
            min_speed: (1024, Duration::from_secs(120)),
            retries: 10,
            unknown_cap: None,
            slots: vec![],
            learned: None,
        }
    }
}

pub const RESERVE: u64 = 1 << 30;

/// what was durable at the last checkpoint (persisted by the caller: sidecar + SQLite)
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub epoch: u64,
    pub final_url: String,
    pub validator: Option<String>,
    pub total: Option<u64>,
    /// [start, end exclusive, committed]
    pub segs: Vec<[u64; 3]>,
}

/// live view of a running job for the UI (polled; never persisted). `want_conns` 0 = automatic ramp.
#[derive(Default, Debug)]
pub struct Live {
    /// bytes the writer has written (not yet necessarily flushed)
    pub written: AtomicU64,
    /// 0 = unknown length
    pub total: AtomicU64,
    pub conns: AtomicUsize,
    pub want_conns: AtomicUsize,
    /// [start, end exclusive, written] per segment
    pub segs: std::sync::Mutex<Vec<[u64; 3]>>,
}

pub type Checkpoint<'a> = &'a mut (dyn FnMut(&Snapshot) -> Result<(), Fail> + Send);

pub struct Request<'a> {
    pub url: Url,
    pub creds: Creds,
    pub policy: Arc<net::Policy>,
    /// the .part handle (opened by the caller relative to the held directory)
    pub file: Arc<File>,
    /// free bytes on the destination volume (for E-03 checks)
    pub free: u64,
    pub resume: Option<Snapshot>,
    pub limit: Option<Arc<JobLimit>>,
    pub checkpoint: Checkpoint<'a>,
    pub live: Option<Arc<Live>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Done {
    pub total: u64,
    pub final_url: String,
    pub validator: Option<String>,
    pub ranged: bool,
    pub peak_conns: usize,
    pub transfer: Duration,
    pub flushes: u64,
    pub flush_time: Duration,
}

struct Seg {
    start: u64,
    end: u64,
    next: u64,
    written: u64,
    committed: u64,
    worker: Option<usize>,
    gen: u64,
    tries: u32,
    fail_pos: u64,
}
impl Seg {
    fn new(start: u64, end: u64, done: u64) -> Seg {
        Seg { start, end, next: done, written: done, committed: done, worker: None, gen: 0, tries: 0, fail_pos: done }
    }
}

enum Ctl {
    Shrink { gen: u64, end: u64 },
}
enum Ev {
    Admitted { wid: usize, next: u64 },
    Ack { wid: usize, gen: u64, end: u64 },
    Nack,
    Exit { wid: usize, next: u64, err: Option<WErr> },
    Written { epoch: u64, seg: u64, upto: u64 },
    WriteFail(String),
}
#[derive(Debug, Clone)]
enum WErr {
    Fatal(Fail),
    NoRange,
    Retry(String),
    RetryAfter(u64),
    Backoff,
}

struct Chunk {
    epoch: u64,
    seg: u64,
    off: u64,
    data: Vec<u8>,
}

struct Shared {
    client: Client,
    url: Url,
    creds: Creds,
    validator: Option<String>,
    total: Option<u64>,
    cfg: Config,
    rx_bytes: AtomicU64,
    wtx: mpsc::Sender<Chunk>,
    limit: Option<Arc<JobLimit>>,
}

fn net_fail(e: &reqwest::Error) -> Fail {
    if let Some(f) = net::forbidden(e) {
        return Fail::Forbidden(f);
    }
    if let Some(t) = net::tls_error(e) {
        return Fail::Tls(t);
    }
    if e.is_timeout() {
        return Fail::Timeout;
    }
    Fail::Io(format!("{e} ({:?})", std::error::Error::source(e).map(|s| s.to_string())))
}

fn get(sh: &Shared, url: &Url, range: String, if_range: bool) -> reqwest::RequestBuilder {
    let mut rq = sh.client.get(url.clone()).header("Range", range).header("Accept-Encoding", "identity");
    if if_range {
        if let Some(v) = &sh.validator {
            rq = rq.header("If-Range", v);
        }
    }
    for (k, v) in sh.creds.for_url(url) {
        rq = rq.header(k, v);
    }
    rq
}

/// the writer thread: the only code that writes the .part
fn writer(file: Arc<File>, epoch: Arc<AtomicU64>, mut rx: mpsc::Receiver<Chunk>, ev: mpsc::UnboundedSender<Ev>) {
    while let Some(c) = rx.blocking_recv() {
        if c.epoch != epoch.load(Ordering::SeqCst) {
            continue;
        } // fenced: a chunk of a restarted download
        let mut done = 0;
        while done < c.data.len() {
            match file.seek_write(&c.data[done..], c.off + done as u64) {
                Ok(0) => {
                    let _ = ev.send(Ev::WriteFail("write returned 0".into()));
                    return;
                }
                Ok(k) => done += k,
                Err(e) => {
                    let _ = ev.send(Ev::WriteFail(e.to_string()));
                    return;
                }
            }
        }
        failpoint::hit("mid_write"); // bytes on disk (in cache), not yet reported
        let _ = ev.send(Ev::Written { epoch: c.epoch, seg: c.seg, upto: c.off + c.data.len() as u64 });
    }
}

pub async fn download(rq: Request<'_>, cfg: Config) -> Result<Done, Fail> {
    // L-02: the first connection (the probe, then the first lease) waits for a slot in every shared limit, so
    // the total cap holds even when other jobs already use every slot
    let mut first_permits = Vec::with_capacity(cfg.slots.len());
    for s in &cfg.slots {
        first_permits.push(s.clone().acquire_owned().await.map_err(|e| Fail::Io(e.to_string()))?);
    }
    let t0 = Instant::now();
    let client = net::client(rq.policy.clone(), true, cfg.read_timeout).map_err(Fail::Io)?;
    rq.policy.check_url(&rq.url).map_err(|e| Fail::Forbidden(e.0))?;
    let resume = rq.resume.clone();
    let mut epoch = resume.as_ref().map_or(0, |r| r.epoch);
    // probe: Range from the first missing byte, identity encoding, If-Range when resuming with a validator
    let first = resume
        .as_ref()
        .and_then(|s| s.segs.iter().find(|x| x[2] < x[1]).map(|x| x[2]))
        .unwrap_or_else(|| resume.as_ref().map_or(0, |s| s.total.unwrap_or(1).saturating_sub(1)));
    let probe_url = resume.as_ref().and_then(|r| Url::parse(&r.final_url).ok()).unwrap_or_else(|| rq.url.clone());
    let mut tries = 0;
    let probe = loop {
        let mut b = client.get(probe_url.clone()).header("Range", format!("bytes={first}-")).header("Accept-Encoding", "identity");
        if let Some(v) = resume.as_ref().and_then(|r| r.validator.as_ref()) {
            b = b.header("If-Range", v);
        }
        for (k, v) in rq.creds.for_url(&probe_url) {
            b = b.header(k, v);
        }
        // the whole response head must arrive within read_timeout (slowloris headers, E-07)
        let sent = tokio::time::timeout(cfg.read_timeout, async { b.send().await.map_err(|e| net_fail(&e)) }).await.unwrap_or(Err(Fail::Timeout));
        let r = match sent {
            Ok(r) => r,
            Err(f) => {
                if matches!(f, Fail::Forbidden(_) | Fail::Tls(_)) || tries >= cfg.retries {
                    return Err(f);
                }
                tries += 1;
                tokio::time::sleep(Duration::from_secs(1 << tries.min(6))).await;
                continue;
            }
        };
        match http::status(r.status().as_u16(), r.headers()) {
            Step::Accept => break r,
            Step::Fatal(f) => return Err(f),
            e if tries < cfg.retries => {
                tries += 1;
                let w = match e {
                    Step::RetryAfter(s) => {
                        // throttled before the first byte: later jobs to this host start at half the cap
                        if let Some(l) = &cfg.learned {
                            l.store((cfg.cap / 2).max(1), Ordering::Relaxed);
                        }
                        s
                    }
                    _ => 1u64 << tries.min(6),
                };
                tokio::time::sleep(Duration::from_secs(w)).await;
            }
            e => return Err(Fail::TooManyRetries(format!("{e:?}"))),
        }
    };
    let final_url = probe.url().clone();
    let ct = http::content_type(probe.headers());
    if ct.starts_with("multipart/byteranges") {
        return Err(Fail::Multipart);
    }
    if ct.starts_with("text/html") {
        return Err(Fail::Html);
    }
    let validator = http::validator(probe.headers());
    let resumed_validator = resume.as_ref().and_then(|r| r.validator.clone());
    let identity = http::identity(probe.headers());
    let (mut ranged, total) = if probe.status() == StatusCode::PARTIAL_CONTENT && identity {
        let (a, _, t) = http::content_range(probe.headers()).ok_or(Fail::BadContentRange)?;
        if a != first {
            return Err(Fail::BadContentRange);
        }
        (true, Some(t.ok_or(Fail::BadContentRange)?))
    } else if probe.status() == StatusCode::PARTIAL_CONTENT {
        return Err(Fail::BadContentRange); // compressed range: offsets would not be file offsets (E-09)
    } else {
        if resume.is_some() && validator.is_some() && resumed_validator.is_some() && validator != resumed_validator {
            return Err(Fail::FileChanged); // 200 under If-Range with another validator
        }
        (false, probe.content_length().filter(|_| identity))
    };
    if let (Some(r), true) = (&resume, ranged) {
        if r.total != total || (r.validator.is_some() && validator != r.validator) {
            return Err(Fail::FileChanged);
        }
    }
    // E-03: space check before preallocation (sparse reserves nothing, so this is the only up-front check)
    let unknown_cap = cfg.unknown_cap.unwrap_or_else(|| (20u64 << 30).min(rq.free.saturating_sub(RESERVE) / 10 * 9));
    let need = total.map(|t| t - resume.as_ref().filter(|_| ranged).map_or(0, |r| r.segs.iter().map(|s| s[2] - s[0]).sum::<u64>()));
    if need.is_some_and(|n| n.saturating_add(RESERVE) > rq.free) {
        return Err(Fail::DiskFull);
    }
    let file = rq.file.clone();
    let fresh = !ranged || resume.is_none();
    if fresh {
        epoch += u64::from(resume.is_some()); // a 200 on resume: new epoch, nothing old may land
    }
    let mut segs: Vec<Seg> = match (&resume, ranged) {
        (Some(s), true) => s.segs.iter().map(|x| Seg::new(x[0], x[1], x[2])).collect(),
        _ => vec![Seg::new(0, total.unwrap_or(unknown_cap), 0)],
    };
    let snap = |segs: &[Seg], pos: &dyn Fn(&Seg) -> u64, epoch: u64| Snapshot {
        epoch,
        final_url: final_url.to_string(),
        validator: validator.clone(),
        total,
        segs: segs.iter().map(|s| [s.start, s.end, pos(s)]).collect(),
    };
    if fresh {
        // the zero-progress checkpoint is persisted BEFORE the file is truncated: a crash in between can never
        // leave old offsets pointing at zeroed bytes (council G5)
        (rq.checkpoint)(&snap(&segs, &|s| s.committed, epoch))?;
        failpoint::hit("after_reset_checkpoint");
        file.set_len(0).map_err(io)?;
        failpoint::hit("after_truncate");
    }
    if let Some(t) = total {
        if file.metadata().map_err(io)?.len() != t {
            // sparse first (NTFS): a write far past the valid data length otherwise makes NTFS zero-fill the gap
            crate::fsx::set_sparse(&file, true);
            file.set_len(t).map_err(io)?;
        }
    }
    let epoch_cell = Arc::new(AtomicU64::new(epoch));
    let (wtx, wrx) = mpsc::channel::<Chunk>(cfg.queue.max(1));
    let (ev_tx, mut ev_rx) = mpsc::unbounded_channel::<Ev>();
    let wthread = {
        let (f, e, t) = (file.clone(), epoch_cell.clone(), ev_tx.clone());
        std::thread::spawn(move || writer(f, e, wrx, t))
    };
    let sh = Arc::new(Shared {
        client,
        url: final_url.clone(),
        creds: rq.creds.clone(),
        validator: validator.clone(),
        total,
        cfg: cfg.clone(),
        rx_bytes: AtomicU64::new(0),
        wtx,
        limit: rq.limit.clone(),
    });
    // seam check (E-04): a persisted offset is only a claim. For every resumed segment re-fetch the last
    // <= 64 KiB below its committed offset and compare with the disk; a mismatch drops that segment's progress.
    if ranged && resume.is_some() {
        let mut dropped = false;
        for s in segs.iter_mut().filter(|s| s.committed > s.start) {
            let a = s.committed - (s.committed - s.start).min(64 * 1024);
            if !seam_ok(&sh, &file, a, s.committed).await {
                *s = Seg::new(s.start, s.end, s.start);
                dropped = true;
            }
        }
        if dropped {
            (rq.checkpoint)(&snap(&segs, &|s| s.committed, epoch))?; // the refused claims are gone before rewriting
        }
    }

    let mut ctl: HashMap<usize, mpsc::UnboundedSender<Ctl>> = HashMap::new();
    let mut tasks = Tasks::default();
    let mut next_wid = 0usize;
    let mut target = if !ranged { 1 } else { cfg.fixed.unwrap_or(2).clamp(1, MAX_CONNS) };
    let mut peak = 0usize;
    let mut pending_split: Option<(u64, u64, u64)> = None; // (seg start, gen, mid)
    let (mut ramp_base, mut ramp_stopped, mut ramp_t, mut ramp_bytes) = (0.0f64, cfg.fixed.is_some() || !ranged, Instant::now(), 0u64);
    let (mut last_flush, mut flushes, mut flush_time) = (Instant::now(), 0u64, Duration::ZERO);
    let mut probe = Some(probe);

    macro_rules! spawn_worker {
        ($i:expr, $resp:expr, $permits:expr) => {{
            let permits: Vec<tokio::sync::OwnedSemaphorePermit> = $permits;
            let wid = next_wid;
            next_wid += 1;
            let (tx, rx) = mpsc::unbounded_channel();
            ctl.insert(wid, tx);
            let s = &mut segs[$i];
            s.worker = Some(wid);
            s.gen += 1;
            tasks.0.push(tokio::spawn(worker(
                permits,
                wid,
                s.gen,
                s.start,
                s.next,
                s.end,
                epoch_cell.load(Ordering::SeqCst),
                sh.clone(),
                $resp,
                rx,
                ev_tx.clone(),
                ranged,
            )));
        }};
    }
    macro_rules! fail {
        ($f:expr) => {{
            epoch_cell.store(u64::MAX, Ordering::SeqCst); // fence: the writer drops everything still queued
            return Err($f); // `tasks` aborts every worker on drop
        }};
    }

    // initial leases: the probe connection keeps the first missing segment; the second starts at half
    if ranged && segs.len() == 1 && target >= 2 {
        let (a, e) = (segs[0].next, segs[0].end);
        let mid = a + (e - a) / 2;
        if e - mid >= cfg.min_split && segs[0].start == a {
            segs[0].end = mid;
            segs.push(Seg::new(mid, e, mid));
        }
    }
    if let Some(i) = segs.iter().position(|s| s.next < s.end) {
        // the probe body starts at `first`: hand it over only if that is still where this lease starts
        let pre = if !ranged || segs[i].next == first { probe.take() } else { None };
        spawn_worker!(i, pre, std::mem::take(&mut first_permits));
    }
    drop(probe);

    let mut tick = tokio::time::interval(Duration::from_millis(100));
    loop {
        // fill idle slots: unassigned segments first, then split the largest remaining half
        let active = segs.iter().filter(|s| s.worker.is_some()).count();
        // a further connection needs a free slot in every shared limit (all or none)
        let permits = || -> Option<Vec<tokio::sync::OwnedSemaphorePermit>> { cfg.slots.iter().map(|s| s.clone().try_acquire_owned().ok()).collect() };
        let free = cfg.slots.iter().all(|s| s.available_permits() > 0) || (active == 0 && !first_permits.is_empty());
        if active < target && free {
            if let Some(i) = segs.iter().position(|s| s.worker.is_none() && s.next < s.end) {
                // the job's own first slots come back when its only connection restarts (retry after an error)
                let p = if active == 0 && !first_permits.is_empty() { std::mem::take(&mut first_permits) } else { permits().unwrap_or_default() };
                if p.len() == cfg.slots.len() {
                    spawn_worker!(i, None, p);
                    continue;
                }
            }
            if ranged && pending_split.is_none() {
                if let Some(s) = segs.iter().filter(|s| s.worker.is_some()).max_by_key(|s| s.end - s.next) {
                    let mid = s.next + (s.end - s.next) / 2;
                    if s.end - mid >= cfg.min_split {
                        let g = s.gen + 1;
                        let _ = ctl[&s.worker.unwrap()].send(Ctl::Shrink { gen: g, end: mid });
                        pending_split = Some((s.start, g, mid));
                        failpoint::hit("lease_revoke"); // shrink sent, worker still receiving, no ack yet
                    }
                }
            }
        }
        peak = peak.max(segs.iter().filter(|s| s.worker.is_some()).count());
        if segs.iter().all(|s| s.written >= s.end && s.worker.is_none()) {
            break;
        }
        tokio::select! {
            ev = ev_rx.recv() => match ev.expect("event channel") {
                Ev::Admitted { wid, next } => {
                    if let Some(s) = segs.iter_mut().find(|s| s.worker == Some(wid)) { s.next = s.next.max(next); }
                }
                Ev::Written { epoch: e, seg, upto } => {
                    if e == epoch_cell.load(Ordering::SeqCst) {
                        if let Some(s) = segs.iter_mut().find(|s| s.start == seg) { s.written = s.written.max(upto.min(s.end)); }
                    }
                }
                Ev::WriteFail(m) => fail!(if m.contains("112") || m.to_ascii_lowercase().contains("not enough space") { Fail::DiskFull } else { Fail::Io(format!("write: {m}")) }),
                Ev::Ack { wid, gen, end } => {
                    if let Some((st, g, mid)) = pending_split.take() {
                        if let Some(i) = segs.iter().position(|s| s.start == st && s.worker == Some(wid)) {
                            if g == gen && end == mid {
                                let old_end = segs[i].end;
                                segs[i].end = mid;
                                segs[i].gen = gen;
                                segs.insert(i + 1, Seg::new(mid, old_end, mid));
                                failpoint::hit("after_ack"); // split in memory, new half not yet leased or committed
                            }
                        }
                    }
                }
                Ev::Nack => { pending_split = None; }
                Ev::Exit { wid, next, err } => {
                    ctl.remove(&wid);
                    let Some(i) = segs.iter().position(|s| s.worker == Some(wid)) else { continue };
                    segs[i].next = segs[i].next.max(next);
                    segs[i].worker = None;
                    if pending_split.is_some_and(|(st, _, _)| st == segs[i].start) { pending_split = None; }
                    if !ranged && total.is_none() && err.is_none() { segs[i].end = segs[i].next; } // EOF of an unknown length
                    match err {
                        None => {}
                        Some(WErr::Fatal(f)) => fail!(f),
                        Some(WErr::NoRange) => {
                            // E-01: stop every worker, mark no-range, restart one stream from 0 in a new epoch
                            tasks.0.iter().for_each(|t| t.abort());
                            for t in tasks.0.drain(..) { let _ = t.await; } // joined: no old worker can enqueue now
                            ctl.clear();
                            pending_split = None;
                            ranged = false;
                            ramp_stopped = true;
                            target = 1;
                            let e = epoch_cell.fetch_add(1, Ordering::SeqCst) + 1; // writer drops older chunks
                            segs = vec![Seg::new(0, total.unwrap_or(unknown_cap), 0)];
                            if let Err(err) = (rq.checkpoint)(&snap(&segs, &|s| s.committed, e)) { fail!(err); }
                        }
                        Some(e) => {
                            // a retry budget is per stall: >= 1 MiB of progress since the last failure refills it
                            if segs[i].next >= segs[i].fail_pos + MIB { segs[i].tries = 0; }
                            segs[i].fail_pos = segs[i].next;
                            segs[i].tries += 1;
                            if segs[i].tries > cfg.retries {
                                fail!(match e { WErr::Retry(m) if m.contains("timed out") || m.contains("timeout") => Fail::Timeout, e => Fail::TooManyRetries(format!("{e:?}")) });
                            }
                            let wait = match e { WErr::RetryAfter(s) => { target = (target / 2).max(1); if let Some(l) = &cfg.learned { l.store(target, Ordering::Relaxed); } s } WErr::Backoff => 1u64 << segs[i].tries.min(6), _ => 0 };
                            if !ranged {
                                // no resume without ranges: start over in a new epoch
                                let e2 = epoch_cell.fetch_add(1, Ordering::SeqCst) + 1;
                                let tries = segs[i].tries;
                                segs[i] = Seg::new(0, total.unwrap_or(unknown_cap), 0);
                                segs[i].tries = tries; // the retry budget survives the restart
                                if let Err(err) = (rq.checkpoint)(&snap(&segs, &|s| s.committed, e2)) { fail!(err); }
                            }
                            if wait > 0 { tokio::time::sleep(Duration::from_secs(wait)).await; }
                        }
                    }
                }
            },
            _ = tick.tick() => {
                if let Some(l) = &rq.live {
                    l.written.store(segs.iter().map(|s| s.written - s.start).sum(), Ordering::Relaxed);
                    l.total.store(total.unwrap_or(0), Ordering::Relaxed);
                    l.conns.store(segs.iter().filter(|s| s.worker.is_some()).count(), Ordering::Relaxed);
                    *l.segs.lock().unwrap_or_else(|p| p.into_inner()) = segs.iter().map(|s| [s.start, s.end, s.written]).collect();
                    // P1 #3: the user's live connection count replaces the ramp. Raising it spawns workers on the
                    // next turn; lowering it stops new leases (running leases finish, never cut mid-chunk).
                    let w = l.want_conns.load(Ordering::Relaxed);
                    if ranged && w > 0 { target = w.clamp(1, MAX_CONNS); ramp_stopped = true; }
                }
                // ramp: +1 every 2 s while total speed rose >= 10 % since the last add (received bytes)
                let rx = sh.rx_bytes.load(Ordering::Relaxed);
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
                // checkpoint: snapshot WRITTEN positions, flush, then persist (never received or queued bytes)
                let unflushed: u64 = segs.iter().map(|s| s.written - s.committed).sum();
                if ranged && (unflushed >= cfg.flush_bytes || (unflushed > 0 && last_flush.elapsed() >= cfg.flush_every)) {
                    let pos: Vec<u64> = segs.iter().map(|s| s.written).collect();
                    let e = epoch_cell.load(Ordering::SeqCst);
                    let f = file.clone();
                    let tf = Instant::now();
                    if let Err(err) = tokio::task::spawn_blocking(move || f.sync_data()).await.expect("flush task") { fail!(io(err)); }
                    flush_time += tf.elapsed();
                    flushes += 1;
                    failpoint::hit("after_sync");
                    let s = Snapshot { epoch: e, final_url: final_url.to_string(), validator: validator.clone(), total, segs: segs.iter().zip(&pos).map(|(s, p)| [s.start, s.end, *p]).collect() };
                    if let Err(err) = (rq.checkpoint)(&s) { fail!(err); }
                    for (s, p) in segs.iter_mut().zip(pos) { s.committed = p; }
                    failpoint::hit("after_commit");
                    last_flush = Instant::now();
                }
            }
        }
    }
    let transfer = t0.elapsed();
    drop(tasks);
    drop(sh); // closes the writer queue
    let _ = wthread.join();
    let got: u64 = segs.iter().map(|s| s.end - s.start).sum();
    if total.is_none() {
        file.set_len(got).map_err(io)?; // before the final sync, so the length is flushed too
    }
    let f = file.clone();
    let tf = Instant::now();
    tokio::task::spawn_blocking(move || f.sync_all()).await.expect("flush task").map_err(io)?;
    failpoint::phase("final_sync", tf);
    flush_time += tf.elapsed();
    let len = file.metadata().map_err(io)?.len();
    if total.is_some_and(|t| t != got) || len != got {
        return Err(Fail::Io(format!("byte count {got}/{len} != total {total:?}")));
    }
    let ts = Instant::now();
    crate::fsx::set_sparse(&file, false); // every byte is written: clearing the flag allocates nothing new
    failpoint::phase("clear_sparse", ts);
    // final checkpoint: everything is synced, so a crash before the rename resumes with a 1-byte probe
    let e = epoch_cell.load(Ordering::SeqCst);
    (rq.checkpoint)(&Snapshot {
        epoch: e,
        final_url: final_url.to_string(),
        validator: validator.clone(),
        total: Some(got),
        segs: segs.iter().map(|s| [s.start, s.end, s.end]).collect(),
    })?;
    failpoint::hit("after_final_checkpoint");
    Ok(Done { total: got, final_url: final_url.to_string(), validator, ranged, peak_conns: peak, transfer, flushes, flush_time })
}

/// bytes [a, b) on the server (same validator) == bytes [a, b) on disk; any doubt => false (refetch)
async fn seam_ok(sh: &Shared, f: &File, a: u64, b: u64) -> bool {
    let Ok(Ok(r)) = tokio::time::timeout(sh.cfg.read_timeout, get(sh, &sh.url, format!("bytes={a}-{}", b - 1), true).send()).await else {
        return false;
    };
    if r.status() != StatusCode::PARTIAL_CONTENT || !http::range_ok(r.headers(), a, b, sh.total) || !http::identity(r.headers()) {
        return false;
    }
    let want = (b - a) as usize;
    let mut r = r;
    let mut body = Vec::with_capacity(want);
    loop {
        match r.chunk().await {
            Ok(Some(c)) => {
                if body.len() + c.len() > want {
                    return false;
                }
                body.extend_from_slice(&c);
            }
            Ok(None) => break,
            Err(_) => return false,
        }
    }
    let mut disk = vec![0u8; want];
    body.len() == want && f.seek_read(&mut disk, a).is_ok_and(|n| n == want) && body == disk
}

#[allow(clippy::too_many_arguments)]
async fn worker(
    _permits: Vec<tokio::sync::OwnedSemaphorePermit>, // released when this connection ends
    wid: usize,
    gen: u64,
    seg: u64,
    start: u64,
    end: u64,
    epoch: u64,
    sh: Arc<Shared>,
    pre: Option<Response>,
    mut ctl: mpsc::UnboundedReceiver<Ctl>,
    ev: mpsc::UnboundedSender<Ev>,
    ranged: bool,
) {
    let n = LIVE_CONNS.fetch_add(1, Ordering::SeqCst) + 1;
    PEAK_CONNS.fetch_max(n, Ordering::SeqCst);
    let mut next = start;
    let r = run_worker(wid, gen, seg, &mut next, end, epoch, &sh, pre, &mut ctl, &ev, ranged).await;
    LIVE_CONNS.fetch_sub(1, Ordering::SeqCst);
    let _ = ev.send(Ev::Exit { wid, next, err: r.err() });
}

#[allow(clippy::too_many_arguments)]
async fn run_worker(
    wid: usize,
    mut _gen: u64,
    seg: u64,
    next: &mut u64,
    mut end: u64,
    epoch: u64,
    sh: &Shared,
    pre: Option<Response>,
    ctl: &mut mpsc::UnboundedReceiver<Ctl>,
    ev: &mpsc::UnboundedSender<Ev>,
    ranged: bool,
) -> Result<(), WErr> {
    let start = *next;
    let resp = match pre {
        Some(r) => r,
        None => {
            let range = if ranged { format!("bytes={start}-{}", end - 1) } else { "bytes=0-".into() };
            let sent = tokio::time::timeout(sh.cfg.read_timeout, get(sh, &sh.url, range, ranged).send()).await;
            let r = match sent {
                Err(_) => return Err(WErr::Retry("timed out waiting for response headers".into())),
                Ok(Err(e)) => {
                    return Err(match net_fail(&e) {
                        f @ (Fail::Forbidden(_) | Fail::Tls(_)) => WErr::Fatal(f),
                        _ => WErr::Retry(e.to_string()),
                    })
                }
                Ok(Ok(r)) => r,
            };
            match http::status(r.status().as_u16(), r.headers()) {
                Step::Accept => {}
                Step::Fatal(f) => return Err(WErr::Fatal(f)),
                Step::RetryAfter(s) => return Err(WErr::RetryAfter(s)),
                Step::Backoff(_) => return Err(WErr::Backoff),
            }
            let ct = http::content_type(r.headers());
            if ct.starts_with("multipart/") {
                return Err(WErr::Fatal(Fail::Multipart));
            }
            if ranged {
                if r.status() == StatusCode::OK {
                    // 200 to a range request: another validator => file changed; same or none => Range ignored
                    let v = http::validator(r.headers());
                    return Err(if v.is_some() && v != sh.validator { WErr::Fatal(Fail::FileChanged) } else { WErr::NoRange });
                }
                if !http::range_ok(r.headers(), start, end, sh.total) || !http::identity(r.headers()) {
                    return Err(WErr::Fatal(Fail::BadContentRange));
                }
                if http::strong_etag(r.headers()).is_some_and(|e| Some(e) != sh.validator) {
                    return Err(WErr::Fatal(Fail::FileChanged));
                }
            } else {
                // a non-ranged retry must be the SAME representation as the probe (council G5)
                if start != 0 {
                    return Err(WErr::Retry("non-ranged restart must begin at 0".into()));
                }
                if ct.starts_with("text/html") {
                    return Err(WErr::Fatal(Fail::Html));
                }
                if sh.validator.is_some() && http::validator(r.headers()) != sh.validator {
                    return Err(WErr::Fatal(Fail::FileChanged));
                }
                let len =
                    if r.status() == StatusCode::PARTIAL_CONTENT { http::content_range(r.headers()).and_then(|x| x.2) } else { r.content_length() };
                if sh.total.is_some() && len != sh.total {
                    return Err(WErr::Fatal(Fail::SizeChanged));
                }
            }
            r
        }
    };
    // bytes THIS response may deliver: never past the lease end, never past its own Content-Range end, never past
    // the declared total; a body that keeps going past that is LongBody (E-01/E-03)
    let declared = if resp.status() == StatusCode::PARTIAL_CONTENT { http::content_range(resp.headers()).map(|(_, b, _)| b + 1) } else { None };
    let promised = match (ranged, declared) {
        (true, Some(d)) => end.min(d),
        (true, None) => end,
        (false, Some(d)) => sh.total.map_or(d, |t| t.min(d)),
        (false, None) => sh.total.unwrap_or(end),
    };
    let mut body = resp.bytes_stream();
    let mut buf: Vec<u8> = Vec::with_capacity(sh.cfg.buf);
    let (mut win_t, mut win_b) = (tokio::time::Instant::now(), 0u64);
    let (min_bps, win) = sh.cfg.min_speed;
    // hand the buffer to the writer; `next` = bytes admitted (queued or written): the split cursor
    macro_rules! flush_buf {
        () => {{
            if !buf.is_empty() {
                let n = buf.len() as u64;
                let data = std::mem::replace(&mut buf, Vec::with_capacity(sh.cfg.buf));
                if sh.wtx.send(Chunk { epoch, seg, off: *next, data }).await.is_err() {
                    return Err(WErr::Retry("writer closed".into()));
                }
                *next += n;
                let _ = ev.send(Ev::Admitted { wid, next: *next });
            }
        }};
    }
    loop {
        let cur = *next + buf.len() as u64;
        if cur >= end {
            flush_buf!();
            return if sh.total.is_none() && !ranged { Err(WErr::Fatal(Fail::SizeExceeded)) } else { Ok(()) };
        }
        tokio::select! {
            biased;
            _ = tokio::time::sleep_until(win_t + win) => {
                if win_b < min_bps * win.as_secs() { flush_buf!(); return Err(WErr::Retry("timed out: below minimum speed".into())); }
                win_t = tokio::time::Instant::now();
                win_b = 0;
            }
            c = ctl.recv() => match c {
                Some(Ctl::Shrink { gen: g, end: e }) => {
                    // boundary ack: everything admitted lies below e, and every later byte is clamped to e
                    if cur >= e { let _ = ev.send(Ev::Nack); } else { end = e; _gen = g; let _ = ev.send(Ev::Ack { wid, gen: g, end: e }); }
                }
                None => return Ok(()),
            },
            chunk = body.next() => match chunk {
                None => {
                    flush_buf!();
                    if sh.total.is_some() && *next < end { return Err(WErr::Retry("short body".into())); }
                    return Ok(());
                }
                Some(Err(e)) => { flush_buf!(); return Err(WErr::Retry(format!("{e} {:?}", std::error::Error::source(&e).map(|s| s.to_string())))); }
                Some(Ok(b)) => {
                    if let Some(l) = &sh.limit { l.take(b.len() as u64).await; }
                    win_b += b.len() as u64;
                    sh.rx_bytes.fetch_add(b.len() as u64, Ordering::Relaxed);
                    let cur = *next + buf.len() as u64;
                    if cur + b.len() as u64 > promised {
                        // the server sent more than promised / asked: keep only what the lease allows, then stop
                        let keep = (end.min(promised) - cur) as usize;
                        buf.extend_from_slice(&b[..keep.min(b.len())]);
                        flush_buf!();
                        if *next >= end && ranged { return Ok(()); }
                        return Err(WErr::Fatal(if sh.total.is_none() { Fail::SizeExceeded } else { Fail::LongBody }));
                    }
                    let room = (end - cur) as usize;
                    buf.extend_from_slice(&b[..room.min(b.len())]);
                    if buf.len() >= sh.cfg.buf || *next + buf.len() as u64 >= end { flush_buf!(); }
                }
            }
        }
    }
}

fn io(e: std::io::Error) -> Fail {
    Fail::Io(e.to_string())
}

/// every early return (error, `?`) aborts the workers
#[derive(Default)]
struct Tasks(Vec<tokio::task::JoinHandle<()>>);
impl Drop for Tasks {
    fn drop(&mut self) {
        self.0.iter().for_each(|t| t.abort());
    }
}
