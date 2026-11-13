// SPDX-License-Identifier: RPL-1.5
// Main window (§06): rail + list (TanStack Virtual) + details pane (wide) / drawer, Download Info, toasts.
// All text is rendered as React text nodes (X-01); actions call commands with job ids only (X-02).
import { useEffect, useMemo, useRef, useState, type DragEvent, type KeyboardEvent as RKE, type MouseEvent as RME } from "react";
import { useTranslation } from "react-i18next";
import { useVirtualizer } from "@tanstack/react-virtual";
import { api, droppedLinks, errText, fileName, type Detail, type QueueView, type Row } from "./api";
import { useStore, type Nav } from "./store";
import { bytes, eta, host, isActive, isDone, speed, view, type Action } from "./status";
import { ErrLine, ftClass, ftIcon, Icon, Menu, Modal, TitleBar, type MenuGroup } from "./ui";
import { Onboarding, SettingsView } from "./Settings";
import { Countdown, G7Dialogs, qName, QueueSelect, rememberDir } from "./Dialogs";

const NONE: Row[] = []; // a stable empty list: a new [] in a selector re-renders forever (React #185)

const catName = (lang: string, c: { name_km: string; name_en: string }) => (lang === "en" ? c.name_en : c.name_km);
/** a video page (not a file): it goes to "Add video" (quality list) instead of being downloaded as an HTML page */
const videoPage = (u: string) => /^https?:\/\/((www|m|music)\.)?(youtube\.com\/(watch|shorts\/|live\/)|youtu\.be\/)/i.test(u.trim());
/** a link typed, pasted or added: video pages open the quality list, everything else is a download */
export function addLink(u: string) {
  if (videoPage(u)) return void useStore.getState().setDlg({ k: "video", url: u.trim() });
  return api.addUrl(u.trim());
}
/** a dialog is open: window-wide shortcuts (Ctrl+N, Ctrl+F, paste a link) must not act behind it */
const dialogOpen = () => !!document.querySelector(".scrim");
/** the address as people read it (Khmer, spaces) instead of %E1%9E...; display only, the stored URL is never changed */
const readable = (u: string) => {
  try {
    return decodeURI(u);
  } catch {
    return u;
  }
};

function useWide() {
  const [w, setW] = useState(window.innerWidth);
  useEffect(() => {
    const f = () => setW(window.innerWidth);
    window.addEventListener("resize", f);
    return () => window.removeEventListener("resize", f);
  }, []);
  return w;
}

export default function App() {
  const settings = useStore((s) => s.settings);
  if (!settings) return null;
  return (
    <>
      <TitleBar />
      {settings.onboarded ? <Shell /> : <Onboarding />}
    </>
  );
}

function Shell() {
  const { t, i18n } = useTranslation();
  const { nav, setNav, settings, jobs, queues } = useStore();
  const setOpen = useStore((s) => s.settingsOpen);
  const [why, setWhy] = useState<string | null>(null);
  const adding = useStore((s) => s.adding);
  const lang = i18n.language;
  const counts = useMemo(() => {
    const c: Record<string, number> = { all: jobs.length, active: 0, unfinished: 0, done: 0 };
    for (const j of jobs) {
      if (isActive(j)) c.active++;
      if (isDone(j)) c.done++;
      else c.unfinished++;
      c["cat:" + j.category] = (c["cat:" + j.category] ?? 0) + 1;
    }
    return c;
  }, [jobs]);
  const navs: [Nav, string, string][] = [
    ["all", "list-bullets", t("nav.all")],
    ["active", "download-simple", t("nav.active")],
    ["unfinished", "clock", t("nav.unfinished")],
    ["done", "check-circle", t("nav.done")],
  ];
  useEffect(() => {
    const k = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key.toLowerCase() === "n" && !dialogOpen()) {
        e.preventDefault();
        useStore.getState().setAdding(true);
      }
    };
    window.addEventListener("keydown", k);
    return () => window.removeEventListener("keydown", k);
  }, []);
  return (
    <>
      <Countdown />
      <div id="shell" className="win">
        <div className="body">
          <nav className="rail" aria-label={t("nav.cats")}>
            {navs.map(([k, ic, label]) => (
              <NavBtn key={k} k={k} ic={ic} label={label} n={counts[k] ?? 0} sel={nav === k} onClick={() => setNav(k)} />
            ))}
            <div className="grp">{t("nav.cats")}</div>
            {settings!.categories.map((c) => (
              <NavBtn key={c.id} k={`cat:${c.id}`} ic={c.id === "general" ? "folder" : ftIcon(c.id)} label={catName(lang, c)} n={counts["cat:" + c.id] ?? 0} sel={nav === `cat:${c.id}`} onClick={() => setNav(`cat:${c.id}`)} />
            ))}
            <div className="grp">{t("nav.queues")}</div>
            {queues.map((q) => (
              <NavBtn key={q.def.id} k={`q:${q.def.id}`} ic={q.def.sync ? "arrow-clockwise" : "queue"} label={qName(t, q.def)} n={q.jobs.length} sel={nav === `q:${q.def.id}`} onClick={() => setNav(`q:${q.def.id}`)} />
            ))}
            <div className="sp" />
            <NavBtn k="settings" ic="gear-six" label={t("nav.settings")} sel={setOpen} onClick={() => useStore.setState({ settingsOpen: true })} />
          </nav>
          <Main onWhy={setWhy} />
        </div>
      </div>
      {setOpen && <SettingsView />}
      <InfoDialog />
      {adding && <AddDialog />}
      {why && <WhyDialog k={why} onClose={() => setWhy(null)} />}
      <G7Dialogs />
      <Toasts />
    </>
  );
}

