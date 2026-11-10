// SPDX-License-Identifier: RPL-1.5
//! gdm-bridge.exe: the native messaging host. Stateless: frame in -> validate -> verified pipe -> frame out.
//!   gdm-bridge <origin>          host mode (started by the browser)
//!   gdm-bridge register          write the host manifest for every known browser (HKCU, B-07; NSIS calls it)
//!   gdm-bridge unregister        remove them again (T-03)
use gdm_bridge::*;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use windows::core::{HSTRING, PWSTR};
use windows::Win32::Foundation::{CloseHandle, ERROR_PIPE_BUSY, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, WriteFile, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_NONE, OPEN_EXISTING, SECURITY_IDENTIFICATION, SECURITY_SQOS_PRESENT,
};
use windows::Win32::System::Pipes::{GetNamedPipeServerProcessId, SetNamedPipeHandleState, WaitNamedPipeW, PIPE_READMODE_MESSAGE};
use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};

const GENERIC_RW: u32 = 0xC000_0000;

struct H(HANDLE);
impl Drop for H {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

fn norm(p: &Path) -> String {
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
        QueryFullProcessImageNameW(ph.0, PROCESS_NAME_WIN32, PWSTR(img.as_mut_ptr()), &mut len).map_err(|e| e.to_string())?;
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

fn frame_out(out: &mut impl Write, v: &[u8]) -> io::Result<()> {
    out.write_all(&(v.len() as u32).to_le_bytes())?;
    out.write_all(v)?;
    out.flush()
}

fn gdm_exe() -> PathBuf {
    std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("gdm.exe"))).unwrap_or_default()
}

fn host(origin: String) -> i32 {
    let (mut inp, mut out) = (io::stdin().lock(), io::stdout().lock());
    let exe = gdm_exe();
    let pipe = pipe_name();
    let mut launched = false;
    loop {
        let mut len = [0u8; 4];
        if inp.read_exact(&mut len).is_err() {
            return 0; // the browser closed the port
        }
        let len = u32::from_le_bytes(len);
        if len > MAX_MSG {
            return 2; // B-03: before allocating
        }
        let mut buf = vec![0u8; len as usize];
        if inp.read_exact(&mut buf).is_err() {
            return 2;
        }
        let reply = match parse(&buf) {
            Err(_) => br#"{"ok":false,"error":"rejected"}"#.to_vec(),
            Ok(msg) => {
                let env = serde_json::to_vec(&Envelope { origin: origin.clone(), msg: msg.clone() }).unwrap_or_default();
                let mut r = call(&pipe, &exe, &env, Duration::from_millis(300));
                if r.as_ref().is_err_and(|e| e.starts_with("no GDM")) && !launched && !matches!(msg, Msg::Status {}) {
                    // B-06: GDM is installed but not running: start it in the tray, then retry (< 3 s ack budget)
                    launched = true;
                    let _ = std::process::Command::new(&exe).arg("--background").spawn();
                    r = call(&pipe, &exe, &env, Duration::from_millis(2500));
                }
                r.unwrap_or_else(|e| {
                    serde_json::to_vec(&serde_json::json!({"ok": false, "error": if e.starts_with("refused") { "untrusted" } else { "offline" }}))
                        .unwrap_or_default()
                })
            }
        };
        if frame_out(&mut out, &reply).is_err() {
            return 2;
        }
    }
}

// ---- per-user registration (HKCU only, no admin) ----
/// (HKCU key, Firefox-style manifest). Chromium forks that read Chrome's key are covered by it (B-07).
const KEYS: &[(&str, bool)] = &[
    (r"Software\Google\Chrome\NativeMessagingHosts", false),
    (r"Software\Chromium\NativeMessagingHosts", false),
    (r"Software\Microsoft\Edge\NativeMessagingHosts", false),
    (r"Software\BraveSoftware\Brave-Browser\NativeMessagingHosts", false),
    (r"Software\Vivaldi\NativeMessagingHosts", false),
    (r"Software\Opera Software\NativeMessagingHosts", false),
    (r"Software\Mozilla\NativeMessagingHosts", true),
    (r"Software\LibreWolf\NativeMessagingHosts", true),
    (r"Software\Waterfox\NativeMessagingHosts", true),
    (r"Software\Floorp\NativeMessagingHosts", true),
];

fn nmh_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_default()).join(r"GDM\nmh")
}

