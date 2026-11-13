// SPDX-License-Identifier: RPL-1.5
//! macOS / Linux filesystem primitives with the contract of `fsx_windows.rs` (N-03, N-04, N-05, N-09).
//! - The destination directory is walked component by component with O_DIRECTORY|O_NOFOLLOW and held as a
//!   descriptor; a path that resolves elsewhere (a symlink anywhere on the way) is refused, like a junction on
//!   Windows. Unix cannot stop a held directory from being renamed, but every operation stays relative to the
//!   held descriptor, so a write can never land in another directory.
//! - Other users (in scope, §10) must not be able to change names on the way (council final review: rename,
//!   replace and hard-link races all need that): like OpenSSH's StrictModes, every component must be owned by
//!   root or by us and not writable by others unless sticky, and the destination itself must be ours and not
//!   writable by group or others. A folder that fails this is refused (fail closed), e.g. an exFAT stick
//!   mounted 0777.
//! - Files: openat(O_NOFOLLOW|O_NONBLOCK) relative to that descriptor, regular files only (a symlink, FIFO or
//!   device planted at the leaf is refused, never followed), one hard link only for files GDM writes into.
//! - "Busy" (Windows share modes): GDM's own handles take flock(LOCK_EX|LOCK_NB); flock belongs to the open file
//!   description, so a second open in the same process sees the first one, as FILE_SHARE_READ does on Windows.
//! - Rename: renameat2(RENAME_NOREPLACE) / renameatx_np(RENAME_EXCL), after checking that the leaf still names
//!   the very file object GDM holds (Unix renames a name, not a handle). Filesystems without no-replace rename
//!   fall back to link + unlink, which also never replaces.
//! - Mark of the web: macOS `com.apple.quarantine` (Gatekeeper checks the file before it first runs); Linux has
//!   no such gate: the origin goes into `user.xdg.origin.url` (freedesktop) and the file is never executable
//!   (0644; running it needs an explicit chmod). No antivirus is called on either system.
use super::FsError;
use rustix::fs::{self as rfs, AtFlags, FileType, FlockOperation, Mode, OFlags, RenameFlags, XattrFlags};
use rustix::io::Errno;
use std::fs::File;
use std::os::fd::OwnedFd;
use std::path::{Component, Path, PathBuf};

fn io(e: impl std::fmt::Display) -> FsError {
    FsError::Io(e.to_string())
}

fn err(e: Errno, what: &str) -> FsError {
    match e {
        Errno::EXIST => FsError::Exists,
        Errno::NOENT | Errno::NOTDIR => FsError::NotFound,
        Errno::LOOP | Errno::MLINK => FsError::Reparse(format!("{what} is a symbolic link")),
        Errno::WOULDBLOCK => FsError::Busy,
        e => FsError::Io(format!("{what}: {e}")),
    }
}

/// one component, never a path (the Windows side also refuses '\\' and ':')
fn leaf_ok(leaf: &str) -> Result<(), FsError> {
    if leaf.is_empty() || leaf == "." || leaf == ".." || leaf.contains(['/', '\\', '\0']) {
        return Err(FsError::Io(format!("not a single component: {leaf:?}")));
    }
    Ok(())
}

const DIR_FLAGS: OFlags = OFlags::RDONLY.union(OFlags::DIRECTORY).union(OFlags::NOFOLLOW).union(OFlags::CLOEXEC);

/// A held destination directory.
pub struct Dir {
    fd: OwnedFd,
    pub path: PathBuf,
}

impl Dir {
    /// Open and hold. Fails when the path does not resolve to itself (a symlink anywhere on the way: the same
    /// rule as a junction on Windows) or when a component is swapped for a symlink while it is walked.
    pub fn open(path: &Path) -> Result<Dir, FsError> {
        let abs = std::path::absolute(path).map_err(io)?;
        let canon = std::fs::canonicalize(&abs).map_err(|e| if e.kind() == std::io::ErrorKind::NotFound { FsError::NotFound } else { io(e) })?;
        if canon != abs {
            return Err(FsError::Reparse(format!("{} resolves to {}", abs.display(), canon.display())));
        }
        let mut fd = rfs::open("/", DIR_FLAGS, Mode::empty()).map_err(|e| err(e, "/"))?;
        let mut at = PathBuf::from("/");
        strict(&fd, &at, false)?;
        for c in canon.components() {
            if let Component::Normal(name) = c {
                fd = rfs::openat(&fd, name, DIR_FLAGS, Mode::empty()).map_err(|e| err(e, &canon.display().to_string()))?;
                at.push(name);
                strict(&fd, &at, at == canon)?;
            }
        }
        Ok(Dir { fd, path: canon })
    }

