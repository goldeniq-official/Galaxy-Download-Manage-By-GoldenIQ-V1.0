// SPDX-License-Identifier: RPL-1.5
//! Site grabber (F09, P3). Explores a site from a start page and lists the files that match the project's
//! filters; the user ticks what to add. Bounded by design (G-01): depth, page/request/byte/time/URL budgets,
//! query-variant and path-loop guards, per-host spacing (1 request/s by default) and robots.txt (RFC 9309).
//! GET only; links matching the "pages to avoid" list are never requested (G-02). Every link goes through
//! the W-02 network policy and other domains only through the project's filter (G-04). Pages are scanned as
//! text, never rendered; script text is scanned for quoted URL literals only when the user turns JS parsing
//! on, nothing is executed (G-05). Mirror folders are created by handle and refuse junctions (G-03).
//! File links are listed, not fetched: their size is known only when downloaded.
use crate::fsx::{Dir, FsError};
use crate::net::{self, Creds, Policy};
use crate::{name, Fail, NewJob};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use url::Url;

pub const AGENT: &str = "galaxydownloadmanager";
/// default "pages to avoid" (G-02 + IDM "logout pages to avoid"): matched in the decoded path and query
const AVOID: &[&str] =
    &["logout", "log-out", "log_out", "logoff", "log-off", "signout", "sign-out", "sign_out", "signoff", "delete", "remove", "unsubscribe"];
const PAGE_EXT: &[&str] = &["html", "htm", "xhtml", "shtml", "php", "asp", "aspx", "jsp", "cgi", "cfm", "pl"];
const MAX_LINKS: usize = 20_000;
const MAX_SKIPPED: usize = 2_000;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Limits {
    pub pages: u32,
    pub requests: u32,
    pub bytes: u64,
    pub page_bytes: u64,
    pub secs: u64,
    pub urls: u32,
    pub query_variants: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Limits { pages: 1_000, requests: 5_000, bytes: 100 << 20, page_bytes: 2 << 20, secs: 15 * 60, urls: 50_000, query_variants: 50 }
    }
}

impl Limits {
    /// hard ceilings whatever the user typed (G-01: pages <= 5,000)
    pub fn clamp(&mut self) {
        self.pages = self.pages.clamp(1, 5_000);
        self.requests = self.requests.clamp(1, 20_000);
        self.bytes = self.bytes.clamp(1, 1 << 30);
        self.page_bytes = self.page_bytes.clamp(1024, 8 << 20);
        self.secs = self.secs.clamp(1, 2 * 3600);
        self.urls = self.urls.clamp(1, 100_000);
        self.query_variants = self.query_variants.clamp(1, 10_000);
    }
}

/// one project (the 4 wizard steps). Lists are comma separated, `*` is a wildcard.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// step 1: start page; `typed` = the user typed it (a LAN start host is then allowed, W-02)
    pub start: String,
    pub typed: bool,
    /// Authorization/Cookie for the start page's origin only (W-01); set by the app, never from the page
    #[serde(skip)]
    pub creds: Vec<(String, String)>,
    pub avoid: String,
    /// step 3: explore
    pub depth: u32,
    pub other_depth: u32,
    pub domains: String,
    pub subdomains: bool,
    pub no_parent: bool,
    pub include: String,
    pub exclude: String,
    pub robots: bool,
    pub js: bool,
    pub delay_ms: u64,
    /// step 4: file types
    pub types: String,
    pub limits: Limits,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            start: String::new(),
            typed: true,
            creds: vec![],
            avoid: String::new(),
            depth: 3,
            other_depth: 0,
            domains: String::new(),
            subdomains: false,
            no_parent: true,
            include: String::new(),
            exclude: String::new(),
            robots: true,
            js: false,
            delay_ms: 1_000,
            types: "*".into(),
            limits: Limits::default(),
        }
    }
}

fn list(s: &str) -> Vec<String> {
    s.split([',', '\n', ';']).map(|x| x.trim().to_lowercase()).filter(|x| !x.is_empty()).collect()
}

