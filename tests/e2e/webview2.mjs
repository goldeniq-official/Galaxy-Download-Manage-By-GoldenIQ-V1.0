// SPDX-License-Identifier: RPL-1.5
// The INSTALLED gdm.exe in its real WebView2 (P1 #30): rule 8 probe, layout/clipping probe, axe, screenshots,
// and "the CSP blocks inline script" — over WebView2's own DevTools port (test only, via
// WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; release builds have no devtools and no port).
// node tests/e2e/webview2.mjs <install dir> <data dir> <shots dir>
import { chromium } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { spawn, spawnSync } from "node:child_process";
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

const ARGS = `--remote-debugging-port=9339 ${EXTRA}`.trim();
const run = (cmd, args, env = process.env) =>
  new Promise((r) => {
    const c = spawn(cmd, args, { env, stdio: ["ignore", "pipe", "ignore"] });
    let o = "";
    c.stdout.on("data", (d) => (o += d));
    c.on("close", () => r(o.trim()));
  });
// The port reaches WebView2 by WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS, or (CI runners: the env route left the
// browser without the switch, no policy set) by the documented per-user registry value named after the exe, which
// WebView2 reads only when no WEBVIEW2_* variable exists (learn.microsoft.com .../webview2-idl,
// CreateCoreWebView2EnvironmentWithOptions). Test only; the value is removed again on exit.
const REG = "HKCU\\Software\\Policies\\Microsoft\\Edge\\WebView2\\AdditionalBrowserArguments";
let app, exited, t0;
const start = async (route, ms) => {
  const env = { ...process.env, GDM_DATA: data };
  if (route === "env") env.WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = ARGS;
  else await run("reg", ["add", REG, "/v", "gdm.exe", "/t", "REG_SZ", "/d", ARGS, "/f"]);
  exited = null;
  app = spawn(path.join(inst, "gdm.exe"), [], { env, stdio: "ignore" });
  app.on("exit", (code) => (exited = code));
  // a first start on a fresh runner (WebView2 runtime warm-up) is slow: wait for the DevTools port itself
  t0 = Date.now();
  while (Date.now() - t0 < ms && exited === null) {
    if (await fetch("http://127.0.0.1:9339/json/version").then((r) => r.ok, () => false)) return true;
    await sleep(500);
  }
  return false;
};
const cleanReg = () => run("reg", ["delete", REG, "/v", "gdm.exe", "/f"]);
process.on("exit", () => route === "registry" && spawnSync("reg", ["delete", REG, "/v", "gdm.exe", "/f"], { stdio: "ignore" }));
let route = "env";
let ok = await start("env", 30000);
if (!ok) {
  // did the variable reach a child of this process at all (evidence for the env route)
  console.log(`--- child sees: ${await run("cmd", ["/c", "set WEBVIEW2"], { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: ARGS })}`);
  console.log(`--- env route: no DevTools port after 30 s (gdm.exe ${exited === null ? "running" : `exited ${exited}`}); trying the registry route`);
  // end this gdm and its WebView2 browser (a new gdm would join that browser and keep its switches)
  await run("taskkill", ["/F", "/T", "/PID", String(app.pid)]);
  const left = "@(Get-CimInstance Win32_Process -Filter \"Name='msedgewebview2.exe'\" | Where-Object { $_.CommandLine -like '*GoldenIQ.GalaxyDownloadManager*' }).Count";
  for (let i = 0; i < 40 && (await run("powershell", ["-NoProfile", "-Command", left])) !== "0"; i++) await sleep(500);
  route = "registry";
  ok = await start("registry", 90000);
}
if (!ok) {
  // evidence for a hang (this never passed on a CI runner): gdm's window title (a dialog?), its WebView2
  // processes and their arguments, and what GDM wrote next to its data
  const ps = await run("tasklist", ["/V", "/FI", "IMAGENAME eq gdm.exe"]);
  const wv = await run("powershell", ["-NoProfile", "-Command", "Get-CimInstance Win32_Process -Filter \"Name='msedgewebview2.exe'\" | ForEach-Object { $_.ProcessId.ToString() + ' ' + $_.CommandLine.Substring(0, [Math]::Min(300, $_.CommandLine.Length)) }"]);
  const logs = ["webview.log", "crash.log"].map((f) => {
    const p = path.join(data, f);
    return `--- ${f}: ${fs.existsSync(p) ? fs.readFileSync(p, "utf8").slice(-2000) : "(none)"}`;
  });
  const wins = await run("powershell", ["-NoProfile", "-Command", "Get-Process gdm -ErrorAction SilentlyContinue | ForEach-Object { 'title=[' + $_.MainWindowTitle + '] responding=' + $_.Responding }"]);
  console.log(`FAIL WebView2 DevTools port 9339 never opened (env route 30 s, then registry route): gdm.exe ${exited === null ? "still running after 90 s" : `exited with ${exited}`}\n${ps}\n${wins}`);
  console.log(`--- msedgewebview2 processes:\n${wv || "(none)"}\n--- data dir: ${fs.readdirSync(data).join(", ")}\n${logs.join("\n")}`);
  // the browser process (no --type): did --remote-debugging-port reach it, what does it listen on, and is remote
  // debugging turned off by an Edge / WebView2 policy on this machine
  console.log(
    await run("powershell", [
      "-NoProfile",
      "-Command",
      "$b = Get-CimInstance Win32_Process -Filter \"Name='msedgewebview2.exe'\" | Where-Object { $_.CommandLine -notlike '*--type=*' }; " +
        "$b | ForEach-Object { '--- browser ' + $_.ProcessId + ': ' + $_.CommandLine; '--- listening: ' + ((Get-NetTCPConnection -OwningProcess $_.ProcessId -State Listen -ErrorAction SilentlyContinue | ForEach-Object { $_.LocalAddress + ':' + $_.LocalPort }) -join ', ') }; " +
        "foreach ($k in 'HKLM:\\SOFTWARE\\Policies\\Microsoft\\Edge', 'HKLM:\\SOFTWARE\\Policies\\Microsoft\\Edge\\WebView2', 'HKCU:\\SOFTWARE\\Policies\\Microsoft\\Edge', 'HKCU:\\SOFTWARE\\Policies\\Microsoft\\Edge\\WebView2') { if (Test-Path $k) { '--- policy ' + $k; (Get-ItemProperty $k | Out-String).Trim() } }; " +
        "'--- env WEBVIEW2_*: ' + ((Get-ChildItem env: | Where-Object Name -like 'WEBVIEW2*' | ForEach-Object { $_.Name + '=' + $_.Value }) -join '; ')",
    ]),
  );
  app.kill();
  process.exit(1);
}
console.log(`PASS WebView2 DevTools port open after ${((Date.now() - t0) / 1000).toFixed(1)} s (${route} route)`);
if (route === "registry") await cleanReg();
const browser = await chromium.connectOverCDP("http://127.0.0.1:9339");
// the main window (skip DevTools pages)
const page = browser.contexts()[0].pages().find((p) => !p.url().startsWith("devtools"));
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
