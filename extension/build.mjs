// SPDX-License-Identifier: RPL-1.5
// Builds extension/dist/chrome (Chrome, Edge, Brave, Vivaldi, Opera) and extension/dist/firefox from one source.
// No bundler: each script is one file without imports, transpiled by TypeScript. Chrome build: no content
// script on YouTube (M-07). Both: incognito "not_allowed" (C-05: GDM never sees private-window downloads).
import fs from "node:fs";
import path from "node:path";
import ts from "typescript";

import { fileURLToPath } from "node:url";
const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(here, "..");
const version = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8")).version;
const key = fs.readFileSync(path.join(here, "static", "key.txt"), "utf8").trim();

const base = {
  manifest_version: 3,
  name: "__MSG_name__",
  description: "__MSG_desc__",
  default_locale: "en",
  version,
  permissions: ["downloads", "nativeMessaging", "contextMenus", "storage", "cookies", "webRequest"],
  host_permissions: ["http://*/*", "https://*/*"],
  action: { default_popup: "popup.html", default_icon: { 32: "icon-32.png", 48: "icon-48.png" } },
  icons: { 16: "icon-32.png", 32: "icon-32.png", 48: "icon-48.png", 128: "icon-128.png" },
  incognito: "not_allowed",
};

const targets = {
  chrome: {
    ...base,
    key, // fixed id khgpllebdmknoollimkfkggepiembabi for unpacked/dev builds; the stores assign their own
    background: { service_worker: "background.js" },
    content_scripts: [{ matches: ["http://*/*", "https://*/*"], exclude_matches: ["*://*.youtube.com/*", "*://youtube.com/*"], js: ["content.js"], run_at: "document_start", all_frames: true }],
    minimum_chrome_version: "120",
  },
  firefox: {
    ...base,
    background: { scripts: ["background.js"] },
    content_scripts: [{ matches: ["http://*/*", "https://*/*"], js: ["content.js"], run_at: "document_start", all_frames: true }],
    browser_specific_settings: { gecko: { id: "gdm@goldeniq.dev", strict_min_version: "128.0", data_collection_permissions: { required: ["none"] } } },
  },
};

for (const [name, manifest] of Object.entries(targets)) {
  const out = path.join(here, "dist", name);
  fs.rmSync(out, { recursive: true, force: true });
  fs.mkdirSync(out, { recursive: true });
  for (const f of ["background", "content", "popup"]) {
    const src = fs.readFileSync(path.join(here, "src", f + ".ts"), "utf8");
    const js = ts.transpileModule(src, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None, removeComments: false } }).outputText;
    fs.writeFileSync(path.join(out, f + ".js"), `const BUILD = ${JSON.stringify(name)};\n` + js.replace(/^"use strict";\s*/, "").replace(/^export \{\};\s*$/m, ""));
  }
  for (const f of ["popup.html", "popup.css"]) fs.copyFileSync(path.join(here, "static", f), path.join(out, f));
  fs.cpSync(path.join(here, "static", "_locales"), path.join(out, "_locales"), { recursive: true });
  const icons = path.join(root, "src-tauri", "icons");
  fs.copyFileSync(path.join(icons, "32x32.png"), path.join(out, "icon-32.png"));
  fs.copyFileSync(path.join(icons, "128x128.png"), path.join(out, "icon-48.png")); // no 48 px file: the browser scales the 128 down (sharper than stretching 44 px)
  fs.copyFileSync(path.join(icons, "128x128.png"), path.join(out, "icon-128.png"));
  fs.writeFileSync(path.join(out, "manifest.json"), JSON.stringify(manifest, null, 2));
  console.log(`built ${path.relative(root, out)} (${version})`);
}
