// SPDX-License-Identifier: RPL-1.5
// Site grabber (F09, IDM "Run site grabber"): the 4-step wizard (start page, save, explore, file types),
// then the found files with check boxes. The folder comes from the native picker only (X-01); the login
// is sent as Basic to the start page's own origin only (W-01). Nothing found is fetched until "Add".
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, errText, type GrabConfig, type GrabFound, type GrabProgress, type GrabReport } from "./api";
import { useStore } from "./store";
import { Icon, Modal } from "./ui";
import { Spinner } from "./Dialogs";

const close = () => useStore.getState().setDlg(null);

/** IDM templates: all files, images, video, audio + documents and archives */
export const TEMPLATES: Record<string, string> = {
  all: "*",
  images: "*.jpg, *.jpeg, *.png, *.gif, *.webp, *.bmp, *.svg",
  video: "*.mp4, *.mkv, *.webm, *.avi, *.mov, *.wmv, *.flv, *.m4v",
  audio: "*.mp3, *.m4a, *.aac, *.flac, *.wav, *.ogg, *.opus, *.wma",
  docs: "*.pdf, *.doc, *.docx, *.xls, *.xlsx, *.ppt, *.pptx, *.txt, *.epub",
  archives: "*.zip, *.rar, *.7z, *.tar, *.gz",
};

const DEFAULTS: GrabConfig = {
  start: "",
  avoid: "",
  depth: 3,
  other_depth: 0,
  domains: "",
  subdomains: false,
  no_parent: true,
  include: "",
  exclude: "",
  robots: true,
  js: false,
  delay_ms: 1000,
  types: "*",
};

function Check({ on, set, label, sub }: { on: boolean; set: (v: boolean) => void; label: string; sub?: string }) {
  return (
    <button type="button" className="fx check" role="checkbox" aria-checked={on} onClick={() => set(!on)}>
      <i>{on && <Icon n="check" />}</i>
      <span>
        {label}
        {sub && <small className="meta block">{sub}</small>}
      </span>
    </button>
  );
}

function Text({ label, value, set, ph, mono }: { label: string; value: string; set: (v: string) => void; ph?: string; mono?: boolean }) {
  return (
    <label className="fld">
      <span>{label}</span>
      <span className="in">
        <input className={mono ? "mono" : undefined} value={value} placeholder={ph} maxLength={4096} onChange={(e) => set(e.target.value)} />
      </span>
    </label>
  );
}

function Num({ label, value, set, min, max }: { label: string; value: number; set: (v: number) => void; min: number; max: number }) {
  return (
    <label className="fld">
      <span>{label}</span>
      <span className="in">
        <input className="mono" type="number" min={min} max={max} value={value} onChange={(e) => set(Math.max(min, Math.min(max, Math.round(Number(e.target.value) || 0))))} />
      </span>
    </label>
  );
}

