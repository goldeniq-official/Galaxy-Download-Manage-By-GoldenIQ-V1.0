// SPDX-License-Identifier: RPL-1.5
//! Windows filesystem primitives (N-03, N-04, N-05, N-09). A destination directory is opened once and held
//! without FILE_SHARE_DELETE (it cannot be renamed, deleted or swapped for a junction while held); every file
//! the engine creates, opens or renames is addressed RELATIVE to that handle with a single sanitized leaf.
use super::FsError;
use std::fs::File;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::{Path, PathBuf};
use windows::core::{GUID, HSTRING, PCWSTR};
use windows::Wdk::Foundation::OBJECT_ATTRIBUTES;
use windows::Wdk::Storage::FileSystem::{
    NtCreateFile, FILE_CREATE, FILE_DIRECTORY_FILE, FILE_NON_DIRECTORY_FILE, FILE_OPEN, FILE_OPEN_IF, FILE_OPEN_REPARSE_POINT,
    FILE_SYNCHRONOUS_IO_NONALERT, NTCREATEFILE_CREATE_DISPOSITION, NTCREATEFILE_CREATE_OPTIONS,
};
use windows::Win32::Foundation::{HANDLE, OBJ_CASE_INSENSITIVE, UNICODE_STRING};
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::IO::IO_STATUS_BLOCK;

const SYNCHRONIZE: u32 = 0x0010_0000;
const DELETE: u32 = 0x0001_0000;
const IO_REPARSE_TAG_NAME_SURROGATE_BIT: u32 = 0x2000_0000;

fn io(e: impl std::fmt::Display) -> FsError {
    FsError::Io(e.to_string())
}

fn wide(s: &std::ffi::OsStr) -> Vec<u16> {
    s.encode_wide().chain(Some(0)).collect()
}

/// FILE_ATTRIBUTE_TAG_INFO of a handle: (attributes, reparse tag)
fn attr_tag(h: HANDLE) -> Result<(u32, u32), FsError> {
    let mut i = FILE_ATTRIBUTE_TAG_INFO::default();
    unsafe { GetFileInformationByHandleEx(h, FileAttributeTagInfo, &mut i as *mut _ as _, std::mem::size_of_val(&i) as u32) }.map_err(io)?;
    Ok((i.FileAttributes, i.ReparseTag))
}

/// junction, symlink and every other "name surrogate" redirect to another named entity; cloud placeholders
/// (OneDrive) and dedup tags do not and stay allowed
fn surrogate(h: HANDLE) -> Result<bool, FsError> {
    // basic attributes first, the tag only for a reparse point (as Rust std does): FAT/exFAT answer
    // FileAttributeTagInfo with STATUS_INVALID_PARAMETER (fastfat FatCommonQueryInformation has no case for it),
    // which refused every download to a FAT drive; a volume without reparse points never has the attribute
    let mut b = BY_HANDLE_FILE_INFORMATION::default();
    unsafe { GetFileInformationByHandle(h, &mut b) }.map_err(io)?;
    if b.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT.0 == 0 {
        return Ok(false);
    }
    let (a, tag) = attr_tag(h)?;
    Ok(a & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0 && tag & IO_REPARSE_TAG_NAME_SURROGATE_BIT != 0)
}

fn final_path(h: HANDLE) -> Result<PathBuf, FsError> {
    let mut buf = vec![0u16; 1024];
    let n = unsafe { GetFinalPathNameByHandleW(h, &mut buf, FILE_NAME_NORMALIZED) } as usize;
    if n == 0 || n > buf.len() {
        return Err(FsError::Io("GetFinalPathNameByHandle".into()));
    }
    Ok(PathBuf::from(String::from_utf16_lossy(&buf[..n])))
}

