// SPDX-License-Identifier: RPL-1.5
//! Galaxy Download Manager download core. Independent of Tauri (ADR-1) so it can become gdm-engine.exe.
//!
//! `Engine::open` -> `Engine::add` (reserves a unique name, N-04) -> `Engine::run` (download, checkpoint,
//! finalise). Every byte offset that reaches SQLite or the sidecar was flushed first (E-04).
pub use reqwest;
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
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
    /// Cookie and/or Authorization (nothing else) for the URL's own origin only (W-01); memory only, never persisted
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
    /// one execution per job at a time
    running: Mutex<std::collections::HashSet<String>>,
}

struct RunGuard<'a>(&'a Engine, String);
impl Drop for RunGuard<'_> {
    fn drop(&mut self) {
        self.0.running.lock().unwrap_or_else(|p| p.into_inner()).remove(&self.1);
    }
}

fn s(e: impl std::fmt::Display) -> Fail {
    Fail::Io(e.to_string())
}

fn new_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("{t:x}-{:x}-{:x}", std::process::id(), N.fetch_add(1, Ordering::Relaxed))
}

fn fid_str(id: (u32, u64)) -> String {
    format!("{:08x}:{:016x}", id.0, id.1)
}

impl Engine {
    /// open (or create) the database: quick_check, restore from backup if corrupt, migrate, daily backup,
    /// then recover jobs from sidecars found in the known download folders (D-01)
    pub fn open(db: &Path, cfg: seg::Config) -> Result<Engine, Fail> {
        let (st, report) = store::Store::open(db).map_err(Fail::Io)?;
        let e =
            Engine { store: Mutex::new(st), report, cfg, global: limiter::Bucket::new(0), creds: Default::default(), running: Default::default() };
        if e.report.restored_from.is_some() || e.report.quarantined.is_some() {
            e.recover_sidecars()?;
        }
        Ok(e)
    }

    fn st(&self) -> std::sync::MutexGuard<'_, store::Store> {
        self.store.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// scan every known download folder for `*.part.gdm` and re-create jobs the database lost
    pub fn recover_sidecars(&self) -> Result<Vec<String>, Fail> {
        let dirs = self.st().dirs().map_err(Fail::Io)?;
        let mut got = vec![];
        for d in dirs {
            let Ok(dh) = fsx::Dir::open(&d) else { continue };
            let Ok(rd) = std::fs::read_dir(&d) else { continue };
            for ent in rd.flatten() {
                let leaf = ent.file_name().to_string_lossy().to_string();
                if !leaf.ends_with(".part.gdm") {
                    continue;
                }
                let Ok(raw) = dh.read_small(&leaf, sidecar::MAX) else { continue };
                let Some(sc) = sidecar::decode(&raw) else { continue }; // untrusted: anything odd is ignored
                if sidecar::leaf(&sc.name) != leaf {
                    continue;
                }
                let mut st = self.st();
                if st.job(&sc.job).map_err(Fail::Io)?.is_some() {
                    continue;
                }
                let dir_id = st.dir_id(&d).map_err(Fail::Io)?;
                let j = store::Job {
                    id: sc.job.clone(),
                    url: sc.url.clone(),
                    final_url: Some(sc.final_url.clone()),
                    referrer: None,
                    name: sc.name.clone(),
                    dir: d.clone(),
                    dir_id,
                    state: "paused".into(),
                    size: Some(sc.total),
                    done: 0,
                    validator: sc.validator.clone(),
                    sha256_expected: None,
                    sha256: None,
                    epoch: sc.epoch,
                    unverified: sc.validator.is_none(),
                    protection: None,
                    error_code: Some("Recovered".into()),
                    file_id: Some(sc.fid.clone()),
                    allow_private: None,
                    target: None,
                };
                // the recovered job keeps exactly its own name (its .part is on disk): no counter
                let name = st.insert_job(&j, &|_| false).map_err(Fail::Io)?;
                if name != sc.name {
                    continue;
                }
                st.save_checkpoint(&sc.job, &sc.final_url, sc.validator.as_deref(), Some(sc.total), sc.epoch, &sc.segs).map_err(Fail::Io)?;
                st.set_file_id(&sc.job, &sc.fid).map_err(Fail::Io)?;
                got.push(sc.job);
            }
        }
        Ok(got)
    }

