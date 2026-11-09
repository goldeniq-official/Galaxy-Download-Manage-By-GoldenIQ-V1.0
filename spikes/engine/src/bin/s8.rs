// SPDX-License-Identifier: RPL-1.5
//! S8 check: staged update -> verify sha256 (manifest) + minisign (artifact) through one held handle
//! (share READ only) -> refuse downgrade/equal (max version ever seen) -> install into versions/<v> ->
//! switch the `current` pointer last (atomic rename). Kill at any point => the launcher starts the old or the
//! fully verified new version, never a partial one (U-04). Path tricks after verify do not change what is
//! installed (U-06). This is plan B of plan.md (A/B folders); real NSIS is not a transaction.
//!   s8                                  run the check
//!   s8 install <root> <pkg> <sha256> <pubkey-b64>   one install attempt (GDM_FAILPOINT=step:N, GDM_SLOW_MS)
//!   s8 launch <root>                    print the version that starts
use gdm_spikes::Check;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::windows::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const SHARE_READ: u32 = 1;
use base64::Engine;
fn b64() -> base64::engine::GeneralPurpose { base64::engine::general_purpose::STANDARD }
fn sha(b: &[u8]) -> String { gdm_spikes::fixture::hex(&Sha256::digest(b)) }

fn ver(v: &str) -> Option<(u32, u32, u32)> {
    let mut p = v.split('.').map(|x| x.parse::<u32>().ok());
    let r = (p.next()??, p.next()??, p.next()??);
    p.next().is_none().then_some(r)
}

fn step(name: &str) {
    if let Ok(ms) = std::env::var("GDM_SLOW_MS") { std::thread::sleep(Duration::from_millis(ms.parse().unwrap_or(0))); }
    if std::env::var("GDM_FAILPOINT").ok().as_deref() == Some(name) { eprintln!("failpoint {name}"); std::process::abort(); }
}

/// write tmp + flush + rename(replace): readers see old or new, never half
fn atomic_write(p: &Path, data: &[u8]) -> std::io::Result<()> {
    let tmp = p.with_extension("tmp");
    let mut f = File::create(&tmp)?;
    f.write_all(data)?;
    f.sync_all()?;
    drop(f);
    fs::rename(&tmp, p)
}

#[derive(Debug, PartialEq)]
enum Refuse { Sha, Signature, Downgrade(String), Format(String) }