    /// a sub-directory relative to this held descriptor (created when missing), held the same way; a symlink
    /// there is refused, never followed (G-03 mirror folders)
    pub fn subdir(&self, leaf: &str) -> Result<Dir, FsError> {
        leaf_ok(leaf)?;
        match rfs::mkdirat(&self.fd, leaf, Mode::from_raw_mode(0o755)) {
            Ok(()) | Err(Errno::EXIST) => {}
            Err(e) => return Err(err(e, leaf)),
        }
        let fd = rfs::openat(&self.fd, leaf, DIR_FLAGS, Mode::empty()).map_err(|e| err(e, leaf))?;
        strict(&fd, &self.path.join(leaf), true)?;
        Ok(Dir { fd, path: self.path.join(leaf) })
    }

    /// a regular file relative to the held directory; `lock`: GDM's exclusive flock (the "Busy" contract)
    fn file(&self, leaf: &str, access: OFlags, create: bool, lock: bool) -> Result<File, FsError> {
        leaf_ok(leaf)?;
        let mut fl = access | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
        if create {
            fl |= OFlags::CREATE | OFlags::EXCL;
        }
        let fd = rfs::openat(&self.fd, leaf, fl, Mode::from_raw_mode(0o644)).map_err(|e| err(e, leaf))?;
        let st = rfs::fstat(&fd).map_err(|e| err(e, leaf))?;
        if FileType::from_raw_mode(st.st_mode) != FileType::RegularFile {
            return Err(FsError::Reparse(format!("{leaf} is not a regular file")));
        }
        if lock {
            rfs::flock(&fd, FlockOperation::NonBlockingLockExclusive).map_err(|e| err(e, leaf))?;
        }
        Ok(File::from(fd))
    }

    /// new file, never an existing one (O_CREAT|O_EXCL), read+write
    pub fn create_new(&self, leaf: &str) -> Result<File, FsError> {
        self.file(leaf, OFlags::RDWR, true, true)
    }
    /// existing file for resume (rejects symlinks and hard links)
    pub fn open_rw(&self, leaf: &str) -> Result<File, FsError> {
        single_link(self.file(leaf, OFlags::RDWR, false, true)?)
    }
    /// read access under GDM's exclusive lock: GDM's own writers are out (Unix has no mandatory share mode)
    pub fn open_locked(&self, leaf: &str) -> Result<File, FsError> {
        single_link(self.file(leaf, OFlags::RDONLY, false, true)?)
    }
    pub fn read_small(&self, leaf: &str, max: usize) -> Result<Vec<u8>, FsError> {
        let f = self.file(leaf, OFlags::RDONLY, false, false)?;
        let mut v = Vec::new();
        use std::io::Read;
        f.take(max as u64 + 1).read_to_end(&mut v).map_err(io)?;
        Ok(v)
    }
    pub fn exists(&self, leaf: &str) -> bool {
        leaf_ok(leaf).is_ok() && rfs::statat(&self.fd, leaf, AtFlags::SYMLINK_NOFOLLOW).is_ok()
    }

    /// write `data` to `leaf` atomically: `leaf.tmp` (created new, flushed) then rename over `leaf`
    pub fn write_atomic(&self, leaf: &str, data: &[u8]) -> Result<(), FsError> {
        let tmp = format!("{leaf}.tmp");
        let _ = self.delete(&tmp);
        let f = self.create_new(&tmp)?;
        use std::io::Write;
        (&f).write_all(data).map_err(io)?;
        f.sync_all().map_err(io)?;
        crate::failpoint::hit("sidecar_tmp");
        rename(&f, self, &tmp, leaf, true)?;
        let _ = rfs::fsync(&self.fd); // the new directory entry is durable too
        Ok(())
    }

