// SPDX-License-Identifier: RPL-1.5
// G7 D5 page video button in real Chromium with the real extension (no GDM needed: the native port is stubbed
// inside the service worker, so nothing reaches the user's GDM):
//  - off by default: a page with <video> shows no badge;
//  - turned on for this site: the badge shows the number of <video> players with an http(s) or blob: (MSE) source;
//  - "Download a video from this page" sends {t:"video", page} for an ordinary page, never for YouTube (M-07);
//  - the Chrome build's content script never runs on youtube.com (manifest);
//  - streams seen on the network (an HLS manifest fetched like hls.js does, media files) are offered in the
//    popup only after the site is turned on, and each one goes to GDM as {t:"video", page: <stream>}.
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
/** wait for a condition (bounded) instead of a fixed sleep */
const until = async (f, ms = 10_000) => {
  const end = Date.now() + ms;
  while (Date.now() < end) {
    if (await f()) return true;
    await sleep(100);
  }
  return false;
};

const srv = http.createServer((q, r) => {
  r.writeHead(200, { "content-type": "text/html; charset=utf-8" });
  r.end(`<!doctype html><title>v</title><video src="/a.mp4"></video><video><source src="/b.webm"></video><video src="blob:x"></video><video></video><script>fetch("/hls/master.m3u8?token=t1")</script>`);
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
  const streamsOf = async () => {
    const pg = await ctx.newPage();
    await pg.goto(sw.url().replace("background.js", "popup.html"));
    const r = await pg.evaluate((id) => chrome.runtime.sendMessage({ t: "popup", tab: id }), tabId);
    await pg.close();
    return r?.streams ?? [];
  };
  check((await streamsOf()).length === 0, "off by default: no stream is recorded on a site that is not turned on");
  await sw.evaluate((h) => chrome.storage.local.set({ videoSites: [h] }), host);
  await page.reload();
  // the content script reports players every 2 s; streams arrive with the page's requests
  await until(async () => (await badge()) === String(3 + (await streamsOf()).length) && (await streamsOf()).some((s) => s.kind === "m3u8"));
  const st = await streamsOf();
  const m3u8 = st.find((s) => s.kind === "m3u8");
  check(!!m3u8 && m3u8.url === `${base}/hls/master.m3u8?token=t1`, `turned on: the HLS manifest fetched by the page is offered (${JSON.stringify(st)})`);
  check((await badge()) === String(3 + st.length), `badge = 3 players (2 http(s) + 1 MSE blob:, not the empty one) + ${st.length} streams (got "${await badge()}")`);
  const popup = await ctx.newPage();
  await popup.goto(sw.url().replace("background.js", "popup.html"));
  await popup.evaluate(() => chrome.runtime.sendMessage({ t: "video", page: "https://www.youtube.com/watch?v=abc" }));
  await popup.evaluate((u) => chrome.runtime.sendMessage({ t: "video", page: u }), base + "/");
  await popup.evaluate((u) => chrome.runtime.sendMessage({ t: "video", page: u }), m3u8?.url);
  await until(async () => (await sw.evaluate(() => globalThis.__sent)).filter((m) => m.t === "video").length >= 2, 3000);
  const sent = (await sw.evaluate(() => globalThis.__sent)).filter((m) => m.t === "video");
  check(
    sent.length === 2 && sent[0].page === base + "/" && sent[1].page === m3u8?.url,
    `the page and the chosen stream go to GDM as {t:"video"}, YouTube never (sent: ${JSON.stringify(sent)})`,
  );
} finally {
  await ctx.close();
  srv.close();
  fs.rmSync(prof, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 }); // leave nothing behind
}
console.log(fails ? `${fails} FAIL` : "ALL PASS");
process.exit(fails ? 1 : 0);
