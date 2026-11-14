// SPDX-License-Identifier: RPL-1.5
// Real capture check (owner 2026-11-14: a real download was not caught): real Chrome (no automation) + unpacked extension + INSTALLED GDM that is NOT running
// (cold start through the bridge), a public 2 MB .zip. GDM data in a throwaway folder (GDM_DATA is inherited
// Chrome -> bridge -> gdm.exe). node tests/e2e/cold-catch.mjs <chromium chrome.exe> <repo>. Branded Chrome 137+ ignores --load-extension: use Chromium.
import { spawn, execFileSync } from "node:child_process";
import { DatabaseSync } from "node:sqlite";
import http from "node:http";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const [exe, repo] = process.argv.slice(2);
const ext = path.join(repo, "extension", "dist", "chrome");
const URL_ZIP = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip";
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const srv = http.createServer((q, r) => {
  r.writeHead(200, { "content-type": "text/html" });
  r.end(`<meta http-equiv="refresh" content="4;url=${URL_ZIP}"><p>control`);
});
await new Promise((ok) => srv.listen(0, "127.0.0.1", ok));
const page = `http://127.0.0.1:${srv.address().port}/`;

const root = fs.mkdtempSync(path.join(os.tmpdir(), "gdm-real-"));
const prof = path.join(root, "prof"), dl = path.join(root, "dl"), data = path.join(root, "data");
for (const d of [prof, dl, data]) fs.mkdirSync(d, { recursive: true });
fs.mkdirSync(path.join(prof, "Default"));
fs.writeFileSync(path.join(prof, "Default", "Preferences"), JSON.stringify({ download: { default_directory: dl, prompt_for_download: false } }));
const t0 = Math.floor(Date.now() / 1000) - 1;
if (/Galaxy Download Manager\\gdm/.test(execFileSync("powershell", ["-NoProfile", "-Command", "(Get-Process gdm -ErrorAction SilentlyContinue).Path"]).toString())) {
  console.log("the installed GDM is running: quit it from its tray first (this check needs a cold start)");
  process.exit(2);
}
const chrome = spawn(exe, [`--user-data-dir=${prof}`, `--disable-extensions-except=${ext}`, `--load-extension=${ext}`, "--no-first-run", "--no-default-browser-check", "--window-position=-2400,0", page], {
  stdio: "ignore",
  env: { ...process.env, GDM_DATA: data },
});
await sleep(45_000);
const pids = execFileSync("powershell", ["-NoProfile", "-Command", `(Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -like '*${path.basename(root)}*' }).ProcessId`]).toString().split(/\s+/).filter(Boolean);
chrome.kill();
for (const p of pids) try { process.kill(Number(p)); } catch { /* gone */ }
srv.close();
await sleep(2000);
let jobs = [];
try {
  const db = new DatabaseSync(path.join(data, "gdm.db"), { readOnly: true });
  jobs = db.prepare("SELECT name, url FROM jobs WHERE created_at >= ?").all(t0);
  db.close();
} catch (e) {
  console.log("no GDM db:", e.message);
}
const browserFiles = fs.readdirSync(dl);
const caught = jobs.some((j) => j.name.startsWith("ripgrep-14.1.1"));
console.log("GDM jobs:", JSON.stringify(jobs));
console.log("browser download folder:", JSON.stringify(browserFiles));
console.log(`${caught ? "PASS" : "FAIL"} cold start: the real download reached GDM`);
console.log(`${!browserFiles.some((f) => f.startsWith("ripgrep") && !f.endsWith(".crdownload")) ? "PASS" : "FAIL"} the browser did not keep its own copy`);
// clean up: the tray GDM this run started (throwaway data) and every file it made
execFileSync("powershell", ["-NoProfile", "-Command", "Get-Process gdm -ErrorAction SilentlyContinue | Where-Object Path -like '*\\Galaxy Download Manager\\gdm.exe' | Stop-Process -Force"]);
await sleep(1000);
fs.rmSync(root, { recursive: true, force: true });
process.exit(caught ? 0 : 1);
