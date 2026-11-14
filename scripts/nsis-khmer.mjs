// SPDX-License-Identifier: RPL-1.5
// T-02: NSIS ships no Khmer. Copy GDM's Khmer.nlf + MUI Khmer.nsh into the NSIS that the Tauri bundler uses
// (%LOCALAPPDATA%\tauri\NSIS). Run before `tauri build` (CI and local); extra arguments (e.g. --target) are
// passed to the warm-up build below.
//
// Tauri deletes an NSIS folder that misses any of its own files and downloads it again (tauri-bundler
// nsis/mod.rs, NSIS_REQUIRED_FILES -> remove_dir_all): on a machine without NSIS (every CI runner) the
// Khmer files copied first were wiped, and makensis failed "Can't open language file Khmer.nlf". So when NSIS
// is not there yet, Tauri installs it its own way (its pinned version and checksums) with an English-only
// build of this app, and Khmer goes in after.
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const tool = path.join(process.env.LOCALAPPDATA ?? "", "tauri", "NSIS");
if (!fs.existsSync(path.join(tool, "makensis.exe"))) {
  const cfg = JSON.stringify({ bundle: { createUpdaterArtifacts: false, windows: { nsis: { languages: ["English"] } } } });
  const tauri = path.join("node_modules", "@tauri-apps", "cli", "tauri.js");
  execFileSync(process.execPath, [tauri, "build", "--bundles", "nsis", "--config", cfg, ...process.argv.slice(2), "--", "--locked"], { stdio: "inherit" });
}
const lang = path.join(tool, "Contrib", "Language files");
fs.mkdirSync(lang, { recursive: true });
for (const f of ["Khmer.nlf", "Khmer.nsh"]) fs.copyFileSync(path.join("src-tauri", "nsis", "lang", f), path.join(lang, f));
console.log("Khmer NSIS language installed in", lang);