    /// create a job: validate the URL, sanitize the name, reserve it (N-04). Nothing is downloaded yet.
    pub fn add(&self, nj: NewJob) -> Result<String, Fail> {
        let url = url::Url::parse(&nj.url).map_err(|e| Fail::Forbidden(format!("bad URL: {e}")))?;
        if !url.username().is_empty() || url.password().is_some() {
            return Err(Fail::Forbidden("credentials in the URL are not stored; use the login fields".into()));
        }
        if !matches!(url.scheme(), "http" | "https") {
            return Err(Fail::Forbidden(format!("scheme {}", url.scheme())));
        }
        for (k, v) in &nj.creds {
            // W-01: only the two credential headers reqwest also strips on a cross-host redirect; any other
            // header would follow a redirect to another origin. W-04: CR/LF/NUL refused before anything is kept.
            if !k.eq_ignore_ascii_case("cookie") && !k.eq_ignore_ascii_case("authorization") {
                return Err(Fail::Forbidden(format!("credential header {k} not allowed")));
            }
            reqwest::header::HeaderValue::from_str(v).map_err(|_| Fail::Forbidden("bad header value".into()))?;
        }
        let name = nj.name.as_deref().map(name::sanitize).unwrap_or_else(|| name::file_name(None, &url));
        let dirh = fsx::Dir::open(&nj.dir).map_err(|e| Fail::Reparse(e.to_string()))?;
        let id = new_id();
        let mut st = self.st();
        let dir_id = st.dir_id(&dirh.path).map_err(Fail::Io)?;
        let j = store::Job {
            id: id.clone(),
            url: url.to_string(),
            final_url: None,
            referrer: nj.referrer,
            name,
            dir: dirh.path.clone(),
            dir_id,
            state: "queued".into(),
            size: None,
            done: 0,
            validator: None,
            sha256_expected: nj.sha256,
            sha256: None,
            epoch: 0,
            unverified: false,
            protection: None,
            error_code: None,
            file_id: None,
            allow_private: nj.typed.then(|| url.host_str().unwrap_or("").to_string()),
            target: None,
        };
        let taken = |n: &str| dirh.exists(n) || dirh.exists(&format!("{n}.part")) || dirh.exists(&sidecar::leaf(n));
        st.insert_job(&j, &taken).map_err(Fail::Io)?;
        drop(st);
        if !nj.creds.is_empty() {
            self.creds
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .insert(id.clone(), net::Creds { origin: Some(net::origin(&url)), headers: nj.creds });
        }
        Ok(id)
    }

    /// the user confirmed "download from the local network?" for this job's host (W-02). A job recovered from
    /// a sidecar never carries this intent: it is asked again.
    pub fn allow_private(&self, id: &str, host: &str) -> Result<(), Fail> {
        self.st().conn.execute("UPDATE jobs SET allow_private = ?2 WHERE id = ?1", [id, host]).map(|_| ()).map_err(s)
    }

    pub fn job(&self, id: &str) -> Result<store::Job, Fail> {
        self.st().job(id).map_err(Fail::Io)?.ok_or_else(|| Fail::Io(format!("no job {id}")))
    }

    /// download (or resume) and finalise one job
    pub async fn run(&self, id: &str, unverified: Unverified, limit: Option<Arc<limiter::JobLimit>>) -> Result<Outcome, Fail> {
        self.run_live(id, unverified, limit, None).await
    }