fn norm(p: &Path) -> String {
    let s = p.to_string_lossy().replace('/', "\\");
    let s = s.strip_prefix(r"\\?\UNC\").map(|r| format!(r"\\{r}")).unwrap_or_else(|| s.strip_prefix(r"\\?\").unwrap_or(&s).to_string());
    s.trim_end_matches('\\').to_lowercase()
}

/// `p` with its 8.3 short names expanded (`p` itself when that fails)
fn long_path(p: &Path) -> PathBuf {
    let w = wide(p.as_os_str());
    let mut buf = vec![0u16; 32768];
    let n = unsafe { GetLongPathNameW(PCWSTR(w.as_ptr()), Some(&mut buf)) } as usize;
    if n == 0 || n >= buf.len() {
        return p.to_path_buf();
    }
    PathBuf::from(std::ffi::OsString::from_wide(&buf[..n]))
}

/// A held destination directory.
pub struct Dir {
    h: OwnedHandle,
    pub path: PathBuf,
}

impl Dir {
    /// Open and hold. Fails when the directory itself is a name surrogate (junction/symlink) or when the
    /// path does not resolve to itself (an ancestor was swapped for a junction before we opened it).
    pub fn open(path: &Path) -> Result<Dir, FsError> {
        let abs = std::path::absolute(path).map_err(io)?;
        let w = wide(abs.as_os_str());
        let h = unsafe {
            CreateFileW(
                PCWSTR(w.as_ptr()),
                (FILE_LIST_DIRECTORY | FILE_TRAVERSE | FILE_READ_ATTRIBUTES | FILE_ADD_FILE).0 | SYNCHRONIZE,
                FILE_SHARE_READ | FILE_SHARE_WRITE, // no FILE_SHARE_DELETE: cannot be renamed/replaced while held
                None,
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
                None,
            )
        }
        .map_err(|e| if e.code().0 as u32 == 0x8007_0002 || e.code().0 as u32 == 0x8007_0003 { FsError::NotFound } else { io(e) })?;
        let h = unsafe { OwnedHandle::from_raw_handle(h.0 as _) };
        let raw = HANDLE(h.as_raw_handle());
        if surrogate(raw)? {
            return Err(FsError::Reparse(format!("{} is a junction or symbolic link", abs.display())));
        }
        let fp = final_path(raw)?;
        // 8.3 names (C:\Users\RUNNER~1) are the same folder, not a junction: compare the long form. GetLongPathNameW
        // only looks names up in each parent; it never follows a junction, so a swapped ancestor still differs.
        if norm(&fp) != norm(&long_path(&abs)) {
            return Err(FsError::Reparse(format!("{} resolves to {}", abs.display(), fp.display())));
        }
        Ok(Dir { h, path: abs })
    }

    fn raw(&self) -> HANDLE {
        HANDLE(self.h.as_raw_handle())
    }

    /// NtCreateFile relative to the held directory: `leaf` is one sanitized component.
    fn nt_open(&self, leaf: &str, access: u32, share: FILE_SHARE_MODE, create: bool) -> Result<File, FsError> {
        self.nt(leaf, access, share, if create { FILE_CREATE } else { FILE_OPEN }, FILE_NON_DIRECTORY_FILE)
    }

    /// a sub-directory relative to this held handle (created when missing), held the same way as `open`
    /// (no FILE_SHARE_DELETE). A junction/symlink there is refused, never followed (G-03 mirror folders).
    pub fn subdir(&self, leaf: &str) -> Result<Dir, FsError> {
        let f = self.nt(
            leaf,
            (FILE_LIST_DIRECTORY | FILE_TRAVERSE | FILE_ADD_FILE | FILE_ADD_SUBDIRECTORY).0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            FILE_OPEN_IF,
            FILE_DIRECTORY_FILE,
        )?;
        Ok(Dir { h: OwnedHandle::from(f), path: self.path.join(leaf) })
    }

    fn nt(
        &self,
        leaf: &str,
        access: u32,
        share: FILE_SHARE_MODE,
        disposition: NTCREATEFILE_CREATE_DISPOSITION,
        kind: NTCREATEFILE_CREATE_OPTIONS,
    ) -> Result<File, FsError> {
        // one component; the only stream allowed is Zone.Identifier (MOTW read-back)
        let base = leaf.strip_suffix(":Zone.Identifier").unwrap_or(leaf);
        if base.is_empty() || base.contains(['\\', '/', ':']) || base == "." || base == ".." {
            return Err(FsError::Io(format!("not a single component: {leaf:?}")));
        }
        let w: Vec<u16> = leaf.encode_utf16().collect();
        let us = UNICODE_STRING {
            Length: (w.len() * 2) as u16,
            MaximumLength: (w.len() * 2) as u16,
            Buffer: windows::core::PWSTR(w.as_ptr() as *mut u16),
        };
        let oa = OBJECT_ATTRIBUTES {
            Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: self.raw(),
            ObjectName: &us,
            Attributes: OBJ_CASE_INSENSITIVE,
            ..Default::default()
        };
        let mut h = HANDLE::default();
        let mut iosb = IO_STATUS_BLOCK::default();
        let st = unsafe {
            NtCreateFile(
                &mut h,
                FILE_ACCESS_RIGHTS(access | SYNCHRONIZE | FILE_READ_ATTRIBUTES.0), // read attributes: the reparse check below
                &oa,
                &mut iosb,
                None,
                FILE_ATTRIBUTE_NORMAL,
                share,
                disposition,
                kind | FILE_OPEN_REPARSE_POINT | FILE_SYNCHRONOUS_IO_NONALERT,
                None,
                0,
            )
        };
        match st.0 as u32 {
            0 => {}
            0xC000_0035 => return Err(FsError::Exists),                 // STATUS_OBJECT_NAME_COLLISION
            0xC000_0043 => return Err(FsError::Busy),                   // STATUS_SHARING_VIOLATION
            0xC000_0034 | 0xC000_003A => return Err(FsError::NotFound), // NAME_NOT_FOUND / PATH_NOT_FOUND
            s => return Err(FsError::Io(format!("NtCreateFile {leaf:?}: NTSTATUS {s:#010x}"))),
        }
        let f = unsafe { File::from_raw_handle(h.0 as _) };
        if surrogate(HANDLE(f.as_raw_handle()))? {
            return Err(FsError::Reparse(format!("{leaf} is a reparse point")));
        }
        Ok(f)
    }

    /// new file, never an existing one (CREATE_NEW semantics), read+write, others may only read
    pub fn create_new(&self, leaf: &str) -> Result<File, FsError> {
        self.nt_open(leaf, (FILE_GENERIC_READ | FILE_GENERIC_WRITE).0 | DELETE, FILE_SHARE_READ, true)
    }
    /// existing file for resume (rejects reparse points and hard links)
    pub fn open_rw(&self, leaf: &str) -> Result<File, FsError> {
        single_link(self.nt_open(leaf, (FILE_GENERIC_READ | FILE_GENERIC_WRITE).0 | DELETE, FILE_SHARE_READ, false)?)
    }
    /// read + delete access, share READ only: nobody can write, delete or rename it while held
    pub fn open_locked(&self, leaf: &str) -> Result<File, FsError> {
        single_link(self.nt_open(leaf, FILE_GENERIC_READ.0 | DELETE, FILE_SHARE_READ, false)?)
    }
    pub fn read_small(&self, leaf: &str, max: usize) -> Result<Vec<u8>, FsError> {
        let f = self.nt_open(leaf, FILE_GENERIC_READ.0, FILE_SHARE_READ, false)?;
        let mut v = Vec::new();
        use std::io::Read;
        f.take(max as u64 + 1).read_to_end(&mut v).map_err(io)?;
        Ok(v)
    }
    pub fn exists(&self, leaf: &str) -> bool {
        matches!(
            self.nt_open(leaf, FILE_READ_ATTRIBUTES.0, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, false),
            Ok(_) | Err(FsError::Reparse(_))
        )
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
        rename(&f, self, &tmp, leaf, true)
    }

    pub fn delete(&self, leaf: &str) -> Result<(), FsError> {
        use windows::Wdk::Storage::FileSystem::{FileDispositionInformation, NtSetInformationFile};
        let f = self.nt_open(leaf, DELETE, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, false)?;
        let mut iosb = IO_STATUS_BLOCK::default();
        let yes: u8 = 1; // FILE_DISPOSITION_INFORMATION { BOOLEAN DeleteFile }
        let st = unsafe { NtSetInformationFile(HANDLE(f.as_raw_handle()), &mut iosb, &yes as *const u8 as _, 1, FileDispositionInformation) };
        if st.0 == 0 {
            Ok(())
        } else {
            Err(FsError::Io(format!("delete {leaf:?}: NTSTATUS {:#010x}", st.0 as u32)))
        }
    }
}

/// rename an open file to `leaf` inside `dir` through the handle; `replace=false` never overwrites.
/// NtSetInformationFile(FileRenameInformation) with RootDirectory = the held directory: the target is
/// resolved relative to that handle (the Win32 SetFileInformationByHandle wrapper refuses a RootDirectory).
/// `_from` (the file's current leaf) is what Unix needs; Windows renames the handle itself.
pub fn rename(f: &File, dir: &Dir, _from: &str, leaf: &str, replace: bool) -> Result<(), FsError> {
    use windows::Wdk::Storage::FileSystem::{FileRenameInformation, NtSetInformationFile};
    let w: Vec<u16> = leaf.encode_utf16().collect();
    let size = std::mem::size_of::<FILE_RENAME_INFO>() + w.len() * 2;
    let mut buf = vec![0u64; size.div_ceil(8)];
    let mut iosb = IO_STATUS_BLOCK::default();
    let st = unsafe {
        let info = buf.as_mut_ptr() as *mut FILE_RENAME_INFO;
        (*info).Anonymous.ReplaceIfExists = replace;
        (*info).RootDirectory = dir.raw();
        (*info).FileNameLength = (w.len() * 2) as u32;
        std::ptr::copy_nonoverlapping(w.as_ptr(), (*info).FileName.as_mut_ptr(), w.len());
        NtSetInformationFile(HANDLE(f.as_raw_handle()), &mut iosb, info as *const _, size as u32, FileRenameInformation)
    };
    match st.0 as u32 {
        0 => Ok(()),
        0xC000_0035 => Err(FsError::Exists), // STATUS_OBJECT_NAME_COLLISION
        s => Err(FsError::Io(format!("rename to {leaf:?}: NTSTATUS {s:#010x}"))),
    }
}

/// a file the engine writes into or publishes has exactly one name: a hard link planted at the .part name would
/// otherwise let a write land in a user's file elsewhere on the volume (N-03)
fn single_link(f: File) -> Result<File, FsError> {
    let mut i = BY_HANDLE_FILE_INFORMATION::default();
    unsafe { GetFileInformationByHandle(HANDLE(f.as_raw_handle()), &mut i) }.map_err(io)?;
    if i.nNumberOfLinks != 1 {
        return Err(FsError::Reparse(format!("file has {} hard links", i.nNumberOfLinks)));
    }
    Ok(f)
}

/// (volume serial, file index): the identity of the file object
pub fn file_id(f: &File) -> Result<(u32, u64), FsError> {
    let mut i = BY_HANDLE_FILE_INFORMATION::default();
    unsafe { GetFileInformationByHandle(HANDLE(f.as_raw_handle()), &mut i) }.map_err(io)?;
    Ok((i.dwVolumeSerialNumber, ((i.nFileIndexHigh as u64) << 32) | i.nFileIndexLow as u64))
}

pub fn fs_name(f: &File) -> String {
    let mut name = [0u16; 64];
    let _ = unsafe { GetVolumeInformationByHandleW(HANDLE(f.as_raw_handle()), None, None, None, None, Some(&mut name)) };
    String::from_utf16_lossy(&name).trim_end_matches('\0').to_string()
}

/// FSCTL_SET_SPARSE; best effort (FAT/exFAT have no sparse files and keep the normal path)
pub fn set_sparse(f: &File, on: bool) -> bool {
    use windows::Win32::System::Ioctl::{FILE_SET_SPARSE_BUFFER, FSCTL_SET_SPARSE};
    use windows::Win32::System::IO::DeviceIoControl;
    let b = FILE_SET_SPARSE_BUFFER { SetSparse: on };
    let mut n = 0u32;
    unsafe {
        DeviceIoControl(
            HANDLE(f.as_raw_handle()),
            FSCTL_SET_SPARSE,
            Some(&b as *const _ as _),
            std::mem::size_of_val(&b) as u32,
            None,
            0,
            Some(&mut n),
            None,
        )
        .is_ok()
    }
}

/// bytes free for this user on the volume holding `dir`
pub fn free_space(dir: &Path) -> u64 {
    let w = wide(dir.as_os_str());
    let mut free = 0u64;
    let _ = unsafe { GetDiskFreeSpaceExW(PCWSTR(w.as_ptr()), Some(&mut free), None, None) };
    free
}

/// Zone.Identifier ADS of `leaf` (read through the held directory, never a re-resolved path)
pub fn zone_identifier(dir: &Dir, leaf: &str) -> Option<String> {
    // read-only access with full sharing: the finaliser holds the file with DELETE access at this point
    let f =
        dir.nt_open(&format!("{leaf}:Zone.Identifier"), FILE_GENERIC_READ.0, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, false).ok()?;
    let mut v = Vec::new();
    use std::io::Read;
    f.take(4096).read_to_end(&mut v).ok()?;
    Some(String::from_utf8_lossy(&v).into_owned())
}

/// move a file to the Recycle Bin (P1 #28, UX rule 2: deleting a download is undoable)
pub fn recycle(path: &Path) -> Result<(), FsError> {
    use windows::Win32::UI::Shell::{SHFileOperationW, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT, FO_DELETE, SHFILEOPSTRUCTW};
    let mut from: Vec<u16> = path.as_os_str().encode_wide().collect();
    from.extend([0, 0]); // double-NUL terminated list of one path
    let mut op = SHFILEOPSTRUCTW {
        wFunc: FO_DELETE,
        pFrom: PCWSTR(from.as_ptr()),
        fFlags: (FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_SILENT | FOF_NOERRORUI).0 as u16,
        ..Default::default()
    };
    let r = unsafe { SHFileOperationW(&mut op) };
    if r != 0 || op.fAnyOperationsAborted.as_bool() {
        return Err(FsError::Io(format!("recycle {}: {r:#x}", path.display())));
    }
    Ok(())
}

pub const GDM_CLIENT: GUID = GUID::from_u128(0x6a1c_2f0e_9b7d_4c3a_8e51_0d47_c2f9_a3b6);

/// IAttachmentExecute::Save in its own STA thread (MOTW + AV, N-05). `local` is the full path of the .part,
/// `final_name` the name the user will see (set before Save so the policy evaluates the real type).
pub fn attachment_save(dir: &Dir, leaf: &str, final_name: &str, source: &str, referrer: Option<&str>) -> Result<(), String> {
    let local = dir.path.join(leaf); // IAttachmentExecute takes a path; finalize then checks the file object id
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
    use windows::Win32::UI::Shell::{AttachmentServices, IAttachmentExecute};
    let (local, final_name, source, referrer) = (local, final_name.to_string(), source.to_string(), referrer.map(String::from));
    std::thread::spawn(move || unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        let r = (|| {
            let ae: IAttachmentExecute = CoCreateInstance(&AttachmentServices, None, CLSCTX_INPROC_SERVER)?;
            ae.SetClientGuid(&GDM_CLIENT)?;
            ae.SetLocalPath(&HSTRING::from(local.as_os_str()))?;
            ae.SetFileName(&HSTRING::from(final_name.as_str()))?;
            ae.SetSource(&HSTRING::from(source.as_str()))?;
            if let Some(r) = &referrer {
                ae.SetReferrer(&HSTRING::from(r.as_str()))?;
            }
            ae.Save()
        })();
        CoUninitialize();
        r
    })
    .join()
    .unwrap_or_else(|_| Err(windows::core::Error::from_hresult(windows::core::HRESULT(0x8000_4005u32 as i32))))
    .map_err(|e| format!("IAttachmentExecute::Save {:#010x}", e.code().0))
}

/// positional read/write that leave no shared cursor assumptions (Windows: seek_read/seek_write)
pub fn read_at(f: &File, buf: &mut [u8], off: u64) -> std::io::Result<usize> {
    std::os::windows::fs::FileExt::seek_read(f, buf, off)
}
pub fn write_at(f: &File, buf: &[u8], off: u64) -> std::io::Result<usize> {
    std::os::windows::fs::FileExt::seek_write(f, buf, off)
}

/// the published file is protected by Windows: NTFS/ReFS and a Zone.Identifier of Internet/Restricted
/// (SmartScreen, Protected View act on it)
pub fn os_protected(f: &File, zone: Option<&str>) -> Result<(), String> {
    let fs = fs_name(f);
    if matches!(fs.as_str(), "NTFS" | "ReFS") && crate::finalize::zone_internet(zone) {
        Ok(())
    } else {
        Err(format!("filesystem {fs} without Mark-of-the-Web"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_rename_relative() {
        let d = std::env::temp_dir().join(format!("gdm-fsx-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let dir = Dir::open(&d).unwrap();
        let f = dir.create_new("a.part").unwrap();
        assert!(matches!(dir.create_new("a.part"), Err(FsError::Exists)));
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
}
