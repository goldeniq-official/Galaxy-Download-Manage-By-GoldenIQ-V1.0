// SPDX-License-Identifier: RPL-1.5
//! Queues + scheduler + actions when a queue finishes (F08). Definitions live in settings.json; job membership,
//! order, retries and scheduled-occurrence claims live in SQLite (council G7: a claim and the start must not
//! be split across two stores). IDM model (idm.md §1.5): main download queue (finished files leave it), main
//! synchronisation queue (files stay and are re-checked, Q-05), unlimited user queues of either type.
use crate::app::{lock, App};
use chrono::{Local, Utc};
use gdm_core::sched::Schedule;
use gdm_core::Unverified;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Queue {
    /// "main", "sync" or "q-<n>" (user queues)
    pub id: String,
    /// user queues only; the two built-in queues are named by the UI in the user's language
    pub name: String,
    pub sync: bool,
    /// N files at the same time
    pub max_active: usize,
    /// retries per failed file (it moves to the end of the queue each time)
    pub retries: u32,
    pub schedule: Schedule,
    /// none | exit | sleep | hibernate | shutdown
    pub on_done: String,
    /// shutdown only: force processes to terminate (Q-02: only when chosen explicitly)
    pub force: bool,
    /// run the program set in Settings (hooks[id], Q-01) when the queue finishes
    pub run_hook: bool,
    /// queue speed limit, KB/s (0 = none)
    pub speed_kbps: u64,
    /// download quota: at most quota_mb every quota_hours (0 = none)
    pub quota_mb: u64,
    pub quota_hours: u32,
    /// wake the computer for the scheduled start (Task Scheduler, AC only, Q-04)
    pub wake: bool,
    /// sync queue on a metered connection: off by default (Q-05)
    pub sync_metered: bool,
}

impl Default for Queue {
    fn default() -> Self {
        Queue {
            id: String::new(),
            name: String::new(),
            sync: false,
            max_active: 1,
            retries: 10,
            schedule: Schedule::default(),
            on_done: "none".into(),
            force: false,
            run_hook: false,
            speed_kbps: 0,
            quota_mb: 0,
            quota_hours: 1,
            wake: false,
            sync_metered: false,
        }
    }
}

pub fn default_queues() -> Vec<Queue> {
    vec![Queue { id: "main".into(), ..Default::default() }, Queue { id: "sync".into(), sync: true, ..Default::default() }]
}

/// runtime state of one queue (never persisted: after a restart a queue is stopped unless its schedule starts it)
#[derive(Default, Debug, Clone, Serialize)]
pub struct QState {
    pub active: bool,
    /// the user stopped it: no action when done (IDM)
    pub manual_stop: bool,
    /// scheduled occurrence being run (Q-03 claim key)
    pub key: Option<String>,
    /// completion generation: a countdown belongs to one generation; any new start invalidates it (Q-02)
    pub gen: u64,
    /// seconds left of the 60 s countdown before the action
    pub countdown: Option<u64>,
    pub action: Option<String>,
    /// periodic sync: when the next round starts
    #[serde(skip)]
    pub next_round: Option<Instant>,
    #[serde(skip)]
    pub deadline: Option<Instant>,
    #[serde(skip)]
    pub last_file: Option<std::path::PathBuf>,
}

pub const COUNTDOWN: Duration = Duration::from_secs(60);

/// power actions go through this so tests record instead of shutting the PC down
pub type Power = Box<dyn Fn(&str, bool) -> Result<(), String> + Send + Sync>;

pub fn real_power() -> Power {
    Box::new(crate::os::power)
}

/// error codes the queue never retries by itself: they need the user (decision, login, disk, changed file)
fn needs_user(code: Option<&str>) -> bool {
    code.is_some_and(|c| {
        let c = c.split(':').next().unwrap_or("");
        matches!(
            c,
            "NeedsDecision"
                | "FileChanged"
                | "SizeChanged"
                | "SizeExceeded"
                | "DiskFull"
                | "Auth"
                | "Gone"
                | "Blocked"
                | "HashMismatch"
                | "Reparse"
                | "Html"
                | "Forbidden"
                | "Tls"
        )
    })
}

impl App {
    pub fn queue_defs(&self) -> Vec<Queue> {
        lock(&self.settings).queues.clone()
    }

    pub fn queue_states(&self) -> HashMap<String, QState> {
        lock(&self.qstate).clone()
    }