    /// `run` with a live progress view for the UI. Dropping the future pauses the job: nothing past the last
    /// checkpoint is claimed (E-04); call `stopped` afterwards so the list does not show "downloading".
    pub async fn run_live(
        &self,
        id: &str,
        unverified: Unverified,
        limit: Option<Arc<limiter::JobLimit>>,
        live: Option<Arc<seg::Live>>,
    ) -> Result<Outcome, Fail> {
        if !self.running.lock().unwrap_or_else(|p| p.into_inner()).insert(id.to_string()) {
            return Err(Fail::Io("this job is already running".into()));
        }
        let _guard = RunGuard(self, id.to_string());
        let r = self.run_inner(id, unverified, limit, live).await;
        if let Err(f) = &r {
            let state = match f {
                Fail::Blocked(_) => "blocked",
                Fail::NeedsDecision(_) | Fail::FileChanged | Fail::SizeChanged | Fail::SizeExceeded | Fail::DiskFull | Fail::Auth | Fail::Gone => {
                    "paused"
                }
                _ => "failed",
            };
            let _ = self.st().set_state(id, state, Some(&format!("{}: {f}", f.code())));
        }
        r
    }

    /// crash after the final rename, before "complete" was recorded: the published file is ours only when it is
    /// the recorded target, the same file object, the recorded size and the hash recorded before the rename
    fn reconcile(&self, job: &store::Job, dir: &fsx::Dir) -> Result<Option<Outcome>, Fail> {
        let (Some(target), Some(want_id), Some(want_sha)) = (job.target.clone(), job.file_id.clone(), job.sha256.clone()) else { return Ok(None) };
        let f = match dir.open_locked(&target) {
            Ok(f) => f,
            Err(fsx::FsError::NotFound) => return Ok(None),
            Err(e) => return Err(Fail::Io(format!("published file: {e}"))),
        };
        let len = f.metadata().map_err(s)?.len();
        if fsx::file_id(&f).map(fid_str).ok() != Some(want_id) || Some(len) != job.size {
            return Err(Fail::NeedsDecision(format!("{target} is not the file this job published")));
        }
        let sha = finalize::sha_handle(&f).map_err(s)?;
        if sha != want_sha {
            return Err(Fail::NeedsDecision(format!("{target} changed after it was published (sha256 differs)")));
        }
        let zone = fsx::zone_identifier(dir, &target);
        let prot = if finalize::zone_internet(zone.as_deref()) && matches!(fsx::fs_name(&f).as_str(), "NTFS" | "ReFS") {
            finalize::Protection::Protected
        } else {
            finalize::Protection::Unprotected("filesystem without Mark-of-the-Web".into())
        };
        let (state, p) = if prot == finalize::Protection::Protected { ("complete", "protected") } else { ("unprotected", "unprotected") };
        let _ = dir.delete(&sidecar::leaf(&job.name));
        self.st().finish(&job.id, state, &target, &sha, p).map_err(Fail::Io)?;
        let transfer = seg::Done {
            total: len,
            final_url: job.final_url.clone().unwrap_or_default(),
            validator: job.validator.clone(),
            ranged: true,
            peak_conns: 0,
            transfer: Default::default(),
            flushes: 0,
            flush_time: Default::default(),
        };
        Ok(Some(Outcome { path: dir.path.join(&target), name: target, sha256: sha, protection: prot, unverified: job.unverified, transfer }))
    }

