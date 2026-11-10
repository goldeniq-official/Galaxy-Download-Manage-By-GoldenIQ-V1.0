// SPDX-License-Identifier: RPL-1.5
//! G7 F08 queues: N at once, retry to the end, finish actions with a recorded power call (Q-02).
use gdm_app::app::{lock, App};
use gdm_app::queue::Queue;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn app(tag: &str, q: Queue) -> (Arc<App>, Arc<Mutex<Vec<String>>>, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!("gdm-q-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let a = App::open(&d.join("data")).unwrap();
    {
        let mut s = lock(&a.settings);
        s.download_dir = d.join("dl").display().to_string();
        s.queues = vec![q, Queue { id: "sync".into(), sync: true, ..Default::default() }];
    }
    a.apply_limit();
    let rec = Arc::new(Mutex::new(vec![]));
    let r2 = rec.clone();
    *lock(&a.power) = Box::new(move |act, force| {
        r2.lock().unwrap().push(format!("{act}{}", if force { "+force" } else { "" }));
        Ok(())
    });
    a.countdown_ms.store(800, Ordering::Relaxed);
    (a, rec, d)
}

fn add(a: &Arc<App>, url: &str) -> String {
    let id = a.add_and_start(url, None, None, vec![], true, false).unwrap();
    a.queue_put(std::slice::from_ref(&id), Some("main")).unwrap();
    id
}

async fn ticks(a: &Arc<App>, secs: u64, mut until: impl FnMut(&Arc<App>) -> bool) -> bool {
    let t0 = std::time::Instant::now();
    let mut last = chrono::Utc::now();
    while t0.elapsed() < Duration::from_secs(secs) {
        let now = chrono::Utc::now();
        a.queue_tick(now, last).await;
        last = now;
        if until(a) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    false
}

#[tokio::test(flavor = "multi_thread")]
async fn q02_finish_countdown_then_power_and_main_queue_empties() {
    let fx = gdm_fixture::Fixture::start("127.0.0.1:0").await;
    let (a, rec, d) = app("done", Queue { id: "main".into(), max_active: 2, on_done: "sleep".into(), ..Default::default() });
    let ids: Vec<String> = (1..=4).map(|i| add(&a, &fx.url(&format!("/p/200000/{i}/400000/f{i}.bin")))).collect();
    a.queue_start("main", None).unwrap();
    let mut peak = 0;
    let done = ticks(&a, 60, |a| {
        let n = ids.iter().filter(|i| a.is_running(i)).count();
        peak = peak.max(n);
        !rec.lock().unwrap().is_empty()
    })
    .await;
    assert!(done, "power action never ran");
    assert!((1..=2).contains(&peak), "N at once = 2, saw {peak}");
    assert_eq!(*rec.lock().unwrap(), vec!["sleep".to_string()]);
    assert!(lock(&a.engine.store).queue_jobs("main").unwrap().is_empty(), "finished files leave the main queue");
    for i in &ids {
        assert_eq!(a.engine.job(i).unwrap().state, "complete");
    }
    let _ = std::fs::remove_dir_all(d);
}

#[tokio::test(flavor = "multi_thread")]
async fn q02_manual_stop_and_cancel_and_new_work_never_shut_down() {
    let fx = gdm_fixture::Fixture::start("127.0.0.1:0").await;
    let (a, rec, d) = app("stop", Queue { id: "main".into(), on_done: "shutdown".into(), ..Default::default() });
    // 1. stopped by hand => no action at all
    let slow = add(&a, &fx.url("/p/5000000/9/100000/slow.bin"));
    a.queue_start("main", None).unwrap();
    ticks(&a, 5, |a| a.is_running(&slow)).await;
    a.queue_stop("main").await;
    ticks(&a, 3, |_| false).await;
    assert!(rec.lock().unwrap().is_empty(), "manual stop => no shutdown");
    a.remove(&slow, false).await.unwrap();

    // 2. finished => countdown; [cancel] => nothing
    add(&a, &fx.url("/p/100000/3/0/a.bin"));
    a.queue_start("main", None).unwrap();
    assert!(ticks(&a, 30, |a| a.queue_states().get("main").is_some_and(|s| s.countdown.is_some())).await);
    a.queue_cancel_action("main");
    ticks(&a, 2, |_| false).await;
    assert!(rec.lock().unwrap().is_empty(), "cancelled countdown => no shutdown");

    // 3. finished => countdown; a download starts meanwhile => re-check refuses the action
    add(&a, &fx.url("/p/100000/4/0/b.bin"));
    a.queue_start("main", None).unwrap();
    assert!(ticks(&a, 30, |a| a.queue_states().get("main").is_some_and(|s| s.countdown.is_some())).await);
    let other = a.add_and_start(&fx.url("/p/5000000/5/100000/other.bin"), None, None, vec![], true, true).unwrap();
    ticks(&a, 3, |_| false).await;
    assert!(rec.lock().unwrap().is_empty(), "a running download => no shutdown");
    a.pause(&other).await;
    let _ = std::fs::remove_dir_all(d);
}

#[tokio::test(flavor = "multi_thread")]
async fn failed_file_retries_then_stops_and_queue_finishes() {
    let (a, rec, d) = app("retry", Queue { id: "main".into(), retries: 1, on_done: "hibernate".into(), ..Default::default() });
    // 418: a plain HTTP error the engine does not retry by itself (not a decision, login or 404 case)
    let fx = gdm_fixture::Fixture::start("127.0.0.1:0").await;
    let bad = add(&a, &fx.url("/status?code=418"));
    a.queue_start("main", None).unwrap();
    assert!(ticks(&a, 120, |_| !rec.lock().unwrap().is_empty()).await, "the queue must finish after the retries");
    let row = a.rows(None).unwrap().into_iter().find(|r| r.row.id == bad).unwrap();
    assert_eq!(row.row.state, "failed");
    assert!(row.row.retries >= 2, "retried once, then given up: {}", row.row.retries);
    let _ = std::fs::remove_dir_all(d);
}
