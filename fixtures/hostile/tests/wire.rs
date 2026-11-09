// SPDX-License-Identifier: RPL-1.5
//! Wire check of every hostile route with a raw TCP client (no HTTP library that could normalise the
//! response). Ported from the G3 S4 spike check; each route is its own test.
use gdm_fixture::{expected_sha256, fill, pct_encode, Fixture};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const T: Duration = Duration::from_secs(5);

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
            Err(_) => {
                timed_out = true;
                break;
            }
            Ok(Ok(0)) | Ok(Err(_)) => break,
            Ok(Ok(n)) => all.extend_from_slice(&buf[..n]),
        }
        // stop after a full Content-Length body (keep-alive)
        if let Some(i) = all.windows(4).position(|w| w == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&all[..i]).to_ascii_lowercase();
            if let Some(cl) = head.lines().find_map(|l| l.strip_prefix("content-length:")).and_then(|v| v.trim().parse::<usize>().ok()) {
                if all.len() >= i + 4 + cl {
                    break;
                }
            }
        }
    }
    match all.windows(4).position(|w| w == b"\r\n\r\n") {
        Some(i) => (String::from_utf8_lossy(&all[..i]).to_string(), all[i + 4..].to_vec(), timed_out),
        None => (String::from_utf8_lossy(&all).to_string(), vec![], timed_out),
    }
}

fn get(path: &str, extra: &str) -> String {
    format!("GET {path} HTTP/1.1\r\nHost: x\r\n{extra}\r\n")
}

async fn fx() -> Fixture {
    Fixture::start("127.0.0.1:0").await
}

#[tokio::test]
async fn file_ranges_validators_416() {
    let a = fx().await;
    let (h, body, _) = raw(a.addr, &get("/file?size=100000&seed=3", "Range: bytes=10-19\r\n"), 1 << 20, T).await;
    let mut want = [0u8; 10];
    fill(3, 10, &mut want);
    assert!(h.starts_with("HTTP/1.1 206") && h.contains("Content-Range: bytes 10-19/100000") && body == want);
    let (h, _, _) = raw(a.addr, &get("/file?size=100000&seed=3", "Range: bytes=10-\r\nIf-Range: \"other\"\r\n"), 1 << 20, T).await;
    assert!(h.starts_with("HTTP/1.1 200"), "If-Range mismatch -> 200 full");
    let (h, _, _) = raw(a.addr, &get("/file?size=100&seed=3", "Range: bytes=500-\r\n"), 1 << 20, T).await;
    assert!(h.starts_with("HTTP/1.1 416") && h.contains("bytes */100"));
    assert_eq!(expected_sha256(3, 0), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    assert!(
        a.log().iter().any(|r| r.target.starts_with("/file") && r.headers.iter().any(|(k, v)| k == "range" && v == "bytes=10-19")),
        "log records headers"
    );
}

#[tokio::test]
async fn norange_and_range_then_200() {
    let a = fx().await;
    let (h, body, _) = raw(a.addr, &get("/norange?size=5000&seed=1", "Range: bytes=100-\r\n"), 1 << 20, T).await;
    assert!(h.starts_with("HTTP/1.1 200") && body.len() == 5000 && !h.contains("Accept-Ranges"));
    let (h1, _, _) = raw(a.addr, &get("/range-then-200?size=5000&seed=9", "Range: bytes=0-\r\n"), 1 << 20, T).await;
    let (h2, _, _) = raw(a.addr, &get("/range-then-200?size=5000&seed=9", "Range: bytes=100-\r\n"), 1 << 20, T).await;
    assert!(h1.starts_with("HTTP/1.1 206") && h2.starts_with("HTTP/1.1 200"));
}

#[tokio::test]
async fn etag_change() {
    let a = fx().await;
    let et = |h: &str| h.lines().find(|l| l.starts_with("ETag")).unwrap_or("").to_string();
    let (e1, b1, _) = raw(a.addr, &get("/etag-change?size=4000&seed=5&after=1", "Range: bytes=0-\r\n"), 1 << 20, T).await;
    let (e2, b2, _) = raw(a.addr, &get("/etag-change?size=4000&seed=5&after=1", "Range: bytes=0-\r\n"), 1 << 20, T).await;
    assert!(et(&e1) != et(&e2) && b1.len() == b2.len() && b1 != b2);
}

#[tokio::test]
async fn infinite_overlong_short_badcr() {
    let a = fx().await;
    let (h, body, to) = raw(a.addr, &get("/infinite", ""), 8 << 20, T).await;
    assert!(h.starts_with("HTTP/1.1 200") && !h.to_ascii_lowercase().contains("content-length") && body.len() >= (8 << 20) - 65536 && !to);
    let (h, body, _) = raw(a.addr, &get("/overlong?size=1000&seed=2", "Range: bytes=0-99\r\n"), 1 << 20, T).await;
    assert!(h.contains("Content-Range: bytes 0-99/1000") && body.len() > 100);
    let (h, body, _) = raw(a.addr, &get("/short?size=1000&seed=2", "Range: bytes=0-999\r\n"), 1 << 20, T).await;
    assert!(h.contains("Content-Length: 1000") && body.len() == 500);
    let (h, _, _) = raw(a.addr, &get("/bad-cr?size=100000&seed=2", "Range: bytes=0-\r\n"), 1 << 20, T).await;
    assert!(h.contains("Content-Range: bytes 4096-"));
}

#[tokio::test]
async fn slowloris_and_slowbody() {
    let a = fx().await;
    let t0 = Instant::now();
    let (h, _, to) = raw(a.addr, &get("/slowloris", ""), 1 << 20, Duration::from_secs(4)).await;
    assert!(to && h.len() <= 6 && t0.elapsed() >= Duration::from_secs(4));
    let (h, body, to) = raw(a.addr, &get("/slowbody?size=100", ""), 1 << 20, Duration::from_secs(5)).await;
    assert!(to && h.starts_with("HTTP/1.1 200") && body.len() <= 3);
}

#[tokio::test]
async fn redirect_429_html_multipart() {
    let a = fx().await;
    let b = fx().await;
    let target = b.url("/echo");
    let (h, _, _) = raw(a.addr, &get(&format!("/redirect?to={}", pct_encode(&target)), ""), 1 << 20, T).await;
    assert!(h.starts_with("HTTP/1.1 302") && h.contains(&format!("Location: {target}")));
    let (h1, _, _) = raw(a.addr, &get("/429?size=10&seed=77", ""), 1 << 20, T).await;
    let (h2, _, _) = raw(a.addr, &get("/429?size=10&seed=77", ""), 1 << 20, T).await;
    assert!(h1.starts_with("HTTP/1.1 429") && h1.contains("Retry-After: 2") && h2.starts_with("HTTP/1.1 200"));
    let (h, body, _) = raw(a.addr, &get("/html", "Range: bytes=0-\r\n"), 1 << 20, T).await;
    assert!(h.starts_with("HTTP/1.1 200") && h.contains("text/html") && body.starts_with(b"<!doctype"));
    let (h, _, _) = raw(a.addr, &get("/multipart", "Range: bytes=0-3\r\n"), 1 << 20, T).await;
    assert!(h.starts_with("HTTP/1.1 206") && h.contains("multipart/byteranges"));
}
