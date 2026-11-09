// SPDX-License-Identifier: RPL-1.5
//! Hostile HTTP/1.1 fixture server (raw tokio TCP, so it can send framing hyper would refuse).
//! Moved from spikes/engine (G3 S4) in G5. Every request is logged (the trace is the evidence).
//!
//! Routes (query: size, seed, rate=bytes/s, after=N, to=URL):
//!   /file          206 ranges, strong ETag, If-Range honoured          (control)
//!   /norange       200 full body, ignores Range                        (E-01)
//!   /range-then-200 first request 206, later range requests 200        (E-01 mid-download)
//!   /etag-change   after N requests: new ETag and new bytes, same size (E-02)
//!   /infinite      200, no length, body never ends                     (E-03)
//!   /overlong      206 correct Content-Range, then extra bytes          (invariant 5)
//!   /short         206, body ends early, connection closed
//!   /bad-cr        206 with Content-Range start != requested start     (invariant 5)
//!   /slowloris     header bytes 1 per second, forever                  (E-03)
//!   /slowbody      headers fine, then 1 body byte every 2 s
//!   /redirect      302 Location: <to>                                  (W-01)
//!   /echo          200 "ok" (the log shows which headers arrived)
//!   /cr-lie        206 whose Content-Range declares 1 KiB, no Content-Length, then sends the whole range
//!   /novalidator   206 ranges with no ETag and no Last-Modified          (E-02 unverified)
//!   /429           first request per seed: 429 Retry-After: 2, then /file
//!   /html          200 text/html page                                   (R-03)
//!   /multipart     206 multipart/byteranges                             (not requested => reject)
//!   /p/<size>/<seed>/<rate>/<name>  same as /file, parameters in the path (URLs without a query)
//!   /static/<name> bytes of a registered blob, ranged (media for S7)
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Clone, Debug)]
pub struct ReqLog {
    pub conn: usize,
    pub method: String,
    pub target: String,
    pub headers: Vec<(String, String)>,
    pub ms: u128,
}

#[derive(Default)]
struct State {
    log: Mutex<Vec<ReqLog>>,
    counters: Mutex<HashMap<String, u64>>,
    pace: Mutex<Option<Instant>>, // shared clock for trate= (total bandwidth cap across connections)
    blobs: Mutex<HashMap<String, Arc<Vec<u8>>>>,
    active: AtomicUsize,
    peak: AtomicUsize,
    conns: AtomicUsize,
}

#[derive(Clone)]
pub struct Fixture {
    pub addr: SocketAddr,
    st: Arc<State>,
    t0: Instant,
}

impl Fixture {
    pub async fn start(bind: &str) -> Fixture {
        let l = TcpListener::bind(bind).await.expect("bind fixture");
        let fx = Fixture { addr: l.local_addr().unwrap(), st: Arc::default(), t0: Instant::now() };
        let f2 = fx.clone();
        tokio::spawn(async move {
            loop {
                let Ok((s, _)) = l.accept().await else { continue };
                let f = f2.clone();
                tokio::spawn(async move {
                    let id = f.st.conns.fetch_add(1, Ordering::SeqCst);
                    let n = f.st.active.fetch_add(1, Ordering::SeqCst) + 1;
                    f.st.peak.fetch_max(n, Ordering::SeqCst);
                    let _ = f.serve(s, id).await;
                    f.st.active.fetch_sub(1, Ordering::SeqCst);
                });
            }
        });
        fx
    }
    pub fn url(&self, p: &str) -> String {
        format!("http://{}{}", self.addr, p)
    }
    pub fn log(&self) -> Vec<ReqLog> {
        self.st.log.lock().unwrap().clone()
    }
    pub fn clear_log(&self) {
        self.st.log.lock().unwrap().clear();
        self.st.peak.store(0, Ordering::SeqCst);
    }
    pub fn peak_conns(&self) -> usize {
        self.st.peak.load(Ordering::SeqCst)
    }
    pub fn add_blob(&self, name: &str, b: Vec<u8>) {
        self.st.blobs.lock().unwrap().insert(name.into(), Arc::new(b));
    }

