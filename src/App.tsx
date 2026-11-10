// SPDX-License-Identifier: RPL-1.5
// Main window (§06): rail + list (TanStack Virtual) + details pane (wide) / drawer, Download Info, toasts.
// All text is rendered as React text nodes (X-01); actions call commands with job ids only (X-02).
import { useEffect, useMemo, useRef, useState, type DragEvent, type KeyboardEvent as RKE } from "react";
import { useTranslation } from "react-i18next";
import { useVirtualizer } from "@tanstack/react-virtual";
import { api, type Detail, type Row } from "./api";
import { useStore, type Nav } from "./store";
import { bytes, eta, host, isActive, isDone, speed, view, type Action } from "./status";
import { ftClass, ftIcon, Icon, Modal } from "./ui";
import { Onboarding, SettingsView } from "./Settings";

const NONE: Row[] = []; // a stable empty list: a new [] in a selector re-renders forever (React #185)

const catName = (lang: string, c: { name_km: string; name_en: string }) => (lang === "en" ? c.name_en : c.name_km);

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
  if (!settings.onboarded) return <Onboarding />;
  return <Shell />;
}

function Shell() {
  const { t, i18n } = useTranslation();
  const { nav, setNav, settings, jobs } = useStore();
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
      if (e.ctrlKey && e.key.toLowerCase() === "n") {
        e.preventDefault();
        useStore.getState().setAdding(true);
      }
    };
    window.addEventListener("keydown", k);
    return () => window.removeEventListener("keydown", k);
  }, []);
  return (
    <>
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
            <div className="sp" />
            <NavBtn k="settings" ic="gear-six" label={t("nav.settings")} sel={nav === "settings"} onClick={() => setNav("settings")} />
          </nav>
          {nav === "settings" ? <SettingsView /> : <Main onWhy={setWhy} />}
        </div>
      </div>
      <InfoDialog />
      {adding && <AddDialog />}
      {why && <WhyDialog k={why} onClose={() => setWhy(null)} />}
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

function filterRows(jobs: Row[], nav: Nav) {
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
    case "newName": {
      const id = await api.addUrl(r.url);
      return id;
    }
  }
}

function Main({ onWhy }: { onWhy: (k: string) => void }) {
  const { t, i18n } = useTranslation();
  const { jobs, nav, q, setQ, sel, setSel, live, tick, settings } = useStore();
  const rows = useMemo(() => filterRows(jobs, nav), [jobs, nav]);
  const w = useWide();
  const [drawer, setDrawer] = useState(false);
  const wide = w >= 1180;
  const scroller = useRef<HTMLDivElement>(null);
  const v = useVirtualizer({ count: rows.length, getScrollElement: () => scroller.current, estimateSize: () => 62, overscan: 8 });
  const selIdx = rows.findIndex((r) => r.id === sel);
  const selected = selIdx >= 0 ? rows[selIdx] : null;
  const total = Object.values(live).reduce((a, l) => a + l.speed, 0);
  const title = nav.startsWith("cat:") ? catName(i18n.language, settings!.categories.find((c) => c.id === nav.slice(4)) ?? { name_km: "", name_en: "" }) : t(`title.${nav}`);
  const searchRef = useRef<HTMLInputElement>(null);

  const onKey = (e: RKE) => {
    if (!rows.length) return;
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
    else return;
    e.preventDefault();
  };
  useEffect(() => {
    const k = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key.toLowerCase() === "f") {
        e.preventDefault();
        searchRef.current?.focus();
      }
    };
    const paste = (e: ClipboardEvent) => {
      const el = document.activeElement;
      if (el && (el.tagName === "INPUT" || el.tagName === "TEXTAREA")) return;
      const s = e.clipboardData?.getData("text/plain")?.trim() ?? "";
      if (/^https?:\/\/\S+$/i.test(s)) void api.addUrl(s);
    };
    window.addEventListener("keydown", k);
    window.addEventListener("paste", paste);
    return () => {
      window.removeEventListener("keydown", k);
      window.removeEventListener("paste", paste);
    };
  }, []);
  const onDrop = (e: DragEvent) => {
    e.preventDefault();
    const s = (e.dataTransfer.getData("text/uri-list") || e.dataTransfer.getData("text/plain")).split(/\r?\n/).find((l) => /^https?:\/\//i.test(l.trim()));
    if (s) void api.addUrl(s.trim());
  };
  const upd = tick?.update;
  const banner = tick?.banner;
  return (
    <main className={"main" + (wide ? " split" : "")} onDragOver={(e) => e.preventDefault()} onDrop={onDrop}>
      <div className="col">
        <div className="top">
          <h1>{title}</h1>
          <label className="search">
            <Icon n="magnifying-glass" />
            <input ref={searchRef} type="search" value={q} placeholder={t("search")} aria-label={t("search")} onChange={(e) => setQ(e.target.value)} />
          </label>
          <button type="button" className="fx btn btn-p fx-strong" onClick={() => useStore.getState().setAdding(true)}>
            <Icon n="plus" />
            {t("add")}
          </button>
          <button type="button" className="fx chip" aria-label={t("pauseAll")} title={t("pauseAll")} onClick={() => void api.pauseAll()}>
            <Icon n="pause" />
          </button>
          {!wide && (
            <button type="button" className="fx chip" aria-label={t("d.title")} title={t("d.title")} aria-pressed={drawer} onClick={() => setDrawer(!drawer)}>
              <Icon n="info" />
            </button>
          )}
        </div>
        {banner && <Banner text={banner.startsWith("restored:") ? t("banner.restored", { name: banner.slice(9) }) : banner === "readonly" ? t("banner.readonly") : t("banner.pipe")} />}
        {upd && (upd.state === "ready" || upd.state === "available" || upd.state === "failed") && <UpdateBanner />}
        <div className="list" ref={scroller} role="grid" aria-label={title} aria-rowcount={rows.length + 1} onKeyDown={onKey}>
          <div className="cols" role="row" aria-rowindex={1}>
            <span role="columnheader" aria-label={t("cols.name")} />
            <span role="columnheader">{t("cols.name")}</span>
            <span role="columnheader">{t("cols.size")}</span>
            <span role="columnheader">{t("cols.status")}</span>
            <span role="columnheader">{t("cols.speed")}</span>
            <span role="columnheader">{t("cols.eta")}</span>
          </div>
          {rows.length === 0 && <div className="empty">{q ? t("noMatch") : t("empty")}</div>}
          <div style={{ height: v.getTotalSize(), position: "relative" }}>
            {v.getVirtualItems().map((it) => {
              const r = rows[it.index];
              return (
                <div key={r.id} style={{ position: "absolute", top: 0, left: 0, right: 0, transform: `translateY(${it.start}px)` }}>
                  <RowView r={r} idx={it.index} sel={r.id === sel} focusable={r.id === sel || (selIdx < 0 && it.index === 0)} onWhy={onWhy} />
                </div>
              );
            })}
          </div>
        </div>
        <footer className="foot">
          <span>
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
          <span className="sp" />
          <span>{t("foot.items", { n: rows.length })}</span>
        </footer>
      </div>
      {(wide || drawer) && <DetailsPane r={selected} onWhy={onWhy} drawer={!wide} onClose={() => setDrawer(false)} />}
    </main>
  );
}

