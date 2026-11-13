// SPDX-License-Identifier: RPL-1.5
// G7 dialogs (IDM: Add batch download, Import preview I-03, Refresh download address, video quality F07) and the
// queue "when done" countdown banner. Commands take ids and plain values only; files come from native pickers.
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";
import { api, errText, type Batch, type BatchPreview, type Candidate, type ImportPreview, type Probe, type Queue } from "./api";
import { useStore } from "./store";
import { bytes } from "./status";
import { Icon, Modal } from "./ui";
import { GrabberDialog } from "./Grabber";

/** the two built-in queues are named in the user's language; user queues keep their own name */
export const qName = (t: TFunction, q: Queue | undefined) => (!q ? "" : q.id === "main" ? t("q.main") : q.id === "sync" ? t("q.sync") : q.name || q.id);

/** a translated error for a command's error code, or the backend's own text */
const errMsg = (t: TFunction, group: string, e: unknown) => {
  const c = errText(e);
  return t(`${group}.${c}`, { defaultValue: c });
};

/** P1 #19 "Remember this folder for <category>": the folder comes from the native picker only */
export async function rememberDir(id: string, clear: boolean) {
  const r = await api.categoryPickDir(id, clear).catch(() => null);
  if (r != null) useStore.setState({ settings: await api.settingsGet() });
}

export function QueueSelect({ value, onChange, label, none }: { value: string; onChange: (v: string) => void; label: string; none?: boolean }) {
  const { t } = useTranslation();
  const queues = useStore((s) => s.settings!.queues);
  return (
    <label className="fld">
      <span>{label}</span>
      <select className="fx sel" value={value} onChange={(e) => onChange(e.target.value)}>
        {none && <option value="">{t("q.none")}</option>}
        {queues.map((q) => (
          <option key={q.id} value={q.id}>
            {qName(t, q)}
          </option>
        ))}
      </select>
    </label>
  );
}

export function Spinner({ text }: { text: string }) {
  return (
    <div className="note" role="status">
      <span className="spin">
        <Icon n="arrow-clockwise" />
      </span>
      <span>{text}</span>
    </div>
  );
}

export function G7Dialogs() {
  const dlg = useStore((s) => s.dlg);
  if (!dlg) return null;
  if (dlg.k === "batch") return <BatchDialog />;
  if (dlg.k === "import") return <ImportDialog pv={dlg.pv} />;
  if (dlg.k === "refresh") return <RefreshDialog id={dlg.id} />;
  if (dlg.k === "url") return <UrlDialog id={dlg.id} />;
  if (dlg.k === "grab") return <GrabberDialog />;
  return <VideoDialog initial={dlg.url} />;
}

const close = () => useStore.getState().setDlg(null);

function BatchDialog() {
  const { t } = useTranslation();
  const [b, setB] = useState<Batch>({ pattern: "", from: "1", to: "10", width: 0 });
  const [pv, setPv] = useState<BatchPreview | null>(null);
  const [err, setErr] = useState("");
  const [ok, setOk] = useState(false);
  const [queue, setQueue] = useState("main");
  useEffect(() => {
    let live = true;
    setOk(false);
    if (!b.pattern.trim()) {
      setPv(null);
      setErr("");
      return;
    }
    const tm = setTimeout(
      () =>
        void api
          .batchPreview(b)
          .then((p) => live && (setPv(p), setErr(p.count > 100_000 ? "too-many" : "")))
          .catch((e) => live && (setPv(null), setErr(errText(e)))),
      150,
    );
    return () => {
      live = false;
      clearTimeout(tm);
    };
  }, [b]);
  const field = (k: "from" | "to") => (
    <label className="fld">
      <span>{t(`batch.${k}`)}</span>
      <span className="in">
        <input className="mono" value={b[k]} maxLength={12} onChange={(e) => setB({ ...b, [k]: e.target.value.trim() })} />
      </span>
    </label>
  );
  const can = !!pv && !err && pv.count > 0 && (!pv.needs_confirm || ok);
  const add = async () => {
    if (!can) return;
    try {
      await api.batchAdd(b, ok, queue || null);
      close();
    } catch (e) {
      setErr(errText(e));
    }
  };
  return (
    <Modal label={t("batch.title")} onClose={close} wide>
      <h2>{t("batch.title")}</h2>
      <label className="fld">
        <span>{t("batch.pattern")}</span>
        <span className="in">
          <input className="mono" value={b.pattern} placeholder="https://example.com/photo-*.jpg" aria-invalid={!!err} onChange={(e) => setB({ ...b, pattern: e.target.value })} />
        </span>
      </label>
      <div className="three">
        {field("from")}
        {field("to")}
        <label className="fld">
          <span>{t("batch.width")}</span>
          <span className="in">
            <input className="mono" type="number" min={0} max={10} value={b.width} onChange={(e) => setB({ ...b, width: Math.max(0, Math.min(10, Math.round(Number(e.target.value) || 0))) })} />
          </span>
        </label>
      </div>
      {err && (
        <p className="err-t" role="alert">
          {errMsg(t, "batch.e", err)}
        </p>
      )}
      {pv && !err && (
        <>
          <span className="lab">{t("batch.count", { n: pv.count.toLocaleString("en-US") })}</span>
          <div className="url lines">
            {pv.first.map((u) => (
              <div key={u} className="clip1" title={u}>
                {u}
              </div>
            ))}
            {pv.count > pv.first.length && <div>…</div>}
          </div>
          {pv.needs_confirm && (
            <button type="button" className="fx check" role="checkbox" aria-checked={ok} onClick={() => setOk(!ok)}>
              <i>{ok && <Icon n="check" />}</i>
              <span>
                {t("batch.confirm", { n: pv.count.toLocaleString("en-US") })}
                <small className="meta block">{t("batch.confirmS")}</small>
              </span>
            </button>
          )}
        </>
      )}
      <QueueSelect value={queue} onChange={setQueue} label={t("batch.queue")} none />
      <div className="mfoot">
        <span className="sp" />
        <button type="button" className="fx btn btn-g" onClick={close}>
          {t("info.cancel")}
        </button>
        <button type="button" className="fx btn btn-p fx-strong" aria-disabled={!can} onClick={() => void add()}>
          {t("batch.add", { n: pv && !err ? pv.count.toLocaleString("en-US") : 0 })}
        </button>
      </div>
    </Modal>
  );
}