    fn bump(&self, key: String) -> u64 {
        let mut c = self.st.counters.lock().unwrap();
        let v = c.entry(key).or_insert(0);
        *v += 1;
        *v
    }

    async fn serve(&self, mut s: TcpStream, conn: usize) -> std::io::Result<()> {
        let mut buf = Vec::new();
        loop {
            // read one request head
            let head = loop {
                if let Some(i) = find(&buf, b"\r\n\r\n") {
                    let h = String::from_utf8_lossy(&buf[..i]).to_string();
                    buf.drain(..i + 4);
                    break h;
                }
                if buf.len() > 64 * 1024 {
                    return Ok(());
                }
                let mut tmp = [0u8; 8192];
                let n = s.read(&mut tmp).await?;
                if n == 0 {
                    return Ok(());
                }
                buf.extend_from_slice(&tmp[..n]);
            };
            let mut lines = head.split("\r\n");
            let first = lines.next().unwrap_or("");
            let mut it = first.split(' ');
            let method = it.next().unwrap_or("").to_string();
            let target = it.next().unwrap_or("/").to_string();
            let headers: Vec<(String, String)> =
                lines.filter_map(|l| l.split_once(':').map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))).collect();
            self.st.log.lock().unwrap().push(ReqLog {
                conn,
                method: method.clone(),
                target: target.clone(),
                headers: headers.clone(),
                ms: self.t0.elapsed().as_millis(),
            });
            let keep = self.respond(&mut s, &method, &target, &headers).await?;
            if !keep {
                return Ok(());
            }
        }
    }

    /// returns false when the connection must close
    async fn respond(&self, s: &mut TcpStream, method: &str, target: &str, h: &[(String, String)]) -> std::io::Result<bool> {
        let (path, q) = target.split_once('?').unwrap_or((target, ""));
        // /p/<size>/<seed>/<rate>/<name> == /file?size=&seed=&rate=
        let pq;
        let (path, q) = match path.strip_prefix("/p/").map(|r| r.split('/').collect::<Vec<_>>()) {
            Some(v) if v.len() >= 3 => {
                pq = format!("size={}&seed={}&rate={}", v[0], v[1], v[2]);
                ("/file", pq.as_str())
            }
            _ => (path, q),
        };
        let qp: HashMap<&str, &str> = q.split('&').filter_map(|kv| kv.split_once('=')).collect();
        let num = |k: &str, d: u64| qp.get(k).and_then(|v| v.parse().ok()).unwrap_or(d);
        let size = num("size", 1 << 20);
        let seed = num("seed", 1);
        let rate = num("rate", 0);
        let trate = num("trate", 0);
        let hdr = |k: &str| h.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str());
        let range = hdr("range").and_then(parse_range);
        let head_only = method == "HEAD";
        match path {
            "/file" => self.ranged(s, size, seed, rate, trate, range, hdr("if-range"), head_only, &format!("\"s{seed}-{size}\"")).await,
            "/429" => {
                if self.bump(format!("429-{seed}")) == 1 {
                    s.write_all(b"HTTP/1.1 429 Too Many Requests\r\nRetry-After: 2\r\nContent-Length: 0\r\n\r\n").await?;
                    return Ok(true);
                }
                self.ranged(s, size, seed, rate, trate, range, hdr("if-range"), head_only, &format!("\"s{seed}-{size}\"")).await
            }
            "/etag-change" => {
                let n = self.bump(format!("etag-{seed}"));
                let after = num("after", 2);
                let sd = if n > after { seed + 1 } else { seed };
                self.ranged(s, size, sd, rate, trate, range, hdr("if-range"), head_only, &format!("\"s{sd}-{size}\"")).await
            }
            "/range-then-200" => {
                let n = self.bump(format!("rt2-{seed}"));
                let r = if n == 1 { range } else { None };
                self.ranged(s, size, seed, rate, trate, r, None, head_only, &format!("\"s{seed}-{size}\"")).await
            }
            "/cr-lie" => {
                let (a, b) = clamp(range, size);
                let lie = (a + 1024).min(b);
                s.write_all(format!("HTTP/1.1 206 Partial Content\r\nContent-Type: application/octet-stream\r\nETag: \"s{seed}-{size}\"\r\nContent-Range: bytes {a}-{}/{size}\r\nConnection: close\r\n\r\n", lie - 1).as_bytes()).await?;
                send_bytes(s, seed, a, b, 0, None).await?;
                Ok(false)
            }
            "/novalidator" => {
                let (a, b) = clamp(range, size);
                let (code, cr) = if range.is_some() {
                    (
                        206,
                        format!(
                            "Content-Range: bytes {a}-{}/{size}
",
                            b.max(1) - 1
                        ),
                    )
                } else {
                    (200, String::new())
                };
                s.write_all(
                    format!(
                        "HTTP/1.1 {code} X
Content-Type: application/octet-stream
Accept-Ranges: bytes
{cr}Content-Length: {}

",
                        b - a
                    )
                    .as_bytes(),
                )
                .await?;
                if !head_only {
                    send_bytes(s, seed, a, b, rate, None).await?;
                }
                Ok(true)
            }
            "/norange" => {
                s.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {size}\r\n\r\n").as_bytes())
                    .await?;
                if !head_only {
                    send_bytes(s, seed, 0, size, rate, None).await?;
                }
                Ok(true)
            }
            "/infinite" => {
                s.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n").await?;
                let chunk = vec![0x5Au8; 64 * 1024];
                loop {
                    s.write_all(&chunk).await?;
                }
            }
            "/overlong" => {
                let (a, b) = clamp(range, size);
                s.write_all(format!("HTTP/1.1 206 Partial Content\r\nContent-Type: application/octet-stream\r\nContent-Range: bytes {a}-{}/{size}\r\nETag: \"o{seed}\"\r\nConnection: close\r\n\r\n", b - 1).as_bytes()).await?;
                send_bytes(s, seed, a, b, 0, None).await?;
                s.write_all(&vec![0xEEu8; 256 * 1024]).await?; // bytes past the range: must never land in the file
                Ok(false)
            }
            "/short" => {
                let (a, b) = clamp(range, size);
                s.write_all(format!("HTTP/1.1 206 Partial Content\r\nContent-Type: application/octet-stream\r\nContent-Range: bytes {a}-{}/{size}\r\nContent-Length: {}\r\nETag: \"s{seed}-{size}\"\r\n\r\n", b - 1, b - a).as_bytes()).await?;
                send_bytes(s, seed, a, a + (b - a) / 2, 0, None).await?;
                Ok(false)
            }
            "/bad-cr" => {
                let (a, b) = clamp(range, size);
                let lie = a + 4096.min(b - a);
                s.write_all(format!("HTTP/1.1 206 Partial Content\r\nContent-Type: application/octet-stream\r\nContent-Range: bytes {lie}-{}/{size}\r\nContent-Length: {}\r\nETag: \"s{seed}-{size}\"\r\n\r\n", b - 1, b - lie).as_bytes()).await?;
                send_bytes(s, seed, lie, b, 0, None).await?;
                Ok(true)
            }
            "/slowloris" => {
                for c in b"HTTP/1.1 200 OK\r\nX-Slow: ".iter().chain(std::iter::repeat(&b'a')) {
                    s.write_all(&[*c]).await?;
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Ok(false)
            }
            "/slowbody" => {
                s.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {size}\r\n\r\n").as_bytes())
                    .await?;
                for _ in 0..size {
                    s.write_all(b"x").await?;
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
                Ok(false)
            }
            "/redirect" => {
                let to = qp.get("to").map(|v| pct_decode(v)).unwrap_or_default();
                s.write_all(format!("HTTP/1.1 302 Found\r\nLocation: {to}\r\nContent-Length: 0\r\n\r\n").as_bytes()).await?;
                Ok(true)
            }
            "/echo" => {
                s.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 2\r\n\r\nok").await?;
                Ok(true)
            }
            "/html" => {
                let body = b"<!doctype html><html><body>Please log in</body></html>";
                s.write_all(
                    format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n", body.len()).as_bytes(),
                )
                .await?;
                s.write_all(body).await?;
                Ok(true)
            }
            "/multipart" => {
                let body = b"--X\r\nContent-Type: application/octet-stream\r\nContent-Range: bytes 0-3/100\r\n\r\nabcd\r\n--X--\r\n";
                s.write_all(
                    format!(
                        "HTTP/1.1 206 Partial Content\r\nContent-Type: multipart/byteranges; boundary=X\r\nContent-Length: {}\r\n\r\n",
                        body.len()
                    )
                    .as_bytes(),
                )
                .await?;
                s.write_all(body).await?;
                Ok(true)
            }
            p if p.starts_with("/static/") => {
                let blob = self.st.blobs.lock().unwrap().get(&p[8..]).cloned();
                let Some(b) = blob else {
                    s.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n").await?;
                    return Ok(true);
                };
                let ct = if p.ends_with(".html") { "text/html" } else { "application/octet-stream" };
                let total = b.len() as u64;
                let (code, a, e, cr) = match range {
                    Some(_) => {
                        let (a, e) = clamp(range, total);
                        (206, a, e, format!("Content-Range: bytes {a}-{}/{total}\r\n", e - 1))
                    }
                    None => (200, 0, total, String::new()),
                };
                s.write_all(
                    format!(
                        "HTTP/1.1 {code} X\r\nContent-Type: {ct}\r\nAccept-Ranges: bytes\r\nETag: \"b{total}\"\r\n{cr}Content-Length: {}\r\n\r\n",
                        e - a
                    )
                    .as_bytes(),
                )
                .await?;
                if !head_only {
                    s.write_all(&b[a as usize..e as usize]).await?;
                }
                Ok(true)
            }
            _ => {
                s.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n").await?;
                Ok(true)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn ranged(
        &self,
        s: &mut TcpStream,
        size: u64,
        seed: u64,
        rate: u64,
        trate: u64,
        range: Option<(u64, Option<u64>)>,
        if_range: Option<&str>,
        head_only: bool,
        etag: &str,
    ) -> std::io::Result<bool> {
        let lm = "Mon, 09 Nov 2026 00:00:00 GMT";
        let pace = (trate > 0).then_some((&self.st.pace, trate));
        let honour = range.is_some() && if_range.is_none_or(|v| v == etag || v == lm);
        let range = if honour { range } else { None };
        match range {
            Some((a, _)) if a >= size => {
                s.write_all(format!("HTTP/1.1 416 Range Not Satisfiable\r\nContent-Range: bytes */{size}\r\nContent-Length: 0\r\n\r\n").as_bytes())
                    .await?;
                Ok(true)
            }
            Some(_) => {
                let (a, b) = clamp(range, size);
                s.write_all(format!("HTTP/1.1 206 Partial Content\r\nContent-Type: application/octet-stream\r\nAccept-Ranges: bytes\r\nETag: {etag}\r\nLast-Modified: {lm}\r\nContent-Range: bytes {a}-{}/{size}\r\nContent-Length: {}\r\n\r\n", b - 1, b - a).as_bytes()).await?;
                if !head_only {
                    send_bytes(s, seed, a, b, rate, pace).await?;
                }
                Ok(true)
            }
            None => {
                s.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nAccept-Ranges: bytes\r\nETag: {etag}\r\nLast-Modified: {lm}\r\nContent-Length: {size}\r\n\r\n").as_bytes()).await?;
                if !head_only {
                    send_bytes(s, seed, 0, size, rate, pace).await?;
                } // a request without Range shares the link cap too
                Ok(true)
            }
        }
    }
}

fn find(h: &[u8], n: &[u8]) -> Option<usize> {
    h.windows(n.len()).position(|w| w == n)
}

/// "bytes=a-" or "bytes=a-b" (single range only)
fn parse_range(v: &str) -> Option<(u64, Option<u64>)> {
    let r = v.strip_prefix("bytes=")?;
    if r.contains(',') {
        return None;
    }
    let (a, b) = r.split_once('-')?;
    Some((a.trim().parse().ok()?, b.trim().parse().ok()))
}

/// -> [a, end) exclusive
fn clamp(r: Option<(u64, Option<u64>)>, size: u64) -> (u64, u64) {
    match r {
        Some((a, Some(b))) => (a.min(size), (b + 1).min(size)),
        Some((a, None)) => (a.min(size), size),
        None => (0, size),
    }
}

fn pct_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut o = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                o.push(v);
                i += 3;
                continue;
            }
        }
        o.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&o).into()
}

