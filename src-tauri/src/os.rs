// SPDX-License-Identifier: RPL-1.5
//! Shell actions without the shell plugin (X-02, CVE-2025-31477): every path comes from the database (looked
//! up by job id), every URL from the fixed table below. Nothing the WebView sends is ever executed.
//! Windows: ShellExecute, IFileDialog, MessageBox, shutdown.exe, Task Scheduler. macOS / Linux (P4): `open` /
//! `xdg-open` with the target as ONE argument (never a shell), native dialogs through rfd from Rust, logind /
//! pmset for power; waking the machine needs root there, so scheduled wake is Windows-only.
use std::path::{Path, PathBuf};
#[cfg(windows)]
use windows::core::{HSTRING, PCWSTR};
#[cfg(windows)]
use windows::Win32::UI::Shell::ShellExecuteW;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// links the UI may open, by key only. Store links go to GDM's own listing once its id is set (gdm-bridge).
pub fn link(key: &str) -> Option<String> {
    if let Some(u) = gdm_bridge::browsers::store_url(key) {
        return Some(u);
    }
    Some(match key {
        "github" => "https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0".into(),
        "releases" => "https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0/releases".into(),
        "license" => "https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0/blob/main/LICENSE".into(),
        "privacy" => "https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0/blob/main/PRIVACY.md".into(),
        #[cfg(windows)]
        "windows-security" => "windowsdefender://threat/".into(),
        _ => return None,
    })
}

#[cfg(unix)]
fn shell_open(target: &str, _verb: &str) -> Result<(), String> {
    use std::process::Stdio;
    let prog = if cfg!(target_vendor = "apple") { "open" } else { "xdg-open" };
    std::process::Command::new(prog)
        .arg(target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("{prog}: {e}"))
}

#[cfg(windows)]
fn shell_open(target: &str, verb: &str) -> Result<(), String> {
    let r = unsafe { ShellExecuteW(None, &HSTRING::from(verb), &HSTRING::from(target), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL) };
    if r.0 as isize > 32 {
        Ok(())
    } else {
        Err(format!("shell {}", r.0 as isize))
    }
}

pub fn open_link(key: &str) -> Result<(), String> {
    shell_open(&link(key).ok_or("unknown link")?, "open")
}

/// a job's source page (J5 refresh, "login needed"): from the database, http(s) only, never a file: or custom scheme
pub fn open_web(url: &str) -> Result<(), String> {
    let u = url::Url::parse(url).map_err(|_| "bad-url")?;
    if !matches!(u.scheme(), "http" | "https") {
        return Err("bad-url".into());
    }
    shell_open(u.as_str(), "open")
}

/// open a finished download with its associated program. Dangerous types are never opened directly (N-02):
/// the UI shows only [open folder] for them, and this refuses even if asked.
pub fn open_file(path: &Path, name: &str) -> Result<(), String> {
    if gdm_core::name::dangerous(name) {
        return Err("dangerous".into());
    }
    if !path.is_file() {
        return Err("missing".into());
    }
    // default deny on macOS / Linux: whatever its name, a file with an execute bit is never launched from GDM
    #[cfg(unix)]
    if std::os::unix::fs::PermissionsExt::mode(&std::fs::metadata(path).map_err(|e| e.to_string())?.permissions()) & 0o111 != 0 {
        return Err("dangerous".into());
    }
    shell_open(&path.display().to_string(), "open")
}

/// the file manager showing the file (macOS: selected in Finder; Linux: its folder)
#[cfg(unix)]
pub fn open_folder(path: &Path) -> Result<(), String> {
    if cfg!(target_vendor = "apple") && path.is_file() {
        return std::process::Command::new("open").arg("-R").arg(path).spawn().map(|_| ()).map_err(|e| e.to_string());
    }
    let dir = if path.is_file() { path.parent().unwrap_or(path) } else { path };
    shell_open(&dir.display().to_string(), "open")
}

/// Explorer with the file selected (falls back to the folder)
#[cfg(windows)]
pub fn open_folder(path: &Path) -> Result<(), String> {
    let mut c = std::process::Command::new(gdm_bridge::browsers::explorer());
    if path.is_file() {
        c.arg(format!("/select,{}", path.display()));
    } else {
        c.arg(path.parent().unwrap_or(path));
    }
    c.spawn().map(|_| ()).map_err(|e| e.to_string())
}

