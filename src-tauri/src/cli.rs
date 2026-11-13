// SPDX-License-Identifier: RPL-1.5
//! gdm.exe command line (F10, I-02): IDM's flags `/d URL /p dir /f name /q /h /n /a` and `/s`, plus
//! subcommands with JSON output (`add URL [--dir D] [--name N] [--later] [--silent]`, `list`, `start-queue`).
//! A CLI call is a client of the same per-user pipe the browser bridge uses: it checks the server image is
//! this gdm.exe before sending, and the server accepts a CLI message only from a client whose image is this
//! gdm.exe. That proves the message came through gdm.exe, not WHO ran it (any same-user process can run
//! gdm.exe): so the CLI cannot set hooks or settings, and `/p` refuses dangerous folders.
use crate::app::{lock, App};
use gdm_bridge::{Envelope, Msg};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

/// parsed command line; None = not a CLI call (start the app normally)
pub fn parse(args: &[String]) -> Option<Msg> {
    let a: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();
    let flag = |f: &str| a.iter().any(|x| x.eq_ignore_ascii_case(f));
    let val = |f: &str| a.iter().position(|x| x.eq_ignore_ascii_case(f)).and_then(|i| a.get(i + 1)).map(|s| s.to_string());
    let mk = |cmd: &str, url: Option<String>, dir, name, add_only, silent, quit_after| Msg::Cli {
        cmd: cmd.into(),
        url,
        dir,
        name,
        add_only,
        silent,
        quit_after,
    };
    match a.first().copied() {
        Some("add") => Some(mk("d", a.get(1).map(|s| s.to_string()), val("--dir"), val("--name"), flag("--later"), flag("--silent"), false)),
        Some("list") => Some(mk("list", None, None, None, false, true, false)),
        Some("start-queue") => Some(mk("s", None, None, None, false, true, false)),
        _ if flag("/d") => Some(mk("d", val("/d"), val("/p"), val("/f"), flag("/a"), flag("/n"), flag("/q"))),
        _ if flag("/s") => Some(mk("s", None, None, None, false, true, false)),
        _ => None, // /h alone, --background, nothing: the normal app
    }
}

/// client side: send, print the JSON reply, return the exit code (0 ok, 1 refused, 2 no GDM)
pub fn main(msg: Msg) -> i32 {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
        let _ = AttachConsole(ATTACH_PARENT_PROCESS); // gdm.exe is a GUI program: print into the caller's console
    }
    let Ok(me) = std::env::current_exe() else { return 2 };
    let pipe = gdm_bridge::pipe_name();
    let send = |env: &Envelope, wait| gdm_bridge::call(&pipe, &me, &serde_json::to_vec(env).unwrap_or_default(), wait);
    // /q applies only to a GDM this call starts (IDM: "only first running copy"): a GDM that is already
    // running receives the request without it
    let mut first = msg.clone();
    if let Msg::Cli { quit_after, .. } = &mut first {
        *quit_after = false;
    }
    let mut r = send(&Envelope { origin: "cli".into(), msg: first }, Duration::from_millis(300));
    // never hand the tray GDM our stdio: the caller's pipe would stay open as long as GDM runs
    let spawn = || {
        use std::process::Stdio;
        // CreateProcess inherits every inheritable handle, not only stdio: the caller's output pipe must not
        // reach the tray GDM, or the caller never sees end-of-file (Unix: Rust opens every fd close-on-exec)
        #[cfg(windows)]
        unsafe {
            use windows::Win32::Foundation::{SetHandleInformation, HANDLE_FLAGS, HANDLE_FLAG_INHERIT};
            use windows::Win32::System::Console::{GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE};
            for s in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
                if let Ok(h) = GetStdHandle(s) {
                    let _ = SetHandleInformation(h, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0));
                }
            }
        }
        std::process::Command::new(&me).arg("--background").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().is_ok()
    };
    if r.as_ref().is_err_and(|e| e.starts_with("no GDM")) && spawn() {
        r = send(&Envelope { origin: "cli".into(), msg }, Duration::from_secs(10));
    }
    match r {
        Ok(b) => {
            let v: Value = serde_json::from_slice(&b).unwrap_or(json!({"ok": false, "error": "bad-reply"}));
            println!("{v}");
            if v["ok"].as_bool() == Some(true) {
                0
            } else {
                1
            }
        }
        Err(e) => {
            println!("{}", json!({"ok": false, "error": if e.starts_with("refused") { "untrusted-server" } else { "offline" }}));
            2
        }
    }
}

