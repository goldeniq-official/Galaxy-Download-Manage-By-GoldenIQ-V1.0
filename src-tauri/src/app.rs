// SPDX-License-Identifier: RPL-1.5
//! App state between the UI, the bridge pipe and gdm-core: settings, categories, running jobs (live progress,
//! pause = drop the run), the Download Info queue, toasts. Commands address jobs by ID only (X-01, X-02).
use gdm_core::{limiter, seg::Live, Engine, Fail, NewJob, Unverified};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

pub fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Category {
    pub id: String,
    pub name_km: String,
    pub name_en: String,
    /// sub-folder of the download folder ("" = the folder itself)
    pub sub: String,
    pub exts: Vec<String>,
}

fn cat(id: &str, km: &str, en: &str, sub: &str, exts: &str) -> Category {
    Category {
        id: id.into(),
        name_km: km.into(),
        name_en: en.into(),
        sub: sub.into(),
        exts: exts.split(' ').filter(|s| !s.is_empty()).map(String::from).collect(),
    }
}

/// IDM 6.43 defaults (idm.md §1.6, §22): six categories, sub-folders of Downloads
pub fn default_categories() -> Vec<Category> {
    vec![
        cat("general", "ទូទៅ", "General", "", ""),
        cat("compressed", "ឯកសារបង្ហាប់", "Compressed", "Compressed", "zip rar 7z arj gz bz2 xz tar tgz sit sitx sea ace lzh z"),
        cat("documents", "ឯកសារ", "Documents", "Documents", "pdf doc docx ppt pptx pps xls xlsx odt ods odp epub"),
        cat("music", "តន្ត្រី", "Music", "Music", "mp3 wav wma mpa ram ra aac aif m4a ogg flac opus"),
        cat("programs", "កម្មវិធី", "Programs", "Programs", "exe msi msu msix appx apk iso img bin"),
        cat("video", "វីដេអូ", "Video", "Video", "avi mpg mpe mpeg asf wmv mov qt rm rmvb mp4 m4v flv webm ogv mkv 3gp ts"),
    ]
}

/// IDM's default "File Types" list (catch rules, §22), as lower-case extensions
pub const CATCH_EXTS: &str = "3gp 7z aac ace aif apk arj asf avi bin bz2 exe gz gzip img iso lzh m4a m4v mkv mov mp3 mp4 mpa mpe mpeg mpg msi msu ogg ogv pdf plj pps ppt qt ra rar rm rmvb sea sit sitx tar tif tiff wav wma wmv xz z zip";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    pub lang: String,
    /// system | dark | light
    pub theme: String,
    pub onboarded: bool,
    pub download_dir: String,
    pub max_conns: usize,
    pub limit_on: bool,
    pub limit_kbps: u64,
    pub catch_enabled: bool,
    pub catch_exts: Vec<String>,
    pub catch_min_kb: u64,
    /// site patterns never caught (C-01); `*` wildcard
    pub catch_skip: Vec<String>,
    pub force_key: String,
    pub bypass_key: String,
    pub show_info: bool,
    pub video_button: bool,
    pub clipboard: bool,
    /// auto | notify | off (§07)
    pub update_mode: String,
    pub update_channel: String,
    pub silent: bool,
    pub autostart: bool,
    pub columns: Vec<String>,
    pub categories: Vec<Category>,
    pub highest_seen: String,
    pub last_check: i64,
    pub update_failures: u32,
    pub queues: Vec<crate::queue::Queue>,
    /// "run a program when done" per queue id: written ONLY by the `hook_set` command (Q-01)
    pub hooks: HashMap<String, crate::hook::Hook>,
    /// sound per event (complete | failed | queue): "" = off, "system:<alias>" = built-in, else a WAV path
    pub sounds: HashMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            lang: "km".into(),
            theme: "system".into(),
            onboarded: false,
            download_dir: downloads_dir().display().to_string(),
            max_conns: 8,
            limit_on: false,
            limit_kbps: 0,
            catch_enabled: true,
            catch_exts: CATCH_EXTS.split(' ').map(String::from).collect(),
            catch_min_kb: 1024,
            catch_skip: vec!["*.reddit.com/compression-dictionary*".into()],
            force_key: "ctrl".into(),
            bypass_key: "alt".into(),
            show_info: true,
            video_button: false,
            clipboard: false,
            update_mode: "auto".into(),
            update_channel: "stable".into(),
            silent: false,
            autostart: false,
            columns: ["name", "size", "status", "speed", "eta", "added"].map(String::from).to_vec(),
            categories: default_categories(),
            highest_seen: env!("CARGO_PKG_VERSION").into(),
            last_check: 0,
            update_failures: 0,
            queues: crate::queue::default_queues(),
            hooks: HashMap::new(),
            sounds: HashMap::new(),
        }
    }
}

