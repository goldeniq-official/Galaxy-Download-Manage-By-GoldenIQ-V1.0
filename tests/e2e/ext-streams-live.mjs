// SPDX-License-Identifier: RPL-1.5
// Stream detection on REAL public players (internet needed; not part of CI): the real extension in real Chromium,
// native port stubbed (nothing reaches the user's GDM). For each site the video button is turned on, the page is
// played, and the popup must offer the stream the player really loaded.
//   HLS  : hls.js demo page (loads an .m3u8 through XHR/fetch)
//   DASH : DASH-IF reference player (loads an .mpd)
//   MP4  : a plain <video src=".mp4"> page
// node tests/e2e/ext-streams-live.mjs <chrome.exe>   -> prints the detected stream URLs (one JSON line per site)
import { chromium } from "@playwright/test";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";

const [exe] = process.argv.slice(2);
const ext = path.resolve("extension", "dist", "chrome");
const SITES = [
  { name: "hls", url: "https://hls-js.netlify.app/demo/", want: "m3u8" },
  { name: "dash", url: "https://reference.dashif.org/dash.js/latest/samples/getting-started/auto-load-single-video.html", want: "mpd" },
  { name: "mp4", url: "https://www.w3schools.com/html/html5_video.asp", want: "mp4" },
];
let fails = 0;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const prof = fs.mkdtempSync(path.join(os.tmpdir(), "gdm-live-"));
const ctx = await chromium.launchPersistentContext(prof, { executablePath: exe, headless: false, args: [`--disable-extensions-except=${ext}`, `--load-extension=${ext}`, "--window-position=-2400,0", "--autoplay-policy=no-user-gesture-required"] });
const out = {};
try {
  let [sw] = ctx.serviceWorkers();
  if (!sw) sw = await ctx.waitForEvent("serviceworker");
  await sw.evaluate(() => {
    chrome.runtime.sendNativeMessage = (_h, m) => Promise.resolve(m.t === "hello" ? { ok: true, catch: { enabled: true, exts: [], min_kb: 1024, skip: [], force: "ctrl", bypass: "alt", video: true } } : { ok: true });
  });
  for (const s of SITES) {
    const page = await ctx.newPage();
    try {
      // the user's way: open the site, turn the video button on for it in the popup (the page reloads)
      await page.goto(s.url, { waitUntil: "domcontentloaded", timeout: 45_000 });
      const host = new URL(page.url()).host; // after redirects (hls-js.netlify.app -> hlsjs.video-dev.org)
      await sw.evaluate(async (h) => {
        const v = (await chrome.storage.local.get("videoSites")).videoSites ?? [];
        await chrome.storage.local.set({ videoSites: [...v, h] });
      }, host);
      await sleep(300);
      await page.reload({ waitUntil: "domcontentloaded", timeout: 45_000 });
      await page.evaluate(() => document.querySelectorAll("video").forEach((v) => v.play().catch(() => {}))).catch(() => {});
      const tabId = await sw.evaluate(async (u) => (await chrome.tabs.query({})).find((t) => t.url === u)?.id, page.url());
      let st = [];
      for (let i = 0; i < 60 && !st.some((x) => x.kind === s.want); i++) {
        await sleep(500);
        const pg = await ctx.newPage();
        await pg.goto(sw.url().replace("background.js", "popup.html"));
        st = (await pg.evaluate((id) => chrome.runtime.sendMessage({ t: "popup", tab: id }), tabId))?.streams ?? [];
        await pg.close();
      }
      const hit = st.find((x) => x.kind === s.want);
      out[s.name] = hit?.url ?? null;
      console.log(`${hit ? "PASS" : "FAIL"} ${s.name}: ${hit ? hit.url : "no " + s.want + " seen (" + JSON.stringify(st) + ")"}`);
      if (!hit) fails++;
    } catch (e) {
      console.log(`FAIL ${s.name}: ${e.message}`);
      fails++;
    }
    await page.close();
  }
} finally {
  await ctx.close();
}
fs.writeFileSync(path.join(os.tmpdir(), "gdm-live-streams.json"), JSON.stringify(out));
console.log(fails ? `${fails} FAIL` : "ALL PASS");
process.exit(fails ? 1 : 0);