function ImportDialog({ pv }: { pv: ImportPreview }) {
  const { t } = useTranslation();
  const [queue, setQueue] = useState("");
  const [restore, setRestore] = useState(false);
  const [err, setErr] = useState("");
  const add = async () => {
    try {
      const n = await api.importCommit(queue || null, restore);
      close();
      useStore.getState().notify("imported", t("imp.added", { n }));
    } catch (e) {
      setErr(errText(e));
    }
  };
  return (
    <Modal label={t("imp.title")} onClose={close} wide>
      <h2>{t("imp.title")}</h2>
      <p className="why">{t("imp.count", { n: pv.count })}</p>
      <span className="lab">{t("imp.first", { n: Math.min(8, pv.sample.length) })}</span>
      <div className="batch imp" role="list">
        {pv.sample.slice(0, 8).map((it, i) => (
          <div key={i} className="imp-row" role="listitem">
            <b className="clip1" title={it.name || it.url}>
              {it.name || it.url}
            </b>
            <span className="mono clip1" title={it.url}>
              {it.url}
            </span>
          </div>
        ))}
      </div>
      {pv.has_settings && (
        <button type="button" className="fx check" role="checkbox" aria-checked={restore} onClick={() => setRestore(!restore)}>
          <i>{restore && <Icon n="check" />}</i>
          <span>{t("imp.settings")}</span>
        </button>
      )}
      <QueueSelect value={queue} onChange={setQueue} label={t("batch.queue")} none />
      {err && (
        <p className="err-t" role="alert">
          {errText(err)}
        </p>
      )}
      <div className="mfoot">
        <span className="sp" />
        <button type="button" className="fx btn btn-g" onClick={close}>
          {t("info.cancel")}
        </button>
        <button type="button" className="fx btn btn-p fx-strong" onClick={() => void add()}>
          {t("batch.add", { n: pv.count })}
        </button>
      </div>
    </Modal>
  );
}

