// SPDX-License-Identifier: RPL-1.5
//! "Run a program when done" (F08, Q-01 critical). Set ONLY by the Settings UI through `hook_set` (the window
//! `main` capability); `settings_set`, import, the pipe (extension, CLI) never touch it. Stored as an absolute
//! program path + argv list and run with Command::new: no shell, no command line string. Placeholders
//! `{file}` and `{folder}` are whole argv items and are filled with paths GDM made. A .bat/.cmd is the
//! user-trusted exception (D15): cmd.exe parses its arguments, so any argument with % ^ & ! | < > " is refused.
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Hook {
    pub program: String,
    pub args: Vec<String>,
}

pub const LOLBINS: &[&str] = &[
    "cmd.exe",
    "powershell.exe",
    "pwsh.exe",
    "wscript.exe",
    "cscript.exe",
    "mshta.exe",
    "rundll32.exe",
    "regsvr32.exe",
    "msbuild.exe",
    "installutil.exe",
    "regasm.exe",
    "regsvcs.exe",
    "msiexec.exe",
    "bash.exe",
    "wsl.exe",
    "conhost.exe",
    "forfiles.exe",
    "schtasks.exe",
    "certutil.exe",
    "bitsadmin.exe",
    "wmic.exe",
    "explorer.exe",
];

/// macOS / Linux programs that would turn the argument list into code or a command line (also with a version
/// suffix: python3, python3.12, perl5.34)
#[cfg(unix)]
const UNIX_INTERPRETERS: &[&str] = &[
    "sh",
    "bash",
    "zsh",
    "dash",
    "ksh",
    "mksh",
    "csh",
    "tcsh",
    "fish",
    "busybox",
    "env",
    "xargs",
    "nohup",
    "sudo",
    "doas",
    "su",
    "pkexec",
    "python",
    "perl",
    "ruby",
    "node",
    "deno",
    "bun",
    "php",
    "lua",
    "tclsh",
    "wish",
    "awk",
    "gawk",
    "osascript",
    "open",
    "xdg-open",
    "launchctl",
    "systemd-run",
    "find",
    "ssh",
];

pub const CMD_META: &[char] = &['%', '^', '&', '!', '|', '<', '>', '"'];

fn is_batch(p: &str) -> bool {
    let l = p.to_ascii_lowercase();
    l.ends_with(".bat") || l.ends_with(".cmd")
}

/// checked when the user saves the hook and again before every run
pub fn validate(h: &Hook) -> Result<(), String> {
    let p = Path::new(&h.program);
    if !p.is_absolute() || !p.is_file() {
        return Err("program-not-found".into());
    }
    let l = h.program.to_ascii_lowercase();
    // interpreters and LOLBins would turn the argument list into a command line (security review G7)
    let leaf = l.rsplit(['\\', '/']).next().unwrap_or("");
    if LOLBINS.contains(&leaf) {
        return Err("program-interpreter".into());
    }
    #[cfg(windows)]
    if !(l.ends_with(".exe") || is_batch(&l)) {
        return Err("program-type".into()); // .exe, .bat, .cmd only (no .ps1/.vbs/.lnk: they need a host or a shell)
    }
    // macOS / Linux (P4): any executable file, run by exec with an argv list (no shell parses the arguments,
    // even for a #! script); shells and interpreters themselves are refused like cmd.exe / powershell.exe
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let interp =
            UNIX_INTERPRETERS.iter().any(|i| leaf == *i || (leaf.starts_with(i) && leaf[i.len()..].chars().all(|c| c.is_ascii_digit() || c == '.')));
        if interp {
            return Err("program-interpreter".into());
        }
        if std::fs::metadata(p).map(|m| m.permissions().mode() & 0o111 == 0).unwrap_or(true) {
            return Err("program-type".into());
        }
    }
    // what the native confirmation shows must be what runs: no control, bidi/format or line/paragraph
    // separator characters (they can hide or reorder text in the dialog), bounded total length
    let hidden = |c: char| {
        c.is_control()
            || matches!(c, '\u{00AD}' | '\u{061C}' | '\u{180E}' | '\u{200B}'..='\u{200F}' | '\u{2028}'..='\u{202E}' | '\u{2060}'..='\u{206F}' | '\u{FEFF}' | '\u{FFF9}'..='\u{FFFB}')
    };
    let total: usize = h.args.iter().map(|a| a.len()).sum();
    if h.args.len() > 32 || total > 4096 || h.args.iter().any(|a| a.chars().any(hidden)) || h.program.chars().any(hidden) {
        return Err("bad-args".into());
    }
    if is_batch(&l) && h.args.iter().any(|a| a.contains(CMD_META)) {
        return Err("batch-metachar".into());
    }
    Ok(())
}