function NavBtn({ ic, label, n, sel, onClick }: { k: string; ic: string; label: string; n?: number; sel: boolean; onClick: () => void }) {
  // labels are visible from 1180 px (rail 220 px); below that the icon button needs its name
  const showLabel = useWide() >= 1180;
  return (
    <button type="button" className={"fx nav" + (sel ? " is-selected" : "")} aria-current={sel ? "page" : undefined} aria-label={showLabel ? undefined : label} title={label} onClick={onClick}>
      <Icon n={ic} lg fill={sel} />
      <span className="lbl">{label}</span>
      {n != null && (
        <span className="n" aria-hidden="true">
          {n}
        </span>
      )}
    </button>
  );
}

export type Col = "name" | "size" | "status" | "speed" | "eta" | "added";
const ALL: Col[] = ["name", "size", "status", "speed", "eta", "added"];
const DEF: Col[] = ["name", "size", "status", "speed", "eta"];
const WIDTH: Record<Col, string> = { name: "minmax(0,1.3fr)", size: "76px", status: "minmax(150px,1fr)", speed: "84px", eta: "64px", added: "92px" };
// ponytail: the sort rides in settings.columns as one "sort:<col>:<asc|desc>" entry (no separate settings field)
function colModel(list: string[]) {
  const shown = list.filter((c): c is Col => (ALL as string[]).includes(c));
  const sort = list.find((c) => c.startsWith("sort:"))?.split(":") ?? [];
  return { shown: shown.length ? shown : DEF, sortBy: (sort[1] as Col) || null, desc: sort[2] === "desc" };
}

function filterRows(jobs: Row[], nav: Nav, queues: QueueView[]) {
  if (nav.startsWith("q:")) {
    // a queue shows its files in queue order (IDM "Files in the queue")
    const by = new Map(jobs.map((j) => [j.id, j]));
    return (queues.find((q) => q.def.id === nav.slice(2))?.jobs ?? []).flatMap((id) => by.get(id) ?? []);
  }
  if (nav === "active") return jobs.filter(isActive);
  if (nav === "done") return jobs.filter(isDone);
  if (nav === "unfinished") return jobs.filter((j) => !isDone(j));
  if (nav.startsWith("cat:")) return jobs.filter((j) => j.category === nav.slice(4));
  return jobs;
}

export async function act(r: Row, a: Action | undefined, onWhy: (k: string) => void, why?: string) {
  switch (a) {
    case "open":
      return api.open(r.id);
    case "openFolder":
      return api.folder(r.id);
    case "why":
      return onWhy(why ?? "blocked");
    case "restart":
    case "fresh":
      return api.start(r.id, "restart");
    case "continueRisk":
      return api.start(r.id, "continue");
    case "resume":
    case "retry":
      return api.start(r.id);
    case "allow":
      return api.allowPrivate(r.id);
    case "openBrowser":
      return api.page(r.id);
    case "pause":
      return api.pause(r.id);
    case "findVideo":
      return useStore.getState().setDlg({ k: "video", url: r.url });
    case "newName": {
      const id = await api.addUrl(r.url);
      return id;
    }
  }
}

