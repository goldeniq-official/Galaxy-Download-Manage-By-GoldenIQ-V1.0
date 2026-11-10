// SPDX-License-Identifier: RPL-1.5
// B-09 kill matrix on the INSTALLED build: the harness plays the extension; gdm.exe (engine) or gdm-bridge.exe is
// killed (TerminateProcess) at a spread of moments around the handoff: before the reply, right after the ack,
// mid-download. Rule checked for every trial: NEVER LOST — either no ack reached the "browser" (so the browser
// keeps its own download), or GDM has the job and, after GDM restarts, publishes the complete file. Duplicates
// (ack lost but GDM also downloads) are allowed and counted, never hidden.
// node tests/e2e/b09.mjs <install dir> <data dir> <download dir> <url> <expected size>
import { spawn, execFileSync } from "node:child_process";
import { DatabaseSync } from "node:sqlite";
import fs from "node:fs";
import path from "node:path";

const [inst, data, dl, url, sizeArg] = process.argv.slice(2);
const size = Number(sizeArg);
const bridge = path.join(inst, "gdm-bridge.exe");
const ORIGIN = "chrome-extension://khgpllebdmknoollimkfkggepiembabi/";
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const killGdm = () => execFileSync("taskkill", ["/IM", "gdm.exe", "/F"], { stdio: "ignore" });
const startGdm = (delay = 0) => spawn(path.join(inst, "gdm.exe"), ["--background"], { env: { ...process.env, GDM_DATA: data, ...(delay ? { GDM_TEST_REPLY_DELAY_MS: String(delay) } : {}) }, stdio: "ignore", detached: true }).unref();
function frame(o) {
  const b = Buffer.from(JSON.stringify(o));
  const h = Buffer.alloc(4);
  h.writeUInt32LE(b.length);
  return Buffer.concat([h, b]);
}

// one handoff; `kill` = { who: "engine" | "bridge" | "ext", afterMs } fires while the exchange is in flight.
// "ext" = the extension's service worker dies: its end of the native port closes, and whatever reply has not
// arrived by then is never seen (so it cannot cancel the browser's own download).
function handoff(i, kill) {
  return new Promise((resolve) => {
    const p = spawn(bridge, [ORIGIN], { stdio: ["pipe", "pipe", "ignore"], env: { ...process.env, GDM_DATA: data } });
    let buf = Buffer.alloc(0);
    let reply = null;
    let dead = false;
    p.stdout.on("data", (d) => {
      if (dead) return;
      buf = Buffer.concat([buf, d]);
      if (buf.length >= 4 && buf.length >= 4 + buf.readUInt32LE(0)) {
        reply = JSON.parse(buf.subarray(4, 4 + buf.readUInt32LE(0)).toString());
        p.stdin.end();
      }
    });
    p.on("exit", () => resolve(reply));
    p.stdin.write(frame({ t: "handoff", hid: `b09-${i}-${Date.now()}`, url, referrer: null, name: `b09-${i}.zip`, size: null, mime: "application/zip", cookie: null }));
    const extDies = () => {
      dead = true;
      const seen = reply;
      p.stdin.destroy();
      p.stdout.destroy();
      resolve(seen); // the extension knows only what it had received before it died
    };
    if (kill) setTimeout(() => (kill.who === "engine" ? (() => { try { killGdm(); } catch { /* not running */ } })() : kill.who === "ext" ? extDies() : p.kill()), kill.afterMs);
  });
}

const plan = [];
for (const ms of [0, 5, 20, 60, 150]) plan.push({ who: "engine", afterMs: ms, phase: "around the reply" });
for (const ms of [0, 5, 20, 60, 150]) plan.push({ who: "bridge", afterMs: ms, phase: "bridge dies mid-exchange" });
for (const ms of [0, 5, 20, 60, 150]) plan.push({ who: "ext", afterMs: ms, phase: "extension worker dies mid-exchange" });
for (const ms of [1500, 3000]) plan.push({ who: "engine-later", afterMs: ms, phase: "mid-download" });
// deterministic pre-ack window: GDM holds its reply 2 s after creating the job (test hook), the engine dies inside it
for (const ms of [300, 1000]) plan.push({ who: "engine", afterMs: ms, phase: "job created, reply not sent", delay: 2000 });
for (const ms of [300, 1000]) plan.push({ who: "ext", afterMs: ms, phase: "extension dies, job created, reply not sent", delay: 2000 });

startGdm();
await sleep(3000);
const results = [];
for (const [i, k] of plan.entries()) {
  try { killGdm(); } catch { /* ok */ }
  startGdm(k.delay ?? 0);
  await sleep(2500);
  const r = await handoff(i, k.who === "engine-later" ? null : k);
  if (k.who === "engine-later") {
    await sleep(k.afterMs);
    try { killGdm(); } catch { /* ok */ }
  }
  results.push({ i, ...k, acked: !!(r && r.ok) });
}
// restart once: interrupted jobs resume by themselves (resume_interrupted), then let them finish
try { killGdm(); } catch { /* ok */ }
startGdm();
const deadline = Date.now() + 120_000;
const db = () => new DatabaseSync(path.join(data, "gdm.db"), { readOnly: true });
let rows = [];
while (Date.now() < deadline) {
  const d = db();
  rows = d.prepare("SELECT name, state FROM jobs WHERE name LIKE 'b09-%'").all();
  d.close();
  if (rows.every((r) => r.state === "complete" || r.state === "failed" || r.state === "paused")) break;
  await sleep(2000);
}
let fails = 0;
let dups = 0;
for (const t of results) {
  const job = rows.find((r) => r.name === `b09-${t.i}.zip`);
  const file = [path.join(dl, `b09-${t.i}.zip`), path.join(dl, "Compressed", `b09-${t.i}.zip`)].find((f) => fs.existsSync(f)) ?? ""; // .zip -> category "Compressed"
  const done = !!job && job.state === "complete" && !!file && fs.statSync(file).size === size;
  const ok = !t.acked || done; // acked => GDM must deliver; not acked => the browser keeps its own copy
  if (!t.acked && job) dups++;
  if (!ok) fails++;
  console.log(`${ok ? "PASS" : "FAIL"} trial ${t.i} kill ${t.who} @${t.afterMs} ms (${t.phase}): ack=${t.acked} gdm=${job ? job.state : "no job"}${done ? " file complete" : ""}`);
}
try { killGdm(); } catch { /* ok */ }
console.log(`never-lost violations: ${fails}; visible duplicates (no ack, GDM also has it): ${dups}`);
console.log(fails ? "B-09 FAIL" : "B-09 PASS");
process.exit(fails ? 1 : 0);
