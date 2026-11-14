// SPDX-License-Identifier: RPL-1.5
// End-to-end check of the INSTALLED build: browser framing -> gdm-bridge.exe -> named pipe -> gdm.exe -> engine.
// node tests/e2e/bridge.mjs <install dir> <data dir> <download dir> <fixture base url>
// Covers B-01 (origin, unknown fields), B-03 (length), B-04 (30/min), B-05 (dup), W-02 (private host), a real
// public download landing with Mark-of-the-Web, and the C-05 manifest flag. Prints PASS/FAIL lines; exit 1 on FAIL.
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const [inst, , dl, fx] = process.argv.slice(2);
const bridge = path.join(inst, "gdm-bridge.exe");
const ORIGIN = "chrome-extension://khgpllebdmknoollimkfkggepiembabi/";
let fails = 0;
const check = (ok, what) => {
  console.log(`${ok ? "PASS" : "FAIL"} ${what}`);
  if (!ok) fails++;
};

function frame(o) {
  const b = Buffer.from(typeof o === "string" ? o : JSON.stringify(o));
  const h = Buffer.alloc(4);
  h.writeUInt32LE(b.length);
  return Buffer.concat([h, b]);
}

/** one bridge process, several messages, replies parsed */
function talk(msgs, origin = ORIGIN, raw = null) {
  return new Promise((resolve) => {
    const p = spawn(bridge, [origin], { stdio: ["pipe", "pipe", "inherit"] });
    let buf = Buffer.alloc(0);
    const out = [];
    p.stdout.on("data", (d) => {
      buf = Buffer.concat([buf, d]);
      while (buf.length >= 4 && buf.length >= 4 + buf.readUInt32LE(0)) {
        const n = buf.readUInt32LE(0);
        out.push(JSON.parse(buf.subarray(4, 4 + n).toString()));
        buf = buf.subarray(4 + n);
      }
      if (out.length === msgs.length) p.stdin.end();
    });
    p.on("exit", (code) => resolve({ code, out }));
    if (raw) p.stdin.end(raw);
    else for (const m of msgs) p.stdin.write(frame(m));
  });
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// B-01: an unknown caller is refused before anything happens
check((await talk([], "chrome-extension://aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/")).code === 1, "B-01 unknown extension origin: exit 1, no pipe");
// B-03: a 4 GB length prefix is refused before allocating
check((await talk([], ORIGIN, Buffer.from([0xff, 0xff, 0xff, 0xff]))).code === 2, "B-03 length 0xFFFFFFFF: clean exit 2");

// B-06: GDM is not running; the bridge starts it and waits at most 2.5 s (inside the extension's 3 s ack
// budget, B-05), so a first-ever start on a slow runner may answer "offline" while GDM comes up. Asking again
// must reach the GDM the bridge started.
const t0 = Date.now();
let h = await talk([{ t: "hello", browser: "chrome", ver: "0.9.0" }]);
for (let i = 0; i < 20 && h.out[0]?.ok !== true; i++) {
  await sleep(1000);
  h = await talk([{ t: "hello", browser: "chrome", ver: "0.9.0" }]);
}
check(h.out[0]?.ok === true && Array.isArray(h.out[0]?.catch?.exts), `hello -> GDM (pipe + server image verified) in ${Date.now() - t0} ms: catch policy received`);

// B-01: protocol has no dir/path field
const bad = await talk([{ t: "handoff", hid: "x1", url: "https://example.com/a.exe", dir: "C:\\Users\\Public" }]);
check(bad.out[0]?.ok === false, "B-01 handoff with a dir field: rejected");

// a real public file: lands under its own name with Mark-of-the-Web (N-05) through the whole chain
const url = "https://www.rfc-editor.org/rfc/rfc9110.txt";
const r = await talk([{ t: "handoff", hid: "pub1", url, referrer: "https://www.rfc-editor.org/", name: "rfc9110.txt", size: null, mime: "text/plain", cookie: null }]);
check(r.out[0]?.ok === true && r.out[0]?.started === true, "handoff of a public URL acknowledged and started");
let got = false;
for (let i = 0; i < 120 && !got; i++) {
  await sleep(500);
  got = fs.existsSync(path.join(dl, "rfc9110.txt"));
}
check(got, "file published in the download folder");
if (got) {
  const z = spawnSync("powershell", ["-NoProfile", "-Command", `Get-Content -LiteralPath '${path.join(dl, "rfc9110.txt")}' -Stream Zone.Identifier`], { encoding: "utf8" });
  check(/ZoneId=3/.test(z.stdout), "Mark-of-the-Web ZoneId=3 on the published file");
  check(!fs.readdirSync(dl).some((f) => f.endsWith(".part") || f.endsWith(".part.gdm")), "no .part / sidecar left");
}

// W-02: a private host from the browser is never fetched without the user's yes
const priv = await talk([{ t: "handoff", hid: "priv1", url: `${fx}/file?size=1048576&seed=3`, referrer: null, name: "lan.bin", size: null, mime: null, cookie: null }]);
check(priv.out[0]?.ok === true, "handoff to a LAN host acknowledged (the engine decides)");
await sleep(1500);
check(!fs.existsSync(path.join(dl, "lan.bin")), "W-02 private host from the browser: not downloaded without asking");

// B-05: the browser finished first -> GDM drops its copy
const d1 = await talk([{ t: "handoff", hid: "dup1", url: `${fx}/file?size=1048576&seed=4`, referrer: null, name: "dup.bin", size: null, mime: null, cookie: null }]);
const dd = await talk([{ t: "dup", hid: "dup1" }]);
check(d1.out[0]?.ok === true && dd.out[0]?.ok === true, "B-05 dup accepted");
const st = await talk([{ t: "status" }]);
check(!st.out[0]?.recent?.some((x) => x.name === "dup.bin"), "B-05 duplicate job removed from GDM");

// council G6: a reused handoff id is refused (no silent remapping of an open proposal)
const re1 = await talk([{ t: "handoff", hid: "same1", url: `${fx}/file?size=1024&seed=7`, referrer: null, name: "same.bin", size: null, mime: null, cookie: null }]);
const re2 = await talk([{ t: "handoff", hid: "same1", url: `${fx}/file?size=1024&seed=8`, referrer: null, name: "same2.bin", size: null, mime: null, cookie: null }]);
check(re1.out[0]?.ok === true && re2.out[0]?.ok === false, "reused handoff id refused");

// P1 #15: "download all links" lands as a selection list (nothing starts by itself)
const ln = await talk([{ t: "links", urls: [`${fx}/file?size=10&seed=1`, `${fx}/file?size=10&seed=2`, `${fx}/file?size=10&seed=3`], referrer: null }]);
check(ln.out[0]?.ok === true && ln.out[0]?.count === 3, "P1 #15 context menu 'all links': 3 links listed for selection");

// B-04: 35 handoffs at once -> at most 30 start by themselves (counting the 3 above in the same minute)
const many = Array.from({ length: 35 }, (_, i) => ({ t: "handoff", hid: "rl" + i, url: `${fx}/file?size=1024&seed=${100 + i}`, referrer: null, name: `rl${i}.bin`, size: null, mime: null, cookie: null }));
const m = await talk(many);
const started = m.out.filter((x) => x.started === true).length;
check(m.out.length === 35 && started <= 30 && m.out.filter((x) => x.started === false).length >= 5, `B-04 rate limit: ${started}/35 started, the rest wait for the user`);

// C-05: the extension cannot run in private windows at all
for (const b of ["chrome", "firefox"]) {
  const man = JSON.parse(fs.readFileSync(path.join("extension", "dist", b, "manifest.json"), "utf8"));
  check(man.incognito === "not_allowed", `C-05 ${b} manifest incognito=not_allowed`);
}
const ch = JSON.parse(fs.readFileSync(path.join("extension", "dist", "chrome", "manifest.json"), "utf8"));
check(ch.content_scripts[0].exclude_matches.some((x) => x.includes("youtube.com")), "M-07 Chrome build: no content script on YouTube");

console.log(fails ? `${fails} FAIL` : "ALL PASS");
process.exit(fails ? 1 : 0);
