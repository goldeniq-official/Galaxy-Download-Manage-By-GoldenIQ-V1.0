// SPDX-License-Identifier: RPL-1.5
// C-01 false-catch check: 20 real sites (incl. Reddit's background compression dictionary, Google Docs,
// Synology) opened in Edge/Chromium with the GDM extension and the INSTALLED GDM. No CDP/automation, so the
// browser's real download manager runs. Positive control in the same run: a local page that navigates to a
// 2 MB .zip must be caught (otherwise "0 catches" would prove nothing).
// node tests/e2e/c01-sites.mjs <chrome.exe> <gdm data dir>
import { spawn, execFileSync } from "node:child_process";
import { DatabaseSync } from "node:sqlite";
import http from "node:http";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const [exe, data] = process.argv.slice(2);
const ext = path.resolve("extension", "dist", "chrome");
const SITES = [
  "https://www.reddit.com/r/rust/", "https://www.reddit.com/", "https://docs.google.com/document/u/0/", "https://www.synology.com/en-global/dsm/feature/office",
  "https://www.youtube.com/", "https://github.com/", "https://www.wikipedia.org/", "https://www.bbc.com/", "https://www.microsoft.com/en-us/",
  "https://www.mozilla.org/en-US/firefox/new/", "https://www.python.org/downloads/", "https://nodejs.org/en/download", "https://www.videolan.org/vlc/",
  "https://www.7-zip.org/", "https://www.facebook.com/", "https://www.amazon.com/", "https://stackoverflow.com/questions", "https://www.apple.com/",
  "https://www.cloudflare.com/", "https://www.rabbitmq.com/",
];
// a public 2 MB .zip (local loopback .zip downloads are held back by this PC's security software)
const CONTROL = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip";
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const big = Buffer.alloc(2 * 1024 * 1024, 7);
const srv = http.createServer((q, r) => {
  if (q.url === "/") {
    r.writeHead(200, { "content-type": "text/html" });
    return r.end(`<meta http-equiv="refresh" content="3;url=${CONTROL}"><p>control`);
  }
  r.writeHead(200, { "content-type": "application/zip", "content-length": big.length, "content-disposition": "attachment" });
  let off = 0;
  const t = setInterval(() => (r.destroyed || off >= big.length ? (clearInterval(t), r.end()) : r.write(big.subarray(off, (off += 64 * 1024)))), 150);
});
await new Promise((ok) => srv.listen(0, "127.0.0.1", ok));
const control = `http://127.0.0.1:${srv.address().port}/`;
const t0 = Math.floor(Date.now() / 1000) - 1;

const prof = fs.mkdtempSync(path.join(os.tmpdir(), "gdm-c01-"));
const dl = fs.mkdtempSync(path.join(os.tmpdir(), "gdm-c01-dl-"));
fs.mkdirSync(path.join(prof, "Default"), { recursive: true });
fs.writeFileSync(path.join(prof, "Default", "Preferences"), JSON.stringify({ download: { default_directory: dl, prompt_for_download: false } }));
const chrome = spawn(exe, [`--user-data-dir=${prof}`, `--disable-extensions-except=${ext}`, `--load-extension=${ext}`, "--no-first-run", "--no-default-browser-check", "--no-sandbox", "--window-position=-2400,0", control, ...SITES], { stdio: "ignore" });
await sleep(60_000); // pages load, run scripts, prefetch, idle
const pids = execFileSync("powershell", ["-NoProfile", "-Command", `(Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -like '*${path.basename(prof)}*' }).ProcessId`]).toString().split(/\s+/).filter(Boolean);
chrome.kill();
for (const p of pids) try { process.kill(Number(p)); } catch { /* gone */ }
await sleep(2000);

const db = new DatabaseSync(path.join(data, "gdm.db"), { readOnly: true });
const jobs = db.prepare("SELECT name, url FROM jobs WHERE created_at >= ?").all(t0);
db.close();
srv.close();
const isControl = (j) => j.name.startsWith("ripgrep-14.1.1");
const caughtControl = jobs.some(isControl);
const falseCatches = jobs.filter((j) => !isControl(j));
console.log(`sites opened: ${SITES.length}`);
console.log(`${caughtControl ? "PASS" : "FAIL"} positive control: a navigation to a 2 MB .zip is caught`);
console.log(`${falseCatches.length === 0 ? "PASS" : "FAIL"} C-01 false catches on 20 real sites: ${falseCatches.length} ${falseCatches.map((j) => j.url).join(" ")}`);
console.log(`browser download folder: ${fs.readdirSync(dl).join(", ") || "(empty)"}`);
fs.rmSync(prof, { recursive: true, force: true });
fs.rmSync(dl, { recursive: true, force: true });
process.exit(caughtControl && falseCatches.length === 0 ? 0 : 1);
