// SPDX-License-Identifier: RPL-1.5
//! S3 check: named pipe with an explicit DACL (current user only, protected), FILE_FLAG_FIRST_PIPE_INSTANCE,
//! PIPE_REJECT_REMOTE_CLIENTS, max 1 instance. The client opens with SECURITY_SQOS_PRESENT|SECURITY_IDENTIFICATION
//! and verifies the connected server process (GetNamedPipeServerProcessId + held process handle + image path)
//! BEFORE sending anything. Other principals are simulated without admin: the anonymous token and a restricted
//! token with the user SID deny-only (a real second account needs admin; see results.md).
//!   s3                      full check
//!   s3 serve <name> <mode>  mode = genuine | squat (squat = default security, logs what it receives)
use gdm_spikes::Check;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use windows::core::{HSTRING, PWSTR};
use windows::Win32::Foundation::{CloseHandle, GetLastError, LocalFree, ERROR_MORE_DATA, ERROR_PIPE_CONNECTED, HANDLE, HLOCAL};
use windows::Win32::Security::Authorization::{
    ConvertSecurityDescriptorToStringSecurityDescriptorW, ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
    GetSecurityInfo, SDDL_REVISION_1, SE_KERNEL_OBJECT,
};
use windows::Win32::Security::{
    CreateRestrictedToken, GetTokenInformation, ImpersonateAnonymousToken, ImpersonateLoggedOnUser, RevertToSelf,
    CREATE_RESTRICTED_TOKEN_FLAGS, DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, SID_AND_ATTRIBUTES,
    SECURITY_IMPERSONATION_LEVEL, TOKEN_ALL_ACCESS, TOKEN_QUERY, TOKEN_USER, TokenImpersonationLevel, TokenUser,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FlushFileBuffers, ReadFile, WriteFile, FILE_FLAGS_AND_ATTRIBUTES, FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_SHARE_NONE,
    OPEN_EXISTING, PIPE_ACCESS_DUPLEX, SECURITY_IDENTIFICATION, SECURITY_SQOS_PRESENT,
};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, GetNamedPipeServerProcessId, ImpersonateNamedPipeClient,
    SetNamedPipeHandleState, PIPE_READMODE_MESSAGE, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_MESSAGE, PIPE_WAIT,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentThread, OpenProcess, OpenProcessToken, OpenThreadToken, QueryFullProcessImageNameW,
    PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};

const GENERIC_RW: u32 = 0xC000_0000;
const GENERIC_READ: u32 = 0x8000_0000;
const MAX_MSG: usize = 64 * 1024;

fn pipe(name: &str) -> String { format!(r"\\.\pipe\{name}") }

/// current user's SID as an SDDL string
fn user_sid() -> String {
    unsafe {
        let mut tok = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut tok).unwrap();
        let mut buf = vec![0u8; 256];
        let mut len = 0;
        GetTokenInformation(tok, TokenUser, Some(buf.as_mut_ptr() as _), buf.len() as u32, &mut len).unwrap();
        let tu = &*(buf.as_ptr() as *const TOKEN_USER);
        let mut s = PWSTR::null();
        ConvertSidToStringSidW(tu.User.Sid, &mut s).unwrap();
        let r = s.to_string().unwrap();
        LocalFree(Some(HLOCAL(s.0 as _)));
        let _ = CloseHandle(tok);
        r
    }
}

/// genuine: protected DACL with one ACE (user SID, generic all); squat: default security, no flags
fn create_pipe(name: &str, genuine: bool) -> Result<HANDLE, u32> {
    unsafe {
        let mut sd = PSECURITY_DESCRIPTOR::default();
        let sa;
        let psa = if genuine {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(&HSTRING::from(format!("D:P(A;;GA;;;{})", user_sid())), SDDL_REVISION_1, &mut sd, None).unwrap();
            sa = SECURITY_ATTRIBUTES { nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32, lpSecurityDescriptor: sd.0, bInheritHandle: false.into() };
            Some(&sa as *const _)
        } else { None }; // ponytail: SD leaks once per server process, fine for a spike
        let (open, mode, max) = if genuine {
            (PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS, 1)
        } else {
            (PIPE_ACCESS_DUPLEX, PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT, 255)
        };
        let h = CreateNamedPipeW(&HSTRING::from(pipe(name)), open, mode, max, 65536, MAX_MSG as u32, 0, psa);
        if h.is_invalid() { Err(GetLastError().0) } else { Ok(h) }
    }
}

