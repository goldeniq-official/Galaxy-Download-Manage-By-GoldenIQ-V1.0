// SPDX-License-Identifier: RPL-1.5
// G7 D5 page video button in real Chromium with the real extension (no GDM needed: the native port is stubbed
// inside the service worker, so nothing reaches the user's GDM):
//  - off by default: a page with <video> shows no badge;
//  - turned on for this site: the badge shows the number of <video> elements with an http(s) source;
//  - "Download a video from this page" sends {t:"video", page} for an ordinary page, never for YouTube (M-07);
//  - the Chrome build's content script never runs on youtube.com (manifest).
// node tests/e2e/ext-video.mjs <chrome.exe>
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
  r.writeHead(200, { "content-type": "text/html; charset=utf-8" });
  r.end(`<!doctype html><title>v</title><video src="/a.mp4"></video><video><source src="/b.webm"></video><video src="blob:x"></video>`);
});
await new Promise((ok) => srv.listen(0, "127.0.0.1", ok));
const base = `http://127.0.0.1:${srv.address().port}`;
const host = new URL(base).host;

const manifest = JSON.parse(fs.readFileSync(path.join(ext, "manifest.json"), "utf8"));
check(manifest.content_scripts[0].exclude_matches.some((m) => m.includes("youtube.com")), "Chrome build: content script excluded from youtube.com (M-07)");

const prof = fs.mkdtempSync(path.join(os.tmpdir(), "gdm-extv-"));
const ctx = await chromium.launchPersistentContext(prof, { executablePath: exe, headless: false, args: [`--disable-extensions-except=${ext}`, `--load-extension=${ext}`, "--window-position=-2400,0"] });
try {
  let [sw] = ctx.serviceWorkers();
  if (!sw) sw = await ctx.waitForEvent("serviceworker");
  // stub the native port: record what would go to GDM
  await sw.evaluate(() => {
    globalThis.__sent = [];
    chrome.runtime.sendNativeMessage = (_h, m) => (globalThis.__sent.push(m), Promise.resolve({ ok: true, catch: { enabled: true, exts: [], min_kb: 1024, skip: [], force: "ctrl", bypass: "alt", video: true } }));
  });
  const page = await ctx.newPage();
  await page.goto(base);
  await sleep(3000);
  const tabId = await sw.evaluate(async (u) => (await chrome.tabs.query({})).find((t) => t.url?.startsWith(u))?.id, base);
  const badge = () => sw.evaluate((id) => chrome.action.getBadgeText({ tabId: id }), tabId);
  check((await badge()) === "", "off by default: no badge on a page with videos (D5)");
  await sw.evaluate((h) => chrome.storage.local.set({ videoSites: [h] }), host);
  await page.reload();
  await sleep(3500);
  check((await badge()) === "2", `turned on for ${host}: badge counts the 2 http(s) videos (got "${await badge()}")`);
  const popup = await ctx.newPage();
  await popup.goto(sw.url().replace("background.js", "popup.html"));
  await popup.evaluate(() => chrome.runtime.sendMessage({ t: "video", page: "https://www.youtube.com/watch?v=abc" }));
  await popup.evaluate((u) => chrome.runtime.sendMessage({ t: "video", page: u }), base + "/");
  await sleep(500);
  const sent = (await sw.evaluate(() => globalThis.__sent)).filter((m) => m.t === "video");
  check(sent.length === 1 && sent[0].page === base + "/", `only the ordinary page goes to GDM as {t:"video"} (sent: ${JSON.stringify(sent)})`);
} finally {
  await ctx.close();
  srv.close();
}
console.log(fails ? `${fails} FAIL` : "ALL PASS");
process.exit(fails ? 1 : 0);