function Main({ onWhy }: { onWhy: (k: string) => void }) {
  const { t, i18n } = useTranslation();
  const { jobs, nav, q, setQ, sel, setSel, live, tick, settings, queues } = useStore();
  const { shown, sortBy, desc } = colModel(settings!.columns);
  const rows = useMemo(() => {
    const r = filterRows(jobs, nav, queues);
    if (!sortBy || nav.startsWith("q:")) return r; // a queue keeps its own order
    const key = (j: Row): number | string =>
      sortBy === "name" ? j.name.toLowerCase() : sortBy === "size" ? (j.size ?? -1) : sortBy === "status" ? j.state : sortBy === "added" ? j.created_at : sortBy === "speed" ? (live[j.id]?.speed ?? 0) : (live[j.id] ? (live[j.id].total - live[j.id].done) / (live[j.id].speed || 1) : Infinity);
    return [...r].sort((a, b) => {
      const x = key(a), y = key(b);
      return (x < y ? -1 : x > y ? 1 : 0) * (desc ? -1 : 1);
    });
    // live speed changes every tick: sorting by speed/eta re-sorts then, by design
  }, [jobs, nav, queues, sortBy, desc, live]);
  const [menu, setMenu] = useState<{ x: number; y: number; label: string; groups: MenuGroup[] } | null>(null);
  const w = useWide();
  const [drawer, setDrawer] = useState(false);
  const wide = w >= 1180;
  // toolbar words need ~665 px beside the 340 px pane and the 220 px rail: from 1260 px (or any width without the pane)
  const words = w >= 1260 || (wide && !jobs.length);
  // compact (< 760 px): name + status; mini (< 480 px): no icon either; wide: speed and time left live in the pane
  const cols = w < 760 ? shown.filter((c) => c === "name" || c === "status").concat(shown.includes("status") ? [] : ["status"]) : wide ? shown.filter((c) => c !== "speed" && c !== "eta") : shown;
  const icon = w >= 480;
  const grid = [...(icon ? ["30px"] : []), ...cols.map((c) => (w < 760 && c === "status" ? "minmax(100px,.9fr)" : WIDTH[c]))].join(" ");
  const saveCols = (list: Col[], sort: [Col, boolean] | null) => void useStore.getState().saveSettings({ columns: [...list, ...(sort ? [`sort:${sort[0]}:${sort[1] ? "desc" : "asc"}`] : [])] });
  const sortClick = (c: Col) => saveCols(shown, sortBy !== c ? [c, false] : !desc ? [c, true] : null);
  const colMenu = (c: Col, x: number, y: number) => {
    const i = shown.indexOf(c);
    const move = (d: number) => {
      const l = [...shown];
      [l[i], l[i + d]] = [l[i + d], l[i]];
      saveCols(l, sortBy ? [sortBy, desc] : null);
    };
    setMenu({
      x,
      y,
      label: t("cols.menu"),
      groups: [
        {
          label: t("cols.menu"),
          items: ALL.map((k) => ({
            label: t(`cols.${k}`),
            checked: shown.includes(k),
            disabled: k === "name",
            onClick: () => saveCols(shown.includes(k) ? shown.filter((x) => x !== k) : [...shown, k], sortBy ? [sortBy, desc] : null),
          })),
        },
        {
          items: [
            { label: t("cols.left", { c: t(`cols.${c}`) }), disabled: i <= 0, onClick: () => move(-1) },
            { label: t("cols.right", { c: t(`cols.${c}`) }), disabled: i < 0 || i >= shown.length - 1, onClick: () => move(1) },
          ],
        },
      ],
    });
  };
  const scroller = useRef<HTMLDivElement>(null);
  const v = useVirtualizer({ count: rows.length, getScrollElement: () => scroller.current, estimateSize: () => 62, overscan: 8 });
  const selIdx = rows.findIndex((r) => r.id === sel);
  const selected = selIdx >= 0 ? rows[selIdx] : null;
  const total = Object.values(live).reduce((a, l) => a + l.speed, 0);
  const title = nav.startsWith("cat:")
    ? catName(i18n.language, settings!.categories.find((c) => c.id === nav.slice(4)) ?? { name_km: "", name_en: "" })
    : nav.startsWith("q:")
      ? qName(t, settings!.queues.find((x) => x.id === nav.slice(2)))
      : t(`title.${nav}`);
  const at = (el: Element) => {
    const b = el.getBoundingClientRect();
    return { x: b.left, y: b.bottom + 4 };
  };
  const after = () => void useStore.getState().refreshQueues();
  const qItems = (f: (id: string) => Promise<void>) => settings!.queues.map((x) => ({ label: qName(t, x), onClick: () => void f(x.id).then(after) }));
  // one Queues menu (IDM Start queue / Stop queue): each queue appears once, under the action it can take now
  const running = (id: string) => !!queues.find((x) => x.def.id === id)?.state.active;
  const queueMenu = (el: Element) =>
    setMenu({
      ...at(el),
      label: t("nav.queues"),
      groups: [
        { label: t("q.start"), items: settings!.queues.filter((x) => !running(x.id)).map((x) => ({ label: qName(t, x), onClick: () => void api.queueStart(x.id).then(after) })) },
        { label: t("q.stop"), items: settings!.queues.filter((x) => running(x.id)).map((x) => ({ label: qName(t, x), onClick: () => void api.queueStop(x.id).then(after) })) },
      ].filter((g) => g.items.length),
    });
  // IDM context menu: Refresh download address, Add to queue, Delete from queue
  // every completed download, not only those matching the search box (the list in the store is the searched one)
  const removeDone = async () => {
    for (const j of (await api.snapshot(null)).jobs.filter(isDone)) await api.remove(j.id, false).catch(() => {});
  };
  const rowMenu = (r: Row, x: number, y: number) => {
    const st = view(r, live[r.id]);
    const own = [...new Set([st.action, st.second].filter((a): a is Action => !!a))];
    setMenu({
      x,
      y,
      label: t("q.menu"),
      groups: [
        ...(own.length ? [{ items: own.map((a) => ({ label: t(`act.${a}`), onClick: () => void act(r, a, onWhy, st.why) })) }] : []),
        {
          items: [
            { label: t("q.refresh"), disabled: isDone(r), onClick: () => useStore.getState().setDlg({ k: "refresh", id: r.id }) },
            { label: t("prop.url"), disabled: isActive(r) || isDone(r), onClick: () => useStore.getState().setDlg({ k: "url", id: r.id }) },
            {
              label: t("act.moveTo"),
              disabled: isActive(r) || isDone(r) || r.done > 0,
              onClick: () =>
                void api.jobMove(r.id).then(
                  (nid) => nid && useStore.getState().setSel(nid),
                  (e) => useStore.getState().notify("error", t(`prop.e.${errText(e)}`, { defaultValue: errText(e) })),
                ),
            },
          ],
        },
        { label: t("q.addTo"), items: qItems((id) => api.queuePut([r.id], id)).filter((_, i) => settings!.queues[i].id !== r.queue) },
        ...(r.queue ? [{ items: [{ label: t("q.removeFrom"), onClick: () => void api.queuePut([r.id], null).then(after) }] }] : []),
        {
          items: [
            { label: t("act.remove"), disabled: isActive(r), onClick: () => void api.remove(r.id, false) },
            ...(isDone(r) ? [{ label: t("act.removeFile"), onClick: () => void api.remove(r.id, true) }] : []),
            { label: t("act.removeDone"), disabled: !jobs.some(isDone), onClick: () => void removeDone() },
          ],
        },
      ],
    });
  };
  const tasks = (el: Element) => {
    const s = useStore.getState();
    setMenu({
      ...at(el),
      label: t("tasks.menu"),
      groups: [
        {
          items: [
            { label: t("tasks.batch"), onClick: () => s.setDlg({ k: "batch" }) },
            { label: t("tasks.video"), onClick: () => s.setDlg({ k: "video" }) },
          ],
        },
        {
          items: [
            { label: t("tasks.import"), onClick: () => void api.importPick().then((pv) => pv && s.setDlg({ k: "import", pv }), (e) => s.notify("error", errText(e))) },
            { label: t("tasks.export"), onClick: () => void api.exportPick().then((p) => p && s.notify("exported", fileName(p)), (e) => s.notify("error", errText(e))) },
          ],
        },
        {
          items: [
            { label: t("tasks.scheduler"), onClick: () => s.openSettings("queues") },
            { label: t("act.removeDone"), onClick: () => void removeDone() },
          ],
        },
        { items: [{ label: t("tasks.grab"), onClick: () => s.setDlg({ k: "grab" }) }] },
      ],
    });
  };
  const searchRef = useRef<HTMLInputElement>(null);

  const onKey = (e: RKE) => {
    if (!rows.length || (e.target as HTMLElement).closest(".cols")) return;
    const go = (i: number) => {
      const k = Math.max(0, Math.min(rows.length - 1, i));
      setSel(rows[k].id);
      v.scrollToIndex(k);
      requestAnimationFrame(() => document.getElementById("row-" + rows[k].id)?.focus());
    };
    if (e.key === "ArrowDown") go(selIdx + 1);
    else if (e.key === "ArrowUp") go(selIdx - 1);
    else if (e.key === "Home") go(0);
    else if (e.key === "End") go(rows.length - 1);
    else if (!selected) return;
    else if (e.key === "Enter") void act(selected, view(selected, live[selected.id]).action, onWhy, view(selected).why);
    else if (e.key === " ") void (isActive(selected) ? api.pause(selected.id) : api.start(selected.id));
    else if (e.key === "Delete") void api.remove(selected.id, e.shiftKey);
    else if (e.key === "ContextMenu" || (e.shiftKey && e.key === "F10")) {
      const b = document.getElementById("row-" + selected.id)!.getBoundingClientRect();
      rowMenu(selected, b.left + 40, b.bottom);
    }
    else return;
    e.preventDefault();
  };
  useEffect(() => {
    const k = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key.toLowerCase() === "f" && !dialogOpen()) {
        e.preventDefault();
        searchRef.current?.focus();
      }
    };
    const paste = (e: ClipboardEvent) => {
      const el = document.activeElement;
      if (dialogOpen() || (el && (el.tagName === "INPUT" || el.tagName === "TEXTAREA"))) return;
      const s = e.clipboardData?.getData("text/plain")?.trim() ?? "";
      if (/^https?:\/\/\S+$/i.test(s)) void addLink(s);
    };
    window.addEventListener("keydown", k);
    window.addEventListener("paste", paste);
    return () => {
      window.removeEventListener("keydown", k);
      window.removeEventListener("paste", paste);
    };
  }, []);
  // P1 #21: links dropped on the window (the WebView gets the drop: dragDropEnabled is false), at most 50
  const onDrop = async (e: DragEvent) => {
    e.preventDefault();
    // a dropped video page opens Add video (the first one; one dialog at a time), every other link is a download
    let video = false;
    for (const u of droppedLinks(e.dataTransfer.getData("text/uri-list"), e.dataTransfer.getData("text/plain"))) {
      if (videoPage(u)) {
        if (!video) void addLink(u);
        video = true;
      } else await api.addUrl(u).catch(() => {});
    }
  };
  const upd = tick?.update;
  const banner = tick?.banner;
  return (
    // no downloads at all: no details pane (nothing to select); keyed on all jobs, not the filtered rows, so switching views never resizes the list
    <main className={"main" + (wide && jobs.length ? " split" : "")} onDragOver={(e) => e.preventDefault()} onDrop={(e) => void onDrop(e)}>
      <div className="col">
        <div className="top">
          <label className="search">
            <Icon n="magnifying-glass" />
            <input ref={searchRef} type="search" value={q} placeholder={t("search")} aria-label={t("search")} onChange={(e) => setQ(e.target.value)} />
          </label>
          {/* every toolbar button says what it does: words when there is room (>= 1180 px), icon + name on hover below that */}
          <button type="button" className="fx btn btn-p fx-strong" onClick={() => useStore.getState().setAdding(true)}>
            <Icon n="plus" />
            {t("add")}
          </button>
          <ToolBtn ic="pause" label={t("pauseAll")} words={words} onClick={() => void api.pauseAll()} />
          <ToolBtn ic="queue" label={t("nav.queues")} words={words} menu onClick={(e) => queueMenu(e.currentTarget)} />
          <ToolBtn ic="list-bullets" label={t("tasks.menu")} words={words} menu onClick={(e) => tasks(e.currentTarget)} />
          {!wide && <ToolBtn ic="info" label={t("d.title")} words={false} pressed={drawer} onClick={() => setDrawer(!drawer)} />}
        </div>
        <div className="list" ref={scroller} role="grid" aria-label={title} aria-rowcount={rows.length + 1} onKeyDown={onKey}>
          <div className="cols" role="row" aria-rowindex={1} style={{ gridTemplateColumns: grid }}>
            {icon && <span role="columnheader" aria-label={t("cols.type")} />}
            {cols.map((c) => (
              <span key={c} role="columnheader" className={c === "name" || c === "status" ? "" : "c-end"} aria-sort={sortBy === c ? (desc ? "descending" : "ascending") : undefined}>
                <button
                  type="button"
                  className="fx colh"
                  title={t("cols.hint")}
                  onClick={() => sortClick(c)}
                  onContextMenu={(e) => (e.preventDefault(), colMenu(c, e.clientX, e.clientY))}
                  onKeyDown={(e) => {
                    if (e.key !== "ContextMenu" && !(e.shiftKey && e.key === "F10")) return;
                    e.preventDefault();
                    const b = e.currentTarget.getBoundingClientRect();
                    colMenu(c, b.left, b.bottom);
                  }}
                >
                  {t(`cols.${c}`)}
                  {sortBy === c && <Icon n="caret-down" />}
                </button>
              </span>
            ))}
          </div>
          {rows.length === 0 && (
            <div role="row">
              <div role="gridcell" className="empty">
                <p>{q ? t("noMatch") : t("empty")}</p>
                {!q && (
                  <button type="button" className="fx btn btn-p fx-strong" onClick={() => useStore.getState().setAdding(true)}>
                    <Icon n="plus" />
                    {t("add")}
                  </button>
                )}
              </div>
            </div>
          )}
          <div style={{ height: v.getTotalSize(), position: "relative" }}>
            {v.getVirtualItems().map((it) => {
              const r = rows[it.index];
              return (
                <div key={r.id} style={{ position: "absolute", top: 0, left: 0, right: 0, transform: `translateY(${it.start}px)` }}>
                  <RowView r={r} idx={it.index} sel={r.id === sel} focusable={r.id === sel || (selIdx < 0 && it.index === 0)} onWhy={onWhy} onMenu={rowMenu} cols={cols} icon={icon} grid={grid} />
                </div>
              );
            })}
          </div>
        </div>
        <footer className="foot">
          <span className="spd">
            {t("foot.speed")} <b className="num">{speed(total)}</b>
          </span>
          <button
            type="button"
            className="fx pill"
            onClick={() => void useStore.getState().saveSettings({ limit_on: !settings!.limit_on, limit_kbps: settings!.limit_kbps || 512 })}
          >
            <Icon n="gauge" />
            {settings!.limit_on ? t("foot.limitOn", { v: `${settings!.limit_kbps} KB/s` }) : t("foot.limitOff")}
          </button>
          {/* notices sit in this one footer slot: the list never moves when one appears.
              An operational warning (read-only database, restored backup, browser pipe) outranks an update notice. */}
          <span className="sp">
            {banner ? (
              <Banner text={banner.startsWith("restored:") ? t("banner.restored", { name: banner.slice(9) }) : banner === "readonly" ? t("banner.readonly") : t("banner.pipe")} />
            ) : (
              upd && (upd.state === "ready" || upd.state === "available" || upd.state === "failed") && <UpdateBanner />
            )}
          </span>
          <span>{t("foot.items", { n: rows.length })}</span>
        </footer>
      </div>
      {((wide && jobs.length > 0) || drawer) && <DetailsPane r={selected} onWhy={onWhy} drawer={!wide} onClose={() => setDrawer(false)} />}
      {menu && <Menu {...menu} onClose={() => setMenu(null)} />}
    </main>
  );
}

