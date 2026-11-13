// SPDX-License-Identifier: RPL-1.5
// Scripted backend for UI tests only (`vite --mode test`); never part of the production bundle.
// ?scene=states|empty|info|batch|xss|big&onboarded=0|1&lang=km|en&theme=dark|light
// G7: &countdown=1 (a queue's shutdown countdown starts) &comp=0 (video components not installed) &refresh=strong|html
import type { Candidate, Hook, Queue, QState, Row, Settings, SiteLogin, SoundEvent } from "./api";

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
].map(([id, name_km, name_en, sub, exts]) => ({ id, name_km, name_en, sub, exts: exts.split(" ").filter(Boolean), sites: id === "video" ? ["*.lectures.example.edu"] : [], dir: "" }));

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
  queues: [],
  total_conns: 48,
  host_limits: [{ host: "*.sourceforge.net", conns: 2 }],
  quota_mb: 0,
  quota_hours: 1,
  proxy: { mode: "none", http: "", socks: "", bypass: ["<local>"], auth: "none", user: "", auth_host: "" },
  sounds: { complete: "system:SystemAsterisk", failed: "", queue: "C:\\Windows\\Media\\chimes.wav" },
  user_agent: "",
  site_agents: [{ host: "*.mirror.example.net", ua: "Wget/1.25" }],
  hash_sidecar: true,
};
const queueDef = (id: string, name: string, sync: boolean, extra: Partial<Queue> = {}): Queue => ({
  id, name, sync, max_active: 1, retries: 10, schedule: { enabled: false, start: null, stop: null, days: 0, date: null, every_min: null },
  on_done: "none", force: false, run_hook: false, speed_kbps: 0, quota_mb: 0, quota_hours: 1, wake: false, sync_metered: false, ...extra,
});
settings.queues = [
  queueDef("main", "", false, { max_active: 2, schedule: { enabled: true, start: "23:00", stop: "06:00", days: 127, date: null, every_min: null }, on_done: "shutdown", run_hook: true }),
  queueDef("sync", "", true, { schedule: { enabled: true, start: "08:00", stop: null, days: 31, date: null, every_min: 120 } }),
  queueDef("q-1", "មេរៀនយប់", false),
];

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
    description: null, created_at: now - seq * 60, finished_at: null, category: catOf(name), dangerous: DANGER.includes(ext), ext, queue: null, qpos: null, retries: 0, ...extra,
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
// default scene: three files wait in the Main queue, one in Synchronization, one in the user queue
if (!P.get("scene") || P.get("scene") === "states")
  for (const [name, q, pos] of [["dataset-2026-q3.zip", "main", 1], ["podcast-ep-112.mp3", "main", 2], ["report-q3.pdf", "main", 3], ["node-v24.11.0-x64.msi", "sync", 1], ["មេរៀន-ភាសាអង់គ្លេស-វគ្គ៤.mp4", "q-1", 1]] as const) {
    const j = jobs.find((x) => x.name === name);
    if (j) Object.assign(j, { queue: q, qpos: pos });
  }
