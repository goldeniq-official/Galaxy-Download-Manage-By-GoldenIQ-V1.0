// SPDX-License-Identifier: RPL-1.5
//! HTTP Digest client (RFC 7616), deliberately limited: qop=auth or RFC 2069
//! style (no qop); MD5, MD5-sess, SHA-256, SHA-256-sess. auth-int and other
//! algorithms are refused. `userhash=true` is supported (§3.4.4). A username
//! that is not plain printable ASCII is sent as `username*` (RFC 8187), or
//! hashed when the server asked for userhash.

use md5::Md5;
use sha2::{Digest as _, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alg {
    Md5,
    Md5Sess,
    Sha256,
    Sha256Sess,
}

impl Alg {
    fn name(self) -> &'static str {
        match self {
            Alg::Md5 => "MD5",
            Alg::Md5Sess => "MD5-sess",
            Alg::Sha256 => "SHA-256",
            Alg::Sha256Sess => "SHA-256-sess",
        }
    }
    fn strength(self) -> u8 {
        matches!(self, Alg::Sha256 | Alg::Sha256Sess) as u8
    }
    fn h(self, s: &str) -> String {
        let d: Vec<u8> = match self {
            Alg::Md5 | Alg::Md5Sess => Md5::digest(s).to_vec(),
            Alg::Sha256 | Alg::Sha256Sess => Sha256::digest(s).to_vec(),
        };
        d.iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Challenge {
    pub realm: String,
    pub nonce: String,
    pub opaque: Option<String>,
    pub algorithm: Alg,
    pub qop_auth: bool,
    pub stale: bool,
    pub userhash: bool,
}

type Params = Vec<(String, String)>;

/// Splits a header value into `(scheme lowercased, params)`; names are
/// lowercased, quoted values unescaped. A token68 (Basic/Negotiate data) is skipped.
fn challenges(s: &str) -> Vec<(String, Params)> {
    let b = s.as_bytes();
    let ws = |c: u8| c == b' ' || c == b'\t';
    let (mut out, mut i, mut after_comma) = (Vec::<(String, Params)>::new(), 0, true);
    loop {
        while i < b.len() && (ws(b[i]) || b[i] == b',') {
            after_comma |= b[i] == b',';
            i += 1;
        }
        if i >= b.len() {
            return out;
        }
        let st = i;
        while i < b.len() && !ws(b[i]) && b[i] != b',' && b[i] != b'=' {
            i += 1;
        }
        let tok = &s[st..i];
        let mut j = i;
        while j < b.len() && ws(b[j]) {
            j += 1;
        }
        if j < b.len() && b[j] == b'=' {
            let mut k = j + 1;
            while k < b.len() && ws(b[k]) {
                k += 1;
            }
            if k >= b.len() || b[k] == b',' || b[k] == b'=' {
                // token68 padding ("abc==")
                while i < b.len() && (b[i] == b'=' || ws(b[i])) {
                    i += 1;
                }
                after_comma = false;
                continue;
            }
            let mut val = String::new();
            if b[k] == b'"' {
                let base = k + 1;
                let mut chars = s[base..].char_indices();
                k = s.len();
                while let Some((n, c)) = chars.next() {
                    match c {
                        '\\' => val.extend(chars.next().map(|(_, c)| c)),
                        '"' => {
                            k = base + n + 1;
                            break;
                        }
                        _ => val.push(c),
                    }
                }
            } else {
                let st = k;
                while k < b.len() && !ws(b[k]) && b[k] != b',' {
                    k += 1;
                }
                val.push_str(&s[st..k]);
            }
            if let Some(last) = out.last_mut() {
                last.1.push((tok.to_ascii_lowercase(), val));
            }
            i = k;
        } else {
            if after_comma {
                out.push((tok.to_ascii_lowercase(), Vec::new()));
            }
            i = j;
        }
        after_comma = false;
    }
}

fn digest_challenge(p: &Params) -> Result<Challenge, String> {
    let get = |n: &str| p.iter().find(|(k, _)| k == n).map(|(_, v)| v.as_str());
    let is_true = |n: &str| get(n).is_some_and(|v| v.eq_ignore_ascii_case("true"));
    let algorithm = match get("algorithm").map(str::to_ascii_lowercase).as_deref() {
        None | Some("md5") => Alg::Md5,
        Some("md5-sess") => Alg::Md5Sess,
        Some("sha-256") => Alg::Sha256,
        Some("sha-256-sess") => Alg::Sha256Sess,
        _ => return Err("unsupported-algorithm".into()),
    };
    let qop_auth = match get("qop") {
        None => false,
        Some(q) if q.split(',').any(|t| t.trim().eq_ignore_ascii_case("auth")) => true,
        Some(_) => return Err("unsupported-qop".into()),
    };
    Ok(Challenge {
        realm: get("realm").ok_or("missing-realm")?.to_string(),
        nonce: get("nonce").ok_or("missing-nonce")?.to_string(),
        opaque: get("opaque").map(str::to_string),
        algorithm,
        qop_auth,
        stale: is_true("stale"),
        userhash: is_true("userhash"),
    })
}

/// Picks the strongest supported `Digest` challenge (SHA-256 over MD5; the
/// first one on a tie). With no usable one, returns the first Digest error.
pub fn parse_challenge(www_authenticate: &str) -> Result<Challenge, String> {
    let mut best: Option<Challenge> = None;
    let mut err = None;
    for (scheme, p) in challenges(www_authenticate) {
        if scheme != "digest" {
            continue;
        }
        match digest_challenge(&p) {
            Ok(c) if best.as_ref().is_none_or(|b| c.algorithm.strength() > b.algorithm.strength()) => best = Some(c),
            Ok(_) => {}
            Err(e) => {
                err.get_or_insert(e);
            }
        }
    }
    best.ok_or_else(|| err.unwrap_or_else(|| "no-digest".into()))
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// `Authorization` header value. `nc` is the nonce count (1 for the first use
/// of a nonce), `cnonce` a fresh random string (see [`new_cnonce`]).
pub fn authorization(ch: &Challenge, user: &str, pass: &str, method: &str, uri: &str, nc: u32, cnonce: &str) -> String {
    let a = ch.algorithm;
    let mut ha1 = a.h(&format!("{user}:{}:{pass}", ch.realm));
    if matches!(a, Alg::Md5Sess | Alg::Sha256Sess) {
        ha1 = a.h(&format!("{ha1}:{}:{cnonce}", ch.nonce));
    }
    let ha2 = a.h(&format!("{method}:{uri}"));
    let nc = format!("{nc:08x}");
    let response = if ch.qop_auth { a.h(&format!("{ha1}:{}:{nc}:{cnonce}:auth:{ha2}", ch.nonce)) } else { a.h(&format!("{ha1}:{}:{ha2}", ch.nonce)) };
    let user_param = if ch.userhash {
        format!("username={}", quote(&a.h(&format!("{user}:{}", ch.realm))))
    } else if user.bytes().all(|c| (0x20..0x7f).contains(&c)) {
        format!("username={}", quote(user))
    } else {
        let enc: String = user
            .bytes()
            .map(|c| match c {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'!' | b'#' | b'$' | b'&' | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~' => {
                    (c as char).to_string()
                }
                _ => format!("%{c:02X}"),
            })
            .collect();
        format!("username*=UTF-8''{enc}")
    };
    let mut v = format!("Digest {user_param}, realm={}, uri={}, algorithm={}, nonce={}", quote(&ch.realm), quote(uri), a.name(), quote(&ch.nonce));
    if ch.qop_auth {
        v += &format!(", nc={nc}, cnonce={}, qop=auth", quote(cnonce));
    }
    v += &format!(", response=\"{response}\"");
    if let Some(o) = &ch.opaque {
        v += &format!(", opaque={}", quote(o));
    }
    if ch.userhash {
        v += ", userhash=true";
    }
    v
}

/// 128 random bits from the OS, hex.
pub fn new_cnonce() -> Result<String, String> {
    let mut b = [0u8; 16];
    getrandom::fill(&mut b).map_err(|e| e.to_string())?;
    Ok(b.iter().map(|b| format!("{b:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONCE: &str = "7ypf/xlj9XXwfDPEoM4URrv/xwf94BcCAzFZH4GiTo0v";
    const OPAQUE: &str = "FQhe/qaU925kfnzjCev0ciny7QMkPqMAFRtzCUYo5tdS";
    const CNONCE: &str = "f2/wE4q74E6zIJEtWaHKaf5wv/H5QzzpXusqGemxURZJ";

    fn rfc(alg: &str) -> String {
        format!("Digest realm=\"http-auth@example.org\", qop=\"auth, auth-int\", algorithm={alg}, nonce=\"{NONCE}\", opaque=\"{OPAQUE}\"")
    }

    fn response(h: &str) -> &str {
        h.split("response=\"").nth(1).unwrap().split('"').next().unwrap()
    }

    #[test]
    fn rfc7616_example_md5_and_sha256() {
        for (alg, want) in
            [("MD5", "8ca523f5e9506fed4657c9700eebdbec"), ("SHA-256", "753927fa0e85d155564e2e272a28d1802ca10daf4496794697cf8db5856cb6c1")]
        {
            let ch = parse_challenge(&rfc(alg)).unwrap();
            assert!(ch.qop_auth && !ch.stale && !ch.userhash);
            assert_eq!(ch.opaque.as_deref(), Some(OPAQUE));
            let h = authorization(&ch, "Mufasa", "Circle of Life", "GET", "/dir/index.html", 1, CNONCE);
            assert_eq!(response(&h), want, "{alg}");
            assert!(h.starts_with("Digest username=\"Mufasa\", realm=\"http-auth@example.org\", uri=\"/dir/index.html\""));
            assert!(h.contains(&format!("algorithm={alg}, nonce=\"{NONCE}\", nc=00000001, cnonce=\"{CNONCE}\", qop=auth")));
            assert!(h.ends_with(&format!("opaque=\"{OPAQUE}\"")));
        }
    }

    #[test]
    fn quoted_commas_escapes_and_stale() {
        let ch = parse_challenge(r#"Digest realm="a, \"b\", c", nonce="n,1", STALE=TRUE, qop="auth""#).unwrap();
        assert_eq!(ch.realm, r#"a, "b", c"#);
        assert_eq!(ch.nonce, "n,1");
        assert!(ch.stale && ch.qop_auth);
        assert_eq!(ch.algorithm, Alg::Md5);
        let h = authorization(&ch, "u", "p", "GET", "/", 2, "c");
        assert!(h.contains(r#"realm="a, \"b\", c""#) && h.contains("nc=00000002"));
    }

    #[test]
    fn multi_challenge_picks_sha256() {
        let h = r#"Basic realm=x, Negotiate YII==, Digest realm="r", nonce="n1", algorithm=SHA-256, qop="auth", Digest realm="r", nonce="n2", algorithm=MD5, qop="auth""#;
        let ch = parse_challenge(h).unwrap();
        assert_eq!((ch.algorithm, ch.nonce.as_str()), (Alg::Sha256, "n1"));
        // order does not matter
        let h = r#"Digest realm="r", nonce="n2", algorithm=MD5, Basic realm="x,y", Digest realm="r", nonce="n1", algorithm=SHA-256-sess"#;
        let ch = parse_challenge(h).unwrap();
        assert_eq!((ch.algorithm, ch.nonce.as_str(), ch.qop_auth), (Alg::Sha256Sess, "n1", false));
    }

    #[test]
    fn refusals() {
        assert_eq!(parse_challenge(r#"Digest realm="r", nonce="n", qop="auth-int""#).unwrap_err(), "unsupported-qop");
        assert_eq!(parse_challenge(r#"Digest realm="r", nonce="n", algorithm=SHA-512-256"#).unwrap_err(), "unsupported-algorithm");
        assert_eq!(parse_challenge("Basic realm=x").unwrap_err(), "no-digest");
        assert_eq!(parse_challenge(r#"Digest nonce="n""#).unwrap_err(), "missing-realm");
        // an unusable challenge next to a usable one is skipped
        let ch = parse_challenge(r#"Digest realm="r", nonce="a", algorithm=SHA-512-256, Digest realm="r", nonce="b""#).unwrap();
        assert_eq!(ch.nonce, "b");
    }

    #[test]
    fn rfc2069_userhash_and_username_star() {
        let ch = parse_challenge(r#"Digest realm="testrealm@host.com", nonce="dcd98b7102dd2f0e8b11d0f600bfb0c093""#).unwrap();
        let h = authorization(&ch, "Mufasa", "Circle Of Life", "GET", "/dir/index.html", 1, "x");
        let ha1 = Alg::Md5.h("Mufasa:testrealm@host.com:Circle Of Life");
        let ha2 = Alg::Md5.h("GET:/dir/index.html");
        assert_eq!(response(&h), Alg::Md5.h(&format!("{ha1}:dcd98b7102dd2f0e8b11d0f600bfb0c093:{ha2}")));
        assert!(!h.contains("qop") && !h.contains("cnonce"));

        let ch = parse_challenge(&(rfc("SHA-256") + ", userhash=true")).unwrap();
        let h = authorization(&ch, "Mufasa", "Circle of Life", "GET", "/dir/index.html", 1, CNONCE);
        let uh = Alg::Sha256.h("Mufasa:http-auth@example.org");
        assert!(h.starts_with(&format!("Digest username=\"{uh}\"")) && h.ends_with("userhash=true"));
        assert!(!h.contains("Mufasa"));

        let ch = parse_challenge(r#"Digest realm="r", nonce="n", qop=auth"#).unwrap();
        let h = authorization(&ch, "J\u{e4}s\u{f8}n Doe", "p", "GET", "/", 1, "c");
        assert!(h.starts_with("Digest username*=UTF-8''J%C3%A4s%C3%B8n%20Doe, "));
        assert_eq!(new_cnonce().unwrap().len(), 32);
    }
}