function ToolBtn({ ic, label, words, menu, pressed, onClick }: { ic: string; label: string; words: boolean; menu?: boolean; pressed?: boolean; onClick: (e: RME<HTMLButtonElement>) => void }) {
  return (
    <button
      type="button"
      className={"fx btn btn-s tb" + (words ? "" : " icon")}
      aria-label={words ? undefined : label}
      title={label}
      aria-haspopup={menu ? "menu" : undefined}
      aria-pressed={pressed}
      onClick={onClick}
    >
      <Icon n={ic} />
      {words && <span>{label}</span>}
      {menu && words && <Icon n="caret-down" />}
    </button>
  );
}

function RowView({ r, idx, sel, focusable, onWhy, onMenu, cols, icon, grid }: { r: Row; idx: number; sel: boolean; focusable: boolean; onWhy: (k: string) => void; onMenu: (r: Row, x: number, y: number) => void; cols: Col[]; icon: boolean; grid: string }) {
  const { t, i18n } = useTranslation();
  const l = useStore((s) => s.live[r.id]);
  const st = view(r, l);
  const text = t(`st.${st.key}`, st.vars ?? {});
  const left = l && l.total ? l.total - l.done : 0;
  return (
    <div
      id={"row-" + r.id}
      role="row"
      aria-rowindex={idx + 2}
      aria-selected={sel}
      tabIndex={focusable ? 0 : -1}
      className={"fx row" + (sel ? " is-selected" : "")}
      style={{ gridTemplateColumns: grid }}
      onClick={() => useStore.getState().setSel(r.id)}
      onFocus={() => useStore.getState().setSel(r.id)}
      onContextMenu={(e) => {
        e.preventDefault();
        useStore.getState().setSel(r.id);
        onMenu(r, e.clientX, e.clientY);
      }}
    >
      {icon && (
        <span role="gridcell" className={"ft " + ftClass(r.category)}>
          <Icon n={ftIcon(r.category)} />
        </span>
      )}
      {cols.map((c) =>
        c === "name" ? (
          <span key={c} role="gridcell" className="nm" title={r.name}>
            <b>{r.name}</b>
            <small>
              <span className="sub">{host(r.url)}</span>
              {r.ext && <span className={"ext" + (r.dangerous ? " danger" : "")}>{r.ext}</span>}
            </small>
          </span>
        ) : c === "status" ? (
          <span key={c} role="gridcell" className={"st " + st.tone}>
            <span className="tx" title={text}>
              {text}
            </span>
            {st.bar != null ? (
              <span className={"bar" + (st.barTone ? " " + st.barTone : "")}>
                <i style={{ width: `${st.bar}%` }} />
              </span>
            ) : (
              st.action && (
                <button type="button" className="fx act" tabIndex={-1} onClick={(e) => (e.stopPropagation(), void act(r, st.action, onWhy, st.why))}>
                  {t(`act.${st.action}`)}
                </button>
              )
            )}
          </span>
        ) : (
          <span key={c} role="gridcell" className="num c-end">
            {c === "size"
              ? bytes(l?.total || r.size)
              : c === "speed"
                ? l
                  ? speed(l.speed)
                  : "—"
                : c === "eta"
                  ? l && r.state !== "finalizing"
                    ? eta(left, l.speed)
                    : "—"
                  : new Date(r.created_at * 1000).toLocaleDateString(i18n.language === "km" ? "km-KH" : "en-GB")}
          </span>
        ),
      )}
    </div>
  );
}

