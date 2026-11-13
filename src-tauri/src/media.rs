// SPDX-License-Identifier: RPL-1.5
//! Video (F07, ADR-4/ADR-6, D10). Paste a page address => yt-dlp runs extract-only (`-J`) through GDM's policy
//! relay with no cookie or credential (M-08) => the quality popup => GDM's own engine downloads the chosen
//! streams (http) or GDM's HLS reader fetches the segments => ffmpeg muxes the LOCAL files only
//! (`-protocol_whitelist file,pipe`) => ffprobe confirms video + audio (M-06) => the final name goes through the
//! media extension allowlist (M-02) and the sanitizer, then Mark-of-the-Web and the antivirus (N-05).
//! DRM => a clear refusal, never a workaround (M-04, D10). Live streams: not in v1.0 (clear message).
use crate::app::{lock, App};
use gdm_core::relay::{Relay, Route};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

#[derive(Serialize, Clone, Debug)]
pub struct ChoiceView {
    pub label: String,
    pub height: Option<u32>,
    pub container: String,
    pub est_size: Option<u64>,
}

#[derive(Serialize, Clone, Debug)]
pub struct Probe {
    pub token: String,
    pub title: String,
    pub choices: Vec<ChoiceView>,
}

/// requests/tunnels the last yt-dlp run's relay passed on (E2E evidence for ADR-6)
pub static LAST_RELAY_SERVED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub struct Pending {
    pub page: String,
    pub info: gdm_media::Info,
    pub choices: Vec<gdm_media::Choice>,
}

