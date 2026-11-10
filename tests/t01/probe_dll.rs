// SPDX-License-Identifier: RPL-1.5
// T-01 probe: a DLL planted next to the installer under commonly hijacked names. If any process loads it, its
// DllMain appends "<pid> <dll name>" to %TEMP%\gdm-t01-hijack.txt. Build: rustc -O --crate-type cdylib probe_dll.rs
#![allow(non_snake_case)]
use std::io::Write;

#[no_mangle]
pub extern "system" fn DllMain(_h: *mut core::ffi::c_void, reason: u32, _r: *mut core::ffi::c_void) -> i32 {
    if reason == 1 {
        let p = std::env::temp_dir().join("gdm-t01-hijack.txt");
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(p) {
            let _ = writeln!(f, "{} {:?}", std::process::id(), std::env::current_exe().ok());
        }
    }
    1
}
