// SPDX-License-Identifier: RPL-1.5
// Scripted backend for UI tests only (`vite --mode test`); never part of the production bundle.
// ?scene=states|empty|info|batch|xss|big&onboarded=0|1&lang=km|en&theme=dark|light
import type { Row, Settings } from "./api";

const P = new URLSearchParams(location.search);
const now = 1_790_000_000;
const MB = 1024 * 1024;

const cats = [
  ["general", "ទូទៅ", "General", "", ""],
  ["compressed", "ឯកសារបង្ហាប់", "Compressed", "Compressed", "zip rar 7z gz"],
  ["documents", "ឯកសារ", "Documents", "Documents", "pdf doc docx"],
  ["music", "តន្ត្រី", "Music", "Music", "mp3 wav m4a"],
  ["programs", "កម្មវិធី", "Programs", "Programs", "exe msi iso"],
  ["video", "វីដេអូ", "Video", "Video", "mp4 mkv webm"],
].map(([id, name_km, name_en, sub, exts]) => ({ id, name_km, name_en, sub, exts: exts.split(" ").filter(Boolean) }));

let settings: Settings = {
  lang: P.get("lang") ?? "km",
  theme: P.get("theme") ?? "dark",
  onboarded: P.get("onboarded") !== "0",
  download_dir: "C:\\Users\\user\\Downloads",
  max_conns: 8,
  limit_on: false,
  limit_kbps: 0,
  catch_enabled: true,
  catch_exts: ["zip", "exe", "iso", "mp4"],
  catch_min_kb: 1024,
  catch_skip: ["*.reddit.com/compression-dictionary*"],
  force_key: "ctrl",
  bypass_key: "alt",
  show_info: true,
  video_button: false,
  clipboard: false,
  update_mode: "auto",
  update_channel: "stable",
  silent: false,
  autostart: false,
  columns: [],
  categories: cats,
};

const catOf = (n: string) => {
  const e = n.split(".").pop()!.toLowerCase();
  return cats.find((c) => c.exts.includes(e))?.id ?? "general";
};
const DANGER = ["exe", "msi", "bat", "cmd", "ps1", "js", "vbs", "scr", "lnk"];
let seq = 0;
function row(name: string, state: string, size: number | null, done: number, extra: Partial<Row> = {}): Row {
  const ext = name.includes(".") ? name.split(".").pop()!.toLowerCase() : "";
  return {
    id: "m" + ++seq, name, url: `https://dl.example.org/${encodeURIComponent(name)}`, dir: settings.download_dir, state, size, done,
    unverified: false, protection: state === "complete" ? "protected" : null, error_code: null, sha256: state === "complete" ? "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08" : null,
    description: null, created_at: now - seq * 60, finished_at: null, category: catOf(name), dangerous: DANGER.includes(ext), ext, ...extra,
  };
}

// X-01 test payloads: plain strings rendered as text; nothing here is ever evaluated
const XSS = [
  `<img src=x onerror="window.__xss=1">.zip`, `"><script>window.__xss=1</script>.exe`, `<svg onload=window.__xss=1>.iso`, `javascript:window.__xss=1//.pdf`,
  `<iframe srcdoc="<script>parent.__xss=1</script>">.mp4`, `{{constructor.constructor('window.__xss=1')()}}.zip`, `<a href=javascript:window.__xss=1>x</a>.zip`,
  `<body onload=window.__xss=1>.zip`, `<details open ontoggle=window.__xss=1>.zip`, `<math><mi xlink:href="javascript:window.__xss=1">x</mi></math>.zip`,
  `<style>@import'javascript:window.__xss=1'</style>.zip`, `<video><source onerror=window.__xss=1></video>.mp4`, `<input autofocus onfocus=window.__xss=1>.zip`,
  `<marquee onstart=window.__xss=1>.zip`, `<object data=javascript:window.__xss=1>.zip`, `<embed src=javascript:window.__xss=1>.zip`,
  `<form action=javascript:window.__xss=1><button>x</button></form>.zip`, `<isindex action=javascript:window.__xss=1>.zip`, `<link rel=import href=data:text/html,<script>window.__xss=1</script>>.zip`,
  `<meta http-equiv=refresh content="0;url=javascript:window.__xss=1">.zip`, `<base href=javascript:window.__xss=1//>.zip`, `<table background=javascript:window.__xss=1>.zip`,
  `<div style="background:url(javascript:window.__xss=1)"r=eval(atob('d2luZG93Ll9feHNzPTE='))>.zip`,
  `<svg><animate onbegin=window.__xss=1 attributeName=x dur=1s>.zip`, `<audio src onloadstart=window.__xss=1>.mp3`, `<keygen autofocus onfocus=window.__xss=1>.zip`,
  `<textarea autofocus onfocus=window.__xss=1>.zip`, `<select autofocus onfocus=window.__xss=1>.zip`,
];

