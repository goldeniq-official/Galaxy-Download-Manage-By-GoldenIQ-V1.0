// SPDX-License-Identifier: RPL-1.5
//! The one filename sanitizer (N-01, N-02, §13). Every name that reaches the filesystem goes through
//! `sanitize`; the result is a single path component. Creation inside the directory is done by handle
//! (`fsx::create_part`), so even a sanitizer bug cannot leave the directory.
use unicode_normalization::UnicodeNormalization;

const MAX_UTF16: usize = 200;
const DANGEROUS: &[&str] = &[
    "exe", "msi", "msix", "appx", "bat", "cmd", "com", "ps1", "psm1", "vbs", "vbe", "js", "jse", "wsf", "wsh", "hta", "scr", "pif", "cpl", "lnk",
    "url", "reg", "iso", "img", "vhd", "vhdx", "jar", "dll", "sys", "msc", "chm", "inf",
];
/// what runs (or installs, or mounts something that runs) on a double-click on this system (P4)
#[cfg(target_vendor = "apple")]
const DANGEROUS_OS: &[&str] = &[
    "py",
    "pyw",
    "pyc",
    "pl",
    "rb",
    "jnlp",
    "afploc",
    "ftploc",
    "bash",
    "zsh",
    "csh",
    "ksh",
    "url",
    "app",
    "command",
    "tool",
    "terminal",
    "sh",
    "scpt",
    "scptd",
    "applescript",
    "workflow",
    "action",
    "pkg",
    "mpkg",
    "dmg",
    "webloc",
    "inetloc",
    "fileloc",
    "prefpane",
    "kext",
    "dylib",
    "osx",
];
#[cfg(all(unix, not(target_vendor = "apple")))]
const DANGEROUS_OS: &[&str] =
    &["desktop", "sh", "bash", "zsh", "ksh", "csh", "run", "bin", "appimage", "deb", "rpm", "snap", "flatpakref", "flatpak", "py", "pl", "rb", "so"];
#[cfg(windows)]
const DANGEROUS_OS: &[&str] = &[];

/// one safe Windows path component, never empty
pub fn sanitize(raw: &str) -> String {
    // last segment only (RFC 6266 §4.3): `/`, `\` and `:` (drive / ADS) are separators, never kept
    let last = raw.rsplit(['/', '\\', ':']).next().unwrap_or("");
    let mut s: String = last
        .nfc()
        .filter(|c| {
            let u = *c as u32;
            !(c.is_control()
                || matches!(c, '*' | '?' | '"' | '<' | '>' | '|')
                || matches!(u, 0x202A..=0x202E | 0x2066..=0x2069 | 0x200E | 0x200F | 0x200B | 0xFEFF))
        })
        .collect();
    // Windows drops trailing dots and spaces ("." and ".." vanish here too)
    s = s.trim_start_matches(' ').trim_end_matches(['.', ' ']).to_string();
    while s.contains("..") {
        s = s.replace("..", ".");
    }
    if s.is_empty() {
        s = "download".into();
    }
    if reserved(&s) {
        s.insert(0, '_');
    }
    let mut s = truncate(s);
    // the engine's own working names (`x.part`, `x.part.gdm`, `.tmp`, `.probe`) are never a final name,
    // so no final file can ever collide with another job's working file
    let l = s.to_ascii_lowercase();
    if [".part", ".gdm", ".tmp", ".probe"].iter().any(|x| l.ends_with(x)) {
        s.push('_');
    }
    s
}

fn reserved(s: &str) -> bool {
    let stem = s.split('.').next().unwrap_or("").trim_end().to_ascii_uppercase();
    if ["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"].contains(&stem.as_str()) {
        return true;
    }
    let mut c = stem.chars();
    let p: String = c.by_ref().take(3).collect();
    let d: Vec<char> = c.collect();
    (p == "COM" || p == "LPT") && d.len() == 1 && (d[0].is_ascii_digit() || "¹²³".contains(d[0]))
}

/// <= 200 UTF-16 units, extension kept, cut on a char boundary.
// ponytail: char boundary, not grapheme (N-06 medium wants grapheme for Khmer clusters); add
// unicode-segmentation when N-06 is built.
fn truncate(s: String) -> String {
    if s.encode_utf16().count() <= MAX_UTF16 {
        return s;
    }
    let ext = match ext_start(&s) {
        Some(i) if s.len() - i <= 16 => s[i..].to_string(),
        _ => String::new(),
    };
    let room = MAX_UTF16 - ext.encode_utf16().count();
    let mut out = String::new();
    let mut n = 0;
    for c in s[..s.len() - ext.len()].chars() {
        if n + c.len_utf16() > room {
            break;
        }
        n += c.len_utf16();
        out.push(c);
    }
    let out = out.trim_end_matches(['.', ' ']).to_string();
    format!("{}{ext}", if out.is_empty() { "download" } else { &out })
}

/// the real (last) extension, lower case, "" when none
pub fn real_extension(name: &str) -> String {
    match name.rfind('.') {
        Some(i) if i + 1 < name.len() => name[i + 1..].to_ascii_lowercase(),
        _ => String::new(),
    }
}

/// never auto-opened, labelled in the UI (N-02)
pub fn dangerous(name: &str) -> bool {
    let e = real_extension(name);
    DANGEROUS.contains(&e.as_str()) || DANGEROUS_OS.contains(&e.as_str())
}