    /// add jobs to the end of a queue (None = take them out). A job that is running keeps running.
    pub fn queue_put(&self, ids: &[String], queue: Option<&str>) -> Result<(), String> {
        if let Some(q) = queue {
            if !self.queue_defs().iter().any(|d| d.id == q) {
                return Err("no-queue".into());
            }
        }
        lock(&self.engine.store).set_queue(ids, queue)?;
        self.bump();
        Ok(())
    }

    /// Start Queue (by hand or by the schedule). A sync queue first re-checks its finished files (Q-05).
    pub fn queue_start(self: &Arc<Self>, id: &str, key: Option<String>) -> Result<(), String> {
        let def = self.queue_defs().into_iter().find(|d| d.id == id).ok_or("no-queue")?;
        {
            let mut qs = lock(&self.qstate);
            let s = qs.entry(id.to_string()).or_default();
            s.active = true;
            s.manual_stop = false;
            s.key = key;
            s.gen += 1;
            s.countdown = None;
            s.deadline = None;
            s.action = None;
            s.next_round = None;
        }
        if def.sync {
            if !def.sync_metered && crate::os::metered() {
                self.push_toast("info", "", "sync-metered");
            } else {
                let me = self.clone();
                let qid = id.to_string();
                tauri::async_runtime::spawn(async move { me.sync_round(&qid).await });
            }
        }
        self.bump();
        Ok(())
    }

    async fn sync_round(self: &Arc<Self>, qid: &str) {
        let ids = lock(&self.engine.store).queue_jobs(qid).unwrap_or_default();
        for id in ids {
            let _ = self.engine.sync_check(&id).await; // an error leaves the previous file as it is
        }
        self.bump();
    }

    /// Stop Queue by hand: its downloads pause, no action runs (IDM: actions do not run after a manual stop)
    pub async fn queue_stop(self: &Arc<Self>, id: &str) {
        {
            let mut qs = lock(&self.qstate);
            let s = qs.entry(id.to_string()).or_default();
            s.active = false;
            s.manual_stop = true;
            s.gen += 1;
            s.countdown = None;
            s.deadline = None;
            s.action = None;
            s.next_round = None;
        }
        let ids = lock(&self.engine.store).queue_jobs(id).unwrap_or_default();
        for j in ids {
            if self.is_running(&j) {
                self.pause(&j).await;
            }
        }
        self.bump();
    }

    /// cancel a countdown (the toast/window button)
    pub fn queue_cancel_action(&self, id: &str) {
        let mut qs = lock(&self.qstate);
        if let Some(s) = qs.get_mut(id) {
            s.gen += 1;
            s.countdown = None;
            s.deadline = None;
            s.action = None;
        }
        self.bump();
    }

    /// one scheduler step (every second). `now` is wall time for schedules, Instant for countdowns.
    pub async fn queue_tick(self: &Arc<Self>, now: chrono::DateTime<Utc>, last: chrono::DateTime<Utc>) {
        for def in self.queue_defs() {
            self.tick_one(&def, now, last).await;
        }
    }

    async fn tick_one(self: &Arc<Self>, def: &Queue, now: chrono::DateTime<Utc>, last: chrono::DateTime<Utc>) {
        let st = lock(&self.qstate).get(&def.id).cloned().unwrap_or_default();
        // 1. scheduled start: an occurrence in (last, now], claimed once in SQLite; after sleep or app downtime it
        //    runs only while still inside its start..stop window (IDM skips a missed start)
        if !st.active {
            let claimed = lock(&self.engine.store).last_claim(&def.id).ok().flatten();
            if let Some((at, key)) = def.schedule.next_start(&Local, last, claimed.as_deref()) {
                if at <= now && def.schedule.within(&Local, &key, now) && lock(&self.engine.store).claim(&def.id, &key).unwrap_or(false) {
                    let _ = self.queue_start(&def.id, Some(key));
                    return;
                }
            }
        }
        if let Some(dl) = st.deadline {
            self.countdown_step(def, &st, dl).await;
            return;
        }
        if !st.active {
            return;
        }
        // 2. stop time reached: stop (not a manual stop, but not "finished" either: no action)
        if let Some(k) = &st.key {
            if !def.schedule.within(&Local, k, now) {
                self.queue_stop(&def.id).await;
                lock(&self.qstate).entry(def.id.clone()).or_default().manual_stop = false;
                return;
            }
        }
        if let Some(t) = st.next_round {
            if Instant::now() >= t {
                let _ = self.queue_start(&def.id, st.key.clone());
            }
            return;
        }
        // 3. keep N files running, in queue order
        let rows: HashMap<String, gdm_core::store::Row> =
            self.rows(None).unwrap_or_default().into_iter().map(|r| (r.row.id.clone(), r.row)).collect();
        let order = lock(&self.engine.store).queue_jobs(&def.id).unwrap_or_default();
        let mut running = order.iter().filter(|j| self.is_running(j)).count();
        let mut waiting = 0;
        for j in &order {
            let Some(r) = rows.get(j) else { continue };
            if self.is_running(j) {
                continue;
            }
            let startable = match r.state.as_str() {
                "queued" => true,
                "paused" => !needs_user(r.error_code.as_deref()),
                "failed" => r.retries <= def.retries && !needs_user(r.error_code.as_deref()),
                _ => false,
            };
            if !startable {
                continue;
            }
            waiting += 1;
            if running < def.max_active.max(1) && !self.finishing.load(Ordering::SeqCst) {
                self.start(j, Unverified::Ask);
                running += 1;
            }
        }
        // 4. finished: nothing running, nothing left to start
        if running == 0 && waiting == 0 {
            if def.sync && def.schedule.every_min.is_some_and(|m| m > 0) {
                let m = def.schedule.every_min.unwrap_or(60) as u64;
                lock(&self.qstate).entry(def.id.clone()).or_default().next_round = Some(Instant::now() + Duration::from_secs(m * 60));
                return;
            }
            self.queue_finished(def).await;
        }
    }

