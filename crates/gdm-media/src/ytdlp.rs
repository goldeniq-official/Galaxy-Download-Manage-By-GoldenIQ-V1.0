// SPDX-License-Identifier: RPL-1.5
//! yt-dlp in "extract only" mode (F07): the argv GDM passes, the `-J` JSON it reads back,
//! and the quality list for the popup (M-06). GDM's own engine downloads the chosen streams.

use serde_json::Value;

/// Options GDM must never pass to yt-dlp (M-01): each one runs code, reads local files,
/// writes files, hands the download to another program or sends credentials.
pub const FORBIDDEN: &[&str] = &[
    "--exec",
    "--config-location",
    "--config-locations",
    "--load-info-json",
    "--plugin-dirs",
    "--batch-file",
    "-a",
    "--downloader",
    "--external-downloader",
    "--netrc",
    "--cookies",
    "--cookies-from-browser",
    "-o",
    "--output",
    "--paths",
];

/// `-J` output larger than this is refused before parsing.
pub const MAX_JSON: usize = 16 * 1024 * 1024;
/// Formats kept from one `-J` result (M-03); extra ones set `Info::truncated`.
pub const MAX_FORMATS: usize = 64;
/// Entries in the quality popup, audio-only choice not counted.
pub const MAX_CHOICES: usize = 12;

fn http_url(s: &str) -> Option<url::Url> {
    if s.is_empty() || s.starts_with('-') || s.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return None;
    }
    let u = url::Url::parse(s).ok()?;
    let ok = matches!(u.scheme(), "http" | "https") && u.host().is_some() && u.username().is_empty() && u.password().is_none();
    ok.then_some(u)
}

/// argv for `yt-dlp` (program name not included). `proxy` "" = direct connection (yt-dlp's own rule).
/// Needs yt-dlp >= 2024.10.22 (first release with `--no-plugin-dirs`); an older one stops on the
/// unknown option, which fails safe.
pub fn ytdlp_args(url: &str, proxy: &str) -> Result<Vec<String>, String> {
    let url = http_url(url).ok_or("bad-url")?;
    if !proxy.is_empty() {
        let p = url::Url::parse(proxy).map_err(|_| "bad-proxy")?;
        let ok = matches!(p.scheme(), "http" | "https" | "socks4" | "socks4a" | "socks5" | "socks5h")
            && !proxy.starts_with('-')
            && !proxy.chars().any(|c| c.is_control() || c.is_whitespace());
        if !ok {
            return Err("bad-proxy".into());
        }
    }
    let args = [
        "--ignore-config",       // no system/user/portable yt-dlp.conf can add options
        "--no-config-locations", // and no --config-locations left over from anywhere
        "--no-plugin-dirs",      // no plugin code from default plugin folders
        "--no-cache-dir",        // writes nothing to the user's cache
        "--no-playlist",         // a video+playlist URL means the one video
        "--dump-single-json",    // -J: print one JSON, simulate, download nothing
        "--no-warnings",         // stdout/stderr stay just the JSON or the error
        "--proxy",               // always given so env proxies are not guessed
        proxy,
        "--socket-timeout",
        "30",
        "--", // M-01: everything after this is a URL, never an option
        url.as_str(),
    ];
    Ok(args.iter().map(|s| s.to_string()).collect())
}

#[derive(Debug, Clone, PartialEq)]
pub struct Format {
    pub id: String,
    pub ext: String,
    pub url: String,
    pub protocol: String,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub height: Option<u32>,
    pub tbr: Option<f64>,
    /// `filesize`, else `filesize_approx`.
    pub filesize: Option<u64>,
    /// Only `has_drm: true`; yt-dlp's `"maybe"` (some HLS) counts as false, the HLS parser refuses DRM keys itself (M-04).
    pub has_drm: bool,
    /// Without Cookie / Authorization / Proxy-Authorization (M-08) and without control characters.
    pub http_headers: Vec<(String, String)>,
    /// DASH (`http_dash_segments`): init + media segments in order, resolved against `fragment_base_url`; every
    /// one an http(s) URL without userinfo, at most `MAX_SEGMENTS`. Empty when any fragment fails a check (the
    /// format is then not offered) and for every other protocol.
    pub fragments: Vec<url::Url>,
}

