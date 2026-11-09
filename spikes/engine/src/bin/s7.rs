// SPDX-License-Identifier: RPL-1.5
//! S7 check (ADR-6): yt-dlp runs extract-only (-J) through a GDM policy proxy on loopback (random port,
//! random Proxy-Authorization per run, closed when the run ends). Policy = resolve -> check every address ->
//! connect to the checked address (no second DNS lookup). The GDM engine downloads the streams (same
//! policy), ffmpeg only muxes local files with -protocol_whitelist file,pipe.
//! Tools: %LOCALAPPDATA%\gdm-spike-tools\{yt-dlp,ffmpeg,ffprobe}.exe (pinned by sha256 in results.md).
use base64::Engine;
use gdm_spikes::fixture::{pct_encode, Fixture};
use gdm_spikes::seg::{check_addrs, download, Config, NetPolicy};
use gdm_spikes::Check;
use std::collections::HashSet;
use std::hash::{BuildHasher, Hasher};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

fn v4_forbidden(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    ip.is_loopback() || ip.is_private() || ip.is_link_local() || ip.is_unspecified() || ip.is_broadcast()
        || ip.is_documentation() || ip.is_multicast() || o[0] == 0 || o[0] >= 240
        || (o[0] == 100 && (o[1] & 0xC0) == 64) || (o[0] == 192 && o[1] == 0 && o[2] == 0) || (o[0] == 198 && (o[1] & 0xFE) == 18)
}

pub fn forbidden(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v) => v4_forbidden(v),
        IpAddr::V6(v) => {
            let s = v.segments();
            if let Some(m) = v.to_ipv4_mapped() { return v4_forbidden(m); }
            if s[..6] == [0; 6] { return true; } // ::/96 incl. ::1, :: and IPv4-compatible
            if s[0] == 0x64 && s[1] == 0xff9b { return v4_forbidden(Ipv4Addr::new((s[6] >> 8) as u8, s[6] as u8, (s[7] >> 8) as u8, s[7] as u8)); } // NAT64
            if s[0] == 0x2002 { return v4_forbidden(Ipv4Addr::new((s[1] >> 8) as u8, s[1] as u8, (s[2] >> 8) as u8, s[2] as u8)); } // 6to4
            v.is_loopback() || v.is_unspecified() || v.is_multicast() || (s[0] & 0xfe00) == 0xfc00 || (s[0] & 0xffc0) == 0xfe80 || (s[0] == 0x2001 && s[1] == 0x0db8)
        }
    }
}

struct Proxy { addr: SocketAddr, auth: String, decisions: Arc<Mutex<Vec<(String, bool)>>>, task: tokio::task::JoinHandle<()> }

/// resolve -> every address must pass -> connect to the checked address. `allow` = test-only exact sockets.
async fn check_target(hostport: &str, allow: &HashSet<SocketAddr>) -> Result<SocketAddr, String> {
    if hostport.contains(['@', '/', ' ']) || hostport.is_empty() { return Err("odd authority".into()); }
    let (h, p) = hostport.rsplit_once(':').ok_or("no port")?;
    let port: u16 = p.parse().map_err(|_| "bad port")?;
    let h = h.trim_start_matches('[').trim_end_matches(']');
    let addrs: Vec<SocketAddr> = match h.parse::<IpAddr>() {
        Ok(ip) => vec![SocketAddr::new(ip, port)],
        Err(_) => tokio::time::timeout(Duration::from_secs(5), tokio::net::lookup_host((h, port))).await.map_err(|_| "dns timeout")?.map_err(|e| format!("dns: {e}"))?.collect(),
    };
    if addrs.is_empty() { return Err("no address".into()); }
    for a in &addrs { if forbidden(a.ip()) && !allow.contains(a) { return Err(format!("forbidden {a}")); } }
    Ok(addrs[0])
}