    pub fn delete(&self, leaf: &str) -> Result<(), FsError> {
        leaf_ok(leaf)?;
        rfs::unlinkat(&self.fd, leaf, AtFlags::empty()).map_err(|e| err(e, leaf))
    }
}

/// StrictModes for one held directory: owned by root or us, never writable by others unless sticky; the
/// destination itself (`last`) must be ours and writable by nobody else
fn strict(fd: &OwnedFd, at: &Path, last: bool) -> Result<(), FsError> {
    let st = rfs::fstat(fd).map_err(io)?;
    let me = rustix::process::geteuid().as_raw();
    let (uid, mode) = (st.st_uid, st.st_mode as u32);
    let sticky = mode & 0o1000 != 0;
    let bad = if last {
        uid != me || mode & 0o022 != 0
    } else {
        (uid != 0 && uid != me) || (mode & 0o002 != 0 && !sticky) || (mode & 0o020 != 0 && uid != 0 && !sticky)
    };
    if bad {
        return Err(FsError::Reparse(format!("{} can be changed by other users (owner {uid}, mode {:o})", at.display(), mode & 0o7777)));
    }
    Ok(())
}

fn same(a: &rfs::Stat, b: &rfs::Stat) -> bool {
    a.st_dev == b.st_dev && a.st_ino == b.st_ino
}

/// rename the open file `f` (now named `from` in `dir`) to `leaf`; `replace=false` never overwrites. The name
/// must still be the very file object GDM holds, before and after (Unix renames names, not handles).
pub fn rename(f: &File, dir: &Dir, from: &str, leaf: &str, replace: bool) -> Result<(), FsError> {
    leaf_ok(from)?;
    leaf_ok(leaf)?;
    let me = rfs::fstat(f).map_err(io)?;
    if me.st_nlink != 1 {
        return Err(FsError::Reparse(format!("{from} has {} hard links", me.st_nlink)));
    }
    let named = rfs::statat(&dir.fd, from, AtFlags::SYMLINK_NOFOLLOW).map_err(|e| err(e, from))?;
    if !same(&me, &named) {
        return Err(FsError::Reparse(format!("{from} is no longer the file being renamed")));
    }
    let r = if replace {
        rfs::renameat(&dir.fd, from, &dir.fd, leaf)
    } else {
        match rfs::renameat_with(&dir.fd, from, &dir.fd, leaf, RenameFlags::NOREPLACE) {
            // no-replace rename unsupported by this filesystem: link (fails on an existing name) + unlink
            Err(Errno::INVAL | Errno::NOSYS | Errno::NOTSUP) => {
                rfs::linkat(&dir.fd, from, &dir.fd, leaf, AtFlags::empty()).and_then(|()| rfs::unlinkat(&dir.fd, from, AtFlags::empty()))
            }
            r => r,
        }
    };
    r.map_err(|e| err(e, leaf))?;
    let now = rfs::statat(&dir.fd, leaf, AtFlags::SYMLINK_NOFOLLOW).map_err(|e| err(e, leaf))?;
    if !same(&me, &now) {
        return Err(FsError::Reparse(format!("{leaf} is not the renamed file")));
    }
    Ok(())
}

/// a file the engine writes into or publishes has exactly one name (N-03, as on Windows)
fn single_link(f: File) -> Result<File, FsError> {
    let st = rfs::fstat(&f).map_err(io)?;
    if st.st_nlink != 1 {
        return Err(FsError::Reparse(format!("file has {} hard links", st.st_nlink)));
    }
    Ok(f)
}

/// (device, inode): the identity of the file object (the device folded to 32 bits like a volume serial)
pub fn file_id(f: &File) -> Result<(u32, u64), FsError> {
    let st = rfs::fstat(f).map_err(io)?;
    let dev = st.st_dev as u64;
    Ok(((dev ^ (dev >> 32)) as u32, st.st_ino as u64))
}

