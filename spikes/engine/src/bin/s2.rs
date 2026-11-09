// SPDX-License-Identifier: RPL-1.5
//! S2 check (host side): MV3 prepared handoff. The extension sends URL + the cookies it needs once; the
//! native host validates the caller origin (argv from the browser), message size, schema and cookie scope
//! (RFC 6265 domain/path/secure), and keeps nothing on disk.
//!   s2            run the check (drives `s2 host <origin>` as a child with Chrome framing)
//!   s2 host <origin>   native-messaging host: 4-byte little-endian length + UTF-8 JSON per message
use gdm_spikes::Check;
use serde::Deserialize;
use std::io::{Read, Write};
use std::process::{Command, Stdio};

const ALLOWED: &[&str] = &["chrome-extension://gdmspikeextensionidaaaaaaaaaaaaaa/"];
const MAX_IN: usize = 64 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Handoff { url: String, referrer: Option<String>, cookies: Vec<Cookie>, expires_at: u64 }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(non_snake_case)]
struct Cookie { name: String, value: String, domain: String, hostOnly: bool, path: String, secure: bool }

fn now() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() }

/// (scheme, host lowercase, path)
fn split_url(u: &str) -> Option<(&str, String, String)> {
    let (scheme, rest) = u.split_once("://")?;
    let (auth, path) = rest.find(['/', '?', '#']).map_or((rest, "/"), |i| (&rest[..i], &rest[i..]));
    if auth.contains('@') || auth.is_empty() { return None; } // no userinfo
    let host = auth.rsplit_once(':').filter(|(_, p)| p.chars().all(|c| c.is_ascii_digit())).map_or(auth, |(h, _)| h);
    let path = path.split(['?', '#']).next().unwrap_or("/");
    Some((scheme, host.to_ascii_lowercase(), if path.is_empty() { "/".into() } else { path.into() }))
}

fn cookie_applies(c: &Cookie, scheme: &str, host: &str, path: &str) -> bool {
    let d = c.domain.trim_start_matches('.').to_ascii_lowercase();
    let domain_ok = if c.hostOnly { host == d } else { host == d || host.ends_with(&format!(".{d}")) };
    let p = &c.path;
    let path_ok = path == p || (path.starts_with(p.as_str()) && (p.ends_with('/') || path[p.len()..].starts_with('/')));
    domain_ok && path_ok && (!c.secure || scheme == "https") && !c.name.contains([';', '\r', '\n']) && !c.value.contains([';', '\r', '\n'])
}

fn validate(origin: &str, raw: &[u8]) -> Result<String, String> {
    if !ALLOWED.contains(&origin) { return Err("origin not allowed".into()); }
    let h: Handoff = serde_json::from_slice(raw).map_err(|e| format!("schema: {e}"))?;
    let (scheme, host, path) = split_url(&h.url).ok_or("bad url")?;
    if scheme != "http" && scheme != "https" { return Err("only http(s)".into()); }
    if h.expires_at < now() || h.expires_at > now() + 60 { return Err("ticket expired or too long-lived".into()); }
    if let Some(r) = &h.referrer { if split_url(r).is_none() { return Err("bad referrer".into()); } }
    if let Some(c) = h.cookies.iter().find(|c| !cookie_applies(c, scheme, &host, &path)) { return Err(format!("cookie '{}' out of scope", c.name)); }
    // the validated ticket goes to the engine in memory; the host writes nothing to disk
    Ok(format!("{{\"ok\":true,\"cookies\":{}}}", h.cookies.len()))
}

fn host(origin: &str) {
    let mut stdin = std::io::stdin().lock();
    let mut out = std::io::stdout().lock();
    loop {
        let mut len = [0u8; 4];
        if stdin.read_exact(&mut len).is_err() { return; } // browser closed the port / short header
        let n = u32::from_le_bytes(len) as usize;
        let reply = if n > MAX_IN {
            // do not read an oversized body: answer and close
            let r = b"{\"ok\":false,\"err\":\"too large\"}";
            let _ = out.write_all(&(r.len() as u32).to_le_bytes()).and_then(|_| out.write_all(r)).and_then(|_| out.flush());
            return;
        } else {
            let mut body = vec![0u8; n];
            if stdin.read_exact(&mut body).is_err() { return; } // truncated frame: exit cleanly
            match validate(origin, &body) { Ok(r) => r, Err(e) => format!("{{\"ok\":false,\"err\":{}}}", serde_json::to_string(&e).unwrap()) }
        };
        let _ = out.write_all(&(reply.len() as u32).to_le_bytes()).and_then(|_| out.write_all(reply.as_bytes())).and_then(|_| out.flush());
    }
}

