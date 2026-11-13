// SPDX-License-Identifier: RPL-1.5
// Settings > Queues and scheduler (IDM Scheduler: tabs "Schedule" and "Files in the queue", F08).
// Definitions are saved with settings_set({queues}); the program to run comes only from hook_set's native picker (Q-01).
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, errText, fileName, type Queue, type Schedule } from "./api";
import { useStore } from "./store";
import { ErrLine, Icon, Tabs, Toggle } from "./ui";
import { qName } from "./Dialogs";
import { NumIn, Opt, Sec } from "./Settings";

const HHMM = /^([01]\d|2[0-3]):[0-5]\d$/;
const DATE = /^\d{4}-\d{2}-\d{2}$/;

function TextIn({ label, value, ph, re, onSave }: { label: string; value: string | null; ph: string; re: RegExp; onSave: (v: string | null) => void }) {
  const [bad, setBad] = useState(false);
  return (
    <label className="fld">
      <span>{label}</span>
      <span className="in">
        <input
          className="mono"
          key={value ?? ""}
          defaultValue={value ?? ""}
          placeholder={ph}
          maxLength={10}
          aria-invalid={bad}
          onBlur={(e) => {
            const v = e.target.value.trim();
            const ok = !v || re.test(v);
            setBad(!ok);
            if (ok && (v || null) !== value) onSave(v || null);
          }}
        />
      </span>
    </label>
  );
}

