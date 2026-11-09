// SPDX-License-Identifier: RPL-1.5
//! S4 check: every hostile route behaves exactly as specified on the wire (raw TCP client, no HTTP library
//! that could normalise the response). `s4 serve <port>` runs the fixture for manual use / IDM benchmark.
use gdm_spikes::fixture::{expected_sha256, fill, pct_encode, Fixture};
use gdm_spikes::Check;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// send one request, read until close / limit / timeout; returns (head, body bytes, timed_out)
async fn raw(addr: std::net::SocketAddr, req: &str, limit: usize, wait: Duration) -> (String, Vec<u8>, bool) {
    let mut s = TcpStream::connect(addr).await.unwrap();
    s.write_all(req.as_bytes()).await.unwrap();
    let mut all = Vec::new();
    let mut buf = [0u8; 65536];
    let dl = Instant::now() + wait;
    let mut timed_out = false;
    while all.len() < limit {
        match tokio::time::timeout_at(dl.into(), s.read(&mut buf)).await {
            Err(_) => { timed_out = true; break; }
            Ok(Ok(0)) | Ok(Err(_)) => break,
            Ok(Ok(n)) => all.extend_from_slice(&buf[..n]),
        }
        // stop after a full Content-Length body (keep-alive)
        if let Some(i) = all.windows(4).position(|w| w == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&all[..i]).to_ascii_lowercase();
            if let Some(cl) = head.lines().find_map(|l| l.strip_prefix("content-length:")).and_then(|v| v.trim().parse::<usize>().ok()) {
                if all.len() >= i + 4 + cl { break; }
            }
        }
    }
    match all.windows(4).position(|w| w == b"\r\n\r\n") {
        Some(i) => (String::from_utf8_lossy(&all[..i]).to_string(), all[i + 4..].to_vec(), timed_out),
        None => (String::from_utf8_lossy(&all).to_string(), vec![], timed_out),
    }
}