fn imp_level(h: HANDLE) -> i32 {
    unsafe {
        if ImpersonateNamedPipeClient(h).is_err() { return -1; }
        let mut t = HANDLE::default();
        let mut lvl = SECURITY_IMPERSONATION_LEVEL(-1);
        if OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, true, &mut t).is_ok() {
            let mut n = 0;
            let _ = GetTokenInformation(t, TokenImpersonationLevel, Some(&mut lvl as *mut _ as _), 4, &mut n);
            let _ = CloseHandle(t);
        }
        RevertToSelf().unwrap();
        lvl.0
    }
}

fn serve(name: &str, genuine: bool) -> ! {
    let h = match create_pipe(name, genuine) {
        Ok(h) => h,
        Err(e) => { println!("FAILCLOSED {e}"); std::process::exit(3) }
    };
    println!("READY");
    std::io::stdout().flush().unwrap();
    let mut buf = vec![0u8; MAX_MSG];
    loop {
        unsafe {
            if let Err(e) = ConnectNamedPipe(h, None) { if e.code() != ERROR_PIPE_CONNECTED.to_hresult() { continue; } }
            let mut n = 0u32;
            let reply = match ReadFile(h, Some(&mut buf), Some(&mut n), None) {
                Err(e) if e.code() == ERROR_MORE_DATA.to_hresult() => r#"{"ok":false,"err":"message > 64 KiB"}"#.to_string(),
                Err(_) => { let _ = DisconnectNamedPipe(h); continue; }
                Ok(()) => {
                    if !genuine { println!("GOT {n} bytes: {}", String::from_utf8_lossy(&buf[..n as usize])); std::io::stdout().flush().unwrap(); }
                    let lvl = imp_level(h);
                    match serde_json::from_slice::<serde_json::Value>(&buf[..n as usize]) {
                        Ok(_) => format!(r#"{{"ok":true,"imp":{lvl}}}"#),
                        Err(e) => format!(r#"{{"ok":false,"err":"json: {}"}}"#, e.to_string().replace('"', "'")),
                    }
                }
            };
            let _ = WriteFile(h, Some(reply.as_bytes()), None, None);
            let _ = FlushFileBuffers(h);
            let _ = DisconnectNamedPipe(h);
        }
    }
}

/// raw open: SQOS identification so a server can never act as us
fn open(path: &str, access: u32) -> Result<HANDLE, u32> {
    let flags = FILE_FLAGS_AND_ATTRIBUTES(SECURITY_SQOS_PRESENT.0 | SECURITY_IDENTIFICATION.0);
    let dl = Instant::now() + Duration::from_secs(2);
    loop {
        match unsafe { CreateFileW(&HSTRING::from(path), access, FILE_SHARE_NONE, None, OPEN_EXISTING, flags, None) } {
            Ok(h) => return Ok(h),
            Err(e) => {
                let code = (e.code().0 & 0xFFFF) as u32;
                if code == 231 && Instant::now() < dl { std::thread::sleep(Duration::from_millis(10)); continue; } // PIPE_BUSY between clients
                return Err(code);
            }
        }
    }
}

fn exchange(h: HANDLE, msg: &[u8]) -> Result<String, String> {
    unsafe {
        SetNamedPipeHandleState(h, Some(&PIPE_READMODE_MESSAGE), None, None).map_err(|e| e.to_string())?;
        WriteFile(h, Some(msg), None, None).map_err(|e| format!("write: {e}"))?;
        let mut buf = vec![0u8; 4096];
        let mut n = 0;
        ReadFile(h, Some(&mut buf), Some(&mut n), None).map_err(|e| format!("read: {e}"))?;
        Ok(String::from_utf8_lossy(&buf[..n as usize]).to_string())
    }
}

fn norm(p: &Path) -> String { p.to_string_lossy().to_lowercase() }

/// the client GDM ships: verify the server image while holding its process handle, then send
fn verified_call(name: &str, expected: &Path, msg: &[u8]) -> Result<String, String> {
    let h = open(&pipe(name), GENERIC_RW).map_err(|e| format!("open: {e}"))?;
    let r = unsafe {
        let mut pid = 0;
        GetNamedPipeServerProcessId(h, &mut pid).map_err(|e| e.to_string())?;
        let ph = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).map_err(|e| e.to_string())?; // held: pid cannot be reused
        let mut img = vec![0u16; 1024];
        let mut len = img.len() as u32;
        QueryFullProcessImageNameW(ph, PROCESS_NAME_WIN32, PWSTR(img.as_mut_ptr()), &mut len).map_err(|e| e.to_string())?;
        let image = PathBuf::from(String::from_utf16_lossy(&img[..len as usize]));
        let r = if norm(&image) != norm(expected) { Err(format!("refused: server image {} != expected", image.display())) } else { exchange(h, msg) };
        let _ = CloseHandle(ph);
        r
    };
    unsafe { let _ = CloseHandle(h); }
    r
}

#[derive(Clone, Copy, Debug)]
enum Other { Anonymous, UserSidDenyOnly }

/// run f on a thread impersonating another principal; also returns whether the impersonation took effect
fn as_other<T: Send + 'static>(who: Other, f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::spawn(move || unsafe {
        match who {
            Other::Anonymous => ImpersonateAnonymousToken(GetCurrentThread()).unwrap(),
            Other::UserSidDenyOnly => {
                let mut tok = HANDLE::default();
                OpenProcessToken(GetCurrentProcess(), TOKEN_ALL_ACCESS, &mut tok).unwrap();
                let mut buf = vec![0u8; 256];
                let mut len = 0;
                GetTokenInformation(tok, TokenUser, Some(buf.as_mut_ptr() as _), buf.len() as u32, &mut len).unwrap();
                let sid = (*(buf.as_ptr() as *const TOKEN_USER)).User.Sid;
                let mut rt = HANDLE::default();
                CreateRestrictedToken(tok, CREATE_RESTRICTED_TOKEN_FLAGS(0), Some(&[SID_AND_ATTRIBUTES { Sid: sid, Attributes: 0 }]), None, None, &mut rt).unwrap();
                ImpersonateLoggedOnUser(rt).unwrap();
            }
        }
        let r = f();
        RevertToSelf().unwrap();
        r
    }).join().unwrap()
}

