// SPDX-License-Identifier: RPL-1.5
//! Token buckets in three levels (§09, L-01): global -> queue -> job. A connection takes from its job bucket,
//! then the queue's, then the global one (child before parent, so no parent capacity is held while waiting on
//! a child). Each bucket is a FIFO async mutex held while sleeping off the debt, so every waiter is served in
//! arrival order and connections share a level evenly. Rates change live; 0 = unlimited.
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;

#[derive(Debug)]
pub struct Bucket {
    rate: AtomicU64,
    state: Mutex<(f64, Instant)>,
}

impl Bucket {
    pub fn new(bytes_per_s: u64) -> Arc<Bucket> {
        Arc::new(Bucket { rate: AtomicU64::new(bytes_per_s), state: Mutex::new((0.0, Instant::now())) })
    }
    pub fn set_rate(&self, bytes_per_s: u64) {
        self.rate.store(bytes_per_s, Ordering::Relaxed);
    }
    pub fn rate(&self) -> u64 {
        self.rate.load(Ordering::Relaxed)
    }

    pub async fn take(&self, n: u64) {
        let r = self.rate();
        if r == 0 {
            return;
        }
        let mut g = self.state.lock().await; // FIFO
        let now = Instant::now();
        let burst = (r as f64 / 4.0).max(64.0 * 1024.0); // at most 250 ms of credit
        g.0 = (g.0 + now.duration_since(g.1).as_secs_f64() * r as f64).min(burst);
        g.1 = now;
        g.0 -= n as f64;
        if g.0 < 0.0 {
            // holding the lock: the next waiter starts after this debt is paid. Accounting continues from the
            // planned instant, not from the wake-up: Windows timers overshoot by up to ~15.6 ms per sleep and
            // that time is credited to the next taker (bounded by `burst`).
            let due = now + Duration::from_secs_f64(-g.0 / r as f64);
            tokio::time::sleep_until(due).await;
            g.0 = 0.0;
            g.1 = due;
        }
    }
}

/// the chain one job's connections draw from
#[derive(Clone, Debug)]
pub struct JobLimit {
    pub job: Arc<Bucket>,
    pub queue: Option<Arc<Bucket>>,
    pub global: Arc<Bucket>,
}

impl JobLimit {
    pub async fn take(&self, n: u64) {
        self.job.take(n).await;
        if let Some(q) = &self.queue {
            q.take(n).await;
        }
        self.global.take(n).await;
    }
}