function RowView({ r, idx, sel, focusable, onWhy }: { r: Row; idx: number; sel: boolean; focusable: boolean; onWhy: (k: string) => void }) {
  const { t } = useTranslation();
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
      onClick={() => useStore.getState().setSel(r.id)}
      onFocus={() => useStore.getState().setSel(r.id)}
    >
      <span role="gridcell" className={"ft " + ftClass(r.category)}>
        <Icon n={ftIcon(r.category)} />
      </span>
      <span role="gridcell" className="nm" title={r.name}>
        <b>{r.name}</b>
        <small>
          <span className="sub">{host(r.url)}</span>
          {r.ext && <span className={"ext" + (r.dangerous ? " danger" : "")}>{r.ext}</span>}
        </small>
      </span>
      <span role="gridcell" className="num">
        {bytes(l?.total || r.size)}
      </span>
      <span role="gridcell" className={"st " + st.tone}>
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
      <span role="gridcell" className="num">
        {l ? speed(l.speed) : "—"}
      </span>
      <span role="gridcell" className="num">
        {l && r.state !== "finalizing" ? eta(left, l.speed) : "—"}
      </span>
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
      <aside className={"pane" + (drawer ? " drawer" : "")} aria-label={t("d.title")}>
        <div className="card">
          <p className="meta">{t("d.none")}</p>
        </div>
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
                <button type="button" className="fx" aria-label="−" onClick={() => void api.conns(r.id, Math.max(1, l.conns - 1))}>
                  <Icon n="minus" />
                </button>
                <span className="num">{l.conns}</span>
                <button type="button" className="fx" aria-label="+" onClick={() => void api.conns(r.id, Math.min(32, l.conns + 1))}>
                  <Icon n="plus" />
                </button>
              </span>
            ) : (
              "—"
            )}
          </dd>
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
        <div className="url">{d?.final_url ?? r.url}</div>
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
    if (!many) return void (await api.infoConfirm(first.id, name, cat, later));
    for (const it of items) {
      if (isOn(it.id)) await api.infoConfirm(it.id, null, null, later);
      else await api.infoCancel(it.id);
    }
  };
  const cancel = async () => {
    hide(items.map((i) => i.id));
    for (const it of items) await api.infoCancel(it.id);
  };
  const started = items.some((i) => live[i.id]);
  const c = settings.categories.find((x) => x.id === cat);
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
              <div className="url clip1" title={settings.download_dir + (c?.sub ? "\\" + c.sub : "")}>
                {settings.download_dir + (c?.sub ? "\\" + c.sub : "")}
              </div>
            </div>
          </div>
          <span className="lab">{t("info.url")}</span>
          <div className="url">{first.url}</div>
        </>
      )}
      <div className="mfoot">
        <span className="meta num">{bytes(live[first.id]?.total || first.size)}</span>
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
      await api.addUrl(url.trim());
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
      {bad && (
        <p className="err-t" role="alert">
          {t("addDlg.bad")}
        </p>
      )}
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
  useEffect(() => {
    if (!toasts.length) return;
    const tm = setTimeout(() => useStore.getState().dismiss(toasts[0].n), 8000);
    return () => clearTimeout(tm);
  }, [toasts]);
  return (
    <div className="toasts" role="status" aria-live="polite">
      {toasts.map((x) => (
        <div key={x.n} className={"toast " + x.kind}>
          <Icon n={x.kind === "blocked" || x.kind === "failed" ? "warning-circle" : "check-circle"} fill />
          <div className="tt">
            <b>{t(`toast.${x.kind}`)}</b>
            <span className="clip1" title={x.name}>
              {x.name}
            </span>
          </div>
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
