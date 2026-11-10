// SPDX-License-Identifier: RPL-1.5
// Real Chromium + the real extension + the INSTALLED gdm-bridge/gdm.exe (B-07 smoke, C-01, C-02, C-04).
// node tests/e2e/extension.mjs <chrome.exe>
// "A plain navigation to a .zip is caught" is covered by tests/e2e/c01-sites.mjs (real Edge, no automation,
// public file): loopback .zip downloads are held back by this PC's security software, so this file covers the
// modifier, size and POST rules.
import { chromium } from "@playwright/test";
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";

const [exe] = process.argv.slice(2);
const ext = path.resolve("extension", "dist", "chrome");
let fails = 0;
const check = (ok, what) => {
  console.log(`${ok ? "PASS" : "FAIL"} ${what}`);
  if (!ok) fails++;
};
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const srv = http.createServer((q, r) => {
  const u = new URL(q.url, "http://x");
  if (u.pathname === "/") {
    r.writeHead(200, { "content-type": "text/html; charset=utf-8" });
    return r.end(`<!doctype html><a id=alt href="/f/alt.zip">alt</a> <a id=small href="/s/small.zip">small</a>
      <a id=ctrl href="/t/forced.txt">ctrl</a><form id=post method=post action="/f/post.zip"><button id=pb>post</button></form>`);
  }
  if (u.pathname.startsWith("/t/")) {
    // paced like a real network: a file the browser finishes before GDM's ack is (correctly) left to the browser (B-05 dup)
    r.writeHead(200, { "content-type": "text/plain", "content-disposition": "attachment", "content-length": 512 * 1024 });
    let off = 0;
    const t = setInterval(() => (r.destroyed || off >= 512 * 1024 ? (clearInterval(t), r.end()) : (r.write("x".repeat(32 * 1024)), (off += 32 * 1024))), 200);
    return;
  }
  const n = u.pathname.startsWith("/s/") ? 2048 : 2 * 1024 * 1024;
  r.writeHead(200, { "content-type": "application/zip", "content-length": n, "content-disposition": "attachment" });
  r.end(Buffer.alloc(n, 7));
});
await new Promise((ok) => srv.listen(0, "127.0.0.1", ok));
const base = `http://127.0.0.1:${srv.address().port}`;

const prof = fs.mkdtempSync(path.join(os.tmpdir(), "gdm-ext-"));
const ctx = await chromium.launchPersistentContext(prof, {
  executablePath: exe,
  headless: false,
  acceptDownloads: true,
  args: [`--disable-extensions-except=${ext}`, `--load-extension=${ext}`, "--window-position=-2400,0"],
});
let [sw] = ctx.serviceWorkers();
if (!sw) sw = await ctx.waitForEvent("serviceworker");
check(sw.url().startsWith("chrome-extension://khgpllebdmknoollimkfkggepiembabi/"), "extension loaded with the fixed id the bridge allows (B-01)");
const native = (m) => sw.evaluate((m) => chrome.runtime.sendNativeMessage("com.goldeniq.gdm", m), m);
const hello = await native({ t: "hello", browser: "chrome", ver: "0.9.0" });
check(hello?.ok === true && hello.catch?.min_kb === 1024, "native messaging from the real extension reaches GDM; catch policy received");
const names = async () => ((await native({ t: "status" }))?.recent ?? []).map((x) => x.name);

const page = await ctx.newPage();
const browserGot = [];
page.on("download", (d) => browserGot.push(d.suggestedFilename()));
await page.goto(base);
await sleep(1000);

await page.click("#alt", { modifiers: ["Alt"] });
await sleep(3000);
check(!(await names()).includes("alt.zip") && browserGot.includes("alt.zip"), "C-04 Alt+click on a 2 MB .zip: the browser keeps it");

await page.click("#small");
await sleep(3000);
check(!(await names()).includes("small.zip"), "C-01 a .zip below the 1 MB minimum is left to the browser");

await page.click("#pb");
await sleep(3000);
check(!(await names()).includes("post.zip"), "C-02 a POST form download is never caught");

await page.goto(base);
await sleep(800);
const before = (await native({ t: "status" }))?.recent?.[0]?.name;
await page.click("#ctrl", { modifiers: ["Control"] });
await sleep(4000);
// Playwright stores downloads under GUID file names, so the job may carry that name: check that a new job appeared
const after = (await native({ t: "status" }))?.recent?.[0]?.name;
check(!!after && after !== before, `C-04 Ctrl+click forces GDM (extension -> bridge -> pipe -> new job "${after}"), even for a .txt`);

await ctx.close();
srv.close();
fs.rmSync(prof, { recursive: true, force: true });
console.log(fails ? `${fails} FAIL` : "ALL PASS");
process.exit(fails ? 1 : 0);