/// the filesystem's name ("apfs", "ext4", "btrfs"...)
pub fn fs_name(f: &File) -> String {
    let Ok(s) = rfs::fstatfs(f) else { return String::new() };
    #[cfg(target_vendor = "apple")]
    {
        let n: Vec<u8> = s.f_fstypename.iter().take_while(|c| **c != 0).map(|c| *c as u8).collect();
        String::from_utf8_lossy(&n).into_owned()
    }
    #[cfg(not(target_vendor = "apple"))]
    {
        #[allow(clippy::unnecessary_cast)] // f_type is i64, i32 or u32 depending on the CPU
        match s.f_type as i64 {
            0xEF53 => "ext4",
            0x9123_683E => "btrfs",
            0x5846_5342 => "xfs",
            0x0102_1994 => "tmpfs",
            0x2FC1_2FC1 => "zfs",
            0xF2F5_2010 => "f2fs",
            0x6969 => "nfs",
            0xFE53_4D42 | 0xFF53_4D42 => "smb",
            0x4d44 => "vfat",
            0x2011_BAB0 => "exfat",
            0x5346_544e | 0x7366_746e => "ntfs",
            0x6573_5546 => "fuse",
            _ => "other",
        }
        .to_string()
    }
}

/// Unix files are sparse by nature (a write past the end leaves a hole): nothing to switch
pub fn set_sparse(_f: &File, _on: bool) -> bool {
    true
}

/// bytes free for this user on the volume holding `dir`
pub fn free_space(dir: &Path) -> u64 {
    rfs::statvfs(dir).map(|s| s.f_bavail.saturating_mul(s.f_frsize)).unwrap_or(0)
}

#[cfg(target_vendor = "apple")]
const ORIGIN_XATTR: &str = "com.apple.quarantine";
#[cfg(not(target_vendor = "apple"))]
const ORIGIN_XATTR: &str = "user.xdg.origin.url";

/// the origin mark of `leaf` (macOS quarantine record / Linux origin URL), read through the held directory
pub fn zone_identifier(dir: &Dir, leaf: &str) -> Option<String> {
    let f = dir.file(leaf, OFlags::RDONLY, false, false).ok()?;
    let mut buf = [0u8; 4096];
    let n = rfs::fgetxattr(&f, ORIGIN_XATTR, &mut buf[..]).ok()?;
    Some(String::from_utf8_lossy(&buf[..n]).into_owned())
}

