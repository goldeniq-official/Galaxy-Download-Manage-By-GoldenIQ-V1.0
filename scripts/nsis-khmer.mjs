// SPDX-License-Identifier: RPL-1.5
// T-02: NSIS ships no Khmer. Copy GDM's Khmer.nlf + MUI Khmer.nsh into the NSIS that the Tauri bundler uses
// (%LOCALAPPDATA%\tauri\NSIS, downloaded on first build). Run before `tauri build` (CI and local).
import fs from "node:fs";
import path from "node:path";
const nsis = path.join(process.env.LOCALAPPDATA ?? "", "tauri", "NSIS", "Contrib", "Language files");
fs.mkdirSync(nsis, { recursive: true });
for (const f of ["Khmer.nlf", "Khmer.nsh"]) fs.copyFileSync(path.join("src-tauri", "nsis", "lang", f), path.join(nsis, f));
console.log("Khmer NSIS language installed in", nsis);