fn dacl_sddl(h: HANDLE) -> String {
    unsafe {
        let mut sd = PSECURITY_DESCRIPTOR::default();
        let e = GetSecurityInfo(h, SE_KERNEL_OBJECT, DACL_SECURITY_INFORMATION, None, None, None, None, Some(&mut sd));
        if e.0 != 0 { return format!("GetSecurityInfo error {}", e.0); }
        let mut s = PWSTR::null();
        ConvertSecurityDescriptorToStringSecurityDescriptorW(sd, SDDL_REVISION_1, DACL_SECURITY_INFORMATION, &mut s, None).unwrap();
        let r = s.to_string().unwrap();
        LocalFree(Some(HLOCAL(s.0 as _)));
        LocalFree(Some(HLOCAL(sd.0 as _)));
        r
    }
}

/// spawn `exe serve name mode`; returns child, first line, and a channel with the rest of stdout
fn spawn_server(exe: &Path, name: &str, mode: &str) -> (Child, String, mpsc::Receiver<String>) {
    let mut ch = Command::new(exe).args(["serve", name, mode]).stdout(Stdio::piped()).spawn().unwrap();
    let mut lines = BufReader::new(ch.stdout.take().unwrap()).lines();
    let first = lines.next().and_then(|l| l.ok()).unwrap_or_default();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || for l in lines.map_while(Result::ok) { let _ = tx.send(l); });
    (ch, first, rx)
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() == 4 && a[1] == "serve" { serve(&a[2], a[3] == "genuine") }

    let me = std::env::current_exe().unwrap();
    let mut c = Check::new("S3 named pipe DACL + server verification");
    let ping = br#"{"cmd":"ping","cookie":"sid=SECRET"}"#;
    let pid = std::process::id();

    // A. genuine server
    let na = format!("gdm-s3-{pid}-a");
    let (mut g, first, _) = spawn_server(&me, &na, "genuine");
    c.ok(first == "READY", &format!("genuine server created the pipe ({first})"));
    let r = verified_call(&na, &me, ping);
    c.ok(r.as_deref().map_or(false, |s| s.contains(r#""ok":true"#)), &format!("client verified the server image and got a reply ({r:?})"));
    c.ok(r.as_deref().map_or(false, |s| s.contains(r#""imp":1"#)), "server sees the client token at SecurityIdentification only (SQOS): it cannot act as the client");
    let sddl = open(&pipe(&na), GENERIC_READ).map(|h| { let s = dacl_sddl(h); unsafe { let _ = CloseHandle(h); } s });
    let want = format!("D:P(A;;FA;;;{})", user_sid());
    c.ok(sddl.as_deref() == Ok(want.as_str()), &format!("DACL read back = protected, one ACE for the current user only ({sddl:?})"));
    for who in [Other::Anonymous, Other::UserSidDenyOnly] {
        let p = pipe(&na);
        let e = as_other(who, move || open(&p, GENERIC_READ).map(|h| unsafe { let _ = CloseHandle(h); }));
        c.ok(e == Err(5), &format!("{who:?} principal cannot open the pipe (ERROR_ACCESS_DENIED) -> {e:?}"));
        let n = na.clone();
        let e = as_other(who, move || create_pipe(&n, false).map(|h| unsafe { let _ = CloseHandle(h); }));
        c.ok(e.is_err(), &format!("{who:?} principal cannot create a second instance while the server runs -> {e:?}"));
    }
    let e = create_pipe(&na, false).map(|h| unsafe { let _ = CloseHandle(h); });
    c.ok(e.is_err(), &format!("same-user second instance while the server runs fails (max instances 1) -> {e:?}"));
    let big = vec![b' '; 100 * 1024];
    let r = verified_call(&na, &me, &big);
    c.ok(!r.as_deref().unwrap_or("").contains(r#""ok":true"#), &format!("100 KiB message (> 64 KiB) rejected ({})", r.as_deref().unwrap_or_else(|e| e)));
    let deep = format!("{}{}", "[".repeat(5000), "]".repeat(5000));
    let r = verified_call(&na, &me, deep.as_bytes());
    c.ok(r.as_deref().map_or(false, |s| s.contains("recursion limit")), &format!("5000-deep JSON (10 KB) rejected by the parser depth limit ({r:?})"));
    let r = verified_call(&na, &me, ping);
    c.ok(r.as_deref().map_or(false, |s| s.contains(r#""ok":true"#)), "server still serves after hostile messages");
    let remote = open(&format!(r"\\127.0.0.1\pipe\{na}"), GENERIC_RW).map(|h| unsafe { let _ = CloseHandle(h); });
    c.ok(remote.is_err(), &format!("remote path \\\\127.0.0.1\\pipe\\… refused (PIPE_REJECT_REMOTE_CLIENTS / SMB) -> {remote:?}"));
    let _ = g.kill();
    let _ = g.wait();

    // B. squatter: a copy of this exe in another folder grabs the name first
    let dir = std::env::temp_dir().join(format!("gdm-s3-evil-{pid}"));
    std::fs::create_dir_all(&dir).unwrap();
    let evil = dir.join("s3squat.exe");
    std::fs::copy(&me, &evil).unwrap();
    let nb = format!("gdm-s3-{pid}-b");
    let (mut s, first, got) = spawn_server(&evil, &nb, "squat");
    c.ok(first == "READY", "squatter (default security) owns the name first");
    for who in [Other::Anonymous, Other::UserSidDenyOnly] {
        let p = pipe(&nb);
        let e = as_other(who, move || open(&p, GENERIC_READ).map(|h| unsafe { let _ = CloseHandle(h); }));
        c.ok(e.is_ok(), &format!("control: {who:?} CAN open the squatter's default-DACL pipe, so the denials above are real -> {e:?}"));
    }
    let remote = open(&format!(r"\\127.0.0.1\pipe\{nb}"), GENERIC_RW).map(|h| unsafe { let _ = CloseHandle(h); });
    c.note(&format!("control for the remote test: squatter pipe (no reject flag) via \\\\127.0.0.1 -> {remote:?}"));
    let (mut g2, first, _) = spawn_server(&me, &nb, "genuine");
    let code = g2.wait().unwrap().code();
    c.ok(first.starts_with("FAILCLOSED") && code == Some(3), &format!("genuine server fails closed when the name is squatted ({first}, exit {code:?})"));
    let rounds = 50;
    let refused = (0..rounds).filter(|_| verified_call(&nb, &me, ping).map_or_else(|e| e.starts_with("refused"), |_| false)).count();
    std::thread::sleep(Duration::from_millis(300));
    let stolen: Vec<String> = got.try_iter().collect();
    c.ok(refused == rounds && stolen.is_empty(), &format!("client refused the squatter {refused}/{rounds} times, squatter received {} messages", stolen.len()));
    let h = open(&pipe(&nb), GENERIC_RW).unwrap();
    let _ = exchange(h, b"{\"canary\":1}");
    unsafe { let _ = CloseHandle(h); }
    let saw = got.recv_timeout(Duration::from_secs(2)).unwrap_or_default();
    c.ok(saw.contains("canary"), &format!("control: an UNverified client does leak to the squatter ({saw}), so verification is what protects"));
    let _ = s.kill();
    let _ = s.wait();
    let _ = std::fs::remove_dir_all(&dir);
    c.note("same-user process that edits the per-user install is out of scope (threat model). Real second account / other session not tested (needs admin).");
    c.note("pre-SignPath fallback = image path check (install dir %LOCALAPPDATA%\\Programs\\…, folder ACL); after SignPath add an Authenticode signer check on the held handle");
    c.finish();
}