fn work_dir(data: &Path) -> Result<PathBuf, String> {
    let d = data
        .join("media-work")
        .join(format!("{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()));
    std::fs::create_dir_all(&d).map_err(|e| e.to_string())?;
    Ok(d)
}

/// run a held component with a minimal environment in `cwd`; stdout capped at `cap` bytes
pub async fn run_component(data: &Path, exe: &str, args: &[String], cwd: &Path, cap: usize, timeout: Duration) -> Result<Vec<u8>, String> {
    use tokio::io::AsyncReadExt;
    let (held, path) = crate::components::hold(data, exe)?;
    let mut c = tokio::process::Command::new(&path);
    c.args(args)
        .current_dir(cwd)
        .env_clear()
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    for k in ["SystemRoot", "SystemDrive", "WINDIR", "NUMBER_OF_PROCESSORS", "PROCESSOR_ARCHITECTURE"] {
        if let Some(v) = std::env::var_os(k) {
            c.env(k, v);
        }
    }
    // yt-dlp's config/cache lookups land in the empty work folder (also --ignore-config / --no-cache-dir)
    c.env("TEMP", cwd).env("TMP", cwd).env("APPDATA", cwd).env("USERPROFILE", cwd).env("HOME", cwd);
    #[cfg(windows)]
    c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    let mut child = c.kill_on_drop(true).spawn().map_err(|e| e.to_string())?;
    drop(held); // the process has its own image open now (M-05)
    let mut out = child.stdout.take().ok_or("no stdout")?;
    let mut err = child.stderr.take().ok_or("no stderr")?;
    let err_task = tokio::spawn(async move {
        let mut e = Vec::new();
        let mut c = [0u8; 4096];
        while let Ok(n) = err.read(&mut c).await {
            if n == 0 {
                break;
            }
            if e.len() < 16 * 1024 {
                e.extend_from_slice(&c[..n]);
            }
        }
        e
    });
    let read = async {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 65536];
        loop {
            let n = out.read(&mut chunk).await.map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            if buf.len() + n > cap {
                return Err("output-too-large".to_string());
            }
            buf.extend_from_slice(&chunk[..n]);
        }
        let st = child.wait().await.map_err(|e| e.to_string())?;
        if !st.success() {
            let e = err_task.await.unwrap_or_default();
            let last = String::from_utf8_lossy(&e).lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("").chars().take(300).collect::<String>();
            return Err(format!("{exe} failed: {last}"));
        }
        Ok(buf)
    };
    tokio::time::timeout(timeout, read).await.map_err(|_| format!("{exe} timeout"))?
}

impl App {
    /// a policy relay for one yt-dlp run: the user's proxy routes (or direct), no private destinations
    async fn ytdlp_relay(&self, page_host: &str) -> Result<(Arc<Relay>, Arc<gdm_core::net::Policy>, String), String> {
        let p = lock(&self.settings).proxy.clone();
        let router = crate::proxy::router(&p).unwrap_or_else(|| Arc::new(|_: &url::Url| vec![Route::Direct]));
        let r = Relay::start(router).await.map_err(|e| e.to_string())?;
        // W-02: the user typed this page, so its own host may be on the LAN (a NAS); nothing else private
        let policy = Arc::new(gdm_core::net::Policy { allow_private: vec![page_host.to_string()] });
        let (u, pw) = r.register(&policy);
        let proxy = format!("http://{u}:{pw}@{}", r.addr);
        Ok((r, policy, proxy))
    }

    /// paste a page address: extract-only yt-dlp => the quality list (or a clear refusal)
    pub async fn media_probe(self: &Arc<Self>, page: &str) -> Result<Probe, String> {
        if !crate::components::has(&self.data, &crate::components::exe("yt-dlp")) {
            return Err("need-component".into());
        }
        let host = url::Url::parse(page).ok().and_then(|u| u.host_str().map(String::from)).ok_or("bad-url")?;
        let (relay, _policy, proxy) = self.ytdlp_relay(&host).await?;
        let args = gdm_media::ytdlp_args(page, &proxy)?;
        let work = work_dir(&self.data)?;
        let out = run_component(&self.data, &crate::components::exe("yt-dlp"), &args, &work, gdm_media::MAX_JSON, Duration::from_secs(120)).await;
        let _ = std::fs::remove_dir_all(&work);
        LAST_RELAY_SERVED.store(relay.served.load(std::sync::atomic::Ordering::Relaxed), std::sync::atomic::Ordering::Relaxed);
        drop(relay);
        let info = gdm_media::parse_info(&String::from_utf8_lossy(&out?))?;
        if info.drm {
            return Err("drm".into()); // M-04, D10: no DRM, ever
        }
        if info.is_live {
            return Err("live".into());
        }
        let choices = gdm_media::choices(&info);
        if choices.is_empty() {
            return Err("no-formats".into());
        }
        let token = format!("{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
        let p = Probe {
            token: token.clone(),
            title: info.title.clone(),
            choices: choices
                .iter()
                .map(|c| ChoiceView { label: c.label.clone(), height: c.height, container: c.container.clone(), est_size: c.est_size })
                .collect(),
        };
        lock(&self.media).insert(token, Pending { page: page.to_string(), info, choices });
        Ok(p)
    }

    /// the user picked a quality: download the streams, mux, check, publish (in the background)
    pub fn media_start(self: &Arc<Self>, token: &str, choice: usize) -> Result<(), String> {
        let p = lock(&self.media).remove(token).ok_or("no-probe")?;
        let c = p.choices.get(choice).cloned().ok_or("bad-choice")?;
        let me = self.clone();
        tauri::async_runtime::spawn(async move {
            let title = p.info.title.clone();
            match me.media_job(&p, &c).await {
                Ok(name) => me.push_toast("complete", "", &name),
                Err(e) => me.push_toast("failed", "", &format!("{title}: {e}")),
            }
            me.bump();
        });
        Ok(())
    }

    pub async fn media_job(self: &Arc<Self>, p: &Pending, c: &gdm_media::Choice) -> Result<String, String> {
        let fmt = |id: &Option<String>| id.as_ref().and_then(|i| p.info.formats.iter().find(|f| &f.id == i).cloned());
        let (v, a) = (fmt(&c.video_id), fmt(&c.audio_id));
        let cat = self.category_dir(&self.category_for(&format!("x.{}", c.container)));
        let parts =
            cat.join(format!(".gdm-media-{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()));
        std::fs::create_dir_all(&parts).map_err(|e| e.to_string())?;
        let r = async {
            let mut files = vec![];
            for (tag, f) in [("video", v), ("audio", a)] {
                let Some(f) = f else { continue };
                // HLS segments are MPEG-TS whatever yt-dlp calls the format: always remuxed by ffmpeg
                let ext = if f.protocol.starts_with("m3u8") {
                    "ts".to_string()
                } else if gdm_media::MEDIA_EXTS.contains(&f.ext.as_str()) {
                    f.ext.clone()
                } else {
                    "bin".into()
                };
                let path = parts.join(format!("{tag}.{ext}"));
                if f.protocol.starts_with("m3u8") {
                    self.hls_fetch(&f.url, &path, &p.page).await?;
                } else if f.protocol == "http_dash_segments" {
                    // DASH: yt-dlp read the .mpd; GDM fetches init + segments itself, like HLS
                    let client = Self::media_client(&p.page)?;
                    Self::fetch_segments(&client, f.fragments.iter().cloned(), &path).await?;
                } else {
                    self.stream_fetch(&f.url, &parts, &format!("{tag}.{ext}"), &p.page).await?;
                }
                files.push(path);
            }
            if files.is_empty() {
                return Err("no-formats".to_string());
            }
            let out = parts.join(format!("out.{}", c.container));
            if files.len() == 1 && !files[0].extension().is_some_and(|e| e == "ts") {
                std::fs::rename(&files[0], &out).map_err(|e| e.to_string())?;
            } else {
                self.mux(&files, &out, &parts).await?;
            }
            self.check_streams(&out, &parts, c.video_id.is_some(), c.audio_id.is_some() || c.video_id.is_none()).await?;
            self.publish(&out, &cat, &p.info.title, &c.container, &p.page)
        }
        .await;
        let _ = std::fs::remove_dir_all(&parts);
        r
    }

    /// one http(s) stream through the GDM engine (segments, resume, W-02), shown in the list while it runs
    async fn stream_fetch(self: &Arc<Self>, url: &str, dir: &Path, name: &str, page: &str) -> Result<(), String> {
        let same_host = url::Url::parse(url).ok().and_then(|u| u.host_str().map(String::from))
            == url::Url::parse(page).ok().and_then(|u| u.host_str().map(String::from));
        let id = self
            .engine
            .add(gdm_core::NewJob { url: url.into(), dir: dir.to_path_buf(), name: Some(name.into()), typed: same_host, ..Default::default() })
            .map_err(|e| e.code())?;
        self.start(&id, gdm_core::Unverified::Restart);
        tokio::time::sleep(Duration::from_millis(200)).await;
        while self.is_running(&id) {
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
        let j = self.engine.job(&id).map_err(|e| e.code())?;
        let _ = self.engine.remove(&id, false); // the part stays on disk for the mux; the row goes
        self.bump();
        if j.state == "complete" || j.state == "unprotected" {
            Ok(())
        } else {
            Err(j.error_code.unwrap_or(j.state))
        }
    }

    /// HLS (unencrypted) fetched by GDM itself: playlist + segments through the engine's network policy;
    /// M-03 limits in the parser; encrypted => unsupported in v1.0; DRM => refused
    /// the client for playlists and segments: GDM's network policy (W-02), the page's own host may be on the LAN
    fn media_client(page: &str) -> Result<gdm_core::reqwest::Client, String> {
        let host = url::Url::parse(page).ok().and_then(|u| u.host_str().map(String::from)).unwrap_or_default();
        gdm_core::net::client(Arc::new(gdm_core::net::Policy { allow_private: vec![host] }), false, Duration::from_secs(30))
    }

    /// init + segments, in order, into one file (HLS and DASH); M-03/E-03 bounds per segment and in total, and
    /// never into the 1 GB free-space reserve
    async fn fetch_segments(client: &gdm_core::reqwest::Client, urls: impl Iterator<Item = url::Url>, out: &Path) -> Result<(), String> {
        use tokio::io::AsyncWriteExt;
        const MAX_SEGMENT: u64 = 256 << 20;
        let dir = out.parent().unwrap_or(Path::new("."));
        let total_cap = (20u64 << 30).min(gdm_core::fsx::free_space(dir).saturating_sub(gdm_core::seg::RESERVE));
        let mut total = 0u64;
        let mut f = tokio::fs::File::create(out).await.map_err(|e| e.to_string())?;
        for s in urls {
            let mut r = client.get(s).send().await.map_err(|e| e.to_string())?;
            if !r.status().is_success() {
                return Err(format!("segment http {}", r.status()));
            }
            let mut seg = 0u64;
            while let Some(ch) = r.chunk().await.map_err(|e| e.to_string())? {
                seg += ch.len() as u64;
                total += ch.len() as u64;
                if seg > MAX_SEGMENT || total > total_cap {
                    drop(f);
                    let _ = tokio::fs::remove_file(out).await;
                    return Err("too-large".into());
                }
                f.write_all(&ch).await.map_err(|e| e.to_string())?;
            }
        }
        f.sync_all().await.map_err(|e| e.to_string())
    }

    async fn hls_fetch(&self, url: &str, out: &Path, page: &str) -> Result<(), String> {
        let client = Self::media_client(page)?;
        let get_text = |u: url::Url| {
            let c = client.clone();
            async move {
                let mut r = c.get(u).send().await.map_err(|e| e.to_string())?;
                if !r.status().is_success() {
                    return Err(format!("playlist http {}", r.status()));
                }
                let mut b = Vec::new();
                while let Some(ch) = r.chunk().await.map_err(|e| e.to_string())? {
                    if b.len() + ch.len() > gdm_media::MAX_TEXT {
                        return Err("too-large".to_string());
                    }
                    b.extend_from_slice(&ch);
                }
                Ok::<_, String>(String::from_utf8_lossy(&b).into_owned())
            }
        };
        let mut u = url::Url::parse(url).map_err(|_| "bad-url")?;
        for _depth in 0..gdm_media::MAX_DEPTH {
            match gdm_media::parse_m3u8(&get_text(u.clone()).await?, &u)? {
                gdm_media::Playlist::Master { variants } => {
                    u = variants.iter().max_by_key(|v| v.bandwidth.unwrap_or(0)).map(|v| v.uri.clone()).ok_or("no-variant")?;
                }
                gdm_media::Playlist::Media { segments, init, endlist, encrypted, .. } => {
                    if encrypted.is_some() {
                        return Err("encrypted-hls".into());
                    }
                    if !endlist {
                        return Err("live".into());
                    }
                    return Self::fetch_segments(&client, init.into_iter().chain(segments), out).await;
                }
            }
        }
        Err("playlist-depth".into())
    }

    async fn mux(&self, inputs: &[PathBuf], out: &Path, cwd: &Path) -> Result<(), String> {
        let mut args: Vec<String> = ["-hide_banner", "-nostdin", "-loglevel", "error", "-protocol_whitelist", "file,pipe"].map(String::from).to_vec();
        // each input's demuxer is pinned from the name GDM gave it and only these demuxers may run, so
        // hls/concat/image2/... (which can open other local files) are never reached (security review G7)
        args.extend(["-format_whitelist", "mpegts,mov,mp4,m4a,matroska,webm"].map(String::from));
        for i in inputs {
            let demux = match i.extension().and_then(|e| e.to_str()).unwrap_or("") {
                "ts" => "mpegts",
                "mp4" | "m4a" => "mov",
                "mkv" | "webm" => "matroska",
                e => return Err(format!("unsupported-input {e}")),
            };
            args.extend(["-f".to_string(), demux.to_string(), "-i".to_string(), i.display().to_string()]);
        }
        if inputs.len() == 2 {
            args.extend(["-map", "0:v:0", "-map", "1:a:0"].map(String::from));
        }
        args.extend(["-c", "copy", "-y"].map(String::from));
        args.push(out.display().to_string());
        run_component(&self.data, &crate::components::exe("ffmpeg"), &args, cwd, 1 << 20, Duration::from_secs(3600)).await.map(|_| ())
    }

    /// M-06: the result really has the streams it should (no silent video without sound)
    async fn check_streams(&self, f: &Path, cwd: &Path, video: bool, audio: bool) -> Result<(), String> {
        if !crate::components::has(&self.data, &crate::components::exe("ffprobe")) {
            return Err("need-component".into());
        }
        let args: Vec<String> = ["-v", "error", "-protocol_whitelist", "file,pipe", "-show_entries", "stream=codec_type", "-of", "csv=p=0"]
            .map(String::from)
            .into_iter()
            .chain([f.display().to_string()])
            .collect();
        let out = String::from_utf8_lossy(
            &run_component(&self.data, &crate::components::exe("ffprobe"), &args, cwd, 1 << 16, Duration::from_secs(60)).await?,
        )
        .to_string();
        // csv lines look like "video" or "video," (side data columns)
        let has = |k: &str| out.lines().any(|l| l.split(',').next().is_some_and(|c| c.trim() == k));
        if (video && !has("video")) || (audio && !has("audio")) {
            return Err(format!("missing-stream (found: {})", out.split_whitespace().collect::<Vec<_>>().join(",")));
        }
        Ok(())
    }

    /// final name (M-02 allowlist + sanitizer), a free name in the category folder, MOTW + antivirus (N-05),
    /// then a finished row in the list
    fn publish(&self, out: &Path, cat: &Path, title: &str, container: &str, page: &str) -> Result<String, String> {
        let base = gdm_core::name::sanitize(&gdm_media::media_name(title, container)?);
        let mut name = base.clone();
        let mut n = 0;
        while cat.join(&name).exists() {
            n += 1;
            name = gdm_core::name::with_counter(&base, n);
        }
        // fail closed (like finalize): the work folder is GDM's own data folder on a normal volume, so a folder
        // that cannot be held or a mark that cannot be written stops the publish instead of skipping MOTW / AV
        let (Some(parent), Some(leaf)) = (out.parent(), out.file_name().and_then(|l| l.to_str())) else { return Err("bad work path".into()) };
        let d = gdm_core::fsx::Dir::open(parent).map_err(|e| format!("work folder: {e}"))?;
        let saved = gdm_core::fsx::attachment_save(&d, leaf, &name, page, None);
        if !out.exists() {
            return Err("blocked".into()); // the antivirus removed it
        }
        saved?;
        std::fs::rename(out, cat.join(&name)).map_err(|e| e.to_string())?;
        let sha = gdm_core::finalize::sha_handle(&std::fs::File::open(cat.join(&name)).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        self.engine.record_finished(page, cat, &name, &sha).map_err(|e| e.code())?;
        Ok(name)
    }
}
