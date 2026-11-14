// SPDX-License-Identifier: RPL-1.5
// Pre-release QA inside Windows Sandbox (a clean Windows), run by tests/sandbox/run.ps1. Drives the REAL product
// like a user: GDM Setup, first run, a browser click -> what pops up -> the buttons, the file on disk, failures.
// No npm packages (the repo's node_modules are pnpm junctions that do not resolve in the guest): DevTools over
// Node's own WebSocket. Clicks that matter go through the real Windows mouse (win.ps1), the rest through CDP.
// Every step: PASS / FAIL / BLOCKED (harness could not tell) / NOTE, a real-screen shot, then out/report.md.
import { spawn, spawnSync } from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const IN = "C:\\sbx\\in";
const OUT = "C:\\sbx\\out";
const SHOTS = path.join(OUT, "shots");
const WIN = path.join(HERE, "win.ps1");
const CHROME = process.env.GDM_QA_CHROME; // Playwright's Chromium: branded Chrome/Edge 137+ ignore --load-extension
const EXT = path.join(IN, "ext");
const DEV_ID = "khgpllebdmknoollimkfkggepiembabi";
const P_GDM = 9339, P_SETUP = 9341, P_BR = 9222;
const HOME = os.homedir();
const LAD = process.env.LOCALAPPDATA;
const INST = path.join(LAD, "Galaxy Download Manager");
fs.mkdirSync(SHOTS, { recursive: true });

