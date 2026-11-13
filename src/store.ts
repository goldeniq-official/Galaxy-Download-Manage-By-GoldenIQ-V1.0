// SPDX-License-Identifier: RPL-1.5
// App state (Zustand). One poll every 500 ms (`tick`: live progress, Info queue, toasts, update state);
// the full list is fetched again only when its revision changes or the search text changes.
import { create } from "zustand";
import { api, type ImportPreview, type Live, type QueueView, type Row, type Settings, type Tick, type Toast } from "./api";
import i18n from "./i18n";

export type Nav = "all" | "active" | "unfinished" | "done" | `cat:${string}` | `q:${string}`;
/** the one G7 dialog open at a time */
export type Dlg = { k: "batch" } | { k: "video"; url?: string } | { k: "import"; pv: ImportPreview } | { k: "refresh"; id: string } | { k: "url"; id: string } | { k: "grab" };

interface S {
  settings: Settings | null;
  jobs: Row[];
  live: Record<string, Live>;
  rev: number;
  q: string;
  nav: Nav;
  sel: string | null;
  toasts: Toast[];
  tick: Tick | null;
  adding: boolean;
  infoClosed: string[];
  queues: QueueView[];
  dlg: Dlg | null;
  /** a queue's "when done" countdown (toast kind "countdown"): queue id + action */
  countdown: { id: string; action: string } | null;
  /** Settings dialog open, and its section (Tasks > Scheduler opens "queues") */
  settingsOpen: boolean;
  sec: string;
  setQ: (q: string) => void;
  setNav: (n: Nav) => void;
  setSel: (id: string | null) => void;
  setAdding: (b: boolean) => void;
  dismiss: (n: number) => void;
  saveSettings: (p: Partial<Settings>) => Promise<void>;
  refresh: () => Promise<void>;
  refreshQueues: () => Promise<void>;
  setDlg: (d: Dlg | null) => void;
  openSettings: (sec: string) => void;
  /** local notices (export saved, ...) use negative numbers so they never collide with backend toasts */
  notify: (kind: string, name: string) => void;
}

let lastToast = 0;

export const useStore = create<S>((set, get) => ({
  settings: null,
  jobs: [],
  live: {},
  rev: -1,
  q: "",
  nav: "all",
  sel: null,
  toasts: [],
  tick: null,
  adding: false,
  infoClosed: [],
  queues: [],
  dlg: null,
  countdown: null,
  settingsOpen: false,
  sec: "general",
  setQ: (q) => {
    set({ q });
    void get().refresh();
  },
  setNav: (nav) => set({ nav }),
  setSel: (sel) => set({ sel }),
  setAdding: (adding) => set({ adding }),
  dismiss: (n) => set({ toasts: get().toasts.filter((t) => t.n !== n) }),
  saveSettings: async (p) => {
    const s = await api.settingsSet(p);
    set({ settings: s });
    applyLook(s);
  },
  refresh: async () => {
    const r = await api.snapshot(get().q || null);
    set({ jobs: r.jobs, rev: r.rev });
  },
  refreshQueues: async () => set({ queues: await api.queueList() }),
  setDlg: (dlg) => set({ dlg }),
  openSettings: (sec) => set({ settingsOpen: true, sec }),
  notify: (kind, name) => set({ toasts: [...get().toasts, { n: -Date.now(), kind, id: "", name, dangerous: false }].slice(-4) }),
}));

export function applyLook(s: Settings) {
  const dark = s.theme === "dark" || (s.theme === "system" && !matchMedia("(prefers-color-scheme: light)").matches);
  document.documentElement.dataset.theme = dark ? "dark" : "light";
  document.documentElement.lang = s.lang;
  if (i18n.language !== s.lang) void i18n.changeLanguage(s.lang);
}

export async function boot() {
  const s = await api.settingsGet();
  useStore.setState({ settings: s });
  applyLook(s);
  matchMedia("(prefers-color-scheme: light)").addEventListener("change", () => {
    const cur = useStore.getState().settings;
    if (cur) applyLook(cur);
  });
  await useStore.getState().refresh();
  await useStore.getState().refreshQueues();
  let n = 0;
  const loop = async () => {
    try {
      const t = await api.tick(lastToast);
      const st = useStore.getState();
      const live: Record<string, Live> = {};
      for (const l of t.live) live[l.id] = l;
      if (t.toasts.length) lastToast = Math.max(...t.toasts.map((x) => x.n));
      // a countdown is a banner with [Cancel], never a toast that times out
      const cd = t.toasts.filter((x) => x.kind === "countdown").pop();
      if (cd) set({ countdown: { id: cd.id, action: cd.name } });
      set({ tick: t, live, toasts: [...st.toasts, ...t.toasts.filter((x) => x.kind !== "countdown")].slice(-4) });
      if (t.rev !== st.rev) {
        await st.refresh();
        // settings changed outside this window (tray speed limit) bump rev too
        set({ settings: await api.settingsGet() });
      }
      // queue membership changes bump rev; running/stopped and countdowns do not: every 2 s is enough
      if (t.rev !== st.rev || t.toasts.length || ++n % 4 === 0) await st.refreshQueues();
    } catch {
      /* backend restarting: try again next tick */
    }
    setTimeout(loop, 500);
  };
  void loop();
}

const set = (p: Partial<S>) => useStore.setState(p);
