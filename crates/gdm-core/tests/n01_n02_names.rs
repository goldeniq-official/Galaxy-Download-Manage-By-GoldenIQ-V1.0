// SPDX-License-Identifier: RPL-1.5
//! N-01 (path traversal via Content-Disposition / URL) and N-02 (RTLO, double extension).
//! Card test: table of 60+ hostile inputs; RTLO name => no U+202E on disk, real extension shown.
use gdm_core::name::{dangerous, file_name, real_extension, sanitize, with_counter};

fn ok(n: &str) -> bool {
    !n.is_empty()
        && n.chars().all(|c| !matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') && !c.is_control())
        && !n.ends_with('.')
        && !n.ends_with(' ')
        && n != "."
        && n != ".."
        && !n.chars().any(|c| matches!(c as u32, 0x202A..=0x202E | 0x2066..=0x2069 | 0x200E | 0x200F | 0x200B | 0xFEFF))
        && n.encode_utf16().count() <= 200
}

#[test]
fn hostile_table_never_escapes_or_breaks_windows_rules() {
    let inputs: &[&str] = &[
        "..\\..\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup\\a.exe",
        "../../etc/passwd",
        "..",
        ".",
        "...",
        "/",
        "\\",
        "C:\\Windows\\System32\\drivers\\etc\\hosts",
        "C:evil.exe",
        "\\\\?\\C:\\x.exe",
        "\\\\server\\share\\x.exe",
        "//server/share/x.exe",
        "a/b/c.txt",
        "a\\b\\c.txt",
        "file.txt:Zone.Identifier",
        "file.txt::$DATA",
        "CON",
        "con.txt",
        "PRN.log",
        "AUX",
        "NUL.tar.gz",
        "COM1",
        "com9.txt",
        "LPT1",
        "lpt9.doc",
        "COM¹.txt",
        "LPT³",
        "CONIN$",
        "CONOUT$",
        "name. . .",
        "name   ",
        "trailing.",
        " leading-space.txt",
        "tab\tname.txt",
        "new\nline.txt",
        "cr\rname.txt",
        "nul\0byte.exe",
        "bell\x07.txt",
        "del\x7f.txt",
        "a*b?c.txt",
        "quote\"name.txt",
        "lt<gt>.txt",
        "pipe|name.txt",
        "invoice\u{202E}fdp.exe",
        "invoice\u{202D}x.exe",
        "a\u{2066}b\u{2069}.exe",
        "a\u{200E}b\u{200F}.txt",
        "zero\u{200B}width.txt",
        "\u{FEFF}bom.txt",
        "report.pdf.exe",
        "",
        "   ",
        "....",
        "%2e%2e%5c%2e%2e%5cx.exe",
        "..%2f..%2fx.exe",
        "x/../../y.exe",
        "folder/",
        "folder\\",
        "a:b:c",
        "~1",
        "PROGRA~1",
        "\u{0000}",
        "\u{001F}x",
        "very long ".repeat(60).leak(),
        "ខ្មែរ".repeat(150).leak(),
        "emoji😀😀😀.txt",
        "e\u{301}.txt",
        "ﬁle.txt",
    ];
    assert!(inputs.len() >= 60, "{} inputs", inputs.len());
    for i in inputs {
        let n = sanitize(i);
        assert!(ok(&n), "input {i:?} -> {n:?}");
        assert!(!n.contains(".."), "input {i:?} -> {n:?} still has ..");
        let stem = n.split('.').next().unwrap().trim_end().to_ascii_uppercase();
        let reserved = ["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"].contains(&stem.as_str())
            || ((stem.starts_with("COM") || stem.starts_with("LPT")) && stem.chars().count() == 4 && stem.chars().last().is_some_and(|c| c.is_ascii_digit() || "¹²³".contains(c)));
        assert!(!reserved, "input {i:?} -> reserved device name {n:?}");
    }
}

#[test]
fn last_segment_only() {
    assert_eq!(sanitize("..\\..\\Startup\\a.exe"), "a.exe");
    assert_eq!(sanitize("../../etc/passwd"), "passwd");
    assert_eq!(sanitize("C:\\Windows\\x.dll"), "x.dll");
    assert_eq!(sanitize("con.txt"), "_con.txt");
    assert_eq!(sanitize("name. . ."), "name");
    assert_eq!(sanitize(""), "download");
    assert_eq!(sanitize("a:b"), "b"); // a drive-relative / ADS colon is a separator, never kept
    // working-file suffixes are never final names (no collision with another job's .part / sidecar)
    for (i, o) in [("a.bin.part", "a.bin.part_"), ("x.PART", "x.PART_"), ("y.part.gdm", "y.part.gdm_"), ("s.gdm.tmp", "s.gdm.tmp_"), ("z.probe", "z.probe_"), ("ok.part1.rar", "ok.part1.rar")] {
        assert_eq!(sanitize(i), o);
    }
}

#[test]
fn n02_bidi_stripped_real_extension_dangerous() {
    let n = sanitize("invoice\u{202E}fdp.exe");
    assert!(!n.contains('\u{202E}'));
    assert_eq!(n, "invoicefdp.exe");
    assert_eq!(real_extension(&n), "exe");
    assert!(dangerous(&n));
    assert_eq!(real_extension("report.pdf.exe"), "exe");
    assert!(dangerous("report.pdf.exe"));
    assert!(dangerous("SETUP.MSI") && dangerous("a.ps1") && dangerous("a.lnk") && dangerous("a.iso") && dangerous("a.vhd"));
    assert!(!dangerous("movie.mp4") && !dangerous("archive.zip"));
    // ZWJ / ZWNJ can carry meaning in scripts: kept
    assert_eq!(sanitize("a\u{200D}b\u{200C}c.txt"), "a\u{200D}b\u{200C}c.txt");
    // NFC: e + combining acute == é
    assert_eq!(sanitize("e\u{301}.txt"), "\u{e9}.txt");
}

#[test]
fn content_disposition_and_url() {
    let u = |s: &str| url::Url::parse(s).unwrap();
    // filename* (RFC 8187) wins over filename (RFC 6266)
    assert_eq!(file_name(Some("attachment; filename=\"plain.txt\"; filename*=UTF-8''%E1%9E%81%E1%9F%92.txt"), &u("http://h/x")), "ខ្.txt");
    assert_eq!(file_name(Some("attachment; filename=\"..\\\\..\\\\evil.exe\""), &u("http://h/x")), "evil.exe");
    assert_eq!(file_name(Some("attachment; filename=simple.zip"), &u("http://h/x")), "simple.zip");
    assert_eq!(file_name(Some("inline"), &u("http://h/dir/file%20name.iso?x=1")), "file name.iso");
    assert_eq!(file_name(None, &u("http://h/a/%2e%2e%5cevil.exe")), "evil.exe");
    assert_eq!(file_name(None, &u("http://h/")), "download");
    assert_eq!(file_name(Some("attachment; filename*=UTF-8''%E2%80%AEfdp.exe"), &u("http://h/")), "fdp.exe");
    assert_eq!(file_name(Some("attachment; filename*=garbage"), &u("http://h/ok.bin")), "ok.bin");
}

#[test]
fn n04_counter_keeps_extension() {
    assert_eq!(with_counter("setup.exe", 1), "setup (1).exe");
    assert_eq!(with_counter("archive.tar.gz", 2), "archive.tar (2).gz");
    assert_eq!(with_counter("README", 3), "README (3)");
}

#[test]
fn long_names_truncated_keep_extension() {
    let n = sanitize(&format!("{}.mp4", "ក".repeat(400)));
    assert!(n.encode_utf16().count() <= 200 && n.ends_with(".mp4"), "{n}");
}