    async fn queue_finished(self: &Arc<Self>, def: &Queue) {
        let last_file = {
            let mut qs = lock(&self.qstate);
            let s = qs.entry(def.id.clone()).or_default();
            s.active = false;
            s.gen += 1;
            s.last_file.clone()
        };
        self.push_toast("queue-done", &def.id, &def.name);
        self.play("queue");
        if def.run_hook {
            let hook = lock(&self.settings).hooks.get(&def.id).cloned();
            if let Some(h) = hook {
                let file = last_file.unwrap_or_else(|| std::path::PathBuf::from(&lock(&self.settings).download_dir));
                if let Err(e) = crate::hook::run(&h, &file) {
                    self.push_toast("failed", &def.id, &format!("hook: {e}"));
                }
            }
        }
        match def.on_done.as_str() {
            "exit" | "sleep" | "hibernate" | "shutdown" => {
                let mut qs = lock(&self.qstate);
                let s = qs.entry(def.id.clone()).or_default();
                let cd = Duration::from_millis(self.countdown_ms.load(Ordering::Relaxed));
                s.deadline = Some(Instant::now() + cd);
                s.countdown = Some(cd.as_secs());
                s.action = Some(def.on_done.clone());
                drop(qs);
                self.push_toast("countdown", &def.id, &def.on_done);
                crate::show_main(self, false);
            }
            _ => {}
        }
        self.bump();
    }

    async fn countdown_step(self: &Arc<Self>, def: &Queue, st: &QState, deadline: Instant) {
        let now = Instant::now();
        if now < deadline {
            let left = (deadline - now).as_secs() + 1;
            if let Some(s) = lock(&self.qstate).get_mut(&def.id) {
                s.countdown = Some(left);
            }
            return;
        }
        // Q-02: final re-check, serialised with starts (`finishing` blocks new starts until the call returns):
        // same generation (nobody started or cancelled meanwhile), queue not active, no download running anywhere
        self.finishing.store(true, Ordering::SeqCst);
        let gen_now = lock(&self.qstate).get(&def.id).map(|s| s.gen);
        let ok = gen_now == Some(st.gen) && !st.active && !st.manual_stop && self.running() == 0;
        if let Some(s) = lock(&self.qstate).get_mut(&def.id) {
            s.deadline = None;
            s.countdown = None;
            s.action = None;
        }
        if !ok {
            self.finishing.store(false, Ordering::SeqCst);
            self.push_toast("info", &def.id, "action-skipped");
            return;
        }
        let action = st.action.clone().unwrap_or_default();
        let r = if action == "exit" {
            self.finishing.store(false, Ordering::SeqCst);
            crate::request_quit();
            Ok(())
        } else {
            let r = (lock(&self.power))(&action, def.force && action == "shutdown");
            self.finishing.store(false, Ordering::SeqCst);
            r
        };
        if let Err(e) = r {
            self.push_toast("failed", &def.id, &format!("{action}: {e}")); // a refused power action is reported
        }
        self.bump();
    }
}
