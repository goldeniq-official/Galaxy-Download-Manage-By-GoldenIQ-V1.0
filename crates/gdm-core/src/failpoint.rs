// SPDX-License-Identifier: RPL-1.5
//! Deterministic crash points for the kill tests: GDM_FAILPOINT=<name>:<n> aborts the process (no unwinding,
//! no destructors: like TerminateProcess) the n-th time `name` is hit. Unset = a relaxed atomic load.
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

fn target() -> &'static Option<(String, u64)> {
    static T: OnceLock<Option<(String, u64)>> = OnceLock::new();
    T.get_or_init(|| {
        let v = std::env::var("GDM_FAILPOINT").ok()?;
        let (k, n) = v.split_once(':')?;
        Some((k.to_string(), n.parse().ok()?))
    })
}

pub fn hit(name: &str) {
    let Some((k, n)) = target() else { return };
    if k != name {
        return;
    }
    static C: Mutex<Option<HashMap<String, u64>>> = Mutex::new(None);
    let mut g = C.lock().unwrap_or_else(|e| e.into_inner());
    let c = g.get_or_insert_with(HashMap::new).entry(name.to_string()).or_insert(0);
    *c += 1;
    if *c == *n {
        eprintln!("failpoint {name}:{n}");
        std::process::abort();
    }
}

/// GDM_TIMING=1: phase timings on stderr (benchmark breakdown; off by default)
pub fn phase(name: &str, since: std::time::Instant) {
    static ON: OnceLock<bool> = OnceLock::new();
    if *ON.get_or_init(|| std::env::var("GDM_TIMING").is_ok_and(|v| v == "1")) {
        eprintln!("phase {name} {:.3}", since.elapsed().as_secs_f64());
    }
}
