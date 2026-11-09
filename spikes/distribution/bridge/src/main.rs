// SPIKE ONLY (G2) - throwaway, never shipped.
// gdm-bridge: native messaging host (stateless) + per-user register helper
// + component fetch with signed manifest. One binary to keep the spike small.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::{env, fs};

const HOST: &str = "com.goldeniq.gdm";
const CHROME_IDS: &[&str] = &["fccmejdfnnlgffbbcbakjpdodjjoeonh"];
const FIREFOX_IDS: &[&str] = &["gdm-spike@goldeniq.dev"];
const MAX_MSG: u32 = 256 * 1024; // B-03: reject before allocating
const VER: &str = match option_env!("GDM_SPIKE_VER") { Some(v) => v, None => env!("CARGO_PKG_VERSION") };
// U-06: the trusted key is compiled in, never read from the manifest.
const PUBKEY: Option<&str> = option_env!("GDM_SPIKE_PUBKEY");

// B-01: typed protocol, no dir/path/hook field; unknown fields are rejected.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
enum Req {
    Ping { nonce: String },
    Ack { nonce: String, ver: String },
    Handoff { url: String, referrer: Option<String>, name: Option<String>, size: Option<u64>, mime: Option<String> },
}

fn log(line: &str) {
    if let Ok(p) = env::var("GDM_SPIKE_LOG") {
        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(p) {
            let _ = writeln!(f, "{} pid={} exe={} {}", VER, std::process::id(),
                env::current_exe().map(|p| p.display().to_string()).unwrap_or_default(), line);
        }
    }
}

fn caller(args: &[String]) -> Option<String> {
    let a1 = args.get(1)?;
    if let Some(id) = a1.strip_prefix("chrome-extension://").and_then(|s| s.strip_suffix('/')) {
        return CHROME_IDS.contains(&id).then(|| a1.clone()); // Chrome/Edge pass the origin
    }
    if a1.to_ascii_lowercase().ends_with(".json") {
        let id = args.get(2)?; // Firefox: argv[1]=manifest path, argv[2]=extension id
        return FIREFOX_IDS.contains(&id.as_str()).then(|| id.clone());
    }
    None
}

fn validate(r: &Req) -> Result<(), &'static str> {
    let short = |s: &str, n: usize| s.len() <= n;
    match r {
        Req::Ping { nonce } => short(nonce, 64).then_some(()).ok_or("nonce too long"),
        Req::Ack { nonce, ver } => (short(nonce, 64) && short(ver, 32)).then_some(()).ok_or("ack too long"),
        Req::Handoff { url, referrer, name, mime, .. } => {
            let ok = short(url, 8192) && (url.starts_with("https://") || url.starts_with("http://"))
                && referrer.as_deref().map_or(true, |s| short(s, 8192))
                && name.as_deref().map_or(true, |s| short(s, 1024))
                && mime.as_deref().map_or(true, |s| short(s, 256));
            ok.then_some(()).ok_or("bad handoff")
        }
    }
}

fn reply(out: &mut impl Write, v: &serde_json::Value) -> io::Result<()> {
    let b = serde_json::to_vec(v)?;
    out.write_all(&(b.len() as u32).to_le_bytes())?;
    out.write_all(&b)?;
    out.flush()
}

fn host(who: String) -> i32 {
    let (mut inp, mut out) = (io::stdin().lock(), io::stdout().lock());
    loop {
        let mut len = [0u8; 4];
        if inp.read_exact(&mut len).is_err() { return 0; } // browser closed the pipe
        let len = u32::from_le_bytes(len);
        if len > MAX_MSG { log(&format!("{who} REJECT size={len}")); return 2; }
        let mut buf = vec![0u8; len as usize];
        if inp.read_exact(&mut buf).is_err() { return 2; }
        let res = serde_json::from_slice::<Req>(&buf).map_err(|e| e.to_string())
            .and_then(|r| validate(&r).map(|_| r).map_err(String::from));
        let v = match res {
            Ok(Req::Ping { nonce }) => { log(&format!("{who} PING {nonce}"));
                serde_json::json!({"ok":true,"type":"pong","nonce":nonce,"ver":VER}) }
            Ok(Req::Ack { nonce, ver }) => { log(&format!("{who} ACK {nonce} ver={ver}"));
                serde_json::json!({"ok":true}) }
            Ok(Req::Handoff { url, .. }) => { log(&format!("{who} HANDOFF {url}"));
                // B-04: the engine decides; the bridge never starts anything itself.
                serde_json::json!({"ok":true,"queued_for_review":true}) }
            Err(e) => { log(&format!("{who} REJECT {e}"));
                serde_json::json!({"ok":false,"error":"rejected"}) }
        };
        if reply(&mut out, &v).is_err() { return 2; }
    }
}