impl Config {
    /// the file name matches the type filter
    pub fn wants(&self, name: &str) -> bool {
        list(&self.types).iter().any(|p| glob(p, name))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Found {
    pub url: String,
    /// the page that linked it (sent as Referer)
    pub referrer: String,
    pub name: String,
    pub size: Option<u64>,
    /// an earlier file has the same name (IDM "hide duplicates")
    pub dup: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Skip {
    pub url: String,
    /// avoid | robots | private | other-domain | filter | parent | loop | query-variants | depth | http-<code> | error
    pub reason: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Report {
    pub files: Vec<Found>,
    pub pages: u32,
    pub requests: u32,
    pub bytes: u64,
    /// done | pages | requests | bytes | time | urls | cancel
    pub stop: String,
    pub skipped: Vec<Skip>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Progress {
    pub pages: u32,
    pub requests: u32,
    pub files: usize,
    pub bytes: u64,
    pub url: String,
}

/// `*` matches anything (also `/`); case-insensitive; whole string
pub fn glob(pat: &str, s: &str) -> bool {
    wild(pat.to_lowercase().as_bytes(), s.to_lowercase().as_bytes())
}

/// classic wildcard match with one backtrack point: `*` = any run of bytes
fn wild(p: &[u8], s: &[u8]) -> bool {
    let (mut i, mut j, mut star, mut mark) = (0, 0, None, 0);
    while j < s.len() {
        if i < p.len() && p[i] == b'*' {
            star = Some(i);
            mark = j;
            i += 1;
        } else if i < p.len() && p[i] == s[j] {
            i += 1;
            j += 1;
        } else if let Some(st) = star {
            i = st + 1;
            mark += 1;
            j = mark;
        } else {
            return false;
        }
    }
    p[i..].iter().all(|&c| c == b'*')
}

/// decoded, lower-case "path?query"
fn decoded(u: &Url) -> String {
    let mut s = name::pct_decode(u.path());
    if let Some(q) = u.query() {
        s.push('?');
        s.push_str(&name::pct_decode(&q.replace('+', " ")));
    }
    s.to_lowercase()
}

/// G-02: a default avoid word, or a user pattern (`*` glob on "path?query", else a plain substring)
pub fn avoided(u: &Url, extra: &[String]) -> bool {
    let d = decoded(u);
    AVOID.iter().any(|w| d.contains(w)) || extra.iter().any(|p| if p.contains('*') { glob(p, &d) } else { d.contains(p.as_str()) })
}

// ---------------------------------------------------------------- robots.txt (RFC 9309)

#[derive(Clone, Debug, Default)]
pub struct Robots {
    /// (allow, normalised pattern)
    rules: Vec<(bool, String)>,
    blocked: bool,
    pub delay_ms: Option<u64>,
}

/// percent-encoding made comparable: unreserved bytes decoded, other escapes upper-case, non-ASCII encoded
fn pnorm(s: &str) -> String {
    let b = s.as_bytes();
    let mut o = String::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or("zz"), 16) {
                if v.is_ascii_alphanumeric() || b"-._~".contains(&v) {
                    o.push(v as char);
                } else {
                    o.push_str(&format!("%{v:02X}"));
                }
                i += 3;
                continue;
            }
        }
        if b[i] >= 0x80 {
            o.push_str(&format!("%{:02X}", b[i]));
        } else {
            o.push(b[i] as char);
        }
        i += 1;
    }
    o
}

impl Robots {
    /// 5xx, network failure, 429: the site is treated as fully disallowed (RFC 9309 §2.3.1.4)
    pub fn unreachable() -> Robots {
        Robots { blocked: true, ..Default::default() }
    }

    pub fn parse(body: &str) -> Robots {
        struct G {
            agents: Vec<String>,
            rules: Vec<(bool, String)>,
            delay: Option<u64>,
        }
        let mut groups: Vec<G> = vec![];
        let mut in_rules = true;
        for line in body.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            let Some((k, v)) = line.split_once(':') else { continue };
            let (k, v) = (k.trim().to_ascii_lowercase(), v.trim());
            match k.as_str() {
                "user-agent" => {
                    if in_rules || groups.is_empty() {
                        groups.push(G { agents: vec![], rules: vec![], delay: None });
                        in_rules = false;
                    }
                    let token = v.split(['/', ' ']).next().unwrap_or("").to_ascii_lowercase();
                    if let Some(g) = groups.last_mut() {
                        g.agents.push(token);
                    }
                }
                "allow" | "disallow" => {
                    in_rules = true;
                    if let (Some(g), false) = (groups.last_mut(), v.is_empty()) {
                        g.rules.push((k == "allow", pnorm(v)));
                    }
                }
                "crawl-delay" => {
                    in_rules = true;
                    if let (Some(g), Ok(s)) = (groups.last_mut(), v.parse::<f64>()) {
                        g.delay = Some((s.clamp(0.0, 30.0) * 1000.0) as u64);
                    }
                }
                _ => {}
            }
        }
        let ours: Vec<&G> = groups.iter().filter(|g| g.agents.iter().any(|a| a == AGENT)).collect();
        let pick = if ours.is_empty() { groups.iter().filter(|g| g.agents.iter().any(|a| a == "*")).collect() } else { ours };
        Robots {
            rules: pick.iter().flat_map(|g| g.rules.iter().cloned()).collect(),
            blocked: false,
            delay_ms: pick.iter().filter_map(|g| g.delay).max(),
        }
    }

    /// longest matching rule wins, Allow wins a tie; no rule = allowed
    pub fn allowed(&self, u: &Url) -> bool {
        if self.blocked {
            return false;
        }
        if u.path() == "/robots.txt" {
            return true;
        }
        let mut p = u.path().to_string();
        if let Some(q) = u.query() {
            p.push('?');
            p.push_str(q);
        }
        let p = pnorm(&p);
        let mut best: Option<(usize, bool)> = None;
        for (allow, pat) in &self.rules {
            let hit = match pat.strip_suffix('$') {
                Some(full) => wild(full.as_bytes(), p.as_bytes()),
                None => wild(format!("{pat}*").as_bytes(), p.as_bytes()),
            };
            if hit {
                let l = pat.len();
                best = match best {
                    Some((bl, ba)) if bl > l || (bl == l && ba) => Some((bl, ba)),
                    _ => Some((l, *allow)),
                };
            }
        }
        best.is_none_or(|(_, a)| a)
    }
}

// ---------------------------------------------------------------- link extraction (text scan, nothing runs)

fn entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut o = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        o.push_str(&rest[..i]);
        rest = &rest[i..];
        let end = rest.as_bytes().iter().take(12).position(|&c| c == b';'); // bytes: never cuts a char
        let rep = end.and_then(|e| {
            let ent = &rest[1..e];
            let c = match ent {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                _ => {
                    let n = if let Some(h) = ent.strip_prefix("#x").or_else(|| ent.strip_prefix("#X")) {
                        u32::from_str_radix(h, 16).ok()
                    } else {
                        ent.strip_prefix('#').and_then(|d| d.parse().ok())
                    };
                    n.and_then(char::from_u32)
                }
            };
            c.map(|c| (c, e))
        });
        match rep {
            Some((c, e)) => {
                o.push(c);
                rest = &rest[e + 1..];
            }
            None => {
                o.push('&');
                rest = &rest[1..];
            }
        }
    }
    o.push_str(rest);
    o
}