pub use gdm_bridge::browsers::browsers;

/// volume checks for the download folder (onboarding step 3, plan-ux §4): filesystem and free bytes
pub fn folder_check(dir: &Path) -> (String, u64) {
    let fs = std::fs::File::open(dir).map(|f| gdm_core::fsx::fs_name(&f)).unwrap_or_default();
    (fs, gdm_core::fsx::free_space(dir))
}

/// the native folder picker (IFileOpenDialog, FOS_PICKFOLDERS) on its own STA thread
pub fn pick_folder(start: &Path) -> Option<PathBuf> {
    pick(start, Pick::Folder)
}

/// a file chosen by the user in the native dialog: the only way a program, WAV or import path enters GDM (X-01)
pub fn pick_file(start: &Path, label: &str, pattern: &str) -> Option<PathBuf> {
    pick(start, Pick::Open(label.to_string(), pattern.to_string()))
}

pub fn pick_save(start: &Path, name: &str) -> Option<PathBuf> {
    pick(start, Pick::Save(name.to_string()))
}

enum Pick {
    Folder,
    Open(String, String),
    Save(String),
}

/// rfd's async dialogs hand themselves to the main thread on macOS (its sync ones must not run elsewhere); run on
/// a plain thread so no async runtime is nested
#[cfg(unix)]
fn pick(start: &Path, kind: Pick) -> Option<PathBuf> {
    let start = start.to_path_buf();
    std::thread::spawn(move || {
        tauri::async_runtime::block_on(async move {
            let mut d = rfd::AsyncFileDialog::new().set_directory(&start);
            if let Pick::Open(l, p) = &kind {
                let exts: Vec<String> =
                    p.split(';').map(|x| x.trim().trim_start_matches("*.").to_string()).filter(|x| !x.is_empty() && x != "*").collect();
                if !exts.is_empty() {
                    d = d.add_filter(l.as_str(), &exts);
                }
            }
            let h = match &kind {
                Pick::Folder => d.pick_folder().await,
                Pick::Open(..) => d.pick_file().await,
                Pick::Save(n) => d.set_file_name(n.as_str()).save_file().await,
            };
            h.map(|h| h.path().to_path_buf())
        })
    })
    .join()
    .ok()
    .flatten()
}

#[cfg(windows)]
fn pick(start: &Path, kind: Pick) -> Option<PathBuf> {
    let start = start.to_path_buf();
    std::thread::spawn(move || unsafe {
        use windows::Win32::System::Com::*;
        use windows::Win32::UI::Shell::Common::COMDLG_FILTERSPEC;
        use windows::Win32::UI::Shell::*;
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().ok()?;
        let r = (|| -> windows::core::Result<PathBuf> {
            let d: IFileDialog = match &kind {
                Pick::Save(_) => CoCreateInstance::<_, IFileSaveDialog>(&FileSaveDialog, None, CLSCTX_INPROC_SERVER)?.into(),
                _ => CoCreateInstance::<_, IFileOpenDialog>(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?.into(),
            };
            let mut opts = d.GetOptions()? | FOS_FORCEFILESYSTEM;
            if matches!(kind, Pick::Folder) {
                opts |= FOS_PICKFOLDERS;
            }
            if matches!(kind, Pick::Open(..)) {
                opts |= FOS_FILEMUSTEXIST;
            }
            d.SetOptions(opts)?;
            let (l, p) = match &kind {
                Pick::Open(l, p) => (HSTRING::from(l.as_str()), HSTRING::from(p.as_str())),
                _ => (HSTRING::new(), HSTRING::new()),
            };
            if !p.is_empty() {
                d.SetFileTypes(&[COMDLG_FILTERSPEC { pszName: PCWSTR(l.as_ptr()), pszSpec: PCWSTR(p.as_ptr()) }])?;
            }
            if let Pick::Save(n) = &kind {
                d.SetFileName(&HSTRING::from(n.as_str()))?;
            }
            if let Ok(item) = SHCreateItemFromParsingName::<_, _, IShellItem>(&HSTRING::from(start.as_os_str()), None) {
                let _ = d.SetFolder(&item);
            }
            d.Show(None)?;
            let p = d.GetResult()?.GetDisplayName(SIGDN_FILESYSPATH)?;
            let s = p.to_string().unwrap_or_default();
            CoTaskMemFree(Some(p.0 as _));
            Ok(PathBuf::from(s))
        })();
        CoUninitialize();
        r.ok()
    })
    .join()
    .ok()
    .flatten()
}

#[cfg(windows)]
fn system32(exe: &str) -> PathBuf {
    std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows")).join("System32").join(exe)
}

/// a native Yes/No box outside the WebView: what a compromised page cannot click for the user
#[cfg(unix)]
pub fn confirm(title: &str, text: &str) -> bool {
    let (title, text) = (title.to_string(), text.to_string());
    std::thread::spawn(move || {
        tauri::async_runtime::block_on(
            rfd::AsyncMessageDialog::new()
                .set_title(title.as_str())
                .set_description(text.as_str())
                .set_level(rfd::MessageLevel::Warning)
                .set_buttons(rfd::MessageButtons::YesNo)
                .show(),
        ) == rfd::MessageDialogResult::Yes
    })
    .join()
    .unwrap_or(false)
}

/// a native Yes/No box (default No) outside the WebView: what a compromised page cannot click for the user
#[cfg(windows)]
pub fn confirm(title: &str, text: &str) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, IDYES, MB_DEFBUTTON2, MB_ICONWARNING, MB_SETFOREGROUND, MB_YESNO};
    unsafe { MessageBoxW(None, &HSTRING::from(text), &HSTRING::from(title), MB_YESNO | MB_ICONWARNING | MB_DEFBUTTON2 | MB_SETFOREGROUND) == IDYES }
}

