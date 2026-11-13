// SPDX-License-Identifier: RPL-1.5
// The installer GDM Setup carries: GDM_SETUP_PAYLOAD = the Tauri NSIS setup .exe (release.yml sets it after
// `tauri build`). Without it (CI checks, tests) Setup is built empty and says so instead of installing.
use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out = out_dir.join("payload.exe");
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
        // setup.rc (icon + manifest) plus the version block SignPath requires on every signed file:
        // ProductName "Galaxy Download Manager" and the workspace version (docs/release/signpath.md).
        let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let base = fs::read_to_string(dir.join("setup.rc")).unwrap();
        let rc = out_dir.join("setup-full.rc");
        fs::write(&rc, format!("{}\n{}", absolute(&base, &dir), version_block("GDM Setup", "gdm-setup.exe"))).unwrap();
        embed_resource::compile(&rc, embed_resource::NONE).manifest_required().unwrap();
    }
}

/// setup.rc's relative paths, made absolute: the generated .rc lives in OUT_DIR.
fn absolute(rc: &str, dir: &std::path::Path) -> String {
    let fix = |name: &str| dir.join(name).display().to_string().replace('\\', "/");
    rc.replace("\"../../src-tauri/icons/icon.ico\"", &format!("\"{}\"", fix("../../src-tauri/icons/icon.ico")))
        .replace("\"setup.manifest\"", &format!("\"{}\"", fix("setup.manifest")))
}

/// A VERSIONINFO resource from this crate's version (one version for everything: the workspace's).
fn version_block(description: &str, file: &str) -> String {
    let v = |k: &str| env::var(k).unwrap();
    let (major, minor, patch) = (v("CARGO_PKG_VERSION_MAJOR"), v("CARGO_PKG_VERSION_MINOR"), v("CARGO_PKG_VERSION_PATCH"));
    let full = v("CARGO_PKG_VERSION");
    format!(
        r#"1 VERSIONINFO
FILEVERSION {major},{minor},{patch},0
PRODUCTVERSION {major},{minor},{patch},0
FILEOS 0x40004
FILETYPE 0x1
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904B0"
    BEGIN
      VALUE "CompanyName", "GoldenIQ"
      VALUE "FileDescription", "{description}"
      VALUE "FileVersion", "{full}"
      VALUE "InternalName", "{file}"
      VALUE "LegalCopyright", "Copyright (c) 2026 GoldenIQ. RPL-1.5"
      VALUE "OriginalFilename", "{file}"
      VALUE "ProductName", "Galaxy Download Manager"
      VALUE "ProductVersion", "{full}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#
    )
}