fn find_ci(h: &[u8], from: usize, n: &[u8]) -> Option<usize> {
    if from >= h.len() {
        return None;
    }
    h[from..].windows(n.len()).position(|w| w.eq_ignore_ascii_case(n)).map(|p| p + from)
}

struct Sink {
    out: Vec<Url>,
    seen: HashSet<String>,
}

impl Sink {
    fn push(&mut self, base: &Url, raw: &str) {
        let raw: String = raw.chars().filter(|c| !matches!(c, '\t' | '\n' | '\r')).collect();
        let raw = raw.trim();
        if raw.is_empty() || raw.starts_with('#') || self.out.len() >= MAX_LINKS {
            return;
        }
        if let Ok(mut u) = base.join(raw) {
            if matches!(u.scheme(), "http" | "https") {
                u.set_fragment(None);
                if self.seen.insert(u.to_string()) {
                    self.out.push(u);
                }
            }
        }
    }
}

fn srcset(s: &str) -> Vec<&str> {
    // WHATWG: skip whitespace and commas, URL = run of non-whitespace (trailing commas removed), descriptors
    // up to the next comma
    let mut v = vec![];
    let mut rest = s;
    loop {
        rest = rest.trim_start_matches(|c: char| c.is_whitespace() || c == ',');
        if rest.is_empty() {
            return v;
        }
        let e = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let url = &rest[..e];
        rest = &rest[e..];
        if url.ends_with(',') {
            v.push(url.trim_end_matches(','));
            continue;
        }
        v.push(url);
        rest = rest.find(',').map(|i| &rest[i + 1..]).unwrap_or("");
    }
}

fn css_urls(text: &str) -> Vec<String> {
    let b = text.as_bytes();
    let mut v = vec![];
    let mut i = 0;
    while let Some(p) = find_ci(b, i, b"url(") {
        let mut j = p + 4;
        while j < b.len() && b[j].is_ascii_whitespace() {
            j += 1;
        }
        let q = if j < b.len() && (b[j] == b'"' || b[j] == b'\'') { Some(b[j]) } else { None };
        if q.is_some() {
            j += 1;
        }
        let s = j;
        while j < b.len() && Some(b[j]) != q && !(q.is_none() && (b[j] == b')' || b[j].is_ascii_whitespace())) && b[j] != b'\n' {
            j += 1;
        }
        v.push(text[s..j].to_string());
        i = j.max(p + 4);
    }
    v
}

/// a quoted literal that names a resource: absolute http(s) or root/relative path, no spaces, last segment
/// with an extension
fn looks_url(s: &str) -> bool {
    if s.len() < 3 || s.len() > 2048 || s.chars().any(|c| c.is_whitespace() || c == '\\' || c == '"' || c == '\'') {
        return false;
    }
    if !(s.starts_with("http://") || s.starts_with("https://") || s.starts_with('/')) {
        return false;
    }
    let path = s.split(['?', '#']).next().unwrap_or("");
    let last = path.rsplit('/').next().unwrap_or("");
    match last.rsplit_once('.') {
        Some((stem, ext)) => !stem.is_empty() && (1..=5).contains(&ext.len()) && ext.chars().all(|c| c.is_ascii_alphanumeric()),
        None => false,
    }
}

