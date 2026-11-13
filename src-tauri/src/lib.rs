// SPDX-License-Identifier: RPL-1.5
//! gdm.exe: Tauri v2 shell around gdm-core (ADR-1, one process). Plugins: updater, single-instance,
//! notification, autostart, window-state only; no shell/fs/http/opener (X-02). Every command takes job IDs or
//! plain values, never paths, URLs to open or command lines (X-01).
pub mod app;
pub mod cli;
pub mod clip;
pub mod components;
pub mod creds;
pub mod hook;
pub mod import;
pub mod media;
pub mod os;
pub mod pipe;
pub mod proxy;
pub mod queue;
pub mod quota;
pub mod refresh;
pub mod update;

use app::{lock, App, RowView, Settings};
use gdm_core::Unverified;
use serde::Serialize;
use serde_json::Value;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, State};

static HANDLE: OnceLock<AppHandle> = OnceLock::new();
static UPDATE: Mutex<update::Status> =
    Mutex::new(update::Status { state: update::State::Idle, version: None, security: false, notes_km: None, notes_en: None, error: None });
#[cfg(not(feature = "store"))]
static STAGED: Mutex<Option<(tauri_plugin_updater::Update, Vec<u8>)>> = Mutex::new(None);

type R<T> = Result<T, String>;
type A<'a> = State<'a, Arc<App>>;

pub fn show_main(_app: &Arc<App>, from_browser: bool) {
    if let Some(w) = HANDLE.get().and_then(|h| h.get_webview_window("main")) {
        let _ = w.show();
        let _ = w.unminimize();
        if from_browser {
            // come over the browser once (C-03), without staying on top
            let _ = w.set_always_on_top(true);
            let _ = w.set_always_on_top(false);
        }
        let _ = w.set_focus();
    }
}

#[derive(Serialize)]
struct Snapshot {
    rev: u64,
    jobs: Vec<RowView>,
}

#[tauri::command]
fn snapshot(app: A, q: Option<String>) -> R<Snapshot> {
    let rev = app.rev.load(std::sync::atomic::Ordering::Relaxed);
    Ok(Snapshot { rev, jobs: app.rows(q.as_deref())? })
}

#[derive(Serialize)]
struct Tick {
    rev: u64,
    live: Vec<app::LiveView>,
    pending: Vec<RowView>,
    toasts: Vec<app::Toast>,
    update: update::Status,
    banner: Option<String>,
    store_build: bool,
}

#[tauri::command]
fn tick(app: A, toast_after: u64) -> R<Tick> {
    let pend = lock(&app.pending).clone();
    let pending = if pend.is_empty() { vec![] } else { app.rows(None)?.into_iter().filter(|r| pend.contains(&r.row.id)).collect() };
    Ok(Tick {
        rev: app.rev.load(std::sync::atomic::Ordering::Relaxed),
        live: app.live(),
        pending,
        toasts: app.toasts_after(toast_after),
        update: lock(&UPDATE).clone(),
        banner: lock(&app.banner).clone(),
        store_build: cfg!(feature = "store"),
    })
}

#[tauri::command]
fn settings_get(app: A) -> Settings {
    lock(&app.settings).clone()
}

/// merge a partial settings object; fields that hold paths or updater state cannot be set from the UI
#[tauri::command]
fn settings_set(app: A, patch: Value) -> R<Settings> {
    let s = app.merge_settings(patch)?;
    apply_settings(app.inner(), s)
}

fn apply_settings(app: &Arc<App>, s: Settings) -> R<Settings> {
    let wake: Vec<queue::Queue> = s.queues.clone();
    let autostart = s.autostart;
    *lock(&app.settings) = s.clone();
    app.save_settings()?;
    app.apply_limit();
    os::sync_wake_tasks(&wake);
    if let Some(h) = HANDLE.get() {
        use tauri_plugin_autostart::ManagerExt;
        let al = h.autolaunch();
        let _ = if autostart { al.enable() } else { al.disable() };
        let _ = rebuild_tray(h, app);
    }
    Ok(s)
}

#[derive(Serialize)]
struct QueueView {
    def: queue::Queue,
    state: queue::QState,
    jobs: Vec<String>,
    hook: Option<hook::Hook>,
}

#[tauri::command]
fn queue_list(app: A) -> Vec<QueueView> {
    let st = app.queue_states();
    let hooks = lock(&app.settings).hooks.clone();
    app.queue_defs()
        .into_iter()
        .map(|d| QueueView {
            jobs: lock(&app.engine.store).queue_jobs(&d.id).unwrap_or_default(),
            state: st.get(&d.id).cloned().unwrap_or_default(),
            hook: hooks.get(&d.id).cloned(),
            def: d,
        })
        .collect()
}

#[tauri::command]
fn queue_put(app: A, ids: Vec<String>, queue: Option<String>) -> R<()> {
    if ids.len() > 100_000 {
        return Err("too-many".into());
    }
    app.queue_put(&ids, queue.as_deref())
}

#[tauri::command]
fn queue_start(app: A, id: String) -> R<()> {
    app.inner().queue_start(&id, None)
}

#[tauri::command]
async fn queue_stop(app: A<'_>, id: String) -> R<()> {
    app.inner().clone().queue_stop(&id).await;
    Ok(())
}

#[tauri::command]
fn queue_cancel_action(app: A, id: String) {
    app.queue_cancel_action(&id);
}

