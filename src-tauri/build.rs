// SPDX-License-Identifier: RPL-1.5
// X-01: app commands get permissions (allow-<command>); a window can call only what its capability grants
const COMMANDS: &[&str] = &[
    "snapshot",
    "tick",
    "settings_get",
    "settings_set",
    "pick_download_dir",
    "folder_check",
    "add_url",
    "info_confirm",
    "info_cancel",
    "job_start",
    "job_pause",
    "job_remove",
    "job_open",
    "job_folder",
    "job_page",
    "job_desc",
    "job_conns",
    "job_limit",
    "job_allow_private",
    "job_detail",
    "pause_all",
    "resume_all",
    "open_link",
    "browsers",
    "update_check",
    "update_restart",
    "queue_list",
    "queue_put",
    "queue_start",
    "queue_stop",
    "queue_cancel_action",
    "hook_set",
    "sound_pick",
    "sound_test",
    "import_pick",
    "import_commit",
    "export_pick",
    "batch_preview",
    "batch_add",
    "grab_run",
    "grab_progress",
    "grab_cancel",
    "grab_pick_dir",
    "grab_add",
    "refresh_start",
    "refresh_paste",
    "refresh_candidate",
    "refresh_decide",
    "login_list",
    "login_set",
    "login_delete",
    "proxy_login_set",
    "media_probe",
    "media_start",
    "components_state",
    "components_install",
    "category_pick_dir",
    "job_set_url",
    "job_move",
];

fn main() {
    // test executables reach tao/wry code that imports TaskDialogIndirect (comctl32 v6 only): give them the
    // same Common-Controls v6 dependency the app manifest has, or they fail with STATUS_ENTRYPOINT_NOT_FOUND
    // (MSVC linker flags: Windows targets only; on Linux/macOS cc would read them as file names)
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg-tests=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'");
    }
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(COMMANDS))).expect("tauri build");
}