/// the user's final path of a folder, lower case, without the \\?\ prefix (junctions resolved)
#[cfg(windows)]
fn real(p: &Path) -> Option<String> {
    let c = std::fs::canonicalize(p).ok()?;
    let s = c.to_string_lossy().to_string();
    let s = s.strip_prefix(r"\\?\UNC\").map(|r| format!(r"\\{r}")).unwrap_or_else(|| s.strip_prefix(r"\\?\").unwrap_or(&s).to_string());
    Some(s.trim_end_matches('\\').to_lowercase())
}

/// the final path of a folder (symlinks resolved), lower case: macOS volumes are case-insensitive by default,
/// and on Linux a lower-case compare can only refuse more, never less
#[cfg(unix)]
fn real(p: &Path) -> Option<String> {
    Some(std::fs::canonicalize(p).ok()?.to_string_lossy().to_lowercase())
}

#[cfg(windows)]
fn known(id: &windows::core::GUID) -> Option<PathBuf> {
    use windows::Win32::UI::Shell::{SHGetKnownFolderPath, KF_FLAG_DEFAULT};
    unsafe {
        let p = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None).ok()?;
        let s = p.to_string().ok();
        windows::Win32::System::Com::CoTaskMemFree(Some(p.0 as _));
        s.map(PathBuf::from)
    }
}

/// folders a script must never make GDM write into (I-02), resolved to their real paths: autostart places,
/// program and system folders, GDM's own install folder
#[cfg(unix)]
pub fn dangerous_roots() -> Vec<String> {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let mut v: Vec<PathBuf> = ["/etc", "/usr", "/bin", "/sbin", "/lib", "/opt", "/boot", "/System", "/Library", "/Applications", "/private/etc"]
        .iter()
        .map(PathBuf::from)
        .collect();
    for rel in [
        ".config/autostart",
        ".config/systemd",
        ".config/environment.d",
        ".local/bin",
        ".local/share/applications",
        "bin",
        ".ssh",
        "Library/LaunchAgents",
        "Library/StartupItems",
        "Applications",
    ] {
        v.push(home.join(rel));
    }
    if let Some(d) = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)) {
        v.push(d);
    }
    v.iter().filter_map(|p| real(p)).collect()
}

/// folders a script must never make GDM write into (I-02), resolved to their real paths
#[cfg(windows)]
pub fn dangerous_roots() -> Vec<String> {
    use windows::Win32::UI::Shell::*;
    let mut v: Vec<PathBuf> =
        [FOLDERID_Startup, FOLDERID_CommonStartup, FOLDERID_Windows, FOLDERID_ProgramFiles, FOLDERID_ProgramFilesX86, FOLDERID_ProgramData]
            .iter()
            .filter_map(known)
            .collect();
    if let Some(d) = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)) {
        v.push(d); // GDM's own install folder
    }
    v.iter().filter_map(|p| real(p)).collect()
}

/// I-02 /p: the folder must exist, be held by handle without a junction on the way (fsx::Dir), and be
/// neither a dangerous folder nor inside one (boundary-aware: C:\Windows2 is not C:\Windows)
pub fn safe_dir(p: &str) -> Result<PathBuf, String> {
    let roots = dangerous_roots();
    let bad = |r: &str| roots.iter().any(|root| r == root || r.starts_with(&format!("{root}{}", std::path::MAIN_SEPARATOR)));
    // before opening (junctions resolved by canonicalize) and again on the held handle's path
    if real(Path::new(p)).as_deref().is_some_and(bad) {
        return Err("dangerous-folder".into());
    }
    let d = gdm_core::fsx::Dir::open(Path::new(p)).map_err(|e| match e {
        gdm_core::fsx::FsError::NotFound => "no-folder".to_string(),
        e => format!("folder: {e}"),
    })?;
    if bad(&real(&d.path).ok_or("no-folder")?) {
        return Err("dangerous-folder".into());
    }
    Ok(d.path.clone())
}

