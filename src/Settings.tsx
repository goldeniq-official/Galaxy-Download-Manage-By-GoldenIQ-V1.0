// SPDX-License-Identifier: RPL-1.5
// Settings (§06 screen 4) and first-run onboarding (J2: 3 steps, skippable; D5 video button off, asked here).
import { useEffect, useRef, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { api, type Browser, type FolderCheck, type Settings } from "./api";
import { useStore } from "./store";
import { bytes } from "./status";
import { Icon, Tabs, Toggle } from "./ui";

const VERSION = "0.9.0";

function Opt({ title, sub, children }: { title: string; sub?: string; children: ReactNode }) {
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

function Sec({ id, title, children }: { id: string; title: string; children: ReactNode }) {
  return (
    <section className="sec" id={"set-" + id} aria-label={title}>
      <h3>{title}</h3>
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
          <div className="brow" key={b.key}>
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
  const [sec, setSec] = useState("general");
  const [copied, setCopied] = useState(false);
  const lang = i18n.language;
  const secs: [string, string, string][] = [
    ["general", "gear-six", t("set.general")],
    ["downloads", "download-simple", t("set.downloads")],
    ["speed", "gauge", t("set.speed")],
    ["browsers", "browser", t("set.browsers")],
    ["update", "arrow-clockwise", t("set.update")],
    ["privacy", "shield-check", t("set.privacy")],
    ["about", "info", t("set.about")],
  ];
  const go = (k: string) => {
    setSec(k);
    document.getElementById("set-" + k)?.scrollIntoView({ block: "start" });
  };
  const num = (v: string, lo: number, hi: number) => Math.max(lo, Math.min(hi, Math.round(Number(v) || 0)));
  return (
    <div className="set">
      <nav className="rail" aria-label={t("nav.settings")}>
        {secs.map(([k, ic, label]) => (
          <button key={k} type="button" className={"fx nav" + (sec === k ? " is-selected" : "")} aria-current={sec === k ? "true" : undefined} onClick={() => go(k)}>
            <Icon n={ic} lg fill={sec === k} />
            <span className="lbl">{label}</span>
          </button>
        ))}
      </nav>
      <main className="main">
        <div className="top">
          <h1>{t("nav.settings")}</h1>
        </div>
        <div className="sbody">
          <Sec id="general" title={t("set.look")}>
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
          </Sec>
          <Sec id="downloads" title={t("set.downloads")}>
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
            <div className="opt col-opt">
              <div className="t">
                <b>{t("set.cats")}</b>
              </div>
              {s.categories.map((c, i) => (
                <label className="fld cat-row" key={c.id}>
                  <span>{lang === "en" ? c.name_en : c.name_km}</span>
                  <span className="in">
                    <input
                      aria-label={(lang === "en" ? c.name_en : c.name_km) + " · " + t("set.exts")}
                      defaultValue={c.exts.join(" ")}
                      placeholder={t("set.exts")}
                      onBlur={(e) => {
                        const cats = s.categories.map((x, j) => (j === i ? { ...x, exts: e.target.value.split(/\s+/).filter(Boolean) } : x));
                        void save({ categories: cats });
                      }}
                    />
                  </span>
                </label>
              ))}
            </div>
          </Sec>
          <Sec id="speed" title={t("set.speed")}>
            <Opt title={t("set.limit")} sub={t("set.limitS")}>
              <span className="in short num-in">
                <input type="number" min={1} aria-label={t("set.kbps")} defaultValue={s.limit_kbps || 512} onBlur={(e) => void save({ limit_kbps: num(e.target.value, 1, 10_000_000) })} />
                <span className="meta">{t("set.kbps")}</span>
              </span>
              <Toggle on={s.limit_on} label={t("set.limit")} onChange={(v) => void save({ limit_on: v, limit_kbps: s.limit_kbps || 512 })} />
            </Opt>
          </Sec>
          <Sec id="browsers" title={t("set.browsers")}>
            <Opt title={t("set.catch")} sub={t("set.catchS")}>
              <Toggle on={s.catch_enabled} label={t("set.catch")} onChange={(v) => void save({ catch_enabled: v })} />
            </Opt>
            <div className="opt col-opt">
              <BrowserRows list={list} />
            </div>
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
            <Opt title={t("set.video")} sub={t("set.videoS")}>
              <Toggle on={s.video_button} label={t("set.video")} onChange={(v) => void save({ video_button: v })} />
            </Opt>
          </Sec>
          <Sec id="update" title={t("set.update")}>
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
                <Opt title={t("set.checkNow")} sub={tick?.update.error ?? tick?.update.state ?? ""}>
                  <button type="button" className="fx btn btn-s" onClick={() => void api.updateCheck()}>
                    <Icon n="arrow-clockwise" />
                    {t("set.checkNow")}
                  </button>
                </Opt>
              </>
            )}
          </Sec>
          <Sec id="privacy" title={t("set.privacy")}>
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
          </Sec>
          <Sec id="about" title={t("set.about")}>
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
                {t("set.license")}
              </button>
              <button type="button" className="fx btn btn-s" onClick={() => void api.openLink("github")}>
                {t("set.github")}
              </button>
              <button
                type="button"
                className="fx btn btn-s"
                onClick={async () => {
                  await navigator.clipboard?.writeText("https://github.com/goldeniq-official/galaxy-download-manager");
                  setCopied(true);
                }}
              >
                <Icon n="copy" />
                {copied ? t("set.copied") : t("set.share")}
              </button>
            </Opt>
          </Sec>
        </div>
      </main>
    </div>
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
    if (step === 3) void api.folderCheck().then(setFc);
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
            </div>
          )}
          {step === 2 && (
            <>
              <BrowserRows list={list} />
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