pub fn downloads_dir() -> PathBuf {
    std::env::var_os("USERPROFILE").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("C:\\")).join("Downloads")
}

pub fn ext_of(name: &str) -> String {
    gdm_core::name::real_extension(name)
}

/// the per-user data folder (db, settings, backups): %LOCALAPPDATA%\<identifier>, the folder the NSIS
/// uninstaller's "delete the application data" box removes (T-03: the user chooses; downloads stay)
pub fn data_dir() -> PathBuf {
    std::env::var_os("GDM_DATA").map(PathBuf::from).unwrap_or_else(|| {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join("GoldenIQ.GalaxyDownloadManager")
    })
}

#[derive(Serialize, Clone, Debug)]
pub struct Toast {
    pub n: u64,
    /// complete | unprotected | blocked | failed | info
    pub kind: String,
    pub id: String,
    pub name: String,
    pub dangerous: bool,
}

struct RunCtl {
    handle: tauri::async_runtime::JoinHandle<()>,
    live: Arc<Live>,
    job_limit: Arc<limiter::Bucket>,
    speed: f64,
    last: (Instant, u64),
}

#[derive(Serialize, Clone, Debug)]
pub struct LiveView {
    pub id: String,
    pub done: u64,
    pub total: u64,
    pub speed: f64,
    pub conns: usize,
    pub segs: Vec<[u64; 3]>,
}

#[derive(Serialize, Clone, Debug)]
pub struct RowView {
    #[serde(flatten)]
    pub row: gdm_core::store::Row,
    pub category: String,
    pub dangerous: bool,
    pub ext: String,
}

pub struct App {
    pub engine: Arc<Engine>,
    pub data: PathBuf,
    pub settings: Mutex<Settings>,
    runs: Mutex<HashMap<String, RunCtl>>,
    pub rev: AtomicU64,
    pub pending: Mutex<Vec<String>>,
    toasts: Mutex<VecDeque<Toast>>,
    toast_n: AtomicU64,
    pub browsers_seen: Mutex<HashMap<String, i64>>,
    handoffs: Mutex<VecDeque<Instant>>,
    /// handoff id -> job id (B-05: the browser finished first => drop our copy)
    pub handoff_jobs: Mutex<HashMap<String, (String, Instant)>>,
    pub banner: Mutex<Option<String>>,
    pub notify: Mutex<Option<Notify>>,
    pub qstate: Mutex<HashMap<String, crate::queue::QState>>,
    /// a finish action is being carried out: nothing new starts (Q-02)
    pub finishing: std::sync::atomic::AtomicBool,
    pub power: Mutex<crate::queue::Power>,
    /// Q-02 countdown before a power action (60 s; tests shorten it)
    pub countdown_ms: AtomicU64,
    /// queue speed buckets (queue id -> bucket)
    pub qbuckets: Mutex<HashMap<String, Arc<limiter::Bucket>>>,
}

pub type Notify = Box<dyn Fn(&Toast) + Send + Sync>;

pub const HANDOFFS_PER_MIN: usize = 30;