export function QueuesSection() {
  const { t } = useTranslation();
  const s = useStore((x) => x.settings)!;
  const save = useStore((x) => x.saveSettings);
  const views = useStore((x) => x.queues);
  const jobs = useStore((x) => x.jobs);
  const [qid, setQid] = useState("main");
  const [tab, setTab] = useState<"sched" | "files">("sched");
  const [adding, setAdding] = useState(false);
  const [newName, setNewName] = useState("");
  const [newSync, setNewSync] = useState(false);
  const [hookErr, setHookErr] = useState("");
  const argsRef = useRef<HTMLTextAreaElement>(null);
  const q = s.queues.find((x) => x.id === qid) ?? s.queues[0];
  const v = views.find((x) => x.def.id === q.id);
  const user = q.id !== "main" && q.id !== "sync";
  const refresh = () => void useStore.getState().refreshQueues();
  const put = async (p: Partial<Queue>) => {
    await save({ queues: useStore.getState().settings!.queues.map((x) => (x.id === q.id ? { ...x, ...p } : x)) });
    refresh();
  };
  const sch = (p: Partial<Schedule>) => void put({ schedule: { ...q.schedule, ...p } });
  const create = async () => {
    const n = Math.max(0, ...s.queues.map((x) => Number(x.id.slice(2)) || 0)) + 1;
    const id = `q-${n}`;
    const def: Queue = {
      id, name: newName.trim() || t("qs.new") + " " + n, sync: newSync, max_active: 1, retries: 10,
      schedule: { enabled: false, start: null, stop: null, days: 0, date: null, every_min: null },
      on_done: "none", force: false, run_hook: false, speed_kbps: 0, quota_mb: 0, quota_hours: 1, wake: false, sync_metered: false,
    };
    await save({ queues: [...s.queues, def] });
    refresh();
    setQid(id);
    setAdding(false);
    setNewName("");
  };
  const remove = async () => {
    if (v?.jobs.length) await api.queuePut(v.jobs, null); // the files stay in the list, outside any queue
    await save({ queues: s.queues.filter((x) => x.id !== q.id) });
    refresh();
    setQid("main");
  };
  const hook = async (pick: boolean, args: string[], clear = false) => {
    try {
      await api.hookSet(q.id, pick, args, clear);
      setHookErr("");
      refresh();
    } catch (e) {
      setHookErr(errText(e));
    }
  };
  // reorder = take the files out, then put them back in the new order (queue_put appends in order)
  const move = async (i: number, d: number) => {
    const ids = [...(v?.jobs ?? [])];
    const j = i + d;
    if (j < 0 || j >= ids.length) return;
    [ids[i], ids[j]] = [ids[j], ids[i]];
    await api.queuePut(ids, null);
    await api.queuePut(ids, q.id);
    refresh();
  };
  const files = v?.jobs ?? [];
  const days = [0, 1, 2, 3, 4, 5, 6];
  return (
    <Sec id="queues" title={t("set.queues")}>
      <div className="opt">
        <label className="fld qpick">
          <span>{t("qs.queue")}</span>
          <select className="fx sel" value={q.id} onChange={(e) => setQid(e.target.value)}>
            {s.queues.map((x) => (
              <option key={x.id} value={x.id}>
                {qName(t, x)}
              </option>
            ))}
          </select>
        </label>
        <span className={"qstate" + (v?.state.active ? " on" : "")}>{v?.state.active ? t("q.running") : t("q.stopped")}</span>
        <button type="button" className="fx btn btn-s" onClick={() => void (v?.state.active ? api.queueStop(q.id) : api.queueStart(q.id)).then(refresh)}>
          <Icon n={v?.state.active ? "square" : "play"} />
          {v?.state.active ? t("q.stop") : t("q.start")}
        </button>
        <button type="button" className="fx btn btn-s" aria-expanded={adding} onClick={() => setAdding(!adding)}>
          <Icon n="plus" />
          {t("qs.new")}
        </button>
        {user && (
          <button type="button" className="fx btn btn-g danger" onClick={() => void remove()}>
            <Icon n="trash" />
            {t("qs.delete")}
          </button>
        )}
      </div>
      {adding && (
        <div className="opt col-opt">
          <label className="fld wide-fld">
            <span>{t("qs.name")}</span>
            <span className="in">
              <input value={newName} maxLength={64} onChange={(e) => setNewName(e.target.value)} />
            </span>
          </label>
          <Opt title={t("qs.type")} sub={t("qs.typeS")}>
            <Tabs value={newSync ? "sync" : "dl"} label={t("qs.type")} items={[["dl", t("qs.tDownload")], ["sync", t("qs.tSync")]]} onChange={(x) => setNewSync(x === "sync")} />
          </Opt>
          <div className="row-btns flush">
            <button type="button" className="fx btn btn-s" onClick={() => void create()}>
              {t("qs.create")}
            </button>
          </div>
        </div>
      )}
      <div className="opt">
        <Tabs value={tab} label={t("set.queues")} items={[["sched", t("qs.tabSched")], ["files", t("qs.tabFiles", { n: files.length })]]} onChange={setTab} />
      </div>
      {tab === "files" ? (
        <div className="opt col-opt">
          {files.length === 0 && <p className="meta">{t("qs.empty")}</p>}
          {files.map((id, i) => {
            const name = jobs.find((j) => j.id === id)?.name ?? id;
            return (
              <div className="hl-row" key={id}>
                <span className="num">{i + 1}</span>
                <span className="clip1" title={name}>
                  {name}
                </span>
                <button type="button" className="fx mini up" aria-label={t("qs.up") + " · " + name} title={t("qs.up")} disabled={i === 0} onClick={() => void move(i, -1)}>
                  <Icon n="caret-down" />
                </button>
                <button type="button" className="fx mini" aria-label={t("qs.down") + " · " + name} title={t("qs.down")} disabled={i === files.length - 1} onClick={() => void move(i, 1)}>
                  <Icon n="caret-down" />
                </button>
                <button type="button" className="fx mini" aria-label={t("q.removeFrom") + " · " + name} title={t("q.removeFrom")} onClick={() => void api.queuePut([id], null).then(refresh)}>
                  <Icon n="x" />
                </button>
              </div>
            );
          })}
        </div>
      ) : (
        <>
          {user && (
            <div className="opt col-opt">
              <label className="fld wide-fld">
                <span>{t("qs.name")}</span>
                <span className="in">
                  <input key={q.id} defaultValue={q.name} maxLength={64} onBlur={(e) => e.target.value.trim() && e.target.value.trim() !== q.name && void put({ name: e.target.value.trim() })} />
                </span>
              </label>
              <Opt title={t("qs.type")} sub={t("qs.typeS")}>
                <Tabs value={q.sync ? "sync" : "dl"} label={t("qs.type")} items={[["dl", t("qs.tDownload")], ["sync", t("qs.tSync")]]} onChange={(x) => void put({ sync: x === "sync" })} />
              </Opt>
            </div>
          )}
          <Opt title={t("qs.sched")} sub={t("qs.schedS")}>
            <Toggle on={q.schedule.enabled} label={t("qs.sched")} onChange={(x) => sch({ enabled: x })} />
          </Opt>
          {q.schedule.enabled && (
            <div className="opt col-opt">
              <div className="two even">
                <TextIn label={t("qs.start")} value={q.schedule.start} ph="23:00" re={HHMM} onSave={(x) => sch({ start: x })} />
                <TextIn label={t("qs.stop")} value={q.schedule.stop} ph="06:00" re={HHMM} onSave={(x) => sch({ stop: x })} />
              </div>
              <div className="fld" role="group" aria-label={t("qs.days")}>
                <span>{t("qs.days")}</span>
                <div className="days">
                  {days.map((d) => {
                    const on = (q.schedule.days & (1 << d)) !== 0;
                    return (
                      <button key={d} type="button" className="fx check" role="checkbox" aria-checked={on} onClick={() => sch({ days: q.schedule.days ^ (1 << d) })}>
                        <i>{on && <Icon n="check" />}</i>
                        <span>{t(`day.${d}`)}</span>
                      </button>
                    );
                  })}
                </div>
              </div>
              {q.schedule.days === 0 && <TextIn label={t("qs.date")} value={q.schedule.date} ph="2026-12-31" re={DATE} onSave={(x) => sch({ date: x })} />}
              {q.sync && (
                <label className="fld">
                  <span>{t("qs.every")}</span>
                  <NumIn label={t("qs.every")} value={q.schedule.every_min ?? 0} lo={0} hi={10_080} onSave={(x) => sch({ every_min: x || null })} />
                </label>
              )}
            </div>
          )}
          <Opt title={t("qs.maxActive")}>
            <NumIn label={t("qs.maxActive")} value={q.max_active} lo={1} hi={32} onSave={(x) => void put({ max_active: x })} />
          </Opt>
          <Opt title={t("qs.retries")} sub={t("qs.retriesS")}>
            <NumIn label={t("qs.retries")} value={q.retries} lo={0} hi={1000} onSave={(x) => void put({ retries: x })} />
          </Opt>
          <Opt title={t("qs.speed")}>
            <NumIn label={t("qs.speed")} value={q.speed_kbps} lo={0} hi={10_000_000} unit={t("set.kbps")} onSave={(x) => void put({ speed_kbps: x })} />
          </Opt>
          <Opt title={t("qs.quota")} sub={t("set.quotaS")}>
            <NumIn label={t("qs.quota") + " MB"} value={q.quota_mb} lo={0} hi={100_000_000} unit="MB" onSave={(x) => void put({ quota_mb: x })} />
            <NumIn label={t("qs.quota") + " · " + t("set.hours")} value={q.quota_hours} lo={1} hi={720} unit={t("set.hours")} onSave={(x) => void put({ quota_hours: x })} />
          </Opt>
          <Opt title={t("qs.onDone")} sub={t("qs.onDoneS")}>
            <select className="fx sel od" aria-label={t("qs.onDone")} value={q.on_done} onChange={(e) => void put({ on_done: e.target.value as Queue["on_done"], force: e.target.value === "shutdown" && q.force })}>
              {(["none", "exit", "sleep", "hibernate", "shutdown"] as const).map((x) => (
                <option key={x} value={x}>
                  {t(`od.${x}`)}
                </option>
              ))}
            </select>
          </Opt>
          {q.on_done === "shutdown" && (
            <Opt title={t("qs.force")} sub={t("qs.forceS")}>
              <Toggle on={q.force} label={t("qs.force")} onChange={(x) => void put({ force: x })} />
            </Opt>
          )}
          <Opt title={t("qs.wake")} sub={t("qs.wakeS")}>
            <Toggle on={q.wake} label={t("qs.wake")} onChange={(x) => void put({ wake: x })} />
          </Opt>
          {q.sync && (
            <Opt title={t("qs.metered")}>
              <Toggle on={q.sync_metered} label={t("qs.metered")} onChange={(x) => void put({ sync_metered: x })} />
            </Opt>
          )}
          <Opt title={t("qs.hook")} sub={v?.hook ? fileName(v.hook.program) : t("qs.hookNone")}>
            <Toggle on={q.run_hook} label={t("qs.hook")} onChange={(x) => void put({ run_hook: x })} />
          </Opt>
          <div className="opt col-opt">
            <label className="fld wide-fld">
              <span>{t("qs.args")}</span>
              <span className="in">
                <textarea
                  ref={argsRef}
                  rows={2}
                  key={q.id + (v?.hook?.args.join("\n") ?? "")}
                  className="mono"
                  defaultValue={v?.hook?.args.join("\n") ?? ""}
                  onBlur={(e) => v?.hook && e.target.value !== v.hook.args.join("\n") && void hook(false, argList(e.target.value))}
                />
              </span>
            </label>
            <p className="meta">{t("qs.argsS")}</p>
            <ErrLine msg={hookErr && t(`qs.e.${hookErr}`, { defaultValue: hookErr })} />
            <div className="row-btns flush">
              <button type="button" className="fx btn btn-s" onClick={() => void hook(true, argList(argsRef.current?.value ?? ""))}>
                <Icon n="folder-open" />
                {t("qs.choose")}
              </button>
              {v?.hook && (
                <button type="button" className="fx btn btn-g" onClick={() => void hook(false, [], true)}>
                  {t("qs.hookRemove")}
                </button>
              )}
            </div>
          </div>
        </>
      )}
    </Sec>
  );
}

const argList = (s: string) => s.split(/\r?\n/).map((x) => x.trim()).filter(Boolean);
