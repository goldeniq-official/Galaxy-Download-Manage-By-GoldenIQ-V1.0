// SPDX-License-Identifier: RPL-1.5
// The INSTALLED gdm.exe in its real WebView2 (P1 #30): rule 8 probe, layout/clipping probe, axe, screenshots,
// and "the CSP blocks inline script" — over WebView2's own DevTools port (test only, via
// WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; release builds have no devtools and no port).
// node tests/e2e/webview2.mjs <install dir> <data dir> <shots dir>
import { chromium } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { spawn } from "node:child_process";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";

const require = createRequire(import.meta.url);
const P = require("../../design/check.cjs");
const [inst, data, shots] = process.argv.slice(2);
// GDM_WV2_MODE=dpi200 | hc: the WebView2 engine renders at device scale 2 / with forced colors through CDP
// emulation (WebView2 ignores --force-device-scale-factor; Windows' own display settings are not changed)
const MODE = process.env.GDM_WV2_MODE ?? "";
const EXTRA = "";
const TAG = MODE ? `-${MODE}` : "";
const HC = MODE === "hc";
fs.mkdirSync(shots, { recursive: true });
let fails = 0;
const check = (ok, what) => {
  console.log(`${ok ? "PASS" : "FAIL"} ${what}`);
  if (!ok) fails++;
};
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const app = spawn(path.join(inst, "gdm.exe"), [], { env: { ...process.env, GDM_DATA: data, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=9339 ${EXTRA}`.trim() }, stdio: "ignore" });
await sleep(6000);
const browser = await chromium.connectOverCDP("http://127.0.0.1:9339");
// the main window (G7 adds the drop-basket window: basket.html)
const page = browser.contexts()[0].pages().find((p) => !p.url().startsWith("devtools") && !p.url().includes("basket"));
check(!!page, `WebView2 page found (${page?.url()})`);
await page.setViewportSize({ width: 1280, height: 760 }).catch(() => {});
if (MODE) {
  const cdp = await page.context().newCDPSession(page);
  globalThis.__cdp = cdp;
  if (HC) await page.emulateMedia({ forcedColors: "active" });
}
const errors = [];
page.on("pageerror", (e) => errors.push(e.message));
await page.reload();
await page.waitForSelector("#shell .row", { timeout: 15000 }).catch(() => {});
if (MODE === "dpi200") {
  await globalThis.__cdp.send("Emulation.setDeviceMetricsOverride", { width: 1280, height: 760, deviceScaleFactor: 2, mobile: false });
  await page.waitForTimeout(800);
}
await page.evaluate(() => document.fonts.ready);
check((await page.locator("#shell .row").count()) > 0, `the real UI renders the download list (${await page.locator("#shell .row").count()} rows)`);
const lang = JSON.parse(fs.readFileSync(path.join(data, "settings.json"), "utf8")).lang ?? "km";
await page.mouse.move(2, 2);
await page.screenshot({ path: path.join(shots, `webview2-list-${lang}${TAG}.png`) });
const rules = await page.evaluate(P.rulesProbe, HC);
if (MODE) console.log(`engine: dpr=${await page.evaluate(() => window.devicePixelRatio)} forced-colors=${await page.evaluate(() => window.matchMedia("(forced-colors: active)").matches)}`);
check(rules.length === 0, `rule 8 on the real UI: ${rules.length ? rules.join("; ") : "no border/outline/ring/glow"}`);
const lay = await page.evaluate(P.layoutProbe);
check(!lay.hClip.length && !lay.offscreen.length, `layout on the real UI: hclip ${lay.hClip.length}, offscreen ${lay.offscreen.length}`);
const axe = await new AxeBuilder({ page }).analyze();
const serious = axe.violations.filter((v) => v.impact === "serious" || v.impact === "critical");
check(serious.length === 0, `axe on the real UI: ${serious.length} serious/critical ${serious.map((v) => v.id).join(",")}`);
// second real screen: Settings
await page.locator(".rail .nav").last().click();
await page.waitForTimeout(800);
await page.mouse.move(2, 2);
await page.screenshot({ path: path.join(shots, `webview2-settings-${lang}${TAG}.png`) });
const rules2 = await page.evaluate(P.rulesProbe, HC);
const lay2 = await page.evaluate(P.layoutProbe);
const axe2 = (await new AxeBuilder({ page }).analyze()).violations.filter((v) => v.impact === "serious" || v.impact === "critical");
check(!rules2.length && !lay2.hClip.length && !lay2.offscreen.length && !axe2.length, `real Settings screen: rule 8 ${rules2.length}, hclip ${lay2.hClip.length}, offscreen ${lay2.offscreen.length}, axe serious ${axe2.length}`);
const csp = await page.evaluate(() => {
  const s = document.createElement("script");
  s.textContent = "window.__inline = 1";
  document.head.append(s);
  return window.__inline === undefined;
});
check(csp, "X-01 CSP: an injected inline script does not run");
const ipc = await page.evaluate(() => typeof window.__TAURI__ === "undefined");
check(ipc, "X-01 withGlobalTauri=false: no window.__TAURI__ global");
check(errors.length === 0, `no page errors: ${errors.join(" | ") || "none"}`);
await browser.close().catch(() => {});
app.kill();
console.log(fails ? `${fails} FAIL` : "ALL PASS");
process.exit(fails ? 1 : 0);