fn random_token() -> String {
    // ponytail: RandomState keys come from the OS RNG; use BCryptGenRandom / getrandom in P1
    (0..4).map(|i| { let mut h = std::collections::hash_map::RandomState::new().build_hasher(); h.write_u32(i); format!("{:016x}", h.finish()) }).collect()
}

async fn start_proxy(allow: HashSet<SocketAddr>) -> Proxy {
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = l.local_addr().unwrap();
    let token = random_token();
    let auth = format!("Basic {}", base64::engine::general_purpose::STANDARD.encode(format!("gdm:{token}")));
    let decisions: Arc<Mutex<Vec<(String, bool)>>> = Arc::default();
    let (a2, d2, allow) = (auth.clone(), decisions.clone(), Arc::new(allow));
    let task = tokio::spawn(async move {
        loop {
            let Ok((mut s, peer)) = l.accept().await else { continue };
            if !peer.ip().is_loopback() { continue; }
            let (auth, dec, allow) = (a2.clone(), d2.clone(), allow.clone());
            tokio::spawn(async move {
                let mut buf = Vec::new();
                let head = loop {
                    let mut t = [0u8; 4096];
                    let n = match s.read(&mut t).await { Ok(0) | Err(_) => return, Ok(n) => n };
                    buf.extend_from_slice(&t[..n]);
                    if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") { let h = String::from_utf8_lossy(&buf[..i]).to_string(); buf.drain(..i + 4); break h; }
                    if buf.len() > 32 * 1024 { return; }
                };
                let mut lines = head.split("\r\n");
                let first: Vec<&str> = lines.next().unwrap_or("").split(' ').collect();
                let hdrs: Vec<&str> = lines.collect();
                let ok_auth = hdrs.iter().any(|l| l.split_once(':').is_some_and(|(k, v)| k.eq_ignore_ascii_case("proxy-authorization") && v.trim() == auth));
                if !ok_auth { let _ = s.write_all(b"HTTP/1.1 407 Proxy Authentication Required\r\nProxy-Authenticate: Basic\r\nContent-Length: 0\r\n\r\n").await; return; }
                let (method, target) = (first.first().copied().unwrap_or(""), first.get(1).copied().unwrap_or(""));
                let hostport = if method == "CONNECT" { target.to_string() } else {
                    let Some(rest) = target.strip_prefix("http://") else { let _ = s.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n").await; return };
                    let auth = rest.split('/').next().unwrap_or("");
                    if auth.contains(':') && !auth.ends_with(']') { auth.to_string() } else { format!("{auth}:80") }
                };
                let dst = match check_target(&hostport, &allow).await {
                    Ok(d) => { dec.lock().unwrap().push((hostport.clone(), true)); d }
                    Err(e) => { dec.lock().unwrap().push((format!("{hostport} ({e})"), false)); let _ = s.write_all(b"HTTP/1.1 403 Forbidden by GDM policy\r\nContent-Length: 0\r\n\r\n").await; return; }
                };
                let Ok(mut up) = TcpStream::connect(dst).await else { let _ = s.write_all(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n").await; return };
                if method == "CONNECT" {
                    let _ = s.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await;
                } else {
                    // origin-form request line, proxy headers removed, one request per connection
                    let path = target.strip_prefix("http://").and_then(|r| r.find('/').map(|i| &r[i..])).unwrap_or("/");
                    let mut out = format!("{method} {path} {}\r\n", first.get(2).copied().unwrap_or("HTTP/1.1"));
                    for h in &hdrs {
                        let k = h.split(':').next().unwrap_or("").to_ascii_lowercase();
                        if !k.starts_with("proxy-") && k != "connection" { out.push_str(h); out.push_str("\r\n"); }
                    }
                    out.push_str("Connection: close\r\n\r\n");
                    if up.write_all(out.as_bytes()).await.is_err() { return; }
                }
                let _ = up.write_all(&buf).await;
                if method == "CONNECT" { let _ = tokio::io::copy_bidirectional(&mut s, &mut up).await; return; }
                // plain HTTP: exactly one request per client connection. A kept-alive second request must open a
                // new connection and be checked again (before: it went raw to the first, already-checked upstream).
                // ponytail: a request body beyond the first read is not forwarded (yt-dlp -J only GETs); parse
                // Content-Length if POST ever matters.
                // The upstream may ignore "Connection: close", so do not wait for its EOF: relay its bytes until the
                // client sends anything else (a second request) or closes, then drop both sockets.
                // Rewrite the response head with "Connection: close" so the client opens a new (checked) connection
                // for its next request instead of reusing this one.
                let mut rb = Vec::new();
                let head_end = loop {
                    let mut t = [0u8; 4096];
                    let n = match up.read(&mut t).await { Ok(0) | Err(_) => return, Ok(n) => n };
                    rb.extend_from_slice(&t[..n]);
                    if let Some(i) = rb.windows(4).position(|w| w == b"\r\n\r\n") { break i; }
                    if rb.len() > 64 * 1024 { return; }
                };
                let head = String::from_utf8_lossy(&rb[..head_end]).to_string();
                let mut out = String::new();
                for (i, l) in head.split("\r\n").enumerate() {
                    let k = l.split(':').next().unwrap_or("").trim().to_ascii_lowercase();
                    if i > 0 && (k == "connection" || k == "keep-alive" || k == "proxy-connection") { continue; }
                    out.push_str(l); out.push_str("\r\n");
                }
                out.push_str("Connection: close\r\n\r\n");
                if s.write_all(out.as_bytes()).await.is_err() || s.write_all(&rb[head_end + 4..]).await.is_err() { return; }
                let (mut cr, mut cw) = s.split();
                let mut probe = [0u8; 1];
                tokio::select! {
                    _ = tokio::io::copy(&mut up, &mut cw) => {}
                    _ = cr.read(&mut probe) => {}
                }
            });
        }
    });
    Proxy { addr, auth, decisions, task }
}

async fn ask(proxy: SocketAddr, req: &str) -> String {
    let Ok(mut s) = TcpStream::connect(proxy).await else { return "connect refused".into() };
    let _ = s.write_all(req.as_bytes()).await;
    let mut b = vec![0u8; 512];
    let n = tokio::time::timeout(Duration::from_secs(8), s.read(&mut b)).await.ok().and_then(|r| r.ok()).unwrap_or(0);
    String::from_utf8_lossy(&b[..n]).lines().next().unwrap_or("").to_string()
}

fn tools() -> PathBuf { PathBuf::from(std::env::var("LOCALAPPDATA").unwrap()).join("gdm-spike-tools") }

/// yt-dlp extract-only, fixed flags, `--` before the URL, clean env, own cwd, hard timeout
fn ytdlp(proxy: &Proxy, url: &str, cwd: &Path) -> (bool, String, String) {
    let token = String::from_utf8(base64::engine::general_purpose::STANDARD.decode(proxy.auth.trim_start_matches("Basic ")).unwrap()).unwrap();
    let mut cmd = Command::new(tools().join("yt-dlp.exe"));
    cmd.args(["-J", "--ignore-config", "--no-plugin-dirs", "--no-cache-dir", "--socket-timeout", "10", "--proxy", &format!("http://{token}@{}", proxy.addr), "--", url])
        .env_clear().env("SystemRoot", std::env::var("SystemRoot").unwrap()).env("TEMP", cwd).env("TMP", cwd)
        .current_dir(cwd).stdin(std::process::Stdio::null());
    let mut ch = cmd.stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn().unwrap();
    let t = std::time::Instant::now();
    while ch.try_wait().unwrap().is_none() {
        if t.elapsed() > Duration::from_secs(90) { let _ = ch.kill(); break; }
        std::thread::sleep(Duration::from_millis(50));
    }
    let o = ch.wait_with_output().unwrap();
    let out = String::from_utf8_lossy(&o.stdout);
    (o.status.success(), out.chars().take(1 << 20).collect(), String::from_utf8_lossy(&o.stderr).to_string())
}

fn ffmpeg(args: &[&str], cwd: &Path) -> (bool, String) {
    let o = Command::new(tools().join("ffmpeg.exe")).args(["-hide_banner", "-nostdin", "-loglevel", "error"]).args(args).current_dir(cwd).output().unwrap();
    (o.status.success(), String::from_utf8_lossy(&o.stderr).to_string())
}

fn streams(file: &Path) -> String {
    let o = Command::new(tools().join("ffprobe.exe")).args(["-v", "error", "-show_entries", "stream=codec_type", "-of", "csv=p=0", file.to_str().unwrap()]).output().unwrap();
    String::from_utf8_lossy(&o.stdout).split_whitespace().collect::<Vec<_>>().join(",")
}

/// engine side: the same policy before any media byte is fetched
fn policy(allow: &HashSet<SocketAddr>) -> NetPolicy {
    let allow = allow.clone();
    Arc::new(move |a: SocketAddr| !forbidden(a.ip()) || allow.contains(&a))
}

/// engine side: the same policy on every connection (resolver + literal + redirect checks inside seg)
async fn engine_fetch(url: &str, part: &Path, allow: &HashSet<SocketAddr>) -> Result<String, String> {
    download(url, part, Config { net_policy: Some(policy(allow)), retries: 1, ..Config::default() }).await.map(|d| d.sha256).map_err(|e| format!("{e:?}"))
}

#[tokio::main]
async fn main() {
    let mut c = Check::new("S7 yt-dlp policy proxy + media gate");
    let dir = std::env::temp_dir().join(format!("gdm-s7-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for t in ["yt-dlp.exe", "ffmpeg.exe", "ffprobe.exe"] { if !tools().join(t).exists() { c.ok(false, &format!("{t} missing in {}", tools().display())); c.finish(); } }

    // canary: a loopback listener nobody may reach; counts connections
    let canary = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let cport = canary.local_addr().unwrap().port();
    let hits = Arc::new(Mutex::new(0u32));
    let h2 = hits.clone();
    tokio::spawn(async move { loop { if canary.accept().await.is_ok() { *h2.lock().unwrap() += 1; } } });

    let fx = Fixture::start("127.0.0.1:0").await;
    let allow: HashSet<SocketAddr> = [fx.addr].into(); // test-only: the fixture stands in for a public site
    let px = start_proxy(allow.clone()).await;
    let a = &px.auth;

    // 1. auth
    c.ok(ask(px.addr, &format!("GET http://{}/echo HTTP/1.1\r\nHost: x\r\n\r\n", fx.addr)).await.contains("407"), "no Proxy-Authorization -> 407");
    c.ok(ask(px.addr, &format!("GET http://{}/echo HTTP/1.1\r\nProxy-Authorization: Basic Z2RtOnd yb25n\r\n\r\n", fx.addr)).await.contains("407"), "wrong Proxy-Authorization -> 407");
    c.ok(ask(px.addr, &format!("GET http://{}/echo HTTP/1.1\r\nHost: x\r\nProxy-Authorization: {a}\r\n\r\n", fx.addr)).await.contains("200"), "right token, allowed (test-only) target -> forwarded 200");

    // 2. SSRF matrix: every target must be refused, the canary must see nothing
    let targets = [
        format!("127.0.0.1:{cport}"), format!("[::1]:{cport}"), format!("[::ffff:127.0.0.1]:{cport}"), format!("[::ffff:7f00:1]:{cport}"),
        format!("[::127.0.0.1]:{cport}"), format!("[64:ff9b::7f00:1]:{cport}"), format!("[2002:7f00:1::]:{cport}"),
        format!("2130706433:{cport}"), format!("0x7f000001:{cport}"), format!("127.1:{cport}"), format!("0177.0.0.1:{cport}"),
        format!("localhost:{cport}"), format!("localtest.me:{cport}"), format!("0.0.0.0:{cport}"),
        "169.254.169.254:80".into(), "10.0.0.1:80".into(), "172.16.0.1:80".into(), "192.168.1.1:80".into(), "100.64.0.1:80".into(),
        "[fd00::1]:80".into(), "[fe80::1]:80".into(), "[ff02::1]:80".into(), "224.0.0.1:80".into(),
    ];
    let mut refused = 0;
    for t in &targets {
        let (g, k) = (ask(px.addr, &format!("GET http://{t}/ HTTP/1.1\r\nHost: x\r\nProxy-Authorization: {a}\r\n\r\n")).await, ask(px.addr, &format!("CONNECT {t} HTTP/1.1\r\nProxy-Authorization: {a}\r\n\r\n")).await);
        let ok = !g.contains(" 200") && !k.contains(" 200");
        if ok { refused += 1; } else { println!("     NOT refused: {t}  GET='{g}' CONNECT='{k}'"); }
    }
    c.ok(refused == targets.len(), &format!("SSRF matrix: {refused}/{} targets refused for GET and CONNECT (loopback, mapped/compat/NAT64/6to4 IPv6, decimal/hex/short/octal IPv4, localhost, public DNS name -> 127.0.0.1, metadata, RFC1918, CGNAT, ULA, link-local, multicast)", targets.len()));
    for t in [format!("evil@127.0.0.1:{cport}"), "127.0.0.1".into(), format!("x/../127.0.0.1:{cport}")] {
        c.ok(!ask(px.addr, &format!("CONNECT {t} HTTP/1.1\r\nProxy-Authorization: {a}\r\n\r\n")).await.contains(" 200"), &format!("odd CONNECT authority '{t}' -> refused"));
    }
    c.ok(forbidden("2001:db8::1".parse().unwrap()) && !forbidden("1.1.1.1".parse().unwrap()) && !forbidden("2606:4700::1111".parse().unwrap()), "policy unit: documentation range refused, public v4/v6 allowed");

    // 3. media on the fixture: video-only + audio-only + progressive, and pages for yt-dlp
    let (ok_v, e) = ffmpeg(&["-f", "lavfi", "-i", "testsrc=duration=2:size=320x240:rate=25", "-c:v", "mpeg4", "-an", "v.mp4"], &dir);
    let (ok_a, _) = ffmpeg(&["-f", "lavfi", "-i", "sine=frequency=440:duration=2", "-c:a", "aac", "-vn", "a.m4a"], &dir);
    let (ok_p, _) = ffmpeg(&["-i", "v.mp4", "-i", "a.m4a", "-c", "copy", "p.mp4"], &dir);
    c.ok(ok_v && ok_a && ok_p, &format!("test media generated locally (lavfi) {e}"));
    for n in ["v.mp4", "a.m4a", "p.mp4"] { fx.add_blob(n, std::fs::read(dir.join(n)).unwrap()); }
    fx.add_blob("page.html", format!("<!doctype html><html><head><title>GDM S7 page</title></head><body><video controls src=\"http://{}/static/p.mp4\"></video></body></html>", fx.addr).into_bytes());
    fx.add_blob("evil.html", format!("<!doctype html><html><head><title>evil</title></head><body><video src=\"http://127.0.0.1:{cport}/secret.mp4\"></video></body></html>").into_bytes());

    // 4. yt-dlp through the proxy
    fx.clear_log();
    let before = px.decisions.lock().unwrap().len();
    let (ok, json, err) = ytdlp(&px, &fx.url("/static/page.html"), &dir);
    let media = serde_json::from_str::<serde_json::Value>(&json).ok().and_then(|j| j["url"].as_str().or(j["formats"][0]["url"].as_str()).map(String::from)).unwrap_or_default();
    let via_proxy = px.decisions.lock().unwrap()[before..].iter().filter(|(_, ok)| *ok).count();
    c.ok(ok && media.ends_with("/static/p.mp4"), &format!("yt-dlp -J (generic extractor) through the proxy returned the media URL '{media}' {}", if ok { "" } else { &err[..err.len().min(300)] }));
    c.ok(via_proxy >= 1 && via_proxy >= fx.log().len(), &format!("every fixture request yt-dlp made passed the proxy ({} proxied, {} at the fixture)", via_proxy, fx.log().len()));
    fx.clear_log();
    let d0 = px.decisions.lock().unwrap().len();
    let (_, _, err2) = ytdlp(&px, &fx.url(&format!("/redirect?to={}", pct_encode(&format!("http://127.0.0.1:{cport}/x")))), &dir);
    for q in fx.log() { println!("     fixture saw: {} {}", q.method, q.target); }
    for (t, ok) in &px.decisions.lock().unwrap()[d0..] { println!("     proxy: {t} -> {}", if *ok { "allowed" } else { "REFUSED" }); }
    let refused_canary = px.decisions.lock().unwrap()[d0..].iter().any(|(t, ok)| !ok && t.starts_with(&format!("127.0.0.1:{cport}")));
    let fx_abs = fx.log().iter().any(|q| q.target.starts_with("http://"));
    c.ok(*hits.lock().unwrap() == 0 && refused_canary && !fx_abs, &format!("redirect to loopback: proxy REFUSED 127.0.0.1:{cport} on its own connection, fixture got no smuggled request, canary 0 (yt-dlp: {})", err2.lines().last().unwrap_or("")));
    let (okr, jr, er) = ytdlp(&px, &fx.url(&format!("/redirect?to={}", pct_encode(&fx.url("/static/page.html")))), &dir);
    c.ok(okr && jr.contains("/static/p.mp4"), &format!("control: the same redirect route to an allowed page works through the proxy {}", if okr { "" } else { &er[..er.len().min(200)] }));
    let (_, j3, _) = ytdlp(&px, &fx.url("/static/evil.html"), &dir);
    c.ok(*hits.lock().unwrap() == 0, "a page pointing at loopback: canary saw 0 connections");
    let evil_url = serde_json::from_str::<serde_json::Value>(&j3).ok().and_then(|j| j["url"].as_str().map(String::from));
    if let Some(u) = &evil_url {
        let r = engine_fetch(u, &dir.join("evil.part"), &allow).await;
        c.ok(r.is_err() && *hits.lock().unwrap() == 0, &format!("yt-dlp returned loopback media URL; GDM engine refused it before fetching ({r:?})"));
    } else { c.note("yt-dlp did not return the loopback media URL (it could not reach it); engine refusal tested directly below"); }
    let r = engine_fetch(&format!("http://127.0.0.1:{cport}/secret.mp4"), &dir.join("evil2.part"), &allow).await;
    c.ok(r.as_ref().is_err_and(|e| e.contains("Forbidden")) && *hits.lock().unwrap() == 0, &format!("GDM engine: loopback literal refused before connecting, canary 0 ({r:?})"));
    let r = engine_fetch(&format!("http://localtest.me:{cport}/secret.mp4"), &dir.join("evil3.part"), &allow).await;
    c.ok(r.as_ref().is_err_and(|e| e.contains("Forbidden")) && *hits.lock().unwrap() == 0, &format!("GDM engine: public DNS name -> 127.0.0.1 refused by the checking resolver, canary 0 ({r:?})"));
    let r = engine_fetch(&fx.url(&format!("/redirect?to={}", pct_encode(&format!("http://127.0.0.1:{cport}/x")))), &dir.join("evil4.part"), &allow).await;
    c.ok(r.as_ref().is_err_and(|e| e.contains("Forbidden") || e.contains("forbidden")) && *hits.lock().unwrap() == 0, &format!("GDM engine: allowed site redirecting to loopback refused at the redirect hop, canary 0 ({r:?})"));
    let r = engine_fetch(&fx.url(&format!("/redirect?to={}", pct_encode(&fx.url("/file?size=1048576&seed=3")))), &dir.join("ok4.part"), &allow).await;
    c.ok(r.is_ok(), &format!("control: the same redirect route to an allowed target downloads ({r:?})"));
    let pol = policy(&HashSet::new());
    let mixed: Vec<SocketAddr> = vec!["93.184.216.34:443".parse().unwrap(), "127.0.0.1:443".parse().unwrap()];
    c.ok(check_addrs(&mixed, &pol).is_err() && check_addrs(&mixed[..1], &pol).is_ok(), "DNS answer with public + private address refused as a whole (public alone allowed)");
    c.note("rebinding: one lookup per connection, and reqwest connects only to the addresses the checking resolver returned (no second lookup). A live rebinding DNS server was not run.");

    // 5. argv injection: a URL that looks like an option stays a URL after `--`
    let (ok, _, err) = ytdlp(&px, "--exec=cmd /c echo pwned>pwned.txt", &dir);
    c.ok(!ok && !dir.join("pwned.txt").exists(), &format!("'--exec=…' after `--` is treated as a URL: no command ran ({})", err.lines().last().unwrap_or("")));

    // 6. engine downloads the streams, ffmpeg muxes local files only
    let v = engine_fetch(&fx.url("/static/v.mp4"), &dir.join("dl-v.mp4"), &allow).await;
    let au = engine_fetch(&fx.url("/static/a.m4a"), &dir.join("dl-a.m4a"), &allow).await;
    let pr = engine_fetch(&media, &dir.join("dl-p.mp4"), &allow).await;
    let (okm, em) = ffmpeg(&["-protocol_whitelist", "file,pipe", "-i", "dl-v.mp4", "-i", "dl-a.m4a", "-map", "0:v", "-map", "1:a", "-c", "copy", "muxed.mp4"], &dir);
    c.ok(v.is_ok() && au.is_ok() && okm && streams(&dir.join("muxed.mp4")) == "video,audio", &format!("separate video + audio: engine download -> ffmpeg mux (file,pipe) -> ffprobe '{}' {em}", streams(&dir.join("muxed.mp4"))));
    c.ok(pr.is_ok() && streams(&dir.join("dl-p.mp4")) == "video,audio", "progressive file from the yt-dlp JSON: engine download -> ffprobe video,audio");
    let (okh, eh) = ffmpeg(&["-protocol_whitelist", "file,pipe", "-i", &fx.url("/static/v.mp4"), "-c", "copy", "net.mp4"], &dir);
    c.ok(!okh && eh.contains("whitelist"), &format!("ffmpeg with -protocol_whitelist file,pipe refuses an http input ({})", eh.lines().next().unwrap_or("")));
    std::fs::write(dir.join("list.m3u8"), format!("#EXTM3U\n#EXT-X-TARGETDURATION:2\n#EXTINF:2,\nhttp://127.0.0.1:{cport}/seg.ts\n#EXT-X-ENDLIST\n")).unwrap();
    let (okl, el) = ffmpeg(&["-protocol_whitelist", "file,pipe", "-i", "list.m3u8", "-c", "copy", "hls.mp4"], &dir);
    c.ok(!okl && *hits.lock().unwrap() == 0, &format!("local playlist pointing at http: refused by the whitelist, canary 0 ({})", el.lines().next().unwrap_or("")));

    // 7. proxy closes with the run
    px.task.abort();
    tokio::time::sleep(Duration::from_millis(200)).await;
    c.ok(TcpStream::connect(px.addr).await.is_err(), "proxy port closed after the run (not an API)");
    c.note("ADR-6 guarantee = only requests that honour the pinned --proxy are policed; a bypassing extractor is not confined (no WFP, not admin). Job Object limits not in this spike.");
    c.finish();
}
