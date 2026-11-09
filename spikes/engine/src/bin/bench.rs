// SPDX-License-Identifier: RPL-1.5
//! Benchmark client (plan.md ង): one download with the S5 segmenter, then real finish = sync + sha256 +
//! IAttachmentExecute::Save + rename. Prints one CSV line:
//!   bytes,transfer_s,done_s,peak_conns,sha256
//! Usage: bench <url> <dir> <name> [fixed-conns]
use gdm_spikes::seg::{download, Config};
use std::path::Path;
use std::time::Instant;
use windows::core::{GUID, HSTRING};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::Shell::{AttachmentServices, IAttachmentExecute};

fn save(path: &Path, name: &str, src: &str) -> windows::core::Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        let ae: IAttachmentExecute = CoCreateInstance(&AttachmentServices, None, CLSCTX_INPROC_SERVER)?;
        ae.SetClientGuid(&GUID::from_u128(0x6a1c_2f0e_9b7d_4c3a_8e51_0d47_c2f9_a3b6))?;
        ae.SetLocalPath(&HSTRING::from(path.as_os_str()))?;
        ae.SetFileName(&HSTRING::from(name))?;
        ae.SetSource(&HSTRING::from(src))?;
        ae.Save()
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (url, dir, name) = (&a[1], Path::new(&a[2]), &a[3]);
    let mut cfg = Config { fixed: a.get(4).and_then(|v| v.parse().ok()), ..Config::default() };
    // experiment knob only (profiling): GDM_FLUSH_MB overrides the 8 MiB commit threshold
    if let Some(mb) = std::env::var("GDM_FLUSH_MB").ok().and_then(|v| v.parse::<u64>().ok()) { cfg.flush_bytes = mb << 20; cfg.flush_every = std::time::Duration::from_secs(3600); }
    if let Some(ms) = std::env::var("GDM_FLUSH_MS").ok().and_then(|v| v.parse::<u64>().ok()) { cfg.flush_every = std::time::Duration::from_millis(ms); }
    cfg.write_through = std::env::var("GDM_WRITE_THROUGH").is_ok_and(|v| v == "1");
    let part = dir.join(format!("{name}.part"));
    let t0 = Instant::now();
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let d = match rt.block_on(download(url, &part, cfg)) { Ok(d) => d, Err(e) => { eprintln!("download failed: {e:?}"); std::process::exit(2) } };
    let transfer = d.transfer.as_secs_f64(); // download() hashes through its handle after this point
    let sha = d.sha256.clone();
    if let Err(e) = save(&part, name, url) { eprintln!("attachment save: {e}"); }
    std::fs::rename(&part, dir.join(name)).unwrap();
    let _ = std::fs::remove_file(gdm_spikes::seg::sidecar_path(&part));
    println!("{},{:.3},{:.3},{},{}", d.total, transfer, t0.elapsed().as_secs_f64(), d.peak_conns, sha);
}
