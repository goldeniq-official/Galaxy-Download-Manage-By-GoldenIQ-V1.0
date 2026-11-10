// SPDX-License-Identifier: RPL-1.5
//! Windows shell actions without the shell plugin (X-02, CVE-2025-31477): every path comes from the database
//! (looked up by job id), every URL from the fixed table below. Nothing the WebView sends is ever executed.
use std::path::{Path, PathBuf};
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// links the UI may open, by key only
pub fn link(key: &str) -> Option<&'static str> {
    Some(match key {
        "store-chrome" => "https://chromewebstore.google.com/search/Galaxy%20Download%20Manager",
        "store-edge" => "https://microsoftedge.microsoft.com/addons/search/Galaxy%20Download%20Manager",
        "store-firefox" => "https://addons.mozilla.org/firefox/search/?q=Galaxy%20Download%20Manager",
        "github" => "https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0",
        "releases" => "https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0/releases",
        "license" => "https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0/blob/main/LICENSE",
        "privacy" => "https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0/blob/main/PRIVACY.md",
        "windows-security" => "windowsdefender://threat/",
        _ => return None,
    })
}

fn shell_open(target: &str, verb: &str) -> Result<(), String> {
    let r = unsafe { ShellExecuteW(None, &HSTRING::from(verb), &HSTRING::from(target), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL) };
    if r.0 as isize > 32 {
        Ok(())
    } else {
        Err(format!("shell {}", r.0 as isize))
    }
}

pub fn open_link(key: &str) -> Result<(), String> {
    shell_open(link(key).ok_or("unknown link")?, "open")
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
    shell_open(&path.display().to_string(), "open")
}

/// Explorer with the file selected (falls back to the folder)
pub fn open_folder(path: &Path) -> Result<(), String> {
    let mut c = std::process::Command::new("explorer.exe");
    if path.is_file() {
        c.arg(format!("/select,{}", path.display()));
    } else {
        c.arg(path.parent().unwrap_or(path));
    }
    c.spawn().map(|_| ()).map_err(|e| e.to_string())
}

/// browsers GDM knows (J2.2): (key, display name, exe candidates under the usual roots)
pub fn browsers() -> Vec<(&'static str, &'static str, bool)> {
    let roots: Vec<PathBuf> =
        ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"].iter().filter_map(|v| std::env::var_os(v).map(PathBuf::from)).collect();
    let has = |rel: &[&str]| roots.iter().any(|r| rel.iter().any(|p| r.join(p).is_file()));
    vec![
        ("chrome", "Chrome", has(&[r"Google\Chrome\Application\chrome.exe"])),
        ("edge", "Edge", has(&[r"Microsoft\Edge\Application\msedge.exe"])),
        ("firefox", "Firefox", has(&[r"Mozilla Firefox\firefox.exe"])),
        ("brave", "Brave", has(&[r"BraveSoftware\Brave-Browser\Application\brave.exe"])),
        ("vivaldi", "Vivaldi", has(&[r"Vivaldi\Application\vivaldi.exe"])),
        ("opera", "Opera", has(&[r"Programs\Opera\opera.exe", r"Opera\opera.exe"])),
    ]
}

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

fn system32(exe: &str) -> PathBuf {
    std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows")).join("System32").join(exe)
}

/// queue finish actions (Q-02). shutdown: `/t 0` so `/f` is NOT implied (shutdown.exe implies /f only for
/// /t > 0); `/f` only when the user chose "force processes to terminate". Our own 60 s countdown came first.
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

/// Windows says the current internet connection is metered (Q-05: sync waits for an unmetered one)
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
