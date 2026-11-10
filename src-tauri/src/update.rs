// SPDX-License-Identifier: RPL-1.5
//! §07 update state machine for the GitHub build (the Store build compiles this out: feature `store`).
//! U-01: before Tauri's updater runs, GDM checks its own signed manifest: minisign signature with the key
//! compiled into this binary, version > the highest version ever seen on this channel (no downgrade),
//! not expired, right channel. Tauri then verifies the installer's own minisign signature.
//! U-02: install only when idle (no running job for 2 minutes), on "Restart now", or on quit.
use serde::{Deserialize, Serialize};

/// minisign public key (Tauri format: base64 of the .pub file). The private key is the owner's (offline +
/// GitHub Environment "release"); see docs/evidence/G6.md "Updater key".
pub const PUBKEY: &str = include_str!("../updater.pub.b64");
pub const MANIFEST_URL: &str = "https://github.com/goldeniq-official/galaxy-download-manager/releases/latest/download/latest.json";

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum State {
    #[default]
    Idle,
    Checking,
    Available,
    Downloading,
    Verifying,
    Ready,
    Installing,
    Failed,
    Store,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Status {
    pub state: State,
    pub version: Option<String>,
    pub security: bool,
    pub notes_km: Option<String>,
    pub notes_en: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Manifest {
    pub version: String,
    pub channel: String,
    pub expires: i64,
    #[serde(default)]
    pub security: bool,
    #[serde(default)]
    pub notes_km: Option<String>,
    #[serde(default)]
    pub notes_en: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Reject {
    Signature,
    Format,
    Expired,
    Channel,
    NotNewer,
}

/// "1.2.3" or "1.2.3-beta.4" -> comparable key; a pre-release sorts below its release
pub fn vkey(v: &str) -> Option<(u64, u64, u64, u64)> {
    let v = v.trim().trim_start_matches('v');
    let (core, pre) = v.split_once('-').map_or((v, None), |(a, b)| (a, Some(b)));
    let mut it = core.split('.').map(|x| x.parse::<u64>().ok());
    let (a, b, c) = (it.next()??, it.next()??, it.next()??);
    if it.next().is_some() {
        return None;
    }
    let p = match pre {
        None => u64::MAX,
        Some(p) => p.rsplit('.').next()?.parse::<u64>().unwrap_or(0),
    };
    Some((a, b, c, p))
}

fn b64(s: &str) -> Option<Vec<u8>> {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::new();
    let (mut acc, mut bits) = (0u32, 0);
    for c in s.trim().bytes().filter(|c| !c.is_ascii_whitespace()) {
        if c == b'=' {
            break;
        }
        acc = (acc << 6) | T.iter().position(|&t| t == c)? as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Some(out)
}

/// key text (Tauri: base64 of "untrusted comment: ...\nRW...") -> minisign public key
fn public_key(pubkey_b64: &str) -> Option<minisign_verify::PublicKey> {
    let txt = String::from_utf8(b64(pubkey_b64)?).ok()?;
    minisign_verify::PublicKey::from_base64(txt.lines().nth(1)?.trim()).ok()
}

pub fn verify_manifest(body: &[u8], sig_b64: &str, pubkey_b64: &str, now: i64, channel: &str, highest_seen: &str) -> Result<Manifest, Reject> {
    let pk = public_key(pubkey_b64).ok_or(Reject::Signature)?;
    let sig_txt = String::from_utf8(b64(sig_b64).ok_or(Reject::Signature)?).map_err(|_| Reject::Signature)?;
    let sig = minisign_verify::Signature::decode(&sig_txt).map_err(|_| Reject::Signature)?;
    pk.verify(body, &sig, false).map_err(|_| Reject::Signature)?;
    let m: Manifest = serde_json::from_slice(body).map_err(|_| Reject::Format)?;
    if m.expires <= now {
        return Err(Reject::Expired);
    }
    // stable users take stable only; beta users take both
    if !(m.channel == channel || (channel == "beta" && m.channel == "stable")) {
        return Err(Reject::Channel);
    }
    let (new, seen, cur) = (vkey(&m.version).ok_or(Reject::Format)?, vkey(highest_seen), vkey(env!("CARGO_PKG_VERSION")));
    if seen.is_some_and(|s| new <= s) || cur.is_some_and(|c| new <= c) {
        return Err(Reject::NotNewer);
    }
    Ok(m)
}

#[cfg(test)]
mod tests {
    use super::*;
    // a throwaway test key pair (generated for these tests only; never used for releases)
    const TEST_PUB: &str = include_str!("../tests/update/test.pub.b64");
    const BODY: &[u8] = include_bytes!("../tests/update/latest.json");
    const SIG: &str = include_str!("../tests/update/latest.json.sig.b64");
    const NOW: i64 = 1_790_000_000; // 2026-09; the test manifest expires 2030

    #[test]
    fn u01_good_manifest_accepted() {
        let m = verify_manifest(BODY, SIG, TEST_PUB, NOW, "stable", "0.9.0").unwrap();
        assert_eq!(m.version, "0.9.1");
    }
    #[test]
    fn u01_forged_body_wrong_key_old_version_expired_channel_refused() {
        let forged = String::from_utf8_lossy(BODY).replace("0.9.1", "9.9.9");
        assert_eq!(verify_manifest(forged.as_bytes(), SIG, TEST_PUB, NOW, "stable", "0.9.0"), Err(Reject::Signature), "forged manifest");
        assert_eq!(verify_manifest(BODY, SIG, PUBKEY, NOW, "stable", "0.9.0"), Err(Reject::Signature), "signed with another key");
        assert_eq!(verify_manifest(BODY, "AAAA", TEST_PUB, NOW, "stable", "0.9.0"), Err(Reject::Signature), "garbage signature");
        assert_eq!(verify_manifest(BODY, SIG, TEST_PUB, NOW, "stable", "0.9.1"), Err(Reject::NotNewer), "same version (replay)");
        assert_eq!(verify_manifest(BODY, SIG, TEST_PUB, NOW, "stable", "1.0.0"), Err(Reject::NotNewer), "downgrade below highest seen");
        assert_eq!(verify_manifest(BODY, SIG, TEST_PUB, 2_000_000_000, "stable", "0.9.0"), Err(Reject::Expired));
        assert!(verify_manifest(BODY, SIG, TEST_PUB, NOW, "beta", "0.9.0").is_ok(), "beta takes stable");
    }
    #[test]
    fn versions_order() {
        assert!(vkey("0.9.1") > vkey("0.9.0"));
        assert!(vkey("1.0.0") > vkey("1.0.0-beta.3"));
        assert!(vkey("1.0.0-beta.3") > vkey("1.0.0-beta.2"));
        assert!(vkey("v1.10.0") > vkey("1.9.9"));
        assert_eq!(vkey("1.2"), None);
        assert_eq!(vkey("1.2.3.4"), None);
    }
}
