// SPDX-License-Identifier: RPL-1.5
//! F07 media helpers: yt-dlp argv and `-J` parsing, the quality list, the HLS reader,
//! DASH/DRM detection and the media file name rule. Pure logic, no I/O.

mod hls;
mod ytdlp;

pub use hls::{parse_m3u8, Key, Playlist, Variant, MAX_DEPTH, MAX_SEGMENTS, MAX_TEXT, MAX_VARIANTS};
pub use ytdlp::{choices, parse_info, ytdlp_args, Choice, Format, Info, FORBIDDEN, MAX_CHOICES, MAX_FORMATS, MAX_JSON};

/// Text looks like a DASH manifest (an `<MPD` element). Detection only: v1.0 has no DASH fetcher.
pub fn is_dash_mpd(text: &str) -> bool {
    text.contains("<MPD") || text.contains(":MPD ")
}

/// Substring scan, no XML parser: true on any `ContentProtection` element or any `pssh` box element.
/// `urn:mpeg:dash:mp4protection:2011` alone is counted too, because it declares CENC-encrypted
/// media that GDM cannot play either way. Limit: an XML comment mentioning ContentProtection also
/// counts (a false refusal, never a false pass).
pub fn mpd_has_drm(text: &str) -> bool {
    text.contains("ContentProtection") || text.contains(":pssh") || text.contains("<pssh")
}

/// Extensions GDM may write for media (M-02, CVE-2024-38519: yt-dlp once let a site pick `.dll`, `.conf`...).
pub const MEDIA_EXTS: &[&str] = &["mp4", "mkv", "webm", "m4a", "mp3", "opus", "srt", "vtt"];

const MAX_TITLE: usize = 150;

