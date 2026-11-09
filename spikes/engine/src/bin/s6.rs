// SPDX-License-Identifier: RPL-1.5
//! S6 check: finalisation (§09 step 8). byte count = total -> optional SHA-256 (through the handle) ->
//! IAttachmentExecute::Save (MOTW + AV) -> same file object (file id) -> rename by handle, never replace.
//! States: Complete, Blocked ("ត្រូវបានរារាំង", only with evidence), Unprotected ("គោលដៅមិនការពារ", FAT/exFAT).
use gdm_spikes::seg::sha_handle;
use gdm_spikes::Check;
use std::fs::{File, OpenOptions};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;
use std::path::{Path, PathBuf};
use windows::core::{GUID, HSTRING, PCWSTR};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{
    FileRenameInfo, GetFileInformationByHandle, GetVolumeInformationByHandleW, SetFileInformationByHandle,
    BY_HANDLE_FILE_INFORMATION, FILE_RENAME_INFO,
};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::Shell::{AttachmentServices, IAttachmentExecute};

const DELETE: u32 = 0x0001_0000;
const GENERIC_READ: u32 = 0x8000_0000;
const SHARE_READ: u32 = 1;
const GDM_CLIENT: GUID = GUID::from_u128(0x6a1c_2f0e_9b7d_4c3a_8e51_0d47_c2f9_a3b6);

#[derive(Debug, PartialEq)]
enum Final {
    Complete(PathBuf),
    Blocked(String),
    Unprotected(PathBuf),
    Failed(String),
}

impl Final {
    fn label_km(&self) -> &'static str {
        match self { Final::Complete(_) => "បានបញ្ចប់", Final::Blocked(_) => "ត្រូវបានរារាំង", Final::Unprotected(_) => "គោលដៅមិនការពារ", Final::Failed(_) => "បរាជ័យ" }
    }
}

fn file_id(f: &File) -> (u32, u32, u32) {
    let mut i = BY_HANDLE_FILE_INFORMATION::default();
    unsafe { GetFileInformationByHandle(HANDLE(f.as_raw_handle()), &mut i).unwrap() };
    (i.dwVolumeSerialNumber, i.nFileIndexHigh, i.nFileIndexLow)
}

fn fs_name(f: &File) -> String {
    let mut name = [0u16; 64];
    unsafe { GetVolumeInformationByHandleW(HANDLE(f.as_raw_handle()), None, None, None, None, Some(&mut name)).unwrap() };
    String::from_utf16_lossy(&name).trim_end_matches('\0').to_string()
}

/// FILE_RENAME_INFO with ReplaceIfExists = false on an open handle
fn rename_no_replace(f: &File, to: &Path) -> windows::core::Result<()> {
    let wide: Vec<u16> = to.as_os_str().encode_wide().collect();
    let size = std::mem::size_of::<FILE_RENAME_INFO>() + wide.len() * 2;
    let mut buf = vec![0u8; size];
    unsafe {
        let info = buf.as_mut_ptr() as *mut FILE_RENAME_INFO;
        (*info).Anonymous.ReplaceIfExists = false;
        (*info).FileNameLength = (wide.len() * 2) as u32;
        std::ptr::copy_nonoverlapping(wide.as_ptr(), (*info).FileName.as_mut_ptr(), wide.len());
        SetFileInformationByHandle(HANDLE(f.as_raw_handle()), FileRenameInfo, info as *const _, size as u32)
    }
}

/// IAttachmentExecute in an STA, as Microsoft requires. Returns the HRESULT of Save.
fn attachment_save(path: &Path, final_name: &str, source: &str) -> windows::core::Result<()> {
    std::thread::scope(|s| {
        s.spawn(|| unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
            let ae: IAttachmentExecute = CoCreateInstance(&AttachmentServices, None, CLSCTX_INPROC_SERVER)?;
            ae.SetClientGuid(&GDM_CLIENT)?;
            ae.SetLocalPath(&HSTRING::from(path.as_os_str()))?;
            ae.SetFileName(&HSTRING::from(final_name))?;
            ae.SetSource(&HSTRING::from(source))?;
            ae.Save()
        }).join().unwrap()
    })
}

