// SPDX-License-Identifier: RPL-1.5
//! Import (.ef2 / .txt / GDM .json), export JSON, batch patterns (F10, I-03, I-04). Every input here is
//! untrusted: bounded size and counts, http(s) URLs only, names through the sanitizer later, and an import
//! never carries a hook, a folder, a cookie, a password or a setting (a settings backup is restored only by
//! its own button, through `merge_settings`, which drops hooks and paths).
use serde::{Deserialize, Serialize};

pub const MAX_FILE: u64 = 50 * 1024 * 1024;
pub const MAX_ITEMS: usize = 100_000;
pub const BATCH_CONFIRM: u64 = 10_000;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct Item {
    pub url: String,
    pub name: Option<String>,
    pub referrer: Option<String>,
    pub description: Option<String>,
}

fn http(s: &str) -> bool {
    s.len() <= gdm_bridge::MAX_URL
        && (s.starts_with("https://") || s.starts_with("http://"))
        && !s.chars().any(|c| c.is_control() || c == ' ')
        && url::Url::parse(s).is_ok_and(|u| u.username().is_empty() && u.password().is_none())
}

fn short(s: &str, n: usize) -> Option<String> {
    let s = s.trim();
    (!s.is_empty() && s.len() <= n && !s.chars().any(|c| c.is_control())).then(|| s.to_string())
}

fn push(out: &mut Vec<Item>, it: Item) -> Result<(), String> {
    if out.len() >= MAX_ITEMS {
        return Err("too-many".into());
    }
    out.push(it);
    Ok(())
}

/// IDM .ef2: blocks `<` URL, then `key: value` lines (referer, User-Agent, filename, cookie, postdata,
/// username, password ...), then `>`. Only the URL, referer and file name are taken.
pub fn parse_ef2(text: &str) -> Result<Vec<Item>, String> {
    let mut out = vec![];
    let mut cur: Option<Item> = None;
    for line in text.lines().map(str::trim) {
        if line == "<" {
            cur = Some(Item::default());
        } else if line == ">" {
            if let Some(it) = cur.take() {
                if http(&it.url) {
                    push(&mut out, it)?;
                }
            }
        } else if let Some(it) = cur.as_mut() {
            if it.url.is_empty() {
                it.url = line.to_string();
            } else if let Some((k, v)) = line.split_once(':') {
                match k.trim().to_ascii_lowercase().as_str() {
                    "referer" => it.referrer = short(v, gdm_bridge::MAX_URL).filter(|r| http(r)),
                    "filename" => it.name = short(v, gdm_bridge::MAX_NAME),
                    _ => {} // cookie, postdata, username, password, User-Agent: never imported
                }
            }
        }
    }
    Ok(out)
}

/// one URL per line; anything else is skipped
pub fn parse_txt(text: &str) -> Result<Vec<Item>, String> {
    let mut out = vec![];
    for l in text.lines().map(str::trim).filter(|l| http(l)) {
        push(&mut out, Item { url: l.to_string(), ..Default::default() })?;
    }
    Ok(out)
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Export {
    pub gdm_export: u32,
    pub app_version: String,
    pub jobs: Vec<Item>,
    /// settings backup (no hooks, no secrets; passwords are in Credential Manager and never exported, P-01)
    pub settings: Option<serde_json::Value>,
}

pub fn parse_json(text: &str) -> Result<(Vec<Item>, Option<serde_json::Value>), String> {
    let e: Export = serde_json::from_str(text).map_err(|_| "bad-json")?;
    if e.gdm_export != 1 {
        return Err("bad-json".into());
    }
    if e.jobs.len() > MAX_ITEMS {
        return Err("too-many".into());
    }
    let jobs = e
        .jobs
        .into_iter()
        .filter(|j| http(&j.url))
        .map(|j| Item {
            url: j.url,
            name: j.name.as_deref().and_then(|n| short(n, gdm_bridge::MAX_NAME)),
            referrer: j.referrer.filter(|r| http(r)),
            description: j.description.as_deref().and_then(|d| short(d, 2000)),
        })
        .collect();
    Ok((jobs, e.settings))
}

/// by extension: .ef2, .json, anything else = text
pub fn parse_file(name: &str, bytes: &[u8]) -> Result<(Vec<Item>, Option<serde_json::Value>), String> {
    if bytes.len() as u64 > MAX_FILE {
        return Err("too-large".into());
    }
    // .ef2 from IDM is often UTF-16 LE with a BOM
    let text = if bytes.starts_with(&[0xFF, 0xFE]) {
        let u: Vec<u16> = bytes[2..].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        String::from_utf16_lossy(&u)
    } else {
        String::from_utf8_lossy(bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes)).into_owned()
    };
    let l = name.to_ascii_lowercase();
    if l.ends_with(".ef2") {
        Ok((parse_ef2(&text)?, None))
    } else if l.ends_with(".json") {
        parse_json(&text)
    } else {
        Ok((parse_txt(&text)?, None))
    }
}