/// verify-to-install: one handle, share READ only, from hashing to extraction
fn install(root: &Path, pkg: &Path, want_sha: &str, pubkey: &str, after_verify: impl FnOnce()) -> Result<String, Refuse> {
    let mut f = OpenOptions::new().read(true).share_mode(SHARE_READ).open(pkg).map_err(|e| Refuse::Format(e.to_string()))?;
    let mut bytes = vec![];
    f.read_to_end(&mut bytes).map_err(|e| Refuse::Format(e.to_string()))?;
    if sha(&bytes) != want_sha { return Err(Refuse::Sha); }
    let sig = fs::read_to_string(pkg.with_extension("pkg.minisig")).map_err(|_| Refuse::Signature)?;
    let pk = minisign_verify::PublicKey::from_base64(pubkey).map_err(|_| Refuse::Signature)?;
    let sg = minisign_verify::Signature::decode(&sig).map_err(|_| Refuse::Signature)?;
    pk.verify(&bytes, &sg, false).map_err(|_| Refuse::Signature)?;
    after_verify(); // test hook: attacker acts here, while we still hold the handle
    // the version comes from inside the signed artifact, compared with the highest version ever seen
    let p: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| Refuse::Format(e.to_string()))?;
    let v = p["version"].as_str().unwrap_or("").to_string();
    let nv = ver(&v).ok_or(Refuse::Format("version".into()))?;
    let seen = fs::read_to_string(root.join("max_seen")).ok().and_then(|s| ver(s.trim())).unwrap_or((0, 0, 0));
    let cur = fs::read_to_string(root.join("current")).ok().and_then(|s| ver(s.trim())).unwrap_or((0, 0, 0));
    // equal to max_seen is allowed only while it is not installed yet (resume after a crash); older is never allowed
    if nv <= cur || nv < seen { return Err(Refuse::Downgrade(v)); }
    step("verified");
    atomic_write(&root.join("max_seen"), v.as_bytes()).unwrap();
    // extract into versions/<v>.tmp, then one directory rename
    let vdir = root.join("versions").join(&v);
    let tdir = root.join("versions").join(format!("{v}.tmp"));
    let _ = fs::remove_dir_all(&tdir);
    if !vdir.join("install.json").exists() {
        fs::create_dir_all(&tdir).unwrap();
        let mut rec = BTreeMap::new();
        for (name, data) in p["files"].as_object().ok_or(Refuse::Format("files".into()))? {
            if name.contains(['/', '\\', ':']) || name.starts_with('.') { return Err(Refuse::Format(format!("file name {name}"))); }
            let d = b64().decode(data.as_str().unwrap_or("")).map_err(|e| Refuse::Format(e.to_string()))?;
            let mut o = File::create(tdir.join(name)).unwrap();
            o.write_all(&d).unwrap();
            o.sync_all().unwrap();
            rec.insert(name.clone(), sha(&d));
            step("file");
        }
        atomic_write(&tdir.join("install.json"), serde_json::to_vec(&rec).unwrap().as_slice()).unwrap();
        step("extracted");
        let _ = fs::remove_dir_all(&vdir);
        fs::rename(&tdir, &vdir).unwrap();
    }
    step("renamed");
    if let Ok(c) = fs::read_to_string(root.join("current")) { atomic_write(&root.join("previous"), c.as_bytes()).unwrap(); }
    step("previous");
    atomic_write(&root.join("current"), v.as_bytes()).unwrap(); // the commit point
    step("switched");
    drop(f);
    Ok(v)
}

/// a version "starts" only if every file matches its install record
fn verified(root: &Path, v: &str) -> bool {
    let d = root.join("versions").join(v);
    let Ok(rec) = fs::read(d.join("install.json")) else { return false };
    let Ok(rec): Result<BTreeMap<String, String>, _> = serde_json::from_slice(&rec) else { return false };
    !rec.is_empty() && rec.iter().all(|(n, h)| fs::read(d.join(n)).map(|b| &sha(&b) == h).unwrap_or(false))
}

/// launcher: current if verified, else previous (+ banner "Update បរាជ័យ")
fn launch(root: &Path) -> Option<(String, bool)> {
    for (k, recovered) in [("current", false), ("previous", true)] {
        if let Ok(v) = fs::read_to_string(root.join(k)) { if verified(root, v.trim()) { return Some((v.trim().to_string(), recovered)); } }
    }
    None
}

fn make_pkg(dir: &Path, v: &str, payload_mb: usize, kp: &minisign::KeyPair, name: &str) -> (PathBuf, String) {
    let big: Vec<u8> = (0..payload_mb << 20).map(|i| (i * 31 % 251) as u8).collect();
    let mut files = serde_json::Map::new();
    files.insert("app.txt".into(), b64().encode(format!("GDM {v}")).into());
    files.insert("data.bin".into(), b64().encode(&big).into());
    let body = serde_json::to_vec(&serde_json::json!({ "version": v, "files": files })).unwrap();
    let p = dir.join(format!("{name}.pkg"));
    fs::write(&p, &body).unwrap();
    let sig = minisign::sign(Some(&kp.pk), &kp.sk, std::io::Cursor::new(&body), None, None).unwrap();
    fs::write(p.with_extension("pkg.minisig"), sig.into_string()).unwrap();
    (p, sha(&body))
}

