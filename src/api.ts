// SPDX-License-Identifier: RPL-1.5
// Typed wrappers around GDM's Tauri commands. Every command takes a job id or a plain value (X-01, X-02).
// Outside Tauri (Playwright UI tests, `vite --mode test`) a scripted mock backend answers instead.
import { invoke as tauriInvoke } from "@tauri-apps/api/core";

export interface Row {
  id: string;
  name: string;
  url: string;
  dir: string;
  state: string;
  size: number | null;
  done: number;
  unverified: boolean;
  protection: string | null;
  error_code: string | null;
  sha256: string | null;
  description: string | null;
  created_at: number;
  finished_at: number | null;
  category: string;
  dangerous: boolean;
  ext: string;
  queue: string | null;
  qpos: number | null;
  retries: number;
}
export interface Live { id: string; done: number; total: number; speed: number; conns: number; segs: [number, number, number][] }
export interface Toast { n: number; kind: string; id: string; name: string; dangerous: boolean }
export interface UpdateStatus { state: string; version: string | null; security: boolean; notes_km: string | null; notes_en: string | null; error: string | null }
export interface Tick { rev: number; live: Live[]; pending: Row[]; toasts: Toast[]; update: UpdateStatus; banner: string | null; store_build: boolean }
export interface Category { id: string; name_km: string; name_en: string; sub: string; exts: string[]; sites: string[]; dir: string }
export interface Settings {
  lang: string; theme: string; onboarded: boolean; download_dir: string; max_conns: number; limit_on: boolean; limit_kbps: number;
  catch_enabled: boolean; catch_exts: string[]; catch_min_kb: number; catch_skip: string[]; force_key: string; bypass_key: string;
  show_info: boolean; video_button: boolean; clipboard: boolean; update_mode: string; update_channel: string; silent: boolean;
  autostart: boolean; columns: string[]; categories: Category[];
  queues: Queue[]; total_conns: number; host_limits: HostLimit[]; quota_mb: number; quota_hours: number;
  proxy: Proxy; sounds: Partial<Record<SoundEvent, string>>;
  user_agent: string; site_agents: { host: string; ua: string }[]; hash_sidecar: boolean;
  /** read-only here: written only by hook_set (queue ids and "scan") */
  hooks?: Record<string, Hook>;
}
export interface Schedule { enabled: boolean; start: string | null; stop: string | null; days: number; date: string | null; every_min: number | null }
export interface Queue {
  id: string; name: string; sync: boolean; max_active: number; retries: number; schedule: Schedule;
  on_done: "none" | "exit" | "sleep" | "hibernate" | "shutdown"; force: boolean; run_hook: boolean;
  speed_kbps: number; quota_mb: number; quota_hours: number; wake: boolean; sync_metered: boolean;
}
export interface QState { active: boolean; manual_stop: boolean; key: string | null; gen: number; countdown: number | null; action: string | null }
export interface Hook { program: string; args: string[] }
export interface QueueView { def: Queue; state: QState; jobs: string[]; hook: Hook | null }
export interface HostLimit { host: string; conns: number }
export interface Proxy { mode: "none" | "system" | "manual"; http: string; socks: string; bypass: string[]; auth: "none" | "basic" | "windows"; user: string; auth_host: string }
export type SoundEvent = "complete" | "failed" | "queue";
export interface ImportItem { url: string; name: string; referrer: string; description: string }
export interface ImportPreview { count: number; sample: ImportItem[]; has_settings: boolean }
export interface Batch { pattern: string; from: string; to: string; width: number }
// F09 site grabber (G9)
export interface GrabConfig {
  start: string; avoid: string; depth: number; other_depth: number; domains: string; subdomains: boolean; no_parent: boolean;
  include: string; exclude: string; robots: boolean; js: boolean; delay_ms: number; types: string;
}
export interface GrabFound { url: string; referrer: string; name: string; size: number | null; dup: boolean }
export interface GrabReport { files: GrabFound[]; pages: number; requests: number; bytes: number; stop: string; skipped: { url: string; reason: string }[] }
export interface GrabProgress { pages: number; requests: number; files: number; bytes: number; url: string }
export interface BatchPreview { count: number; first: string[]; needs_confirm: boolean }
export interface RefreshCheck { size: number | null; same_size: boolean; strong: boolean; seam: boolean | null }
export interface Candidate { url: string; check: RefreshCheck | null; error: string | null }
export interface SiteLogin { id: string; prefix: string; user: string }
export interface MediaChoice { label: string; height: number | null; container: string; est_size: number | null }
export interface Probe { token: string; title: string; choices: MediaChoice[] }
export interface Detail { url: string; final_url: string | null; referrer: string | null; dir: string; sha256: string | null; segments: [number, number, number][]; history: [string, string | null][] }
export interface Browser { key: string; name: string; installed: boolean; connected: boolean }
export interface FolderCheck { fs: string; free: number; dir: string }

type Args = Record<string, unknown>;
let backend: ((cmd: string, args: Args) => Promise<unknown>) | null = null;

export const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function call<T>(cmd: string, args: Args = {}): Promise<T> {
  if (inTauri) return tauriInvoke<T>(cmd, args);
  if (!backend && import.meta.env.MODE === "test") backend = (await import("./mock")).mock;
  if (!backend) throw new Error("GDM backend not available");
  return backend(cmd, args) as Promise<T>;
}

