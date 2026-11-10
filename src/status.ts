// SPDX-License-Identifier: RPL-1.5
// Engine state -> one sentence + one action (docs/p1/plan-ux.md §2). Colour is never the only signal.
import type { Live, Row } from "./api";

export type Tone = "" | "info" | "done" | "warn" | "err";
export type Action =
  | "open" | "openFolder" | "why" | "restart" | "fresh" | "continueRisk" | "resume" | "allow" | "retry" | "openBrowser" | "newName" | "pause";

export interface View {
  key: string;
  vars?: Record<string, string | number>;
  tone: Tone;
  action?: Action;
  second?: Action;
  bar?: number;
  barTone?: "warn" | "done";
  why?: string;
}

export const code = (r: Row) => (r.error_code ?? "").split(":")[0];
const detail = (r: Row) => (r.error_code ?? "").slice(code(r).length + 1);

export function privateHost(r: Row): string {
  const m = /([^\s"(]+) -> \S+ \(private/.exec(detail(r));
  return m ? m[1] : "";
}

const pct = (done: number, total: number) => (total > 0 ? Math.min(100, Math.floor((done / total) * 100)) : 0);

export function view(r: Row, live?: Live): View {
  const total = live?.total || r.size || 0;
  const done = live?.done ?? r.done;
  switch (r.state) {
    case "downloading":
    case "queued":
      if (!live && r.state === "queued") return { key: "queued", tone: "" };
      return total ? { key: "downloading", vars: { p: pct(done, total) }, tone: "", bar: pct(done, total), action: "pause" } : { key: "downloadingUnknown", tone: "", bar: 0, action: "pause" };
    case "finalizing":
      return { key: "finalizing", tone: "info", bar: 100 };
    case "complete":
      return { key: r.unverified ? "unverified" : "complete", tone: "done", action: r.dangerous ? "openFolder" : "open", second: "openFolder" };
    case "unprotected":
      return { key: "unprotected", tone: "warn", action: "openFolder", why: "unprotected" };
    case "blocked":
      return { key: "blocked", tone: "err", action: "why", why: "blocked" };
  }
  const c = code(r);
  const paused = r.state === "paused";
  const bar = total ? pct(done, total) : undefined;
  switch (c) {
    case "":
      return paused ? { key: "paused", tone: "warn", action: "resume", bar, barTone: "warn" } : { key: "failed", tone: "err", action: "retry" };
    case "Recovered":
      return { key: "recovered", tone: "warn", action: "resume", bar, barTone: "warn" };
    case "FileChanged":
    case "SizeChanged":
      return { key: "changed", tone: "warn", action: "restart", bar, barTone: "warn" };
    case "NeedsDecision":
      return detail(r).includes(".part")
        ? { key: "foreignPart", tone: "warn", action: "newName" }
        : { key: "askResume", tone: "warn", action: "fresh", second: "continueRisk", bar, barTone: "warn" };
    case "DiskFull":
      return { key: "diskFull", tone: "err", action: "retry" };
    case "SizeExceeded":
      return { key: "sizeExceeded", tone: "warn", action: "retry" };
    case "Auth":
      return { key: "auth", tone: "warn", action: "openBrowser" };
    case "Gone":
      return { key: "gone", tone: "err", action: "openBrowser" };
    case "Html":
      return { key: "html", tone: "err", action: "openBrowser" };
    case "Forbidden":
      return privateHost(r) ? { key: "privateNet", vars: { host: privateHost(r) }, tone: "warn", action: "allow" } : { key: "insecure", tone: "err", action: "why", why: "insecure" };
    case "Tls":
      return { key: "tls", tone: "err", action: "why", why: "tls" };
    case "Timeout":
      return { key: "timeout", tone: "warn", action: "retry" };
    case "TooManyRetries":
      return { key: "retries", tone: "warn", action: "retry" };
    case "HashMismatch":
      return { key: "hashMismatch", tone: "err", action: "restart" };
    case "Reparse":
      return { key: "reparse", tone: "err" };
    case "BadContentRange":
    case "LongBody":
    case "Multipart":
      return { key: "badRange", tone: "err", action: "retry" };
    case "Blocked":
      return { key: "blocked", tone: "err", action: "why", why: "blocked" };
    default:
      return { key: "failed", tone: "err", action: "retry" };
  }
}

export const isActive = (r: Row) => r.state === "downloading" || r.state === "queued" || r.state === "finalizing";
export const isDone = (r: Row) => r.state === "complete" || r.state === "unprotected";

export function bytes(n: number | null | undefined): string {
  if (n == null) return "—";
  const u = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  let v = n;
  while (v >= 1024 && i < u.length - 1) {
    v /= 1024;
    i++;
  }
  return `${i === 0 ? v : v.toFixed(v < 10 ? 1 : 0)} ${u[i]}`;
}

export const speed = (bps: number) => (bps > 0 ? `${bytes(bps)}/s` : "—");

export function eta(left: number, bps: number): string {
  if (!(bps > 0) || left <= 0) return "—";
  const s = Math.round(left / bps);
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const ss = s % 60;
  return h ? `${h}:${String(m).padStart(2, "0")}:${String(ss).padStart(2, "0")}` : `${m}:${String(ss).padStart(2, "0")}`;
}

export const host = (u: string) => {
  try {
    return new URL(u).host;
  } catch {
    return "";
  }
};
