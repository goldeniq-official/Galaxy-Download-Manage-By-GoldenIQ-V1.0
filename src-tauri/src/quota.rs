// SPDX-License-Identifier: RPL-1.5
//! Download quota (F11, IDM "Download limits": at most X MB every Y hours, total and per queue). Like IDM,
//! downloads pause when the cap is reached and resume by themselves when the period ends (a connection is
//! never parked for an hour: the server would drop it and the retries would be spent).
use crate::app::{lock, App};
use gdm_core::Unverified;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Default, Debug)]
pub struct QuotaState {
    /// limit key ("" = total, else queue id) -> (period start, bytes used in it)
    used: HashMap<String, (Instant, u64)>,
    /// bytes each running job had written at the last tick
    seen: HashMap<String, u64>,
    /// jobs paused by a quota: key -> ids to resume when its period ends
    held: HashMap<String, Vec<String>>,
    /// tests: a period of seconds instead of hours
    pub test_period: Option<Duration>,
}

impl App {
    /// limits now in force: key -> (bytes, period)
    fn quota_limits(&self) -> Vec<(String, u64, Duration)> {
        let tp = lock(&self.quota).test_period;
        let s = lock(&self.settings);
        let mut v = vec![];
        if s.quota_mb > 0 {
            v.push((String::new(), s.quota_mb * 1024 * 1024, Duration::from_secs(s.quota_hours.max(1) as u64 * 3600)));
        }
        for q in &s.queues {
            if q.quota_mb > 0 {
                v.push((q.id.clone(), q.quota_mb * 1024 * 1024, Duration::from_secs(q.quota_hours.max(1) as u64 * 3600)));
            }
        }
        if let Some(p) = tp {
            for l in &mut v {
                l.2 = p;
            }
        }
        v
    }

    /// is this limit exhausted right now (the queue scheduler does not start its files)
    pub fn quota_blocked(&self, key: &str) -> bool {
        lock(&self.quota).held.contains_key(key) || (!key.is_empty() && lock(&self.quota).held.contains_key(""))
    }

    /// once a second: count new bytes, pause what went over, resume what a new period freed
    pub async fn quota_tick(self: &Arc<Self>) {
        let limits = self.quota_limits();
        let now = Instant::now();
        let live = self.live();
        let mut to_pause = vec![];
        let mut to_resume = vec![];
        {
            let mut q = lock(&self.quota);
            // bytes since the last tick, per job
            let mut delta: Vec<(String, u64)> = vec![];
            for l in &live {
                // a resumed run reports 0 and then jumps to its resume offset: those bytes are old, so the
                // baseline is the committed progress in the database
                let base = match q.seen.get(&l.id) {
                    Some(b) => *b,
                    None => self.engine.job(&l.id).map(|j| j.done).unwrap_or(0).max(l.done),
                };
                q.seen.insert(l.id.clone(), base.max(l.done));
                delta.push((l.id.clone(), l.done.saturating_sub(base)));
            }
            q.seen.retain(|id, _| live.iter().any(|l| &l.id == id));
            for (key, cap, period) in &limits {
                let e = q.used.entry(key.clone()).or_insert((now, 0));
                if now.duration_since(e.0) >= *period {
                    *e = (now, 0); // a new period
                    if let Some(ids) = q.held.remove(key) {
                        to_resume.extend(ids);
                    }
                }
                let mut add = 0;
                for (id, d) in &delta {
                    let in_scope = key.is_empty() || lock(&self.engine.store).queue_of(id).as_deref() == Some(key.as_str());
                    if in_scope {
                        add += d;
                    }
                }
                let e = q.used.get_mut(key).expect("inserted above");
                e.1 += add;
                if e.1 >= *cap && !q.held.contains_key(key) {
                    let ids: Vec<String> = live
                        .iter()
                        .map(|l| l.id.clone())
                        .filter(|id| key.is_empty() || lock(&self.engine.store).queue_of(id).as_deref() == Some(key.as_str()))
                        .collect();
                    q.held.insert(key.clone(), ids.clone());
                    to_pause.extend(ids);
                }
            }
            // a limit that was switched off releases its files
            let keys: Vec<String> = q.held.keys().cloned().collect();
            for k in keys {
                if !limits.iter().any(|(key, _, _)| key == &k) {
                    to_resume.extend(q.held.remove(&k).unwrap_or_default());
                }
            }
        }
        if !to_pause.is_empty() {
            for id in &to_pause {
                self.pause(id).await;
            }
            self.push_toast("info", "", "quota-reached");
        }
        for id in to_resume {
            if !self.quota_blocked("") {
                self.start(&id, Unverified::Ask);
            }
        }
    }
}
