// SPDX-License-Identifier: RPL-1.5
//! Site logins (F16, P-01; IDM "Site Logins", per site or folder). The password lives only in the system's
//! secret store under `GDM/site/<id>`: Windows Credential Manager (DPAPI underneath), the macOS Keychain, or the
//! Linux Secret Service (GNOME Keyring / KWallet); with no store available the login is not saved, never written
//! in plaintext. settings.json keeps the id, the address prefix and the user name. Exports never include
//! passwords. None of these stores protects against malware running as the same user (§10).
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use windows::core::{HSTRING, PWSTR};
#[cfg(windows)]
use windows::Win32::Security::Credentials::{
    CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_FLAGS, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct SiteLogin {
    /// "l-<n>"
    pub id: String,
    /// https://host[:port]/folder/ : applies to that origin and everything under that folder
    pub prefix: String,
    pub user: String,
}

fn target(id: &str) -> String {
    format!("GDM/site/{id}")
}

pub fn valid_id(id: &str) -> bool {
    id.strip_prefix("l-").is_some_and(|r| (1..=16).contains(&r.len()) && r.bytes().all(|b| b.is_ascii_alphanumeric()))
}

/// normalised prefix: http(s), no userinfo/query/fragment, path ending in '/'
pub fn norm_prefix(p: &str) -> Option<String> {
    let mut u = url::Url::parse(p.trim()).ok()?;
    if !matches!(u.scheme(), "http" | "https") || !u.username().is_empty() || u.password().is_some() || u.host_str().is_none() {
        return None;
    }
    u.set_query(None);
    u.set_fragment(None);
    let mut path = u.path().to_string();
    if !path.ends_with('/') {
        path = path.rsplit_once('/').map(|(a, _)| format!("{a}/")).unwrap_or("/".into());
    }
    u.set_path(&path);
    Some(u.to_string())
}

/// the most specific login whose origin is the URL's and whose folder contains it (boundary-aware:
/// `/private/` does not match `/private-other/`)
pub fn matching<'a>(logins: &'a [SiteLogin], url: &str) -> Option<&'a SiteLogin> {
    let u = url::Url::parse(url).ok()?;
    logins
        .iter()
        .filter(|l| url::Url::parse(&l.prefix).is_ok_and(|p| p.origin() == u.origin() && u.path().starts_with(p.path())))
        .max_by_key(|l| l.prefix.len())
}

#[cfg(unix)]
fn entry(id: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new("Galaxy Download Manager", &target(id)).map_err(|e| e.to_string())
}

/// the secret is [user, password] as JSON (one item per login, like the Windows credential)
#[cfg(unix)]
pub fn save(id: &str, user: &str, password: &str) -> Result<(), String> {
    let v = serde_json::to_string(&[user, password]).map_err(|e| e.to_string())?;
    entry(id)?.set_password(&v).map_err(|e| e.to_string())
}

#[cfg(unix)]
pub fn load(id: &str) -> Option<(String, String)> {
    let v: [String; 2] = serde_json::from_str(&entry(id).ok()?.get_password().ok()?).ok()?;
    let [u, p] = v;
    Some((u, p))
}

#[cfg(unix)]
pub fn delete(id: &str) {
    if let Ok(e) = entry(id) {
        let _ = e.delete_credential();
    }
}

#[cfg(windows)]
pub fn save(id: &str, user: &str, password: &str) -> Result<(), String> {
    let t = HSTRING::from(target(id));
    let u = HSTRING::from(user);
    let blob: Vec<u8> = password.encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
    let c = CREDENTIALW {
        Flags: CRED_FLAGS(0),
        Type: CRED_TYPE_GENERIC,
        TargetName: PWSTR(t.as_ptr() as *mut u16),
        CredentialBlobSize: blob.len() as u32,
        CredentialBlob: blob.as_ptr() as *mut u8,
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        UserName: PWSTR(u.as_ptr() as *mut u16),
        ..Default::default()
    };
    unsafe { CredWriteW(&c, 0) }.map_err(|e| e.message())
}

/// (user, password) from Credential Manager
#[cfg(windows)]
pub fn load(id: &str) -> Option<(String, String)> {
    unsafe {
        let mut p: *mut CREDENTIALW = std::ptr::null_mut();
        CredReadW(&HSTRING::from(target(id)), CRED_TYPE_GENERIC, None, &mut p).ok()?;
        let c = &*p;
        let blob = std::slice::from_raw_parts(c.CredentialBlob, c.CredentialBlobSize as usize);
        let pw: Vec<u16> = blob.chunks_exact(2).map(|b| u16::from_le_bytes([b[0], b[1]])).collect();
        let out = (c.UserName.to_string().unwrap_or_default(), String::from_utf16_lossy(&pw));
        CredFree(p as *const _);
        Some(out)
    }
}

#[cfg(windows)]
pub fn delete(id: &str) {
    unsafe {
        let _ = CredDeleteW(&HSTRING::from(target(id)), CRED_TYPE_GENERIC, None);
    }
}

/// `Authorization: Basic` value (preemptive, for the matching folder only)
pub fn basic(user: &str, pass: &str) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let raw = format!("{user}:{pass}").into_bytes();
    let mut s = String::with_capacity(raw.len() * 4 / 3 + 4);
    for c in raw.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            s.push(if i <= c.len() { T[(n >> (18 - 6 * i) & 63) as usize] as char } else { '=' });
        }
    }
    format!("Basic {s}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_and_match_are_boundary_aware() {
        assert_eq!(norm_prefix("https://e.com/private/file.zip?x=1#f").as_deref(), Some("https://e.com/private/"));
        assert!(norm_prefix("ftp://e.com/").is_none() && norm_prefix("https://u:p@e.com/").is_none());
        let l = vec![
            SiteLogin { id: "l-1".into(), prefix: "https://e.com/".into(), user: "a".into() },
            SiteLogin { id: "l-2".into(), prefix: "https://e.com/private/".into(), user: "b".into() },
        ];
        assert_eq!(matching(&l, "https://e.com/private/x.zip").unwrap().id, "l-2", "most specific folder");
        assert_eq!(matching(&l, "https://e.com/private-other/x.zip").unwrap().id, "l-1", "/private/ is not /private-other/");
        assert!(matching(&l, "http://e.com/private/x.zip").is_none(), "other scheme = other origin");
        assert!(matching(&l, "https://e.com.evil.net/private/x.zip").is_none());
        assert_eq!(basic("Aladdin", "open sesame"), "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==", "RFC 7617 example");
    }

    /// P-01: the password is in Credential Manager only; the round trip works and delete removes it (a CI Linux
    /// runner has no Secret Service session, so the Unix store is checked on real machines)
    #[cfg(windows)]
    #[test]
    fn p01_credential_manager_round_trip() {
        let id = format!("l-t{}", std::process::id());
        save(&id, "user@example", "GDM-test-pw-7c1f").unwrap();
        assert_eq!(load(&id), Some(("user@example".to_string(), "GDM-test-pw-7c1f".to_string())));
        delete(&id);
        assert!(load(&id).is_none());
    }
}
