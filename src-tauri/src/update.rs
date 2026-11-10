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
pub const MANIFEST_URL: &str = "https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0/releases/latest/download/latest.json";

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
    /// Tauri's per-platform installer url + minisign signature: covered by OUR signature, and the update Tauri
    /// fetches again must carry exactly these (council G6: otherwise a second response could relabel an older
    /// signed installer with the new version number)
    pub platforms: std::collections::BTreeMap<String, Platform>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Platform {
    pub url: String,
    pub signature: String,
}

impl Manifest {
    /// the update Tauri is about to download is the one this signed manifest describes
    /// Tauri picks "<os>-<arch>-<installer>" before "<os>-<arch>"; either entry of OUR signed manifest may match
    pub fn binds(&self, platform: &str, version: &str, url: &str, signature: &str) -> bool {
        self.version == version
            && [format!("{platform}-nsis"), platform.to_string()]
                .iter()
                .filter_map(|k| self.platforms.get(k))
                .any(|p| p.url == url && p.signature.trim() == signature.trim())
    }
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

pub fn b64(s: &str) -> Option<Vec<u8>> {
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
pub fn public_key(pubkey_b64: &str) -> Option<minisign_verify::PublicKey> {
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

/// U-02 / §07: a staged update installs by itself only in Automatic mode, after fewer than 2 failed installs,
/// when no job has run for 2 minutes (a running download, finalise or hash keeps it waiting)
pub fn may_install(state: &State, mode: &str, failures: u32, running: usize, idle_for: Option<std::time::Duration>) -> bool {
    *state == State::Ready && mode == "auto" && failures < 2 && running == 0 && idle_for.is_some_and(|d| d >= std::time::Duration::from_secs(120))
}

/// U-06 (verify-to-execute): write the verified installer into GDM's own updates folder, close it, reopen it
/// READ-only with share mode READ (nobody can write, delete or rename it while held), check its SHA-256 through
/// that handle against the bytes the minisign check accepted, and keep the handle until the installer runs.
pub fn stage_and_hold(dir: &std::path::Path, bytes: &[u8]) -> Result<(std::fs::File, std::path::PathBuf), String> {
    use std::io::Write;
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    if std::fs::symlink_metadata(dir).map_err(|e| e.to_string())?.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err("updates folder is a junction/link".into());
    }
    // leftovers of an earlier failed install (best effort; a held one cannot be deleted)
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        if e.file_name().to_string_lossy().starts_with("gdm-update-") {
            let _ = std::fs::remove_file(e.path());
        }
    }
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    let path = dir.join(format!("gdm-update-{nanos:x}-{}.exe", std::process::id()));
    {
        let mut f = std::fs::OpenOptions::new().write(true).create_new(true).share_mode(0).open(&path).map_err(|e| e.to_string())?;
        f.write_all(bytes).map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?;
    }
    let held = hold(&path, &gdm_core::finalize::hex(&sha256(bytes)))?;
    Ok((held, path))
}

fn sha256(b: &[u8]) -> Vec<u8> {
    use sha2::Digest;
    sha2::Sha256::digest(b).to_vec()
}

/// open read-only, deny writers and deleters, refuse a reparse point, and compare the hash read through this handle
pub fn hold(path: &std::path::Path, want_sha: &str) -> Result<std::fs::File, String> {
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    let f = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(1) // FILE_SHARE_READ only
        .custom_flags(0x0020_0000) // FILE_FLAG_OPEN_REPARSE_POINT: never follow a planted link
        .open(path)
        .map_err(|e| e.to_string())?;
    if f.metadata().map_err(|e| e.to_string())?.file_attributes() & 0x400 != 0 {
        return Err("installer is a reparse point".into());
    }
    let got = gdm_core::finalize::sha_handle(&f).map_err(|e| e.to_string())?;
    if !got.eq_ignore_ascii_case(want_sha) {
        return Err("installer changed after verification".into());
    }
    Ok(f)
}

/// run the held installer like Tauri does (NSIS passive, /UPDATE, restart GDM afterwards), then exit
pub fn launch_held(path: &std::path::Path) -> Result<(), String> {
    use windows::core::{HSTRING, PCWSTR};
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
    let r = unsafe {
        ShellExecuteW(None, &HSTRING::from("open"), &HSTRING::from(path.as_os_str()), &HSTRING::from("/P /UPDATE /R"), PCWSTR::null(), SW_SHOW)
    };
    if r.0 as isize > 32 {
        Ok(())
    } else {
        Err(format!("installer did not start ({})", r.0 as isize))
    }
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
    fn u02_installs_only_when_idle() {
        use std::time::Duration;
        let idle = Some(Duration::from_secs(121));
        assert!(may_install(&State::Ready, "auto", 0, 0, idle));
        assert!(!may_install(&State::Ready, "auto", 0, 1, idle), "a running job blocks the install");
        assert!(!may_install(&State::Ready, "auto", 0, 0, Some(Duration::from_secs(60))), "idle < 2 min");
        assert!(!may_install(&State::Ready, "auto", 0, 0, None));
        assert!(!may_install(&State::Ready, "notify", 0, 0, idle), "notify mode never installs by itself");
        assert!(!may_install(&State::Ready, "auto", 2, 0, idle), "after 2 failed installs the user decides");
        assert!(!may_install(&State::Downloading, "auto", 0, 0, idle));
    }
    #[test]
    fn u01_tauri_update_must_match_the_signed_manifest() {
        let m = verify_manifest(BODY, SIG, TEST_PUB, NOW, "stable", "0.9.0").unwrap();
        let p = &m.platforms["windows-x86_64"];
        assert!(m.binds("windows-x86_64", "0.9.1", &p.url, &p.signature));
        assert!(
            !m.binds("windows-x86_64", "0.9.1", "https://github.com/x/y/releases/download/v0.8.0/old.exe", &p.signature),
            "relabelled older installer"
        );
        assert!(!m.binds("windows-x86_64", "0.9.1", &p.url, "other-signature"));
        assert!(!m.binds("windows-x86_64", "0.9.2", &p.url, &p.signature));
        assert!(!m.binds("windows-aarch64", "0.9.1", &p.url, &p.signature));
    }
    #[test]
    fn u06_held_installer_cannot_be_swapped_and_tampering_is_seen() {
        let d = std::env::temp_dir().join(format!("gdm-u06-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let (held, path) = stage_and_hold(&d, b"MZ installer bytes").unwrap();
        assert!(std::fs::OpenOptions::new().write(true).open(&path).is_err(), "no writer while held");
        assert!(std::fs::remove_file(&path).is_err(), "no delete while held");
        assert!(std::fs::rename(&path, d.join("x.exe")).is_err(), "no rename while held");
        assert!(std::fs::read(&path).is_ok(), "readers (the installer itself) still allowed");
        drop(held);
        // a swap between the write and the hold is detected by the hash read through the held handle
        std::fs::write(&path, b"MZ evil").unwrap();
        assert_eq!(hold(&path, &gdm_core::finalize::hex(&sha256(b"MZ installer bytes"))).unwrap_err(), "installer changed after verification");
        // a junction as the updates folder is refused
        let j = std::env::temp_dir().join(format!("gdm-u06j-{}", std::process::id()));
        let ok = std::process::Command::new("cmd").args(["/c", "mklink", "/J"]).arg(&j).arg(&d).output().unwrap().status.success();
        assert!(ok, "mklink /J");
        assert_eq!(stage_and_hold(&j, b"MZ x").unwrap_err(), "updates folder is a junction/link");
        let _ = std::fs::remove_dir(&j);
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test]
    fn u06_a_held_installer_still_starts() {
        // the Windows loader opens the image with share READ, compatible with our hold: a real exe must run
        let d = std::env::temp_dir().join(format!("gdm-u06r-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let exe = std::fs::read(r"C:\Windows\System32\whoami.exe").unwrap();
        let (held, path) = stage_and_hold(&d, &exe).unwrap();
        let out = std::process::Command::new(&path).output().expect("held installer starts");
        assert!(out.status.success() && !out.stdout.is_empty());
        drop(held);
        let _ = std::fs::remove_dir_all(&d);
    }
    /// U-03 restore drill: a manifest signed with a key RESTORED from backup verifies against the key compiled
    /// into the shipped app. Run with GDM_DRILL_BODY=<file> GDM_DRILL_SIG=<file.sig> (skipped otherwise).
    #[test]
    fn u03_restore_drill() {
        let (Ok(b), Ok(g)) = (std::env::var("GDM_DRILL_BODY"), std::env::var("GDM_DRILL_SIG")) else { return };
        let body = std::fs::read(b).unwrap();
        let sig = std::fs::read_to_string(g).unwrap();
        let m =
            verify_manifest(&body, &sig, PUBKEY, gdm_core::store::now(), "stable", "0.9.0").expect("restored key verifies against the shipped key");
        println!("u03 drill: manifest {} verified with the production public key", m.version);
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
