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
}
export interface Live { id: string; done: number; total: number; speed: number; conns: number; segs: [number, number, number][] }
export interface Toast { n: number; kind: string; id: string; name: string; dangerous: boolean }
export interface UpdateStatus { state: string; version: string | null; security: boolean; notes_km: string | null; notes_en: string | null; error: string | null }
export interface Tick { rev: number; live: Live[]; pending: Row[]; toasts: Toast[]; update: UpdateStatus; banner: string | null; store_build: boolean }
export interface Category { id: string; name_km: string; name_en: string; sub: string; exts: string[] }
export interface Settings {
  lang: string; theme: string; onboarded: boolean; download_dir: string; max_conns: number; limit_on: boolean; limit_kbps: number;
  catch_enabled: boolean; catch_exts: string[]; catch_min_kb: number; catch_skip: string[]; force_key: string; bypass_key: string;
  show_info: boolean; video_button: boolean; clipboard: boolean; update_mode: string; update_channel: string; silent: boolean;
  autostart: boolean; columns: string[]; categories: Category[];
}
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
};