// ---------- reporting ----------
const results = [];
let shotN = 0;
let blankShots = 0;
const rec = (status, id, what, detail = "") => {
  const line = `${status.padEnd(7)} ${id.padEnd(5)} ${what}${detail ? ` :: ${detail}` : ""}`;
  console.log(line);
  fs.appendFileSync(path.join(OUT, "results.txt"), line + "\n");
  results.push({ status, id, what, detail: String(detail) });
};
const check = (id, ok, what, detail = "") => (rec(ok ? "PASS" : "FAIL", id, what, detail), ok);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const win = (...a) => spawnSync("powershell", ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", WIN, ...a.map(String)], { encoding: "utf8" }).stdout.trim();
const pwsh = (c) => spawnSync("powershell", ["-NoProfile", "-Command", c], { encoding: "utf8" }).stdout.trim();
const shot = (id, name) => {
  const f = path.join(SHOTS, `${String(++shotN).padStart(2, "0")}-${id}-${name}.png`);
  const r = win("shot", f);
  if (r !== "ok") blankShots++;
  return path.basename(f) + (r === "ok" ? "" : " (BLANK: session not visible)");
};
async function until(fn, ms, step = 400) {
  const end = Date.now() + ms;
  for (;;) {
    let v;
    try {
      v = await fn();
    } catch {
      v = null;
    }
    if (v) return v;
    if (Date.now() > end) return null;
    await sleep(step);
  }
}
const sha = (f) => crypto.createHash("sha256").update(fs.readFileSync(f)).digest("hex");
const zone = (f) => pwsh(`Get-Content -LiteralPath '${f.replace(/'/g, "''")}' -Stream Zone.Identifier -ErrorAction SilentlyContinue`);

// ---------- DevTools (CDP) ----------
class Cdp {
  constructor(ws) {
    this.ws = ws;
    this.n = 0;
    this.wait = new Map();
    ws.onmessage = (m) => {
      const d = JSON.parse(m.data);
      const w = d.id && this.wait.get(d.id);
      if (!w) return;
      this.wait.delete(d.id);
      if (d.error) w[1](new Error(d.error.message));
      else w[0](d.result);
    };
  }
  static open(url) {
    return new Promise((ok, ko) => {
      const ws = new WebSocket(url);
      ws.onopen = () => ok(new Cdp(ws));
      ws.onerror = () => ko(new Error(`cannot open ${url}`));
    });
  }
  send(method, params = {}) {
    const id = ++this.n;
    this.ws.send(JSON.stringify({ id, method, params }));
    return new Promise((ok, ko) => {
      this.wait.set(id, [ok, ko]);
      setTimeout(() => this.wait.delete(id) && ko(new Error(`timeout ${method}`)), 20000);
    });
  }
  async eval(expr) {
    const r = await this.send("Runtime.evaluate", { expression: expr, awaitPromise: true, returnByValue: true });
    if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
    return r.result.value;
  }
  close() {
    try {
      this.ws.close();
    } catch {
      /* gone */
    }
  }
}
const list = (port) => fetch(`http://127.0.0.1:${port}/json/list`).then((r) => r.json(), () => null);
async function attach(port, match, ms = 30000) {
  const t = await until(async () => (await list(port))?.find(match), ms);
  return t ? Cdp.open(t.webSocketDebuggerUrl) : null;
}
const isApp = (t) => t.type === "page" && !t.url.startsWith("devtools");
// a selector's box in CSS px, plus the page's scale and size
const box = (c, sel) =>
  c.eval(`(() => { const e = document.querySelector(${JSON.stringify(sel)}); if (!e) return null; e.scrollIntoView({block:"nearest"});
    const r = e.getBoundingClientRect(); const s = getComputedStyle(e);
    return { x: r.x, y: r.y, w: r.width, h: r.height, dpr: devicePixelRatio, iw: innerWidth, ih: innerHeight,
      on: r.width > 0 && r.height > 0 && s.visibility !== "hidden" && r.bottom <= innerHeight + 1 && r.right <= innerWidth + 1 && e.getAttribute("aria-disabled") !== "true" }; })()`);
async function cdpClick(c, sel) {
  const b = await box(c, sel);
  if (!b) throw new Error(`no ${sel}`);
  const x = b.x + b.w / 2, y = b.y + b.h / 2;
  for (const type of ["mousePressed", "mouseReleased"]) await c.send("Input.dispatchMouseEvent", { type, x, y, button: "left", clickCount: 1 });
}
// the real Windows mouse on an element: the page's viewport origin on screen comes from the window rectangle
// (a frameless WebView2 window: the client area; Chromium: below its toolbar, equal side borders)
async function osClick(c, sel, proc) {
  // a user clicks the browser forward first (GDM's dialog/window stays over it after a capture)
  if (proc === "chrome") win("front", "chrome");
  const b = await until(() => box(c, sel), proc === "chrome" ? 60000 : 5000);
  if (!b) throw new Error(`no ${sel}`);
  if (!b.on) throw new Error(`${sel} is not visible/enabled inside the window (${JSON.stringify(b)})`);
  const w = win("win", proc).split(" ").map(Number);
  if (w.length < 6) throw new Error(`no window for ${proc}`);
  const [wx, wy, ww, wh, cx, cy] = w;
  let ox = cx, oy = cy;
  if (proc === "chrome") {
    const side = (ww - b.iw * b.dpr) / 2;
    ox = wx + side;
    oy = wy + wh - b.ih * b.dpr - side;
  }
  win("click", Math.round(ox + (b.x + b.w / 2) * b.dpr), Math.round(oy + (b.y + b.h / 2) * b.dpr));
}
const visible = (c, sel) => box(c, sel).then((b) => b?.on ?? false, () => false);

// ---------- fixture web server (deterministic bytes, Range, throttle, cookies, redirects) ----------
const FILES = {
  "gdm-qa-a.zip": 6 << 20,
  "gdm-qa-b.zip": 3 << 20,
  "gdm-qa-c.zip": 2 << 20,
  "gdm-qa-d.zip": 2 << 20,
  "gdm-qa-e.zip": 2 << 20,
  "gdm-qa-f.zip": 2 << 20,
  "gdm-qa-g.zip": 2 << 20,
  "gdm-qa-m1.zip": 2 << 20,
  "gdm-qa-m2.zip": 2 << 20,
  "gdm-qa-m3.zip": 2 << 20,
  "gdm-qa-slow.zip": 24 << 20,
  "gdm-qa-crash.zip": 24 << 20,
  "gdm-qa-net.zip": 24 << 20,
  "gdm-qa-small.zip": 200 << 10,
  "gdm-qa-auth.zip": 2 << 20,
  "gdm-qa-redir.zip": 2 << 20,
};
const KHMER = "ឯកសារ-តេស្ត GDM.zip";
const body = (name) => {
  const n = FILES[name] ?? FILES["gdm-qa-c.zip"];
  const b = Buffer.alloc(n);
  let s = crypto.createHash("md5").update(name).digest().readUInt32LE(0) || 1;
  for (let i = 0; i < n; i += 4) {
    s ^= s << 13;
    s ^= s >>> 17;
    s ^= s << 5;
    b.writeUInt32LE(s >>> 0, i);
  }
  return b;
};
const cache = new Map();
const bytesOf = (n) => cache.get(n) ?? (cache.set(n, body(n)), cache.get(n));
const served = []; // [path, range header]
const RATE = { "gdm-qa-slow.zip": 1 << 20, "gdm-qa-crash.zip": 1 << 20, "gdm-qa-net.zip": 768 << 10, "gdm-qa-a.zip": 2 << 20 };
const page = (host) => `<!doctype html><meta charset="utf-8"><title>GDM QA downloads</title>
<style>body{font:18px Segoe UI;margin:24px}a,button{display:block;width:520px;margin:10px 0;padding:14px 18px;background:#1565c0;color:#fff;text-decoration:none;border:0;font:inherit;text-align:left}</style>
<h1>GDM QA</h1>
${Object.keys(FILES)
  .filter((f) => !/auth|redir|m\d/.test(f))
  .map((f) => `<a id="${f.replace(/\W/g, "_")}" href="http://${host}/f/${f}">${f}</a>`)
  .join("\n")}
<a id="khmer" href="http://${host}/cd">Khmer name (Content-Disposition)</a>
<a id="redir" href="http://${host}/redir">Redirect</a>
<a id="auth" href="http://${host}/auth/gdm-qa-auth.zip">Cookie-protected</a>
<button id="many" onclick="for (const f of ['m1','m2','m3']) { const a = document.createElement('a'); a.href='/f/gdm-qa-'+f+'.zip'; document.body.append(a); a.click(); }">Three files at once</button>
<script>document.cookie = "sess=gdm-qa; path=/";</script>`;
// a Wi-Fi drop as the client sees it: while `outage` is on, open connections go silent (no FIN, no RST) and new
// ones get no answer; connections from before the outage stay dead afterwards (their path is gone)
let outage = false;
let era = 0;
const srv = http.createServer((q, r) => {
  const u = new URL(q.url, "http://x");
  if (outage && u.pathname.includes("gdm-qa-net")) return; // never answered
  served.push([u.pathname, q.headers.range ?? "", q.headers.cookie ?? "", q.headers["user-agent"] ?? ""]);
  if (u.pathname === "/") return r.writeHead(200, { "content-type": "text/html; charset=utf-8" }), r.end(page(q.headers.host));
  if (u.pathname === "/redir") return r.writeHead(302, { location: "/f/gdm-qa-redir.zip" }), r.end();
  let name, extra = {};
  if (u.pathname === "/cd") {
    name = "gdm-qa-c.zip";
    extra = { "content-disposition": `attachment; filename="khmer.zip"; filename*=UTF-8''${encodeURIComponent(KHMER)}` };
  }
  else if (u.pathname.startsWith("/auth/")) {
    if (!/sess=gdm-qa/.test(q.headers.cookie ?? "")) return r.writeHead(403, { "content-type": "text/plain" }), r.end("forbidden: no session cookie");
    name = "gdm-qa-auth.zip";
  } else if (u.pathname.startsWith("/f/")) name = decodeURIComponent(u.pathname.slice(3));
  if (!name || !(name in FILES)) return r.writeHead(404, { "content-type": "text/plain" }), r.end("not found");
  const all = bytesOf(name);
  let a = 0, z = all.length - 1, code = 200;
  const m = /^bytes=(\d+)-(\d*)$/.exec(q.headers.range ?? "");
  if (m) {
    a = +m[1];
    z = m[2] ? Math.min(+m[2], z) : z;
    code = 206;
  }
  r.writeHead(code, {
    "content-type": "application/zip",
    "content-length": z - a + 1,
    "accept-ranges": "bytes",
    etag: `"${name}-v1"`,
    "last-modified": "Sat, 14 Nov 2026 00:00:00 GMT",
    ...(code === 206 ? { "content-range": `bytes ${a}-${z}/${all.length}` } : {}),
    ...extra,
  });
  if (q.method === "HEAD") return r.end();
  // internet-like speed by default (1 MiB/s): over loopback a 2 MB file finished before its dialog could be used
  const rate = RATE[name] ?? (name === "gdm-qa-small.zip" ? 0 : 1 << 20);
  if (!rate) return r.end(all.subarray(a, z + 1));
  let p = a;
  const born = era;
  const tick = setInterval(() => {
    if (r.destroyed) return clearInterval(tick);
    if (name === "gdm-qa-net.zip" && (outage || born !== era)) return; // silent: a dead link
    const n = Math.min(rate / 10, z + 1 - p);
    r.write(all.subarray(p, p + n));
    p += n;
    if (p > z) {
      clearInterval(tick);
      r.end();
    }
  }, 100);
  r.on("close", () => clearInterval(tick));
});
await new Promise((ok) => srv.listen(8765, "0.0.0.0", ok));
const want = (name) => crypto.createHash("sha256").update(bytesOf(name)).digest("hex");

// ---------- product handles ----------
let gdm = null; // Cdp to GDM's main window
const gdmProcs = () => pwsh("@(Get-Process gdm -ErrorAction SilentlyContinue).Count");
const killGdm = () => pwsh("Stop-Process -Name gdm -Force -ErrorAction SilentlyContinue; Start-Sleep 2");
async function gdmPage(ms = 30000) {
  gdm?.close();
  gdm = await attach(P_GDM, isApp, ms);
  return gdm;
}
const inv = (cmd, args = {}) => gdm.eval(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(args)})`);
const jobs = () => inv("snapshot", { q: null }).then((s) => s.jobs);
const jobBy = async (name) => (await jobs()).find((j) => j.name === name || j.url.endsWith("/" + name));
const fileOf = (j) => path.join(j.dir, j.name);
// a job whose host is private (this fixture: 127.0.0.1 / the LAN address) fails with "Forbidden: host -> ip
// (private ...)" until the user clicks the row's "Allow" (job_allow_private, then it starts again)
async function unblock(j) {
  if (j?.state === "failed" && /private/i.test(j.error_code ?? "")) {
    await inv("job_allow_private", { id: j.id });
    await inv("job_start", { id: j.id, decision: null });
    allowClicks++;
  }
  return j;
}
let allowClicks = 0;
async function finish(name, ms = 90000) {
  return until(async () => {
    const j = await unblock(await jobBy(name));
    return j?.state === "complete" ? j : null;
  }, ms, 700);
}
// Add URL puts the file in the Download info dialog, not started (IDM's flow): Start = info_confirm
async function addStarted(url) {
  const id = await inv("add_url", { url });
  return inv("info_confirm", { id, name: null, category: null, later: false });
}
const dialogOpen = () => visible(gdm, '[role="dialog"] .mfoot .btn-p');
const dialogText = () => gdm.eval(`document.querySelector('[role="dialog"]')?.innerText ?? ""`);

let br = null; // Cdp to the Chromium tab
let chrome = null;
const BRDL = path.join(HOME, "Downloads", "browser-own"); // where the browser itself would save (fallback)
async function startBrowser() {
  const prof = path.join(os.tmpdir(), `gdm-qa-prof-${Date.now()}`);
  fs.mkdirSync(path.join(prof, "Default"), { recursive: true });
  fs.mkdirSync(BRDL, { recursive: true });
  fs.writeFileSync(path.join(prof, "Default", "Preferences"), JSON.stringify({ download: { default_directory: BRDL, prompt_for_download: false } }));
  chrome = spawn(CHROME, [`--user-data-dir=${prof}`, `--disable-extensions-except=${EXT}`, `--load-extension=${EXT}`, `--remote-debugging-port=${P_BR}`, "--no-first-run", "--no-default-browser-check", "--start-maximized", "http://127.0.0.1:8765/"], { stdio: "ignore" });
  br = await attach(P_BR, (t) => t.type === "page" && t.url.startsWith("http://127.0.0.1:8765"), 30000);
  await until(() => br.eval("document.readyState === 'complete'"), 15000);
  return br;
}
async function stopBrowser() {
  br?.close();
  try {
    chrome?.kill();
  } catch {
    /* gone */
  }
  pwsh("Get-Process chrome -ErrorAction SilentlyContinue | Stop-Process -Force; Start-Sleep 1");
}
const browserSaved = () => fs.readdirSync(BRDL).filter((f) => !f.endsWith(".crdownload") && !f.endsWith(".tmp"));
const extWorker = async () => (await list(P_BR))?.find((t) => t.type === "service_worker" && t.url.includes(DEV_ID));

// one browser click -> GDM's "Download info" -> Start; returns the finished job
async function captureAndStart(id, linkSel, name, { allowFg = true } = {}) {
  await osClick(br, linkSel, "chrome");
  // the dialog, or (a file faster than the user) the job already finished: GDM closes the dialog then and says
  // "Downloaded" in a toast
  const up = await until(async () => (gdm || (await gdmPage(3000))) && ((await dialogOpen()) || ((await jobBy(name))?.state === "complete" && "done")), 25000);
  if (up === "done") {
    rec("NOTE", id, "finished before the dialog was used (GDM closes it and shows a toast)", shot(id, "done-first"));
    return jobBy(name);
  }
  if (!up) {
    // evidence for the miss: did the browser hand it over (the job exists), and what did GDM's window show
    const st = await gdm?.eval(`JSON.stringify({ vis: document.visibilityState, iw: innerWidth, ih: innerHeight, dlg: !!document.querySelector('[role="dialog"]') })`).catch((e) => e.message);
    const j = gdm ? await jobBy(name).catch(() => null) : null;
    rec("NOTE", id, "after the click", `job ${j ? `${j.state} ${j.done}` : "none"} · window ${win("win", "gdm")} · fg ${win("fg")} · page ${st}`);
    rec("NOTE", id, "screen", shot(id, "no-dialog"));
  }
  if (!check(id, !!up, `click ${name} in the browser -> GDM's Download info dialog appears`, up ? "" : `gdm processes ${gdmProcs()}`)) return null;
  const fg = win("fg");
  if (allowFg) rec(fg.startsWith("gdm|") ? "PASS" : "FAIL", id, "the dialog comes over the browser (foreground window is GDM)", fg);
  const text = await dialogText();
  rec("NOTE", id, `dialog shows`, text.replace(/\s+/g, " ").slice(0, 200));
  rec("NOTE", id, `screen`, shot(id, "dialog"));
  await sleep(500); // C-03: Start takes clicks after 400 ms
  await osClick(gdm, '[role="dialog"] .mfoot .btn-p', "gdm");
  const closed = await until(async () => !(await dialogOpen()), 5000);
  check(id, !!closed, "Start closes the dialog");
  return finish(name);
}
function verify(id, j, name, expectName = name) {
  if (!check(id, !!j, `${expectName} finished in GDM`)) return;
  const f = fileOf(j);
  check(id, j.name === expectName, `saved under the right name`, j.name);
  check(id, fs.existsSync(f) && sha(f) === want(name), `bytes on disk are exact (sha-256)`, f);
  const z = zone(f);
  check(id, /ZoneId=3/.test(z), "Mark of the Web on the file (SmartScreen/Office see it came from the internet)", z.replace(/\s+/g, " "));
  check(id, browserSaved().length === 0, "the browser did not ALSO save a copy", browserSaved().join(", "));
}

