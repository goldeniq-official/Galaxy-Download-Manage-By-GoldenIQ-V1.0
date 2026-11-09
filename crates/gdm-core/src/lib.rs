// SPDX-License-Identifier: RPL-1.5
//! Galaxy Download Manager download core. Independent of Tauri (ADR-1) so it can become gdm-engine.exe.
//!
//! `Engine::open` -> `Engine::add` (reserves a unique name, N-04) -> `Engine::run` (download, checkpoint,
//! finalise). Every byte offset that reaches SQLite or the sidecar was flushed first (E-04).
pub mod failpoint;
pub mod finalize;
pub mod fsx;
pub mod http;
pub mod limiter;
pub mod name;
pub mod net;
pub mod seg;
pub mod sidecar;
pub mod store;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// why a job stopped; `code()` is what the database and the UI see
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fail {
    /// validator changed / 200 under If-Range (E-02): pause and ask, keep the old .part
    FileChanged,
    /// 416 (E-02)
    SizeChanged,
    /// text/html instead of the file (R-03)
    Html,
    /// multipart/byteranges was never requested
    Multipart,
    /// Content-Range does not match the request (E-01)
    BadContentRange,
    /// more bytes than declared (E-03)
    LongBody,
    /// unknown length reached the cap (E-03): ask
    SizeExceeded,
    /// not enough free space (E-03, keeps a 1 GB reserve)
    DiskFull,
    Auth,
    Gone,
    Http(u16),
    Timeout,
    /// network policy (W-02, scheme, https -> http)
    Forbidden(String),
    /// certificate / TLS failure (W-03): never retried into success
    Tls(String),
    TooManyRetries(String),
    /// resume without a strong validator after a restart (E-02): the user decides, default restart
    NeedsDecision(String),
    /// antivirus / policy removed or refused the file (N-05): never Complete
    Blocked(String),
    HashMismatch,
    /// junction / symlink / path swap (N-03)
    Reparse(String),
    Io(String),
}

impl Fail {
    pub fn code(&self) -> String {
        let s = format!("{self:?}");
        s.split(['(', ' ']).next().unwrap_or("Io").to_string()
    }
}
impl std::fmt::Display for Fail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") }
}

/// E-02 resume policy for a job without a strong validator
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Unverified {
    /// short break in the same session (< 10 min, no restart): continue with seam checks
    SameSession,
    /// after a restart: stop with NeedsDecision (default answer in the UI: restart)
    #[default]
    Ask,
    /// the user chose [continue (unverified)]
    Continue,
    /// the user chose [restart]
    Restart,
}

#[derive(Clone, Debug, Default)]
pub struct NewJob {
    pub url: String,
    pub dir: PathBuf,
    /// None => from the URL (Content-Disposition naming happens in the Download Info flow, `name::file_name`)
    pub name: Option<String>,
    pub referrer: Option<String>,
    /// Cookie / Authorization for the URL's own origin only (W-01); kept in memory, never persisted
    pub creds: Vec<(String, String)>,
    pub sha256: Option<String>,
    /// the user typed this URL (a LAN NAS is fine without asking, W-02)
    pub typed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    pub name: String,
    pub path: PathBuf,
    pub sha256: String,
    pub protection: finalize::Protection,
    pub unverified: bool,
    pub transfer: seg::Done,
}

pub struct Engine {
    pub store: Mutex<store::Store>,
    pub report: store::OpenReport,
    pub cfg: seg::Config,
    pub global: Arc<limiter::Bucket>,
    /// creds per job live only here (never in SQLite or the sidecar)
    creds: Mutex<std::collections::HashMap<String, net::Creds>>,
}

fn s(e: impl std::fmt::Display) -> Fail { Fail::Io(e.to_string()) }

fn new_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("{t:x}-{:x}-{:x}", std::process::id(), N.fetch_add(1, Ordering::Relaxed))
}

fn fid_str(id: (u32, u64)) -> String { format!("{:08x}:{:016x}", id.0, id.1) }

impl Engine {
    /// open (or create) the database: quick_check, restore from backup if corrupt, migrate, daily backup,
    /// then recover jobs from sidecars found in the known download folders (D-01)
    pub fn open(db: &Path, cfg: seg::Config) -> Result<Engine, Fail> {
        let (st, report) = store::Store::open(db).map_err(Fail::Io)?;
        let e = Engine { store: Mutex::new(st), report, cfg, global: limiter::Bucket::new(0), creds: Default::default() };
        if e.report.restored_from.is_some() || e.report.quarantined.is_some() { e.recover_sidecars()?; }
        Ok(e)
    }

    fn st(&self) -> std::sync::MutexGuard<'_, store::Store> { self.store.lock().unwrap_or_else(|p| p.into_inner()) }

