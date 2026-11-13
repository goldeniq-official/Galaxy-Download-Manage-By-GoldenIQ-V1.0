// SPDX-License-Identifier: RPL-1.5
//! Finalisation (§09 step 8, N-05, N-09). Order (council G5): the .part handle is closed (the antivirus gets its
//! on-close scan) -> IAttachmentExecute Save with the FINAL name (MOTW + AV judge the real type; Save may alter
//! the file) -> reopen through the held directory with read + DELETE access, share READ only (the antivirus
//! on-open gate; nobody can write, delete or rename it from now on) -> same file object -> byte count -> SHA-256
//! through that handle -> rename through the SAME handle, never replacing. The bytes that get the final name are
//! exactly the bytes that were hashed, AFTER every party that may change them had its chance.
//! Measured cost (G5): this read of a formerly sparse file comes from disk (~170 MB/s on the D: HDD, ~6 s/GiB),
//! while the writing handle would read it from cache in 0.7 s; the faster order was rejected by council because
//! Save may alter the file. Out of scope (§10 A4): a same-user process holding a writable mapping across the
//! closed-handle window.
use crate::fsx::{self, Dir, FsError};
use crate::{failpoint, Fail};
use sha2::{Digest, Sha256};
use std::fs::File;
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub enum Protection {
    /// NTFS/ReFS and Zone.Identifier present
    Protected,
    /// FAT/exFAT/share without ADS: published, but no direct open (N-09 "គោលដៅមិនការពារ")
    Unprotected(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Finished {
    pub name: String,
    pub sha256: String,
    pub protection: Protection,
    pub zone: Option<String>,
}

/// HostUrl / ReferrerUrl without userinfo, query or fragment (secrets live there)
pub fn clean_url(u: &str) -> String {
    match Url::parse(u) {
        Ok(mut p) => {
            let _ = p.set_username("");
            let _ = p.set_password(None);
            p.set_query(None);
            p.set_fragment(None);
            p.to_string()
        }
        Err(_) => String::new(),
    }
}

pub fn sha_handle(f: &File) -> std::io::Result<String> {
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 4 << 20];
    let mut off = 0u64;
    loop {
        let n = fsx::read_at(f, &mut buf, off)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
        off += n as u64;
    }
    Ok(hex(&h.finalize()))
}

/// Zone.Identifier says Internet (3) or Restricted (4): only then SmartScreen / Protected View act
pub fn zone_internet(z: Option<&str>) -> bool {
    z.is_some_and(|z| z.lines().any(|l| matches!(l.trim(), "ZoneId=3" | "ZoneId=4")))
}

pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// The .part vanished: AV (Bitdefender deletes silently, G3) or something else. Differential evidence: a
/// benign file written the same way in the same folder survives => "Blocked" (suspected security software).
fn vanished(dir: &Dir, part: &str) -> Fail {
    let probe = format!("{part}.probe");
    let Ok(f) = dir.create_new(&probe) else { return Fail::Io("file removed; no evidence of the remover".into()) };
    let survived = {
        use std::io::Write;
        let ok = (&f).write_all(&vec![b'G'; 1 << 20]).is_ok();
        drop(f);
        std::thread::sleep(std::time::Duration::from_millis(1500));
        ok && dir.exists(&probe)
    };
    let _ = dir.delete(&probe); // only the probe this call created
    if survived {
        Fail::Blocked("suspected security software: file removed while a benign probe survived".into())
    } else {
        Fail::Io("file and a benign probe both removed: storage or cleanup problem".into())
    }
}

/// `part`: leaf of the .part; `name`: wanted final leaf; `id`: file identity recorded at creation
#[allow(clippy::too_many_arguments)]
pub fn finalize(
    dir: &Dir,
    part: &str,
    name: &str,
    total: u64,
    expected: Option<&str>,
    source: &str,
    referrer: Option<&str>,
    id: Option<(u32, u64)>,
    replaces: Option<&str>,
    intent: &mut dyn FnMut(&str, &str) -> Result<(), Fail>,
) -> Result<Finished, Fail> {
    if !dir.exists(part) {
        return Err(vanished(dir, part));
    }
    let saved = fsx::attachment_save(dir, part, name, &clean_url(source), referrer.map(clean_url).as_deref());
    failpoint::hit("after_save");
    let f = match dir.open_locked(part) {
        Ok(f) => f,
        Err(FsError::NotFound) => return Err(Fail::Blocked(format!("removed during the antivirus scan (Save {saved:?})"))),
        Err(e) => return Err(Fail::Io(format!("reopen: {e}"))),
    };
    if let Some(want) = id {
        if fsx::file_id(&f).map_err(|e| Fail::Io(e.to_string()))? != want {
            return Err(Fail::Io("file object changed before finalisation".into()));
        }
    }
    let len = f.metadata().map_err(|e| Fail::Io(e.to_string()))?.len();
    if len != total {
        return Err(Fail::Io(format!("byte count {len} != total {total}")));
    }
    let fs = fsx::fs_name(&f);
    // a failing Save with the file still present is an operational error, never proof of malware (Windows:
    // only where MOTW can exist; FAT/exFAT have no streams. macOS/Linux: marking failed on a normal volume)
    let must_mark = if cfg!(windows) {
        matches!(fs.as_str(), "NTFS" | "ReFS")
    } else {
        !matches!(fs.as_str(), "vfat" | "msdos" | "exfat" | "smb" | "smbfs" | "nfs" | "fuse" | "other" | "")
    };
    if let Err(e) = &saved {
        if must_mark {
            return Err(Fail::Io(e.clone()));
        }
    }
    let t = std::time::Instant::now();
    let sha = sha_handle(&f).map_err(|e| Fail::Io(e.to_string()))?;
    failpoint::phase("sha256", t);
    if expected.is_some_and(|e| !e.eq_ignore_ascii_case(&sha)) {
        return Err(Fail::HashMismatch);
    }
    let zone = fsx::zone_identifier(dir, part);
    failpoint::hit("before_rename");
    let mut fin = name.to_string();
    let mut n = 0;
    loop {
        // persisted BEFORE the rename: a crash right after it is reconciled against this target + hash
        intent(&fin, &sha)?;
        // sync queue (Q-05): replace the old copy only when the file under the name is still the very file
        // object this job published (file id); anything else that took the name keeps it (counter name)
        let own = n == 0
            && replaces.is_some_and(|want| {
                dir.open_locked(&fin).ok().and_then(|o| fsx::file_id(&o).ok()).map(|i| format!("{:08x}:{:016x}", i.0, i.1)).as_deref() == Some(want)
            });
        match fsx::rename(&f, dir, part, &fin, own) {
            Ok(()) => {
                failpoint::hit("after_rename");
                break;
            }
            Err(FsError::Exists) if n < 100 => {
                n += 1;
                fin = crate::name::with_counter(name, n);
            } // N-04: someone took the name meanwhile
            Err(e) => return Err(Fail::Io(format!("rename: {e}"))),
        }
    }
    let protection = match fsx::os_protected(&f, zone.as_deref()) {
        Ok(()) => Protection::Protected,
        Err(why) => Protection::Unprotected(why),
    };
    Ok(Finished { name: fin, sha256: sha, protection, zone })
}
