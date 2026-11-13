// SPDX-License-Identifier: RPL-1.5
//! Browser <-> gdm-bridge <-> gdm.exe protocol (B-01, B-03) and the per-user pipe (B-02).
//! The protocol has no dir/path/hook/auto-start field at all: the browser can only propose a URL with its
//! referrer, a suggested name, size, mime and the cookie for that URL. Unknown fields reject the message.
use serde::{Deserialize, Serialize};

pub const HOST: &str = "com.goldeniq.gdm";
/// B-03: rejected before allocating
pub const MAX_MSG: u32 = 256 * 1024;
pub const MAX_URL: usize = 8 * 1024;
pub const MAX_NAME: usize = 1024;
pub const MAX_COOKIE: usize = 32 * 1024;
pub const MAX_LINKS: usize = 500;
pub const MAX_UA: usize = 512;

/// Extension ids (B-01). The stores assign their own: the owner fills CWS_ID / EDGE_ID when the store item
/// is reserved (uploaded, not yet published), BEFORE building the app that ships with it. Both the bridge
/// allow-list and the app's "Install extension" links read them; an empty id is never allowed.
pub const DEV_ID: &str = "khgpllebdmknoollimkfkggepiembabi"; // unpacked build (key in extension/build.mjs)
pub const CWS_ID: &str = ""; // Chrome Web Store (also Brave, Vivaldi, Opera with its Chrome add-on)
pub const EDGE_ID: &str = ""; // Edge Add-ons
pub const FIREFOX_IDS: &[&str] = &["gdm@goldeniq.dev"]; // fixed in the manifest, AMO keeps it

pub mod browsers;

pub fn chrome_ids() -> impl Iterator<Item = &'static str> {
    [DEV_ID, CWS_ID, EDGE_ID].into_iter().filter(|i| !i.is_empty())
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "t", rename_all = "lowercase", deny_unknown_fields)]
pub enum Msg {
    /// popup/onboarding: "connected" (J2.2); the reply carries the catch settings the extension applies
    Hello { browser: String, ver: String },
    /// one caught download (B-05: the reply is the ack; no ack within 3 s => the browser keeps it)
    Handoff {
        hid: String,
        url: String,
        referrer: Option<String>,
        name: Option<String>,
        size: Option<u64>,
        mime: Option<String>,
        cookie: Option<String>,
        /// the browser's User-Agent (IDM parity: a cookie such as Cloudflare's cf_clearance only works with it)
        ua: Option<String>,
    },
    /// the browser had already finished this download when we tried to cancel it: drop GDM's copy
    Dup { hid: String },
    /// context menu "download all links" (P1 #15): shown for selection, nothing starts by itself
    /// `cookie` (the browser's, like a handoff) belongs to the origin of `urls[0]`: GDM gives it only to the
    /// links of that same origin, never to another site's links
    Links { urls: Vec<String>, referrer: Option<String>, cookie: Option<String>, ua: Option<String> },
    /// popup: recent downloads
    Status {},
    /// popup "download a video from this page" (D5, opt-in per site): GDM asks in its own window (D10)
    Video { page: String },
    /// gdm.exe command line (I-02). Never accepted from a browser: `parse` (the bridge's decoder) refuses it,
    /// and the pipe server accepts it only from a client process whose image is gdm.exe itself.
    Cli {
        /// d (IDM /d, `add`), s (IDM /s, `start-queue`), list
        cmd: String,
        url: Option<String>,
        /// /p: an existing folder (checked by handle in gdm.exe; dangerous folders refused)
        dir: Option<String>,
        /// /f
        name: Option<String>,
        /// /a: add to the main queue, do not start
        add_only: bool,
        /// /n: no questions (no Download Info dialog)
        silent: bool,
        /// /q: exit GDM after this download, only when this call started GDM
        quit_after: bool,
    },
}

/// what reaches gdm.exe: the message plus the caller the bridge verified from argv (B-01)
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub origin: String,
    pub msg: Msg,
}

fn http_url(s: &str) -> bool {
    s.len() <= MAX_URL && (s.starts_with("https://") || s.starts_with("http://")) && !s.chars().any(|c| c.is_control())
}