/// queue finish actions (Q-02) on macOS / Linux: logind (`systemctl`, allowed for the active session) or the
/// macOS ways that need no root (System Events asks apps to quit; `pmset sleepnow`)
#[cfg(unix)]
pub fn power(action: &str, force: bool) -> Result<(), String> {
    let mac = cfg!(target_vendor = "apple");
    let (prog, args): (&str, Vec<&str>) = match (action, mac) {
        ("shutdown", true) => ("osascript", vec!["-e", "tell application \"System Events\" to shut down"]),
        ("sleep", true) => ("pmset", vec!["sleepnow"]),
        ("hibernate", true) => return Err("hibernate is not offered on macOS".into()),
        ("shutdown", false) => ("systemctl", if force { vec!["poweroff", "-i"] } else { vec!["poweroff"] }),
        ("sleep", false) => ("systemctl", vec!["suspend"]),
        ("hibernate", false) => ("systemctl", vec!["hibernate"]),
        _ => return Err("unknown action".into()),
    };
    let st = std::process::Command::new(prog).args(&args).status().map_err(|e| e.to_string())?;
    st.success().then_some(()).ok_or(format!("{prog} {st}"))
}

/// queue finish actions (Q-02). shutdown: `/t 0` so `/f` is NOT implied (shutdown.exe implies /f only for
/// /t > 0); `/f` only when the user chose "force processes to terminate". Our own 60 s countdown came first.
#[cfg(windows)]
pub fn power(action: &str, force: bool) -> Result<(), String> {
    match action {
        "shutdown" => {
            let mut c = std::process::Command::new(system32("shutdown.exe"));
            c.args(["/s", "/t", "0"]);
            if force {
                c.arg("/f");
            }
            let st = c.status().map_err(|e| e.to_string())?;
            st.success().then_some(()).ok_or(format!("shutdown.exe {st}"))
        }
        "sleep" | "hibernate" => unsafe {
            windows::Win32::System::Power::SetSuspendState(action == "hibernate", false, false)
                .then_some(())
                .ok_or_else(|| windows::core::Error::from_win32().message())
        },
        _ => Err("unknown action".into()),
    }
}

/// the connection is metered (Q-05): Linux asks NetworkManager (1 = yes, 3 = guessed yes); macOS has no public
/// signal, so it counts as unmetered
#[cfg(unix)]
pub fn metered() -> bool {
    if cfg!(target_vendor = "apple") {
        return false;
    }
    std::process::Command::new("busctl")
        .args(["get-property", "org.freedesktop.NetworkManager", "/org/freedesktop/NetworkManager", "org.freedesktop.NetworkManager", "Metered"])
        .output()
        .ok()
        .is_some_and(|o| matches!(String::from_utf8_lossy(&o.stdout).trim(), "u 1" | "u 3"))
}

