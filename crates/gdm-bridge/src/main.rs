// SPDX-License-Identifier: RPL-1.5
//! gdm-bridge.exe: the native messaging host. Stateless: frame in -> validate -> verified pipe -> frame out.
//!   gdm-bridge <origin>          host mode (started by the browser)
//!   gdm-bridge register          write the host manifest for every known browser (Windows: HKCU, B-07, NSIS
//!                                calls it; macOS/Linux: each installed browser's NativeMessagingHosts folder)
//!   gdm-bridge unregister        remove them again (T-03)
use gdm_bridge::*;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::time::Duration;
#[cfg(windows)]
use windows::core::HSTRING;

fn frame_out(out: &mut impl Write, v: &[u8]) -> io::Result<()> {
    out.write_all(&(v.len() as u32).to_le_bytes())?;
    out.write_all(v)?;
    out.flush()
}

fn gdm_exe() -> PathBuf {
    let name = if cfg!(windows) { "gdm.exe" } else { "gdm" };
    std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join(name))).unwrap_or_default()
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
                    // never hand GDM our stdio: stdout is the browser's native messaging channel
                    let _ = std::process::Command::new(&exe)
                        .arg("--background")
                        .stdin(std::process::Stdio::null())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn();
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

/// the host manifest a browser reads: Chromium family (allowed_origins) or Firefox family (allowed_extensions)
fn manifest(firefox: bool) -> Result<Vec<u8>, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut m = serde_json::json!({"name": HOST, "description": "Galaxy Download Manager", "path": exe, "type": "stdio"});
    if firefox {
        m["allowed_extensions"] = serde_json::json!(FIREFOX_IDS);
    } else {
        m["allowed_origins"] = chrome_ids().map(|i| format!("chrome-extension://{i}/")).collect();
    }
    serde_json::to_vec_pretty(&m).map_err(|e| e.to_string())
}

// ---- per-user registration (no admin) ----
/// (HKCU key, Firefox-style manifest). Chromium forks that read Chrome's key are covered by it (B-07).
#[cfg(windows)]
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

#[cfg(windows)]
fn nmh_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_default()).join(r"GDM\nmh")
}

#[cfg(windows)]
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

#[cfg(windows)]
fn reg_delete(key: &str) -> bool {
    use windows::Win32::System::Registry::*;
    unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, &HSTRING::from(key)).is_ok() && RegDeleteKeyW(HKEY_CURRENT_USER, &HSTRING::from(key)).is_ok() }
}

#[cfg(windows)]
fn register() -> Result<(), String> {
    let dir = nmh_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for (ff, file) in [(false, "chromium.json"), (true, "firefox.json")] {
        std::fs::write(dir.join(file), manifest(ff)?).map_err(|e| e.to_string())?;
    }
    for (key, ff) in KEYS {
        let file = dir.join(if *ff { "firefox.json" } else { "chromium.json" });
        reg_set_default(&format!(r"{key}\{HOST}"), &file.display().to_string())?;
    }
    println!("registered {} browser keys -> {}", KEYS.len(), dir.display());
    Ok(())
}

#[cfg(windows)]
fn unregister() {
    let n = KEYS.iter().filter(|(k, _)| reg_delete(&format!(r"{k}\{HOST}"))).count();
    let _ = std::fs::remove_dir_all(nmh_dir());
    println!("unregistered ({n} keys removed)");
}

/// (browser profile root under $HOME, its NativeMessagingHosts folder, Firefox-style manifest). A manifest is
/// written only for a browser whose profile root exists (an installed, used browser).
#[cfg(unix)]
fn unix_hosts() -> Vec<(PathBuf, PathBuf, bool)> {
    let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    #[cfg(target_vendor = "apple")]
    let list: &[(&str, &str, bool)] = &[
        ("Library/Application Support/Google/Chrome", "NativeMessagingHosts", false),
        ("Library/Application Support/Google/Chrome Beta", "NativeMessagingHosts", false),
        ("Library/Application Support/Chromium", "NativeMessagingHosts", false),
        ("Library/Application Support/Microsoft Edge", "NativeMessagingHosts", false),
        ("Library/Application Support/BraveSoftware/Brave-Browser", "NativeMessagingHosts", false),
        ("Library/Application Support/Vivaldi", "NativeMessagingHosts", false),
        ("Library/Application Support/Arc/User Data", "NativeMessagingHosts", false),
        ("Library/Application Support/Mozilla", "NativeMessagingHosts", true),
        ("Library/Application Support/LibreWolf", "NativeMessagingHosts", true),
    ];
    #[cfg(not(target_vendor = "apple"))]
    let list: &[(&str, &str, bool)] = &[
        (".config/google-chrome", "NativeMessagingHosts", false),
        (".config/google-chrome-beta", "NativeMessagingHosts", false),
        (".config/chromium", "NativeMessagingHosts", false),
        (".config/microsoft-edge", "NativeMessagingHosts", false),
        (".config/BraveSoftware/Brave-Browser", "NativeMessagingHosts", false),
        (".config/vivaldi", "NativeMessagingHosts", false),
        (".config/opera", "NativeMessagingHosts", false),
        (".mozilla", "native-messaging-hosts", true),
        (".librewolf", "native-messaging-hosts", true),
        (".waterfox", "native-messaging-hosts", true),
        (".floorp", "native-messaging-hosts", true),
    ];
    list.iter().map(|(root, sub, ff)| (home.join(root), home.join(root).join(sub), *ff)).collect()
}

#[cfg(unix)]
fn register() -> Result<(), String> {
    // a path that will not exist next time must never be written into a browser: an AppImage runs from a
    // temporary mount (/tmp/.mount_*), and macOS runs an app opened from Downloads / the disk image from a
    // random App Translocation path. Browser integration needs the installed .deb / .rpm, or the app in
    // /Applications.
    let exe = std::env::current_exe().map_err(|e| e.to_string())?.display().to_string();
    if std::env::var_os("APPIMAGE").is_some() || exe.contains("/.mount_") || exe.contains("/AppTranslocation/") {
        return Err(format!("not registered: {exe} is a temporary location (install the package / move GDM to Applications)"));
    }
    let mut n = 0;
    for (root, dir, ff) in unix_hosts() {
        if !root.is_dir() {
            continue;
        }
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        std::fs::write(dir.join(format!("{HOST}.json")), manifest(ff)?).map_err(|e| format!("{}: {e}", dir.display()))?;
        n += 1;
    }
    println!("registered {n} browsers");
    Ok(())
}

#[cfg(unix)]
fn unregister() {
    let n = unix_hosts().iter().filter(|(_, dir, _)| std::fs::remove_file(dir.join(format!("{HOST}.json"))).is_ok()).count();
    println!("unregistered ({n} manifests removed)");
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

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Storage::FileSystem::{FlushFileBuffers, ReadFile, WriteFile, PIPE_ACCESS_DUPLEX};
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