fn fresh_root(base: &Path, tag: &str, kp: &minisign::KeyPair, pk: &str) -> PathBuf {
    let root = base.join(tag);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("versions")).unwrap();
    let (p, h) = make_pkg(base, "1.0.0", 1, kp, &format!("{tag}-old"));
    install(&root, &p, &h, pk, || {}).unwrap();
    root
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("install") => {
            let r = install(Path::new(&a[2]), Path::new(&a[3]), &a[4], &a[5], || {});
            println!("{r:?}");
            std::process::exit(if r.is_ok() { 0 } else { 3 });
        }
        Some("launch") => { println!("{:?}", launch(Path::new(&a[2]))); return; }
        _ => {}
    }
    let mut c = Check::new("S8 update install + recover");
    let base = std::env::temp_dir().join(format!("gdm-s8-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let kp = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
    let pk = kp.pk.to_base64();
    let other = minisign::KeyPair::generate_unencrypted_keypair().unwrap();

    // 1. happy path
    let root = fresh_root(&base, "ok", &kp, &pk);
    let (p, h) = make_pkg(&base, "1.1.0", 1, &kp, "new");
    c.ok(install(&root, &p, &h, &pk, || {}) == Ok("1.1.0".into()) && launch(&root) == Some(("1.1.0".into(), false)), "signed 1.1.0 over 1.0.0 -> installed, launcher starts 1.1.0");

    // 2. refusals
    let mut bad = fs::read(&p).unwrap();
    let n = bad.len() - 3;
    bad[n] ^= 1;
    let pt = base.join("tampered.pkg");
    fs::write(&pt, &bad).unwrap();
    fs::copy(p.with_extension("pkg.minisig"), pt.with_extension("pkg.minisig")).unwrap();
    c.ok(install(&root, &pt, &h, &pk, || {}) == Err(Refuse::Sha), "1 flipped byte -> sha256 refused");
    c.ok(install(&root, &pt, &sha(&bad), &pk, || {}) == Err(Refuse::Signature), "1 flipped byte with a matching (attacker) sha256 -> minisign refused");
    let (po, ho) = make_pkg(&base, "1.2.0", 1, &other, "otherkey");
    c.ok(install(&root, &po, &ho, &pk, || {}) == Err(Refuse::Signature), "signed by another key -> refused");
    let (pd, hd) = make_pkg(&base, "1.0.5", 1, &kp, "down");
    c.ok(install(&root, &pd, &hd, &pk, || {}) == Err(Refuse::Downgrade("1.0.5".into())), "genuine but older (1.0.5 < 1.1.0) -> downgrade refused");
    c.ok(install(&root, &p, &h, &pk, || {}) == Err(Refuse::Downgrade("1.1.0".into())), "same version again -> refused");
    c.ok(launch(&root) == Some(("1.1.0".into(), false)), "after all refusals the launcher still starts 1.1.0");

    // 3. U-06: attacker acts between verify and install while we hold the handle
    let root = fresh_root(&base, "u06", &kp, &pk);
    let sdir = base.join("staging");
    fs::create_dir_all(&sdir).unwrap();
    let (p, h) = make_pkg(&sdir, "1.1.0", 1, &kp, "staged");
    let (evil, _) = make_pkg(&base, "9.9.9", 1, &other, "evil");
    let mut seen = vec![];
    let r = install(&root, &p, &h, &pk, || {
        seen.push(("overwrite staged file", OpenOptions::new().write(true).open(&p).is_ok()));
        seen.push(("delete staged file", fs::remove_file(&p).is_ok()));
        seen.push(("rename staged file", fs::rename(&p, sdir.join("x.pkg")).is_ok()));
        let moved = fs::rename(&sdir, base.join("staging-moved")).is_ok();
        seen.push(("rename parent staging folder", moved));
        if moved { fs::create_dir_all(&sdir).unwrap(); fs::copy(&evil, sdir.join("staged.pkg")).unwrap(); }
    });
    for (what, worked) in &seen { println!("     U-06 attacker: {what}: {}", if *worked { "succeeded" } else { "blocked" }); }
    c.ok(!seen[0].1 && !seen[1].1 && !seen[2].1, "while the verified handle is held: overwrite / delete / rename of the staged file are blocked");
    let installed = fs::read_to_string(root.join("versions/1.1.0/app.txt")).unwrap_or_default();
    c.ok(r == Ok("1.1.0".into()) && installed == "GDM 1.1.0" && launch(&root) == Some(("1.1.0".into(), false)), &format!("installed bytes = the verified bytes even if the folder was swapped (parent rename {})", if seen[3].1 { "succeeded, evil package planted, ignored" } else { "blocked" }));

    // 4. U-04: kill at every step (failpoints), then random TerminateProcess kills
    let exe = std::env::current_exe().unwrap();
    let (p, h) = make_pkg(&base, "1.1.0", 24, &kp, "big");
    let run_install = |root: &Path, fp: Option<&str>| {
        let mut cmd = Command::new(&exe);
        cmd.args(["install", root.to_str().unwrap(), p.to_str().unwrap(), &h, &pk]).env("GDM_SLOW_MS", "40").stdout(Stdio::null()).stderr(Stdio::null());
        if let Some(f) = fp { cmd.env("GDM_FAILPOINT", f); }
        cmd
    };
    let mut fp_ok = true;
    for fp in ["verified", "file", "extracted", "renamed", "previous", "switched"] {
        let root = fresh_root(&base, &format!("fp-{fp}"), &kp, &pk);
        let st = run_install(&root, Some(fp)).status().unwrap();
        let l = launch(&root);
        let good = !st.success() && matches!(&l, Some((v, _)) if v == "1.0.0" || v == "1.1.0");
        println!("     failpoint {fp}: launcher -> {l:?}");
        // and the next attempt completes
        let again = run_install(&root, None).status().unwrap().success() || launch(&root).map(|x| x.0) == Some("1.1.0".into());
        fp_ok &= good && again && launch(&root).map(|x| x.0) == Some("1.1.0".into());
    }
    c.ok(fp_ok, "crash at each install step: launcher starts 1.0.0 or verified 1.1.0, and the retry finishes on 1.1.0");
    let (mut rng, mut ok, mut olds, mut news) = (0xC0FFEEu64, 0, 0, 0);
    let rounds = 20;
    let t0 = Instant::now();
    for i in 0..rounds {
        let root = fresh_root(&base, &format!("kill-{i}"), &kp, &pk);
        let mut ch = run_install(&root, None).spawn().unwrap();
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        std::thread::sleep(Duration::from_millis(20 + (rng >> 33) % 1500));
        let _ = ch.kill();
        let _ = ch.wait();
        match launch(&root) { Some((v, _)) if v == "1.0.0" => { ok += 1; olds += 1; } Some((v, _)) if v == "1.1.0" => { ok += 1; news += 1; } _ => {} }
    }
    c.ok(ok == rounds, &format!("{ok}/{rounds} random kills during install: launcher starts a verified version ({olds} x 1.0.0, {news} x 1.1.0, {:.0}s)", t0.elapsed().as_secs_f64()));
    // pointer to a broken version dir -> previous + banner
    let root = fresh_root(&base, "broken", &kp, &pk);
    let (p2, h2) = make_pkg(&base, "1.1.0", 1, &kp, "b2");
    install(&root, &p2, &h2, &pk, || {}).unwrap();
    fs::write(root.join("versions/1.1.0/app.txt"), "corrupted").unwrap();
    c.ok(launch(&root) == Some(("1.0.0".into(), true)), "new version files damaged after install -> launcher falls back to 1.0.0 with the 'Update បរាជ័យ' banner flag");
    c.note("NSIS passive in-place install is NOT the tested path: plan.md plan B (A/B folders + pointer) is what passed; the real Tauri/NSIS updater path is a P1 decision");
    c.finish();
}
