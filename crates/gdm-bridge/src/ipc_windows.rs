// SPDX-License-Identifier: RPL-1.5
//! Windows: the per-user named pipe (B-02).
use crate::MAX_MSG;
use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, LocalFree, HANDLE, HLOCAL};
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::Security::{GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// the current user's SID (S-1-5-...)
pub fn user_sid() -> Option<String> {
    unsafe {
        let mut tok = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut tok).ok()?;
        let mut buf = vec![0u8; 256];
        let mut len = 0;
        let r = GetTokenInformation(tok, TokenUser, Some(buf.as_mut_ptr() as _), buf.len() as u32, &mut len);
        let _ = CloseHandle(tok);
        r.ok()?;
        let tu = &*(buf.as_ptr() as *const TOKEN_USER);
        let mut s = PWSTR::null();
        ConvertSidToStringSidW(tu.User.Sid, &mut s).ok()?;
        let out = s.to_string().ok();
        LocalFree(Some(HLOCAL(s.0 as _)));
        out
    }
}

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use windows::core::{HSTRING, PWSTR as PW};
use windows::Win32::Foundation::ERROR_PIPE_BUSY;
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, WriteFile, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_NONE, OPEN_EXISTING, SECURITY_IDENTIFICATION, SECURITY_SQOS_PRESENT,
};
use windows::Win32::System::Pipes::{GetNamedPipeServerProcessId, SetNamedPipeHandleState, WaitNamedPipeW, PIPE_READMODE_MESSAGE};
use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};

/// the image path of a process, read while holding its handle (the pid cannot be reused meanwhile)
pub fn image_of(pid: u32) -> Option<PathBuf> {
    unsafe {
        let ph = H(OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?);
        let mut img = vec![0u16; 1024];
        let mut len = img.len() as u32;
        QueryFullProcessImageNameW(ph.0, PROCESS_NAME_WIN32, PW(img.as_mut_ptr()), &mut len).ok()?;
        Some(PathBuf::from(String::from_utf16_lossy(&img[..len as usize])))
    }
}

const GENERIC_RW: u32 = 0xC000_0000;

pub struct H(pub HANDLE);
impl Drop for H {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

pub fn norm(p: &Path) -> String {
    p.to_string_lossy().to_lowercase()
}

/// B-02: open the pipe with SQOS identification (the server can never act as us), then check the server
/// process image while holding its process handle (the pid cannot be reused) BEFORE sending a byte
pub fn call(pipe: &str, expected: &Path, msg: &[u8], wait: Duration) -> Result<Vec<u8>, String> {
    let flags = FILE_FLAGS_AND_ATTRIBUTES(SECURITY_SQOS_PRESENT.0 | SECURITY_IDENTIFICATION.0);
    let deadline = Instant::now() + wait;
    let h = loop {
        match unsafe { CreateFileW(&HSTRING::from(pipe), GENERIC_RW, FILE_SHARE_NONE, None, OPEN_EXISTING, flags, None) } {
            Ok(h) => break H(h),
            Err(e) if Instant::now() < deadline => {
                if e.code() == ERROR_PIPE_BUSY.to_hresult() {
                    unsafe {
                        let _ = WaitNamedPipeW(&HSTRING::from(pipe), 500);
                    }
                } else {
                    std::thread::sleep(Duration::from_millis(100)); // gdm.exe still starting
                }
            }
            Err(e) => return Err(format!("no GDM: {e}")),
        }
    };
    unsafe {
        let mut pid = 0;
        GetNamedPipeServerProcessId(h.0, &mut pid).map_err(|e| e.to_string())?;
        let ph = H(OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).map_err(|e| e.to_string())?);
        let mut img = vec![0u16; 1024];
        let mut len = img.len() as u32;
        QueryFullProcessImageNameW(ph.0, PROCESS_NAME_WIN32, PW(img.as_mut_ptr()), &mut len).map_err(|e| e.to_string())?;
        let image = PathBuf::from(String::from_utf16_lossy(&img[..len as usize]));
        if norm(&image) != norm(expected) {
            return Err(format!("refused: pipe server is {}", image.display()));
        }
        SetNamedPipeHandleState(h.0, Some(&PIPE_READMODE_MESSAGE), None, None).map_err(|e| e.to_string())?;
        WriteFile(h.0, Some(msg), None, None).map_err(|e| format!("write: {e}"))?;
        let mut buf = vec![0u8; MAX_MSG as usize];
        let mut n = 0;
        ReadFile(h.0, Some(&mut buf), Some(&mut n), None).map_err(|e| format!("read: {e}"))?;
        buf.truncate(n as usize);
        drop(ph);
        Ok(buf)
    }
}

/// B-02: the pipe name carries the user's SID
pub fn pipe_name() -> String {
    format!(r"\\.\pipe\GDM-{}", user_sid().unwrap_or_else(|| "nosid".into()))
}
