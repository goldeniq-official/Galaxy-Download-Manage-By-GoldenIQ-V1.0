// SPDX-License-Identifier: RPL-1.5
//! Opt-in media components (ADR-4, M-05, D10): yt-dlp and ffmpeg/ffprobe are not in the installer. On the
//! user's consent GDM reads `components.json` (minisign-signed with the release key compiled into gdm.exe,
//! with an expiry), downloads each upstream file over HTTPS, and accepts it only when its sha256 matches the
//! signed manifest. Before every run the executable is opened read-only with writers and deleters denied, its
//! hash is checked through that handle, and the handle is held until the process has started. GDM never runs
//! `yt-dlp -U`. Malware already running as the same user is outside what this can stop (§10).
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const MANIFEST_URL: &str =
    "https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0/releases/latest/download/components.json";
pub const MAX_DOWNLOAD: u64 = 300 * 1024 * 1024;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ComponentFile {
    /// name on disk in the components folder: yt-dlp.exe | ffmpeg.exe | ffprobe.exe
    pub name: String,
    /// path inside the archive (ffmpeg zip), absent for a plain .exe download
    #[serde(default)]
    pub path: Option<String>,
    pub sha256: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Component {
    pub name: String,
    pub version: String,
    pub url: String,
    pub sha256: String,
    pub size: u64,
    pub files: Vec<ComponentFile>,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Manifest {
    pub v: u32,
    pub expires: i64,
    pub components: Vec<Component>,
}

const ALLOWED: &[&str] = &["yt-dlp.exe", "ffmpeg.exe", "ffprobe.exe"];

/// U-01-style checks: signature with the compiled key, not expired, only known file names, https upstream
pub fn verify(body: &[u8], sig: &str, pubkey_b64: &str, now: i64) -> Result<Manifest, String> {
    let pk = crate::update::public_key(pubkey_b64).ok_or("key")?;
    // the .sig as Tauri writes it (base64 of the minisign text) or the minisign text itself
    let sig_txt = String::from_utf8(crate::update::b64(sig.trim()).unwrap_or_else(|| sig.as_bytes().to_vec())).map_err(|_| "signature")?;
    let s = minisign_verify::Signature::decode(&sig_txt).map_err(|_| "signature")?;
    pk.verify(body, &s, false).map_err(|_| "signature")?;
    let m: Manifest = serde_json::from_slice(body).map_err(|_| "bad-manifest")?;
    if m.v != 1 || m.expires <= now {
        return Err("expired".into());
    }
    for c in &m.components {
        if !c.url.starts_with("https://") || c.size > MAX_DOWNLOAD || c.sha256.len() != 64 {
            return Err("bad-manifest".into());
        }
        if c.files.iter().any(|f| !ALLOWED.contains(&f.name.as_str()) || f.sha256.len() != 64) {
            return Err("bad-manifest".into());
        }
    }
    Ok(m)
}

/// the folder GDM owns for components (%LOCALAPPDATA%\GoldenIQ.GalaxyDownloadManager\components)
pub fn dir(data: &Path) -> PathBuf {
    data.join("components")
}

/// the installed component list (name + sha256 of each file) kept next to the files
pub fn installed(data: &Path) -> Vec<ComponentFile> {
    std::fs::read(dir(data).join("installed.json")).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

pub fn has(data: &Path, name: &str) -> bool {
    installed(data).iter().any(|f| f.name == name) && dir(data).join(name).is_file()
}

/// put downloaded bytes in place: a plain exe, or files taken out of a zip with Windows' own tar.exe; every
/// file must match the signed hash before it is recorded as installed
pub fn install(data: &Path, c: &Component, bytes: &[u8]) -> Result<(), String> {
    use sha2::Digest;
    if gdm_core::finalize::hex(&sha2::Sha256::digest(bytes)) != c.sha256.to_ascii_lowercase() {
        return Err("hash-mismatch".into());
    }
    let d = dir(data);
    std::fs::create_dir_all(&d).map_err(|e| e.to_string())?;
    let stage = d.join(format!("stage-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&stage);
    std::fs::create_dir_all(&stage).map_err(|e| e.to_string())?;
    let r = (|| -> Result<(), String> {
        if c.url.to_ascii_lowercase().ends_with(".zip") {
            let z = stage.join("pkg.zip");
            std::fs::write(&z, bytes).map_err(|e| e.to_string())?;
            let tar = std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows")).join(r"System32\tar.exe");
            for f in &c.files {
                let inner = f.path.as_deref().ok_or("bad-manifest")?;
                if inner.contains("..") || inner.starts_with(['/', '\\']) || inner.contains(':') {
                    return Err("bad-manifest".into());
                }
                let st = std::process::Command::new(&tar).arg("-xf").arg(&z).arg("-C").arg(&stage).arg(inner).status().map_err(|e| e.to_string())?;
                if !st.success() {
                    return Err(format!("unzip {}", f.name));
                }
                std::fs::rename(stage.join(inner), stage.join(&f.name)).map_err(|e| e.to_string())?;
            }
        } else {
            let f = c.files.first().ok_or("bad-manifest")?;
            std::fs::write(stage.join(&f.name), bytes).map_err(|e| e.to_string())?;
        }
        for f in &c.files {
            let got =
                gdm_core::finalize::sha_handle(&std::fs::File::open(stage.join(&f.name)).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            if got != f.sha256.to_ascii_lowercase() {
                return Err(format!("hash-mismatch {}", f.name));
            }
            let _ = std::fs::remove_file(d.join(&f.name));
            std::fs::rename(stage.join(&f.name), d.join(&f.name)).map_err(|e| e.to_string())?;
        }
        Ok(())
    })();
    let _ = std::fs::remove_dir_all(&stage);
    r?;
    let mut list: Vec<ComponentFile> = installed(data).into_iter().filter(|x| !c.files.iter().any(|f| f.name == x.name)).collect();
    list.extend(c.files.iter().cloned());
    std::fs::write(d.join("installed.json"), serde_json::to_vec(&list).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

/// M-05: open the component read-only, deny writers/deleters, check its recorded hash THROUGH this handle,
/// and return the handle: the caller keeps it until the process has started
pub fn hold(data: &Path, name: &str) -> Result<(std::fs::File, PathBuf), String> {
    let want = installed(data).into_iter().find(|f| f.name == name).ok_or("not-installed")?.sha256;
    let p = dir(data).join(name);
    let h = crate::update::hold(&p, &want.to_ascii_lowercase()).map_err(|e| {
        if e.contains("changed") || e.contains("hash") {
            "tampered".to_string()
        } else {
            e
        }
    })?;
    Ok((h, p))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake(data: &Path, name: &str, body: &[u8]) -> ComponentFile {
        use sha2::Digest;
        let sha = gdm_core::finalize::hex(&sha2::Sha256::digest(body));
        let c = Component {
            name: name.into(),
            version: "1".into(),
            url: "https://x/y.exe".into(),
            sha256: sha.clone(),
            size: body.len() as u64,
            files: vec![ComponentFile { name: name.into(), path: None, sha256: sha }],
        };
        install(data, &c, body).unwrap();
        c.files[0].clone()
    }

    /// M-05 card test: one byte changed => refused (and GDM offers to download it again)
    #[test]
    fn m05_tampered_component_refused() {
        let d = std::env::temp_dir().join(format!("gdm-comp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let exe = std::fs::read(std::env::current_exe().unwrap()).unwrap();
        fake(&d, "yt-dlp.exe", &exe[..4096]);
        assert!(has(&d, "yt-dlp.exe"));
        let (h, _) = hold(&d, "yt-dlp.exe").unwrap();
        // while held, nobody can replace or delete it
        assert!(std::fs::write(dir(&d).join("yt-dlp.exe"), b"x").is_err(), "held: no writer");
        assert!(std::fs::remove_file(dir(&d).join("yt-dlp.exe")).is_err(), "held: no delete");
        drop(h);
        let mut b = exe[..4096].to_vec();
        b[100] ^= 1;
        std::fs::write(dir(&d).join("yt-dlp.exe"), &b).unwrap();
        assert_eq!(hold(&d, "yt-dlp.exe").unwrap_err(), "tampered");
        let c = Component { name: "x".into(), version: "1".into(), url: "https://x/y.exe".into(), sha256: "0".repeat(64), size: 1, files: vec![] };
        assert_eq!(install(&d, &c, b"abc").unwrap_err(), "hash-mismatch");
        let _ = std::fs::remove_dir_all(&d);
    }

    /// the install path (council G7 final #5): a real minisign signature (throwaway test key; only its public
    /// key is in the repo) is accepted; a changed byte, an expired manifest and a signed manifest naming a file
    /// GDM does not know are refused
    #[test]
    fn m05_signed_manifest_accepted_forgery_expiry_and_unknown_names_refused() {
        const PUB: &str = include_str!("../tests/components/test.pub.b64");
        let body = include_bytes!("../tests/components/components.json");
        let sig = include_str!("../tests/components/components.json.sig");
        let m = verify(body, sig, PUB, 1_800_000_000).unwrap();
        assert_eq!(m.components[0].files[0].name, "yt-dlp.exe");
        let mut forged = body.to_vec();
        let i = forged.iter().position(|&b| b == b'f').unwrap();
        forged[i] = b'e';
        assert_eq!(verify(&forged, sig, PUB, 1_800_000_000).unwrap_err(), "signature");
        assert_eq!(verify(body, sig, PUB, 1_900_000_000).unwrap_err(), "expired");
        assert_eq!(verify(body, sig, crate::update::PUBKEY, 1_800_000_000).unwrap_err(), "signature", "another key");
        let bad = include_bytes!("../tests/components/bad-name.json");
        let bad_sig = include_str!("../tests/components/bad-name.json.sig");
        assert_eq!(verify(bad, bad_sig, PUB, 1_800_000_000).unwrap_err(), "bad-manifest", "evil.dll is not a component file");
    }

    /// ffmpeg arrives as a zip: only the listed inner paths are extracted (Windows tar.exe), each must match its
    /// signed hash, and an inner path that climbs out ("..", absolute, drive) is refused before extracting
    #[test]
    fn zip_install_extracts_only_listed_files_and_refuses_escaping_paths() {
        use sha2::Digest;
        let d = std::env::temp_dir().join(format!("gdm-zip-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let src = d.join("src").join("pkg").join("bin");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("ffmpeg.exe"), b"fake ffmpeg").unwrap();
        std::fs::write(src.join("ffprobe.exe"), b"fake ffprobe").unwrap();
        std::fs::write(src.join("extra.dll"), b"not listed").unwrap();
        let zip = d.join("pkg.zip");
        let tar = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join(r"System32\tar.exe");
        assert!(std::process::Command::new(&tar).arg("-a").arg("-cf").arg(&zip).arg("-C").arg(d.join("src")).arg("pkg").status().unwrap().success());
        let bytes = std::fs::read(&zip).unwrap();
        let h = |b: &[u8]| gdm_core::finalize::hex(&sha2::Sha256::digest(b));
        let file = |n: &str, path: &str, body: &[u8]| ComponentFile { name: n.into(), path: Some(path.into()), sha256: h(body) };
        let mut c = Component {
            name: "ffmpeg".into(),
            version: "t".into(),
            url: "https://x/ffmpeg.zip".into(),
            sha256: h(&bytes),
            size: bytes.len() as u64,
            files: vec![file("ffmpeg.exe", "pkg/bin/ffmpeg.exe", b"fake ffmpeg"), file("ffprobe.exe", "pkg/bin/ffprobe.exe", b"fake ffprobe")],
        };
        let data = d.join("data");
        install(&data, &c, &bytes).unwrap();
        assert!(has(&data, "ffmpeg.exe") && has(&data, "ffprobe.exe"));
        let names: Vec<String> = std::fs::read_dir(dir(&data)).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        assert!(!names.iter().any(|n| n.contains("extra")), "only listed files: {names:?}");
        for evil in ["../x.exe", "pkg/../../x.exe", r"C:\Windows\x.exe", "/etc/x"] {
            c.files[0].path = Some(evil.into());
            assert_eq!(install(&data, &c, &bytes).unwrap_err(), "bad-manifest", "{evil}");
        }
        c.files[0].path = Some("pkg/bin/ffmpeg.exe".into());
        c.files[0].sha256 = "0".repeat(64);
        assert!(install(&data, &c, &bytes).unwrap_err().starts_with("hash-mismatch"), "an extracted file must match its signed hash");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn manifest_needs_our_signature() {
        let body = br#"{"v":1,"expires":99999999999,"components":[]}"#;
        assert_eq!(verify(body, "untrusted comment: x\nRWQ=\n", crate::update::PUBKEY, 0).unwrap_err(), "signature");
    }
}
