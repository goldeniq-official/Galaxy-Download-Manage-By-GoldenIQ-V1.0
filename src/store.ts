// SPDX-License-Identifier: RPL-1.5
// App state (Zustand). One poll every 500 ms (`tick`: live progress, Info queue, toasts, update state);
// the full list is fetched again only when its revision changes or the search text changes.
import { create } from "zustand";
import { api, type Live, type Row, type Settings, type Tick, type Toast } from "./api";
import i18n from "./i18n";

export type Nav = "all" | "active" | "unfinished" | "done" | `cat:${string}` | "settings";

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
  setQ: (q: string) => void;
  setNav: (n: Nav) => void;
  setSel: (id: string | null) => void;
  setAdding: (b: boolean) => void;
  dismiss: (n: number) => void;
  saveSettings: (p: Partial<Settings>) => Promise<void>;
  refresh: () => Promise<void>;
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
  const loop = async () => {
    try {
      const t = await api.tick(lastToast);
      const st = useStore.getState();
      const live: Record<string, Live> = {};
      for (const l of t.live) live[l.id] = l;
      if (t.toasts.length) lastToast = Math.max(...t.toasts.map((x) => x.n));
      set({ tick: t, live, toasts: [...st.toasts, ...t.toasts].slice(-4) });
      if (t.rev !== st.rev) await st.refresh();
    } catch {
      /* backend restarting: try again next tick */
    }
    setTimeout(loop, 500);
  };
  void loop();
}

const set = (p: Partial<S>) => useStore.setState(p);