// ---- per-user registration helper (plan B for MSIX, and what NSIS calls) ----
const KEYS: &[(&str, &str, bool)] = &[ // (browser, HKCU key, firefox-style manifest)
    ("chrome", r"Software\Google\Chrome\NativeMessagingHosts", false),
    ("edge", r"Software\Microsoft\Edge\NativeMessagingHosts", false),
    ("firefox", r"Software\Mozilla\NativeMessagingHosts", true),
];

fn local(rel: &str) -> PathBuf { PathBuf::from(env::var("LOCALAPPDATA").expect("LOCALAPPDATA")).join(rel) }

fn register(exe: &Path, only: Option<&str>) -> io::Result<()> {
    let dir = local(r"GDM\nmh");
    fs::create_dir_all(&dir)?;
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    for (b, key, ff) in KEYS {
        if only.is_some_and(|o| o != *b) { continue; }
        let mut m = serde_json::json!({"name":HOST,"description":"GDM bridge spike","path":exe,"type":"stdio"});
        if *ff { m["allowed_extensions"] = serde_json::json!(FIREFOX_IDS); }
        else { m["allowed_origins"] = CHROME_IDS.iter().map(|i| format!("chrome-extension://{i}/")).collect(); }
        let file = dir.join(format!("{b}.json"));
        fs::write(&file, serde_json::to_vec_pretty(&m)?)?;
        let (k, _) = hkcu.create_subkey(format!(r"{key}\{HOST}"))?;
        k.set_value("", &file.display().to_string())?;
        println!("registered {b}: HKCU\\{key}\\{HOST} = {}", file.display());
    }
    Ok(())
}

fn unregister() {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    for (b, key, _) in KEYS {
        let r = hkcu.delete_subkey_all(format!(r"{key}\{HOST}"));
        println!("unregister {b}: {}", if r.is_ok() { "removed" } else { "absent" });
    }
    let _ = fs::remove_dir_all(local(r"GDM\nmh"));
}

// ---- component fetch: signed manifest -> download -> sha256 -> verify on held handle ----
#[derive(Deserialize)]
struct Comp { name: String, url: String, sha256: String, file: String, args: Vec<String> }