fn get(path: &str, extra: &str) -> String { format!("GET {path} HTTP/1.1\r\nHost: x\r\n{extra}\r\n") }

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("serve") {
        let fx = Fixture::start(&format!("127.0.0.1:{}", args.get(2).map(String::as_str).unwrap_or("8790"))).await;
        println!("fixture on http://{} (e.g. /file?size=1073741824&seed=7)", fx.addr);
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let log = fx.log();
            if !log.is_empty() { println!("requests={} distinct_conns={} peak_active={}", log.len(), log.iter().map(|r| r.conn).collect::<std::collections::HashSet<_>>().len(), fx.peak_conns()); }
        }
    }
    let mut c = Check::new("S4 hostile fixture");
    let a = Fixture::start("127.0.0.1:0").await;
    let b = Fixture::start("127.0.0.1:0").await;
    let t = Duration::from_secs(5);

    // control: ranged file, exact bytes and validator
    let (h, body, _) = raw(a.addr, &get("/file?size=100000&seed=3", "Range: bytes=10-19\r\n"), 1 << 20, t).await;
    let mut want = [0u8; 10];
    fill(3, 10, &mut want);
    c.ok(h.starts_with("HTTP/1.1 206") && h.contains("Content-Range: bytes 10-19/100000") && body == want, "/file 206 exact range bytes");
    let (h, _, _) = raw(a.addr, &get("/file?size=100000&seed=3", "Range: bytes=10-\r\nIf-Range: \"other\"\r\n"), 1 << 20, t).await;
    c.ok(h.starts_with("HTTP/1.1 200"), "/file If-Range mismatch -> 200 full");
    let (h, _, _) = raw(a.addr, &get("/file?size=100&seed=3", "Range: bytes=500-\r\n"), 1 << 20, t).await;
    c.ok(h.starts_with("HTTP/1.1 416") && h.contains("bytes */100"), "/file range past end -> 416");
    c.ok(expected_sha256(3, 0) == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", "oracle sha256 of empty body is the known constant");

    let (h, body, _) = raw(a.addr, &get("/norange?size=5000&seed=1", "Range: bytes=100-\r\n"), 1 << 20, t).await;
    c.ok(h.starts_with("HTTP/1.1 200") && body.len() == 5000 && !h.contains("Accept-Ranges"), "/norange ignores Range (200, full body)");

    let (h1, _, _) = raw(a.addr, &get("/range-then-200?size=5000&seed=9", "Range: bytes=0-\r\n"), 1 << 20, t).await;
    let (h2, _, _) = raw(a.addr, &get("/range-then-200?size=5000&seed=9", "Range: bytes=100-\r\n"), 1 << 20, t).await;
    c.ok(h1.starts_with("HTTP/1.1 206") && h2.starts_with("HTTP/1.1 200"), "/range-then-200: 206 then 200 mid-download");

    let et = |h: &str| h.lines().find(|l| l.starts_with("ETag")).unwrap_or("").to_string();
    let (e1, b1, _) = raw(a.addr, &get("/etag-change?size=4000&seed=5&after=1", "Range: bytes=0-\r\n"), 1 << 20, t).await;
    let (e2, b2, _) = raw(a.addr, &get("/etag-change?size=4000&seed=5&after=1", "Range: bytes=0-\r\n"), 1 << 20, t).await;
    c.ok(et(&e1) != et(&e2) && b1.len() == b2.len() && b1 != b2, "/etag-change: same size, new ETag, new bytes");

    let (h, body, to) = raw(a.addr, &get("/infinite", ""), 8 << 20, t).await;
    c.ok(h.starts_with("HTTP/1.1 200") && !h.to_ascii_lowercase().contains("content-length") && body.len() >= (8 << 20) - 65536 && !to, "/infinite: no length, body keeps coming (read 8 MiB)");

    let (h, body, _) = raw(a.addr, &get("/overlong?size=1000&seed=2", "Range: bytes=0-99\r\n"), 1 << 20, t).await;
    c.ok(h.contains("Content-Range: bytes 0-99/1000") && body.len() > 100, "/overlong: more bytes than Content-Range");
    let (h, body, _) = raw(a.addr, &get("/short?size=1000&seed=2", "Range: bytes=0-999\r\n"), 1 << 20, t).await;
    c.ok(h.contains("Content-Length: 1000") && body.len() == 500, "/short: body ends at half and connection closes");
    let (h, _, _) = raw(a.addr, &get("/bad-cr?size=100000&seed=2", "Range: bytes=0-\r\n"), 1 << 20, t).await;
    c.ok(h.contains("Content-Range: bytes 4096-"), "/bad-cr: Content-Range start differs from requested");

    let t0 = Instant::now();
    let (h, _, to) = raw(a.addr, &get("/slowloris", ""), 1 << 20, Duration::from_secs(4)).await;
    c.ok(to && h.len() <= 6 && t0.elapsed() >= Duration::from_secs(4), "/slowloris: < 6 header bytes in 4 s, never finishes");
    let (h, body, to) = raw(a.addr, &get("/slowbody?size=100", ""), 1 << 20, Duration::from_secs(5)).await;
    c.ok(to && h.starts_with("HTTP/1.1 200") && body.len() <= 3, "/slowbody: 1 byte per 2 s");

    let target = b.url("/echo");
    let (h, _, _) = raw(a.addr, &get(&format!("/redirect?to={}", pct_encode(&target)), ""), 1 << 20, t).await;
    c.ok(h.starts_with("HTTP/1.1 302") && h.contains(&format!("Location: {target}")), "/redirect: 302 to another origin (different port)");

    let (h1, _, _) = raw(a.addr, &get("/429?size=10&seed=77", ""), 1 << 20, t).await;
    let (h2, _, _) = raw(a.addr, &get("/429?size=10&seed=77", ""), 1 << 20, t).await;
    c.ok(h1.starts_with("HTTP/1.1 429") && h1.contains("Retry-After: 2") && h2.starts_with("HTTP/1.1 200"), "/429: first 429 + Retry-After: 2, then 200");

    let (h, body, _) = raw(a.addr, &get("/html", "Range: bytes=0-\r\n"), 1 << 20, t).await;
    c.ok(h.starts_with("HTTP/1.1 200") && h.contains("text/html") && body.starts_with(b"<!doctype"), "/html: text/html instead of binary");
    let (h, _, _) = raw(a.addr, &get("/multipart", "Range: bytes=0-3\r\n"), 1 << 20, t).await;
    c.ok(h.starts_with("HTTP/1.1 206") && h.contains("multipart/byteranges"), "/multipart: multipart/byteranges to a single range");

    let log = a.log();
    c.ok(log.iter().any(|r| r.target.starts_with("/file") && r.headers.iter().any(|(k, v)| k == "range" && v == "bytes=10-19")), "server log records every request with headers (trace evidence)");
    c.finish();
}