/// Q-01: the ONLY way a hook is set. The program comes from the native file dialog (never a path string from
/// the WebView); the arguments are a list. `pick: false` keeps the program and changes the arguments only.
#[tauri::command]
fn hook_set(app: A, queue: String, pick: bool, args: Vec<String>, clear: bool) -> R<Option<hook::Hook>> {
    // "scan" = the user's own virus scanner, run with {file} after each download (§04 Safety row)
    if queue != "scan" && !app.queue_defs().iter().any(|q| q.id == queue) {
        return Err("no-queue".into());
    }
    if clear {
        lock(&app.settings).hooks.remove(&queue);
        app.save_settings()?;
        return Ok(None);
    }
    let program = if pick {
        let start = std::env::var_os("ProgramFiles").map(std::path::PathBuf::from).unwrap_or_default();
        match os::pick_file(&start, "Program", "*.exe;*.bat;*.cmd") {
            Some(p) => p.display().to_string(),
            None => return Ok(lock(&app.settings).hooks.get(&queue).cloned()),
        }
    } else {
        lock(&app.settings).hooks.get(&queue).map(|h| h.program.clone()).ok_or("no-program")?
    };
    let h = hook::Hook { program, args };
    hook::validate(&h)?;
    // the exact program and argv are confirmed in a native box, never only in the WebView (security review G7)
    let km = lock(&app.settings).lang != "en";
    let ask = if km {
        "ដំណើរការកម្មវិធីនេះពេល queue បញ្ចប់?"
    } else {
        "Run this program when the queue finishes?"
    };
    let args: String = h.args.iter().enumerate().map(|(i, a)| format!("[{}] \"{a}\"\n", i + 1)).collect();
    let shown = format!("{ask}\n\n\"{}\"\n{args}", h.program);
    if !os::confirm("Galaxy Download Manager", &shown) {
        return Ok(lock(&app.settings).hooks.get(&queue).cloned());
    }
    lock(&app.settings).hooks.insert(queue, h.clone());
    app.save_settings()?;
    Ok(Some(h))
}

/// a custom WAV for an event, from the native file dialog only
#[tauri::command]
fn sound_pick(app: A, event: String) -> R<Option<String>> {
    if !matches!(event.as_str(), "complete" | "failed" | "queue") {
        return Err("bad-event".into());
    }
    let start = std::env::var_os("SystemRoot").map(|p| std::path::PathBuf::from(p).join("Media")).unwrap_or_default();
    let Some(p) = os::pick_file(&start, "WAV", "*.wav") else { return Ok(None) };
    let p = p.display().to_string();
    lock(&app.settings).sounds.insert(event, p.clone());
    app.save_settings()?;
    Ok(Some(p))
}

#[tauri::command]
fn sound_test(app: A, event: String) {
    app.play(&event);
}

#[derive(Serialize)]
struct ImportPreview {
    count: usize,
    sample: Vec<import::Item>,
    has_settings: bool,
}

/// Tasks > Import: the file comes from the native dialog; nothing is added before [Add] (I-03 preview)
#[tauri::command]
fn import_pick(app: A) -> R<Option<ImportPreview>> {
    let start = app::downloads_dir();
    let Some(p) = os::pick_file(&start, "IDM .ef2, text, GDM .json", "*.ef2;*.txt;*.json") else { return Ok(None) };
    let len = std::fs::metadata(&p).map_err(|e| e.to_string())?.len();
    if len > import::MAX_FILE {
        return Err("too-large".into());
    }
    let bytes = std::fs::read(&p).map_err(|e| e.to_string())?;
    let (items, set) = import::parse_file(&p.display().to_string(), &bytes)?;
    let pv = ImportPreview { count: items.len(), sample: items.iter().take(50).cloned().collect(), has_settings: set.is_some() };
    *lock(&app.import_pending) = Some((items, set));
    Ok(Some(pv))
}

#[tauri::command]
fn import_commit(app: A, queue: Option<String>, restore_settings: bool) -> R<usize> {
    let Some((items, set)) = lock(&app.import_pending).take() else { return Err("nothing".into()) };
    let ids = app.add_items(&items, queue.as_deref())?;
    if restore_settings {
        if let Some(v) = set {
            apply_settings(app.inner(), app.restore_settings(v)?)?; // low-risk keys only, queue actions disarmed
        }
    }
    Ok(ids.len())
}

#[tauri::command]
fn export_pick(app: A) -> R<Option<String>> {
    let Some(p) = os::pick_save(&app::downloads_dir(), "gdm-export.json") else { return Ok(None) };
    std::fs::write(&p, app.export_json()?).map_err(|e| e.to_string())?;
    Ok(Some(p.display().to_string()))
}

#[derive(Serialize)]
struct BatchPreview {
    count: u64,
    first: Vec<String>,
    needs_confirm: bool,
}

#[tauri::command]
fn batch_preview(b: import::Batch) -> R<BatchPreview> {
    let count = b.count()?;
    Ok(BatchPreview { count, first: b.urls(5)?, needs_confirm: count > import::BATCH_CONFIRM })
}

/// I-04: > 10 000 needs `confirmed`; > 100 000 is refused; one transaction
#[tauri::command]
fn batch_add(app: A, b: import::Batch, confirmed: bool, queue: Option<String>) -> R<usize> {
    let n = b.count()?;
    if n > import::MAX_ITEMS as u64 {
        return Err("too-many".into());
    }
    if n > import::BATCH_CONFIRM && !confirmed {
        return Err("confirm".into());
    }
    let items: Vec<import::Item> = b.urls(n as usize)?.into_iter().map(|url| import::Item { url, ..Default::default() }).collect();
    Ok(app.add_items(&items, queue.as_deref())?.len())
}

// ---------------- site grabber (F09): one project at a time, results stay in the UI until "Add"

static GRAB_STOP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static GRAB_BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static GRAB_PROGRESS: Mutex<Option<gdm_core::grab::Progress>> = Mutex::new(None);
/// the folder chosen with the native picker (X-01: no path ever comes from the WebView)
static GRAB_DIR: Mutex<Option<std::path::PathBuf>> = Mutex::new(None);

fn grab_dir() -> std::path::PathBuf {
    lock(&GRAB_DIR).clone().unwrap_or_else(|| app::downloads_dir().join("Grabber"))
}