function Segments({ segs, total }: { segs: [number, number, number][]; total: number }) {
  if (!total || !segs.length) return null;
  // 60 cells: a cell is "on" when an active segment's write head is inside it, "got" when written
  const cells = Array.from({ length: 60 }, (_, i) => {
    const a = (i / 60) * total;
    const b = ((i + 1) / 60) * total;
    const s = segs.find((x) => x[0] < b && x[1] > a);
    if (!s) return "";
    if (s[2] >= b) return "got";
    if (s[2] > a && s[2] < s[1]) return "on";
    return "";
  });
  return (
    <div className="seg" aria-hidden="true">
      {cells.map((c, i) => (
        <i key={i} className={c} />
      ))}
    </div>
  );
}

function DetailsPane({ r, onWhy, drawer, onClose }: { r: Row | null; onWhy: (k: string) => void; drawer: boolean; onClose: () => void }) {
  const { t } = useTranslation();
  const settings = useStore((s) => s.settings);
  const l = useStore((s) => (r ? s.live[r.id] : undefined));
  const [d, setD] = useState<Detail | null>(null);
  const [desc, setDesc] = useState("");
  useEffect(() => {
    setD(null);
    if (r) void api.detail(r.id).then(setD).catch(() => setD(null));
    setDesc(r?.description ?? "");
  }, [r?.id, r?.state, r?.description, r]);
  if (!r)
    return (
      <aside className={"pane none" + (drawer ? " drawer" : "")} aria-label={t("d.title")}>
        <Icon n="info" lg />
        <p className="meta">{t("d.none")}</p>
      </aside>
    );
  const st = view(r, l);
  const total = l?.total || r.size || 0;
  const done = l?.done ?? r.done;
  const segs = l?.segs ?? d?.segments ?? [];
  return (
    <aside className={"pane" + (drawer ? " drawer" : "")} aria-label={t("d.title")}>
      <div className="card">
        {drawer && (
          <button type="button" className="fx mini close-d" aria-label={t("d.close")} onClick={onClose}>
            <Icon n="x" />
          </button>
        )}
        <h2>{r.name}</h2>
        <div className="sub">{host(r.url)}</div>
        <div className={"st big-st " + st.tone}>
          <span className="tx2">{t(`st.${st.key}`, st.vars ?? {})}</span>
        </div>
        <div className="big">
          <b>{bytes(done)}</b>
          <span className="meta">/ {bytes(total || null)}</span>
        </div>
        <Segments segs={segs} total={total} />
        <dl className="kv">
          <dt>{t("d.speed")}</dt>
          <dd className="num">{l ? speed(l.speed) : "—"}</dd>
          <dt>{t("d.left")}</dt>
          <dd className="num">{l && total ? eta(total - done, l.speed) : "—"}</dd>
          <dt>{t("d.conns")}</dt>
          <dd>
            {l ? (
              <span className="step">
                <button type="button" className="fx" aria-label={t("d.connsLess")} title={t("d.connsLess")} disabled={l.conns <= 1} onClick={() => void api.conns(r.id, Math.max(1, l.conns - 1))}>
                  <Icon n="minus" />
                </button>
                <span className="num">{l.conns}</span>
                <button type="button" className="fx" aria-label={t("d.connsMore")} title={t("d.connsMore")} disabled={l.conns >= 32} onClick={() => void api.conns(r.id, Math.min(32, l.conns + 1))}>
                  <Icon n="plus" />
                </button>
              </span>
            ) : (
              "—"
            )}
          </dd>
          {r.queue && (
            <>
              <dt>{t("d.queue")}</dt>
              <dd className="clip1">{qName(t, settings?.queues.find((x) => x.id === r.queue))}</dd>
            </>
          )}
          <dt>{t("d.folder")}</dt>
          <dd className="clip1" title={r.dir}>
            {r.dir}
          </dd>
          <dt>{t("d.sha")}</dt>
          <dd className="mono clip1" title={r.sha256 ?? ""}>
            {r.sha256 ? r.sha256.slice(0, 16) + "…" : t("d.shaWait")}
          </dd>
        </dl>
        <span className="lab">{t("d.url")}</span>
        <div className="url">{readable(d?.final_url ?? r.url)}</div>
        <label className="fld">
          <span>{t("d.desc")}</span>
          <span className="in">
            <input value={desc} maxLength={2000} onChange={(e) => setDesc(e.target.value)} onBlur={() => desc !== (r.description ?? "") && void api.desc(r.id, desc)} />
          </span>
        </label>
        {isActive(r) && (
          <label className="fld">
            <span>{t("d.limit")}</span>
            <span className="in">
              <input type="number" min={0} defaultValue={0} onChange={(e) => void api.limit(r.id, Math.max(0, Number(e.target.value) || 0))} />
            </span>
          </label>
        )}
        <div className="row-btns">
          {st.action && (
            <button type="button" className="fx btn btn-p fx-strong" onClick={() => void act(r, st.action, onWhy, st.why)}>
              {t(`act.${st.action}`)}
            </button>
          )}
          {st.second && st.second !== st.action && (
            <button type="button" className="fx btn btn-s" onClick={() => void act(r, st.second, onWhy, st.why)}>
              {t(`act.${st.second}`)}
            </button>
          )}
          {!isActive(r) && (
            <button type="button" className="fx btn btn-g" onClick={() => void api.remove(r.id, false)}>
              <Icon n="trash" />
              {t("act.remove")}
            </button>
          )}
          {isDone(r) && (
            <button type="button" className="fx btn btn-g danger" onClick={() => void api.remove(r.id, true)}>
              {t("act.removeFile")}
            </button>
          )}
        </div>
      </div>
    </aside>
  );
}