/// batch download (IDM "Add batch download"): one `*` in the URL, replaced by numbers `from..=to` (zero-padded
/// to `width`) or by letters `a..=z` / `A..=Z`
#[derive(Deserialize, Clone, Debug)]
pub struct Batch {
    pub pattern: String,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub width: usize,
}

impl Batch {
    fn range(&self) -> Result<(u64, u64, Option<u8>), String> {
        if self.pattern.matches('*').count() != 1 || !http(&self.pattern.replace('*', "1")) {
            return Err("bad-pattern".into());
        }
        let (f, t) = (self.from.trim(), self.to.trim());
        if let (Ok(a), Ok(b)) = (f.parse::<u64>(), t.parse::<u64>()) {
            return if a <= b { Ok((a, b, None)) } else { Err("bad-range".into()) };
        }
        let one = |s: &str| (s.len() == 1 && s.as_bytes()[0].is_ascii_alphabetic()).then(|| s.as_bytes()[0]);
        match (one(f), one(t)) {
            (Some(a), Some(b)) if a <= b && a.is_ascii_lowercase() == b.is_ascii_lowercase() => Ok((0, (b - a) as u64, Some(a))),
            _ => Err("bad-range".into()),
        }
    }

    /// I-04: the count is known before anything is created
    pub fn count(&self) -> Result<u64, String> {
        let (a, b, _) = self.range()?;
        Ok(b - a + 1)
    }