impl App {
    pub fn open(data: &Path) -> Result<Arc<App>, String> {
        std::fs::create_dir_all(data).map_err(|e| e.to_string())?;
        let sp = data.join("settings.json");
        let settings: Settings = match std::fs::read(&sp) {
            Err(_) => Settings::default(),
            Ok(b) => serde_json::from_slice(&b).unwrap_or_else(|_| {
                // a damaged settings file is kept for the user, never silently overwritten (UX rule 2)
                let _ = std::fs::rename(&sp, data.join(format!("settings.json.bad-{}", gdm_core::store::now())));
                Settings::default()
            }),
        };
        let engine = Engine::open(&data.join("gdm.db"), gdm_core::seg::Config { cap: settings.max_conns.clamp(1, 32), ..Default::default() })
            .map_err(|e| e.to_string())?;
        let banner = engine
            .report
            .restored_from
            .as_ref()
            .map(|p| format!("restored:{}", p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()));
        let banner = banner.or(engine.report.read_only.then(|| "readonly".to_string()));
        let app = Arc::new(App {
            engine: Arc::new(engine),
            data: data.to_path_buf(),
            settings: Mutex::new(settings),
            runs: Default::default(),
            rev: AtomicU64::new(1),
            pending: Default::default(),
            toasts: Default::default(),
            toast_n: AtomicU64::new(0),
            browsers_seen: Default::default(),
            handoffs: Default::default(),
            handoff_jobs: Default::default(),
            banner: Mutex::new(banner),
            notify: Mutex::new(None),
            qstate: Default::default(),
            finishing: Default::default(),
            power: Mutex::new(crate::queue::real_power()),
            countdown_ms: AtomicU64::new(crate::queue::COUNTDOWN.as_millis() as u64),
            qbuckets: Default::default(),
        });
        app.apply_limit();
        Ok(app)
    }