function Banner({ text }: { text: string }) {
  return (
    <div className="banner" role="status">
      <Icon n="info" />
      <span>{text}</span>
    </div>
  );
}

function UpdateBanner() {
  const { t, i18n } = useTranslation();
  const u = useStore((s) => s.tick?.update);
  if (!u) return null;
  const notes = i18n.language === "en" ? u.notes_en : u.notes_km;
  if (u.state === "failed")
    return (
      <div className="banner warn" role="status">
        <Icon n="warning" />
        <span>{t("banner.updFailed")}</span>
        <button type="button" className="fx act" onClick={() => void api.updateCheck()}>
          {t("banner.updRetry")}
        </button>
        <button type="button" className="fx act" onClick={() => void api.openLink("releases")}>
          {t("banner.updGithub")}
        </button>
      </div>
    );
  return (
    <div className={"banner" + (u.security ? " sec" : "")} role="status" title={notes ?? ""}>
      <Icon n={u.security ? "shield-check" : "arrow-clockwise"} />
      <span>
        {u.state === "ready" ? t("banner.updReady", { v: u.version }) : t("banner.updAvail", { v: u.version })}
        {u.security ? " · " + t("banner.updSecurity") : ""}
      </span>
      {u.state === "ready" ? (
        <button type="button" className="fx act" onClick={() => void api.updateRestart()}>
          {t("banner.updNow")}
        </button>
      ) : (
        <button type="button" className="fx act" onClick={() => void api.updateCheck()}>
          {t("banner.updGet")}
        </button>
      )}
    </div>
  );
}