fn reg_set_default(key: &str, value: &str) -> Result<(), String> {
    use windows::Win32::System::Registry::*;
    unsafe {
        let mut hk = HKEY::default();
        RegCreateKeyExW(HKEY_CURRENT_USER, &HSTRING::from(key), None, None, REG_OPTION_NON_VOLATILE, KEY_SET_VALUE, None, &mut hk, None)
            .ok()
            .map_err(|e| e.to_string())?;
        let w: Vec<u16> = value.encode_utf16().chain([0]).collect();
        let b = std::slice::from_raw_parts(w.as_ptr() as *const u8, w.len() * 2);
        let r = RegSetValueExW(hk, None, None, REG_SZ, Some(b)).ok().map_err(|e| e.to_string());
        let _ = RegCloseKey(hk);
        r
    }
}

fn reg_delete(key: &str) -> bool {
    use windows::Win32::System::Registry::*;
    unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, &HSTRING::from(key)).is_ok() && RegDeleteKeyW(HKEY_CURRENT_USER, &HSTRING::from(key)).is_ok() }
}

fn register() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = nmh_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for (ff, file) in [(false, "chromium.json"), (true, "firefox.json")] {
        let mut m = serde_json::json!({"name": HOST, "description": "Galaxy Download Manager", "path": exe, "type": "stdio"});
        if ff {
            m["allowed_extensions"] = serde_json::json!(FIREFOX_IDS);
        } else {
            m["allowed_origins"] = CHROME_IDS.iter().map(|i| format!("chrome-extension://{i}/")).collect();
        }
        std::fs::write(dir.join(file), serde_json::to_vec_pretty(&m).unwrap_or_default()).map_err(|e| e.to_string())?;
    }
    for (key, ff) in KEYS {
        let file = dir.join(if *ff { "firefox.json" } else { "chromium.json" });
        reg_set_default(&format!(r"{key}\{HOST}"), &file.display().to_string())?;
    }
    println!("registered {} browser keys -> {}", KEYS.len(), dir.display());
    Ok(())
}

fn unregister() {
    let n = KEYS.iter().filter(|(k, _)| reg_delete(&format!(r"{k}\{HOST}"))).count();
    let _ = std::fs::remove_dir_all(nmh_dir());
    println!("unregistered ({n} keys removed)");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("register") => std::process::exit(match register() {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("register failed: {e}");
                1
            }
        }),
        Some("unregister") => unregister(),
        _ => match caller(&args) {
            Some(origin) => std::process::exit(host(origin)),
            None => std::process::exit(1), // B-01: unknown caller, no pipe connect
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Storage::FileSystem::{FlushFileBuffers, PIPE_ACCESS_DUPLEX};
    use windows::Win32::System::Pipes::{
        ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_MESSAGE, PIPE_TYPE_MESSAGE, PIPE_WAIT,
    };

    /// B-02: a squatted pipe (another image) never receives the payload; the genuine image does
    #[test]
    fn b02_bridge_refuses_a_pipe_served_by_another_image() {
        let name = format!(r"\\.\pipe\gdm-test-{}", std::process::id());
        let got = std::sync::Arc::new(std::sync::Mutex::new(Vec::<usize>::new()));
        let (n2, g2) = (name.clone(), got.clone());
        let srv = std::thread::spawn(move || unsafe {
            let h = CreateNamedPipeW(
                &HSTRING::from(n2.as_str()),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                1,
                65536,
                65536,
                0,
                None,
            );
            for _ in 0..2 {
                let _ = ConnectNamedPipe(h, None);
                let mut buf = vec![0u8; 65536];
                let mut n = 0;
                if ReadFile(h, Some(&mut buf), Some(&mut n), None).is_ok() {
                    g2.lock().unwrap().push(n as usize);
                    let _ = WriteFile(h, Some(b"{\"ok\":true}"), None, None);
                    let _ = FlushFileBuffers(h);
                }
                let _ = DisconnectNamedPipe(h);
            }
            let _ = CloseHandle(h);
        });
        std::thread::sleep(Duration::from_millis(200));
        let other = PathBuf::from(r"C:\Program Files\GDM\gdm.exe");
        let e = call(&name, &other, b"secret cookie", Duration::from_secs(2)).unwrap_err();
        assert!(e.starts_with("refused"), "{e}");
        let me = std::env::current_exe().unwrap();
        let r = call(&name, &me, b"hello", Duration::from_secs(2)).unwrap();
        assert_eq!(r, b"{\"ok\":true}");
        srv.join().unwrap();
        assert_eq!(*got.lock().unwrap(), vec![5], "only the verified call reached the server");
    }
}
