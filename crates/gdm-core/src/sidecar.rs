// SPDX-License-Identifier: RPL-1.5
//! `name.part.gdm` (ADR-3, D-01): the minimum needed to recover a job when the database is lost. It holds no
//! secrets (no cookies, no Authorization, URLs without userinfo/query/fragment: a signed URL's token stays in
//! SQLite only; a recovered job whose URL needed it fails with Auth and goes through Refresh, F06) and is UNTRUSTED on load: size cap before parsing, strict shape and
//! arithmetic checks; anything odd => None. Its offsets are claims that the seam check re-verifies.
use serde::{Deserialize, Serialize};

pub const MAX: usize = 64 * 1024;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Sidecar {
    pub v: u32,
    pub job: String,
    pub url: String,
    pub final_url: String,
    pub name: String,
    pub validator: Option<String>,
    pub total: u64,
    pub epoch: u64,
    /// [start, end exclusive, committed]
    pub segs: Vec<[u64; 3]>,
}

pub fn leaf(name: &str) -> String {
    format!("{name}.part.gdm")
}

/// merge a fully committed segment into the next one: [a,b,b] + [b,c,x] = [a,c,x] (keeps the file small)
pub fn compact(segs: &[[u64; 3]]) -> Vec<[u64; 3]> {
    let mut out: Vec<[u64; 3]> = Vec::with_capacity(segs.len());
    for s in segs {
        match out.last_mut() {
            Some(p) if p[2] == p[1] && p[1] == s[0] => {
                p[1] = s[1];
                p[2] = s[2];
            }
            _ => out.push(*s),
        }
    }
    out
}

pub fn encode(s: &Sidecar) -> Option<Vec<u8>> {
    let v = serde_json::to_vec(s).ok()?;
    (v.len() <= MAX).then_some(v)
}

pub fn decode(raw: &[u8]) -> Option<Sidecar> {
    if raw.len() > MAX {
        return None;
    }
    let sc: Sidecar = serde_json::from_slice(raw).ok()?;
    if sc.v != 1 || sc.segs.is_empty() || sc.segs.len() > 4096 || sc.total == 0 {
        return None;
    }
    if sc.job.is_empty() || sc.job.len() > 64 || !sc.job.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        return None;
    }
    for u in [&sc.url, &sc.final_url] {
        let p = url::Url::parse(u).ok()?;
        if !matches!(p.scheme(), "http" | "https") || p.query().is_some() || !p.username().is_empty() || p.fragment().is_some() {
            return None;
        }
    }
    if sc.name != crate::name::sanitize(&sc.name) {
        return None;
    }
    let mut prev = 0;
    for &[a, e, d] in &sc.segs {
        if a != prev || e <= a || e > sc.total || d < a || d > e {
            return None;
        }
        prev = e;
    }
    (prev == sc.total).then_some(sc)
}