#[allow(clippy::too_many_arguments)]
pub async fn serve(
    app: &Arc<App>,
    cmd: &str,
    url: Option<String>,
    dir: Option<String>,
    name: Option<String>,
    add_only: bool,
    silent: bool,
    quit_after: bool,
) -> Value {
    match cmd {
        "list" => {
            let rows = app.rows(None).unwrap_or_default();
            let v: Vec<Value> = rows
                .iter()
                .map(|r| json!({"id": r.row.id, "name": r.row.name, "state": r.row.state, "size": r.row.size, "done": r.row.done, "url": gdm_core::finalize::clean_url(&r.row.url)}))
                .collect();
            json!({"ok": true, "jobs": v})
        }
        "s" => match app.queue_start("main", None) {
            Ok(()) => json!({"ok": true}),
            Err(e) => json!({"ok": false, "error": e}),
        },
        "d" => {
            let Some(url) = url else { return json!({"ok": false, "error": "no-url"}) };
            let target = match dir.as_deref().map(safe_dir) {
                None => None,
                Some(Ok(p)) => Some(p),
                Some(Err(e)) => return json!({"ok": false, "error": e}),
            };
            // a local command, like a typed URL: a LAN or NAS address is allowed (W-02 guards web pages, not the user)
            let r = match target {
                Some(d) => app.engine.add(gdm_core::NewJob { url, dir: d, name, typed: true, ..Default::default() }).map_err(|e| e.code()),
                None => app.add_and_start(&url, name, None, vec![], true, false),
            };
            let id = match r {
                Ok(id) => id,
                Err(e) => return json!({"ok": false, "error": e}),
            };
            if add_only {
                let _ = app.queue_put(std::slice::from_ref(&id), Some("main"));
            } else {
                if quit_after {
                    lock(&app.quit_after).insert(id.clone());
                }
                app.start(&id, gdm_core::Unverified::Ask);
                if !silent {
                    lock(&app.pending).push(id.clone()); // /n absent: the Download Info dialog, like IDM
                    crate::show_main(app, true);
                }
            }
            app.bump();
            json!({"ok": true, "id": id})
        }
        _ => json!({"ok": false, "error": "rejected"}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn a(v: &[&str]) -> Vec<String> {
        std::iter::once("gdm.exe").chain(v.iter().copied()).map(String::from).collect()
    }

    #[test]
    fn i02_idm_flags_and_subcommands() {
        let m = parse(&a(&["/d", "https://e.com/a.zip", "/p", "C:/D", "/f", "b.zip", "/n", "/a", "/q", "/h"])).unwrap();
        assert_eq!(
            m,
            Msg::Cli {
                cmd: "d".into(),
                url: Some("https://e.com/a.zip".into()),
                dir: Some("C:/D".into()),
                name: Some("b.zip".into()),
                add_only: true,
                silent: true,
                quit_after: true
            }
        );
        assert!(matches!(parse(&a(&["/s"])), Some(Msg::Cli { cmd, .. }) if cmd == "s"));
        assert!(matches!(parse(&a(&["list"])), Some(Msg::Cli { cmd, .. }) if cmd == "list"));
        assert!(matches!(parse(&a(&["add", "https://e.com/x", "--later"])), Some(Msg::Cli { add_only: true, .. })));
        assert!(parse(&a(&["--background"])).is_none());
        assert!(parse(&a(&[])).is_none());
        // /d without a URL or with a non-http URL fails validation (never reaches the engine)
        assert!(parse(&a(&["/d", "file:///C:/Windows/win.ini"])).unwrap().validate().is_err());
    }

    /// I-02: Startup, Windows, Program Files and GDM's folder are refused, also through a junction; a normal
    /// user folder is fine
    #[test]
    #[cfg(windows)]
    fn i02_dangerous_folders_refused_even_through_a_junction() {
        let startup = known(&windows::Win32::UI::Shell::FOLDERID_Startup).unwrap();
        assert_eq!(safe_dir(&startup.display().to_string()), Err("dangerous-folder".into()));
        let win = known(&windows::Win32::UI::Shell::FOLDERID_Windows).unwrap();
        assert_eq!(safe_dir(&win.join("System32").display().to_string()), Err("dangerous-folder".into()));
        assert_eq!(safe_dir("C:/definitely/not/here"), Err("no-folder".into()));
        let base = std::env::temp_dir().join(format!("gdm-i02-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        assert!(safe_dir(&base.display().to_string()).is_ok(), "a normal folder");
        // a junction pointing at Startup (mklink /J needs no admin)
        let j = base.join("j");
        let st = std::process::Command::new("cmd").args(["/c", "mklink", "/J"]).arg(&j).arg(&startup).output().unwrap();
        assert!(st.status.success(), "mklink: {}", String::from_utf8_lossy(&st.stdout));
        assert!(safe_dir(&j.display().to_string()).is_err(), "junction to Startup refused");
        let _ = std::fs::remove_dir(&j);
        let _ = std::fs::remove_dir_all(&base);
    }
}
