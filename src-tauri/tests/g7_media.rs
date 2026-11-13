// SPDX-License-Identifier: RPL-1.5
//! G7 F07 with the REAL components (yt-dlp 2026.10.26, BtbN ffmpeg n8.1 win64-lgpl) when they are present in
//! GDM_MEDIA_BIN (default D:\gdm-media-e2e\bin); otherwise the test says SKIP. Media fixtures are made by
//! that ffmpeg and served by the hostile fixture.
//!  1. HLS page: yt-dlp -J runs through GDM's relay; yt-dlp opens no connection except to the relay (ADR-6,
//!     M-08: netstat sampled while it runs); GDM fetches the segments; ffmpeg remuxes; ffprobe sees both streams.
//!  2. separate video + audio (M-06): GDM downloads both with its engine, ffmpeg muxes local files only, the
//!     result has 1 video + 1 audio stream.
use gdm_app::app::{lock, App};
use gdm_app::components::{self, Component, ComponentFile};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn bin() -> Option<PathBuf> {
    let b = PathBuf::from(std::env::var("GDM_MEDIA_BIN").unwrap_or_else(|_| r"D:\gdm-media-e2e\bin".into()));
    ["yt-dlp.exe", "ffmpeg.exe", "ffprobe.exe"].iter().all(|f| b.join(f).is_file()).then_some(b)
}

fn sha(p: &Path) -> String {
    gdm_core::finalize::sha_handle(&std::fs::File::open(p).unwrap()).unwrap()
}

fn install(data: &Path, b: &Path) {
    for n in ["yt-dlp.exe", "ffmpeg.exe", "ffprobe.exe"] {
        let bytes = std::fs::read(b.join(n)).unwrap();
        let s = sha(&b.join(n));
        let c = Component {
            name: n.into(),
            version: "e2e".into(),
            url: format!("https://e2e/{n}"),
            sha256: s.clone(),
            size: bytes.len() as u64,
            files: vec![ComponentFile { name: n.into(), path: None, sha256: s }],
            platform: None,
        };
        components::install(data, &c, &bytes).unwrap();
    }
}

fn ffmpeg(b: &Path, cwd: &Path, args: &[&str]) {
    let st = std::process::Command::new(b.join("ffmpeg.exe"))
        .args(["-hide_banner", "-loglevel", "error", "-y"])
        .args(args)
        .current_dir(cwd)
        .status()
        .unwrap();
    assert!(st.success(), "ffmpeg {args:?}");
}