// ================= the run =================
const step = async (id, fn) => {
  try {
    await fn();
  } catch (e) {
    rec("FAIL", id, "step crashed", e.stack?.split("\n").slice(0, 3).join(" | ") ?? e);
    rec("NOTE", id, "screen", shot(id, "crash"));
  }
};
// DevTools ports for the two WebView2 apps: machine policy keyed by exe name (an elevated host ignores the
// WEBVIEW2_* variable). Test only; the Sandbox is thrown away.
for (const [exe, port] of [["gdm.exe", P_GDM], ["gdm-setup.exe", P_SETUP]])
  spawnSync("reg", ["add", "HKLM\\SOFTWARE\\Policies\\Microsoft\\Edge\\WebView2\\AdditionalBrowserArguments", "/v", exe, "/t", "REG_SZ", "/d", `--remote-debugging-port=${port}`, "/f"]);

await step("A", async () => {
  rec("NOTE", "A1", "Windows", pwsh("(Get-CimInstance Win32_OperatingSystem | ForEach-Object { $_.Caption + ' ' + $_.Version })"));
  rec("NOTE", "A1", "WebView2 runtime", pwsh("(Get-ItemProperty 'HKLM:\\SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' -ErrorAction SilentlyContinue).pv"));
  rec("NOTE", "A1", "screen / work area (physical px) / scale", `${win("screen")} / ${win("area")} / ${pwsh("(Get-ItemProperty 'HKCU:\\Control Panel\\Desktop\\WindowMetrics' -ErrorAction SilentlyContinue).AppliedDPI")} dpi`);
  for (const f of ["gdm-setup.exe", "setup.exe"]) rec("NOTE", "A1", `artifact ${f}`, sha(path.join(IN, f)));
  rec("NOTE", "A1", "built from", fs.readFileSync(path.join(IN, "built.txt"), "utf8").split("\n")[0]);
  check("A2", pwsh("Test-Path C:\\Windows\\System32\\vcruntime140.dll") === "False", "clean machine: no Visual C++ runtime installed (the product must not need it)");
});