function InfoDialog() {
  const { t, i18n } = useTranslation();
  const pending = useStore((s) => s.tick?.pending ?? NONE);
  const live = useStore((s) => s.live);
  const settings = useStore((s) => s.settings)!;
  const closed = useStore((s) => s.infoClosed);
  const items = pending.filter((p) => !closed.includes(p.id));
  const [checked, setChecked] = useState<Record<string, boolean>>({});
  const [name, setName] = useState("");
  const [cat, setCat] = useState("");
  const [armed, setArmed] = useState(false);
  const [queue, setQueue] = useState("main");
  const first = items[0];
  const key = items.map((i) => i.id).join(",");
  useEffect(() => {
    // C-03: the Start button takes Enter only after 400 ms (typing in the browser must not confirm it)
    setArmed(false);
    const tm = setTimeout(() => setArmed(true), 400);
    if (first) {
      setName(first.name);
      setCat(first.category);
    }
    return () => clearTimeout(tm);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key]);
  if (!items.length) return null;
  const hide = (ids: string[]) => useStore.setState({ infoClosed: [...useStore.getState().infoClosed, ...ids] });
  const many = items.length > 1;
  const isOn = (id: string) => checked[id] ?? true;
  const confirm = async (later: boolean) => {
    hide(items.map((i) => i.id));
    const ids: string[] = [];
    if (!many) ids.push(await api.infoConfirm(first.id, name, cat, later));
    else
      for (const it of items) {
        if (isOn(it.id)) ids.push(await api.infoConfirm(it.id, null, null, later));
        else await api.infoCancel(it.id);
      }
    // IDM "Download later" with the queue selection panel: the files wait in the chosen queue
    if (later && ids.length) {
      await api.queuePut(ids, queue);
      await useStore.getState().refreshQueues();
    }
  };
  const cancel = async () => {
    hide(items.map((i) => i.id));
    for (const it of items) await api.infoCancel(it.id);
  };
  const started = items.some((i) => live[i.id]);
  // the footer size is what will be downloaded: the checked files; unknown sizes are said, never counted as 0
  const sizeOf = (it: Row) => live[it.id]?.total || it.size || 0;
  const picked = many ? items.filter((i) => isOn(i.id)) : [first];
  const known = picked.reduce((a, i) => a + sizeOf(i), 0);
  const unknown = picked.filter((i) => !sizeOf(i)).length;
  const c = settings.categories.find((x) => x.id === cat);
  const saveTo = c?.dir || settings.download_dir + (c?.sub ? "\\" + c.sub : "");
  return (
    <Modal label={many ? t("info.batch", { n: items.length }) : t("info.title")} onClose={() => void cancel()} wide={many}>
      <h2>{many ? t("info.batch", { n: items.length }) : t("info.title")}</h2>
      <div className="note">
        <Icon n="lightning" />
        <span>{started ? t("info.note") : t("info.notStarted")}</span>
      </div>
      {many ? (
        <div className="batch" role="group" aria-label={t("info.batch", { n: items.length })}>
          {items.map((it) => (
            <button key={it.id} type="button" className="fx check" role="checkbox" aria-checked={isOn(it.id)} onClick={() => setChecked({ ...checked, [it.id]: !isOn(it.id) })}>
              <i>{isOn(it.id) && <Icon n="check" />}</i>
              <span className="clip1" title={it.name}>
                {it.name}
              </span>
              <span className="num">{bytes(live[it.id]?.total || it.size)}</span>
            </button>
          ))}
        </div>
      ) : (
        <>
          <label className="fld">
            <span>{t("info.name")}</span>
            <span className="in">
              <input value={name} onChange={(e) => setName(e.target.value)} />
            </span>
          </label>
          <div className="two">
            <label className="fld">
              <span>{t("info.cat")}</span>
              <select className="fx sel" value={cat} onChange={(e) => setCat(e.target.value)}>
                {settings.categories.map((x) => (
                  <option key={x.id} value={x.id}>
                    {catName(i18n.language, x)}
                  </option>
                ))}
              </select>
            </label>
            <div className="fld">
              <span>{t("info.save")}</span>
              <div className="url clip1" title={saveTo}>
                {saveTo}
              </div>
            </div>
          </div>
          {c && (
            <div className="row-btns tight">
              <button type="button" className="fx btn btn-s" onClick={() => void rememberDir(c.id, false)}>
                <Icon n="folder-open" />
                {t("info.remember", { c: catName(i18n.language, c) })}
              </button>
              {c.dir && (
                <button type="button" className="fx btn btn-g" onClick={() => void rememberDir(c.id, true)}>
                  {t("info.forget")}
                </button>
              )}
            </div>
          )}
          <span className="lab">{t("info.url")}</span>
          <div className="url">{readable(first.url)}</div>
        </>
      )}
      <QueueSelect value={queue} onChange={setQueue} label={t("info.queue")} />
      <div className="mfoot">
        <span className="meta num">
          {many && t("info.picked", { n: picked.length }) + " · "}
          {known || !unknown ? bytes(known) : ""}
          {unknown > 0 && (known ? " + " : "") + t("info.unknownN", { n: unknown })}
        </span>
        <span className="sp" />
        <button type="button" className="fx btn btn-g" onClick={() => void cancel()}>
          {t("info.cancel")}
        </button>
        <button type="button" className="fx btn btn-s" onClick={() => void confirm(true)}>
          {t("info.later")}
        </button>
        <button type="button" className="fx btn btn-p fx-strong" aria-disabled={!armed} onClick={() => armed && void confirm(false)}>
          {t("info.start")}
        </button>
      </div>
    </Modal>
  );
}