/// yt-dlp's fragment list for a DASH format (hostile input): all-or-nothing
fn dash_fragments(v: &Value) -> Vec<url::Url> {
    let base = s(v, "fragment_base_url").and_then(|b| http_url(&b));
    let Some(list) = v.get("fragments").and_then(Value::as_array) else { return vec![] };
    if list.is_empty() || list.len() > crate::MAX_SEGMENTS {
        return vec![];
    }
    let mut out = Vec::with_capacity(list.len());
    for f in list {
        let u = match (s(f, "url"), s(f, "path"), &base) {
            (Some(u), _, _) => http_url(&u),
            (None, Some(p), Some(b)) if !p.starts_with('-') && !p.chars().any(|c| c.is_control()) => {
                b.join(&p).ok().and_then(|u| http_url(u.as_str()))
            }
            _ => None,
        };
        match u {
            Some(u) => out.push(u),
            None => return vec![],
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Info {
    pub title: String,
    pub duration: Option<f64>,
    pub is_live: bool,
    /// Nothing playable is left and yt-dlp saw DRM (any `has_drm` format or top-level `_has_drm`).
    pub drm: bool,
    pub truncated: bool,
    pub formats: Vec<Format>,
}

fn s(v: &Value, k: &str) -> Option<String> {
    v.get(k)?.as_str().map(str::to_string)
}

fn format(v: &Value) -> Option<Format> {
    let url = s(v, "url").filter(|u| http_url(u).is_some())?;
    let headers = v.get("http_headers").and_then(Value::as_object).map_or_else(Vec::new, |m| {
        m.iter()
            .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
            .filter(|(k, v)| {
                !["cookie", "authorization", "proxy-authorization"].contains(&k.to_ascii_lowercase().as_str())
                    && !k.chars().chain(v.chars()).any(char::is_control)
            })
            .collect()
    });
    let size = |k: &str| v.get(k).and_then(Value::as_f64).filter(|n| *n >= 0.0).map(|n| n as u64);
    Some(Format {
        id: s(v, "format_id").unwrap_or_default(),
        ext: s(v, "ext").unwrap_or_default(),
        protocol: s(v, "protocol").unwrap_or_else(|| url.split(':').next().unwrap_or_default().to_string()),
        url,
        vcodec: s(v, "vcodec"),
        acodec: s(v, "acodec"),
        height: v.get("height").and_then(Value::as_u64).and_then(|h| u32::try_from(h).ok()),
        tbr: v.get("tbr").and_then(Value::as_f64),
        filesize: size("filesize").or_else(|| size("filesize_approx")),
        has_drm: v.get("has_drm") == Some(&Value::Bool(true)),
        http_headers: headers,
        fragments: if s(v, "protocol").as_deref() == Some("http_dash_segments") { dash_fragments(v) } else { vec![] },
    })
}

/// Parses `yt-dlp -J` stdout (hostile input). A result with no `formats` array but a top-level
/// `url` is read as a single format.
pub fn parse_info(json: &str) -> Result<Info, String> {
    if json.len() > MAX_JSON {
        return Err("too-large".into());
    }
    let v: Value = serde_json::from_str(json).map_err(|_| "bad-json")?;
    if !v.is_object() {
        return Err("bad-json".into());
    }
    let list: Vec<&Value> = match v.get("formats").and_then(Value::as_array) {
        Some(a) => a.iter().collect(),
        None => vec![&v],
    };
    let mut info = Info {
        title: s(&v, "title").unwrap_or_default(),
        duration: v.get("duration").and_then(Value::as_f64),
        is_live: v.get("is_live") == Some(&Value::Bool(true)),
        ..Info::default()
    };
    for f in list.into_iter().filter_map(format) {
        if info.formats.len() == MAX_FORMATS {
            info.truncated = true;
            break;
        }
        info.formats.push(f);
    }
    let saw_drm = v.get("_has_drm") == Some(&Value::Bool(true)) || info.formats.iter().any(|f| f.has_drm);
    info.drm = saw_drm && info.formats.iter().all(|f| f.has_drm);
    Ok(info)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    pub label: String,
    pub height: Option<u32>,
    pub video_id: Option<String>,
    pub audio_id: Option<String>,
    pub container: String,
    pub est_size: Option<u64>,
}

fn has(c: &Option<String>, prefixes: &[&str]) -> bool {
    c.as_deref().is_some_and(|c| prefixes.iter().any(|p| c.starts_with(p)))
}
fn is_none(c: &Option<String>) -> bool {
    c.as_deref() == Some("none")
}
/// 2 = H.264 (mp4), 1 = VP9/AV1 (mkv), 0 = anything else (mkv).
fn family(f: &Format) -> u8 {
    if has(&f.vcodec, &["avc1", "h264"]) {
        2
    } else if has(&f.vcodec, &["vp9", "vp09", "av01"]) {
        1
    } else {
        0
    }
}
const AAC: &[&str] = &["mp4a", "aac"];
const OPUS: &[&str] = &["opus", "vorbis"];

fn best<'a>(it: impl Iterator<Item = &'a Format>, key: impl Fn(&Format) -> u8) -> Option<&'a Format> {
    it.max_by(|a, b| (key(a), a.tbr.unwrap_or(0.0)).partial_cmp(&(key(b), b.tbr.unwrap_or(0.0))).unwrap_or(std::cmp::Ordering::Equal))
}

/// Quality list for the popup (M-06). Never offers DRM formats. Offers progressive http(s), HLS (`m3u8`,
/// `m3u8_native`) and DASH (`http_dash_segments` with a checked fragment list): GDM fetches both itself;
/// `f4m`, `ism` and every other protocol are left out because GDM has no fetcher for them.
pub fn choices(info: &Info) -> Vec<Choice> {
    let ok: Vec<&Format> = info
        .formats
        .iter()
        .filter(|f| {
            !f.has_drm
                && (matches!(f.protocol.as_str(), "http" | "https" | "m3u8" | "m3u8_native")
                    || (f.protocol == "http_dash_segments" && !f.fragments.is_empty()))
        })
        .collect();
    let audio: Vec<&Format> = ok.iter().copied().filter(|f| is_none(&f.vcodec) && !is_none(&f.acodec)).collect();
    let best_audio = |p: &[&str]| best(audio.iter().copied().filter(|f| has(&f.acodec, p)), |_| 0);
    let video: Vec<&Format> = ok.iter().copied().filter(|f| !is_none(&f.vcodec) && f.height.is_some()).collect();
    let mut heights: Vec<u32> = video.iter().filter_map(|f| f.height).collect();
    heights.sort_unstable_by(|a, b| b.cmp(a));
    heights.dedup();
    let mut out = Vec::new();
    for h in heights.into_iter().take(MAX_CHOICES) {
        let Some(v) = best(video.iter().copied().filter(|f| f.height == Some(h)), family) else { continue };
        let (a, container) = if !is_none(&v.acodec) {
            // muxed: keep its own container when it is one we write, else mkv
            (None, if ["mp4", "webm", "mkv"].contains(&v.ext.as_str()) { v.ext.clone() } else { "mkv".into() })
        } else if family(v) == 2 {
            match best_audio(AAC) {
                Some(a) => (Some(a), "mp4".into()),
                None => (best_audio(OPUS).or_else(|| best_audio(&[""])), "mkv".into()),
            }
        } else {
            (best_audio(OPUS).or_else(|| best_audio(AAC)).or_else(|| best_audio(&[""])), "mkv".into())
        };
        let est_size = match a {
            Some(a) => v.filesize.zip(a.filesize).map(|(x, y)| x + y),
            None => v.filesize,
        };
        out.push(Choice {
            label: format!("{h}p {}", container.to_uppercase()),
            height: Some(h),
            video_id: Some(v.id.clone()),
            audio_id: a.map(|a| a.id.clone()),
            container,
            est_size,
        });
    }
    // a page whose formats say nothing about their height (generic HLS without RESOLUTION): one "best" entry
    if out.is_empty() {
        let unknown = ok.iter().copied().filter(|f| !is_none(&f.vcodec) && f.height.is_none());
        if let Some(v) = best(unknown, |_| 0) {
            out.push(Choice {
                label: "Best MKV".into(),
                height: None,
                video_id: Some(v.id.clone()),
                audio_id: None,
                container: "mkv".into(),
                est_size: v.filesize,
            });
        }
    }
    if let Some((a, c)) = best_audio(AAC).map(|a| (a, "m4a")).or_else(|| best_audio(&["opus"]).map(|a| (a, "opus"))) {
        out.push(Choice {
            label: format!("Audio only {}", c.to_uppercase()),
            height: None,
            video_id: None,
            audio_id: Some(a.id.clone()),
            container: c.into(),
            est_size: a.filesize,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DASH (IDM parity): offered only with a fully checked fragment list (the shape yt-dlp writes for a real .mpd)
    #[test]
    fn dash_fragments_all_or_nothing() {
        let d = |frags: &str| {
            parse_info(&format!(
                r#"{{"title":"t","formats":[{{"format_id":"v","ext":"mp4","url":"https://cdn/x.mpd","protocol":"http_dash_segments","vcodec":"avc1","acodec":"mp4a","height":480,"fragment_base_url":"https://cdn/a/./","fragments":{frags}}}]}}"#
            ))
            .unwrap()
        };
        let i = d(r#"[{"path":"v/init.mp4"},{"path":"v/1.m4s","duration":4},{"url":"https://cdn2/abs/2.m4s"}]"#);
        let f = &i.formats[0].fragments;
        assert_eq!(f.iter().map(|u| u.as_str()).collect::<Vec<_>>(), ["https://cdn/a/v/init.mp4", "https://cdn/a/v/1.m4s", "https://cdn2/abs/2.m4s"]);
        assert_eq!(choices(&i).len(), 1, "DASH with fragments is offered");
        for bad in
            [r#"[{"url":"file:///C:/x"}]"#, r#"[{"path":"v/1.m4s"},{"url":"https://u:p@cdn/x"}]"#, r#"[{"path":"-o"}]"#, r#"[]"#, r#"[{"x":1}]"#]
        {
            let i = d(bad);
            assert!(i.formats[0].fragments.is_empty() && choices(&i).is_empty(), "{bad}");
        }
    }

    #[test]
    fn unknown_height_gets_one_best_choice() {
        let i = parse_info(r#"{"title":"t","formats":[{"format_id":"0","ext":"mp4","url":"https://h/v.m3u8","protocol":"m3u8_native"}]}"#).unwrap();
        let c = choices(&i);
        assert_eq!(c.len(), 1);
        assert_eq!((c[0].label.as_str(), c[0].container.as_str(), c[0].video_id.as_deref()), ("Best MKV", "mkv", Some("0")));
    }

    #[test]
    fn argv_shape_and_url_after_double_dash() {
        let a = ytdlp_args("https://example.com/watch?v=1", "").unwrap();
        assert_eq!(a[a.len() - 2], "--");
        assert_eq!(a[a.len() - 1], "https://example.com/watch?v=1");
        assert!(a.contains(&"--dump-single-json".into()) && a.contains(&"--ignore-config".into()));
        let p = a.iter().position(|x| x == "--proxy").unwrap();
        assert_eq!(a[p + 1], "");
        let a = ytdlp_args("http://x.org/v", "socks5://127.0.0.1:9050").unwrap();
        assert_eq!(a[p + 1], "socks5://127.0.0.1:9050");
    }

    #[test]
    fn never_passes_forbidden() {
        for (u, p) in [("https://a.b/c", ""), ("https://a.b/c?x=--exec", "http://p:1"), ("https://a.b/-o", "")] {
            for arg in ytdlp_args(u, p).unwrap() {
                assert!(!FORBIDDEN.contains(&arg.as_str()), "{arg}");
                assert!(!FORBIDDEN.iter().any(|f| arg.starts_with(&format!("{f}="))), "{arg}");
            }
        }
    }

    #[test]
    fn m01_option_looking_urls_refused() {
        for u in [
            "--exec=calc",
            "--exec calc",
            "-o x",
            "file:///C:/Windows/win.ini",
            "ftp://a.b/c",
            "https://user:pw@a.b/c",
            "https://a.b/c\n--exec",
            "https://a.b/c d",
            "",
            "data:text/plain,hi",
        ] {
            assert_eq!(ytdlp_args(u, ""), Err("bad-url".into()), "{u}");
        }
        assert_eq!(ytdlp_args("https://a.b/c", "--exec=calc"), Err("bad-proxy".into()));
        assert_eq!(ytdlp_args("https://a.b/c", "file:///x"), Err("bad-proxy".into()));
    }

    const J: &str = r#"{
      "title": "Clip", "duration": 61.5, "is_live": false,
      "formats": [
        {"format_id":"drm1","ext":"mp4","url":"https://cdn/drm","protocol":"https","vcodec":"avc1.640028","acodec":"none","height":2160,"tbr":9000,"has_drm":true},
        {"format_id":"140","ext":"m4a","url":"https://cdn/a140","protocol":"https","vcodec":"none","acodec":"mp4a.40.2","tbr":129,"filesize":1000},
        {"format_id":"251","ext":"webm","url":"https://cdn/a251","protocol":"https","vcodec":"none","acodec":"opus","tbr":140,"filesize":900},
        {"format_id":"137","ext":"mp4","url":"https://cdn/v137","protocol":"https","vcodec":"avc1.640028","acodec":"none","height":1080,"tbr":4000,"filesize":50000,
         "http_headers":{"User-Agent":"UA","Cookie":"sid=1","authorization":"Bearer x","Referer":"https://site/"}},
        {"format_id":"248","ext":"webm","url":"https://cdn/v248","protocol":"https","vcodec":"vp9","acodec":"none","height":1080,"tbr":5000},
        {"format_id":"303","ext":"webm","url":"https://cdn/v303","protocol":"https","vcodec":"vp9","acodec":"none","height":1440,"tbr":8000,"filesize_approx":80000},
        {"format_id":"18","ext":"mp4","url":"https://cdn/m18","protocol":"https","vcodec":"avc1.42001E","acodec":"mp4a.40.2","height":360,"tbr":500,"filesize":7000},
        {"format_id":"hls-720","ext":"mp4","url":"https://cdn/720.m3u8","protocol":"m3u8_native","vcodec":"avc1.4d401f","acodec":"mp4a.40.2","height":720,"tbr":2500},
        {"format_id":"dash-480","ext":"mp4","url":"https://cdn/x.mpd","protocol":"http_dash_segments","vcodec":"avc1","acodec":"none","height":480},
        {"format_id":"local","ext":"mp4","url":"file:///C:/x.mp4","protocol":"file","vcodec":"avc1","acodec":"mp4a","height":240}
      ]}"#;

    #[test]
    fn parses_and_strips_credentials() {
        let i = parse_info(J).unwrap();
        assert_eq!((i.title.as_str(), i.duration, i.is_live, i.drm, i.truncated), ("Clip", Some(61.5), false, false, false));
        assert_eq!(i.formats.len(), 9, "file: format dropped");
        let f = i.formats.iter().find(|f| f.id == "137").unwrap();
        assert_eq!(f.http_headers.len(), 2);
        assert!(f.http_headers.iter().all(|(k, _)| !k.eq_ignore_ascii_case("cookie") && !k.eq_ignore_ascii_case("authorization")));
        assert!(i.formats[0].has_drm);
        assert_eq!(i.formats.iter().find(|f| f.id == "303").unwrap().filesize, Some(80000));
    }

    #[test]
    fn m06_choices() {
        let c = choices(&parse_info(J).unwrap());
        let labels: Vec<&str> = c.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(labels, ["1440p MKV", "1080p MP4", "720p MP4", "360p MP4", "Audio only M4A"], "no 2160 (DRM), no 480 (DASH)");
        assert_eq!((c[0].video_id.as_deref(), c[0].audio_id.as_deref(), c[0].est_size), (Some("303"), Some("251"), Some(80900)));
        assert_eq!((c[1].video_id.as_deref(), c[1].audio_id.as_deref(), c[1].est_size), (Some("137"), Some("140"), Some(51000)));
        assert_eq!((c[2].video_id.as_deref(), c[2].audio_id.as_deref()), (Some("hls-720"), None), "muxed HLS needs no audio");
        assert_eq!((c[3].container.as_str(), c[3].est_size), ("mp4", Some(7000)));
        assert_eq!((c[4].audio_id.as_deref(), c[4].container.as_str()), (Some("140"), "m4a"));
    }

    #[test]
    fn drm_only_and_top_level_flag() {
        let all = r#"{"title":"t","formats":[{"format_id":"a","url":"https://c/a","has_drm":true,"vcodec":"avc1","height":720}]}"#;
        let i = parse_info(all).unwrap();
        assert!(i.drm && choices(&i).is_empty());
        // yt-dlp already removed the DRM formats and left only `_has_drm`
        let i = parse_info(r#"{"title":"t","_has_drm":true,"formats":[]}"#).unwrap();
        assert!(i.drm);
        let i = parse_info(r#"{"title":"t","_has_drm":true,"formats":[{"url":"https://c/a","vcodec":"avc1","height":1}]}"#).unwrap();
        assert!(!i.drm);
    }

    #[test]
    fn limits_and_odd_shapes() {
        let fs: Vec<String> =
            (0..70).map(|n| format!(r#"{{"format_id":"{n}","url":"https://c/{n}","vcodec":"avc1","acodec":"none","height":{n}}}"#)).collect();
        let i = parse_info(&format!(r#"{{"formats":[{}]}}"#, fs.join(","))).unwrap();
        assert_eq!((i.formats.len(), i.truncated), (64, true));
        assert_eq!(choices(&i).len(), MAX_CHOICES);
        assert_eq!(parse_info(&" ".repeat(MAX_JSON + 1)), Err("too-large".into()));
        assert_eq!(parse_info("[1]"), Err("bad-json".into()));
        let i = parse_info(r#"{"title":"one","url":"https://c/v.mp4","ext":"mp4","height":"720","has_drm":"maybe"}"#).unwrap();
        assert_eq!((i.formats.len(), i.formats[0].protocol.as_str(), i.formats[0].height, i.drm), (1, "https", None, false));
    }
}