// ---- B: GDM Setup, the file people download ----
await step("B", async () => {
  const t0 = Date.now();
  const p = spawn(path.join(IN, "gdm-setup.exe"), [], { stdio: "ignore" });
  const su = await attach(P_SETUP, isApp, 40000);
  if (!check("B1", !!su, "GDM Setup opens its window", `${((Date.now() - t0) / 1000).toFixed(1)} s`)) return;
  await until(() => visible(su, "#go"), 10000);
  rec("NOTE", "B1", "screen", shot("B1", "setup-welcome"));
  const dir = await su.eval(`document.querySelector("#dir")?.textContent`);
  check("B2", /AppData\\Local/i.test(dir ?? ""), "per-user install folder shown (no admin needed)", dir);
  check("B2", (await su.eval(`document.querySelector("#nopayload")?.hidden`)) === true, "the installer carries the app (no 'empty setup' note)");
  await osClick(su, "#go", "gdm-setup");
  rec("NOTE", "B3", "screen", shot("B3", "setup-running"));
  const done = await until(() => su.eval(`!document.querySelector("#p-done").hidden || !document.querySelector("#p-fail").hidden`), 120000, 700);
  const ok = done && (await su.eval(`!document.querySelector("#p-done").hidden`));
  rec("NOTE", "B3", "screen", shot("B3", "setup-done"));
  if (!check("B3", !!ok, "install finishes on the 'done' page", ok ? `${((Date.now() - t0) / 1000).toFixed(1)} s` : await su.eval(`document.querySelector("#err")?.textContent`))) return;
  rec("NOTE", "B3", "browsers listed by Setup", (await su.eval(`document.querySelector("#browsers")?.innerText`))?.replace(/\s+/g, " "));
  check("B4", fs.existsSync(path.join(INST, "gdm.exe")) && fs.existsSync(path.join(INST, "gdm-bridge.exe")), "gdm.exe + gdm-bridge.exe installed", INST);
  const un = pwsh("Get-ChildItem HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall | ForEach-Object { Get-ItemProperty $_.PSPath } | Where-Object DisplayName -like 'Galaxy*' | ForEach-Object { $_.DisplayName + '|' + $_.DisplayVersion + '|' + $_.Publisher }");
  check("B4", /Galaxy Download Manager\|\d+\.\d+\.\d+\|GoldenIQ/.test(un), "Apps & features entry (name, version, publisher)", un);
  const lnk = pwsh("@(Get-ChildItem \"$env:APPDATA\\Microsoft\\Windows\\Start Menu\" -Recurse -Filter 'Galaxy*.lnk').Count");
  check("B4", +lnk >= 1, "Start menu shortcut");
  // native messaging host: the installer's step (not repaired by this harness)
  for (const k of ["Google\\Chrome", "Microsoft\\Edge", "Mozilla"]) {
    const v = pwsh(`(Get-ItemProperty 'HKCU:\\Software\\${k}\\NativeMessagingHosts\\*' -ErrorAction SilentlyContinue | Where-Object PSChildName -like '*gdm*').'(default)'`);
    const ok2 = v && fs.existsSync(v);
    const m = ok2 ? JSON.parse(fs.readFileSync(v, "utf8")) : {};
    const good = ok2 && fs.existsSync(m.path) && (k === "Mozilla" ? (m.allowed_extensions ?? []).length > 0 : (m.allowed_origins ?? []).includes(`chrome-extension://${DEV_ID}/`));
    check("B5", !!good, `browser link registered for ${k} (manifest + allowed extension + bridge path)`, v || "missing");
  }
  await osClick(su, "#open", "gdm-setup");
  const opened = await gdmPage(30000);
  check("B6", !!opened, "'Open GDM' on the done page starts GDM");
  check("B6", !!(await until(() => p.exitCode !== null, 10000)), "GDM Setup closes after that");
  su.close();
});