/// the argv that would run for a finished file (exactly what `run` passes)
pub fn argv(h: &Hook, file: &Path) -> Result<Vec<String>, String> {
    let folder = file.parent().map(|p| p.display().to_string()).unwrap_or_default();
    let out: Vec<String> = h.args.iter().map(|a| a.replace("{file}", &file.display().to_string()).replace("{folder}", &folder)).collect();
    // a downloaded name may legally hold & or % (N-01 keeps them): never hand it to cmd.exe
    if is_batch(&h.program) && out.iter().any(|a| a.contains(CMD_META)) {
        return Err("batch-metachar".into());
    }
    Ok(out)
}

pub fn run(h: &Hook, file: &Path) -> Result<(), String> {
    validate(h)?;
    let args = argv(h, file)?;
    // Rust >= 1.77.2 escapes .bat/.cmd arguments or refuses them (CVE-2024-24576); our check comes first
    Command::new(&h.program).args(args).spawn().map(|_| ()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str, body: &str) -> String {
        let d = std::env::temp_dir().join(format!("gdm-hook-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join(name);
        std::fs::write(&p, body).unwrap();
        p.display().to_string()
    }

    #[test]
    #[cfg(windows)]
    fn q01_validate_and_batch_metachars() {
        let exe = std::env::current_exe().unwrap().display().to_string();
        assert!(validate(&Hook { program: exe.clone(), args: vec!["{file}".into()] }).is_ok());
        assert_eq!(validate(&Hook { program: "notepad.exe".into(), args: vec![] }), Err("program-not-found".into()), "relative path");
        assert_eq!(validate(&Hook { program: r"C:\Windows\System32\cmd.exe".into(), args: vec![] }), Err("program-interpreter".into()));
        let ps = tmp("x.ps1", "calc");
        assert_eq!(validate(&Hook { program: ps, args: vec![] }), Err("program-type".into()));
        let exe0 = std::env::current_exe().unwrap().display().to_string();
        for bad in ["a\u{202E}gpj.exe", "x\u{200B}", "a\u{2028}b", "\u{FEFF}"] {
            assert_eq!(validate(&Hook { program: exe0.clone(), args: vec![bad.into()] }), Err("bad-args".into()), "{bad:?}");
        }
        assert!(validate(&Hook { program: exe0.clone(), args: vec!["ឯកសារ {file}".into()] }).is_ok(), "Khmer text is fine");
        assert_eq!(validate(&Hook { program: exe0, args: vec!["x".repeat(4097)] }), Err("bad-args".into()));
        let bat = tmp("post.bat", "@echo %1 > nul");
        for bad in ["a&calc", "%PATH%", "x|y", "\"q\"", "a^b", "!x!", "<in", ">out"] {
            assert_eq!(validate(&Hook { program: bat.clone(), args: vec![bad.into()] }), Err("batch-metachar".into()), "{bad}");
        }
        let h = Hook { program: bat.clone(), args: vec!["{file}".into(), "--done".into()] };
        assert!(validate(&h).is_ok());
        // a downloaded file named "a&calc.zip" is legal on disk but is never given to cmd.exe
        assert_eq!(argv(&h, Path::new("C:\\D\\a&calc.zip")), Err("batch-metachar".into()));
        assert_eq!(argv(&h, Path::new("C:\\D\\ok.zip")).unwrap(), vec!["C:\\D\\ok.zip".to_string(), "--done".to_string()]);
        // an .exe receives the same name as one argv item (no shell): allowed
        let e = Hook { program: exe, args: vec!["{folder}".into(), "{file}".into()] };
        assert_eq!(argv(&e, Path::new("C:\\D\\a&b.zip")).unwrap(), vec!["C:\\D".to_string(), "C:\\D\\a&b.zip".to_string()]);
    }

    /// Q-01: the hook really runs without a shell: an argument with spaces and a quote-free payload arrives as
    /// one argv item, and the .bat sees it as %1 without cmd evaluating anything else
    #[test]
    #[cfg(windows)]
    fn q01_runs_without_shell() {
        let out = std::env::temp_dir().join(format!("gdm-hook-out-{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&out);
        let bat = tmp("echo1.bat", &format!("@echo off\r\necho [%~1]> \"{}\"\r\n", out.display()));
        let h = Hook { program: bat, args: vec!["{file}".into()] };
        run(&h, Path::new("C:\\Some Dir\\my file.zip")).unwrap();
        for _ in 0..50 {
            if out.exists() && std::fs::metadata(&out).map(|m| m.len() > 0).unwrap_or(false) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        let got = std::fs::read_to_string(&out).unwrap();
        assert_eq!(got.trim(), "[C:\\Some Dir\\my file.zip]");
    }

    /// P4 (macOS / Linux): an executable file is accepted, a non-executable one and an interpreter are not
    #[test]
    #[cfg(unix)]
    fn q01_unix_validate() {
        use std::os::unix::fs::PermissionsExt;
        let exe = std::env::current_exe().unwrap().display().to_string();
        assert!(
            validate(&Hook { program: exe.clone(), args: vec!["{file}".into(), "a;b&c|d".into()] }).is_ok(),
            "no shell: metachars are plain text"
        );
        assert_eq!(validate(&Hook { program: "true".into(), args: vec![] }), Err("program-not-found".into()), "relative path");
        let plain = tmp("notes.txt", "hi");
        std::fs::set_permissions(&plain, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(validate(&Hook { program: plain, args: vec![] }), Err("program-type".into()));
        for i in ["/bin/sh", "/bin/bash", "/usr/bin/env"] {
            if Path::new(i).is_file() {
                assert_eq!(validate(&Hook { program: i.into(), args: vec![] }), Err("program-interpreter".into()), "{i}");
            }
        }
        assert_eq!(validate(&Hook { program: exe, args: vec!["a\u{202E}gpj".into()] }), Err("bad-args".into()));
    }

    /// Q-01 on macOS / Linux: a #! script gets the finished file's path as ONE argv item and nothing in it runs
    #[test]
    #[cfg(unix)]
    fn q01_unix_runs_without_shell() {
        use std::os::unix::fs::PermissionsExt;
        let out = std::env::temp_dir().join(format!("gdm-hook-out-{}.txt", std::process::id()));
        let pwned = std::env::temp_dir().join(format!("gdm-hook-pwned-{}.zip", std::process::id()));
        let _ = std::fs::remove_file(&out);
        let script = tmp("echo1.sh", &format!("#!/bin/sh\nprintf '[%s]' \"$1\" > '{}'\n", out.display()));
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
        let h = Hook { program: script, args: vec!["{file}".into()] };
        let name = format!("/Some Dir/a;touch {}", pwned.display()); // what a shell would run
        run(&h, Path::new(&name)).unwrap();
        for _ in 0..50 {
            if std::fs::metadata(&out).map(|m| m.len() > 0).unwrap_or(false) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        assert_eq!(std::fs::read_to_string(&out).unwrap(), format!("[{name}]"));
        assert!(!pwned.exists(), "nothing in the name ran");
        let _ = std::fs::remove_file(&out);
        let _ = std::fs::remove_dir_all(std::env::temp_dir().join(format!("gdm-hook-{}", std::process::id())));
    }
}