/// G-05: quoted string literals in script text, read as data. Computed or template strings are not followed.
fn js_urls(text: &str) -> Vec<String> {
    let b = text.as_bytes();
    let mut v = vec![];
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'/' if b.get(i + 1) == Some(&b'/') => i = b[i..].iter().position(|&c| c == b'\n').map(|p| p + i).unwrap_or(b.len()),
            b'/' if b.get(i + 1) == Some(&b'*') => i = find_ci(b, i + 2, b"*/").map(|p| p + 2).unwrap_or(b.len()),
            q @ (b'"' | b'\'' | b'`') => {
                let s = i + 1;
                let mut j = s;
                let mut esc = false;
                while j < b.len() && b[j] != q && !(q != b'`' && b[j] == b'\n') {
                    esc |= b[j] == b'\\';
                    j += if b[j] == b'\\' { 2 } else { 1 };
                }
                let lit = &text[s.min(b.len())..j.min(b.len())];
                if !esc && !lit.contains("${") && looks_url(lit) {
                    v.push(lit.to_string());
                }
                i = j + 1;
            }
            _ => i += 1,
        }
    }
    v
}

/// every http(s) link in a page, in order, without fragments, deduplicated. `<base href>` changes the base
/// from where it appears. Forms, meta refresh and `javascript:` are never followed.
pub fn links(html: &str, base: &Url, js: bool) -> Vec<Url> {
    let b = html.as_bytes();
    let mut cur = base.clone();
    let mut base_set = false;
    let mut sink = Sink { out: vec![], seen: HashSet::new() };
    let mut i = 0;
    while let Some(off) = b[i.min(b.len())..].iter().position(|&c| c == b'<') {
        i += off;
        if b[i..].starts_with(b"<!--") {
            i = find_ci(b, i + 4, b"-->").map(|e| e + 3).unwrap_or(b.len());
            continue;
        }
        let mut j = i + 1;
        let n0 = j;
        while j < b.len() && b[j].is_ascii_alphanumeric() {
            j += 1;
        }
        if j == n0 {
            i += 1;
            continue;
        }
        let tag = html[n0..j].to_ascii_lowercase();
        let mut attrs: Vec<(String, String)> = vec![];
        loop {
            while j < b.len() && (b[j].is_ascii_whitespace() || b[j] == b'/') {
                j += 1;
            }
            if j >= b.len() || b[j] == b'>' {
                break;
            }
            let k0 = j;
            while j < b.len() && !b[j].is_ascii_whitespace() && !matches!(b[j], b'=' | b'>' | b'/') {
                j += 1;
            }
            if j == k0 {
                j += 1;
                continue;
            }
            let key = html[k0..j].to_ascii_lowercase();
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            let mut val = "";
            if j < b.len() && b[j] == b'=' {
                j += 1;
                while j < b.len() && b[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < b.len() && (b[j] == b'"' || b[j] == b'\'') {
                    let q = b[j];
                    let v0 = j + 1;
                    let e = b[v0.min(b.len())..].iter().position(|&c| c == q).map(|p| p + v0).unwrap_or(b.len());
                    val = &html[v0.min(b.len())..e];
                    j = (e + 1).min(b.len());
                } else {
                    let v0 = j;
                    while j < b.len() && !b[j].is_ascii_whitespace() && b[j] != b'>' {
                        j += 1;
                    }
                    val = &html[v0..j];
                }
            }
            attrs.push((key, entities(val)));
        }
        i = (j + 1).min(b.len());
        match tag.as_str() {
            "base" => {
                if let (false, Some((_, h))) = (base_set, attrs.iter().find(|(k, _)| k == "href")) {
                    if let Ok(u) = base.join(h.trim()) {
                        cur = u;
                        base_set = true;
                    }
                }
            }
            "form" | "meta" => {}
            _ => {
                for (k, v) in &attrs {
                    match k.as_str() {
                        "href" | "src" | "poster" | "data" | "background" => sink.push(&cur, v),
                        "srcset" => srcset(v).into_iter().for_each(|u| sink.push(&cur, u)),
                        "style" => css_urls(v).iter().for_each(|u| sink.push(&cur, u)),
                        _ => {}
                    }
                }
            }
        }
        if tag == "script" || tag == "style" {
            let end = find_ci(b, i, format!("</{tag}").as_bytes()).unwrap_or(b.len());
            let text = &html[i.min(end)..end];
            if tag == "style" {
                css_urls(text).iter().for_each(|u| sink.push(&cur, u));
            } else if js {
                js_urls(text).iter().for_each(|u| sink.push(&cur, u));
            }
            i = end;
        }
    }
    sink.out
}

// ---------------------------------------------------------------- mirror folders (G-03)

/// "keep folder structure": host folder, then each directory segment decoded once and sanitized as one
/// component. `.`/`..` become `_dot`/`_dotdot` (Windows strips trailing dots, so the card's `_..` would
/// become `_`); decoded separators, `:` and NUL never split or escape.
pub fn mirror_dirs(u: &Url) -> Vec<String> {
    let host = match u.port() {
        Some(p) => format!("{}_{p}", u.host_str().unwrap_or("host")),
        None => u.host_str().unwrap_or("host").to_string(),
    };
    let mut v = vec![name::sanitize(&host.replace(':', "_"))];
    let segs: Vec<&str> = u.path().split('/').collect();
    for s in &segs[..segs.len().saturating_sub(1)] {
        if s.is_empty() {
            continue;
        }
        let d = name::pct_decode(s);
        v.push(match d.as_str() {
            "." => "_dot".into(),
            ".." => "_dotdot".into(),
            _ => name::sanitize(&d.replace(['/', '\\', ':', '\0'], "_")),
        });
    }
    v
}

/// create/open the mirror folders of `u` under `top` by handle; every level is held (no rename/swap) and a
/// junction anywhere is refused
pub fn mirror(top: &Dir, u: &Url) -> Result<Vec<Dir>, FsError> {
    let mut chain: Vec<Dir> = vec![];
    for seg in mirror_dirs(u) {
        let next = chain.last().unwrap_or(top).subdir(&seg)?;
        chain.push(next);
    }
    Ok(chain)
}

/// the chosen files as jobs: one folder, or mirror folders (`keep`). The returned folders must stay alive
/// until `Engine::add_many` has opened them. `typed_host`: the start host the user typed (W-02 intent for
/// that host only).
pub fn jobs(files: &[Found], dir: &Path, keep: bool, typed_host: Option<&str>) -> Result<(Vec<NewJob>, Vec<Dir>), Fail> {
    let top = Dir::open(dir).map_err(|e| Fail::Reparse(e.to_string()))?;
    let mut held: HashMap<PathBuf, Dir> = HashMap::new();
    let mut out = Vec::with_capacity(files.len());
    for f in files {
        let u = Url::parse(&f.url).map_err(|e| Fail::Forbidden(format!("bad URL: {e}")))?;
        let mut path = top.path.clone();
        if keep {
            for seg in mirror_dirs(&u) {
                let next = path.join(&seg);
                if !held.contains_key(&next) {
                    let d = held.get(&path).unwrap_or(&top).subdir(&seg).map_err(|e| Fail::Reparse(e.to_string()))?;
                    held.insert(next.clone(), d);
                }
                path = next;
            }
        }
        out.push(NewJob {
            url: f.url.clone(),
            dir: path,
            name: Some(f.name.clone()),
            referrer: Some(f.referrer.clone()),
            typed: typed_host.is_some_and(|h| u.host_str().is_some_and(|x| x.eq_ignore_ascii_case(h))),
            ..Default::default()
        });
    }
    let mut v: Vec<Dir> = held.into_values().collect();
    v.push(top);
    Ok((out, v))
}

// ---------------------------------------------------------------- the crawl

struct Item {
    url: Url,
    depth: u32,
    /// pages since leaving the start site
    other: u32,
}

enum Fetched {
    Html(Url, String),
    File(Url, Option<String>, Option<u64>),
    Skip(Skip),
}

struct Run<'a> {
    c: &'a Config,
    start: Url,
    start_dir: String,
    client: reqwest::Client,
    policy: Policy,
    creds: Creds,
    stop: &'a AtomicBool,
    deadline: Instant,
    last: HashMap<String, Instant>,
    robots: HashMap<net::Origin, Robots>,
    avoid: Vec<String>,
    r: Report,
}

