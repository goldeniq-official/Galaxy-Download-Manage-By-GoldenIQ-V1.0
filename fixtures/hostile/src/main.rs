// SPDX-License-Identifier: RPL-1.5
//! `gdm-fixture [port]`: run the hostile fixture for manual use and the IDM benchmark.
//! GDM_FIXTURE_TRACE=1 prints every request with its Range header.
use gdm_fixture::Fixture;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let port = std::env::args().nth(1).unwrap_or_else(|| "8790".into());
    let fx = Fixture::start(&format!("127.0.0.1:{port}")).await;
    println!("fixture on http://{} (e.g. /file?size=1073741824&seed=7)", fx.addr);
    let trace = std::env::var("GDM_FIXTURE_TRACE").is_ok();
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        let log = fx.log();
        if trace {
            for r in &log { println!("conn {} {} range={:?}", r.conn, r.target, r.headers.iter().find(|(k, _)| k == "range").map(|(_, v)| v)); }
            fx.clear_log();
        } else if !log.is_empty() {
            println!("requests={} distinct_conns={} peak_active={}", log.len(), log.iter().map(|r| r.conn).collect::<std::collections::HashSet<_>>().len(), fx.peak_conns());
        }
    }
}
