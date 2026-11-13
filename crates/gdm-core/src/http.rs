// SPDX-License-Identifier: RPL-1.5
//! Pure response rules (§09 server-response table). No I/O here, so every row is table-tested.
use crate::Fail;
use reqwest::header::HeaderMap;

/// what a worker / probe does with a status code
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    Accept,
    /// 429 / 503: wait (Retry-After, capped 60 s) and halve the connection target (E-05)
    RetryAfter(u64),
    /// other 5xx: backoff 1 -> 60 s, 10 tries, then pause
    Backoff(u16),
    Fatal(Fail),
}

pub fn status(code: u16, h: &HeaderMap) -> Step {
    match code {
        200 | 206 => Step::Accept,
        429 | 503 => Step::RetryAfter(retry_after(h).unwrap_or(5).min(60)),
        401 | 403 => Step::Fatal(Fail::Auth),
        404 | 410 => Step::Fatal(Fail::Gone),
        416 => Step::Fatal(Fail::SizeChanged),
        500..=599 => Step::Backoff(code),
        c => Step::Fatal(Fail::Http(c)),
    }
}

fn retry_after(h: &HeaderMap) -> Option<u64> {
    let v = hv(h, "retry-after")?;
    if let Ok(s) = v.trim().parse() {
        return Some(s);
    }
    let at = httpdate::parse_http_date(&v).ok()?;
    Some(at.duration_since(std::time::SystemTime::now()).map_or(0, |d| d.as_secs()))
}

pub fn hv(h: &HeaderMap, k: &str) -> Option<String> {
    h.get(k).and_then(|v| v.to_str().ok()).map(String::from)
}

/// strong ETag only ("W/" is weak and never a validator for If-Range, RFC 9110 §13.1.5)
pub fn strong_etag(h: &HeaderMap) -> Option<String> {
    hv(h, "etag").map(|e| e.trim().to_string()).filter(|e| e.starts_with('"') && e.len() >= 2)
}

/// validator for If-Range: strong ETag, else Last-Modified only when it is strong per RFC 9110 §8.8.2.2
/// (a Date header at least one second later). None => resume integrity is a heuristic (E-02 "unverified").
pub fn validator(h: &HeaderMap) -> Option<String> {
    if let Some(e) = strong_etag(h) {
        return Some(e);
    }
    let lm = hv(h, "last-modified")?;
    let date = hv(h, "date")?;
    let (l, d) = (httpdate::parse_http_date(&lm).ok()?, httpdate::parse_http_date(&date).ok()?);
    (d.duration_since(l).is_ok_and(|x| x.as_secs() >= 1)).then_some(lm)
}

/// "bytes a-b/total" -> (a, b inclusive, Some(total)); "/*" -> None total
pub fn content_range(h: &HeaderMap) -> Option<(u64, u64, Option<u64>)> {
    let v = hv(h, "content-range")?;
    let r = v.trim().strip_prefix("bytes ")?;
    let (ab, t) = r.split_once('/')?;
    let (a, b) = ab.split_once('-')?;
    let (a, b): (u64, u64) = (a.trim().parse().ok()?, b.trim().parse().ok()?);
    if b < a {
        return None;
    }
    let total = if t.trim() == "*" { None } else { Some(t.trim().parse().ok()?) };
    if total.is_some_and(|t: u64| b >= t) {
        return None;
    }
    Some((a, b, total))
}

/// E-01 acceptance of a ranged reply: 206 AND Content-Range starts at the requested `a` AND the total equals the
/// initial one. The server may send less than asked, or more (some servers ignore the range end and send to the
/// end of the file, IDM accepts it): the worker keeps only its lease [a, end) and drops the connection there.
pub fn range_ok(h: &HeaderMap, a: u64, _end: u64, total: Option<u64>) -> bool {
    match content_range(h) {
        Some((x, _, t)) => x == a && t == total,
        None => false,
    }
}

/// a body must be stored byte for byte (E-09): anything but identity encoding is refused on ranges
pub fn identity(h: &HeaderMap) -> bool {
    hv(h, "content-encoding").is_none_or(|e| e.trim().eq_ignore_ascii_case("identity"))
}

pub fn content_type(h: &HeaderMap) -> String {
    hv(h, "content-type").unwrap_or_default().to_ascii_lowercase()
}
