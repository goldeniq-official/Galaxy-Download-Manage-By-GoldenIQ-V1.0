// SPDX-License-Identifier: RPL-1.5
// Windows: the version block SignPath requires on every signed file (ProductName "Galaxy Download Manager",
// the workspace version; docs/release/signpath.md). Same block as crates/gdm-setup/build.rs.
use std::{env, fs, path::PathBuf};

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let v = |k: &str| env::var(k).unwrap();
    let (major, minor, patch) = (v("CARGO_PKG_VERSION_MAJOR"), v("CARGO_PKG_VERSION_MINOR"), v("CARGO_PKG_VERSION_PATCH"));
    let full = v("CARGO_PKG_VERSION");
    let rc = PathBuf::from(v("OUT_DIR")).join("bridge.rc");
    fs::write(
        &rc,
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
      VALUE "FileDescription", "GDM browser bridge"
      VALUE "FileVersion", "{full}"
      VALUE "InternalName", "gdm-bridge.exe"
      VALUE "LegalCopyright", "Copyright (c) 2026 GoldenIQ. RPL-1.5"
      VALUE "OriginalFilename", "gdm-bridge.exe"
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
        ),
    )
    .unwrap();
    embed_resource::compile(&rc, embed_resource::NONE).manifest_optional().unwrap();
}