impl Msg {
    pub fn validate(&self) -> Result<(), &'static str> {
        let short = |s: &str, n: usize| s.len() <= n && !s.chars().any(|c| c.is_control());
        let ok = match self {
            Msg::Hello { browser, ver } => short(browser, 32) && short(ver, 32),
            Msg::Handoff { hid, url, referrer, name, mime, cookie, ua, .. } => {
                short(hid, 64)
                    && http_url(url)
                    && referrer.as_deref().is_none_or(http_url)
                    && name.as_deref().is_none_or(|s| short(s, MAX_NAME))
                    && mime.as_deref().is_none_or(|s| short(s, 256))
                    && cookie.as_deref().is_none_or(|s| short(s, MAX_COOKIE))
                    && ua.as_deref().is_none_or(|s| short(s, MAX_UA))
            }
            Msg::Dup { hid } => short(hid, 64),
            Msg::Links { urls, referrer, cookie, ua } => {
                !urls.is_empty()
                    && urls.len() <= MAX_LINKS
                    && urls.iter().all(|u| http_url(u))
                    && referrer.as_deref().is_none_or(http_url)
                    && cookie.as_deref().is_none_or(|s| short(s, MAX_COOKIE))
                    && ua.as_deref().is_none_or(|s| short(s, MAX_UA))
            }
            Msg::Status {} => true,
            Msg::Video { page } => http_url(page),
            Msg::Cli { cmd, url, dir, name, .. } => {
                matches!(cmd.as_str(), "d" | "s" | "list")
                    && (cmd != "d" || url.as_deref().is_some_and(http_url))
                    && dir.as_deref().is_none_or(|d| short(d, 1024))
                    && name.as_deref().is_none_or(|n| short(n, MAX_NAME))
            }
        };
        ok.then_some(()).ok_or("rejected")
    }
}

/// strict parse: typed, unknown fields refused, recursion limited by serde_json (128), limits checked
pub fn parse(b: &[u8]) -> Result<Msg, String> {
    if b.len() > MAX_MSG as usize {
        return Err("too large".into());
    }
    let m: Msg = serde_json::from_slice(b).map_err(|e| e.to_string())?;
    if matches!(m, Msg::Cli { .. }) {
        return Err("rejected".into()); // the browser can never speak for the command line (I-02)
    }
    m.validate()?;
    Ok(m)
}

/// Chrome/Edge pass the caller origin as argv[1]; Firefox passes the manifest path and the extension id
pub fn caller(args: &[String]) -> Option<String> {
    let a1 = args.get(1)?;
    if let Some(id) = a1.strip_prefix("chrome-extension://").and_then(|s| s.strip_suffix('/')) {
        return chrome_ids().any(|i| i == id).then(|| a1.clone());
    }
    if a1.to_ascii_lowercase().ends_with(".json") {
        let id = args.get(2)?;
        return FIREFOX_IDS.contains(&id.as_str()).then(|| id.clone());
    }
    None
}

