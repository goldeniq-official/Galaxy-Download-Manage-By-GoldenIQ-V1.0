// SPDX-License-Identifier: RPL-1.5
#![allow(dead_code)]
use gdm_core::{seg, Engine, Fail, NewJob, Outcome, Unverified};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const MIB: u64 = 1 << 20;

/// a fresh directory under %TEMP% (outside OneDrive)
pub fn tmpdir(tag: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let d = std::env::temp_dir().join(format!("gdm-t-{tag}-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

pub fn quick() -> seg::Config {
    seg::Config { read_timeout: Duration::from_secs(3), min_speed: (1024, Duration::from_secs(4)), retries: 2, ..seg::Config::default() }
}

pub struct T {
    pub root: PathBuf,
    pub dl: PathBuf,
    pub engine: Engine,
}

pub fn engine(tag: &str, cfg: seg::Config) -> T {
    let root = tmpdir(tag);
    let dl = root.join("Downloads");
    std::fs::create_dir_all(&dl).unwrap();
    let engine = Engine::open(&root.join("data").join("gdm.db"), cfg).unwrap();
    T { root, dl, engine }
}

impl T {
    pub fn job(&self, url: &str) -> NewJob {
        NewJob { url: url.into(), dir: self.dl.clone(), typed: true, ..Default::default() }
    }
    pub async fn get(&self, url: &str) -> (String, Result<Outcome, Fail>) {
        let id = self.engine.add(self.job(url)).unwrap();
        let r = tokio::time::timeout(Duration::from_secs(90), self.engine.run(&id, Unverified::Ask, None)).await
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