    /// scan every known download folder for `*.part.gdm` and re-create jobs the database lost
    pub fn recover_sidecars(&self) -> Result<Vec<String>, Fail> {
        let dirs = self.st().dirs().map_err(Fail::Io)?;
        let mut got = vec![];
        for d in dirs {
            let Ok(dh) = fsx::Dir::open(&d) else { continue };
            let Ok(rd) = std::fs::read_dir(&d) else { continue };
            for ent in rd.flatten() {
                let leaf = ent.file_name().to_string_lossy().to_string();
                if !leaf.ends_with(".part.gdm") { continue; }
                let Ok(raw) = dh.read_small(&leaf, sidecar::MAX) else { continue };
                let Some(sc) = sidecar::decode(&raw) else { continue }; // untrusted: anything odd is ignored
                if sidecar::leaf(&sc.name) != leaf { continue; }
                let mut st = self.st();
                if st.job(&sc.job).map_err(Fail::Io)?.is_some() { continue; }
                let dir_id = st.dir_id(&d).map_err(Fail::Io)?;
                let j = store::Job {
                    id: sc.job.clone(), url: sc.url.clone(), final_url: Some(sc.final_url.clone()), referrer: None, name: sc.name.clone(),
                    dir: d.clone(), dir_id, state: "paused".into(), size: Some(sc.total), done: 0, validator: sc.validator.clone(),
                    sha256_expected: None, sha256: None, epoch: sc.epoch, unverified: sc.validator.is_none(), protection: None,
                    error_code: Some("Recovered".into()), file_id: None, allow_private: None,
                };
                // the recovered job keeps exactly its own name (its .part is on disk): no counter
                let name = st.insert_job(&j, &|_| false).map_err(Fail::Io)?;
                if name != sc.name { continue; }
                st.save_checkpoint(&sc.job, &sc.final_url, sc.validator.as_deref(), Some(sc.total), sc.epoch, &sc.segs).map_err(Fail::Io)?;
                got.push(sc.job);
            }
        }
        Ok(got)
    }

    /// create a job: validate the URL, sanitize the name, reserve it (N-04). Nothing is downloaded yet.
    pub fn add(&self, nj: NewJob) -> Result<String, Fail> {
        let url = url::Url::parse(&nj.url).map_err(|e| Fail::Forbidden(format!("bad URL: {e}")))?;
        if !matches!(url.scheme(), "http" | "https") { return Err(Fail::Forbidden(format!("scheme {}", url.scheme()))); }
        for (k, v) in &nj.creds {
            // W-04: header injection is refused here, before anything is stored
            reqwest::header::HeaderName::from_bytes(k.as_bytes()).map_err(|_| Fail::Forbidden("bad header name".into()))?;
            reqwest::header::HeaderValue::from_str(v).map_err(|_| Fail::Forbidden("bad header value".into()))?;
        }
        let name = nj.name.as_deref().map(name::sanitize).unwrap_or_else(|| name::file_name(None, &url));
        let dirh = fsx::Dir::open(&nj.dir).map_err(|e| Fail::Reparse(e.to_string()))?;
        let id = new_id();
        let mut st = self.st();
        let dir_id = st.dir_id(&dirh.path).map_err(Fail::Io)?;
        let j = store::Job {
            id: id.clone(), url: url.to_string(), final_url: None, referrer: nj.referrer, name, dir: dirh.path.clone(), dir_id,
            state: "queued".into(), size: None, done: 0, validator: None, sha256_expected: nj.sha256, sha256: None, epoch: 0,
            unverified: false, protection: None, error_code: None, file_id: None,
            allow_private: nj.typed.then(|| url.host_str().unwrap_or("").to_string()),
        };
        let taken = |n: &str| dirh.exists(n) || dirh.exists(&format!("{n}.part")) || dirh.exists(&sidecar::leaf(n));
        st.insert_job(&j, &taken).map_err(Fail::Io)?;
        drop(st);
        if !nj.creds.is_empty() {
            self.creds.lock().unwrap_or_else(|p| p.into_inner()).insert(id.clone(), net::Creds { origin: Some(net::origin(&url)), headers: nj.creds });
        }
        Ok(id)
    }

    pub fn job(&self, id: &str) -> Result<store::Job, Fail> {
        self.st().job(id).map_err(Fail::Io)?.ok_or_else(|| Fail::Io(format!("no job {id}")))
    }

    /// download (or resume) and finalise one job
    pub async fn run(&self, id: &str, unverified: Unverified, limit: Option<Arc<limiter::JobLimit>>) -> Result<Outcome, Fail> {
        let r = self.run_inner(id, unverified, limit).await;
        if let Err(f) = &r {
            let state = match f { Fail::Blocked(_) => "blocked", Fail::NeedsDecision(_) | Fail::FileChanged | Fail::SizeChanged | Fail::SizeExceeded | Fail::DiskFull | Fail::Auth | Fail::Gone => "paused", _ => "failed" };
            let _ = self.st().set_state(id, state, Some(&format!("{}: {f}", f.code())));
        }
        r
    }