/// explore the site; `user`/`password` = optional login, sent as Basic to the start page's origin only (W-01)
#[tauri::command]
async fn grab_run(cfg: gdm_core::grab::Config, user: Option<String>, password: Option<String>) -> R<gdm_core::grab::Report> {
    use std::sync::atomic::Ordering::SeqCst;
    let mut cfg = cfg;
    if cfg.start.len() > gdm_bridge::MAX_URL || [&cfg.avoid, &cfg.domains, &cfg.include, &cfg.exclude, &cfg.types].iter().any(|s| s.len() > 4096) {
        return Err("bad-url".into());
    }
    cfg.typed = true; // the wizard's start page is always typed or pasted by the user
    cfg.creds = match (user.as_deref().map(str::trim).filter(|u| !u.is_empty()), password) {
        (Some(u), p) if u.len() <= 256 && !u.contains(':') && !u.chars().any(char::is_control) => {
            vec![("Authorization".into(), creds::basic(u, &p.unwrap_or_default()))]
        }
        (Some(_), _) => return Err("bad-login".into()),
        _ => vec![],
    };
    if GRAB_BUSY.swap(true, SeqCst) {
        return Err("busy".into());
    }
    GRAB_STOP.store(false, SeqCst);
    let last = Mutex::new(Instant::now() - Duration::from_secs(1));
    let r = gdm_core::grab::crawl(&cfg, &GRAB_STOP, &|p| {
        let mut l = lock(&last);
        if l.elapsed() >= Duration::from_millis(250) {
            *l = Instant::now();
            *lock(&GRAB_PROGRESS) = Some(p.clone());
        }
    })
    .await;
    GRAB_BUSY.store(false, SeqCst);
    *lock(&GRAB_PROGRESS) = None;
    r
}

#[tauri::command]
fn grab_progress() -> Option<gdm_core::grab::Progress> {
    lock(&GRAB_PROGRESS).clone()
}

