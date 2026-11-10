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
];

fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(COMMANDS))).expect("tauri build");
}