/// the Unix "Save": mark the origin (macOS quarantine: Gatekeeper checks the file before it first runs; Linux:
/// freedesktop origin attributes) and make sure the file is not executable. No antivirus exists to call. The
/// file is opened through the held directory (never a re-resolved path), regular files only.
pub fn attachment_save(dir: &Dir, leaf: &str, _final_name: &str, source: &str, referrer: Option<&str>) -> Result<(), String> {
    let f = single_link(dir.file(leaf, OFlags::RDONLY, false, false).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let st = rfs::fstat(&f).map_err(|e| e.to_string())?;
    rfs::fchmod(&f, Mode::from_raw_mode(st.st_mode & 0o666)).map_err(|e| e.to_string())?;
    set_origin(&OwnedFd::from(f), source, referrer)
}

#[cfg(target_vendor = "apple")]
fn set_origin(fd: &OwnedFd, _source: &str, _referrer: Option<&str>) -> Result<(), String> {
    // "flags;hex time;agent;event id" (0081 = downloaded, user-approved Gatekeeper check pending)
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let mut r = [0u8; 16];
    getrandom::fill(&mut r).map_err(|e| e.to_string())?;
    let h = crate::finalize::hex(&r).to_uppercase();
    let id = format!("{}-{}-{}-{}-{}", &h[..8], &h[8..12], &h[12..16], &h[16..20], &h[20..]);
    let v = format!("0081;{secs:08x};Galaxy Download Manager;{id}");
    rfs::fsetxattr(fd, ORIGIN_XATTR, v.as_bytes(), XattrFlags::empty()).map_err(|e| e.to_string())
}

#[cfg(not(target_vendor = "apple"))]
fn set_origin(fd: &OwnedFd, source: &str, referrer: Option<&str>) -> Result<(), String> {
    let set = |k: &str, v: &str| match rfs::fsetxattr(fd, k, v.as_bytes(), XattrFlags::empty()) {
        Ok(()) | Err(Errno::NOTSUP) => Ok(()), // a filesystem without user xattrs: the mode rule still holds
        Err(e) => Err(e.to_string()),
    };
    set(ORIGIN_XATTR, source)?;
    if let Some(r) = referrer {
        set("user.xdg.referrer.url", r)?;
    }
    Ok(())
}

/// the published file is protected by the OS: macOS = quarantine record present (Gatekeeper); Linux = not
/// executable (running it needs an explicit chmod by the user)
pub fn os_protected(f: &File, zone: Option<&str>) -> Result<(), String> {
    #[cfg(target_vendor = "apple")]
    {
        // read from the held file itself; "FFFF;..." with 4 hex flag digits, not yet user-approved (0x0040)
        let _ = zone;
        let mut buf = [0u8; 1024];
        let n = rfs::fgetxattr(f, ORIGIN_XATTR, &mut buf[..]).map_err(|_| "no quarantine record (Gatekeeper will not check it)".to_string())?;
        let v = String::from_utf8_lossy(&buf[..n]).into_owned();
        let flags = v.get(..4).filter(|_| v.as_bytes().get(4) == Some(&b';')).and_then(|h| u16::from_str_radix(h, 16).ok());
        match flags {
            Some(fl) if fl & 0x0040 == 0 => Ok(()),
            _ => Err(format!("quarantine record {v:?} does not make Gatekeeper check the file")),
        }
    }
    #[cfg(not(target_vendor = "apple"))]
    {
        let _ = zone;
        let st = rfs::fstat(f).map_err(|e| e.to_string())?;
        if st.st_mode & 0o111 == 0 {
            Ok(())
        } else {
            Err("the file is executable".into())
        }
    }
}

pub fn read_at(f: &File, buf: &mut [u8], off: u64) -> std::io::Result<usize> {
    std::os::unix::fs::FileExt::read_at(f, buf, off)
}
pub fn write_at(f: &File, buf: &[u8], off: u64) -> std::io::Result<usize> {
    std::os::unix::fs::FileExt::write_at(f, buf, off)
}

/// move a file to the Trash (P1 #28, UX rule 2: deleting a download is undoable). macOS: ~/.Trash; Linux: the
/// freedesktop trash of the user's home volume (a file on another volume is refused, never silently deleted)
pub fn recycle(path: &Path) -> Result<(), FsError> {
    let home = PathBuf::from(std::env::var_os("HOME").ok_or_else(|| io("no HOME"))?);
    let name = path.file_name().ok_or_else(|| io("no file name"))?.to_string_lossy().into_owned();
    #[cfg(target_vendor = "apple")]
    let (files, info) = (home.join(".Trash"), None::<PathBuf>);
    #[cfg(not(target_vendor = "apple"))]
    let (files, info) = {
        let base = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|| home.join(".local/share")).join("Trash");
        (base.join("files"), Some(base.join("info")))
    };
    std::fs::create_dir_all(&files).map_err(io)?;
    let tdir = Dir::open(&files)?;
    let mut n = 0;
    let leaf = loop {
        let leaf = if n == 0 { name.clone() } else { crate::name::with_counter(&name, n) };
        if !tdir.exists(&leaf) {
            break leaf;
        }
        n += 1;
        if n > 1000 {
            return Err(FsError::Exists);
        }
    };
    if let Some(info) = info {
        std::fs::create_dir_all(&info).map_err(io)?;
        let idir = Dir::open(&info)?;
        let enc: String = path
            .to_string_lossy()
            .bytes()
            .map(|b| if b.is_ascii_alphanumeric() || b"/-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") })
            .collect();
        let date = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S");
        let f = idir.create_new(&format!("{leaf}.trashinfo"))?;
        use std::io::Write;
        (&f).write_all(format!("[Trash Info]\nPath={enc}\nDeletionDate={date}\n").as_bytes()).map_err(io)?;
    }
    let src = Dir::open(path.parent().ok_or_else(|| io("no parent"))?)?;
    rfs::renameat_with(&src.fd, name.as_str(), &tdir.fd, leaf.as_str(), RenameFlags::NOREPLACE).map_err(|e| err(e, &name))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("gdm-fsx-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::canonicalize(d).unwrap() // macOS: /var -> /private/var
    }
    #[test]
    fn create_rename_relative() {
        let d = tmp("a");
        let dir = Dir::open(&d).unwrap();
        let f = dir.create_new("a.part").unwrap();
        assert!(matches!(dir.create_new("a.part"), Err(FsError::Exists)));
        assert!(matches!(dir.open_rw("a.part"), Err(FsError::Busy)), "GDM's own writer holds it");
        rename(&f, &dir, "a.part", "a.bin", false).unwrap();
        drop(f);
        assert!(d.join("a.bin").exists());
        dir.write_atomic("s.gdm", b"one").unwrap();
        dir.write_atomic("s.gdm", b"two").unwrap();
        assert_eq!(std::fs::read(d.join("s.gdm")).unwrap(), b"two");
        std::fs::write(d.join("taken"), b"user").unwrap();
        let g = dir.create_new("b.part").unwrap();
        assert!(matches!(rename(&g, &dir, "b.part", "taken", false), Err(FsError::Exists)));
        assert_eq!(std::fs::read(d.join("taken")).unwrap(), b"user");
        drop(g);
        dir.delete("s.gdm").unwrap();
        assert!(!d.join("s.gdm").exists());
    }
    /// N-03: a symlink planted at the leaf or on the way is refused, never followed
    #[test]
    fn symlinks_refused() {
        let d = tmp("b");
        let outside = tmp("b-out");
        std::os::unix::fs::symlink(outside.join("victim"), d.join("x.part")).unwrap();
        let dir = Dir::open(&d).unwrap();
        assert!(matches!(dir.create_new("x.part"), Err(FsError::Exists)));
        assert!(matches!(dir.open_rw("x.part"), Err(FsError::Reparse(_))));
        assert!(!outside.join("victim").exists());
        std::os::unix::fs::symlink(&outside, d.join("link")).unwrap();
        assert!(matches!(Dir::open(&d.join("link")), Err(FsError::Reparse(_))));
        assert!(matches!(dir.subdir("link"), Err(FsError::Reparse(_)) | Err(FsError::NotFound)));
        std::fs::write(d.join("h"), b"x").unwrap();
        std::fs::hard_link(d.join("h"), d.join("h2")).unwrap();
        assert!(matches!(dir.open_rw("h"), Err(FsError::Reparse(_))), "hard link refused");
    }
    /// council final review: a folder another user can change names in is refused (rename / hard-link races)
    #[test]
    fn folders_other_users_can_change_are_refused() {
        use std::os::unix::fs::PermissionsExt;
        let d = tmp("e");
        std::fs::set_permissions(&d, std::fs::Permissions::from_mode(0o777)).unwrap();
        assert!(matches!(Dir::open(&d), Err(FsError::Reparse(_))), "world-writable destination");
        std::fs::set_permissions(&d, std::fs::Permissions::from_mode(0o775)).unwrap();
        assert!(matches!(Dir::open(&d), Err(FsError::Reparse(_))), "group-writable destination");
        std::fs::set_permissions(&d, std::fs::Permissions::from_mode(0o755)).unwrap();
        let dir = Dir::open(&d).unwrap();
        std::fs::create_dir(d.join("open")).unwrap();
        std::fs::set_permissions(d.join("open"), std::fs::Permissions::from_mode(0o777)).unwrap();
        assert!(matches!(dir.subdir("open"), Err(FsError::Reparse(_))));
    }
    #[test]
    fn origin_mark_and_not_executable() {
        let d = tmp("c");
        let dir = Dir::open(&d).unwrap();
        drop(dir.create_new("m.part").unwrap());
        rfs::chmod(d.join("m.part"), Mode::from_raw_mode(0o755)).unwrap();
        attachment_save(&dir, "m.part", "m.bin", "https://e.com/m.bin", None).unwrap();
        let f = dir.open_locked("m.part").unwrap();
        assert!(os_protected(&f, zone_identifier(&dir, "m.part").as_deref()).is_ok(), "{:?}", zone_identifier(&dir, "m.part"));
    }
}