/// waits for `fut` unless the user cancels or the time budget ends (checked every 50 ms)
async fn guarded<T>(stop: &AtomicBool, deadline: Instant, fut: impl std::future::Future<Output = T>) -> Result<T, &'static str> {
    let watch = async {
        loop {
            if stop.load(Ordering::SeqCst) {
                return "cancel";
            }
            if Instant::now() >= deadline {
                return "time";
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    };
    tokio::select! {
        v = fut => Ok(v),
        why = watch => Err(why),
    }
}

impl Run<'_> {
    fn budget(&self) -> Option<&'static str> {
        let l = &self.c.limits;
        if self.stop.load(Ordering::SeqCst) {
            Some("cancel")
        } else if Instant::now() >= self.deadline {
            Some("time")
        } else if self.r.requests >= l.requests {
            Some("requests")
        } else if self.r.bytes >= l.bytes {
            Some("bytes")
        } else {
            None
        }
    }

    fn skip(&mut self, u: &Url, reason: &str) {
        if self.r.skipped.len() < MAX_SKIPPED {
            self.r.skipped.push(Skip { url: u.to_string(), reason: reason.into() });
        }
    }

    /// one GET (no redirect following), spaced per host; Err = the crawl must stop
    async fn get(&mut self, u: &Url) -> Result<Result<reqwest::Response, String>, &'static str> {
        if let Some(why) = self.budget() {
            return Err(why);
        }
        let host = u.host_str().unwrap_or("").to_ascii_lowercase();
        let robots_delay = self.robots.get(&net::origin(u)).and_then(|r| r.delay_ms).unwrap_or(0);
        let gap = Duration::from_millis(self.c.delay_ms.max(robots_delay));
        if let Some(t) = self.last.get(&host) {
            let wait = (*t + gap).saturating_duration_since(Instant::now());
            guarded(self.stop, self.deadline, tokio::time::sleep(wait)).await?;
        }
        self.last.insert(host.clone(), Instant::now());
        self.r.requests += 1;
        let mut req = self.client.get(u.clone());
        for (k, v) in self.creds.headers_for(u) {
            req = req.header(k, v);
        }
        let resp = guarded(self.stop, self.deadline, tokio::time::timeout(Duration::from_secs(30), req.send())).await?;
        // the request may have waited on the server: the time budget ends now
        self.last.insert(host, Instant::now());
        Ok(match resp {
            Ok(Ok(r)) => Ok(r),
            Ok(Err(e)) => Err(net::forbidden(&e).map(|_| "private".to_string()).unwrap_or_else(|| "error".into())),
            Err(_) => Err("error".into()),
        })
    }

    /// body up to the page cap and the byte budget
    /// body up to `cap`; the flag is false when the body broke off (error, timeout, byte budget) before its
    /// end or the cap. ponytail: one network chunk is counted whole, so the byte budget can be passed by at
    /// most one chunk before the crawl stops.
    async fn body(&mut self, mut resp: reqwest::Response, cap: u64) -> Result<(String, bool), &'static str> {
        let mut v: Vec<u8> = vec![];
        let whole = loop {
            let chunk = guarded(self.stop, self.deadline, tokio::time::timeout(Duration::from_secs(30), resp.chunk())).await?;
            let c = match chunk {
                Ok(Ok(Some(c))) => c,
                Ok(Ok(None)) => break true,
                _ => break false,
            };
            self.r.bytes += c.len() as u64;
            v.extend_from_slice(&c[..c.len().min((cap as usize).saturating_sub(v.len()))]);
            if v.len() as u64 >= cap {
                break true;
            }
            if self.r.bytes >= self.c.limits.bytes {
                break false;
            }
        };
        Ok((String::from_utf8_lossy(&v).into_owned(), whole))
    }

    async fn robots_for(&mut self, u: &Url) -> Result<(), &'static str> {
        let o = net::origin(u);
        if !self.c.robots || self.robots.contains_key(&o) {
            return Ok(());
        }
        let mut at = u.join("/robots.txt").map_err(|_| "error")?;
        let mut prev: Vec<Url> = vec![];
        if avoided(&at, &self.avoid) {
            // the user listed robots.txt itself as a page to avoid: it is never requested (G-02), so no rules
            self.robots.insert(o, Robots::default());
            return Ok(());
        }
        let rb = loop {
            let resp = match self.get(&at).await? {
                Ok(r) => r,
                Err(_) => break Robots::unreachable(),
            };
            let st = resp.status().as_u16();
            if (300..400).contains(&st) && prev.len() < 5 {
                let next = resp.headers().get("location").and_then(|l| l.to_str().ok()).and_then(|l| at.join(l).ok());
                // a robots hop gets the same checks as a page hop (W-02, avoid list, scope); refused => no robots.txt
                prev.push(at.clone()); // history includes the current URL: no https -> http on the first hop
                match next.filter(|n| net::redirect_ok(&prev, n, &self.policy).is_ok() && !avoided(n, &self.avoid) && self.scope(n).is_some()) {
                    Some(n) => {
                        at = n;
                        continue;
                    }
                    None => break Robots::default(),
                }
            }
            break match st {
                // a robots.txt that broke off is not a permission (council G9 final #2)
                200..=299 => match self.body(resp, 512 << 10).await? {
                    (text, true) => Robots::parse(&text),
                    (_, false) => Robots::unreachable(),
                },
                429 => Robots::unreachable(),
                400..=499 => Robots::default(),
                _ => Robots::unreachable(),
            };
        };
        self.robots.insert(o, rb);
        Ok(())
    }

    /// path rules: include/exclude filters (pages and files), and for pages on the start origin "do not
    /// explore parent folders" and the path-loop guard
    fn path_rule(&self, u: &Url, home: bool, page: bool) -> Option<&'static str> {
        let p = name::pct_decode(u.path()).to_lowercase();
        let (inc, exc) = (list(&self.c.include), list(&self.c.exclude));
        if exc.iter().any(|x| glob(x, &p)) || (!inc.is_empty() && !inc.iter().any(|x| glob(x, &p))) {
            return Some("filter");
        }
        if page && home && self.c.no_parent && net::origin(u) == net::origin(&self.start) && !u.path().starts_with(&self.start_dir) {
            return Some("parent");
        }
        if page && looping(u) {
            return Some("loop");
        }
        None
    }

    /// a page: robots, then GET with redirects followed by hand (each hop: W-02, avoid, scope, depth on
    /// other sites, path rules, robots). `other` = pages since leaving the start site.
    async fn fetch(&mut self, start: &Url, other: u32) -> Result<Fetched, &'static str> {
        let mut u = start.clone();
        let mut prev: Vec<Url> = vec![];
        loop {
            self.robots_for(&u).await?;
            if self.c.robots && !self.robots[&net::origin(&u)].allowed(&u) {
                return Ok(Fetched::Skip(Skip { url: u.to_string(), reason: "robots".into() }));
            }
            let resp = match self.get(&u).await? {
                Ok(r) => r,
                Err(why) => return Ok(Fetched::Skip(Skip { url: u.to_string(), reason: why })),
            };
            let st = resp.status().as_u16();
            if (300..400).contains(&st) {
                let next = resp.headers().get("location").and_then(|l| l.to_str().ok()).and_then(|l| u.join(l).ok());
                let Some(mut next) = next else { return Ok(Fetched::Skip(Skip { url: u.to_string(), reason: format!("http-{st}") })) };
                next.set_fragment(None);
                // the current URL is part of the history: an https page never redirects to http (council G9 final #1)
                prev.push(u.clone());
                // a redirect lands on a page fetch: every rule a link to that page would meet applies (#4, #5)
                let why = if net::redirect_ok(&prev, &next, &self.policy).is_err() {
                    Some("private")
                } else if avoided(&next, &self.avoid) {
                    Some("avoid")
                } else {
                    match self.scope(&next) {
                        None => Some("other-domain"),
                        Some(false) if other.max(1) > self.c.other_depth => Some("depth"),
                        Some(home) => self.path_rule(&next, home, true),
                    }
                };
                if let Some(why) = why {
                    return Ok(Fetched::Skip(Skip { url: next.to_string(), reason: why.into() }));
                }
                u = next;
                continue;
            }
            if !(200..300).contains(&st) {
                return Ok(Fetched::Skip(Skip { url: u.to_string(), reason: format!("http-{st}") }));
            }
            let ct = crate::http::content_type(resp.headers());
            if ct.starts_with("text/html") || ct.starts_with("application/xhtml+xml") {
                let (body, _) = self.body(resp, self.c.limits.page_bytes).await?; // a cut page still has links
                return Ok(Fetched::Html(u, body));
            }
            // not a page: list it, do not read it
            let cd = crate::http::hv(resp.headers(), "content-disposition");
            return Ok(Fetched::File(u, cd, resp.content_length()));
        }
    }

    /// Some(true) = the start site, Some(false) = an allowed other domain, None = out of scope
    fn scope(&self, u: &Url) -> Option<bool> {
        let start = Url::parse(&self.c.start).ok()?;
        let sh = start.host_str()?.to_ascii_lowercase();
        let h = u.host_str()?.to_ascii_lowercase();
        let main = sh.strip_prefix("www.").unwrap_or(&sh);
        if h == sh || (self.c.subdomains && (h == main || h.ends_with(&format!(".{main}")))) {
            return Some(true);
        }
        list(&self.c.domains).iter().any(|d| glob(d, &h)).then_some(false)
    }
}