fn sha256_hex(mut r: impl Read) -> io::Result<String> {
    let mut h = Sha256::new();
    io::copy(&mut r, &mut h)?;
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

fn fetch(manifest: &Path, sig: &Path, dest: Option<PathBuf>) -> Result<(), String> {
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    let pk = minisign::PublicKey::from_base64(PUBKEY.ok_or("built without GDM_SPIKE_PUBKEY")?).map_err(|e| e.to_string())?;
    let data = fs::read(manifest).map_err(|e| e.to_string())?;
    let sbox = minisign::SignatureBox::from_string(&fs::read_to_string(sig).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    minisign::verify(&pk, &sbox, io::Cursor::new(&data), true, false, false).map_err(|e| format!("manifest signature: {e}"))?;
    println!("manifest signature OK");
    let comps: Vec<Comp> = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
    let dir = dest.unwrap_or_else(|| local(r"GDM\components"));
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    if fs::symlink_metadata(&dir).map_err(|e| e.to_string())?.file_attributes() & 0x400 != 0 {
        return Err("components dir is a reparse point".into()); // U-06
    }
    for c in comps {
        if c.file.contains(['\\', '/', ':']) || c.file.starts_with('.') { return Err("bad file name".into()); }
        let part = dir.join(format!("{}.part", c.file));
        let fin = dir.join(&c.file);
        let resp = ureq::get(&c.url).call().map_err(|e| e.to_string())?;
        let mut f = fs::File::create(&part).map_err(|e| e.to_string())?;
        io::copy(&mut resp.into_reader(), &mut f).map_err(|e| e.to_string())?;
        drop(f);
        let ok = |p: &Path| -> Result<(fs::File, String), String> {
            // A handle that denies write/delete; the hash is taken through it.
            let h = fs::OpenOptions::new().read(true).share_mode(1 /*FILE_SHARE_READ*/).open(p).map_err(|e| e.to_string())?;
            let got = sha256_hex(&h).map_err(|e| e.to_string())?;
            Ok((h, got))
        };
        let (h, got) = ok(&part)?; // never give an unverified file its final name
        drop(h);
        if !got.eq_ignore_ascii_case(&c.sha256) { let _ = fs::remove_file(&part); return Err(format!("{}: sha256 mismatch {got}", c.name)); }
        fs::rename(&part, &fin).map_err(|e| e.to_string())?;
        // Re-verify on a held handle and run while still holding it (U-06 verify-to-execute).
        let (held, got) = ok(&fin)?;
        if !got.eq_ignore_ascii_case(&c.sha256) { drop(held); let _ = fs::remove_file(&fin); return Err(format!("{}: swapped after rename", c.name)); }
        println!("{}: sha256 OK {got} -> {}", c.name, fin.display());
        let tamper = fs::OpenOptions::new().write(true).open(&fin).is_err();
        println!("{}: write while held blocked = {tamper}", c.name);
        let o = std::process::Command::new(&fin).args(&c.args).output().map_err(|e| format!("run: {e}"))?;
        println!("{}: run exit={} stdout={}", c.name, o.status, String::from_utf8_lossy(&o.stdout).trim());
        drop(held);
    }
    Ok(())
}

fn keygen(dir: &Path) -> Result<(), String> {
    let kp = minisign::KeyPair::generate_unencrypted_keypair().map_err(|e| e.to_string())?;
    fs::write(dir.join("spike.pub"), kp.pk.to_base64()).map_err(|e| e.to_string())?;
    fs::write(dir.join("spike.key"), kp.sk.to_bytes()).map_err(|e| e.to_string())?; // spike: raw, unencrypted
    Ok(())
}

fn sign(file: &Path, key: &Path) -> Result<(), String> {
    let sk = minisign::SecretKey::from_bytes(&fs::read(key).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let data = fs::read(file).map_err(|e| e.to_string())?;
    let sig = minisign::sign(None, &sk, io::Cursor::new(data), Some("gdm components spike"), None).map_err(|e| e.to_string())?;
    fs::write(format!("{}.minisig", file.display()), sig.to_string()).map_err(|e| e.to_string())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let a = |i: usize| args.get(i).map(String::as_str);
    let code = match a(1) {
        Some(x) if x.starts_with("chrome-extension://") || x.to_ascii_lowercase().ends_with(".json") => match caller(&args) {
            Some(who) => host(who),
            None => { log(&format!("REJECT origin {:?}", &args[1..])); 3 } // B-01
        },
        None | Some("register") => {
            let exe = a(2).filter(|s| !s.starts_with("--")).map(PathBuf::from)
                .unwrap_or_else(|| env::current_exe().unwrap());
            let only = args.iter().position(|s| s == "--only").and_then(|i| a(i + 1));
            register(&exe, only).map(|_| 0).unwrap_or_else(|e| { eprintln!("{e}"); 1 })
        }
        Some("unregister") => { unregister(); 0 }
        Some("fetch-component") => fetch(Path::new(a(2).unwrap()), Path::new(a(3).unwrap()), a(4).map(PathBuf::from))
            .map(|_| 0).unwrap_or_else(|e| { eprintln!("FAIL {e}"); 1 }),
        Some("keygen") => keygen(Path::new(a(2).unwrap())).map(|_| 0).unwrap_or_else(|e| { eprintln!("{e}"); 1 }),
        Some("sign") => sign(Path::new(a(2).unwrap()), Path::new(a(3).unwrap())).map(|_| 0).unwrap_or_else(|e| { eprintln!("{e}"); 1 }),
        Some(x) => { log(&format!("REJECT unknown caller {x}")); 3 }
    };
    std::process::exit(code);
}