export function GrabberDialog() {
  const { t } = useTranslation();
  const [step, setStep] = useState(1);
  const [c, setC] = useState<GrabConfig>(DEFAULTS);
  const [tpl, setTpl] = useState("all");
  const [user, setUser] = useState("");
  const [pass, setPass] = useState("");
  const [dir, setDir] = useState("");
  const [keep, setKeep] = useState(false);
  const [startNow, setStartNow] = useState(false);
  const [hideDup, setHideDup] = useState(true);
  const [busy, setBusy] = useState(false);
  const [prog, setProg] = useState<GrabProgress | null>(null);
  const [rep, setRep] = useState<GrabReport | null>(null);
  const [off, setOff] = useState<Set<string>>(new Set());
  const [err, setErr] = useState("");
  const set = <K extends keyof GrabConfig>(k: K) => (v: GrabConfig[K]) => setC({ ...c, [k]: v });

  useEffect(() => void api.grabPickDir(false).then(setDir, () => {}), []);
  useEffect(() => {
    if (!busy) return;
    const h = setInterval(() => void api.grabProgress().then((p) => p && setProg(p), () => {}), 500);
    return () => clearInterval(h);
  }, [busy]);

  const startOk = /^https?:\/\/[^\s/]+/i.test(c.start.trim());
  const run = async () => {
    setErr("");
    setBusy(true);
    setProg(null);
    try {
      const r = await api.grabRun({ ...c, start: c.start.trim() }, user.trim() || null, pass || null);
      setRep(r);
      setOff(new Set());
    } catch (e) {
      setErr(errText(e));
    } finally {
      setBusy(false);
    }
  };
  const shown: GrabFound[] = useMemo(() => (rep ? rep.files.filter((f) => !(hideDup && f.dup)) : []), [rep, hideDup]);
  const chosen = shown.filter((f) => !off.has(f.url));
  const add = async () => {
    if (!chosen.length) return;
    try {
      const n = await api.grabAdd(chosen, c.start.trim(), keep, startNow);
      useStore.getState().notify("grabbed", String(n));
      close();
    } catch (e) {
      setErr(errText(e));
    }
  };
  const title = t("grab.title");

  if (busy || rep) {
    return (
      <Modal label={title} onClose={() => (busy ? void api.grabCancel() : close())} wide>
        <h2>{title}</h2>
        {busy && (
          <>
            <Spinner text={t("grab.running", { pages: prog?.pages ?? 0, files: prog?.files ?? 0 })} />
            {prog && (
              <div className="url clip1 meta" title={prog.url}>
                {prog.url}
              </div>
            )}
          </>
        )}
        {rep && (
          <>
            <span className="lab">{t("grab.summary", { files: rep.files.length, pages: rep.pages, requests: rep.requests })}</span>
            <p className="meta">{t(`grab.stop.${rep.stop}`, { defaultValue: rep.stop })}</p>
            <Check on={hideDup} set={setHideDup} label={t("grab.hideDup")} />
            <div className="row-btns">
              <button type="button" className="fx btn btn-g" onClick={() => setOff(new Set())}>
                {t("grab.all")}
              </button>
              <button type="button" className="fx btn btn-g" onClick={() => setOff(new Set(shown.map((f) => f.url)))}>
                {t("grab.none")}
              </button>
            </div>
            <div className="url lines grab-list" role="group" aria-label={t("grab.found")}>
              {shown.slice(0, 2000).map((f) => (
                <Check
                  key={f.url}
                  on={!off.has(f.url)}
                  set={(v) => {
                    const n = new Set(off);
                    if (v) n.delete(f.url);
                    else n.add(f.url);
                    setOff(n);
                  }}
                  label={f.name}
                  sub={f.url}
                />
              ))}
              {shown.length > 2000 && <div className="meta">{t("grab.more", { n: shown.length - 2000 })}</div>}
              {!shown.length && <div className="meta">{t("grab.nothing")}</div>}
            </div>
          </>
        )}
        <p className="err-line" role="alert">
          {err && t(`grab.e.${err}`, { defaultValue: err })}
        </p>
        <div className="mfoot">
          <span className="sp" />
          {busy ? (
            <button type="button" className="fx btn btn-g" onClick={() => void api.grabCancel()}>
              {t("grab.stopBtn")}
            </button>
          ) : (
            <>
              <button type="button" className="fx btn btn-g" onClick={() => (setRep(null), setStep(4))}>
                {t("grab.back")}
              </button>
              <button type="button" className="fx btn btn-p fx-strong" aria-disabled={!chosen.length} onClick={() => void add()}>
                {t("grab.add", { n: chosen.length })}
              </button>
            </>
          )}
        </div>
      </Modal>
    );
  }

  return (
    <Modal label={title} onClose={close} wide>
      <h2>{title}</h2>
      <p className="lab">{t("grab.step", { n: step, name: t(`grab.s${step}`) })}</p>
      {step === 1 && (
        <>
          <Text label={t("grab.start")} value={c.start} set={set("start")} ph="https://example.com/gallery/" mono />
          <label className="fld">
            <span>{t("grab.template")}</span>
            <select
              className="fx sel"
              value={tpl}
              onChange={(e) => {
                setTpl(e.target.value);
                setC({ ...c, types: TEMPLATES[e.target.value] });
              }}
            >
              {Object.keys(TEMPLATES).map((k) => (
                <option key={k} value={k}>
                  {t(`grab.tpl.${k}`)}
                </option>
              ))}
            </select>
          </label>
          <div className="two">
            <Text label={t("grab.user")} value={user} set={setUser} />
            <label className="fld">
              <span>{t("grab.pass")}</span>
              <span className="in">
                <input type="password" value={pass} maxLength={1024} onChange={(e) => setPass(e.target.value)} />
              </span>
            </label>
          </div>
          <Text label={t("grab.avoid")} value={c.avoid} set={set("avoid")} ph="*/cart/empty*, /account/close" mono />
          <p className="meta">{t("grab.avoidS")}</p>
        </>
      )}
      {step === 2 && (
        <>
          <label className="fld">
            <span>{t("grab.dir")}</span>
            <span className="in">
              <input className="mono" value={dir} readOnly />
              <button type="button" className="fx btn btn-g" onClick={() => void api.grabPickDir(true).then(setDir, (e) => setErr(errText(e)))}>
                {t("grab.change")}
              </button>
            </span>
          </label>
          <Check on={keep} set={setKeep} label={t("grab.keep")} sub={t("grab.keepS")} />
          <Check on={startNow} set={setStartNow} label={t("grab.startNow")} sub={t("grab.startNowS")} />
        </>
      )}
      {step === 3 && (
        <>
          <div className="three">
            <Num label={t("grab.depth")} value={c.depth} set={set("depth")} min={0} max={20} />
            <Num label={t("grab.otherDepth")} value={c.other_depth} set={set("other_depth")} min={0} max={5} />
            <Num label={t("grab.delay")} value={c.delay_ms / 1000} set={(v) => set("delay_ms")(v * 1000)} min={1} max={60} />
          </div>
          <Text label={t("grab.domains")} value={c.domains} set={set("domains")} ph="cdn.example.com, *.example.net" mono />
          <Check on={c.subdomains} set={set("subdomains")} label={t("grab.subdomains")} />
          <Check on={c.no_parent} set={set("no_parent")} label={t("grab.noParent")} />
          <div className="two">
            <Text label={t("grab.include")} value={c.include} set={set("include")} ph="/gallery/*" mono />
            <Text label={t("grab.exclude")} value={c.exclude} set={set("exclude")} ph="*/thumbs/*" mono />
          </div>
          <Check on={c.robots} set={set("robots")} label={t("grab.robots")} sub={t("grab.robotsS")} />
          <Check on={c.js} set={set("js")} label={t("grab.js")} sub={t("grab.jsS")} />
        </>
      )}
      {step === 4 && (
        <>
          <Text label={t("grab.types")} value={c.types} set={set("types")} ph="*.jpg, *.png" mono />
          <Check on={hideDup} set={setHideDup} label={t("grab.hideDup")} sub={t("grab.hideDupS")} />
          <p className="meta">{t("grab.limits")}</p>
        </>
      )}
      <p className="err-line" role="alert">
        {err ? t(`grab.e.${err}`, { defaultValue: err }) : step === 1 && c.start && !startOk ? t("grab.e.bad-url") : ""}
      </p>
      <div className="mfoot">
        <span className="sp" />
        <button type="button" className="fx btn btn-g" onClick={() => (step === 1 ? close() : setStep(step - 1))}>
          {step === 1 ? t("info.cancel") : t("grab.back")}
        </button>
        {step < 4 ? (
          <button type="button" className="fx btn btn-p fx-strong" aria-disabled={!startOk} onClick={() => startOk && setStep(step + 1)}>
            {t("grab.next")}
          </button>
        ) : (
          <button type="button" className="fx btn btn-p fx-strong" aria-disabled={!startOk || !c.types.trim()} onClick={() => startOk && c.types.trim() && void run()}>
            {t("grab.run")}
          </button>
        )}
      </div>
    </Modal>
  );
}