const qstate: Record<string, QState> = {};
const qs = (id: string) => (qstate[id] ??= { active: false, manual_stop: false, key: null, gen: 0, countdown: null, action: null });
const hooks: Record<string, Hook> = { main: { program: "C:\\Tools\\after-download.bat", args: ["{folder}"] } };
const logins: SiteLogin[] = [{ id: "l-1", prefix: "https://files.example.edu/course/", user: "student01" }];
const comps = { ytdlp: P.get("comp") !== "0", ffmpeg: P.get("comp") !== "0" };
const t0 = Date.now();
// scheduled backend toasts: tick returns those after toastAfter
const toastsOut: { n: number; kind: string; id: string; name: string; dangerous: boolean }[] = [];
if (P.get("countdown") === "1") {
  Object.assign(qs("main"), { countdown: 60, action: "shutdown" });
  toastsOut.push({ n: 1, kind: "countdown", id: "main", name: "shutdown", dangerous: false });
}
if (P.get("videolink") === "1") toastsOut.push({ n: 2, kind: "video-link", id: "", name: "https://video.example.org/watch?v=abc", dangerous: false });
let refreshPolls = 0;
let pendingImport = 0;
const fail = (code: string) => Promise.reject(code); // Tauri rejects with the command's error string
type B = { pattern: string; from: string; to: string; width: number };
function batchCount(b: B): number {
  if (b.pattern.split("*").length !== 2 || !/^https?:\/\//i.test(b.pattern)) throw "bad-pattern";
  const num = /^\d+$/.test(b.from) && /^\d+$/.test(b.to);
  if (!num && !(/^[a-z]$/i.test(b.from) && /^[a-z]$/i.test(b.to))) throw "bad-range";
  const [a, z] = num ? [Number(b.from), Number(b.to)] : [b.from.charCodeAt(0), b.to.charCodeAt(0)];
  if (z < a) throw "bad-range";
  return z - a + 1;
}
function batchUrls(b: B, n: number): string[] {
  const num = /^\d+$/.test(b.from);
  const a = num ? Number(b.from) : b.from.charCodeAt(0);
  return Array.from({ length: n }, (_, i) => b.pattern.replace("*", num ? String(a + i).padStart(b.width, "0") : String.fromCharCode(a + i)));
}
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
      return { ...settings, hooks: { ...hooks } };
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
        rev, live, pending: jobs.filter((j) => pending.includes(j.id)), toasts: toastsOut.filter((x) => x.n > Number(args.toastAfter ?? 0)), banner: null, store_build: false,
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
    case "queue_list": {
      const m = qs("main");
      if (m.countdown != null) m.countdown = Math.max(0, 60 - Math.floor((Date.now() - t0) / 1000));
      return settings.queues.map((def) => ({
        def, state: qs(def.id), hook: hooks[def.id] ?? null,
        jobs: jobs.filter((j) => j.queue === def.id).sort((a, b) => (a.qpos ?? 0) - (b.qpos ?? 0)).map((j) => j.id),
      }));
    }
    case "queue_put": {
      const q = args.queue as string | null;
      let pos = Math.max(0, ...jobs.filter((j) => q && j.queue === q).map((j) => j.qpos ?? 0));
      for (const id of args.ids as string[]) {
        const j = jobs.find((x) => x.id === id);
        if (j) Object.assign(j, { queue: q, qpos: q ? ++pos : null });
      }
      rev++;
      return null;
    }
    case "queue_start":
      Object.assign(qs(String(args.id)), { active: true, manual_stop: false });
      return null;
    case "queue_stop":
      Object.assign(qs(String(args.id)), { active: false, manual_stop: true });
      return null;
    case "queue_cancel_action":
      Object.assign(qs(String(args.id)), { countdown: null, action: null });
      return null;
    case "hook_set": {
      const q = String(args.queue);
      if (args.clear) {
        delete hooks[q];
        return null;
      }
      if (!args.pick && !hooks[q]) return fail("no-program");
      if ((args.args as string[]).some((a) => /[&|<>^%]/.test(a))) return fail("batch-metachar");
      hooks[q] = { program: args.pick ? "C:\\Tools\\after-download.bat" : hooks[q].program, args: args.args as string[] };
      return hooks[q];
    }
    case "sound_pick":
      settings.sounds = { ...settings.sounds, [args.event as SoundEvent]: "C:\\Windows\\Media\\tada.wav" };
      return "C:\\Windows\\Media\\tada.wav";
    case "import_pick":
      pendingImport = 3;
      return {
        count: 3, has_settings: true,
        sample: [
          { url: "https://dl.example.org/lecture-01.mp4", name: "lecture-01.mp4", referrer: "https://school.example.org/", description: "មេរៀនទី១" },
          { url: "https://dl.example.org/lecture-02.mp4", name: "lecture-02.mp4", referrer: "", description: "" },
          { url: "https://mirror.example.net/tools/7z2501-x64.exe", name: "7z2501-x64.exe", referrer: "", description: "" },
        ],
      };
    case "import_commit": {
      if (!pendingImport) return fail("nothing");
      for (const n of ["lecture-01.mp4", "lecture-02.mp4", "7z2501-x64.exe"]) jobs.push(row(n, "paused", null, 0, { queue: (args.queue as string | null) ?? null }));
      rev++;
      const n = pendingImport;
      pendingImport = 0;
      return n;
    }
    case "export_pick":
      return "C:\\Users\\user\\Downloads\\gdm-export.json";
    case "grab_pick_dir":
      return "C:\\Users\\user\\Downloads\\Grabber";
    case "grab_progress":
      return { pages: 3, requests: 4, files: 2, bytes: 20480, url: "https://example.com/gallery/page2.html" };
    case "grab_cancel":
      return null;
    case "grab_run": {
      const start = String((args.cfg as { start: string }).start);
      if (!/^https?:\/\//.test(start)) return fail("bad-url");
      await new Promise((r) => setTimeout(r, 600));
      const f = (name: string, dup = false) => ({ url: new URL(name, start).href, referrer: start, name: name.split("/").pop()!, size: null, dup });
      return { files: [f("photo-1.jpg"), f("photo-2.jpg"), f("sub/photo-1.jpg", true), f("album.zip")], pages: 3, requests: 4, bytes: 20480, stop: "done", skipped: [{ url: start + "logout", reason: "avoid" }] };
    }
    case "grab_add": {
      const files = args.files as { url: string; name: string }[];
      for (const x of files) jobs.push(row(x.name, args.start ? "queued" : "paused", null, 0, { url: x.url }));
      rev++;
      return files.length;
    }
    case "batch_preview":
    case "batch_add": {
      const b = args.b as B;
      let n: number;
      try {
        n = batchCount(b);
      } catch (e) {
        return fail(String(e));
      }
      if (cmd === "batch_preview") return { count: n, first: batchUrls(b, Math.min(5, n)), needs_confirm: n > 10_000 };
      if (n > 100_000) return fail("too-many");
      if (n > 10_000 && !args.confirmed) return fail("confirm");
      for (const u of batchUrls(b, Math.min(n, 50))) jobs.push(row(u.split("/").pop() || "file", "paused", null, 0, { url: u, queue: (args.queue as string | null) ?? null }));
      rev++;
      return n;
    }
    case "refresh_start":
    case "refresh_paste":
      refreshPolls = 0;
      return null;
    case "refresh_candidate": {
      if (++refreshPolls < 2) return null;
      const j = jobs.find((x) => x.id === args.id)!;
      const c: Candidate =
        P.get("refresh") === "html"
          ? { url: "https://dl.example.org/login?next=" + encodeURIComponent(j.name), check: null, error: "Html" }
          : { url: j.url + "?token=a81f3c9e", check: { size: j.size, same_size: true, strong: P.get("refresh") === "strong", seam: true }, error: null };
      return c;
    }
    case "refresh_decide":
      return args.choice === "restart" ? "m" + ++seq : args.id;
    case "login_list":
      return logins;
    case "login_set": {
      const l = { id: "l-" + (logins.length + 1), prefix: String(args.prefix), user: String(args.user) };
      if (!/^https?:\/\/\S+$/i.test(l.prefix) || !l.user) return fail("bad-login");
      logins.push(l);
      return l;
    }
    case "login_delete":
      logins.splice(logins.findIndex((l) => l.id === args.id), 1);
      return null;
    case "proxy_login_set":
      settings.proxy = { ...settings.proxy, auth: args.windows ? "windows" : "basic", user: String(args.user), auth_host: String(args.host) };
      return null;
    case "components_state":
      return { ...comps };
    case "components_install":
      await new Promise((r) => setTimeout(r, 400));
      comps.ytdlp = comps.ffmpeg = true;
      return null;
    case "media_probe": {
      const u = String(args.url);
      await new Promise((r) => setTimeout(r, 300));
      if (!comps.ytdlp || !comps.ffmpeg) return fail("need-component");
      if (u.includes("drm")) return fail("drm");
      if (u.includes("live")) return fail("live");
      if (u.includes("none")) return fail("no-formats");
      return {
        token: "t-1", title: "វគ្គសិក្សា Python មេរៀនទី៤ - Loop",
        choices: [
          { label: "1080p", height: 1080, container: "mp4", est_size: 412 * MB },
          { label: "720p", height: 720, container: "mp4", est_size: 198 * MB },
          { label: "480p", height: 480, container: "webm", est_size: 96 * MB },
          { label: "Audio", height: null, container: "m4a", est_size: null },
        ],
      };
    }
    case "category_pick_dir": {
      const dir = args.clear ? "" : "D:\\Lectures";
      settings.categories = settings.categories.map((c) => (c.id === args.id ? { ...c, dir } : c));
      return dir;
    }
    case "job_set_url": {
      const j = jobs.find((x) => x.id === args.id);
      if (!j || !/^https?:\/\/\S+$/i.test(String(args.url))) return fail("bad-url");
      j.url = String(args.url);
      rev++;
      return null;
    }
    case "job_move": {
      const j = jobs.find((x) => x.id === args.id);
      if (!j || j.done > 0) return fail("has-progress");
      j.dir = "D:\\Other";
      rev++;
      return j.id;
    }
    default:
      return null;
  }
}