/// repeated decoded path segment (>= 3 times): /a/a/a/ loops
fn looping(u: &Url) -> bool {
    let mut n: HashMap<String, u32> = HashMap::new();
    u.path().split('/').filter(|s| !s.is_empty()).any(|s| {
        let c = n.entry(name::pct_decode(s).to_lowercase()).or_insert(0);
        *c += 1;
        *c >= 3
    })
}

fn page_like(u: &Url) -> bool {
    let last = u.path().rsplit('/').next().unwrap_or("");
    match last.rsplit_once('.') {
        None => true,
        Some((_, ext)) => PAGE_EXT.contains(&ext.to_ascii_lowercase().as_str()),
    }
}

/// explore from `c.start`. Returns what was found so far when a budget ends or the user cancels.
pub async fn crawl(c: &Config, stop: &AtomicBool, progress: &(dyn Fn(&Progress) + Send + Sync)) -> Result<Report, String> {
    let mut c = c.clone();
    c.limits.clamp();
    c.delay_ms = c.delay_ms.min(60_000);
    c.depth = c.depth.min(20);
    c.other_depth = c.other_depth.min(5);
    let mut start = Url::parse(c.start.trim()).map_err(|_| "bad-url".to_string())?;
    start.set_fragment(None);
    c.start = start.to_string();
    let policy = Policy { allow_private: if c.typed { vec![start.host_str().unwrap_or("").to_string()] } else { vec![] } };
    policy.check_url(&start).map_err(|e| format!("forbidden: {}", e.0))?;
    let client = net::client_manual(
        Arc::new(policy.clone()),
        Duration::from_secs(30),
        Some(&format!("GalaxyDownloadManager/{} (site grabber)", env!("CARGO_PKG_VERSION"))),
    )?;
    let creds = Creds { origin: Some(net::origin(&start)), headers: c.creds.clone(), digest: None };
    let avoid = list(&c.avoid);
    let start_dir = {
        let p = start.path();
        p[..p.rfind('/').map(|i| i + 1).unwrap_or(p.len())].to_string()
    };
    let mut run = Run {
        c: &c,
        start: start.clone(),
        start_dir,
        client,
        policy,
        creds,
        stop,
        deadline: Instant::now() + Duration::from_secs(c.limits.secs),
        last: HashMap::new(),
        robots: HashMap::new(),
        avoid,
        r: Report::default(),
    };
    if avoided(&start, &run.avoid) {
        run.skip(&start, "avoid");
        run.r.stop = "done".into();
        return Ok(run.r);
    }
    let mut seen: HashSet<String> = HashSet::from([start.to_string()]);
    let mut variants: HashMap<(net::Origin, String), u32> = HashMap::new();
    let mut names: HashSet<String> = HashSet::new();
    let mut queue = VecDeque::from([Item { url: start.clone(), depth: 0, other: 0 }]);
    let report = |run: &Run, url: &Url| {
        progress(&Progress { pages: run.r.pages, requests: run.r.requests, files: run.r.files.len(), bytes: run.r.bytes, url: url.to_string() })
    };
    let add_file = |run: &mut Run, names: &mut HashSet<String>, u: &Url, from: &Url, name: String, size: Option<u64>| {
        let dup = !names.insert(name.to_lowercase());
        run.r.files.push(Found { url: u.to_string(), referrer: from.to_string(), name, size, dup });
    };
    let mut stop_why = "done";
    while let Some(it) = queue.pop_front() {
        if run.r.pages >= c.limits.pages {
            stop_why = "pages";
            break;
        }
        let got = match run.fetch(&it.url, it.other).await {
            Ok(g) => g,
            Err(why) => {
                stop_why = why;
                break;
            }
        };
        report(&run, &it.url);
        let (page, html) = match got {
            Fetched::Skip(s) => {
                if run.r.skipped.len() < MAX_SKIPPED {
                    run.r.skipped.push(s);
                }
                continue;
            }
            Fetched::File(u, cd, size) => {
                let n = name::file_name(cd.as_deref(), &u);
                if c.wants(&n) && seen.insert(format!("file:{u}")) {
                    add_file(&mut run, &mut names, &u, &it.url, n, size);
                }
                continue;
            }
            Fetched::Html(u, h) => (u, h),
        };
        run.r.pages += 1;
        // where the page really is after redirects: the start site resets the other-site count
        let here = if run.scope(&page) == Some(true) { 0 } else { it.other.max(1) };
        for l in links(&html, &page, c.js) {
            if !seen.insert(l.to_string()) {
                continue;
            }
            if seen.len() as u32 > c.limits.urls {
                stop_why = "urls";
                break;
            }
            if run.policy.check_url(&l).is_err() {
                run.skip(&l, "private");
                continue;
            }
            if avoided(&l, &run.avoid) {
                run.skip(&l, "avoid");
                continue;
            }
            let Some(home) = run.scope(&l) else {
                run.skip(&l, "other-domain");
                continue;
            };
            if let Some(why) = run.path_rule(&l, home, false) {
                run.skip(&l, why);
                continue;
            }
            if !page_like(&l) {
                let n = name::file_name(None, &l);
                if c.wants(&n) {
                    add_file(&mut run, &mut names, &l, &page, n, None);
                }
                continue;
            }
            let (depth, other) = if home { (it.depth + 1, 0) } else { (it.depth + 1, here + 1) };
            if depth > c.depth || other > c.other_depth {
                continue;
            }
            if let Some(why) = run.path_rule(&l, home, true) {
                run.skip(&l, why);
                continue;
            }
            if l.query().is_some() {
                let k = (net::origin(&l), l.path().to_string());
                let n = variants.entry(k).or_insert(0);
                *n += 1;
                if *n > c.limits.query_variants {
                    run.skip(&l, "query-variants");
                    continue;
                }
            }
            queue.push_back(Item { url: l, depth, other });
        }
        if stop_why == "urls" {
            break;
        }
    }
    if stop_why == "done" && run.r.bytes >= c.limits.bytes {
        stop_why = "bytes"; // the budget ran out on the last page (council G9 final r2)
    }
    // ponytail: a redirect chain other site -> start site -> other site keeps the first count, so it may stop
    // one step early (never later); track the count per hop if users hit it
    run.r.stop = stop_why.into();
    Ok(run.r)
}