#[tauri::command]
fn grab_cancel() {
    GRAB_STOP.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// step 2 "save to": native folder picker; returns the folder to show (default Downloads\Grabber)
#[tauri::command]
fn grab_pick_dir(pick: bool) -> R<String> {
    if pick {
        if let Some(p) = os::pick_folder(&grab_dir()) {
            gdm_core::fsx::Dir::open(&p).map_err(|e| e.to_string())?; // no junction on the way (N-03)
            *lock(&GRAB_DIR) = Some(p);
        }
    }
    Ok(grab_dir().display().to_string())
}

/// the ticked files become jobs (paused, like IDM's "add checked files"; `start` = put them in the main
/// queue and start it). `keep` = mirror the site's folders (G-03).
#[tauri::command]
fn grab_add(app: A, files: Vec<gdm_core::grab::Found>, start_url: String, keep: bool, start: bool) -> R<usize> {
    if files.len() > import::MAX_ITEMS || files.iter().any(|f| f.url.len() > gdm_bridge::MAX_URL || f.referrer.len() > gdm_bridge::MAX_URL) {
        return Err("too-many".into());
    }
    let host = url::Url::parse(&start_url).ok().and_then(|u| u.host_str().map(str::to_string));
    let dir = grab_dir();
    if !dir.exists() {
        // the default Downloads\Grabber: created by handle under a checked Downloads (no junction followed)
        let (Some(parent), Some(leaf)) = (dir.parent(), dir.file_name().and_then(|n| n.to_str())) else { return Err("dir".into()) };
        gdm_core::fsx::Dir::open(parent).and_then(|p| p.subdir(leaf)).map_err(|e| e.to_string())?;
    }
    let (jobs, held) = gdm_core::grab::jobs(&files, &dir, keep, host.as_deref()).map_err(|e| e.code())?;
    let ids = app.engine.add_many(jobs, true).map_err(|e| e.code())?;
    drop(held);
    app.bump();
    if start && !ids.is_empty() {
        app.queue_put(&ids, Some("main"))?;
        app.inner().queue_start("main", None)?;
    }
    Ok(ids.len())
}

#[tauri::command]
fn refresh_start(app: A, id: String) -> R<()> {
    app.refresh_start(&id)
}

#[tauri::command]
fn refresh_paste(app: A, id: String, url: String) -> R<()> {
    if url.len() > gdm_bridge::MAX_URL {
        return Err("bad-url".into());
    }
    app.refresh_paste(&id, &url)
}

#[tauri::command]
async fn refresh_candidate(app: A<'_>, id: String) -> R<Option<refresh::Candidate>> {
    app.inner().clone().refresh_candidate(&id).await
}

#[tauri::command]
async fn refresh_decide(app: A<'_>, id: String, choice: String) -> R<String> {
    app.inner().clone().refresh_decide(&id, &choice).await
}

/// proxy login password: straight to Credential Manager ("GDM/proxy"), never into settings
#[tauri::command]
fn proxy_login_set(app: A, host: String, user: String, password: String, windows: bool) -> R<()> {
    if host.trim().is_empty()
        || host.len() > 256
        || user.len() > 256
        || password.len() > 1024
        || format!("{host}{user}").chars().any(|c| c.is_control())
    {
        return Err("bad-login".into());
    }
    let mut p = lock(&app.settings).proxy.clone();
    if windows {
        p.auth = "windows".into(); // the logged-on user's Windows sign-in (SSPI), nothing stored
    } else {
        creds::save("proxy", &user, &password)?;
        p.auth = "basic".into();
    }
    p.user = user;
    p.auth_host = gdm_core::relay::hostport(&host, 80);
    lock(&app.settings).proxy = p.clone();
    app.save_settings()?;
    proxy::apply(&p);
    Ok(())
}

#[tauri::command]
async fn media_probe(app: A<'_>, url: String) -> R<media::Probe> {
    if url.len() > gdm_bridge::MAX_URL {
        return Err("bad-url".into());
    }
    app.inner().clone().media_probe(url.trim()).await
}

#[tauri::command]
fn media_start(app: A, token: String, choice: usize) -> R<()> {
    app.inner().media_start(&token, choice)
}

#[derive(Serialize)]
struct ComponentState {
    ytdlp: bool,
    ffmpeg: bool,
}

#[tauri::command]
fn components_state(app: A) -> ComponentState {
    ComponentState {
        ytdlp: components::has(&app.data, "yt-dlp.exe"),
        ffmpeg: components::has(&app.data, "ffmpeg.exe") && components::has(&app.data, "ffprobe.exe"),
    }
}

/// D10 consent given in the UI: fetch the signed components.json, then each component (sha256-checked)
#[tauri::command]
async fn components_install(app: A<'_>) -> R<()> {
    let a = app.inner().clone();
    let c = gdm_core::net::client(Arc::new(Default::default()), false, Duration::from_secs(60))?;
    let get = |u: String, cap: u64| {
        let c = c.clone();
        async move {
            let mut r = c.get(u).send().await.map_err(|e| e.to_string())?;
            if !r.status().is_success() {
                return Err(format!("http {}", r.status()));
            }
            let mut b = Vec::new();
            while let Some(ch) = r.chunk().await.map_err(|e| e.to_string())? {
                if (b.len() + ch.len()) as u64 > cap {
                    return Err("too-large".to_string());
                }
                b.extend_from_slice(&ch);
            }
            Ok(b)
        }
    };
    let body = get(components::MANIFEST_URL.to_string(), 256 * 1024).await?;
    let sig = get(format!("{}.sig", components::MANIFEST_URL), 4096).await?;
    let m = components::verify(&body, &String::from_utf8_lossy(&sig), update::PUBKEY, gdm_core::store::now())?;
    for comp in &m.components {
        let bytes = get(comp.url.clone(), comp.size.min(components::MAX_DOWNLOAD)).await?;
        let (d, cc) = (a.data.clone(), comp.clone());
        tauri::async_runtime::spawn_blocking(move || components::install(&d, &cc, &bytes)).await.map_err(|e| e.to_string())??;
    }
    Ok(())
}

/// P1 #19 "remember this folder for the category": the folder comes from the native picker only
#[tauri::command]
fn category_pick_dir(app: A, id: String, clear: bool) -> R<Option<String>> {
    let cur = lock(&app.settings).categories.iter().find(|c| c.id == id).cloned().ok_or("no-category")?;
    let dir = if clear {
        String::new()
    } else {
        match os::pick_folder(&app.category_dir(&cur)) {
            Some(p) => {
                gdm_core::fsx::Dir::open(&p).map_err(|e| e.to_string())?; // no junction on the way (N-03)
                p.display().to_string()
            }
            None => return Ok(None),
        }
    };
    for c in lock(&app.settings).categories.iter_mut().filter(|c| c.id == id) {
        c.dir = dir.clone();
    }
    app.save_settings()?;
    Ok(Some(dir))
}

/// P1 #26 Properties: a stopped job downloads from a new address (same checks as a refreshed link: the
/// validator is checked on the next run, a changed file stops with "file changed")
#[tauri::command]
fn job_set_url(app: A, id: String, url: String) -> R<()> {
    if url.len() > gdm_bridge::MAX_URL || app.is_running(&id) {
        return Err("bad-url".into());
    }
    app.engine.refresh_apply(&id, url.trim(), vec![]).map_err(|e| e.code())?;
    app.bump();
    Ok(())
}

/// P1 #26 Properties: another folder for a job that has not downloaded anything yet (native picker)
#[tauri::command]
async fn job_move(app: A<'_>, id: String) -> R<Option<String>> {
    let j = app.engine.job(&id).map_err(|e| e.code())?;
    if j.done > 0 || app.is_running(&id) || matches!(j.state.as_str(), "complete" | "unprotected") {
        return Err("has-progress".into());
    }
    let Some(dir) = os::pick_folder(&j.dir) else { return Ok(None) };
    let creds: Vec<(String, String)> =
        app.engine.creds_of(&id).into_iter().filter(|(k, _)| k.eq_ignore_ascii_case("cookie") || k.eq_ignore_ascii_case("user-agent")).collect();
    app.inner().clone().remove(&id, false).await?;
    let nid = app
        .engine
        .add(gdm_core::NewJob {
            url: j.url,
            dir,
            name: Some(j.name),
            referrer: j.referrer,
            creds,
            typed: j.allow_private.is_some(),
            ..Default::default()
        })
        .map_err(|e| e.code())?;
    let _ = app.engine.stopped(&nid);
    app.bump();
    Ok(Some(nid))
}

#[tauri::command]
fn login_list(app: A) -> Vec<creds::SiteLogin> {
    lock(&app.settings).site_logins.clone()
}

/// P-01: the password goes straight to Credential Manager; it is never returned to the WebView
#[tauri::command]
fn login_set(app: A, prefix: String, user: String, password: String) -> R<creds::SiteLogin> {
    app.login_set(&prefix, &user, &password)
}

#[tauri::command]
fn login_delete(app: A, id: String) -> R<()> {
    app.login_delete(&id)
}

#[tauri::command]
fn pick_download_dir(app: A) -> R<Option<String>> {
    let cur = std::path::PathBuf::from(&lock(&app.settings).download_dir);
    let Some(p) = os::pick_folder(&cur) else { return Ok(None) };
    lock(&app.settings).download_dir = p.display().to_string();
    app.save_settings()?;
    Ok(Some(p.display().to_string()))
}

#[derive(Serialize)]
struct FolderCheck {
    fs: String,
    free: u64,
    dir: String,
}

#[tauri::command]
fn folder_check(app: A) -> FolderCheck {
    let dir = lock(&app.settings).download_dir.clone();
    let (fs, free) = os::folder_check(std::path::Path::new(&dir));
    FolderCheck { fs, free, dir }
}

/// [+] / paste / drop: the user typed this URL (W-02: a LAN host is fine without asking)
#[tauri::command]
fn add_url(app: A, url: String) -> R<String> {
    let url = url.trim().to_string();
    if url.len() > gdm_bridge::MAX_URL {
        return Err("bad-url".into());
    }
    let id = app.add_and_start(&url, None, None, vec![], true, true)?;
    lock(&app.pending).push(id.clone());
    Ok(id)
}

#[tauri::command]
async fn info_confirm(app: A<'_>, id: String, name: Option<String>, category: Option<String>, later: bool) -> R<String> {
    app.inner().clone().info_confirm(&id, name, category, later).await
}

#[tauri::command]
async fn info_cancel(app: A<'_>, id: String) -> R<()> {
    app.inner().clone().info_cancel(&id).await
}

#[tauri::command]
fn job_start(app: A, id: String, decision: Option<String>) -> R<()> {
    let d = match decision.as_deref() {
        Some("restart") => Unverified::Restart,
        Some("continue") => Unverified::Continue,
        _ => Unverified::Ask,
    };
    app.engine.job(&id).map_err(|e| e.code())?;
    app.inner().start(&id, d);
    Ok(())
}

#[tauri::command]
async fn job_pause(app: A<'_>, id: String) -> R<()> {
    app.inner().clone().pause(&id).await;
    Ok(())
}

#[tauri::command]
async fn job_remove(app: A<'_>, id: String, delete_file: bool) -> R<()> {
    app.inner().clone().remove(&id, delete_file).await
}

fn finished_path(app: &App, id: &str) -> R<(std::path::PathBuf, gdm_core::store::Job)> {
    let j = app.engine.job(id).map_err(|e| e.code())?;
    Ok((j.dir.join(&j.name), j))
}

/// X-02: the path comes from the database; only a finished, protected, non-dangerous file opens directly
#[tauri::command]
fn job_open(app: A, id: String) -> R<()> {
    let (p, j) = finished_path(&app, &id)?;
    if j.state != "complete" {
        return Err("not-openable".into());
    }
    os::open_file(&p, &j.name)
}

#[tauri::command]
fn job_folder(app: A, id: String) -> R<()> {
    let (p, j) = finished_path(&app, &id)?;
    let target = if matches!(j.state.as_str(), "complete" | "unprotected") { p } else { j.dir.clone() };
    os::open_folder(&target)
}

/// X-02: the page comes from the job's own referrer (or its URL), looked up by id
#[tauri::command]
fn job_page(app: A, id: String) -> R<()> {
    let j = app.engine.job(&id).map_err(|e| e.code())?;
    os::open_web(j.referrer.as_deref().filter(|r| r.starts_with("http")).unwrap_or(&j.url))
}

#[tauri::command]
fn job_desc(app: A, id: String, text: String) -> R<()> {
    let t: String = text.chars().take(2000).collect();
    lock(&app.engine.store).set_description(&id, (!t.is_empty()).then_some(t.as_str()))?;
    app.bump();
    Ok(())
}

#[tauri::command]
fn job_conns(app: A, id: String, n: usize) {
    app.set_conns(&id, n);
}

#[tauri::command]
fn job_limit(app: A, id: String, kbps: u64) {
    app.set_job_limit(&id, kbps);
}

/// W-02: the user confirmed "download from the local network?" for this job's host
#[tauri::command]
fn job_allow_private(app: A, id: String) -> R<()> {
    let j = app.engine.job(&id).map_err(|e| e.code())?;
    let host = url::Url::parse(&j.url).ok().and_then(|u| u.host_str().map(String::from)).ok_or("bad-url")?;
    app.engine.allow_private(&id, &host).map_err(|e| e.code())?;
    app.inner().start(&id, Unverified::Ask);
    Ok(())
}

#[derive(Serialize)]
struct Detail {
    url: String,
    final_url: Option<String>,
    referrer: Option<String>,
    dir: String,
    sha256: Option<String>,
    segments: Vec<[u64; 3]>,
    history: Vec<(String, Option<String>)>,
}

#[tauri::command]
fn job_detail(app: A, id: String) -> R<Detail> {
    let j = app.engine.job(&id).map_err(|e| e.code())?;
    let st = lock(&app.engine.store);
    Ok(Detail {
        url: j.url,
        final_url: j.final_url,
        referrer: j.referrer,
        dir: j.dir.display().to_string(),
        sha256: j.sha256,
        segments: st.segments(&id)?,
        history: st.history(&id)?,
    })
}

#[tauri::command]
async fn pause_all(app: A<'_>) -> R<()> {
    app.inner().clone().pause_all().await;
    Ok(())
}

#[tauri::command]
fn resume_all(app: A) -> R<()> {
    for r in app.rows(None)? {
        if r.row.state == "paused" && r.row.error_code.is_none() {
            app.inner().start(&r.row.id, Unverified::Ask);
        }
    }
    Ok(())
}

#[tauri::command]
fn open_link(key: String) -> R<()> {
    os::open_link(&key)
}

#[derive(Serialize)]
struct Browser {
    key: &'static str,
    name: &'static str,
    installed: bool,
    connected: bool,
}

#[tauri::command]
fn browsers(app: A) -> Vec<Browser> {
    let seen = lock(&app.browsers_seen).clone();
    os::browsers().into_iter().map(|(key, name, installed)| Browser { key, name, installed, connected: seen.contains_key(key) }).collect()
}

#[tauri::command]
async fn update_check(app: A<'_>) -> R<()> {
    #[cfg(not(feature = "store"))]
    if let Some(h) = HANDLE.get() {
        check_update(h, app.inner(), true).await;
    }
    let _ = app;
    Ok(())
}

#[tauri::command]
async fn update_restart(app: A<'_>) -> R<()> {
    #[cfg(not(feature = "store"))]
    install_update(app.inner()).await;
    let _ = app;
    Ok(())
}

fn set_update(f: impl FnOnce(&mut update::Status)) {
    f(&mut lock(&UPDATE));
}

#[cfg(not(feature = "store"))]
static CHECKING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// single flight (council G6): two overlapping checks could stage a lower version after a higher one
#[cfg(not(feature = "store"))]
async fn check_update(h: &AppHandle, app: &Arc<App>, manual: bool) {
    use std::sync::atomic::Ordering::SeqCst;
    if CHECKING.swap(true, SeqCst) {
        return;
    }
    check_update_inner(h, app, manual).await;
    CHECKING.store(false, SeqCst);
}

#[cfg(not(feature = "store"))]
async fn check_update_inner(h: &AppHandle, app: &Arc<App>, manual: bool) {
    use tauri_plugin_updater::UpdaterExt;
    let (mode, channel, seen) = {
        let s = lock(&app.settings);
        (s.update_mode.clone(), s.update_channel.clone(), s.highest_seen.clone())
    };
    if (mode == "off" && !manual) || matches!(lock(&UPDATE).state, update::State::Downloading | update::State::Ready | update::State::Installing) {
        return;
    }
    set_update(|u| {
        u.state = update::State::Checking;
        u.error = None;
    });
    lock(&app.settings).last_check = gdm_core::store::now();
    let _ = app.save_settings();
    let fetch = async {
        let c = gdm_core::net::client(Arc::new(Default::default()), false, Duration::from_secs(30))?;
        let get = |u: String| {
            let c = c.clone();
            async move {
                let mut r = c.get(u).send().await.map_err(|e| e.to_string())?;
                if !r.status().is_success() {
                    return Err(format!("http {}", r.status()));
                }
                // capped while streaming, not after buffering
                let mut b = Vec::new();
                while let Some(chunk) = r.chunk().await.map_err(|e| e.to_string())? {
                    if b.len() + chunk.len() > 256 * 1024 {
                        return Err("manifest too large".to_string());
                    }
                    b.extend_from_slice(&chunk);
                }
                Ok(b)
            }
        };
        let body = get(update::MANIFEST_URL.to_string()).await?;
        let sig = get(format!("{}.sig", update::MANIFEST_URL)).await?;
        Ok::<_, String>((body, String::from_utf8_lossy(&sig).to_string()))
    };
    let (body, sig) = match fetch.await {
        Ok(v) => v,
        Err(e) => return set_update(|u| (u.state, u.error) = (update::State::Idle, Some(e))),
    };
    let m = match update::verify_manifest(&body, &sig, update::PUBKEY, gdm_core::store::now(), &channel, &seen) {
        Ok(m) => m,
        Err(update::Reject::NotNewer) => return set_update(|u| u.state = update::State::Idle),
        Err(e) => return set_update(|u| (u.state, u.error) = (update::State::Idle, Some(format!("refused: {e:?}")))),
    };
    set_update(|u| {
        u.version = Some(m.version.clone());
        u.security = m.security;
        u.notes_km = m.notes_km.clone();
        u.notes_en = m.notes_en.clone();
        u.state = if mode == "notify" && !manual { update::State::Available } else { update::State::Downloading };
    });
    if mode == "notify" && !manual {
        return;
    }
    let res = async {
        let up = h.updater().map_err(|e| e.to_string())?.check().await.map_err(|e| e.to_string())?.ok_or("no update")?;
        if !m.binds("windows-x86_64", &up.version, up.download_url.as_str(), &up.signature) {
            return Err(format!("refused: the updater's {} is not the installer the signed manifest names ({})", up.version, m.version));
        }
        set_update(|u| u.state = update::State::Verifying);
        let bytes = up.download(|_, _| {}, || {}).await.map_err(|e| e.to_string())?; // minisign checked by Tauri
        Ok::<_, String>((up, bytes))
    };
    match res.await {
        Ok(staged) => {
            *lock(&STAGED) = Some(staged);
            {
                let mut st = lock(&app.settings);
                if update::vkey(&m.version) > update::vkey(&st.highest_seen) {
                    st.highest_seen = m.version.clone(); // never lowered
                }
            }
            let _ = app.save_settings();
            set_update(|u| u.state = update::State::Ready);
        }
        Err(e) => set_update(|u| (u.state, u.error) = (update::State::Idle, Some(e))),
    }
}

/// U-02: pause every job (each keeps its last checkpoint), then hand over to the NSIS installer (passive);
/// Tauri exits the process. On failure the old version keeps running with a banner (§07 Failed install).
#[cfg(not(feature = "store"))]
async fn install_update(app: &Arc<App>) {
    let Some((up, bytes)) = lock(&STAGED).take() else { return };
    set_update(|u| u.state = update::State::Installing);
    app.pause_all().await;
    let _ = up; // Tauri verified the minisign signature of `bytes`; GDM runs them itself (U-06)
    let res: Result<(), String> = update::stage_and_hold(&app.data.join("updates"), &bytes).and_then(|(held, path)| {
        update::launch_held(&path)?;
        drop(held); // the installer has opened its own image (share READ) before we let go
        std::process::exit(0)
    });
    if let Err(e) = res {
        let n = {
            let mut s = lock(&app.settings);
            s.update_failures += 1;
            s.update_failures
        };
        let _ = app.save_settings();
        set_update(|u| (u.state, u.error) = (update::State::Failed, Some(format!("{e} ({n})"))));
    }
}

fn tray_labels(lang: &str) -> [&'static str; 5] {
    if lang == "en" {
        ["Open GDM", "Pause all", "Speed limit on", "Speed limit off", "Quit"]
    } else {
        ["បើក GDM", "ផ្អាកទាំងអស់", "បើកការកំណត់ល្បឿន", "បិទការកំណត់ល្បឿន", "ចាកចេញ"]
    }
}

fn rebuild_tray(h: &AppHandle, app: &App) -> tauri::Result<()> {
    let (lang, on) = {
        let s = lock(&app.settings);
        (s.lang.clone(), s.limit_on)
    };
    let l = tray_labels(&lang);
    let menu = Menu::with_items(
        h,
        &[
            &MenuItem::with_id(h, "open", l[0], true, None::<&str>)?,
            &MenuItem::with_id(h, "pause", l[1], true, None::<&str>)?,
            &MenuItem::with_id(h, "limit", if on { l[3] } else { l[2] }, true, None::<&str>)?,
            &MenuItem::with_id(h, "quit", l[4], true, None::<&str>)?,
        ],
    )?;
    if let Some(t) = h.tray_by_id("gdm") {
        t.set_menu(Some(menu))?;
        // I-01: the tray says when the clipboard is being watched
        let tip = match (lock(&app.settings).clipboard, lang == "en") {
            (true, true) => "Galaxy Download Manager (watching the clipboard)",
            (true, false) => "Galaxy Download Manager (កំពុងមើល clipboard)",
            _ => "Galaxy Download Manager",
        };
        t.set_tooltip(Some(tip))?;
    }
    Ok(())
}

/// the main window's behaviour: closing hides it (§07); a WebView2 process failure rebuilds it (council G7,
/// ADR-1: the engine keeps downloading through a WebView crash, the window must come back too)
fn attach_main(h: &AppHandle, w: &tauri::WebviewWindow) {
    let w2 = w.clone();
    w.on_window_event(move |e| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = e {
            api.prevent_close(); // closing the window is not quitting (§07): GDM stays in the tray
            let _ = w2.hide();
        }
    });
    let h2 = h.clone();
    let _ = w.with_webview(move |pw| unsafe {
        use webview2_com::ProcessFailedEventHandler;
        let Ok(core) = pw.controller().CoreWebView2() else { return };
        let mut token = 0i64;
        let h3 = h2.clone();
        let _ = core.add_ProcessFailed(
            &ProcessFailedEventHandler::create(Box::new(move |_, _| {
                let h4 = h3.clone();
                let _ = h3.run_on_main_thread(move || rebuild_main(&h4));
                Ok(())
            })),
            &mut token,
        );
    });
}