/// drive the host like Chrome does: argv[1] = caller origin, framed stdin; returns the replies
fn drive(origin: &str, frames: &[Vec<u8>], raw_tail: &[u8]) -> (Vec<String>, bool) {
    let dir = std::env::temp_dir().join(format!("gdm-s2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut ch = Command::new(std::env::current_exe().unwrap()).args(["host", origin]).current_dir(&dir)
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap();
    {
        let mut si = ch.stdin.take().unwrap();
        for f in frames { let _ = si.write_all(&(f.len() as u32).to_le_bytes()); let _ = si.write_all(f); }
        let _ = si.write_all(raw_tail);
    }
    let out = ch.wait_with_output().unwrap();
    let mut replies = vec![];
    let mut b = &out.stdout[..];
    while b.len() >= 4 {
        let n = u32::from_le_bytes(b[..4].try_into().unwrap()) as usize;
        if b.len() < 4 + n { break; }
        replies.push(String::from_utf8_lossy(&b[4..4 + n]).to_string());
        b = &b[4 + n..];
    }
    let clean = b.is_empty(); // nothing outside the protocol on stdout
    let wrote_nothing = std::fs::read_dir(&dir).unwrap().next().is_none();
    (replies, clean && wrote_nothing && out.status.code().is_some())
}

fn msg(url: &str, cookies: &str, exp: u64) -> Vec<u8> {
    format!("{{\"url\":\"{url}\",\"referrer\":\"https://shop.example.com/p\",\"cookies\":[{cookies}],\"expires_at\":{exp}}}").into_bytes()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.get(1).map(String::as_str) == Some("host") { host(a.get(2).map(String::as_str).unwrap_or("")); return; }
    let mut c = Check::new("S2 MV3 handoff (host side)");
    let ok_origin = ALLOWED[0];
    let exp = now() + 30;
    let ck = |name: &str, domain: &str, host_only: bool, path: &str, secure: bool| format!("{{\"name\":\"{name}\",\"value\":\"v\",\"domain\":\"{domain}\",\"hostOnly\":{host_only},\"path\":\"{path}\",\"secure\":{secure}}}");
    let good = msg("https://dl.example.com/files/a.zip", &ck("sid", ".example.com", false, "/", true), exp);

    let (r, clean) = drive(ok_origin, &[good.clone()], b"");
    c.ok(r.len() == 1 && r[0].contains("\"ok\":true") && clean, "allowed origin + in-scope cookie -> accepted; stdout only protocol frames; no file written");
    let (r, _) = drive("chrome-extension://someotherextensionidaaaaaaaaaaaa/", &[good.clone()], b"");
    c.ok(r.len() == 1 && r[0].contains("origin not allowed"), "other extension origin -> rejected");
    let (r, clean) = drive(ok_origin, &[], &(10_000_000u32).to_le_bytes());
    c.ok(r.len() == 1 && r[0].contains("too large") && clean, "declared 10 MB message (> 64 KiB) -> rejected without reading the body");
    let (r, clean) = drive(ok_origin, &[], &[1, 0]);
    c.ok(r.is_empty() && clean, "short length header -> clean exit, no crash");
    let mut trunc = (500u32).to_le_bytes().to_vec();
    trunc.extend_from_slice(b"{\"url\":");
    let (r, clean) = drive(ok_origin, &[], &trunc);
    c.ok(r.is_empty() && clean, "length larger than payload (truncated) -> clean exit, no crash");
    let (r, _) = drive(ok_origin, &[vec![0xff, 0xfe, 0x00]], b"");
    c.ok(r.len() == 1 && r[0].contains("schema"), "invalid UTF-8 / JSON -> rejected, host keeps running");
    let cases = [
        ("cookie for a sibling host (hostOnly)", msg("https://dl.example.com/a", &ck("sid", "www.example.com", true, "/", true), exp)),
        ("cookie for another site", msg("https://dl.example.com/a", &ck("sid", "evil.test", false, "/", true), exp)),
        ("Secure cookie to http://", msg("http://dl.example.com/a", &ck("sid", "example.com", false, "/", true), exp)),
        ("cookie Path /admin to /files", msg("https://dl.example.com/files/a", &ck("sid", "example.com", false, "/admin", true), exp)),
        ("Path /fil must not match /files", msg("https://dl.example.com/files/a", &ck("sid", "example.com", false, "/fil", true), exp)),
        ("cookie value with header injection", msg("https://dl.example.com/a", "{\"name\":\"a\",\"value\":\"x\\r\\nX-Evil: 1\",\"domain\":\"example.com\",\"hostOnly\":false,\"path\":\"/\",\"secure\":true}", exp)),
        ("file:// URL", msg("file:///C:/Windows/win.ini", "", exp)),
        ("URL with userinfo", msg("https://user:pw@dl.example.com/a", "", exp)),
        ("expired ticket", msg("https://dl.example.com/a", "", now() - 1)),
        ("ticket living > 60 s", msg("https://dl.example.com/a", "", now() + 3600)),
        ("unknown field (e.g. a save path)", b"{\"url\":\"https://dl.example.com/a\",\"cookies\":[],\"expires_at\":9999999999,\"path\":\"C:\\\\x\"}".to_vec()),
    ];
    for (name, m) in cases {
        let (r, _) = drive(ok_origin, &[m], b"");
        c.ok(r.len() == 1 && r[0].contains("\"ok\":false"), &format!("{name} -> rejected ({})", r.first().map(String::as_str).unwrap_or("-")));
    }
    let (r, _) = drive(ok_origin, &[msg("https://dl.example.com/files/a", &ck("sid", "dl.example.com", true, "/files", true), exp)], b"");
    c.ok(r.len() == 1 && r[0].contains("\"ok\":true"), "control: hostOnly cookie for the exact host with matching path -> accepted");
    c.note("browser side (real MV3 service worker, chrome.cookies, connectNative, durable ticket + 200 handoffs) NOT TESTED here: S1a in G2 owns the real browser round trip");
    c.finish();
}