export const api = {
  snapshot: (q: string | null) => call<{ rev: number; jobs: Row[] }>("snapshot", { q }),
  tick: (toastAfter: number) => call<Tick>("tick", { toastAfter }),
  settingsGet: () => call<Settings>("settings_get"),
  settingsSet: (patch: Partial<Settings>) => call<Settings>("settings_set", { patch }),
  pickDownloadDir: () => call<string | null>("pick_download_dir"),
  folderCheck: () => call<FolderCheck>("folder_check"),
  addUrl: (url: string) => call<string>("add_url", { url }),
  infoConfirm: (id: string, name: string | null, category: string | null, later: boolean) => call<string>("info_confirm", { id, name, category, later }),
  infoCancel: (id: string) => call<void>("info_cancel", { id }),
  start: (id: string, decision?: "restart" | "continue") => call<void>("job_start", { id, decision: decision ?? null }),
  pause: (id: string) => call<void>("job_pause", { id }),
  remove: (id: string, deleteFile: boolean) => call<void>("job_remove", { id, deleteFile }),
  open: (id: string) => call<void>("job_open", { id }),
  folder: (id: string) => call<void>("job_folder", { id }),
  page: (id: string) => call<void>("job_page", { id }),
  desc: (id: string, text: string) => call<void>("job_desc", { id, text }),
  conns: (id: string, n: number) => call<void>("job_conns", { id, n }),
  limit: (id: string, kbps: number) => call<void>("job_limit", { id, kbps }),
  allowPrivate: (id: string) => call<void>("job_allow_private", { id }),
  detail: (id: string) => call<Detail>("job_detail", { id }),
  pauseAll: () => call<void>("pause_all"),
  resumeAll: () => call<void>("resume_all"),
  openLink: (key: string) => call<void>("open_link", { key }),
  browsers: () => call<Browser[]>("browsers"),
  updateCheck: () => call<void>("update_check"),
  updateRestart: () => call<void>("update_restart"),
  // G7: queues + scheduler (F08), import/export/batch (I-0x), refresh address, logins, proxy, sounds, video (F07)
  queueList: () => call<QueueView[]>("queue_list"),
  queuePut: (ids: string[], queue: string | null) => call<void>("queue_put", { ids, queue }),
  queueStart: (id: string) => call<void>("queue_start", { id }),
  queueStop: (id: string) => call<void>("queue_stop", { id }),
  queueCancelAction: (id: string) => call<void>("queue_cancel_action", { id }),
  hookSet: (queue: string, pick: boolean, args: string[], clear: boolean) => call<Hook | null>("hook_set", { queue, pick, args, clear }),
  soundPick: (event: SoundEvent) => call<string | null>("sound_pick", { event }),
  soundTest: (event: SoundEvent) => call<void>("sound_test", { event }),
  importPick: () => call<ImportPreview | null>("import_pick"),
  importCommit: (queue: string | null, restoreSettings: boolean) => call<number>("import_commit", { queue, restoreSettings }),
  exportPick: () => call<string | null>("export_pick"),
  batchPreview: (b: Batch) => call<BatchPreview>("batch_preview", { b }),
  batchAdd: (b: Batch, confirmed: boolean, queue: string | null) => call<number>("batch_add", { b, confirmed, queue }),
  grabRun: (cfg: GrabConfig, user: string | null, password: string | null) => call<GrabReport>("grab_run", { cfg, user, password }),
  grabProgress: () => call<GrabProgress | null>("grab_progress"),
  grabCancel: () => call<void>("grab_cancel"),
  grabPickDir: (pick: boolean) => call<string>("grab_pick_dir", { pick }),
  grabAdd: (files: GrabFound[], startUrl: string, keep: boolean, start: boolean) => call<number>("grab_add", { files, startUrl, keep, start }),
  refreshStart: (id: string) => call<void>("refresh_start", { id }),
  refreshPaste: (id: string, url: string) => call<void>("refresh_paste", { id, url }),
  refreshCandidate: (id: string) => call<Candidate | null>("refresh_candidate", { id }),
  refreshDecide: (id: string, choice: "resume" | "continue" | "restart" | "cancel") => call<string>("refresh_decide", { id, choice }),
  loginList: () => call<SiteLogin[]>("login_list"),
  loginSet: (prefix: string, user: string, password: string) => call<SiteLogin>("login_set", { prefix, user, password }),
  loginDelete: (id: string) => call<void>("login_delete", { id }),
  proxyLoginSet: (host: string, user: string, password: string, windows: boolean) => call<void>("proxy_login_set", { host, user, password, windows }),
  componentsState: () => call<{ ytdlp: boolean; ffmpeg: boolean }>("components_state"),
  componentsInstall: () => call<void>("components_install"),
  mediaProbe: (url: string) => call<Probe>("media_probe", { url }),
  mediaStart: (token: string, choice: number) => call<void>("media_start", { token, choice }),
  categoryPickDir: (id: string, clear: boolean) => call<string | null>("category_pick_dir", { id, clear }),
  jobSetUrl: (id: string, url: string) => call<void>("job_set_url", { id, url }),
  jobMove: (id: string) => call<string | null>("job_move", { id }),
};

/** a Tauri command error is a plain string code ("drm", "confirm", ...); anything else becomes its message */
export const errText = (e: unknown) => (typeof e === "string" ? e : e instanceof Error ? e.message : String(e));
/** a WAV path or program path is shown by its file name only */
export const fileName = (p: string) => p.split(/[\\/]/).pop() ?? p;

/** the http(s) links in a drop (text/uri-list, else plain text), one per line, at most 50 */
export function droppedLinks(uriList: string, text: string): string[] {
  const lines = (uriList || text).split(/\r?\n/).map((l) => l.trim());
  return [...new Set(lines.filter((l) => /^https?:\/\/\S+$/i.test(l)))].slice(0, 50);
}
