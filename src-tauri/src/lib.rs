// SPDX-License-Identifier: RPL-1.5
//! gdm.exe: Tauri v2 shell around gdm-core (ADR-1, one process). Plugins: updater, single-instance,
//! notification, autostart, window-state only; no shell/fs/http/opener (X-02). Every command takes job IDs or
//! plain values, never paths, URLs to open or command lines (X-01).
pub mod app;
pub mod os;
pub mod pipe;
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
    let Value::Object(p) = patch else { return Err("bad".into()) };
    let mut cur = serde_json::to_value(&*lock(&app.settings)).map_err(|e| e.to_string())?;
    for (k, v) in p {
        if matches!(k.as_str(), "download_dir" | "highest_seen" | "last_check" | "update_failures") {
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
        c.exts = c.exts.iter().map(|e| e.trim().trim_start_matches('.').to_ascii_lowercase()).filter(|e| !e.is_empty() && e.len() <= 16).collect();
    }
    s.max_conns = s.max_conns.clamp(1, 32);
    let autostart = s.autostart;
    *lock(&app.settings) = s.clone();
    app.save_settings()?;
    app.apply_limit();
    if let Some(h) = HANDLE.get() {
        use tauri_plugin_autostart::ManagerExt;
        let al = h.autolaunch();
        let _ = if autostart { al.enable() } else { al.disable() };
        let _ = rebuild_tray(h, &app);
    }
    Ok(s)
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
async fn check_update(h: &AppHandle, app: &Arc<App>, manual: bool) {
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
                let r = c.get(u).send().await.map_err(|e| e.to_string())?;
                if !r.status().is_success() {
                    return Err(format!("http {}", r.status()));
                }
                let b = r.bytes().await.map_err(|e| e.to_string())?;
                if b.len() > 256 * 1024 {
                    return Err("manifest too large".to_string());
                }
                Ok(b.to_vec())
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
        if up.version != m.version {
            return Err(format!("updater offered {} but the signed manifest says {}", up.version, m.version));
        }
        set_update(|u| u.state = update::State::Verifying);
        let bytes = up.download(|_, _| {}, || {}).await.map_err(|e| e.to_string())?; // minisign checked by Tauri
        Ok::<_, String>((up, bytes))
    };
    match res.await {
        Ok(staged) => {
            *lock(&STAGED) = Some(staged);
            lock(&app.settings).highest_seen = m.version.clone();
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
    if let Err(e) = up.install(bytes) {
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
    }
    Ok(())
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
    let background = std::env::args().any(|a| a == "--background");
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
            update_restart
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
                let w2 = w.clone();
                w.on_window_event(move |e| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = e {
                        api.prevent_close(); // closing the window is not quitting (§07): GDM stays in the tray
                        let _ = w2.hide();
                    }
                });
            }
            app.resume_interrupted();
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
                        if ready && mode == "auto" && failures < 2 && idle_since.is_some_and(|t| t.elapsed() > Duration::from_secs(120)) {
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