    async fn run_inner(&self, id: &str, unverified: Unverified, limit: Option<Arc<limiter::JobLimit>>) -> Result<Outcome, Fail> {
        if self.st().read_only { return Err(Fail::Io("database is from a newer GDM: read-only".into())); }
        let job = self.job(id)?;
        if job.state == "complete" || job.state == "unprotected" { return Err(Fail::Io("already finished".into())); }
        let dir = fsx::Dir::open(&job.dir).map_err(|e| Fail::Reparse(e.to_string()))?;
        let part = format!("{}.part", job.name);
        let side = sidecar::leaf(&job.name);
        let url = url::Url::parse(&job.url).map_err(s)?;
        let segs = self.st().segments(id).map_err(Fail::Io)?;
        let mut resume = (!segs.is_empty() && job.size.is_some()).then(|| seg::Snapshot {
            epoch: job.epoch, final_url: job.final_url.clone().unwrap_or_else(|| job.url.clone()), validator: job.validator.clone(), total: job.size, segs,
        });
        let progress = resume.as_ref().is_some_and(|r| r.segs.iter().any(|s| s[2] > s[0]));
        if progress && job.validator.is_none() {
            match unverified {
                Unverified::Ask => return Err(Fail::NeedsDecision("no strong validator: restart (default) or continue unverified".into())),
                Unverified::Restart => resume = None,
                Unverified::SameSession | Unverified::Continue => {}
            }
        }
        // the .part: open ours (resume) or create it new relative to the held directory (N-03)
        let mut opened = dir.open_rw(&part);
        for _ in 0..50 {
            // a cancelled run's writer thread may still be closing its handle
            if !matches!(opened, Err(fsx::FsError::Busy)) { break; }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            opened = dir.open_rw(&part);
        }
        let file = match opened {
            Ok(f) => {
                let fid = fsx::file_id(&f).map_err(s)?;
                if job.file_id.as_deref().is_some_and(|x| x != fid_str(fid)) { resume = None; } // another file object: restart
                f
            }
            Err(fsx::FsError::NotFound) => { resume = None; dir.create_new(&part).map_err(|e| Fail::Reparse(e.to_string()))? }
            Err(fsx::FsError::Reparse(m)) => return Err(Fail::Reparse(m)),
            Err(e) => return Err(Fail::Io(e.to_string())),
        };
        let fid = fsx::file_id(&file).map_err(s)?;
        self.st().set_file_id(id, &fid_str(fid)).map_err(Fail::Io)?;
        self.st().set_state(id, "downloading", None).map_err(Fail::Io)?;
        let creds = self.creds.lock().unwrap_or_else(|p| p.into_inner()).get(id).cloned().unwrap_or_default();
        let policy = Arc::new(net::Policy { allow_private: job.allow_private.iter().filter(|h| !h.is_empty()).cloned().collect() });
        let free = fsx::free_space(&dir.path);
        let mut cp = |snap: &seg::Snapshot| -> Result<(), Fail> {
            let segs = sidecar::compact(&snap.segs);
            if let Some(total) = snap.total {
                let sc = sidecar::Sidecar { v: 1, job: id.to_string(), url: job.url.clone(), final_url: snap.final_url.clone(), name: job.name.clone(), validator: snap.validator.clone(), total, epoch: snap.epoch, segs: segs.clone() };
                if let Some(raw) = sidecar::encode(&sc) { dir.write_atomic(&side, &raw).map_err(s)?; }
            }
            failpoint::hit("after_sidecar");
            self.st().save_checkpoint(id, &snap.final_url, snap.validator.as_deref(), snap.total, snap.epoch, &segs).map_err(Fail::Io)
        };
        let file = Arc::new(file);
        let rq = seg::Request { url, creds, policy, file: file.clone(), free, resume, limit, checkpoint: &mut cp };
        let done = seg::download(rq, self.cfg.clone()).await?;
        drop(file); // the writer thread has exited: this was the last handle
        self.st().set_state(id, "finalizing", None).map_err(Fail::Io)?;
        let fin = {
            let (dir, part, name, exp, url, refr) = (&dir, part.clone(), job.name.clone(), job.sha256_expected.clone(), job.url.clone(), job.referrer.clone());
            let total = done.total;
            tokio::task::block_in_place(|| finalize::finalize(dir, &part, &name, total, exp.as_deref(), &url, refr.as_deref(), Some(fid)))?
        };
        let (state, prot) = match &fin.protection { finalize::Protection::Protected => ("complete", "protected"), finalize::Protection::Unprotected(_) => ("unprotected", "unprotected") };
        {
            let st = self.st();
            st.finish(id, state, &fin.name, &fin.sha256, prot).map_err(Fail::Io)?;
            st.claim_name(job.dir_id, &fin.name, id).map_err(Fail::Io)?;
        }
        let _ = dir.delete(&side);
        self.creds.lock().unwrap_or_else(|p| p.into_inner()).remove(id);
        Ok(Outcome { path: dir.path.join(&fin.name), name: fin.name, sha256: fin.sha256, protection: fin.protection, unverified: done.validator.is_none(), transfer: done })
    }
}