/// `"<title>.<ext>"` with an allow-listed ext. Title: at most 150 chars; control chars, `\ / : * ? " < > |`
/// and leading/trailing dots and spaces become `_`; Windows device names get a `_` prefix.
/// The app runs its own sanitizer on top.
pub fn media_name(title: &str, ext: &str) -> Result<String, String> {
    let ext = ext.to_ascii_lowercase();
    if !MEDIA_EXTS.contains(&ext.as_str()) {
        return Err("bad-ext".into());
    }
    let mut t: Vec<char> = title.chars().take(MAX_TITLE).map(|c| if c.is_control() || r#"\/:*?"<>|"#.contains(c) { '_' } else { c }).collect();
    for i in (0..t.len()).take_while(|&i| matches!(t[i], '.' | ' ')).collect::<Vec<_>>() {
        t[i] = '_';
    }
    for i in (0..t.len()).rev().take_while(|&i| matches!(t[i], '.' | ' ')).collect::<Vec<_>>() {
        t[i] = '_';
    }
    let mut t: String = t.into_iter().collect();
    if t.is_empty() {
        t = "media".into();
    }
    let stem = t.split('.').next().unwrap_or("").trim_end().to_ascii_uppercase();
    let device = ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
        || (stem.len() == 4 && (stem.starts_with("COM") || stem.starts_with("LPT")) && stem.as_bytes()[3].is_ascii_digit());
    if device {
        t.insert(0, '_');
    }
    Ok(format!("{t}.{ext}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dash_detect() {
        let clear = r#"<?xml version="1.0"?><MPD xmlns="urn:mpeg:dash:schema:mpd:2011"><Period/></MPD>"#;
        assert!(is_dash_mpd(clear) && !mpd_has_drm(clear));
        let wv = r#"<MPD><AdaptationSet><ContentProtection schemeIdUri="urn:mpeg:dash:mp4protection:2011" value="cenc"/>
            <ContentProtection schemeIdUri="urn:uuid:edef8ba9-79d6-4ace-a3c8-27dcd51d21ed"><cenc:pssh>AAAA</cenc:pssh></ContentProtection></AdaptationSet></MPD>"#;
        assert!(mpd_has_drm(wv));
        assert!(mpd_has_drm(r#"<MPD><ContentProtection schemeIdUri="urn:mpeg:dash:mp4protection:2011" value="cenc"/></MPD>"#));
        assert!(!is_dash_mpd("#EXTM3U\n"));
    }

    #[test]
    fn m02_names() {
        for e in ["dll", "conf", "exe", "bat", "lnk", "url", "", "mp4.exe", "mp4 "] {
            assert_eq!(media_name("x", e), Err("bad-ext".into()), "{e}");
        }
        assert_eq!(media_name("My: clip?", "MP4").unwrap(), "My_ clip_.mp4");
        assert_eq!(media_name(" ..a/b\\c.. ", "mkv").unwrap(), "___a_b_c___.mkv");
        assert_eq!(media_name("a\r\nb\t", "srt").unwrap(), "a__b_.srt");
        assert_eq!(media_name("", "m4a").unwrap(), "media.m4a");
        assert_eq!(media_name("con", "mp4").unwrap(), "_con.mp4");
        assert_eq!(media_name("COM1", "mp4").unwrap(), "_COM1.mp4");
        assert_eq!(media_name("Console", "mp4").unwrap(), "Console.mp4");
        let long = media_name(&"é".repeat(400), "webm").unwrap();
        assert_eq!(long.chars().count(), 150 + 5);
    }

    /// 20k mutated inputs each: the parsers return Ok or Err, never panic.
    #[test]
    fn mutation_never_panics() {
        let base = url::Url::parse("https://cdn.example/a/b.m3u8").unwrap();
        let m3u8_seeds = [
            "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1,RESOLUTION=1x2\nv.m3u8\n",
            "#EXTM3U\n#EXT-X-TARGETDURATION:6\n#EXT-X-MAP:URI=\"i.mp4\"\n#EXT-X-KEY:METHOD=AES-128,URI=\"k\",IV=0x1\n#EXTINF:6,\na.ts\n#EXT-X-ENDLIST\n",
        ];
        let json_seeds = [
            r#"{"title":"t","_has_drm":false,"formats":[{"format_id":"1","url":"https://c/1","vcodec":"avc1","acodec":"none","height":720,"tbr":1.5,"filesize":9,"http_headers":{"Cookie":"x","A":"b"}},{"format_id":"2","url":"https://c/2","vcodec":"none","acodec":"opus","has_drm":true}]}"#,
        ];
        let tokens: &[&str] =
            &["\"", ",", "=", ":", "\n", "{", "}", "[", "]", "#EXT-X-KEY:", "#EXT-X-STREAM-INF:", "\u{feff}", "é", "-1", "1e400", "null", "x"];
        let mut seed = 0x9E37_79B9_7F4A_7C15u64;
        let mut rnd = move |n: usize| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed % n as u64) as usize
        };
        for i in 0..40_000 {
            let src = if i % 2 == 0 { m3u8_seeds[rnd(2)] } else { json_seeds[0] };
            let mut s = src.to_string();
            for _ in 0..1 + rnd(6) {
                let at = (0..=s.len()).filter(|&p| s.is_char_boundary(p)).nth(rnd(s.chars().count() + 1)).unwrap_or(0);
                match rnd(3) {
                    0 => s.insert_str(at, tokens[rnd(tokens.len())]),
                    1 => {
                        let end = (at..=s.len()).filter(|&p| s.is_char_boundary(p)).nth(rnd(8)).unwrap_or(s.len());
                        s.replace_range(at..end, "");
                    }
                    _ => {
                        let end = (at..=s.len()).filter(|&p| s.is_char_boundary(p)).nth(1).unwrap_or(s.len());
                        s.replace_range(at..end, &char::from(rnd(128) as u8).to_string());
                    }
                }
            }
            if i % 2 == 0 {
                let _ = parse_m3u8(&s, &base);
            } else if let Ok(info) = parse_info(&s) {
                let _ = choices(&info);
            }
        }
    }
}