function AddDialog() {
  const { t } = useTranslation();
  const [url, setUrl] = useState("");
  const [bad, setBad] = useState(false);
  const close = () => useStore.getState().setAdding(false);
  const go = async () => {
    if (!/^https?:\/\/\S+$/i.test(url.trim())) return setBad(true);
    try {
      await addLink(url);
      close();
    } catch {
      setBad(true);
    }
  };
  return (
    <Modal label={t("addDlg.title")} onClose={close}>
      <h2>{t("addDlg.title")}</h2>
      <label className="fld">
        <span>{t("addDlg.url")}</span>
        <span className="in">
          <input className="mono" value={url} aria-invalid={bad} onChange={(e) => (setUrl(e.target.value), setBad(false))} onKeyDown={(e) => e.key === "Enter" && void go()} />
        </span>
      </label>
      <ErrLine msg={bad && t("addDlg.bad")} />
      <div className="mfoot">
        <span className="sp" />
        <button type="button" className="fx btn btn-g" onClick={close}>
          {t("info.cancel")}
        </button>
        <button type="button" className="fx btn btn-p fx-strong" onClick={() => void go()}>
          {t("addDlg.go")}
        </button>
      </div>
    </Modal>
  );
}

function WhyDialog({ k, onClose }: { k: string; onClose: () => void }) {
  const { t } = useTranslation();
  return (
    <Modal label={t("act.why")} onClose={onClose}>
      <h2>{t("act.why")}</h2>
      <p className="why">{t(`why.${k}`)}</p>
      <div className="mfoot">
        <span className="sp" />
        {k === "blocked" && (
          <button type="button" className="fx btn btn-s" onClick={() => void api.openLink("windows-security")}>
            Windows Security
          </button>
        )}
        <button type="button" className="fx btn btn-p fx-strong" onClick={onClose}>
          {t("d.close")}
        </button>
      </div>
    </Modal>
  );
}

function Toasts() {
  const { t } = useTranslation();
  const toasts = useStore((s) => s.toasts);
  const jobs = useStore((s) => s.jobs);
  const queues = useStore((s) => s.settings!.queues);
  // queue-done names the queue (the built-in ones come as ""); refresh names the file
  const sub = (x: { kind: string; id: string; name: string }) =>
    x.kind === "queue-done" ? qName(t, queues.find((q) => q.id === x.id)) : x.kind === "refresh" ? (jobs.find((j) => j.id === x.id)?.name ?? "") : x.name;
  useEffect(() => {
    if (!toasts.length) return;
    const tm = setTimeout(() => useStore.getState().dismiss(toasts[0].n), 8000);
    return () => clearTimeout(tm);
  }, [toasts]);
  return (
    <div className="toasts" role="status" aria-live="polite">
      {toasts.map((x) => (
        <div key={x.n} className={"toast " + x.kind}>
          <Icon n={x.kind === "blocked" || x.kind === "failed" || x.kind === "error" ? "warning-circle" : x.kind === "info" ? "info" : x.kind === "video-link" ? "file-video" : "check-circle"} fill={x.kind !== "info"} />
          <div className="tt">
            <b>{x.kind === "info" ? t(`toast.infoMsg.${x.name}`, { defaultValue: x.name }) : t(`toast.${x.kind}`, { defaultValue: x.kind })}</b>
            {x.kind !== "info" && (
              <span className="clip1" title={sub(x)}>
                {sub(x)}
              </span>
            )}
          </div>
          {x.kind === "video-link" && (
            <>
              <button type="button" className="fx act" onClick={() => (useStore.getState().setDlg({ k: "video", url: x.name }), useStore.getState().dismiss(x.n))}>
                {t("toast.chooseQuality")}
              </button>
              <button type="button" className="fx act" onClick={() => useStore.getState().dismiss(x.n)}>
                {t("toast.dismiss")}
              </button>
            </>
          )}
          {x.kind === "refresh" && (
            <button type="button" className="fx act" onClick={() => (useStore.getState().setDlg({ k: "refresh", id: x.id }), useStore.getState().dismiss(x.n))}>
              {t("toast.choose")}
            </button>
          )}
          {x.kind === "complete" && !x.dangerous && (
            <button type="button" className="fx act" onClick={() => void api.open(x.id)}>
              {t("act.open")}
            </button>
          )}
          {(x.kind === "complete" || x.kind === "unprotected") && (
            <button type="button" className="fx act" onClick={() => void api.folder(x.id)}>
              {t("act.openFolder")}
            </button>
          )}
          <button type="button" className="fx mini" aria-label={t("toast.close")} onClick={() => useStore.getState().dismiss(x.n)}>
            <Icon n="x" />
          </button>
        </div>
      ))}
    </div>
  );
}