fn wv_log(h: &AppHandle, m: &str) {
    if let Some(a) = h.try_state::<Arc<App>>() {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(a.data.join("webview.log")) {
            let _ = writeln!(f, "{} {m}", gdm_core::store::now());
        }
    }
}

static REBUILDS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// a new main window from the same config; it reads all state from the engine again (no job is touched)
fn rebuild_main(h: &AppHandle) {
    use std::sync::atomic::Ordering;
    if REBUILDS.fetch_add(1, Ordering::SeqCst) > 20 {
        return; // a WebView that keeps dying: stop trying (downloads continue; the tray still works)
    }
    let visible = h.get_webview_window("main").and_then(|w| w.is_visible().ok()).unwrap_or(false);
    wv_log(h, "process failed: rebuilding the main window");
    if let Some(w) = h.get_webview_window("main") {
        let _ = w.destroy();
    }
    // the label is free only once the old window is gone (destroy is asynchronous)
    let h2 = h.clone();
    std::thread::spawn(move || {
        for _ in 0..50 {
            if h2.get_webview_window("main").is_none() {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let h3 = h2.clone();
        let _ = h2.run_on_main_thread(move || {
            let Some(cfg) = h3.config().app.windows.iter().find(|c| c.label == "main").cloned() else { return };
            match tauri::WebviewWindowBuilder::from_config(&h3, &cfg).and_then(|b| b.build()) {
                Ok(w) => {
                    attach_main(&h3, &w);
                    if visible {
                        let _ = w.show();
                    }
                    wv_log(&h3, "main window rebuilt");
                }
                Err(e) => wv_log(&h3, &format!("rebuild failed: {e}")),
            }
        });
    });
}

/// "exit GDM when the queue is done" and the CLI /q
pub fn request_quit() {
    if let Some(h) = HANDLE.get() {
        if let Some(a) = h.try_state::<Arc<App>>() {
            quit(h, a.inner().clone());
        }
    }
}

fn quit(h: &AppHandle, app: Arc<App>) {
    let h = h.clone();
    tauri::async_runtime::spawn(async move {
        #[cfg(not(feature = "store"))]
        if lock(&STAGED).is_some() {
            install_update(&app).await; // §07: "on quit" installs a staged update
        }
        app.pause_all().await;
        h.exit(0);
    });
}

fn notifier(h: AppHandle, app: &Arc<App>) {
    let a2 = Arc::downgrade(app);
    *lock(&app.notify) = Some(Box::new(move |t: &app::Toast| {
        use tauri_plugin_notification::NotificationExt;
        let visible = h.get_webview_window("main").and_then(|w| w.is_visible().ok()).unwrap_or(false);
        if visible {
            return; // the in-app toast shows it
        }
        let km = a2.upgrade().map(|a| lock(&a.settings).lang != "en").unwrap_or(true);
        let title = match (t.kind.as_str(), km) {
            ("blocked", true) => "ត្រូវ antivirus រារាំង",
            ("blocked", false) => "Blocked by antivirus",
            ("failed", true) => "ការផ្ទៀងផ្ទាត់មិនត្រូវ",
            ("failed", false) => "Verification failed",
            (_, true) => "បានទាញ",
            (_, false) => "Downloaded",
        };
        let _ = h.notification().builder().title(title).body(&t.name).show();
    }));
}

pub fn run() {
    // I-02: `gdm.exe /d URL ...` is a pipe client of the running GDM (started in the tray if needed), then exits
    let argv: Vec<String> = std::env::args().collect();
    if let Some(m) = cli::parse(&argv) {
        std::process::exit(cli::main(m));
    }
    let background = argv.iter().any(|a| a == "--background");
    let app = match App::open(&app::data_dir()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("GDM cannot open its data folder: {e}");
            std::process::exit(1);
        }
    };
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|h, _args, _cwd| {
            if let Some(a) = h.try_state::<Arc<App>>() {
                show_main(a.inner(), false);
            }
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--background"])));
    #[cfg(not(feature = "store"))]
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    builder
        .manage(app.clone())
        .invoke_handler(tauri::generate_handler![
            snapshot,
            tick,
            settings_get,
            settings_set,
            pick_download_dir,
            folder_check,
            add_url,
            info_confirm,
            info_cancel,
            job_start,
            job_pause,
            job_remove,
            job_open,
            job_folder,
            job_page,
            job_desc,
            job_conns,
            job_limit,
            job_allow_private,
            job_detail,
            pause_all,
            resume_all,
            open_link,
            browsers,
            update_check,
            update_restart,
            queue_list,
            queue_put,
            queue_start,
            queue_stop,
            queue_cancel_action,
            hook_set,
            sound_pick,
            sound_test,
            import_pick,
            import_commit,
            export_pick,
            batch_preview,
            batch_add,
            grab_run,
            grab_progress,
            grab_cancel,
            grab_pick_dir,
            grab_add,
            refresh_start,
            refresh_paste,
            refresh_candidate,
            refresh_decide,
            login_list,
            login_set,
            login_delete,
            proxy_login_set,
            media_probe,
            media_start,
            components_state,
            components_install,
            category_pick_dir,
            job_set_url,
            job_move
        ])
        .setup(move |tapp| {
            let h = tapp.handle().clone();
            let _ = HANDLE.set(h.clone());
            notifier(h.clone(), &app);
            if let Err(e) = pipe::serve(app.clone(), tauri::async_runtime::handle()) {
                *lock(&app.banner) = Some(e); // B-02: never serve on a pipe someone else owns
            }
            let tray_menu = Menu::with_items(&h, &[&MenuItem::with_id(&h, "open", "GDM", true, None::<&str>)?])?;
            let a2 = app.clone();
            TrayIconBuilder::with_id("gdm")
                .icon(tapp.default_window_icon().cloned().ok_or("no icon")?)
                .tooltip("Galaxy Download Manager")
                .menu(&tray_menu)
                .on_menu_event(move |h, ev| match ev.id().as_ref() {
                    "open" => show_main(&a2, false),
                    "pause" => {
                        let a = a2.clone();
                        tauri::async_runtime::spawn(async move { a.pause_all().await });
                    }
                    "limit" => {
                        {
                            let mut s = lock(&a2.settings);
                            s.limit_on = !s.limit_on;
                            if s.limit_on && s.limit_kbps == 0 {
                                s.limit_kbps = 512;
                            }
                        }
                        let _ = a2.save_settings();
                        a2.apply_limit();
                        let _ = rebuild_tray(h, &a2);
                    }
                    "quit" => quit(h, a2.clone()),
                    _ => {}
                })
                .build(&h)?;
            rebuild_tray(&h, &app)?;
            if let Some(w) = h.get_webview_window("main") {
                if !background {
                    let _ = w.show();
                }
                attach_main(&h, &w);
            }
            app.resume_interrupted();
            // I-01: clipboard monitor (reads nothing while off); a URL opens the Download Info dialog, unstarted
            let (ac, ao) = (Arc::downgrade(&app), Arc::downgrade(&app));
            clip::spawn(
                Box::new(move || {
                    ac.upgrade().and_then(|a| {
                        let video = components::has(&a.data, "yt-dlp.exe");
                        let s = lock(&a.settings);
                        s.clipboard.then(|| (s.catch_exts.clone(), video))
                    })
                }),
                Box::new(move |u, is_video| {
                    if let Some(a) = ao.upgrade() {
                        if is_video {
                            // a copied video page: ask in the app (quality flow), never start by itself
                            a.push_toast("video-link", "", &u);
                            return;
                        }
                        if let Ok(id) = a.add_and_start(&u, None, None, vec![], false, false) {
                            lock(&a.pending).push(id);
                            show_main(&a, true);
                        }
                    }
                }),
            );
            // queue scheduler: every second (schedules use wall time, countdowns use Instant)
            let aq = app.clone();
            tauri::async_runtime::spawn(async move {
                let mut last = chrono::Utc::now() - chrono::Duration::seconds(90);
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    let now = chrono::Utc::now();
                    aq.quota_tick().await;
                    aq.queue_tick(now, last).await;
                    last = now;
                }
            });
            // backups + updater timer (§07: daily, at start after a minute, never more often than 6 h)
            let a3 = app.clone();
            tauri::async_runtime::spawn(async move {
                let mut idle_since: Option<Instant> = None;
                let t0 = Instant::now();
                loop {
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    let _ = lock(&a3.engine.store).backup_if_due();
                    #[cfg(not(feature = "store"))]
                    {
                        let (mode, last) = {
                            let s = lock(&a3.settings);
                            (s.update_mode.clone(), s.last_check)
                        };
                        let age = gdm_core::store::now() - last;
                        if mode != "off" && t0.elapsed() > Duration::from_secs(60) && age > 6 * 3600 {
                            check_update(&h, &a3, false).await;
                        }
                        idle_since = if a3.running() == 0 { idle_since.or(Some(Instant::now())) } else { None };
                        let ready = lock(&UPDATE).state == update::State::Ready;
                        let failures = lock(&a3.settings).update_failures;
                        let state = if ready { update::State::Ready } else { update::State::Idle };
                        if update::may_install(&state, &mode, failures, a3.running(), idle_since.map(|t| t.elapsed())) {
                            install_update(&a3).await;
                        }
                    }
                    #[cfg(feature = "store")]
                    {
                        let _ = (&h, &mut idle_since, t0);
                        set_update(|u| u.state = update::State::Store);
                    }
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running GDM");
}