/// "setup.exe", 1 -> "setup (1).exe"; "archive.tar.gz" -> "archive (1).tar.gz" (N-04, as Chrome and Firefox)
pub fn with_counter(name: &str, n: u32) -> String {
    match ext_start(name) {
        Some(i) => format!("{} ({n}){}", &name[..i], &name[i..]),
        None => format!("{name} ({n})"),
    }
}

/// compression suffixes that make one extension with a short part before them: Chromium's
/// kCommonDoubleExtensionSuffixes (base/files/file_path.cc) plus lz4 (".tar.lz4")
const DOUBLE_SUFFIXES: &[&str] = &["bz", "bz2", "gz", "lz", "lz4", "lzma", "lzo", "xz", "z", "zst"];

/// the byte index of the dot that starts the whole extension, None when there is none (or a dot file):
/// "a.zip" -> ".zip"; "a.tar.gz" -> ".tar.gz" (Chromium: a compression suffix after a 1-4 char part);
/// "a.7z.001" -> ".7z.001" (a split volume: 3 digits after a 1-4 char part); "a.part1.rar" -> ".part1.rar".
/// Names only: what a file IS (and whether it is dangerous) is never decided from this.
pub fn ext_start(name: &str) -> Option<usize> {
    let last = name.rfind('.').filter(|&i| i > 0 && i + 1 < name.len())?;
    let fin = name[last + 1..].to_ascii_lowercase();
    let Some(prev) = name[..last].rfind('.').filter(|&i| i > 0) else { return Some(last) };
    let mid = &name[prev + 1..last];
    let short = (1..=4).contains(&mid.len()) && mid.bytes().all(|b| b.is_ascii_alphanumeric());
    let double = short && DOUBLE_SUFFIXES.contains(&fin.as_str());
    let volume = short && fin.len() == 3 && fin.bytes().all(|b| b.is_ascii_digit());
    let rar_part = fin == "rar"
        && mid.len() <= 8
        && mid.to_ascii_lowercase().strip_prefix("part").is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()));
    Some(if double || volume || rar_part { prev } else { last })
}

/// the type a name sorts and is caught by (categories, catch lists), lower case, "" when none: the last
/// extension, except split volumes, which take their archive's type: "a.7z.001" -> "7z", "a.r00"/"a.s01" ->
/// "rar", "a.z01" -> "zip". A routing hint only: `dangerous` keeps the real last extension.
pub fn type_extension(name: &str) -> String {
    let e = real_extension(name);
    let b = e.as_bytes();
    if b.len() == 3 && b[1].is_ascii_digit() && b[2].is_ascii_digit() {
        match b[0] {
            b'r' | b's' => return "rar".into(),
            b'z' => return "zip".into(),
            _ => {}
        }
    }
    if let Some(i) = ext_start(name) {
        let full = name[i + 1..].to_ascii_lowercase();
        if let Some((inner, vol)) = full.split_once('.') {
            if vol.len() == 3 && vol.bytes().all(|c| c.is_ascii_digit()) {
                return inner.to_string();
            }
        }
    }
    e
}

/// name from Content-Disposition (filename* per RFC 8187 first, then filename), else the URL's last path
/// segment; always sanitized
pub fn file_name(content_disposition: Option<&str>, url: &url::Url) -> String {
    if let Some(cd) = content_disposition {
        let params: Vec<(String, String)> = split_params(cd);
        let star = params.iter().find(|(k, _)| k == "filename*").and_then(|(_, v)| ext_value(v));
        let plain = params.iter().find(|(k, _)| k == "filename").map(|(_, v)| v.clone());
        if let Some(n) = star.or(plain).filter(|n| !n.trim().is_empty()) {
            return sanitize(&n);
        }
    }
    let seg = url.path_segments().and_then(|mut s| s.next_back()).unwrap_or("");
    sanitize(&pct_decode(seg))
}

/// `k=v; k="quoted \" v"` -> [(k lower, v unquoted)]
fn split_params(s: &str) -> Vec<(String, String)> {
    let mut out = vec![];
    let b: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < b.len() {
        while i < b.len() && (b[i] == ';' || b[i] == ' ') {
            i += 1;
        }
        let k0 = i;
        while i < b.len() && b[i] != '=' && b[i] != ';' {
            i += 1;
        }
        let key = b[k0..i].iter().collect::<String>().trim().to_ascii_lowercase();
        if i >= b.len() || b[i] == ';' {
            continue;
        }
        i += 1; // '='
        let mut v = String::new();
        if i < b.len() && b[i] == '"' {
            i += 1;
            while i < b.len() && b[i] != '"' {
                if b[i] == '\\' && i + 1 < b.len() {
                    i += 1;
                }
                v.push(b[i]);
                i += 1;
            }
            i += 1;
        } else {
            while i < b.len() && b[i] != ';' {
                v.push(b[i]);
                i += 1;
            }
            v = v.trim().to_string();
        }
        out.push((key, v));
    }
    out
}

/// RFC 8187 ext-value: charset'lang'pct-encoded (UTF-8 only; anything else is ignored)
fn ext_value(v: &str) -> Option<String> {
    let mut p = v.splitn(3, '\'');
    let cs = p.next()?;
    let _lang = p.next()?;
    let val = p.next()?;
    cs.eq_ignore_ascii_case("utf-8").then(|| pct_decode(val))
}

pub(crate) fn pct_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut o = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Some(v) = std::str::from_utf8(&b[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()) {
                o.push(v);
                i += 3;
                continue;
            }
        }
        o.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&o).into_owned()
}