function scene(): Row[] {
  switch (P.get("scene")) {
    case "empty":
      return [];
    case "xss":
      return XSS.map((n, i) => row(n, i % 2 ? "complete" : "paused", 1000 + i, i % 2 ? 1000 + i : 10, { url: `https://e.com/${i}?q=<script>window.__xss=1</script>`, description: n }));
    case "big":
      return Array.from({ length: 10_000 }, (_, i) => row(i % 97 === 0 ? `មេរៀនគណិតវិទ្យា-${i}.pdf` : `setup-tool-${i}.zip`, "complete", 1_000_000 + i, 1_000_000 + i));
    default:
      return [
        row("ubuntu-24.04.3-desktop-amd64.iso", "downloading", 5.9 * 1024 * MB, 3.7 * 1024 * MB),
        row("វគ្គសិក្សា Python មេរៀនទី៣ - អថេរ និងប្រភេទទិន្នន័យ.mp4", "downloading", 1.2 * 1024 * MB, 340 * MB),
        row("ubuntu-24.04.3-desktop-amd64.iso.zip", "finalizing", 5.9 * 1024 * MB, 5.9 * 1024 * MB),
        row("សៀវភៅណែនាំ-គណិតវិទ្យា-ថ្នាក់ទី១២.pdf", "complete", 18 * MB, 18 * MB, { unverified: true }),
        row("VLC-3.0.22-win64.exe", "unprotected", 44 * MB, 44 * MB, { protection: "unprotected" }),
        row("free-crack-installer.exe", "blocked", 2 * MB, 2 * MB, { error_code: "Blocked: Blocked(\"removed\")" }),
        row("dataset-2026-q3.zip", "paused", 2.4 * 1024 * MB, 900 * MB, { error_code: "FileChanged: FileChanged" }),
        row("មេរៀន-ភាសាអង់គ្លេស-វគ្គ៤.mp4", "paused", 860 * MB, 300 * MB, { error_code: "NeedsDecision: NeedsDecision(\"no strong validator\")" }),
        row("backup-nas.7z", "failed", null, 0, { error_code: 'Forbidden: Forbidden("192.168.1.20 -> 192.168.1.20 (private or special address)")' }),
        row("driver.msi", "failed", null, 0, { error_code: 'Tls: Tls("invalid peer certificate")' }),
        row("report-q3.pdf", "paused", 4 * MB, 2 * MB, { error_code: "Html: Html" }),
        row("node-v24.11.0-x64.msi", "complete", 31 * MB, 31 * MB),
        row("podcast-ep-112.mp3", "paused", 58 * MB, 12 * MB),
      ];
  }
}

const jobs: Row[] = scene();
const pending: string[] = P.get("scene") === "info" ? [jobs[0].id] : P.get("scene") === "batch" ? jobs.slice(0, 5).map((j) => j.id) : [];
let rev = 1;
const calls: [string, unknown][] = [];
(window as unknown as { __calls: typeof calls }).__calls = calls;

export async function mock(cmd: string, args: Record<string, unknown>): Promise<unknown> {
  calls.push([cmd, args]);
  await new Promise((r) => setTimeout(r, 25)); // like real IPC: the UI must render before the first answers arrive
  const q = String(args.q ?? "").toLowerCase();
  switch (cmd) {
    case "settings_get":
      return settings;
    case "settings_set":
      settings = { ...settings, ...(args.patch as Partial<Settings>) };
      return settings;
    case "snapshot":
      return { rev, jobs: q ? jobs.filter((j) => j.name.toLowerCase().includes(q) || j.url.toLowerCase().includes(q)) : jobs };
    case "tick": {
      const live = jobs
        .filter((j) => j.state === "downloading")
        .map((j) => ({ id: j.id, done: j.done, total: j.size ?? 0, speed: 8.4 * MB, conns: 6, segs: [[0, (j.size ?? 0) / 2, j.done / 2], [(j.size ?? 0) / 2, j.size ?? 0, (j.size ?? 0) / 2 + j.done / 2]] }));
      return {
        rev, live, pending: jobs.filter((j) => pending.includes(j.id)), toasts: [], banner: null, store_build: false,
        update: { state: P.get("update") ?? "idle", version: "0.9.1", security: P.get("update") === "ready", notes_km: null, notes_en: null, error: null },
      };
    }
    case "browsers":
      return [
        { key: "chrome", name: "Chrome", installed: true, connected: true },
        { key: "edge", name: "Edge", installed: true, connected: false },
        { key: "firefox", name: "Firefox", installed: true, connected: false },
      ];
    case "folder_check":
      return { fs: "NTFS", free: 120 * 1024 * MB, dir: settings.download_dir };
    case "job_detail": {
      const j = jobs.find((x) => x.id === args.id)!;
      return { url: j.url, final_url: j.url, referrer: null, dir: j.dir, sha256: j.sha256, segments: [], history: [] };
    }
    case "info_confirm":
    case "info_cancel":
      pending.splice(0, pending.length, ...pending.filter((p) => p !== args.id));
      return args.id;
    case "add_url": {
      const r = row(String(args.url).split("/").pop() || "file", "downloading", 10 * MB, 0, { url: String(args.url) });
      jobs.unshift(r);
      pending.push(r.id);
      rev++;
      return r.id;
    }
    case "job_remove": {
      const i = jobs.findIndex((j) => j.id === args.id);
      if (i >= 0) jobs.splice(i, 1);
      rev++;
      return null;
    }
    default:
      return null;
  }
}
