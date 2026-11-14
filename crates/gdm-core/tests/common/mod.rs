// SPDX-License-Identifier: RPL-1.5
#![allow(dead_code)]
use gdm_core::{seg, Engine, Fail, NewJob, Outcome, Unverified};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const MIB: u64 = 1 << 20;

/// run a job until its first checkpoint reached the database, then interrupt it (drop the run). A fixed time
/// window was flaky: when the disk is busy the first flush lands later (bounded by `max`).
pub async fn run_until_checkpoint(e: &Engine, id: &str, mode: Unverified, max: Duration) {
    let run = e.run(id, mode, None);
    tokio::pin!(run);
    let deadline = std::time::Instant::now() + max;
    while e.job(id).unwrap().done == 0 && std::time::Instant::now() < deadline {
        tokio::select! {
            _ = &mut run => return,
            _ = tokio::time::sleep(Duration::from_millis(100)) => {}
        }
    }
}

/// a fresh directory under %TEMP% (outside OneDrive), removed when the returned guard drops
pub fn tmpdir(tag: &str) -> Tmp {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let d = std::env::temp_dir().join(format!("gdm-t-{tag}-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    Tmp(d)
}

/// a test folder: a passing test removes it (downloads can be 100+ MiB); a failing one keeps it for inspection.
/// Declare it before the Engine that uses it: locals drop in reverse order, so the open gdm.db is closed first
/// (Windows refuses to delete an open file).
pub struct Tmp(PathBuf);

impl std::ops::Deref for Tmp {
    type Target = PathBuf;
    fn deref(&self) -> &PathBuf {
        &self.0
    }
}

impl AsRef<Path> for Tmp {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

pub fn quick() -> seg::Config {
    seg::Config { read_timeout: Duration::from_secs(3), min_speed: (1024, Duration::from_secs(4)), retries: 2, ..seg::Config::default() }
}

pub struct T {
    pub root: PathBuf,
    pub dl: PathBuf,
    pub engine: Engine,
    // last field: fields drop in declaration order, so the engine (and its open gdm.db) is closed before the
    // folder is removed; a Drop on T itself ran first and left every test folder behind (20 GB in %TEMP%)
    _dir: Tmp,
}

pub fn engine(tag: &str, cfg: seg::Config) -> T {
    let dir = tmpdir(tag);
    let root = dir.to_path_buf();
    let dl = root.join("Downloads");
    std::fs::create_dir_all(&dl).unwrap();
    let engine = Engine::open(&root.join("data").join("gdm.db"), cfg).unwrap();
    // the request traces of the G5/G6 tests have no `<file>.sha256` probe (P1 #10 tests turn it on)
    engine.hash_sidecar.store(false, std::sync::atomic::Ordering::Relaxed);
    T { root, dl, engine, _dir: dir }
}

impl T {
    pub fn job(&self, url: &str) -> NewJob {
        NewJob { url: url.into(), dir: self.dl.clone(), typed: true, ..Default::default() }
    }
    pub async fn get(&self, url: &str) -> (String, Result<Outcome, Fail>) {
        let id = self.engine.add(self.job(url)).unwrap();
        let r = tokio::time::timeout(Duration::from_secs(90), self.engine.run(&id, Unverified::Ask, None))
            .await
            .unwrap_or_else(|_| Err(Fail::Io("test deadline 90 s".into())));
        (id, r)
    }
    /// every file in the download folder (names)
    pub fn files(&self) -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(&self.dl).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        v.sort();
        v
    }
}

pub fn sha_file(p: &Path) -> String {
    gdm_core::finalize::sha_handle(&std::fs::File::open(p).unwrap()).unwrap()
}