/// Windows says the current internet connection is metered (Q-05: sync waits for an unmetered one)
#[cfg(windows)]
pub fn metered() -> bool {
    use windows::Networking::Connectivity::{NetworkCostType, NetworkInformation};
    (|| -> windows::core::Result<bool> {
        let cost = NetworkInformation::GetInternetConnectionProfile()?.GetConnectionCost()?;
        let t = cost.NetworkCostType()?;
        Ok(t == NetworkCostType::Fixed || t == NetworkCostType::Variable || cost.OverDataLimit()? || cost.Roaming()?)
    })()
    .unwrap_or(false)
}

/// event sounds (§04 row "sounds"): "system:<alias>" = one of the built-in Windows sounds, else a WAV the user
/// picked in the file dialog. Asynchronous, never blocks a download.
pub const SYSTEM_SOUNDS: [&str; 3] = ["SystemAsterisk", "SystemExclamation", "SystemNotification"];

/// macOS: afplay (system sounds from /System/Library/Sounds); Linux: canberra (theme sounds) / paplay (WAV)
#[cfg(unix)]
pub fn play(sound: &str) {
    use std::process::Stdio;
    let mac = cfg!(target_vendor = "apple");
    let (prog, arg): (&str, Vec<String>) = if let Some(a) = sound.strip_prefix("system:") {
        let i = SYSTEM_SOUNDS.iter().position(|s| *s == a);
        let Some(i) = i else { return };
        if mac {
            ("afplay", vec![format!("/System/Library/Sounds/{}.aiff", ["Glass", "Basso", "Ping"][i])])
        } else {
            ("canberra-gtk-play", vec!["-i".into(), ["dialog-information", "dialog-warning", "message-new-instant"][i].into()])
        }
    } else if sound.to_ascii_lowercase().ends_with(".wav") && Path::new(sound).is_file() {
        (if mac { "afplay" } else { "paplay" }, vec![sound.to_string()])
    } else {
        return;
    };
    let _ = std::process::Command::new(prog).args(arg).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
}

#[cfg(windows)]
pub fn play(sound: &str) {
    use windows::Win32::Media::Audio::{PlaySoundW, SND_ALIAS, SND_ASYNC, SND_FILENAME, SND_NODEFAULT};
    if sound.is_empty() {
        return;
    }
    unsafe {
        if let Some(a) = sound.strip_prefix("system:") {
            if SYSTEM_SOUNDS.contains(&a) {
                let _ = PlaySoundW(&HSTRING::from(a), None, SND_ALIAS | SND_ASYNC | SND_NODEFAULT);
            }
        } else if sound.to_ascii_lowercase().ends_with(".wav") && Path::new(sound).is_file() {
            let _ = PlaySoundW(&HSTRING::from(sound), None, SND_FILENAME | SND_ASYNC | SND_NODEFAULT);
        }
    }
}

/// Q-04 "wake to download": one Task Scheduler task per queue with `wake` on, run as the user (not elevated),
/// WakeToRun + AC power only (never on battery); its action starts `gdm.exe --background`, whose scheduler then
/// starts the queue. Off by default; a queue without wake (or without a timed start) has no task.
pub fn wake_task_xml(q: &crate::queue::Queue, exe: &Path) -> Option<String> {
    let sch = &q.schedule;
    if !q.wake || !sch.enabled {
        return None;
    }
    let t = gdm_core::sched::hm(sch.start.as_deref()?)?;
    let today = chrono::Local::now().date_naive();
    let (date, rule) = if sch.days == 0 {
        (chrono::NaiveDate::parse_from_str(sch.date.as_deref()?, "%Y-%m-%d").ok()?, String::new())
    } else {
        let names = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
        let days: String = (0..7).filter(|i| sch.days & (1 << i) != 0).map(|i| format!("<{}/>", names[i])).collect();
        (today, format!("<ScheduleByWeek><DaysOfWeek>{days}</DaysOfWeek><WeeksInterval>1</WeeksInterval></ScheduleByWeek>"))
    };
    let start = date.and_time(t).format("%Y-%m-%dT%H:%M:%S");
    let trigger = if rule.is_empty() {
        format!("<TimeTrigger><StartBoundary>{start}</StartBoundary><Enabled>true</Enabled></TimeTrigger>")
    } else {
        format!("<CalendarTrigger><StartBoundary>{start}</StartBoundary><Enabled>true</Enabled>{rule}</CalendarTrigger>")
    };
    let exe = exe.display().to_string().replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    Some(format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo><Description>Galaxy Download Manager: wake for a scheduled queue</Description></RegistrationInfo>
  <Triggers>{trigger}</Triggers>
  <Principals><Principal id="Author"><LogonType>InteractiveToken</LogonType><RunLevel>LeastPrivilege</RunLevel></Principal></Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>true</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>true</StopIfGoingOnBatteries>
    <WakeToRun>true</WakeToRun>
    <StartWhenAvailable>false</StartWhenAvailable>
    <ExecutionTimeLimit>PT5M</ExecutionTimeLimit>
    <Enabled>true</Enabled>
  </Settings>
  <Actions Context="Author"><Exec><Command>{exe}</Command><Arguments>--background</Arguments></Exec></Actions>
</Task>"#
    ))
}