// ---- C: first run ----
await step("C", async () => {
  if (!gdm) await gdmPage();
  await until(() => visible(gdm, ".ob .card"), 15000);
  const [x, y, w, h] = win("win", "gdm").split(" ").map(Number);
  const [ax, ay, aw, ah] = win("area").split(" ").map(Number);
  check("C1", x >= ax && y >= ay && x + w <= ax + aw && y + h <= ay + ah, "the window fits inside the screen (no button below the taskbar)", `window ${x},${y} ${w}x${h} / work area ${ax},${ay} ${aw}x${ah}`);
  for (let s = 1; s <= 3; s++) {
    rec("NOTE", "C2", `onboarding step ${s}`, shot("C2", `onboarding-${s}`));
    const ok = await visible(gdm, ".ob .mfoot .btn-p");
    if (!check("C2", ok, `step ${s}: the main button is visible and reachable`)) break;
    await osClick(gdm, ".ob .mfoot .btn-p", "gdm");
    await sleep(700);
  }
  const main = await until(() => visible(gdm, "#shell .rail"), 10000);
  check("C2", !!main, "onboarding ends in the main window");
  check("C2", (await inv("settings_get")).onboarded === true, "onboarding remembered");
  rec("NOTE", "C2", "main window", shot("C2", "main"));
  // second launch = the same GDM (single instance)
  spawn(path.join(INST, "gdm.exe"), [], { stdio: "ignore", detached: true }).unref();
  await sleep(5000);
  check("C3", gdmProcs() === "1", "starting GDM again opens the same GDM (one process)", `gdm processes ${gdmProcs()}`);
  // closing the window keeps GDM in the tray
  await osClick(gdm, ".tbar .close, button.close[aria-label]", "gdm").catch(() => cdpClick(gdm, "button.close"));
  await sleep(2000);
  check("C4", gdmProcs() === "1" && win("win", "gdm") === "none", "closing the window keeps GDM running in the tray", `procs ${gdmProcs()} window ${win("win", "gdm")}`);
  spawn(path.join(INST, "gdm.exe"), [], { stdio: "ignore", detached: true }).unref();
  check("C4", !!(await until(() => win("win", "gdm") !== "none", 8000)), "starting GDM from the Start menu shows the window again");
});