    pub fn urls(&self, limit: usize) -> Result<Vec<String>, String> {
        let (a, b, letter) = self.range()?;
        let w = self.width.min(20);
        Ok((a..=b)
            .take(limit)
            .map(|i| {
                let v = match letter {
                    Some(c) => ((c + i as u8) as char).to_string(),
                    None => format!("{i:0w$}"),
                };
                self.pattern.replace('*', &v)
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i03_ef2_takes_url_referer_name_only() {
        let ef2 = "<\r\nhttps://a.example/x.zip\r\nreferer: https://a.example/page\r\nUser-Agent: Mozilla\r\ncookie: SID=secret\r\npassword: hunter2\r\nfilename: My File.zip\r\n>\r\n<\r\nfile:///C:/Windows/win.ini\r\n>\r\n<\r\nhttps://user:pw@b.example/y\r\n>\r\n";
        let v = parse_ef2(ef2).unwrap();
        assert_eq!(v.len(), 1, "file: and userinfo URLs are dropped");
        assert_eq!(v[0].url, "https://a.example/x.zip");
        assert_eq!(v[0].referrer.as_deref(), Some("https://a.example/page"));
        assert_eq!(v[0].name.as_deref(), Some("My File.zip"));
        let dbg = format!("{v:?}");
        assert!(!dbg.contains("secret") && !dbg.contains("hunter2"), "no cookie or password imported");
        // UTF-16 LE with BOM (IDM writes this)
        let mut b = vec![0xFF, 0xFE];
        b.extend("<\nhttp://c.example/z.iso\n>\n".encode_utf16().flat_map(|u| u.to_le_bytes()));
        assert_eq!(parse_file("list.EF2", &b).unwrap().0[0].url, "http://c.example/z.iso");
    }

    #[test]
    fn i03_txt_json_and_limits() {
        let v = parse_txt("https://a/1\n  not a url\nftp://x/y\nhttp://b/2 \njavascript:alert(1)\n").unwrap();
        assert_eq!(v.iter().map(|i| i.url.as_str()).collect::<Vec<_>>(), vec!["https://a/1", "http://b/2"]);
        let j = r#"{"gdm_export":1,"jobs":[{"url":"https://a/x","name":"n","dir":"C:\\Windows","hook":{"program":"calc"}},{"url":"file:///etc"}],"settings":{"lang":"en"}}"#;
        let (jobs, set) = parse_json(j).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(set.unwrap()["lang"], "en");
        assert!(parse_json(r#"{"gdm_export":2,"jobs":[]}"#).is_err());
        assert_eq!(parse_file("x.txt", &vec![b'a'; (MAX_FILE + 1) as usize]).unwrap_err(), "too-large");
        let many = "https://a/b\n".repeat(MAX_ITEMS + 1);
        assert_eq!(parse_txt(&many).unwrap_err(), "too-many");
    }

    /// I-03 "fuzz": random and mutated inputs never panic and every result obeys the limits
    #[test]
    fn i03_randomized_inputs_never_panic() {
        let seeds = ["<\nhttps://a/x\nreferer: https://r/\nfilename: f\n>\n", r#"{"gdm_export":1,"jobs":[{"url":"https://a/x"}]}"#, "https://a/1\n"];
        let mut x: u64 = 0x9E3779B97F4A7C15;
        let mut rnd = || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        for i in 0..20_000 {
            let mut b = seeds[i % 3].as_bytes().to_vec();
            for _ in 0..(rnd() % 8) {
                let p = (rnd() as usize) % (b.len() + 1);
                match rnd() % 3 {
                    0 if !b.is_empty() => {
                        b.remove(p.min(b.len() - 1));
                    }
                    1 => b.insert(p, rnd() as u8),
                    _ => b.insert(p, b"<>:*\n\"{}[]"[(rnd() % 10) as usize]),
                }
            }
            for name in ["a.ef2", "a.json", "a.txt"] {
                if let Ok((v, _)) = parse_file(name, &b) {
                    assert!(
                        v.len() <= MAX_ITEMS && v.iter().all(|it| http(&it.url) && it.name.as_ref().is_none_or(|n| n.len() <= gdm_bridge::MAX_NAME))
                    );
                }
            }
        }
    }

    #[test]
    fn i04_batch_count_and_expansion() {
        let b = Batch { pattern: "https://x.example/img*.jpg".into(), from: "8".into(), to: "11".into(), width: 3 };
        assert_eq!(b.count().unwrap(), 4);
        assert_eq!(
            b.urls(10).unwrap(),
            vec!["https://x.example/img008.jpg", "https://x.example/img009.jpg", "https://x.example/img010.jpg", "https://x.example/img011.jpg"]
        );
        let l = Batch { pattern: "http://x/*.bin".into(), from: "a".into(), to: "c".into(), width: 0 };
        assert_eq!(l.urls(10).unwrap(), vec!["http://x/a.bin", "http://x/b.bin", "http://x/c.bin"]);
        let huge = Batch { pattern: "http://x/*".into(), from: "1".into(), to: "1000000000".into(), width: 0 };
        assert_eq!(huge.count().unwrap(), 1_000_000_000, "known before anything is created");
        assert!(huge.count().unwrap() > BATCH_CONFIRM);
        assert_eq!(huge.urls(3).unwrap().len(), 3);
        for bad in [("http://x/**", "1", "2"), ("http://x/a", "1", "2"), ("file:///*", "1", "2"), ("http://x/*", "5", "1"), ("http://x/*", "a", "Z")]
        {
            assert!(Batch { pattern: bad.0.into(), from: bad.1.into(), to: bad.2.into(), width: 0 }.count().is_err(), "{bad:?}");
        }
    }
}