pub fn wake_task_name(id: &str) -> String {
    format!(r"\GoldenIQ\GDM wake {id}")
}

/// create, replace or delete the wake tasks (in the background; failures are not fatal: the queue still runs
/// on time when the PC is awake)
#[cfg(unix)]
pub fn sync_wake_tasks(_queues: &[crate::queue::Queue]) {}

#[cfg(windows)]
pub fn sync_wake_tasks(queues: &[crate::queue::Queue]) {
    let Ok(exe) = std::env::current_exe() else { return };
    let jobs: Vec<(String, Option<String>)> = queues.iter().map(|q| (wake_task_name(&q.id), wake_task_xml(q, &exe))).collect();
    std::thread::spawn(move || {
        let schtasks = system32("schtasks.exe");
        for (name, xml) in jobs {
            match xml {
                Some(x) => {
                    let f = std::env::temp_dir().join(format!("gdm-wake-{}.xml", std::process::id()));
                    let utf16: Vec<u8> = std::iter::once(0xFEFFu16).chain(x.encode_utf16()).flat_map(|u| u.to_le_bytes()).collect();
                    if std::fs::write(&f, utf16).is_ok() {
                        let _ = std::process::Command::new(&schtasks).args(["/Create", "/TN", &name, "/XML"]).arg(&f).arg("/F").output();
                        let _ = std::fs::remove_file(&f);
                    }
                }
                None => {
                    let _ = std::process::Command::new(&schtasks).args(["/Delete", "/TN", &name, "/F"]).output();
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    /// X-02: nothing the WebView sends becomes a file:, UNC, smb: or custom-scheme launch
    #[test]
    fn x02_only_fixed_links_and_http_pages() {
        assert!(link("store-chrome").is_some());
        assert!(link("file:///C:/Windows/System32/calc.exe").is_none());
        assert!(link("evil").is_none());
        for bad in ["file:///C:/Windows/System32/calc.exe", r"\\server\share\x.exe", "smb://server/share", "ms-msdt:/id", "javascript:alert(1)"] {
            assert!(open_web(bad).is_err(), "{bad}");
        }
        assert_eq!(open_file(Path::new("C:/Windows/System32/calc.exe"), "calc.exe"), Err("dangerous".into()), "dangerous types never open directly");
        assert_eq!(open_file(Path::new("C:/definitely/missing.pdf"), "missing.pdf"), Err("missing".into()));
    }

    /// Q-04: wake is off by default; when on, the task wakes only on AC power and runs not elevated
    #[test]
    fn q04_wake_task_ac_only_not_elevated() {
        let mut q = crate::queue::Queue { id: "main".into(), ..Default::default() };
        assert!(wake_task_xml(&q, Path::new("C:/x/gdm.exe")).is_none(), "off by default");
        q.wake = true;
        q.schedule = gdm_core::sched::Schedule { enabled: true, start: Some("02:30".into()), days: 0b1000001, ..Default::default() };
        let x = wake_task_xml(&q, Path::new("C:/x/gdm.exe")).unwrap();
        for need in [
            "<WakeToRun>true</WakeToRun>",
            "<DisallowStartIfOnBatteries>true</DisallowStartIfOnBatteries>",
            "<StopIfGoingOnBatteries>true</StopIfGoingOnBatteries>",
            "<RunLevel>LeastPrivilege</RunLevel>",
            "<Monday/><Sunday/>",
            "T02:30:00",
            "<Arguments>--background</Arguments>",
        ] {
            assert!(x.contains(need), "{need}");
        }
    }
}