    async fn run_inner(
        &self,
        id: &str,
        unverified: Unverified,
        limit: Option<Arc<limiter::JobLimit>>,
        live: Option<Arc<seg::Live>>,
    ) -> Result<Outcome, Fail> {
        if self.st().read_only {
            return Err(Fail::Io("database is from a newer GDM: read-only".into()));
        }
        let job = self.job(id)?;
        if job.state == "complete" || job.state == "unprotected" {
            return Err(Fail::Io("already finished".into()));
        }
        let dir = fsx::Dir::open(&job.dir).map_err(|e| Fail::Reparse(e.to_string()))?;
        let part = format!("{}.part", job.name);
        let side = sidecar::leaf(&job.name);
        let url = url::Url::parse(&job.url).map_err(s)?;
        // a recorded publication intent survives errors (e.g. rename done, then the "complete" write failed):
        // reconcile on the intent, whatever state a failed run left behind
        if job.target.is_some() && !dir.exists(&part) {
            if let Some(o) = self.reconcile(&job, &dir)? {
                return Ok(o);
            }
        }
        let segs = self.st().segments(id).map_err(Fail::Io)?;
        let mut resume = (!segs.is_empty() && job.size.is_some()).then(|| seg::Snapshot {
            epoch: job.epoch,
            final_url: job.final_url.clone().unwrap_or_else(|| job.url.clone()),
            validator: job.validator.clone(),
            total: job.size,
            segs,
        });
        let progress = resume.as_ref().is_some_and(|r| r.segs.iter().any(|s| s[2] > s[0]));
        if progress && job.validator.is_none() {
            match unverified {
                Unverified::Ask => return Err(Fail::NeedsDecision("no strong validator: restart (default) or continue unverified".into())),
                Unverified::Restart => resume = None,
                Unverified::SameSession | Unverified::Continue => {}
            }
        }
        // The .part (N-03, N-10). A job that never created one must not adopt a file it finds there, except an
        // empty single-link file (its own crash between create and recording the file id: nothing to lose).
        let mut opened = dir.open_rw(&part);
        for _ in 0..50 {
            // a cancelled run's writer thread may still be closing its handle
            if !matches!(opened, Err(fsx::FsError::Busy)) {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            opened = dir.open_rw(&part);
        }
        let file = match (opened, job.file_id.as_deref()) {
            (Ok(f), Some(want)) => {
                if fsx::file_id(&f).map(fid_str).map_err(s)? != want {
                    return Err(Fail::NeedsDecision("the .part is a different file than the one this job created".into()));
                }
                f
            }
            (Ok(f), None) => {
                if f.metadata().map_err(s)?.len() != 0 {
                    return Err(Fail::NeedsDecision("a .part this job did not create is in the folder".into()));
                }
                resume = None;
                f
            }
            (Err(fsx::FsError::NotFound), _) => {
                resume = None;
                self.forget(id, &dir, &side, job.epoch + 1)?; // before a new file replaces the old claims
                let f = dir.create_new(&part).map_err(|e| Fail::Reparse(e.to_string()))?;
                failpoint::hit("after_part_created");
                f
            }
            (Err(fsx::FsError::Reparse(m)), _) => return Err(Fail::Reparse(m)),
            (Err(e), _) => return Err(Fail::Io(e.to_string())),
        };
        if resume.is_none() && job.done > 0 {
            self.forget(id, &dir, &side, job.epoch + 1)?; // old offsets are gone BEFORE anything is truncated
        }
        let fid = fsx::file_id(&file).map_err(s)?;
        self.st().set_file_id(id, &fid_str(fid)).map_err(Fail::Io)?;
        self.st().set_state(id, "downloading", None).map_err(Fail::Io)?;
        let creds = self.creds.lock().unwrap_or_else(|p| p.into_inner()).get(id).cloned().unwrap_or_default();
        let policy = Arc::new(net::Policy { allow_private: job.allow_private.iter().filter(|h| !h.is_empty()).cloned().collect() });
        let free = fsx::free_space(&dir.path);
        let limit = limit.or_else(|| {
            (self.global.rate() > 0).then(|| Arc::new(limiter::JobLimit { job: limiter::Bucket::new(0), queue: None, global: self.global.clone() }))
        });
        let mut cp = |snap: &seg::Snapshot| -> Result<(), Fail> {
            let segs = sidecar::compact(&snap.segs);
            if let Some(total) = snap.total {
                let sc = sidecar::Sidecar {
                    v: 1,
                    job: id.to_string(),
                    url: finalize::clean_url(&job.url),
                    final_url: finalize::clean_url(&snap.final_url),
                    name: job.name.clone(),
                    validator: snap.validator.clone(),
                    total,
                    epoch: snap.epoch,
                    fid: fid_str(fid),
                    segs: segs.clone(),
                };
                if let Some(raw) = sidecar::encode(&sc) {
                    dir.write_atomic(&side, &raw).map_err(s)?;
                }
            }
            failpoint::hit("after_sidecar");
            self.st().save_checkpoint(id, &snap.final_url, snap.validator.as_deref(), snap.total, snap.epoch, &segs).map_err(Fail::Io)
        };
        let file = Arc::new(file);
        let rq = seg::Request { url, creds, policy, file: file.clone(), free, resume, limit, checkpoint: &mut cp, live };
        let done = seg::download(rq, self.cfg.clone()).await?;
        drop(file); // the writer thread has exited: this was the last handle
        failpoint::hit("before_finalizing");
        self.st().set_state(id, "finalizing", None).map_err(Fail::Io)?;
        let fin = {
            let (dir, part, name, exp, url, refr) =
                (&dir, part.clone(), job.name.clone(), job.sha256_expected.clone(), job.url.clone(), job.referrer.clone());
            let total = done.total;
            let mut intent = |target: &str, sha: &str| self.st().set_intent(id, target, sha).map_err(Fail::Io);
            tokio::task::block_in_place(|| {
                finalize::finalize(dir, &part, &name, total, exp.as_deref(), &url, refr.as_deref(), Some(fid), &mut intent)
            })?
        };
        let (state, prot) = match &fin.protection {
            finalize::Protection::Protected => ("complete", "protected"),
            finalize::Protection::Unprotected(_) => ("unprotected", "unprotected"),
        };
        // the sidecar goes first: after a crash here the job is still "finalizing" and reconcile() finishes it
        let _ = dir.delete(&side);
        failpoint::hit("before_finish");
        self.st().finish(id, state, &fin.name, &fin.sha256, prot).map_err(Fail::Io)?;
        failpoint::hit("after_finish");
        self.creds.lock().unwrap_or_else(|p| p.into_inner()).remove(id);
        Ok(Outcome {
            path: dir.path.join(&fin.name),
            name: fin.name,
            sha256: fin.sha256,
            protection: fin.protection,
            unverified: done.validator.is_none(),
            transfer: done,
        })
    }

    pub fn is_running(&self, id: &str) -> bool {
        self.running.lock().unwrap_or_else(|p| p.into_inner()).contains(id)
    }

    /// after a cancelled (paused) run: a job left "downloading" becomes "paused"; "finalizing" stays so the
    /// next run reconciles the publication intent
    pub fn stopped(&self, id: &str) -> Result<(), Fail> {
        let j = self.job(id)?;
        if j.state == "downloading" || j.state == "queued" {
            self.st().set_state(id, "paused", None).map_err(Fail::Io)?;
        }
        Ok(())
    }

    /// remove a job from the list. An unfinished job's own .part and sidecar are deleted (a .part that is not
    /// the file this job created is left alone, N-03); a finished file goes to the Recycle Bin only when asked
    /// and only when it is still the file this job published.
    pub fn remove(&self, id: &str, delete_file: bool) -> Result<(), Fail> {
        if self.is_running(id) {
            return Err(Fail::Io("pause the job first".into()));
        }
        let job = self.job(id)?;
        if let Ok(dir) = fsx::Dir::open(&job.dir) {
            let open = |leaf: &str| {
                let mut r = dir.open_locked(leaf);
                for _ in 0..50 {
                    // a cancelled run's writer thread may still be closing its handle
                    if !matches!(r, Err(fsx::FsError::Busy)) {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    r = dir.open_locked(leaf);
                }
                r
            };
            let ours = |leaf: &str| match (open(leaf), job.file_id.as_deref()) {
                (Ok(f), Some(want)) => fsx::file_id(&f).map(fid_str).ok().as_deref() == Some(want),
                _ => false,
            };
            if job.state == "complete" || job.state == "unprotected" {
                if delete_file && ours(&job.name) {
                    fsx::recycle(&dir.path.join(&job.name)).map_err(|e| Fail::Io(e.to_string()))?;
                }
            } else {
                let part = format!("{}.part", job.name);
                if ours(&part) {
                    dir.delete(&part).map_err(|e| Fail::Io(e.to_string()))?;
                }
                let _ = dir.delete(&sidecar::leaf(&job.name));
            }
        }
        self.creds.lock().unwrap_or_else(|p| p.into_inner()).remove(id);
        self.st().delete_job(id).map_err(Fail::Io)
    }

    /// drop every committed offset of a job (database first, then its sidecar) before its bytes are replaced
    fn forget(&self, id: &str, dir: &fsx::Dir, side: &str, epoch: u64) -> Result<(), Fail> {
        self.st().reset(id, epoch).map_err(Fail::Io)?;
        let _ = dir.delete(side);
        failpoint::hit("after_reset");
        Ok(())
    }
}