function RefreshDialog({ id }: { id: string }) {
  const { t } = useTranslation();
  const job = useStore((s) => s.jobs.find((j) => j.id === id));
  const [c, setC] = useState<Candidate | null>(null);
  const [paste, setPaste] = useState("");
  const [err, setErr] = useState("");
  const def = useRef<HTMLButtonElement>(null);
  const started = useRef(""); // the browser opens once, also under StrictMode's double effect
  useEffect(() => {
    if (started.current === id) return;
    started.current = id;
    void api.refreshStart(id).catch((e) => setErr(errText(e)));
  }, [id]);
  useEffect(() => {
    if (c) return;
    const tm = setInterval(() => void api.refreshCandidate(id).then((x) => x && setC(x), () => {}), 1000);
    return () => clearInterval(tm);
  }, [id, c]);
  useEffect(() => def.current?.focus(), [c]);
  const decide = async (choice: "resume" | "continue" | "restart" | "cancel") => {
    try {
      await api.refreshDecide(id, choice);
      close();
    } catch (e) {
      setErr(errText(e));
    }
  };
  const pasteUrl = async () => {
    if (!/^https?:\/\/\S+$/i.test(paste.trim())) return setErr(t("addDlg.bad"));
    setErr("");
    setC(null);
    await api.refreshPaste(id, paste.trim()).catch((e) => setErr(errText(e)));
  };
  const ck = c?.check;
  const good = c && !c.error && ck;
  return (
    <Modal label={t("ref.title")} onClose={() => void decide("cancel")} wide>
      <h2>{t("ref.title")}</h2>
      {job && (
        <p className="meta clip1" title={job.name}>
          {job.name}
        </p>
      )}
      {!c && <Spinner text={t("ref.wait")} />}
      {c?.error && (
        <p className="err-t" role="alert">
          {c.error === "Html" ? t("ref.html") : t("ref.err", { e: c.error })}
        </p>
      )}
      {c && (
        <>
          <span className="lab">{t("ref.new")}</span>
          <div className="url">{c.url}</div>
        </>
      )}
      {good && (
        <>
          <p className="why">
            {t("ref.size", { s: bytes(ck.size) })} · {ck.same_size ? t("ref.same") : t("ref.diff")}
          </p>
          {!ck.strong && ck.same_size && ck.seam !== false && (
            <p className="warn-t flush" role="note">
              {t("ref.risk")}
            </p>
          )}
          <p className="meta">{t("ref.restartS")}</p>
        </>
      )}
      {(!c || c.error) && (
        <div className="fld-row">
          <label className="fld">
            <span>{t("ref.paste")}</span>
            <span className="in">
              <input className="mono" value={paste} onChange={(e) => setPaste(e.target.value)} onKeyDown={(e) => e.key === "Enter" && void pasteUrl()} />
            </span>
          </label>
          <button type="button" className="fx btn btn-s" onClick={() => void pasteUrl()}>
            {t("ref.use")}
          </button>
        </div>
      )}
      {err && (
        <p className="err-t" role="alert">
          {err}
        </p>
      )}
      <div className="mfoot">
        <span className="sp" />
        <button type="button" className="fx btn btn-g" onClick={() => void decide("cancel")}>
          {t("info.cancel")}
        </button>
        {good && (
          <>
            <button type="button" className="fx btn btn-s" disabled={!ck.strong} onClick={() => void decide("resume")}>
              {t("ref.resume")}
            </button>
            {!ck.strong && ck.same_size && ck.seam !== false && (
              <button type="button" className="fx btn btn-s" onClick={() => void decide("continue")}>
                {t("ref.continue")}
              </button>
            )}
            <button ref={def} type="button" className="fx btn btn-p fx-strong" onClick={() => void decide("restart")}>
              {t("ref.restart")}
            </button>
          </>
        )}
      </div>
    </Modal>
  );
}

/** Properties: a new address for a stopped download (P1 #26); the validator is checked on the next run */
function UrlDialog({ id }: { id: string }) {
  const { t } = useTranslation();
  const job = useStore((s) => s.jobs.find((j) => j.id === id));
  const [url, setUrl] = useState(job?.url ?? "");
  const [err, setErr] = useState("");
  const go = async () => {
    if (!/^https?:\/\/\S+$/i.test(url.trim())) return setErr(t("addDlg.bad"));
    try {
      await api.jobSetUrl(id, url.trim());
      close();
    } catch (e) {
      setErr(t(`prop.e.${errText(e)}`, { defaultValue: errText(e) }));
    }
  };
  return (
    <Modal label={t("prop.url")} onClose={close}>
      <h2>{t("prop.url")}</h2>
      {job && (
        <p className="meta clip1" title={job.name}>
          {job.name}
        </p>
      )}
      <label className="fld">
        <span>{t("addDlg.url")}</span>
        <span className="in">
          <input className="mono" value={url} aria-invalid={!!err} onChange={(e) => (setUrl(e.target.value), setErr(""))} onKeyDown={(e) => e.key === "Enter" && void go()} />
        </span>
      </label>
      <p className="meta">{t("prop.urlS")}</p>
      {err && (
        <p className="err-t" role="alert">
          {err}
        </p>
      )}
      <div className="mfoot">
        <span className="sp" />
        <button type="button" className="fx btn btn-g" onClick={close}>
          {t("info.cancel")}
        </button>
        <button type="button" className="fx btn btn-p fx-strong" onClick={() => void go()}>
          {t("prop.save")}
        </button>
      </div>
    </Modal>
  );
}

/** D10 notice + install button, shared by the video dialog and Settings > Video */
export function Components({ onDone }: { onDone?: () => void }) {
  const { t } = useTranslation();
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");
  const install = async () => {
    setBusy(true);
    setErr("");
    try {
      await api.componentsInstall();
      onDone?.();
    } catch (e) {
      setErr(errText(e));
    }
    setBusy(false);
  };
  return (
    <div className="col-tip">
      <Icon n="info" />
      <div>
        <p className="flush">{t("vid.notice")}</p>
        {busy ? (
          <Spinner text={t("vid.installing")} />
        ) : (
          <button type="button" className="fx btn btn-s" onClick={() => void install()}>
            <Icon n="download-simple" />
            {t("vid.install")}
          </button>
        )}
        {err && (
          <p className="err-t" role="alert">
            {t("vid.installErr", { e: err })}
          </p>
        )}
      </div>
    </div>
  );
}

