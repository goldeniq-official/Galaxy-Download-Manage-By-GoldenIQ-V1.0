// SPDX-License-Identifier: RPL-1.5
// The installer GDM Setup carries: GDM_SETUP_PAYLOAD = the Tauri NSIS setup .exe (release.yml sets it after
// `tauri build`). Without it (CI checks, tests) Setup is built empty and says so instead of installing.
use std::{env, fs, path::PathBuf};

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("payload.exe");
    println!("cargo:rerun-if-env-changed=GDM_SETUP_PAYLOAD");
    match env::var_os("GDM_SETUP_PAYLOAD") {
        Some(src) => {
            println!("cargo:rerun-if-changed={}", PathBuf::from(&src).display());
            fs::copy(&src, &out).expect("GDM_SETUP_PAYLOAD must name the NSIS setup .exe");
        }
        None => fs::write(&out, b"").unwrap(),
    }
    println!("cargo:rerun-if-changed=setup.rc");
    println!("cargo:rerun-if-changed=setup.manifest");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("setup.rc", embed_resource::NONE).manifest_required().unwrap();
    }
}
