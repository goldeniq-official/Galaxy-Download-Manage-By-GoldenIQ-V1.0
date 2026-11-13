// SPDX-License-Identifier: RPL-1.5
// Settings (§06 screen 4) and first-run onboarding (J2: 3 steps, skippable; D5 video button off, asked here).
import { useEffect, useRef, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { api, errText, fileName, type Browser, type FolderCheck, type HostLimit, type Settings, type SiteLogin, type SoundEvent } from "./api";
import { useStore } from "./store";
import { bytes } from "./status";
import { ErrLine, Icon, Modal, Tabs, Toggle } from "./ui";
import { Components, rememberDir } from "./Dialogs";
import { QueuesSection } from "./Queues";

import { version as VERSION } from "../package.json";
// §21 M1: the owner's KHQR, bundled at build time (no network). Absent file = text-only "not in this build yet".
const KHQR = Object.values(import.meta.glob<string>("./assets/khqr.png", { eager: true, query: "?url", import: "default" }))[0];

export function Opt({ title, sub, children }: { title: string; sub?: string; children: ReactNode }) {
  return (
    <div className="opt">
      <div className="t">
        <b>{title}</b>
        {sub && <small>{sub}</small>}
      </div>
      {children}
    </div>
  );
}

/** one group of rows; the section's name is the dialog heading, so a group shows a small heading only when it names a sub-group */
export function Sec({ id, title, sub, children }: { id: string; title: string; sub?: boolean; children: ReactNode }) {
  return (
    <section className="sec" id={"set-" + id} aria-label={title}>
      {sub && <h3>{title}</h3>}
      <div className="card list-card">{children}</div>
    </section>
  );
}

function useBrowsers() {
  const [b, setB] = useState<Browser[]>([]);
  useEffect(() => {
    const f = () => void api.browsers().then(setB);
    f();
    const tm = setInterval(f, 2000);
    return () => clearInterval(tm);
  }, []);
  return b;
}

const storeKey = (k: string) => (k === "firefox" ? "store-firefox" : k === "edge" ? "store-edge" : "store-chrome");

function BrowserRows({ list }: { list: Browser[] }) {
  const { t } = useTranslation();
  return (
    <>
      {list
        .filter((b) => b.installed || b.connected)
        .map((b) => (
          <div className="opt" key={b.key}>
            <Icon n="browser" lg />
            <span className="t">
              <b>{b.name}</b>
            </span>
            {b.connected ? (
              <span className="ok">
                <Icon n="check-circle" fill />
                {t("set.connected")}
              </span>
            ) : (
              <button type="button" className="fx btn btn-s" onClick={() => void api.openLink(storeKey(b.key))}>
                <Icon n="puzzle-piece" />
                {t("set.installExt")}
              </button>
            )}
          </div>
        ))}
    </>
  );
}

const KEYS: [string, string][] = [
  ["ctrl", "Ctrl"],
  ["alt", "Alt"],
  ["shift", "Shift"],
];

export function SettingsView() {
  const { t, i18n } = useTranslation();
  const s = useStore((x) => x.settings)!;
  const save = useStore((x) => x.saveSettings);
  const tick = useStore((x) => x.tick);
  const list = useBrowsers();
  const sec = useStore((x) => x.sec);
  const [copied, setCopied] = useState(false);
  const lang = i18n.language;
  const secs: [string, string, string][] = [
    ["general", "gear-six", t("set.general")],
    ["downloads", "download-simple", t("set.downloads")],
    ["connection", "plugs-connected", t("set.connection")],
    ["queues", "queue", t("set.queues")],
    ["proxy", "globe", t("set.proxy")],
    ["logins", "hash", t("set.logins")],
    ["sounds", "play", t("set.sounds")],
    ["video", "file-video", t("set.videoSec")],
    ["browsers", "browser", t("set.browsers")],
    ["update", "arrow-clockwise", t("set.update")],
    ["privacy", "shield-check", t("set.privacy")],
    ["about", "info", t("set.about")],
  ];
  // one dialog over the app (IDM Options, Mazer Settings): sections left, one section's rows right
  const go = (k: string) => useStore.setState({ sec: k });
  const close = () => useStore.setState({ settingsOpen: false });
  // opening lands on the section shown (Tasks > Scheduler opens Queues), not always on the first item
  useEffect(() => document.querySelector<HTMLElement>(".setnav .is-selected")?.focus(), []);
  const cur = secs.find((x) => x[0] === sec) ?? secs[0];
  return (
    <Modal label={t("nav.settings")} onClose={close} cls="setdlg">
      <nav className="setnav" aria-label={t("nav.settings")}>
        <h2>{t("nav.settings")}</h2>
        {secs.map(([k, ic, label]) => (
          <button key={k} type="button" className={"fx nav" + (cur[0] === k ? " is-selected" : "")} aria-current={cur[0] === k ? "true" : undefined} onClick={() => go(k)}>
            <Icon n={ic} lg fill={cur[0] === k} />
            <span className="lbl">{label}</span>
          </button>
        ))}
      </nav>
      <div className="sethead">
        <h2>{cur[2]}</h2>
        <button type="button" className="fx chip" aria-label={t("d.close")} data-tip={t("d.close")} onClick={close}>
          <Icon n="x" />
        </button>
      </div>
      <div className="sbody">
          {cur[0] === "general" && <Sec id="general" title={t("set.look")}>
            <Opt title={t("set.lang")} sub={t("set.langS")}>
              <Tabs value={s.lang} label={t("set.lang")} items={[["km", "ខ្មែរ"], ["en", "English"]]} onChange={(v) => void save({ lang: v })} />
            </Opt>
            <Opt title={t("set.theme")} sub={t("set.themeS")}>
              <Tabs value={s.theme} label={t("set.theme")} items={[["system", t("set.tSys")], ["light", t("set.tLight")], ["dark", t("set.tDark")]]} onChange={(v) => void save({ theme: v })} />
            </Opt>
            <Opt title={t("set.startup")} sub={t("set.startupS")}>
              <Toggle on={s.autostart} label={t("set.startup")} onChange={(v) => void save({ autostart: v })} />
            </Opt>
            <Opt title={t("set.silent")} sub={t("set.silentS")}>
              <Toggle on={s.silent} label={t("set.silent")} onChange={(v) => void save({ silent: v })} />
            </Opt>
          </Sec>}
          {cur[0] === "downloads" && <Sec id="downloads" title={t("set.downloads")}>
            <Opt title={t("set.folder")} sub={s.download_dir}>
              <button
                type="button"
                className="fx btn btn-s"
                onClick={async () => {
                  const p = await api.pickDownloadDir();
                  if (p) useStore.setState({ settings: { ...s, download_dir: p } });
                }}
              >
                <Icon n="folder-open" />
                {t("set.change")}
              </button>
            </Opt>
            <Opt title={t("set.conns")} sub={t("set.connsS")}>
              <span className="in short num-in">
                <input type="number" min={1} max={32} aria-label={t("set.conns")} defaultValue={s.max_conns} onBlur={(e) => void save({ max_conns: num(e.target.value, 1, 32) })} />
              </span>
            </Opt>
            <Opt title={t("set.info")} sub={t("set.infoS")}>
              <Toggle on={s.show_info} label={t("set.info")} onChange={(v) => void save({ show_info: v })} />
            </Opt>
            <Opt title={t("set.sidecar")} sub={t("set.sidecarS")}>
              <Toggle on={s.hash_sidecar} label={t("set.sidecar")} onChange={(v) => void save({ hash_sidecar: v })} />
            </Opt>
            <ScanRow />
          </Sec>}
          {cur[0] === "downloads" && <Sec id="cats" title={t("set.cats")} sub>
            {/* one grid: the column names sit once on top, every category is one aligned row */}
            <div className="cat-grid" role="table" aria-label={t("set.cats")}>
              <div className="cat-h" role="row">
                <span role="columnheader">{t("info.cat")}</span>
                <span role="columnheader">{t("set.exts")}</span>
                <span role="columnheader">{t("set.sites")}</span>
                <span role="columnheader">{t("d.folder")}</span>
              </div>
              {s.categories.map((c, i) => {
                const nm = lang === "en" ? c.name_en : c.name_km;
                const put = (k: "exts" | "sites", v: string) => void save({ categories: s.categories.map((x, j) => (j === i ? { ...x, [k]: v.split(/\s+/).filter(Boolean) } : x)) });
                return (
                  <div className="cat-r" role="row" key={c.id}>
                    <b role="cell" className="clip1">{nm}</b>
                    <span role="cell" className="in">
                      <input aria-label={nm + " · " + t("set.exts")} defaultValue={c.exts.join(" ")} onBlur={(e) => put("exts", e.target.value)} />
                    </span>
                    <span role="cell" className="in">
                      <input aria-label={nm + " · " + t("set.sites")} defaultValue={(c.sites ?? []).join(" ")} onBlur={(e) => put("sites", e.target.value)} />
                    </span>
                    <span role="cell" className="cat-dir">
                      <button type="button" className="fx btn btn-s" aria-label={(c.dir ? fileName(c.dir) : t("set.catDir")) + " · " + nm} data-tip={c.dir || t("set.catDir")} onClick={() => void rememberDir(c.id, false)}>
                        <Icon n="folder-open" />
                        <span className="clip1">{c.dir ? fileName(c.dir) : t("set.catDir")}</span>
                      </button>
                      {c.dir && (
                        <button type="button" className="fx mini" aria-label={t("info.forget") + " · " + nm} data-tip={t("info.forget")} onClick={() => void rememberDir(c.id, true)}>
                          <Icon n="x" />
                        </button>
                      )}
                    </span>
                  </div>
                );
              })}
            </div>
          </Sec>}
          {cur[0] === "connection" && <ConnectionSection />}
          {cur[0] === "queues" && <QueuesSection />}
          {cur[0] === "proxy" && <ProxySection />}
          {cur[0] === "logins" && <LoginsSection />}
          {cur[0] === "sounds" && <SoundsSection />}
          {cur[0] === "video" && <VideoSection />}
          {cur[0] === "browsers" && <Sec id="browsers" title={t("set.browsers")}>
            <Opt title={t("set.catch")} sub={t("set.catchS")}>
              <Toggle on={s.catch_enabled} label={t("set.catch")} onChange={(v) => void save({ catch_enabled: v })} />
            </Opt>
            <BrowserRows list={list} />
            <Opt title={t("set.force")}>
              <Tabs value={s.force_key} label={t("set.force")} items={KEYS} onChange={(v) => void save({ force_key: v })} />
            </Opt>
            <Opt title={t("set.bypass")}>
              <Tabs value={s.bypass_key} label={t("set.bypass")} items={KEYS} onChange={(v) => void save({ bypass_key: v })} />
            </Opt>
            <Opt title={t("set.minSize")}>
              <span className="in short num-in">
                <input type="number" min={0} aria-label={t("set.minSize")} defaultValue={s.catch_min_kb} onBlur={(e) => void save({ catch_min_kb: num(e.target.value, 0, 10_000_000) })} />
              </span>
            </Opt>
            <div className="opt col-opt">
              <label className="fld wide-fld">
                <span>{t("set.catchExts")}</span>
                <span className="in">
                  <input defaultValue={s.catch_exts.join(" ")} onBlur={(e) => void save({ catch_exts: e.target.value.split(/\s+/).filter(Boolean).map((x) => x.replace(/^\./, "").toLowerCase()) })} />
                </span>
              </label>
              <label className="fld wide-fld">
                <span>{t("set.skip")}</span>
                <span className="in">
                  <textarea rows={3} defaultValue={s.catch_skip.join("\n")} onBlur={(e) => void save({ catch_skip: e.target.value.split(/\r?\n/).map((x) => x.trim()).filter(Boolean) })} />
                </span>
              </label>
            </div>
          </Sec>}
          {cur[0] === "update" && <Sec id="update" title={t("set.update")}>
            {tick?.store_build ? (
              <Opt title={t("set.store")}>
                <span />
              </Opt>
            ) : (
              <>
                <Opt title={t("set.updMode")} sub={t("set.updModeS")}>
                  <Tabs value={s.update_mode} label={t("set.updMode")} items={[["auto", t("set.updAuto")], ["notify", t("set.updNotify")], ["off", t("set.updOff")]]} onChange={(v) => void save({ update_mode: v })} />
                </Opt>
                <Opt title={t("set.channel")}>
                  <Tabs value={s.update_channel} label={t("set.channel")} items={[["stable", t("set.stable")], ["beta", t("set.beta")]]} onChange={(v) => void save({ update_channel: v })} />
                </Opt>
                <Opt title={t("set.checkNow")} sub={tick?.update.error ? t("banner.updFailed") : t(`set.upd.${tick?.update.state ?? "idle"}`, { v: tick?.update.version ?? "", defaultValue: "" })}>
                  <button type="button" className="fx btn btn-s" onClick={() => void api.updateCheck()}>
                    <Icon n="arrow-clockwise" />
                    {t("set.checkNow")}
                  </button>
                </Opt>
              </>
            )}
          </Sec>}
          {cur[0] === "privacy" && <Sec id="privacy" title={t("set.privacy")}>
            <div className="opt">
              <p className="t why">{t("set.privacyText")}</p>
            </div>
            <Opt title={t("set.clipboard")} sub={t("set.clipboardS")}>
              <Toggle on={s.clipboard} label={t("set.clipboard")} onChange={(v) => void save({ clipboard: v })} />
            </Opt>
            <Opt title={t("set.privacyLink")}>
              <button type="button" className="fx btn btn-s" onClick={() => void api.openLink("privacy")}>
                <Icon n="link" />
                {t("set.privacyLink")}
              </button>
            </Opt>
          </Sec>}
          {cur[0] === "about" && <Sec id="about" title={t("set.about")}>
            <div className="opt">
              <img className="logo" src="/gdm-icon-256.png" alt="" width={44} height={44} />
              <div className="t">
                <b>{t("app")}</b>
                <small>{t("set.version", { v: VERSION })}</small>
                <small>{t("set.attribution")}</small>
              </div>
            </div>
            <Opt title={t("set.license")}>
              <button type="button" className="fx btn btn-s" onClick={() => void api.openLink("license")}>
                <Icon n="link" />
                {t("set.readLicense")}
              </button>
            </Opt>
            <Opt title={t("set.github")}>
              <button type="button" className="fx btn btn-s" onClick={() => void api.openLink("github")}>
                <Icon n="link" />
                {t("set.github")}
              </button>
              <button
                type="button"
                className="fx btn btn-s"
                onClick={async () => {
                  await navigator.clipboard?.writeText("https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0");
                  setCopied(true);
                }}
              >
                <Icon n={copied ? "check" : "copy"} />
                {t("set.share")}
              </button>
            </Opt>
            <div className="opt top">
              <div className="t">
                <b>{t("set.support")}</b>
                <small>{t("set.supportS")}</small>
                <small>{t(KHQR ? "set.supportScan" : "set.supportNone")}</small>
              </div>
              {KHQR && <img src={KHQR} alt="KHQR" width={240} height={339} />}
            </div>
          </Sec>}
      </div>
    </Modal>
  );
}

export const num = (v: string, lo: number, hi: number) => Math.max(lo, Math.min(hi, Math.round(Number(v) || 0)));

/** a number field that saves on blur (same pattern as max_conns) */
export function NumIn({ label, value, lo, hi, unit, onSave }: { label: string; value: number; lo: number; hi: number; unit?: string; onSave: (v: number) => void }) {
  return (
    <span className="in short num-in">
      <input type="number" min={lo} max={hi} aria-label={label} defaultValue={value} key={value} onBlur={(e) => num(e.target.value, lo, hi) !== value && onSave(num(e.target.value, lo, hi))} />
      {unit && <span className="meta">{unit}</span>}
    </span>
  );
}

function ConnectionSection() {
  const { t } = useTranslation();
  const s = useStore((x) => x.settings)!;
  const save = useStore((x) => x.saveSettings);
  const [host, setHost] = useState("");
  const [conns, setConns] = useState(2);
  const [uaHost, setUaHost] = useState("");
  const [ua, setUa] = useState("");
  const addUa = () => {
    const h = uaHost.trim().toLowerCase();
    if (!h || !ua.trim()) return;
    void save({ site_agents: [...s.site_agents.filter((x) => x.host !== h), { host: h, ua: ua.trim() }] });
    setUaHost("");
    setUa("");
  };
  const add = () => {
    const h = host.trim().toLowerCase();
    if (!h || /\s/.test(h)) return;
    const rows: HostLimit[] = [...s.host_limits.filter((x) => x.host !== h), { host: h, conns: num(String(conns), 1, 32) }];
    void save({ host_limits: rows });
    setHost("");
  };
  return (
    <Sec id="connection" title={t("set.connection")}>
      <Opt title={t("set.limit")} sub={t("set.limitS")}>
        <NumIn label={t("set.limit")} value={s.limit_kbps || 512} lo={1} hi={10_000_000} unit={t("set.kbps")} onSave={(v) => void save({ limit_kbps: v })} />
        <Toggle on={s.limit_on} label={t("set.limit")} onChange={(v) => void save({ limit_on: v, limit_kbps: s.limit_kbps || 512 })} />
      </Opt>
      <Opt title={t("set.total")} sub={t("set.totalS")}>
        <NumIn label={t("set.total")} value={s.total_conns} lo={1} hi={256} onSave={(v) => void save({ total_conns: v })} />
      </Opt>
      <div className="opt col-opt">
        <div className="t">
          <b>{t("set.hosts")}</b>
          <small>{t("set.hostsS")}</small>
        </div>
        {s.host_limits.map((h) => (
          <div className="hl-row" key={h.host}>
            <span className="mono clip1" data-tip-overflow data-tip={h.host}>
              {h.host}
            </span>
            <span className="num">{t("set.connsN", { n: h.conns })}</span>
            <button type="button" className="fx mini" aria-label={t("set.removeX", { x: h.host })} data-tip={t("set.removeX", { x: h.host })} onClick={() => void save({ host_limits: s.host_limits.filter((x) => x.host !== h.host) })}>
              <Icon n="trash" />
            </button>
          </div>
        ))}
        <div className="hl-add">
          <label className="fld">
            <span>{t("set.host")}</span>
            <span className="in">
              <input className="mono" value={host} placeholder="*.example.com" onChange={(e) => setHost(e.target.value)} onKeyDown={(e) => e.key === "Enter" && add()} />
            </span>
          </label>
          <label className="fld">
            <span>{t("set.hostConns")}</span>
            <span className="in">
              <input className="mono" type="number" min={1} max={32} value={conns} onChange={(e) => setConns(Number(e.target.value))} />
            </span>
          </label>
          <button type="button" className="fx btn btn-s" onClick={add}>
            <Icon n="plus" />
            {t("set.addRow")}
          </button>
        </div>
      </div>
      <div className="opt col-opt">
        <label className="fld wide-fld">
          <span>{t("set.ua")}</span>
          <span className="in">
            <input className="mono" key={s.user_agent} defaultValue={s.user_agent} maxLength={512} placeholder={t("set.uaDefault")} onBlur={(e) => e.target.value.trim() !== s.user_agent && void save({ user_agent: e.target.value.trim() })} />
          </span>
        </label>
        <div className="t">
          <small>{t("set.uaSites")}</small>
        </div>
        {s.site_agents.map((a) => (
          <div className="hl-row" key={a.host}>
            <span className="mono clip1" data-tip-overflow data-tip={a.host}>
              {a.host}
            </span>
            <span className="mono clip1" data-tip-overflow data-tip={a.ua}>
              {a.ua}
            </span>
            <button type="button" className="fx mini" aria-label={t("set.removeX", { x: a.host })} data-tip={t("set.removeX", { x: a.host })} onClick={() => void save({ site_agents: s.site_agents.filter((x) => x.host !== a.host) })}>
              <Icon n="trash" />
            </button>
          </div>
        ))}
        <div className="hl-add">
          <label className="fld">
            <span>{t("set.host")}</span>
            <span className="in">
              <input className="mono" value={uaHost} placeholder="*.example.com" onChange={(e) => setUaHost(e.target.value)} />
            </span>
          </label>
          <label className="fld">
            <span>User-Agent</span>
            <span className="in">
              <input className="mono" value={ua} onChange={(e) => setUa(e.target.value)} />
            </span>
          </label>
          <button type="button" className="fx btn btn-s" onClick={addUa}>
            <Icon n="plus" />
            {t("set.addRow")}
          </button>
        </div>
      </div>
      <Opt title={t("set.quota")} sub={t("set.quotaS")}>
        <NumIn label={t("set.quota") + " MB"} value={s.quota_mb} lo={0} hi={100_000_000} unit="MB" onSave={(v) => void save({ quota_mb: v })} />
        <NumIn label={t("set.hours")} value={s.quota_hours} lo={1} hi={720} unit={t("set.hours")} onSave={(v) => void save({ quota_hours: v })} />
      </Opt>
    </Sec>
  );
}

/** §04 "scan each finished file with my antivirus program" (IDM Virus checking): hook "scan", native picker only */
function ScanRow() {
  const { t } = useTranslation();
  const h = useStore((x) => x.settings!.hooks?.scan);
  const [err, setErr] = useState("");
  const set = async (pick: boolean, args: string[], clear: boolean) => {
    try {
      await api.hookSet("scan", pick, args, clear);
      setErr("");
      useStore.setState({ settings: await api.settingsGet() });
    } catch (e) {
      setErr(errText(e));
    }
  };
  // an error replaces the description line (red, announced): nothing below moves
  return (
    <div className="opt">
      <div className="t">
        <b>{t("set.scan")}</b>
        {err ? (
          <small className="err-s clip1" role="alert" data-tip-overflow data-tip={t(`qs.e.${err}`, { defaultValue: err })}>
            {t(`qs.e.${err}`, { defaultValue: err })}
          </small>
        ) : (
          <small>{h ? fileName(h.program) + " " + h.args.join(" ") : t("set.scanS")}</small>
        )}
      </div>
      {h && (
        <button type="button" className="fx btn btn-g" onClick={() => void set(false, [], true)}>
          {t("qs.hookRemove")}
        </button>
      )}
      <button type="button" className="fx btn btn-s" onClick={() => void set(true, h?.args ?? ["{file}"], false)}>
        <Icon n="shield-check" />
        {t("qs.choose")}
      </button>
    </div>
  );
}

function ProxySection() {
  const { t } = useTranslation();
  const s = useStore((x) => x.settings)!;
  const save = useStore((x) => x.saveSettings);
  const p = s.proxy;
  const [auth, setAuth] = useState(p.auth);
  const [host, setHost] = useState(p.auth_host || p.http);
  const [user, setUser] = useState(p.user);
  const [pass, setPass] = useState("");
  const [msg, setMsg] = useState("");
  const setP = (q: Partial<typeof p>) => void save({ proxy: { ...useStore.getState().settings!.proxy, ...q } });
  const login = async (windows: boolean) => {
    try {
      await api.proxyLoginSet(host.trim(), windows ? "" : user.trim(), windows ? "" : pass, windows);
      setPass(""); // never kept after saving (P-01)
      setMsg("");
      useStore.setState({ settings: await api.settingsGet() });
    } catch (e) {
      setMsg(errText(e));
    }
  };
  const fld = (k: "http" | "socks", label: string) => (
    <label className="fld wide-fld">
      <span>{label}</span>
      <span className="in">
        <input className="mono" defaultValue={p[k]} placeholder="127.0.0.1:8080" onBlur={(e) => e.target.value.trim() !== p[k] && setP({ [k]: e.target.value.trim() })} />
      </span>
    </label>
  );
  return (
    <Sec id="proxy" title={t("set.proxy")}>
      <Opt title={t("px.mode")} sub={p.mode === "system" ? t("px.systemS") : undefined}>
        <Tabs value={p.mode} label={t("px.mode")} items={[["none", t("px.none")], ["system", t("px.system")], ["manual", t("px.manual")]]} onChange={(v) => setP({ mode: v })} />
      </Opt>
      {p.mode === "manual" && (
        <div className="opt col-opt">
          {fld("http", t("px.http"))}
          {fld("socks", t("px.socks"))}
          <label className="fld wide-fld">
            <span>{t("px.bypass")}</span>
            <span className="in">
              <textarea rows={3} defaultValue={p.bypass.join("\n")} onBlur={(e) => setP({ bypass: e.target.value.split(/\r?\n/).map((x) => x.trim()).filter(Boolean) })} />
            </span>
          </label>
        </div>
      )}
      {p.mode !== "none" && (
        <>
          <Opt title={t("px.auth")} sub={p.auth !== "none" ? t("px.saved", { host: p.auth_host, user: p.auth === "windows" ? "Windows" : p.user }) : undefined}>
            <Tabs
              value={auth}
              label={t("px.auth")}
              items={[["none", t("px.authNone")], ["basic", t("px.authBasic")], ["windows", t("px.authWin")]]}
              onChange={(v) => (setAuth(v), v === "none" && setP({ auth: "none", user: "" }))}
            />
          </Opt>
          {auth !== "none" && (
            <div className="opt col-opt">
              <label className="fld wide-fld">
                <span>{t("px.host")}</span>
                <span className="in">
                  <input className="mono" value={host} placeholder="proxy.example.org:8080" onChange={(e) => setHost(e.target.value)} />
                </span>
              </label>
              {auth === "basic" && (
                <div className="two even">
                  <label className="fld">
                    <span>{t("px.user")}</span>
                    <span className="in">
                      <input value={user} autoComplete="off" onChange={(e) => setUser(e.target.value)} />
                    </span>
                  </label>
                  <label className="fld">
                    <span>{t("px.pass")}</span>
                    <span className="in">
                      <input type="password" value={pass} autoComplete="new-password" onChange={(e) => setPass(e.target.value)} />
                    </span>
                  </label>
                </div>
              )}
              <p className="meta">{auth === "basic" ? t("px.passS") : t("px.winS")}</p>
              <ErrLine msg={msg} />
              <div className="row-btns flush">
                <button type="button" className="fx btn btn-s" onClick={() => void login(auth === "windows")}>
                  {auth === "windows" ? t("px.useWin") : t("px.save")}
                </button>
              </div>
            </div>
          )}
        </>
      )}
    </Sec>
  );
}

function LoginsSection() {
  const { t } = useTranslation();
  const [list, setList] = useState<SiteLogin[]>([]);
  const [prefix, setPrefix] = useState("");
  const [user, setUser] = useState("");
  const [pass, setPass] = useState("");
  const [err, setErr] = useState("");
  useEffect(() => void api.loginList().then(setList), []);
  const add = async () => {
    try {
      await api.loginSet(prefix.trim(), user.trim(), pass);
      setPrefix("");
      setUser("");
      setPass(""); // P-01: the password lives in Credential Manager only
      setErr("");
      setList(await api.loginList());
    } catch (e) {
      setErr(errText(e));
    }
  };
  return (
    <Sec id="logins" title={t("set.logins")}>
      <div className="opt col-opt">
        {list.length === 0 && <p className="meta">{t("lg.empty")}</p>}
        {list.map((l) => (
          <div className="hl-row" key={l.id}>
            <span className="mono clip1" data-tip-overflow data-tip={l.prefix}>
              {l.prefix}
            </span>
            <span className="clip1" data-tip-overflow data-tip={l.user}>
              {l.user}
            </span>
            <button type="button" className="fx mini" aria-label={t("lg.delete", { p: l.prefix })} data-tip={t("lg.delete", { p: l.prefix })} onClick={() => void api.loginDelete(l.id).then(() => api.loginList().then(setList))}>
              <Icon n="trash" />
            </button>
          </div>
        ))}
      </div>
      <div className="opt col-opt">
        <div className="t">
          <b>{t("lg.new")}</b>
          <small>{t("px.passS")}</small>
        </div>
        <label className="fld wide-fld">
          <span>{t("lg.prefix")}</span>
          <span className="in">
            <input className="mono" value={prefix} placeholder="https://files.example.org/folder/" onChange={(e) => setPrefix(e.target.value)} />
          </span>
        </label>
        <div className="two even">
          <label className="fld">
            <span>{t("px.user")}</span>
            <span className="in">
              <input value={user} autoComplete="off" onChange={(e) => setUser(e.target.value)} />
            </span>
          </label>
          <label className="fld">
            <span>{t("px.pass")}</span>
            <span className="in">
              <input type="password" value={pass} autoComplete="new-password" onChange={(e) => setPass(e.target.value)} />
            </span>
          </label>
        </div>
        <ErrLine msg={err && t(`lg.e.${err}`, { defaultValue: err })} />
        <div className="row-btns flush">
          <button type="button" className="fx btn btn-s" onClick={() => void add()}>
            <Icon n="plus" />
            {t("lg.save")}
          </button>
        </div>
      </div>
    </Sec>
  );
}

const SYS = ["SystemAsterisk", "SystemExclamation", "SystemNotification"];

function SoundsSection() {
  const { t } = useTranslation();
  const s = useStore((x) => x.settings)!;
  const save = useStore((x) => x.saveSettings);
  const events: SoundEvent[] = ["complete", "failed", "queue"];
  return (
    <Sec id="sounds" title={t("set.sounds")}>
      {events.map((ev) => {
        const v = s.sounds[ev] ?? "";
        const custom = v && !v.startsWith("system:");
        return (
          <Opt key={ev} title={t(`snd.${ev}`)}>
            <select className="fx sel snd" aria-label={t(`snd.${ev}`)} value={v} onChange={(e) => void save({ sounds: { ...s.sounds, [ev]: e.target.value } })}>
              <option value="">{t("snd.off")}</option>
              {SYS.map((a) => (
                <option key={a} value={"system:" + a}>
                  {t(`snd.${a}`)}
                </option>
              ))}
              {custom && <option value={v}>{fileName(v)}</option>}
            </select>
            <button type="button" className="fx btn btn-s" onClick={() => void api.soundPick(ev).then(async (p) => p && useStore.setState({ settings: await api.settingsGet() }))}>
              {t("snd.pick")}
            </button>
            <button type="button" className="fx mini" aria-label={t("snd.play") + " · " + t(`snd.${ev}`)} data-tip={t("snd.play")} disabled={!v} onClick={() => void api.soundTest(ev)}>
              <Icon n="play" />
            </button>
          </Opt>
        );
      })}
    </Sec>
  );
}

function VideoSection() {
  const { t } = useTranslation();
  const s = useStore((x) => x.settings)!;
  const save = useStore((x) => x.saveSettings);
  const [st, setSt] = useState<{ ytdlp: boolean; ffmpeg: boolean } | null>(null);
  const load = () => void api.componentsState().then(setSt);
  useEffect(load, []);
  const row = (name: string, ok: boolean | undefined) => (
    <Opt title={name}>
      {ok ? (
        <span className="ok">
          <Icon n="check-circle" fill />
          {t("vid.ok")}
        </span>
      ) : (
        <span className="meta">{t("vid.missing")}</span>
      )}
    </Opt>
  );
  return (
    <Sec id="video" title={t("set.videoSec")}>
      {row("yt-dlp", st?.ytdlp)}
      {row("FFmpeg", st?.ffmpeg)}
      <Opt title={t("set.video")} sub={t("set.videoS")}>
        <Toggle on={s.video_button} label={t("set.video")} onChange={(v) => void save({ video_button: v })} />
      </Opt>
      <div className="opt col-opt">{st && st.ytdlp && st.ffmpeg ? <p className="meta flush">{t("vid.notice")}</p> : <Components onDone={load} />}</div>
    </Sec>
  );
}

export function Onboarding() {
  const { t, i18n } = useTranslation();
  const s = useStore((x) => x.settings) as Settings;
  const save = useStore((x) => x.saveSettings);
  const [step, setStep] = useState(1);
  const list = useBrowsers();
  const [fc, setFc] = useState<FolderCheck | null>(null);
  useEffect(() => {
    if (step === 1) void api.folderCheck().then(setFc);
  }, [step, s.download_dir]);
  const finish = () => void save({ onboarded: true });
  const head = useRef<HTMLHeadingElement>(null);
  const first = useRef(true);
  useEffect(() => {
    // keyboard users keep their place: the new step's heading takes focus (the Next button was re-rendered)
    if (first.current) first.current = false;
    else head.current?.focus();
  }, [step]);
  const keyName = (k: string) => k.charAt(0).toUpperCase() + k.slice(1);
  return (
    <div id="shell" className="win">
      <div className="ob">
        <div className="card" role="dialog" aria-label={t(`ob.t${step}`)}>
          <div className="dots" aria-hidden="true">
            {[1, 2, 3].map((n) => (
              <i key={n} className={n === step ? "on" : n < step ? "past" : ""} />
            ))}
          </div>
          <p className="meta">{t("ob.step", { n: i18n.language === "km" ? "១២៣"[step - 1] : step })}</p>
          <h2 ref={head} tabIndex={-1}>
            {t(`ob.t${step}`)}
          </h2>
          <p>{t(`ob.p${step}`)}</p>
          {step === 1 && (
            <div className="card list-card">
              <Opt title={t("set.lang")}>
                <Tabs value={s.lang} label={t("set.lang")} items={[["km", "ខ្មែរ"], ["en", "English"]]} onChange={(v) => void save({ lang: v })} />
              </Opt>
              <Opt title={t("set.theme")}>
                <Tabs value={s.theme} label={t("set.theme")} items={[["system", t("set.tSys")], ["light", t("set.tLight")], ["dark", t("set.tDark")]]} onChange={(v) => void save({ theme: v })} />
              </Opt>
              <Opt title={t("set.folder")} sub={s.download_dir}>
                <button
                  type="button"
                  className="fx btn btn-s"
                  onClick={async () => {
                    const p = await api.pickDownloadDir();
                    if (p) useStore.setState({ settings: { ...s, download_dir: p } });
                  }}
                >
                  {t("set.change")}
                </button>
              </Opt>
              {fc && fc.fs && fc.fs !== "NTFS" && fc.fs !== "ReFS" && <p className="warn-t">{t("ob.fsWarn", { fs: fc.fs })}</p>}
              {fc && fc.free > 0 && fc.free < 2 * 1024 ** 3 && <p className="warn-t">{t("ob.spaceWarn", { free: bytes(fc.free) })}</p>}
            </div>
          )}
          {step === 2 && (
            <>
              <div className="card list-card">
                <BrowserRows list={list} />
              </div>
              <div className="tip">
                <Icon n="info" />
                <span>{t("ob.tip", { force: keyName(s.force_key), bypass: keyName(s.bypass_key) })}</span>
              </div>
            </>
          )}
          {step === 3 && (
            <div className="card list-card">
              <Opt title={t("set.video")} sub={t("ob.videoS")}>
                <Toggle on={s.video_button} label={t("set.video")} onChange={(v) => void save({ video_button: v })} />
              </Opt>
              <Opt title={t("set.clipboard")} sub={t("ob.clipS")}>
                <Toggle on={s.clipboard} label={t("set.clipboard")} onChange={(v) => void save({ clipboard: v })} />
              </Opt>
              <Opt title={t("set.updMode")} sub={t("ob.updS")}>
                <Tabs value={s.update_mode} label={t("set.updMode")} items={[["auto", t("set.updAuto")], ["notify", t("set.updNotify")], ["off", t("set.updOff")]]} onChange={(v) => void save({ update_mode: v })} />
              </Opt>
            </div>
          )}
          <div className="mfoot">
            <button type="button" className="fx btn btn-g" onClick={finish}>
              {t("ob.skip")}
            </button>
            <span className="sp" />
            {step > 1 && (
              <button type="button" className="fx btn btn-s" onClick={() => setStep(step - 1)}>
                {t("ob.back")}
              </button>
            )}
            <button type="button" className="fx btn btn-p fx-strong" onClick={() => (step < 3 ? setStep(step + 1) : finish())}>
              {step < 3 ? t("ob.next") : t("ob.finish")}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
