// SPDX-License-Identifier: RPL-1.5
// Extension handoff and link-menu rules in real Chromium with the real extension (audit 2026-11-13). The native
// port is stubbed inside the service worker, so nothing reaches the user's GDM:
//  - a caught download carries the browser's User-Agent and cookie; on GDM's ack the browser copy is cancelled;
//  - no ack in 3 s: the browser keeps its download and GDM is told to drop a late job ({t:"dup"});
//  - link menu: one message per origin, the cookie only with its own origin's links, every message < 150 KB;
//  - an older GDM that rejects the new fields gets the message again without them;
//  - a malformed %-escape in the URL does not break the catch.
// Clicks hold Ctrl (force, C-04): Playwright saves downloads under a GUID without an extension, so the type rule
// cannot match here; the type rule itself is covered by tests/e2e/c01-sites.mjs in real Edge.
// node tests/e2e/ext-handoff.mjs <chrome.exe>
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
/** wait for a condition (bounded) instead of a fixed sleep: fixed windows made tests flaky on a busy machine */
const until = async (f, ms = 10_000) => {
  const end = Date.now() + ms;
  while (Date.now() < end) {
    if (await f()) return true;
    await sleep(100);
  }
  return false;
};

const srv = http.createServer((q, r) => {
  const u = new URL(q.url, "http://x");
  if (u.pathname === "/") {
    r.writeHead(200, { "content-type": "text/html; charset=utf-8" });
    return r.end(`<!doctype html><a id=a href="/f/one.bin">one</a> <a id=b href="/f/two.bin">two</a> <a id=c href="/f/bad%E0.bin">bad</a>`);
  }
  // paced: GDM's (stubbed) answer arrives while the browser is still downloading
  r.writeHead(200, { "content-type": "application/octet-stream", "content-length": 4 * 1024 * 1024 });
  let off = 0;
  const t = setInterval(() => (r.destroyed || off >= 4 * 1024 * 1024 ? (clearInterval(t), r.end()) : (r.write(Buffer.alloc(64 * 1024, 7)), (off += 64 * 1024))), 100);
});
await new Promise((ok) => srv.listen(0, "127.0.0.1", ok));
const base = `http://127.0.0.1:${srv.address().port}`;

const prof = fs.mkdtempSync(path.join(os.tmpdir(), "gdm-exth-"));
const ctx = await chromium.launchPersistentContext(prof, {
  executablePath: exe,
  headless: false,
  acceptDownloads: true,
  args: [`--disable-extensions-except=${ext}`, `--load-extension=${ext}`, "--window-position=-2400,0"],
});
try {
  let [sw] = ctx.serviceWorkers();
  if (!sw) sw = await ctx.waitForEvent("serviceworker");
  await ctx.addCookies([{ name: "sid", value: "S1", domain: "127.0.0.1", path: "/" }]);
  // stub: `mode` decides the answer to a handoff; every message is recorded
  const stub = (mode) =>
    sw.evaluate((mode) => {
      globalThis.__sent = [];
      const pol = { ok: true, catch: { enabled: true, exts: ["bin"], min_kb: 1, skip: [], force: "ctrl", bypass: "alt", video: false } };
      chrome.runtime.sendNativeMessage = (_h, m) => {
        globalThis.__sent.push(m);
        if (m.t === "hello") return Promise.resolve(pol);
        if (m.t === "handoff" && mode === "silent") return new Promise(() => {});
        if (mode === "old" && "ua" in m) return Promise.resolve({ ok: false, error: "rejected" });
        return Promise.resolve({ ok: true, id: "j1", started: true });
      };
      lastHello = 0;
    }, mode);
  const sent = () => sw.evaluate(() => globalThis.__sent);
  const ua = await sw.evaluate(() => navigator.userAgent);
  const page = await ctx.newPage();
  await page.goto(base);
  const state = (name) =>
    sw.evaluate(async (n) => (await chrome.downloads.search({})).filter((d) => d.url.includes(n)).map((d) => d.state), name);

  // 1. ack: handoff carries UA + cookie; the browser copy is cancelled and erased
  await stub("ok");
  await page.click("#a", { modifiers: ["Control"] });
  await until(async () => (await sent()).some((m) => m.t === "handoff") && (await state("one.bin")).length === 0);
  const h = (await sent()).find((m) => m.t === "handoff");
  check(h?.ua === ua && h?.cookie === "sid=S1", `handoff carries the browser UA and cookie (${JSON.stringify(h && { ua: h.ua, cookie: h.cookie })})`);
  check((await state("one.bin")).length === 0, "GDM acked: the browser's own copy is gone");

  // 2. no ack: the browser keeps downloading, a dup withdraws a late GDM job
  await stub("silent");
  await page.click("#b", { modifiers: ["Control"] });
  await until(async () => (await sent()).some((m) => m.t === "dup"));
  const s2 = await sent();
  const h2 = s2.find((m) => m.t === "handoff");
  check(!!h2 && s2.some((m) => m.t === "dup" && m.hid === h2.hid), "no ack in 3 s: dup sent for that hid");
  check((await state("two.bin")).some((x) => x === "in_progress" || x === "complete"), `no ack: the browser keeps its download (${await state("two.bin")})`);

  // 3. malformed %-escape: still caught (no URIError out of onCreated)
  await stub("ok");
  await page.click("#c", { modifiers: ["Control"] });
  await until(async () => (await sent()).some((m) => m.t === "handoff" && m.url.includes("bad%E0")));
  check((await sent()).some((m) => m.t === "handoff" && m.url.includes("bad%E0")), "a malformed %-escape in the URL is still handed off");
  const nm = await sw.evaluate((u) => nameOf({ filename: "", url: u, finalUrl: u }), `${base}/f/bad%E0.bin`);
  check(nm === "bad%E0.bin", `the name of a URL with a malformed %-escape is kept as is, no URIError (${nm})`);

  // 4. link menu: per origin, cookie only for its origin, chunks under the native cap
  await stub("ok");
  const long = (i) => `${base}/l/${i}/${"x".repeat(1000)}`;
  const urls = [...Array.from({ length: 300 }, (_, i) => long(i)), "http://localhost:9/other.bin"];
  await sw.evaluate((u) => sendLinks(u, null), urls);
  const links = (await sent()).filter((m) => m.t === "links");
  const mine = links.filter((m) => m.urls[0].startsWith(base));
  const other = links.filter((m) => m.urls[0].startsWith("http://localhost:9"));
  check(mine.length >= 2 && mine.every((m) => JSON.stringify(m).length < 160_000) && mine.flatMap((m) => m.urls).length === 300, `300 long links split into ${mine.length} messages under the cap`);
  check(mine.every((m) => m.cookie === "sid=S1" && m.ua === ua), "the page origin's links carry its cookie and the UA");
  check(other.length === 1 && other[0].cookie === null && other[0].urls.every((u) => u.startsWith("http://localhost:9")), "another origin's links never get that cookie");

  // 5. an older GDM rejects the new fields: the message is sent again without them
  await stub("old");
  await sw.evaluate((u) => sendLinks(u, null), [`${base}/l/old`]);
  const s5 = (await sent()).filter((m) => m.t === "links");
  check(s5.length === 2 && "ua" in s5[0] && !("ua" in s5[1]) && !("cookie" in s5[1]), "older GDM: retried without ua/cookie");
} finally {
  await ctx.close();
  srv.close();
}
console.log(fails ? `${fails} FAILED` : "ALL PASS");
process.exit(fails ? 1 : 0);