struct Evidence { pre_exists: bool, hr: String, post_exists: bool, zone: Option<String>, fs: String }

fn finalize(part: &Path, dest: &Path, total: u64, sha: Option<&str>, source: &str, fake_fs: Option<&str>) -> (Final, Evidence) {
    let mut ev = Evidence { pre_exists: part.exists(), hr: String::new(), post_exists: false, zone: None, fs: String::new() };
    if !ev.pre_exists { return (Final::Blocked("file gone before Save (quarantined on write)".into()), ev); }
    // 1. byte count + hash through one handle (share read only: nobody can write meanwhile)
    let id1;
    {
        let f = match OpenOptions::new().read(true).share_mode(SHARE_READ).open(part) { Ok(f) => f, Err(e) => return (Final::Failed(format!("open: {e}")), ev) };
        let len = f.metadata().unwrap().len();
        if len != total { return (Final::Failed(format!("byte count {len} != total {total}")), ev); }
        if let Some(want) = sha { if sha_handle(&f).unwrap() != want { return (Final::Failed("sha256 mismatch".into()), ev); } }
        id1 = file_id(&f);
        ev.fs = fake_fs.map(String::from).unwrap_or_else(|| fs_name(&f));
    } // handle closed: AV must be free to quarantine/delete during Save
    // 2. attachment services: MOTW + AV
    let r = attachment_save(part, &dest.file_name().unwrap().to_string_lossy(), source);
    ev.hr = match &r { Ok(()) => "S_OK".into(), Err(e) => format!("{:#010x} {}", e.code().0, e.message()) };
    ev.post_exists = part.exists();
    ev.zone = std::fs::read_to_string(format!("{}:Zone.Identifier", part.display())).ok();
    if !ev.post_exists { return (Final::Blocked(format!("Save {} and file removed", ev.hr)), ev); }
    if r.is_err() { return (Final::Failed(format!("Save {} but file still present: operational failure, not proof of malware", ev.hr)), ev); }
    // 3. same file object, then rename by handle without replace
    let f = match OpenOptions::new().access_mode(GENERIC_READ | DELETE).share_mode(SHARE_READ).open(part) { Ok(f) => f, Err(e) => return (Final::Failed(format!("reopen: {e}")), ev) };
    if file_id(&f) != id1 { return (Final::Failed("file object changed between hash and rename".into()), ev); }
    if let Err(e) = rename_no_replace(&f, dest) { return (Final::Failed(format!("rename: {:#010x} {}", e.code().0, e.message())), ev); }
    let protected = matches!(ev.fs.as_str(), "NTFS" | "ReFS") && ev.zone.is_some();
    (if protected { Final::Complete(dest.into()) } else { Final::Unprotected(dest.into()) }, ev)
}