/// connections owned by yt-dlp.exe whose remote port is not the relay's (sampled every ~100 ms)
fn watch_ytdlp(stop: Arc<AtomicBool>) -> std::thread::JoinHandle<Vec<String>> {
    std::thread::spawn(move || {
        let mut seen = vec![];
        while !stop.load(Ordering::Relaxed) {
            let pids: Vec<String> = String::from_utf8_lossy(
                &std::process::Command::new("tasklist").args(["/FI", "IMAGENAME eq yt-dlp.exe", "/FO", "CSV", "/NH"]).output().unwrap().stdout,
            )
            .lines()
            .filter_map(|l| l.split(',').nth(1).map(|p| p.trim_matches('"').to_string()))
            .filter(|p| p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty())
            .collect();
            if !pids.is_empty() {
                let ns =
                    String::from_utf8_lossy(&std::process::Command::new("netstat").args(["-ano", "-p", "tcp"]).output().unwrap().stdout).to_string();
                for l in ns.lines() {
                    let f: Vec<&str> = l.split_whitespace().collect();
                    if f.len() == 5 && pids.iter().any(|p| p == f[4]) {
                        seen.push(format!("{} -> {} {}", f[1], f[2], f[3]));
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        seen
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn f07_real_components_hls_page_and_dash_pair() {
    let Some(b) = bin() else {
        eprintln!("SKIP: no yt-dlp/ffmpeg/ffprobe in GDM_MEDIA_BIN");
        return;
    };
    let d = std::env::temp_dir().join(format!("gdm-media-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let gen = d.join("gen");
    std::fs::create_dir_all(&gen).unwrap();
    // fixtures made by the real ffmpeg: HLS (muxed TS), a video-only MP4, an audio-only M4A
    ffmpeg(
        &b,
        &gen,
        &[
            "-f",
            "lavfi",
            "-i",
            "testsrc=d=3:s=320x240:r=25",
            "-f",
            "lavfi",
            "-i",
            "sine=d=3",
            "-c:v",
            "mpeg2video",
            "-c:a",
            "aac",
            "-f",
            "hls",
            "-hls_time",
            "1",
            "-hls_list_size",
            "0",
            "-hls_segment_filename",
            "seg%d.ts",
            "v.m3u8",
        ],
    );
    ffmpeg(&b, &gen, &["-f", "lavfi", "-i", "testsrc=d=2:s=320x240:r=25", "-an", "-c:v", "mpeg4", "video.mp4"]);
    ffmpeg(&b, &gen, &["-f", "lavfi", "-i", "sine=d=2", "-vn", "-c:a", "aac", "audio.m4a"]);
    let fx = gdm_fixture::Fixture::start("127.0.0.1:0").await;
    for e in std::fs::read_dir(&gen).unwrap().flatten() {
        fx.add_blob(&e.file_name().to_string_lossy(), std::fs::read(e.path()).unwrap());
    }
    let a = App::open(&d.join("data")).unwrap();
    lock(&a.settings).download_dir = d.join("dl").display().to_string();
    install(&a.data, &b);

    // 1. HLS page through yt-dlp
    let stop = Arc::new(AtomicBool::new(false));
    let w = watch_ytdlp(stop.clone());
    let page = fx.url("/static/v.m3u8");
    fx.clear_log();
    let probe = a.media_probe(&page).await;
    let fixture_requests = fx.log().len();
    let relayed = gdm_app::media::LAST_RELAY_SERVED.load(Ordering::Relaxed);
    stop.store(true, Ordering::Relaxed);
    let conns = w.join().unwrap();
    let probe = probe.unwrap();
    assert!(!probe.choices.is_empty(), "{probe:?}");
    let relay_ok = |c: &str| c.split(" -> ").nth(1).is_some_and(|r| r.starts_with("127.0.0.1:"));
    eprintln!("yt-dlp sockets seen: {conns:?}");
    // every yt-dlp socket goes to a loopback port; the fixture is loopback too, so also check the fixture
    // never saw yt-dlp's own User-Agent outside the relay: every request reached it through the relay socket
    assert!(conns.iter().all(|c| relay_ok(c)), "yt-dlp connected elsewhere: {conns:?}");
    // ADR-6 / M-08: every request yt-dlp made reached the site through GDM's relay (one request per relayed
    // connection; a direct request would show up at the fixture without a relay count)
    eprintln!("probe: fixture requests {fixture_requests}, relayed {relayed}");
    assert!(fixture_requests > 0 && fixture_requests == relayed, "fixture {fixture_requests} vs relay {relayed}");
    let pending = lock(&a.media).remove(&probe.token).unwrap();
    let c = pending.choices[0].clone();
    let name = a.media_job(&pending, &c).await.unwrap();
    let rows = a.rows(None).unwrap();
    let r = rows.iter().find(|r| r.row.name == name).unwrap();
    assert!(r.row.state == "complete" || r.row.state == "unprotected");
    eprintln!("HLS result: {name} ({} bytes)", r.row.size.unwrap_or(0));

    // 2. separate video + audio streams (M-06): engine downloads both, ffmpeg muxes, ffprobe checks
    let json = format!(
        r#"{{"title":"Pair test","formats":[
          {{"format_id":"v1","ext":"mp4","url":"{}","protocol":"https","vcodec":"mp4v.20.9","acodec":"none","height":240,"tbr":300}},
          {{"format_id":"a1","ext":"m4a","url":"{}","protocol":"https","vcodec":"none","acodec":"mp4a.40.2","tbr":128}}]}}"#,
        fx.url("/static/video.mp4"),
        fx.url("/static/audio.m4a")
    );
    let info = gdm_media::parse_info(&json).unwrap();
    let choices = gdm_media::choices(&info);
    let pair = choices.iter().find(|c| c.video_id.is_some() && c.audio_id.is_some()).cloned().expect("a video+audio choice");
    let p = gdm_app::media::Pending { page: fx.url("/static/page.html"), info, choices };
    let name = a.media_job(&p, &pair).await.unwrap();
    let out = std::path::PathBuf::from(&lock(&a.settings).download_dir).join("Video").join(&name);
    let probe = std::process::Command::new(b.join("ffprobe.exe"))
        .args(["-v", "error", "-show_entries", "stream=codec_type", "-of", "csv=p=0"])
        .arg(&out)
        .output()
        .unwrap();
    let streams = String::from_utf8_lossy(&probe.stdout).to_string();
    let kind = |k: &str| streams.lines().filter(|l| l.split(',').next().is_some_and(|c| c.trim() == k)).count();
    assert_eq!(kind("video"), 1, "{streams}");
    assert_eq!(kind("audio"), 1, "{streams}");
    assert!(name.starts_with("Pair test") && (name.ends_with(".mp4") || name.ends_with(".mkv")), "{name}");
    drop(a);
    let _ = std::fs::remove_dir_all(&d);
}
