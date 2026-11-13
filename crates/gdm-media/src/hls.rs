// SPDX-License-Identifier: RPL-1.5
//! HLS playlist reader (RFC 8216) for GDM's own HLS fetcher, with the M-03 limits and the
//! M-04 DRM refusal. Errors: "too-large", "not-m3u8", "limit", "drm", "unsupported", "bad-uri", "bad-playlist".

use url::Url;

pub const MAX_TEXT: usize = 5 * 1024 * 1024;
/// More variants than this is refused ("limit"), not truncated: a real master has a handful.
pub const MAX_VARIANTS: usize = 64;
pub const MAX_SEGMENTS: usize = 200_000;
/// Master -> master nesting the caller may follow; the parser sees one playlist at a time.
pub const MAX_DEPTH: usize = 3;

const DRM_KEYFORMATS: &[&str] = &["urn:uuid:edef8ba9-79d6-4ace-a3c8-27dcd51d21ed", "com.microsoft.playready", "com.apple.streamingkeydelivery"];

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub uri: Url,
    pub bandwidth: Option<u64>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Key {
    /// Whole-segment AES-128. v1.0 reports "unsupported: encrypted HLS" (ADR-4: later).
    Aes128 { uri: Url, iv: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // one value per parse, never stored in bulk
pub enum Playlist {
    Master {
        variants: Vec<Variant>,
    },
    Media {
        segments: Vec<Url>,
        /// `#EXT-X-MAP` init segment (fMP4 HLS).
        init: Option<Url>,
        target_duration: Option<f64>,
        /// false = live: the caller asks the user for a duration.
        endlist: bool,
        /// First AES-128 key seen.
        encrypted: Option<Key>,
    },
}

/// `KEY=value,KEY="quoted, value"` -> pairs, quotes removed.
fn attrs(s: &str) -> Vec<(&str, &str)> {
    let (mut out, mut start, mut quoted) = (Vec::new(), 0, false);
    for (i, c) in s.char_indices().chain([(s.len(), ',')]) {
        match c {
            '"' => quoted = !quoted,
            ',' if !quoted => {
                if let Some((k, v)) = s[start..i].split_once('=') {
                    out.push((k.trim(), v.trim().trim_matches('"')));
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    out
}

fn get<'a>(a: &[(&str, &'a str)], k: &str) -> Option<&'a str> {
    a.iter().find(|(n, _)| n.eq_ignore_ascii_case(k)).map(|(_, v)| *v)
}

/// Any URI in the playlist: resolved against `base`, must end up http(s), else the whole playlist is refused.
fn resolve(base: &Url, u: &str) -> Result<Url, String> {
    match base.join(u) {
        Ok(u) if matches!(u.scheme(), "http" | "https") && u.host().is_some() => Ok(u),
        _ => Err("bad-uri".into()),
    }
}

pub fn parse_m3u8(text: &str, base: &Url) -> Result<Playlist, String> {
    if text.len() > MAX_TEXT {
        return Err("too-large".into());
    }
    let mut lines = text.trim_start_matches('\u{feff}').lines().map(str::trim).filter(|l| !l.is_empty());
    if lines.next() != Some("#EXTM3U") {
        return Err("not-m3u8".into());
    }
    let (mut variants, mut segments) = (Vec::new(), Vec::new());
    let (mut pending, mut init, mut target, mut endlist, mut encrypted) = (None::<Variant>, None, None, false, None);
    for l in lines {
        let (tag, rest) = l.split_once(':').unwrap_or((l, ""));
        match tag {
            "#EXT-X-KEY" | "#EXT-X-SESSION-KEY" => {
                let a = attrs(rest);
                let method = get(&a, "METHOD").unwrap_or("");
                let keyformat = get(&a, "KEYFORMAT").unwrap_or("").to_ascii_lowercase();
                if method.starts_with("SAMPLE-AES") || DRM_KEYFORMATS.contains(&keyformat.as_str()) {
                    return Err("drm".into());
                }
                match method {
                    "NONE" => {}
                    "AES-128" if encrypted.is_none() => {
                        let uri = resolve(base, get(&a, "URI").ok_or("bad-playlist")?)?;
                        encrypted = Some(Key::Aes128 { uri, iv: get(&a, "IV").map(str::to_string) });
                    }
                    "AES-128" => {}
                    _ => return Err("unsupported".into()),
                }
            }
            "#EXT-X-STREAM-INF" => {
                let a = attrs(rest);
                pending = Some(Variant {
                    uri: base.clone(),
                    bandwidth: get(&a, "BANDWIDTH").and_then(|b| b.parse().ok()),
                    height: get(&a, "RESOLUTION").and_then(|r| r.split_once(['x', 'X'])).and_then(|(_, h)| h.parse().ok()),
                });
            }
            "#EXT-X-MAP" => {
                let a = attrs(rest);
                if get(&a, "BYTERANGE").is_some() {
                    return Err("unsupported".into());
                }
                let u = resolve(base, get(&a, "URI").ok_or("bad-playlist")?)?;
                if init.as_ref().is_some_and(|i| *i != u) {
                    return Err("unsupported".into()); // ponytail: one init segment only; per-discontinuity maps when a real stream needs it
                }
                init = Some(u);
            }
            // ponytail: byte-range segments refused in v1.0; add ranged GETs when a site needs it
            "#EXT-X-BYTERANGE" => return Err("unsupported".into()),
            "#EXT-X-TARGETDURATION" => target = rest.parse().ok(),
            "#EXT-X-ENDLIST" => endlist = true,
            _ if l.starts_with('#') => {} // other tags and comments
            _ => {
                let uri = resolve(base, l)?;
                if let Some(mut v) = pending.take() {
                    if variants.len() == MAX_VARIANTS {
                        return Err("limit".into());
                    }
                    v.uri = uri;
                    variants.push(v);
                } else {
                    if segments.len() == MAX_SEGMENTS {
                        return Err("limit".into());
                    }
                    segments.push(uri);
                }
            }
        }
    }
    match (variants.is_empty(), segments.is_empty() && init.is_none()) {
        (false, true) => Ok(Playlist::Master { variants }),
        (true, _) => Ok(Playlist::Media { segments, init, target_duration: target, endlist, encrypted }),
        (false, false) => Err("bad-playlist".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Url {
        Url::parse("https://cdn.example/v/index.m3u8").unwrap()
    }

    #[test]
    fn master() {
        let t = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1280000,CODECS=\"avc1.4d401f,mp4a.40.2\",RESOLUTION=1280x720\n720.m3u8\n\
                 #EXT-X-STREAM-INF:BANDWIDTH=640000\r\nhttps://other.example/360.m3u8\n";
        let Playlist::Master { variants } = parse_m3u8(t, &base()).unwrap() else { panic!() };
        assert_eq!(variants.len(), 2);
        assert_eq!(
            (variants[0].uri.as_str(), variants[0].bandwidth, variants[0].height),
            ("https://cdn.example/v/720.m3u8", Some(1280000), Some(720))
        );
        assert_eq!((variants[1].uri.as_str(), variants[1].height), ("https://other.example/360.m3u8", None));
    }

    #[test]
    fn m03_limits() {
        let mut t = String::from("#EXTM3U\n");
        for n in 0..65 {
            t += &format!("#EXT-X-STREAM-INF:BANDWIDTH={n}\nv{n}.m3u8\n");
        }
        assert_eq!(parse_m3u8(&t, &base()), Err("limit".into()), "65 variants refused");
        let mut t = String::from("#EXTM3U\n#EXT-X-TARGETDURATION:2\n");
        for n in 0..200_001 {
            t += &format!("#EXTINF:2,\ns{n}.ts\n");
        }
        assert_eq!(parse_m3u8(&t, &base()), Err("limit".into()), "200001 segments refused");
        assert_eq!(parse_m3u8(&"#EXTM3U\n".repeat(MAX_TEXT / 8 + 1), &base()), Err("too-large".into()));
        assert_eq!(parse_m3u8("<html>", &base()), Err("not-m3u8".into()));
    }

    #[test]
    fn media_vod_and_live() {
        let t = "#EXTM3U\n#EXT-X-TARGETDURATION:6\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:6,\na.m4s\n#EXTINF:6,\nb.m4s\n#EXT-X-ENDLIST\n";
        let Playlist::Media { segments, init, target_duration, endlist, encrypted } = parse_m3u8(t, &base()).unwrap() else { panic!() };
        assert_eq!((segments.len(), target_duration, endlist, encrypted), (2, Some(6.0), true, None));
        assert_eq!(init.unwrap().as_str(), "https://cdn.example/v/init.mp4");
        let live = "#EXTM3U\n#EXT-X-TARGETDURATION:4\n#EXTINF:4,\nlive1.ts\n";
        assert!(matches!(parse_m3u8(live, &base()).unwrap(), Playlist::Media { endlist: false, .. }));
    }

    #[test]
    fn m04_keys() {
        let drm = "#EXTM3U\n#EXT-X-KEY:METHOD=SAMPLE-AES,URI=\"data:text/plain;base64,AAAA\",KEYFORMAT=\"urn:uuid:edef8ba9-79d6-4ace-a3c8-27dcd51d21ed\"\n#EXTINF:4,\na.ts\n";
        assert_eq!(parse_m3u8(drm, &base()), Err("drm".into()));
        for k in [
            "METHOD=SAMPLE-AES,URI=\"skd://x\"",
            "METHOD=SAMPLE-AES-CTR,URI=\"https://k/1\"",
            "METHOD=AES-128,URI=\"https://k/1\",KEYFORMAT=\"com.microsoft.playready\"",
            "METHOD=SAMPLE-AES,URI=\"skd://x\",KEYFORMAT=\"com.apple.streamingkeydelivery\"",
        ] {
            assert_eq!(parse_m3u8(&format!("#EXTM3U\n#EXT-X-KEY:{k}\na.ts\n"), &base()), Err("drm".into()), "{k}");
        }
        let session = "#EXTM3U\n#EXT-X-SESSION-KEY:METHOD=SAMPLE-AES,URI=\"skd://x\"\n#EXT-X-STREAM-INF:BANDWIDTH=1\nv.m3u8\n";
        assert_eq!(parse_m3u8(session, &base()), Err("drm".into()));
        let aes = "#EXTM3U\n#EXT-X-KEY:METHOD=NONE\n#EXT-X-KEY:METHOD=AES-128,URI=\"key.bin\",IV=0x0102\n#EXTINF:4,\na.ts\n#EXT-X-ENDLIST\n";
        let Playlist::Media { encrypted, .. } = parse_m3u8(aes, &base()).unwrap() else { panic!() };
        assert_eq!(encrypted, Some(Key::Aes128 { uri: Url::parse("https://cdn.example/v/key.bin").unwrap(), iv: Some("0x0102".into()) }));
        assert!(matches!(parse_m3u8("#EXTM3U\n#EXT-X-KEY:METHOD=NONE\na.ts\n", &base()).unwrap(), Playlist::Media { encrypted: None, .. }));
    }

    #[test]
    fn bad_uris_and_unsupported() {
        for t in [
            "#EXTM3U\n#EXTINF:4,\nfile:///C:/Windows/win.ini\n",
            "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1\ndata:text/plain,x\n",
            "#EXTM3U\n#EXT-X-MAP:URI=\"file:///x\"\na.ts\n",
            "#EXTM3U\n#EXT-X-KEY:METHOD=AES-128,URI=\"file:///k\"\na.ts\n",
        ] {
            assert_eq!(parse_m3u8(t, &base()), Err("bad-uri".into()), "{t}");
        }
        assert_eq!(parse_m3u8("#EXTM3U\n#EXT-X-BYTERANGE:100@0\na.ts\n", &base()), Err("unsupported".into()));
        assert_eq!(parse_m3u8("#EXTM3U\n#EXT-X-KEY:METHOD=FOO\na.ts\n", &base()), Err("unsupported".into()));
        assert_eq!(parse_m3u8("#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1\nv.m3u8\nseg.ts\n", &base()), Err("bad-playlist".into()));
    }
}
