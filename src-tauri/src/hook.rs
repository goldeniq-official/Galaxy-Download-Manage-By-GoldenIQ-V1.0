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
    if !(l.ends_with(".exe") || is_batch(&l)) {
        return Err("program-type".into()); // .exe, .bat, .cmd only (no .ps1/.vbs/.lnk: they need a host or a shell)
    }
    if h.args.len() > 32 || h.args.iter().any(|a| a.len() > 1024 || a.chars().any(|c| c.is_control())) {
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
    fn q01_validate_and_batch_metachars() {
        let exe = std::env::current_exe().unwrap().display().to_string();
        assert!(validate(&Hook { program: exe.clone(), args: vec!["{file}".into()] }).is_ok());
        assert_eq!(validate(&Hook { program: "notepad.exe".into(), args: vec![] }), Err("program-not-found".into()), "relative path");
        assert_eq!(validate(&Hook { program: r"C:\Windows\System32\cmd.exe".into(), args: vec![] }), Err("program-interpreter".into()));
        let ps = tmp("x.ps1", "calc");
        assert_eq!(validate(&Hook { program: ps, args: vec![] }), Err("program-type".into()));
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
}