pub fn pct_encode(s: &str) -> String {
    s.bytes().map(|c| if c.is_ascii_alphanumeric() || b"-_.~".contains(&c) { (c as char).to_string() } else { format!("%{c:02X}") }).collect()
}

/// Deterministic content: 8-byte block k = splitmix64(seed, k). Random access, so any range can be produced.
pub fn fill(seed: u64, off: u64, out: &mut [u8]) {
    let mut i = 0usize;
    while i < out.len() {
        let pos = off + i as u64;
        let k = pos / 8;
        let blk = splitmix(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ k).to_le_bytes();
        let start = (pos % 8) as usize;
        let n = (8 - start).min(out.len() - i);
        out[i..i + n].copy_from_slice(&blk[start..start + n]);
        i += n;
    }
}

fn splitmix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

async fn send_bytes(s: &mut TcpStream, seed: u64, a: u64, b: u64, rate: u64, pace: Option<(&Mutex<Option<Instant>>, u64)>) -> std::io::Result<()> {
    let mut buf = vec![0u8; 256 * 1024];
    let mut pos = a;
    let t0 = Instant::now();
    while pos < b {
        let n = ((b - pos) as usize).min(if rate > 0 { (rate as usize / 20).clamp(1024, buf.len()) } else { buf.len() });
        let n = match pace {
            Some((_, t)) => n.min((t as usize / 50).clamp(1024, buf.len())),
            None => n,
        };
        if let Some((clock, t)) = pace {
            // reserve n bytes on the shared clock: all connections together get at most t bytes/s
            let at = {
                let mut c = clock.lock().unwrap();
                let now = Instant::now();
                let at = c.map_or(now, |x| x.max(now - Duration::from_millis(20))); // 20 ms credit repays timer oversleep
                *c = Some(at + Duration::from_secs_f64(n as f64 / t as f64));
                at
            };
            // Windows timers tick ~15.6 ms: sleeping per 256 KiB chunk capped each connection at ~17 MB/s.
            // Sleep only when >= 20 ms ahead of the shared clock; the average stays <= t (burst <= 20 ms).
            if at > Instant::now() + Duration::from_millis(20) {
                tokio::time::sleep_until(at.into()).await;
            }
        }
        fill(seed, pos, &mut buf[..n]);
        s.write_all(&buf[..n]).await?;
        pos += n as u64;
        if rate > 0 {
            let due = Duration::from_secs_f64((pos - a) as f64 / rate as f64);
            if let Some(w) = due.checked_sub(t0.elapsed()) {
                tokio::time::sleep(w).await;
            }
        }
    }
    Ok(())
}

/// sha256 of the deterministic body (the oracle for "file is correct")
pub fn expected_sha256(seed: u64, size: u64) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    let mut pos = 0;
    while pos < size {
        let n = ((size - pos) as usize).min(buf.len());
        fill(seed, pos, &mut buf[..n]);
        h.update(&buf[..n]);
        pos += n as u64;
    }
    hex(&h.finalize())
}

pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