fn main() {
    let mut c = Check::new("S6 finalisation");
    let d = std::env::temp_dir().join(format!("gdm-s6-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    let src = "https://downloads.example.com/tool/setup.exe";
    let body = b"MZ harmless test payload".to_vec();
    let sha = { use sha2::Digest; gdm_spikes::fixture::hex(&sha2::Sha256::digest(&body)) };

    // 1. normal NTFS file
    let part = d.join("setup.exe.part");
    std::fs::write(&part, &body).unwrap();
    let dest = d.join("setup.exe");
    let (st, ev) = finalize(&part, &dest, body.len() as u64, Some(&sha), src, None);
    println!("     fs={} Save={} zone={:?}", ev.fs, ev.hr, ev.zone.as_deref().map(|z| z.replace("\r\n", " | ")));
    c.ok(st == Final::Complete(dest.clone()), &format!("NTFS: Save S_OK, renamed -> {} ({:?})", st.label_km(), st));
    let z = std::fs::read_to_string(format!("{}:Zone.Identifier", dest.display())).unwrap_or_default();
    c.ok(z.contains("ZoneId=3"), "MOTW Zone.Identifier ZoneId=3 survives the rename (ADS kept)");
    c.ok(z.contains("HostUrl=") || z.contains("ZoneId=3"), &format!("HostUrl present: {} (policy-dependent, not promised)", z.contains("HostUrl=")));

    // 2. wrong size / wrong hash => never Complete, nothing renamed
    let p2 = d.join("b.bin.part");
    std::fs::write(&p2, &body).unwrap();
    let (st, _) = finalize(&p2, &d.join("b.bin"), body.len() as u64 + 1, None, src, None);
    c.ok(matches!(st, Final::Failed(ref m) if m.contains("byte count")) && !d.join("b.bin").exists(), "byte count != total -> Failed, no final name");
    let (st, _) = finalize(&p2, &d.join("b.bin"), body.len() as u64, Some("00"), src, None);
    c.ok(matches!(st, Final::Failed(ref m) if m.contains("sha256")) && !d.join("b.bin").exists(), "sha256 mismatch -> Failed, no final name");

    // 3. destination exists => rename refuses, existing file byte-for-byte intact
    let p3 = d.join("c.txt.part");
    std::fs::write(&p3, b"new").unwrap();
    std::fs::write(d.join("c.txt"), b"user's own file").unwrap();
    let (st, _) = finalize(&p3, &d.join("c.txt"), 3, None, src, None);
    c.ok(matches!(st, Final::Failed(ref m) if m.contains("rename")) && std::fs::read(d.join("c.txt")).unwrap() == b"user's own file", &format!("existing destination: no replace, user file intact ({st:?})"));

    // 4. EICAR test file => AV blocks: evidence decides, not the HRESULT alone
    let eicar: String = "*H+H$!ELIF-TSET-SURIVITNA-DRADNATS-RACIE$}7)CC7)^P(45XZP\\4[PA@%P!O5X".chars().rev().collect();
    let p4 = d.join("eicar.com.part");
    let wrote = std::fs::write(&p4, eicar.as_bytes()).is_ok();
    std::thread::sleep(std::time::Duration::from_millis(1500));
    let (st, ev) = finalize(&p4, &d.join("eicar.com"), eicar.len() as u64, None, src, None);
    println!("     eicar: wrote={wrote} pre_exists={} Save={} post_exists={}", ev.pre_exists, ev.hr, ev.post_exists);
    c.ok(matches!(st, Final::Blocked(_)) && !d.join("eicar.com").exists(), &format!("EICAR -> {} ({st:?}); never Complete, no final name", st.label_km()));

    // 5. untaggable target: the FS check is real code, the FAT/exFAT volume is injected (no FAT volume, not admin)
    let p5 = d.join("e.exe.part");
    std::fs::write(&p5, &body).unwrap();
    let (st, _) = finalize(&p5, &d.join("e.exe"), body.len() as u64, None, src, Some("exFAT"));
    c.ok(matches!(st, Final::Unprotected(_)), &format!("exFAT (injected fs name) -> {} ({st:?})", st.label_km()));
    c.note("NOT TESTED on a real FAT/exFAT volume: needs admin (VHD) or a USB stick; run `s6 <path-on-fat-volume>` there");
    if let Some(p) = std::env::args().nth(1) {
        let p6 = Path::new(&p).join("gdm-s6.exe.part");
        std::fs::write(&p6, &body).unwrap();
        let (st, ev) = finalize(&p6, &Path::new(&p).join("gdm-s6.exe"), body.len() as u64, None, src, None);
        c.ok(matches!(st, Final::Unprotected(_)), &format!("real volume {p} fs={} Save={} zone={:?} -> {}", ev.fs, ev.hr, ev.zone, st.label_km()));
    }
    c.finish();
}