function VideoDialog({ initial }: { initial?: string }) {
  const { t } = useTranslation();
  const [url, setUrl] = useState(initial ?? "");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");
  const [pr, setPr] = useState<Probe | null>(null);
  const [pick, setPick] = useState(0);
  const find = async () => {
    if (!/^https?:\/\/\S+$/i.test(url.trim())) return setErr("bad-url");
    setBusy(true);
    setErr("");
    setPr(null);
    try {
      setPr(await api.mediaProbe(url.trim()));
      setPick(0);
    } catch (e) {
      setErr(errText(e));
    }
    setBusy(false);
  };
  const start = async () => {
    if (!pr) return;
    try {
      await api.mediaStart(pr.token, pick);
      close();
      useStore.getState().notify("videoStarted", pr.title);
    } catch (e) {
      setErr(errText(e));
    }
  };
  // a copied video link (toast "video-link") opens here and looks it up; it never starts by itself
  const asked = useRef(false); // once, also under StrictMode's double effect
  useEffect(() => {
    if (initial && !asked.current) {
      asked.current = true;
      void find();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  const known = ["drm", "live", "no-formats", "need-component", "bad-url"];
  return (
    <Modal label={t("vid.title")} onClose={close} wide>
      <h2>{t("vid.title")}</h2>
      <div className="fld-row">
        <label className="fld">
          <span>{t("vid.page")}</span>
          <span className="in">
            <input className="mono" value={url} aria-invalid={!!err} onChange={(e) => (setUrl(e.target.value), setErr(""))} onKeyDown={(e) => e.key === "Enter" && void find()} />
          </span>
        </label>
        <button type="button" className="fx btn btn-s" onClick={() => void find()}>
          {t("vid.find")}
        </button>
      </div>
      {busy && <Spinner text={t("vid.busy")} />}
      {err && (
        <p className="err-t" role="alert">
          {known.includes(err) ? t(`vid.e.${err}`) : t("vid.err", { e: err })}
        </p>
      )}
      {err === "need-component" && <Components onDone={() => void find()} />}
      {pr && (
        <>
          <span className="lab clip1" title={pr.title}>
            {pr.title}
          </span>
          <div className="batch" role="radiogroup" aria-label={t("vid.quality")}>
            {pr.choices.map((c, i) => (
              <button key={i} type="button" className="fx check" role="radio" aria-checked={pick === i} onClick={() => setPick(i)}>
                <i>{pick === i && <Icon n="check" />}</i>
                <span className="clip1">
                  {c.label} · {c.container}
                </span>
                <span className="num">{c.est_size ? "≈ " + bytes(c.est_size) : t("vid.unknown")}</span>
              </button>
            ))}
          </div>
        </>
      )}
      <div className="mfoot">
        <span className="sp" />
        <button type="button" className="fx btn btn-g" onClick={close}>
          {t("info.cancel")}
        </button>
        <button type="button" className="fx btn btn-p fx-strong" aria-disabled={!pr} onClick={() => void start()}>
          {t("vid.go")}
        </button>
      </div>
    </Modal>
  );
}

/** "Shutting down in N s" with [Cancel]: reads the seconds left from queue_list each second while shown */
export function Countdown() {
  const { t } = useTranslation();
  const cd = useStore((s) => s.countdown);
  const q = useStore((s) => s.queues.find((v) => v.def.id === cd?.id));
  const btn = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (!cd) return;
    btn.current?.focus();
    const tm = setInterval(() => void useStore.getState().refreshQueues(), 1000);
    return () => clearInterval(tm);
  }, [cd]);
  useEffect(() => {
    if (cd && q && q.state.countdown == null) useStore.setState({ countdown: null });
  }, [cd, q]);
  if (!cd) return null;
  const n = q?.state.countdown ?? 60;
  const cancel = async () => {
    await api.queueCancelAction(cd.id);
    useStore.setState({ countdown: null });
  };
  return (
    <div className="cd" role="alertdialog" aria-label={t(`cd.${cd.action}`, { n, defaultValue: cd.action })}>
      <Icon n="warning" lg />
      <div className="tt">
        <b>{t(`cd.${cd.action}`, { n, defaultValue: cd.action })}</b>
        <span>{t("cd.why", { q: qName(t, q?.def) })}</span>
      </div>
      <button ref={btn} type="button" className="fx btn btn-p fx-strong" onClick={() => void cancel()}>
        {t("cd.cancel")}
      </button>
    </div>
  );
}