    pub fn save_settings(&self) -> Result<(), String> {
        let s = serde_json::to_vec_pretty(&*lock(&self.settings)).map_err(|e| e.to_string())?;
        let tmp = self.data.join("settings.json.tmp");
        std::fs::write(&tmp, s).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, self.data.join("settings.json")).map_err(|e| e.to_string())
    }

    /// settings_set: merge a partial settings object into a validated copy (not applied yet). Paths, hooks
    /// (Q-01) and updater state are never taken from the patch.
    pub fn merge_settings(&self, patch: serde_json::Value) -> Result<Settings, String> {
        let serde_json::Value::Object(p) = patch else { return Err("bad".into()) };
        let mut cur = serde_json::to_value(&*lock(&self.settings)).map_err(|e| e.to_string())?;
        for (k, v) in p {
            // paths, hooks (Q-01) and updater state are never set through this command
            if matches!(k.as_str(), "download_dir" | "highest_seen" | "last_check" | "update_failures" | "hooks") {
                continue;
            }
            cur[k] = v;
        }
        let mut s: Settings = serde_json::from_value(cur).map_err(|e| e.to_string())?;
        for c in &mut s.categories {
            c.sub = gdm_core::name::sanitize(&c.sub).replace(['\\', '/'], "");
            if c.sub == "_" {
                c.sub.clear();
            }
            c.exts =
                c.exts.iter().map(|e| e.trim().trim_start_matches('.').to_ascii_lowercase()).filter(|e| !e.is_empty() && e.len() <= 16).collect();
        }
        s.max_conns = s.max_conns.clamp(1, 32);
        {
            let old = lock(&self.settings);
            s.hooks = old.hooks.clone();
            // a sound is off, a built-in one, or the WAV already picked in the file dialog (never a path from the UI)
            for (ev, v) in s.sounds.iter_mut() {
                let keep = v.is_empty() || v.strip_prefix("system:").is_some_and(|a| crate::os::SYSTEM_SOUNDS.contains(&a));
                if !keep && old.sounds.get(ev) != Some(v) {
                    *v = old.sounds.get(ev).cloned().unwrap_or_default();
                }
            }
            s.sounds.retain(|k, _| matches!(k.as_str(), "complete" | "failed" | "queue"));
            // queues: the two built-in ones always exist; ids are kept; user queues get q-<n>
            let mut seen = std::collections::HashSet::new();
            s.queues.retain(|q| seen.insert(q.id.clone()) && (q.id == "main" || q.id == "sync" || q.id.starts_with("q-")));
            for d in crate::queue::default_queues() {
                if !s.queues.iter().any(|q| q.id == d.id) {
                    s.queues.push(d);
                }
            }
            for q in &mut s.queues {
                q.max_active = q.max_active.clamp(1, 32);
                q.retries = q.retries.min(1000);
                q.name = q.name.chars().filter(|c| !c.is_control()).take(64).collect();
                if q.id == "main" {
                    q.sync = false;
                }
                if q.id == "sync" {
                    q.sync = true;
                }
                if !matches!(q.on_done.as_str(), "none" | "exit" | "sleep" | "hibernate" | "shutdown") {
                    q.on_done = "none".into();
                }
            }
        }
        Ok(s)
    }

    pub fn apply_limit(&self) {
        let s = lock(&self.settings);
        self.engine.global.set_rate(if s.limit_on { s.limit_kbps * 1024 } else { 0 });
        let mut b = lock(&self.qbuckets);
        for q in &s.queues {
            b.entry(q.id.clone()).or_insert_with(|| limiter::Bucket::new(0)).set_rate(q.speed_kbps * 1024);
        }
    }

    pub fn bump(&self) {
        self.rev.fetch_add(1, Ordering::Relaxed);
    }

    pub fn category_for(&self, name: &str) -> Category {
        let e = ext_of(name);
        let cats = lock(&self.settings).categories.clone();
        cats.iter().find(|c| !e.is_empty() && c.exts.iter().any(|x| x == &e)).cloned().unwrap_or_else(|| cats[0].clone())
    }

    pub fn category_dir(&self, c: &Category) -> PathBuf {
        let base = PathBuf::from(&lock(&self.settings).download_dir);
        if c.sub.is_empty() {
            base
        } else {
            base.join(&c.sub)
        }
    }

    pub fn rows(&self, q: Option<&str>) -> Result<Vec<RowView>, String> {
        let rows = lock(&self.engine.store).list(q, 20_000)?;
        Ok(rows
            .into_iter()
            .map(|row| {
                let c = self.category_for(&row.name);
                RowView { category: c.id, dangerous: gdm_core::name::dangerous(&row.name), ext: ext_of(&row.name), row }
            })
            .collect())
    }

    pub fn live(&self) -> Vec<LiveView> {
        let mut runs = lock(&self.runs);
        runs.iter_mut()
            .map(|(id, c)| {
                let done = c.live.written.load(Ordering::Relaxed);
                let now = Instant::now();
                let dt = now.duration_since(c.last.0).as_secs_f64();
                if dt >= 0.25 {
                    // speed: EMA with a 5 s time constant (UX rule 5)
                    let inst = done.saturating_sub(c.last.1) as f64 / dt;
                    let a = 1.0 - (-dt / 5.0).exp();
                    c.speed = if c.speed == 0.0 { inst } else { c.speed + a * (inst - c.speed) };
                    c.last = (now, done);
                }
                LiveView {
                    id: id.clone(),
                    done,
                    total: c.live.total.load(Ordering::Relaxed),
                    speed: c.speed,
                    conns: c.live.conns.load(Ordering::Relaxed),
                    segs: lock(&c.live.segs).clone(),
                }
            })
            .collect()
    }

    pub fn running(&self) -> usize {
        lock(&self.runs).len()
    }

    pub fn is_running(&self, id: &str) -> bool {
        lock(&self.runs).contains_key(id)
    }

    pub fn start(self: &Arc<Self>, id: &str, decision: Unverified) {
        if self.finishing.load(Ordering::SeqCst) {
            return; // Q-02: a shutdown/sleep is being carried out
        }
        let queue = lock(&self.engine.store).queue_of(id);
        let qb = queue.and_then(|q| lock(&self.qbuckets).get(&q).cloned());
        let mut runs = lock(&self.runs);
        if runs.contains_key(id) {
            return;
        }
        let live = Arc::new(Live::default());
        let job_limit = limiter::Bucket::new(0);
        let limit = Arc::new(limiter::JobLimit { job: job_limit.clone(), queue: qb, global: self.engine.global.clone() });
        let (me, id2, l2) = (self.clone(), id.to_string(), live.clone());
        let handle = tauri::async_runtime::spawn(async move {
            let r = me.engine.run_live(&id2, decision, Some(limit), Some(l2)).await;
            lock(&me.runs).remove(&id2);
            me.finished(&id2, r);
        });
        runs.insert(id.to_string(), RunCtl { handle, live, job_limit, speed: 0.0, last: (Instant::now(), 0) });
        drop(runs);
        self.bump();
    }

    fn finished(&self, id: &str, r: Result<gdm_core::Outcome, Fail>) {
        self.bump();
        let queue = lock(&self.engine.store).queue_of(id);
        if let Some(q) = queue {
            let sync = self.queue_defs().iter().any(|d| d.id == q && d.sync);
            match &r {
                Ok(o) => {
                    lock(&self.qstate).entry(q.clone()).or_default().last_file = Some(o.path.clone());
                    if !sync {
                        let _ = lock(&self.engine.store).set_queue(&[id.to_string()], None);
                        // main queue: done files leave
                    }
                }
                Err(_) => {
                    let failed = self.engine.job(id).map(|j| j.state == "failed").unwrap_or(false);
                    if failed {
                        let _ = lock(&self.engine.store).requeue_failed(id);
                    }
                }
            }
        }
        match &r {
            Ok(_) => self.play("complete"),
            Err(Fail::Io(m)) if m.contains("already running") => {}
            Err(_) => self.play("failed"),
        }
        let toast = match &r {
            Ok(o) => Some((if o.protection == gdm_core::finalize::Protection::Protected { "complete" } else { "unprotected" }, o.name.clone())),
            Err(Fail::Blocked(_)) => self.engine.job(id).ok().map(|j| ("blocked", j.name)),
            Err(Fail::HashMismatch) => self.engine.job(id).ok().map(|j| ("failed", j.name)),
            Err(_) => None,
        };
        lock(&self.pending).retain(|p| p != id || r.is_err());
        if let Some((kind, name)) = toast {
            self.push_toast(kind, id, &name);
        }
    }

    pub fn push_toast(&self, kind: &str, id: &str, name: &str) {
        let t = Toast {
            n: self.toast_n.fetch_add(1, Ordering::Relaxed) + 1,
            kind: kind.into(),
            id: id.into(),
            name: name.into(),
            dangerous: gdm_core::name::dangerous(name),
        };
        // silent mode (P1 #25) hides success only; a blocked file is always shown (plan-ux scenario 4)
        if lock(&self.settings).silent && (kind == "complete" || kind == "unprotected") {
            return;
        }
        if let Some(f) = lock(&self.notify).as_ref() {
            f(&t);
        }
        let mut q = lock(&self.toasts);
        q.push_back(t);
        while q.len() > 20 {
            q.pop_front();
        }
    }

    pub fn play(&self, event: &str) {
        let snd = lock(&self.settings).sounds.get(event).cloned().unwrap_or_default();
        crate::os::play(&snd);
    }

    pub fn toasts_after(&self, n: u64) -> Vec<Toast> {
        lock(&self.toasts).iter().filter(|t| t.n > n).cloned().collect()
    }

    /// pause = cancel the run; the engine claims nothing past the last checkpoint (E-04)
    pub async fn pause(self: &Arc<Self>, id: &str) {
        let c = lock(&self.runs).remove(id);
        if let Some(c) = c {
            c.handle.abort();
            let _ = c.handle.await;
        }
        let _ = self.engine.stopped(id);
        self.bump();
    }

    pub async fn pause_all(self: &Arc<Self>) {
        let ids: Vec<String> = lock(&self.runs).keys().cloned().collect();
        for id in ids {
            self.pause(&id).await;
        }
    }

    pub fn set_conns(&self, id: &str, n: usize) {
        if let Some(c) = lock(&self.runs).get(id) {
            c.live.want_conns.store(n.min(32), Ordering::Relaxed);
        }
    }

    pub fn set_job_limit(&self, id: &str, kbps: u64) {
        if let Some(c) = lock(&self.runs).get(id) {
            c.job_limit.set_rate(kbps * 1024);
        }
    }

    pub async fn remove(self: &Arc<Self>, id: &str, delete_file: bool) -> Result<(), String> {
        self.pause(id).await;
        lock(&self.pending).retain(|p| p != id);
        let r = self.engine.remove(id, delete_file).map_err(|e| e.code());
        self.bump();
        r
    }

    /// add + start (the Download Info flow starts downloading while the dialog is open, like IDM; C-03)
    pub fn add_and_start(
        self: &Arc<Self>,
        nj_url: &str,
        name: Option<String>,
        referrer: Option<String>,
        creds: Vec<(String, String)>,
        typed: bool,
        start: bool,
    ) -> Result<String, String> {
        let u = url::Url::parse(nj_url).map_err(|_| "bad-url".to_string())?;
        let guess = name.clone().unwrap_or_else(|| gdm_core::name::file_name(None, &u));
        let c = self.category_for(&guess);
        let dir = self.category_dir(&c);
        std::fs::create_dir_all(&dir).map_err(|e| format!("dir: {e}"))?;
        let id = self.engine.add(NewJob { url: nj_url.into(), dir, name, referrer, creds, sha256: None, typed }).map_err(|e| e.code())?;
        if start {
            self.start(&id, Unverified::Ask);
        } else {
            let _ = self.engine.stopped(&id);
        }
        self.bump();
        Ok(id)
    }

    /// B-04: at most 30 handoffs a minute start by themselves; the rest wait in the Info dialog unstarted
    pub fn handoff_allowed(&self) -> bool {
        let mut q = lock(&self.handoffs);
        let now = Instant::now();
        while q.front().is_some_and(|t| now.duration_since(*t) > Duration::from_secs(60)) {
            q.pop_front();
        }
        q.push_back(now);
        q.len() <= HANDOFFS_PER_MIN
    }

    /// Download Info [Start] with possibly a new name or category: unchanged => keep downloading;
    /// changed => cancel the prepared job (its .part is deleted) and add it again under the new name/folder
    pub async fn info_confirm(self: &Arc<Self>, id: &str, name: Option<String>, category: Option<String>, later: bool) -> Result<String, String> {
        lock(&self.pending).retain(|p| p != id);
        let j = self.engine.job(id).map_err(|e| e.code())?;
        let cur_cat = self.category_for(&j.name).id;
        let name_changed = name.as_deref().is_some_and(|n| gdm_core::name::sanitize(n) != j.name);
        let cat_changed = category.as_deref().is_some_and(|c| c != cur_cat);
        let mut id = id.to_string();
        if name_changed || cat_changed {
            let new_name = name.unwrap_or(j.name.clone());
            let cats = lock(&self.settings).categories.clone();
            let c = category.and_then(|c| cats.iter().find(|x| x.id == c).cloned()).unwrap_or_else(|| self.category_for(&new_name));
            let dir = self.category_dir(&c);
            std::fs::create_dir_all(&dir).map_err(|e| format!("dir: {e}"))?;
            let creds = self.engine.creds_of(&id); // the browser's cookie follows the renamed job
            self.remove(&id, false).await?;
            id = self
                .engine
                .add(NewJob {
                    url: j.url.clone(),
                    dir,
                    name: Some(new_name),
                    referrer: j.referrer.clone(),
                    creds,
                    sha256: None,
                    typed: j.allow_private.is_some(),
                })
                .map_err(|e| e.code())?;
            if !later {
                self.start(&id, Unverified::Ask);
            }
        } else if later {
            self.pause(&id).await;
        } else if !self.is_running(&id) && !matches!(j.state.as_str(), "complete" | "unprotected") {
            self.start(&id, Unverified::Ask);
        }
        self.bump();
        Ok(id)
    }

    /// [Cancel] in Download Info: stop and delete what was downloaded in advance
    pub async fn info_cancel(self: &Arc<Self>, id: &str) -> Result<(), String> {
        lock(&self.pending).retain(|p| p != id);
        let st = self.engine.job(id).map(|j| j.state).unwrap_or_default();
        if st == "complete" || st == "unprotected" {
            return Ok(()); // already finished while the dialog was open: keep the user's file
        }
        self.remove(id, false).await
    }

    /// jobs a crash or an update interrupted resume by themselves (§07 Installing: resume after restart)
    pub fn resume_interrupted(self: &Arc<Self>) {
        let ids: Vec<String> = lock(&self.engine.store)
            .list(None, 100_000)
            .unwrap_or_default()
            .into_iter()
            .filter(|r| r.state == "downloading" || r.state == "finalizing")
            .map(|r| r.id)
            .collect();
        for id in ids {
            self.start(&id, Unverified::Ask);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// X-02: a made-up id resolves to nothing (commands look paths up by id, never take them from the UI)
    #[test]
    fn x02_fake_job_id_refused_and_settings_survive_damage() {
        let d = std::env::temp_dir().join(format!("gdm-app-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("settings.json"), b"{ not json").unwrap();
        let app = App::open(&d).unwrap();
        assert!(app.engine.job("../../Windows/System32/calc.exe").is_err());
        assert!(app.engine.remove("fake-id", true).is_err());
        assert_eq!(lock(&app.settings).lang, "km", "defaults: Khmer first");
        assert!(
            std::fs::read_dir(&d).unwrap().flatten().any(|e| e.file_name().to_string_lossy().starts_with("settings.json.bad-")),
            "damaged settings kept"
        );
        assert_eq!(app.category_for("setup.EXE").id, "programs");
        assert_eq!(app.category_for("a.tar.gz").id, "compressed");
        assert_eq!(app.category_for("noext").id, "general");
        // defaults that touch privacy or interruption (J2.3, D5, §22): off / Khmer / automatic updates
        {
            let s = lock(&app.settings);
            assert!(!s.video_button && !s.clipboard && !s.autostart && !s.silent && s.show_info && s.catch_enabled);
            assert_eq!(
                (s.update_mode.as_str(), s.max_conns, s.catch_min_kb, s.force_key.as_str(), s.bypass_key.as_str()),
                ("auto", 8, 1024, "ctrl", "alt")
            );
            assert_eq!(s.categories.len(), 6);
        }
        // settings persist across a restart
        lock(&app.settings).lang = "en".into();
        lock(&app.settings).limit_kbps = 321;
        app.save_settings().unwrap();
        drop(app);
        let again = App::open(&d).unwrap();
        let s2 = lock(&again.settings).clone();
        assert_eq!((s2.lang.as_str(), s2.limit_kbps), ("en", 321));
        // Q-01: settings_set never sets a hook, a download path or a sound path
        let p = serde_json::json!({"hooks": {"main": {"program": r"C:\Windows\System32\cmd.exe", "args": ["/c", "calc"]}},
            "download_dir": r"C:\Windows", "sounds": {"complete": r"C:\x\evil.wav", "failed": "system:SystemHand", "queue": "system:SystemAsterisk"},
            "queues": [{"id": "../x", "on_done": "format c:"}, {"id": "main", "on_done": "rm"}]});
        let m = again.merge_settings(p).unwrap();
        assert!(m.hooks.is_empty());
        assert_ne!(m.download_dir, r"C:\Windows");
        assert_eq!(m.sounds.get("complete").map(String::as_str), Some(""), "a WAV path from the UI is dropped");
        assert_eq!(m.sounds.get("failed").map(String::as_str), Some(""), "only the 3 built-in sounds");
        assert_eq!(m.sounds.get("queue").map(String::as_str), Some("system:SystemAsterisk"));
        assert_eq!(m.queues.iter().map(|q| q.id.as_str()).collect::<Vec<_>>(), vec!["main", "sync"]);
        assert_eq!(m.queues[0].on_done, "none");
        drop(again);
        let _ = std::fs::remove_dir_all(&d);
    }
}
