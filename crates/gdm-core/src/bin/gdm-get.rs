// SPDX-License-Identifier: RPL-1.5
//! `gdm-get <db> <url> <dir> [--fixed N] [--new] [--name NAME]`: download one URL with the engine (resumes the unfinished job
//! for the same URL in that database). Test / benchmark tool, not shipped. Prints one line:
//!   OK <sha256> <bytes> <transfer_s> <done_s> <peak_conns> <flushes> <flush_s> <path>      exit 0
//!   FAIL <code> <detail>                                                                  exit 3
use gdm_core::{seg, Engine, NewJob, Unverified};
use std::time::Instant;

#[tokio::main]
async fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 4 {
        eprintln!("usage: gdm-get <db> <url> <dir> [--fixed N] [--new] [--name NAME]");
        std::process::exit(2);
    }
    let t0 = Instant::now();
    let arg = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned();
    let fixed = arg("--fixed").and_then(|v| v.parse().ok());
    let e = match Engine::open(std::path::Path::new(&a[1]), seg::Config { fixed, ..seg::Config::default() }) {
        Ok(e) => e,
        Err(f) => {
            println!("FAIL {} {f}", f.code());
            std::process::exit(3)
        }
    };
    let existing = if a.iter().any(|x| x == "--new") {
        None
    } else {
        let st = e.store.lock().unwrap();
        st.job_ids().unwrap().into_iter().find(|id| {
            st.job(id).ok().flatten().is_some_and(|j| {
                j.url == url::Url::parse(&a[2]).map(|u| u.to_string()).unwrap_or_default() && j.state != "complete" && j.state != "unprotected"
            })
        })
    };
    let id = match existing {
        Some(id) => id,
        None => match e.add(NewJob { url: a[2].clone(), dir: a[3].clone().into(), typed: true, name: arg("--name"), ..Default::default() }) {
            Ok(id) => id,
            Err(f) => {
                println!("FAIL {} {f}", f.code());
                std::process::exit(3)
            }
        },
    };
    match e.run(&id, Unverified::Continue, None).await {
        Ok(o) => {
            let t = &o.transfer;
            println!(
                "OK {} {} {:.3} {:.3} {} {} {:.3} {}",
                o.sha256,
                t.total,
                t.transfer.as_secs_f64(),
                t0.elapsed().as_secs_f64(),
                t.peak_conns,
                t.flushes,
                t.flush_time.as_secs_f64(),
                o.path.display()
            );
        }
        Err(f) => {
            println!("FAIL {} {f}", f.code());
            std::process::exit(3)
        }
    }
}