// ---- D: browser -> GDM, like a user ----
await step("D0", async () => {
  if (!check("D0", !!CHROME && fs.existsSync(CHROME), "test browser present (Chromium)", CHROME)) return;
  killGdm(); // cold: GDM is not running when the first click comes
  gdm = null;
  await startBrowser();
  check("D0", !!(await until(extWorker, 15000)), "GDM extension loaded in the browser", DEV_ID);
  rec("NOTE", "D0", "screen", shot("D0", "browser-page"));
});
await step("D1", async () => {
  const j = await captureAndStart("D1", "#gdm_qa_a_zip", "gdm-qa-a.zip");
  verify("D1", j, "gdm-qa-a.zip");
  const hit = served.filter(([p]) => p === "/f/gdm-qa-a.zip");
  check("D1", hit.every(([, , , ua]) => /Chrome\//.test(ua)), "GDM asked with the browser's own User-Agent", hit.map(([, , , ua]) => ua.slice(0, 40)).join(" ; "));
});
await step("D2", async () => {
  const j = await captureAndStart("D2", "#khmer", KHMER);
  verify("D2", j, "gdm-qa-c.zip", KHMER);
});
await step("D3", async () => {
  await osClick(br, "#gdm_qa_b_zip", "chrome");
  if (!check("D3", !!(await until(dialogOpen, 20000)), "dialog for gdm-qa-b.zip")) return;
  await sleep(500);
  await osClick(gdm, '[role="dialog"] .mfoot .btn-g', "gdm"); // Cancel
  await sleep(4000);
  const j = await jobBy("gdm-qa-b.zip");
  const left = fs.readdirSync(path.join(HOME, "Downloads"), { recursive: true }).filter((f) => String(f).includes("gdm-qa-b"));
  check("D3", !j && !left.length && !browserSaved().length, "Cancel: no job, no file, no partial file anywhere", `${j?.state ?? "no job"} ${left.join(",")}`);
});
await step("D4", async () => {
  const first = await jobBy("gdm-qa-a.zip");
  const before = sha(fileOf(first));
  await osClick(br, "#gdm_qa_a_zip", "chrome");
  if (!check("D4", !!(await until(dialogOpen, 20000)), "same file again -> dialog")) return;
  rec("NOTE", "D4", "dialog shows", (await dialogText()).replace(/\s+/g, " ").slice(0, 200));
  rec("NOTE", "D4", "screen", shot("D4", "duplicate"));
  await sleep(500);
  await osClick(gdm, '[role="dialog"] .mfoot .btn-p', "gdm");
  const s = await until(async () => {
    const j = await unblock((await jobs()).find((x) => x.id !== first.id && x.url.endsWith("gdm-qa-a.zip")));
    return j?.state === "complete" ? j : null;
  }, 60000, 700);
  check("D4", !!s && s.name !== first.name, "a second copy gets a new name, never overwrites", s?.name);
  check("D4", sha(fileOf(first)) === before, "the first file is untouched");
});
await step("D5", async () => verify("D5", await captureAndStart("D5", "#redir", "gdm-qa-redir.zip"), "gdm-qa-redir.zip"));
await step("D6", async () => {
  const j = await captureAndStart("D6", "#auth", "gdm-qa-auth.zip");
  verify("D6", j, "gdm-qa-auth.zip");
  check("D6", served.some(([p, , c]) => p === "/auth/gdm-qa-auth.zip" && /sess=gdm-qa/.test(c)), "the browser's login cookie went with GDM's request");
});
await step("D7", async () => {
  await osClick(br, "#many", "chrome");
  const up = await until(async () => (await dialogOpen()) && /3/.test(await dialogText()), 20000);
  rec("NOTE", "D7", "dialog shows", (await dialogText()).replace(/\s+/g, " ").slice(0, 240));
  rec("NOTE", "D7", "screen", shot("D7", "batch"));
  if (!check("D7", !!up, "three files at once -> one dialog listing 3")) return;
  await sleep(500);
  await osClick(gdm, '[role="dialog"] .mfoot .btn-p', "gdm");
  for (const f of ["gdm-qa-m1.zip", "gdm-qa-m2.zip", "gdm-qa-m3.zip"]) {
    const j = await finish(f, 60000);
    check("D7", !!j && sha(fileOf(j)) === want(f), `${f} complete and exact`);
  }
});
await step("D8", async () => {
  await osClick(br, "#gdm_qa_small_zip", "chrome");
  const saved = await until(() => browserSaved().includes("gdm-qa-small.zip"), 15000);
  check("D8", !!saved && !(await dialogOpen()), "a file under the 1 MB catch size stays with the browser (no dialog)", browserSaved().join(","));
  for (const f of browserSaved()) fs.rmSync(path.join(BRDL, f));
});
await step("D9", async () => {
  await stopBrowser();
  await startBrowser();
  check("D9", !!(await until(extWorker, 15000)), "after a browser restart the extension is back");
  verify("D9", await captureAndStart("D9", "#gdm_qa_e_zip", "gdm-qa-e.zip"), "gdm-qa-e.zip");
});
await step("D10", async () => {
  // MV3: Chrome stops an idle service worker; a click must wake it and still be caught
  const w = await extWorker();
  if (w) await fetch(`http://127.0.0.1:${P_BR}/json/close/${w.id}`).catch(() => {});
  await sleep(2000);
  rec("NOTE", "D10", "worker after stop", (await extWorker()) ? "still listed" : "stopped");
  verify("D10", await captureAndStart("D10", "#gdm_qa_f_zip", "gdm-qa-f.zip"), "gdm-qa-f.zip");
});
await step("D11", async () => {
  // GDM cannot start (broken install): the click must never be lost; the browser keeps the download
  killGdm();
  gdm = null;
  fs.renameSync(path.join(INST, "gdm.exe"), path.join(INST, "gdm.exe.qa-off"));
  try {
    await osClick(br, "#gdm_qa_g_zip", "chrome");
    const saved = await until(() => browserSaved().includes("gdm-qa-g.zip"), 45000, 700);
    check("D11", !!saved, "GDM unavailable -> the browser saves the file itself (nothing lost)", browserSaved().join(","));
    if (saved) check("D11", sha(path.join(BRDL, "gdm-qa-g.zip")) === want("gdm-qa-g.zip"), "and that copy is exact");
    rec("NOTE", "D11", "screen", shot("D11", "fallback"));
  } finally {
    fs.renameSync(path.join(INST, "gdm.exe.qa-off"), path.join(INST, "gdm.exe"));
    for (const f of browserSaved()) fs.rmSync(path.join(BRDL, f));
  }
});
await step("D12", async () => {
  // pause / resume: the rest is fetched with a Range request, not from zero
  verifyStarted: {
    await osClick(br, "#gdm_qa_slow_zip", "chrome");
    if (!(await until(async () => (gdm || (await gdmPage(3000))) && (await dialogOpen()), 25000))) break verifyStarted;
    await sleep(500);
    await osClick(gdm, '[role="dialog"] .mfoot .btn-p', "gdm");
  }
  const j = await until(async () => {
    const x = await jobBy("gdm-qa-slow.zip");
    await unblock(x);
    return x && x.done > 4 << 20 && x.done < 20 << 20 ? x : null;
  }, 40000, 300);
  if (!check("D12", !!j, "slow file is downloading", JSON.stringify(await jobBy("gdm-qa-slow.zip")))) return;
  await inv("job_pause", { id: j.id });
  await sleep(2500);
  const a = await jobBy("gdm-qa-slow.zip");
  await sleep(2000);
  const b = await jobBy("gdm-qa-slow.zip");
  check("D12", a.state === "paused" && a.done === b.done, "Pause stops the transfer", `${a.state} ${a.done} -> ${b.done}`);
  rec("NOTE", "D12", "screen", shot("D12", "paused"));
  const n0 = served.length;
  await inv("job_start", { id: j.id, decision: null });
  const fin = await finish("gdm-qa-slow.zip", 60000);
  verify("D12", fin, "gdm-qa-slow.zip");
  const ranges = served.slice(n0).filter(([p, r]) => p.endsWith("gdm-qa-slow.zip") && r);
  check("D12", ranges.length > 0 && ranges.every(([, r]) => !/^bytes=0-/.test(r) || ranges.length > 1), "Resume continued with Range requests (did not restart from 0)", ranges.map(([, r]) => r).join(" "));
});
await step("D13", async () => {
  // GDM is killed mid-download (crash / power cut of the app): restart, resume, exact file
  await osClick(br, "#gdm_qa_crash_zip", "chrome");
  if (await until(dialogOpen, 20000)) {
    await sleep(500);
    await osClick(gdm, '[role="dialog"] .mfoot .btn-p', "gdm");
  }
  const j = await until(async () => {
    const x = await jobBy("gdm-qa-crash.zip");
    await unblock(x);
    return x && x.done > 6 << 20 ? x : null;
  }, 40000, 300);
  if (!check("D13", !!j, "file is downloading before the crash", JSON.stringify(await jobBy("gdm-qa-crash.zip")))) return;
  killGdm();
  gdm = null;
  spawn(path.join(INST, "gdm.exe"), [], { stdio: "ignore", detached: true }).unref();
  await gdmPage();
  const back = await until(() => jobBy("gdm-qa-crash.zip"), 15000);
  check("D13", back && back.done > 0, "after the crash the job is still listed with its progress", `${back?.state} ${back?.done}`);
  rec("NOTE", "D13", "screen", shot("D13", "after-crash"));
  if (back && back.state !== "complete") await inv("job_start", { id: back.id, decision: null }).catch((e) => rec("NOTE", "D13", "resume said", e.message));
  verify("D13", await finish("gdm-qa-crash.zip", 90000), "gdm-qa-crash.zip");
});
await step("D14", async () => {
  // the connection dies mid-download (Wi-Fi drop: silence, no reset) for 20 s, then the network is back
  await gdmPage();
  const id = await addStarted("http://127.0.0.1:8765/f/gdm-qa-net.zip");
  const going = await until(async () => ((await jobs()).find((j) => j.id === id)?.done ?? 0) > 3 << 20, 30000, 300);
  if (!check("D14", !!going, "file is downloading before the drop")) return;
  outage = true;
  era++;
  await sleep(20000);
  const mid = (await jobs()).find((j) => j.id === id);
  rec("NOTE", "D14", "during the drop", `${mid?.state} ${mid?.done} ${mid?.error_code ?? ""}`);
  rec("NOTE", "D14", "screen", shot("D14", "offline"));
  outage = false;
  const back = Date.now();
  // a timeline, not a guess: what GDM shows every 5 s after the network is back, with no click (3 minutes)
  const line = [];
  let fin = null;
  while (!fin && Date.now() - back < 180000) {
    const x = (await jobs()).find((j) => j.id === id);
    line.push(`${Math.round((Date.now() - back) / 1000)}s ${x?.state} ${x?.done} ${x?.error_code ?? ""}`.trim());
    if (x?.state === "complete") fin = x;
    else await sleep(5000);
  }
  rec("NOTE", "D14", "timeline after the network came back", line.join(" | "));
  check("D14", !!fin, "after the network comes back the download finishes BY ITSELF (no click)", fin ? `${Math.round((Date.now() - back) / 1000)} s after` : "");
  if (fin) check("D14", sha(fileOf(fin)) === want("gdm-qa-net.zip"), "bytes exact after the drop");
});
await step("D15", async () => {
  // what a user sees for a dead link
  const id = await addStarted("http://127.0.0.1:8765/f/missing.zip");
  const j = await until(async () => {
    const x = (await jobs()).find((y) => y.id === id);
    await unblock(x);
    return x && x.state !== "downloading" && x.state !== "queued" && x.error_code && !/private/i.test(x.error_code) ? x : null;
  }, 30000, 700);
  check("D15", !!j, "a 404 link stops with a reason (the row says the file is gone, Open in browser)", `${j?.state} ${j?.error_code}`);
  rec("NOTE", "D15", "screen", shot("D15", "404"));
});
await step("D16", async () => {
  // one real HTTPS download from the internet through a browser click
  await br.eval(`location.href = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"`);
  const rg = async () => (await jobs()).find((x) => x.name.startsWith("ripgrep-14.1.1"));
  const up = await until(async () => (await dialogOpen()) || (await rg()), 30000);
  if (!check("D16", !!up, "real HTTPS link (GitHub release, redirects to a CDN) -> caught by GDM")) return;
  rec("NOTE", "D16", "screen", shot("D16", "https"));
  if (await dialogOpen()) {
    await sleep(500);
    await osClick(gdm, '[role="dialog"] .mfoot .btn-p', "gdm");
  }
  const j = await until(async () => (await jobs()).find((x) => x.name.startsWith("ripgrep-14.1.1") && x.state === "complete"), 120000, 1000);
  if (!check("D16", !!j, "real HTTPS download completes")) return;
  const ref = path.join(os.tmpdir(), "rg-ref.zip");
  spawnSync("curl.exe", ["-sSL", "-o", ref, "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"]);
  check("D16", sha(fileOf(j)) === sha(ref), "same bytes as curl");
  check("D16", /ZoneId=3/.test(zone(fileOf(j))) && /HostUrl=https/.test(zone(fileOf(j))), "Mark of the Web with the source address", zone(fileOf(j)).replace(/\s+/g, " "));
  await br.eval(`location.href = "http://127.0.0.1:8765/"`).catch(() => {});
});

// ---- E: settings that touch Windows ----
await step("E", async () => {
  if (!gdm) await gdmPage();
  await inv("settings_set", { patch: { autostart: true } });
  await sleep(1000);
  // any Run value that starts gdm.exe, whatever its name (the uninstaller removes it by name: record that too)
  const runs = "(Get-Item HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run).Property | ForEach-Object { $_ + '=' + (Get-ItemProperty HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run).$_ } | Where-Object { $_ -match 'gdm' }";
  const on = pwsh(runs);
  check("E1", /gdm\.exe/i.test(on), "'Start with Windows' on -> Run entry", on || `settings say autostart=${(await inv("settings_get")).autostart}`);
  check("E1", !on || on.startsWith("Galaxy Download Manager="), "the Run value has the name the uninstaller removes", on);
  await inv("settings_set", { patch: { autostart: false } });
  await sleep(1000);
  const off = pwsh(runs);
  check("E1", !off, "'Start with Windows' off -> Run entry removed", off);
  await inv("settings_set", { patch: { lang: "en" } });
  await gdm.eval("location.reload()").catch(() => {});
  await sleep(2500);
  await gdmPage();
  rec("NOTE", "E2", "English UI", shot("E2", "english"));
  await inv("settings_set", { patch: { lang: "km" } });
  // resources after work, idle
  await sleep(20000);
  const mem = pwsh("$p = @(Get-Process gdm) + @(Get-CimInstance Win32_Process -Filter \"Name='msedgewebview2.exe'\" | Where-Object { $_.CommandLine -like '*GalaxyDownloadManager*' } | ForEach-Object { Get-Process -Id $_.ProcessId }); '{0:N0} MB in {1} processes' -f (($p | Measure-Object WorkingSet64 -Sum).Sum / 1MB), $p.Count");
  rec("NOTE", "E3", "memory when idle (GDM + its WebView2)", mem);
  const cpu = pwsh("$a = (Get-Process gdm).TotalProcessorTime.TotalMilliseconds; Start-Sleep 10; $b = (Get-Process gdm).TotalProcessorTime.TotalMilliseconds; '{0:N2} %' -f (($b - $a) / 100 / [Environment]::ProcessorCount)");
  rec("NOTE", "E3", "gdm.exe CPU when idle (10 s)", cpu);
});

// ---- F: reinstall over a running GDM keeps everything ----
await step("F", async () => {
  if (!gdm) await gdmPage();
  const n = (await jobs()).length;
  const r = spawnSync(path.join(IN, "setup.exe"), ["/S"], { timeout: 180000 });
  check("F1", r.status === 0, "reinstall (silent, same version) while GDM runs", `exit ${r.status}`);
  spawn(path.join(INST, "gdm.exe"), [], { stdio: "ignore", detached: true }).unref();
  await gdmPage(40000);
  const n2 = await until(async () => (await jobs()).length, 15000);
  check("F1", n2 === n, "the download list survives a reinstall", `${n} -> ${n2}`);
  check("F1", (await inv("settings_get")).onboarded === true, "settings survive (no second onboarding)");
});

// ---- G: uninstall while GDM runs in the tray ----
await step("G", async () => {
  const keep = (await jobs()).filter((j) => j.state === "complete").map(fileOf);
  gdm?.close();
  const us = pwsh("(Get-ChildItem HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall | ForEach-Object { Get-ItemProperty $_.PSPath } | Where-Object DisplayName -like 'Galaxy*').UninstallString").replace(/^"([^"]+)".*$/, "$1");
  spawnSync(us, ["/S"], { timeout: 120000 });
  await until(() => !fs.existsSync(path.join(INST, "gdm.exe")), 30000, 1000);
  check("G1", gdmProcs() === "0", "uninstall closes GDM", `gdm processes ${gdmProcs()}`);
  check("G1", !fs.existsSync(path.join(INST, "gdm.exe")) && !fs.existsSync(path.join(INST, "gdm-bridge.exe")), "program files removed", fs.existsSync(INST) ? fs.readdirSync(INST).join(",") : "folder gone");
  const hosts = pwsh("@(Get-ChildItem 'HKCU:\\Software\\Google\\Chrome\\NativeMessagingHosts', 'HKCU:\\Software\\Microsoft\\Edge\\NativeMessagingHosts', 'HKCU:\\Software\\Mozilla\\NativeMessagingHosts' -ErrorAction SilentlyContinue | Where-Object PSChildName -like '*gdm*').Count");
  check("G1", hosts === "0", "browser links removed", hosts);
  check("G1", !pwsh("(Get-ItemProperty HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run -ErrorAction SilentlyContinue).'Galaxy Download Manager'"), "no autostart left");
  check("G1", !pwsh("Get-ChildItem HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall | ForEach-Object { Get-ItemProperty $_.PSPath } | Where-Object DisplayName -like 'Galaxy*'"), "Apps & features entry removed");
  check("G1", keep.length > 0 && keep.every((f) => fs.existsSync(f)), "the user's downloaded files are kept", `${keep.filter((f) => fs.existsSync(f)).length}/${keep.length}`);
  const data = [path.join(LAD, "GoldenIQ.GalaxyDownloadManager"), path.join(process.env.APPDATA, "GoldenIQ.GalaxyDownloadManager"), path.join(LAD, "GDM")].filter((d) => fs.existsSync(d));
  rec("NOTE", "G1", "data left after uninstall (list/settings for a reinstall)", data.join(", ") || "none");
});

rec("NOTE", "UX", `"Allow" clicks a user needed for downloads from this PC / the local network`, String(allowClicks));
await stopBrowser();
srv.close();
spawnSync("reg", ["delete", "HKLM\\SOFTWARE\\Policies\\Microsoft\\Edge\\WebView2\\AdditionalBrowserArguments", "/f"]);

// ---------- report ----------
const NOT = [
  ["Windows 10", "Sandbox runs the host's Windows 11 build only: needs a Windows 10 VM before release"],
  ["Windows on ARM64", "needs an ARM64 device (the ARM64 build is cross-compiled only)"],
  ["Standard (non-admin) user", "Sandbox runs as an administrator: needs a standard-user account on a VM"],
  ["Branded Chrome / Edge from the stores, Firefox", "store items not published yet; this run used Chromium with the unpacked extension"],
  ["Display scaling 125-200 %, small and 4K screens", "Sandbox keeps one scale; the window-fit logic has a unit test"],
  ["Reboot / power loss during a download", "Sandbox resets on reboot (the app-kill test D13 covers the app side)"],
  ["SmartScreen / antivirus reputation of the signed installer", "needs the SignPath-signed release build"],
  ["Upgrade from the previous public version", "no public version exists yet (first release 1.0.0)"],
];
const count = (s) => results.filter((r) => r.status === s).length;
const md = [
  `# GDM pre-release QA (Windows Sandbox)`,
  ``,
  `PASS ${count("PASS")} · FAIL ${count("FAIL")} · BLOCKED ${count("BLOCKED")} · screenshots ${shotN}${blankShots ? ` (${blankShots} BLANK: the run is INCONCLUSIVE where a shot is blank)` : ""}`,
  ``,
  `| | step | check | detail |`,
  `|---|---|---|---|`,
  ...results.map((r) => `| ${r.status} | ${r.id} | ${r.what.replace(/\|/g, "/")} | ${r.detail.replace(/\|/g, "/").replace(/\n/g, " ").slice(0, 300)} |`),
  ``,
  `## Not tested here (and what is needed)`,
  ...NOT.map(([a, b]) => `- **${a}**: ${b}`),
];
fs.writeFileSync(path.join(OUT, "report.md"), md.join("\n"));
console.log(`DONE PASS ${count("PASS")} FAIL ${count("FAIL")} BLOCKED ${count("BLOCKED")}`);
process.exit(0);