// ---- the verified local channel to gdm.exe: Windows named pipe (SID in the name, server image checked) or a
// Unix socket in a private folder (peer uid + executable checked); same contract, B-02 ----
#[cfg(windows)]
#[path = "ipc_windows.rs"]
mod ipc;
#[cfg(unix)]
#[path = "ipc_unix.rs"]
mod ipc;
pub use ipc::*;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn b01_unknown_or_path_fields_rejected() {
        assert!(parse(br#"{"t":"handoff","hid":"1","url":"https://e.com/a.exe","dir":"C:/Startup"}"#).is_err());
        assert!(parse(br#"{"t":"handoff","hid":"1","url":"https://e.com/a.exe","path":"x"}"#).is_err());
        assert!(parse(br#"{"t":"handoff","hid":"1","url":"https://e.com/a.exe","auto_start":true}"#).is_err());
        assert!(parse(br#"{"t":"handoff","hid":"1","url":"file:///C:/x"}"#).is_err());
        assert!(parse(br#"{"t":"handoff","hid":"1","url":"https://e.com/a\r\nX: y"}"#).is_err());
        assert!(parse(br#"{"t":"hook","cmd":"calc"}"#).is_err());
        let cli = br#"{"t":"cli","cmd":"d","url":"https://e.com/a.exe","dir":"C:/Users/x/AppData/Roaming/Microsoft/Windows/Start Menu/Programs/Startup","name":null,"add_only":false,"silent":true,"quit_after":false}"#;
        assert!(serde_json::from_slice::<Msg>(cli).is_ok_and(|m| m.validate().is_ok()), "a well-formed CLI message");
        assert!(parse(cli).is_err(), "I-02: the browser bridge never forwards a CLI message");
        assert!(parse(br#"{"t":"video","page":"https://e.com/watch"}"#).is_ok(), "D5 page video message");
        assert!(parse(br#"{"t":"video","page":"file:///C:/x"}"#).is_err() && parse(br#"{"t":"video","page":"https://e.com/","auto":true}"#).is_err());
        assert!(parse(br#"{"t":"handoff","hid":"1","url":"https://e.com/a.zip","name":"a.zip","size":5}"#).is_ok());
        assert!(parse(br#"{"t":"handoff","hid":"1","url":"https://e.com/a.zip","ua":"Mozilla/5.0"}"#).is_ok(), "browser UA");
        assert!(parse(br#"{"t":"handoff","hid":"1","url":"https://e.com/a.zip","ua":"x\r\nCookie: y"}"#).is_err(), "UA header injection");
        assert!(parse(br#"{"t":"links","urls":["https://e.com/a"],"cookie":"s=1","ua":"Mozilla/5.0"}"#).is_ok(), "link menu cookie");
        assert!(parse(br#"{"t":"links","urls":["https://e.com/a"],"referrer":null}"#).is_ok(), "old extensions (no cookie/ua) still work");
    }
    #[test]
    fn b03_limits_and_depth() {
        let long = format!(r#"{{"t":"handoff","hid":"1","url":"https://e.com/{}"}}"#, "a".repeat(MAX_URL));
        assert!(parse(long.as_bytes()).is_err());
        let deep = format!("{}{}", "[".repeat(100_000), "]".repeat(100_000));
        assert!(parse(deep.as_bytes()).is_err(), "deep JSON refused without overflow");
        assert!(parse(&vec![b' '; MAX_MSG as usize + 1]).is_err());
        let many = format!(r#"{{"t":"links","urls":[{}]}}"#, vec![r#""https://e.com/a""#; MAX_LINKS + 1].join(","));
        assert!(parse(many.as_bytes()).is_err());
    }
    /// B-03 randomized decoder campaign (not coverage-guided; cargo-fuzz runs in CI): random bytes and
    /// mutations of valid messages never panic, and every accepted message respects the limits.
    /// GDM_FUZZ_ITERS (default 20 000) sets the size; G6 evidence ran 5 000 000.
    #[test]
    fn b03_randomized_decoder() {
        let n: u64 = std::env::var("GDM_FUZZ_ITERS").ok().and_then(|v| v.parse().ok()).unwrap_or(20_000);
        let seeds: [&[u8]; 4] = [
            br#"{"t":"handoff","hid":"1","url":"https://e.com/a.zip","referrer":null,"name":"a.zip","size":5,"mime":"x","cookie":"a=b"}"#,
            br#"{"t":"links","urls":["https://e.com/a","http://e.com/b"],"referrer":"https://e.com/"}"#,
            br#"{"t":"hello","browser":"chrome","ver":"0.9.0"}"#,
            br#"{"t":"dup","hid":"x"}"#,
        ];
        let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut rnd = || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        let mut accepted = 0u64;
        for i in 0..n {
            let mut b: Vec<u8> = if i % 4 == 0 { (0..(rnd() % 512)).map(|_| rnd() as u8).collect() } else { seeds[(rnd() % 4) as usize].to_vec() };
            for _ in 0..(rnd() % 6) {
                if b.is_empty() {
                    break;
                }
                let k = (rnd() as usize) % b.len();
                match rnd() % 4 {
                    0 => b[k] = rnd() as u8,
                    1 => drop(b.remove(k)),
                    2 => b.insert(k, [b'"', b'{', b'}', b'[', b']', b',', b':', 0x5C, 0xC3, 0x80][(rnd() % 10) as usize]),
                    _ => {
                        let r: Vec<u8> = b[k..].iter().take(64).copied().collect();
                        b.splice(k..k, r);
                    }
                }
            }
            if let Ok(m) = parse(&b) {
                accepted += 1;
                assert!(m.validate().is_ok());
                if let Msg::Handoff { url, name, cookie, .. } = &m {
                    assert!(url.len() <= MAX_URL && (url.starts_with("http://") || url.starts_with("https://")));
                    assert!(name.as_deref().is_none_or(|v| v.len() <= MAX_NAME) && cookie.as_deref().is_none_or(|v| v.len() <= MAX_COOKIE));
                }
            }
        }
        println!("b03 randomized: {n} inputs, {accepted} accepted, 0 panics");
    }

    #[test]
    fn b01_caller_allow_list() {
        let a = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(caller(&a(&["gdm-bridge", &format!("chrome-extension://{DEV_ID}/")])).is_some());
        assert!(caller(&a(&["gdm-bridge", "chrome-extension:///"])).is_none(), "an unset store id is never allowed");
        assert!(caller(&a(&["gdm-bridge", "chrome-extension://aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/"])).is_none());
        assert!(caller(&a(&["gdm-bridge", "C:\\x\\firefox.json", FIREFOX_IDS[0]])).is_some());
        assert!(caller(&a(&["gdm-bridge", "C:\\x\\firefox.json", "evil@x"])).is_none());
        assert!(caller(&a(&["gdm-bridge"])).is_none());
    }
}
