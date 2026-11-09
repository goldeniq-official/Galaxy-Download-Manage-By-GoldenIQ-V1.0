// SPDX-License-Identifier: RPL-1.5
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let s = String::from_utf8_lossy(data);
    for n in [gdm_core::name::sanitize(&s), gdm_core::name::file_name(Some(&s), &url::Url::parse("http://h/x").unwrap())] {
        assert!(!n.is_empty() && n.encode_utf16().count() <= 201);
        assert!(!n.chars().any(|c| matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control()), "{n:?}");
        assert!(!n.contains("..") && !n.ends_with(' ') && !n.ends_with('.'), "{n:?}");
        assert!(!n.chars().any(|c| matches!(c as u32, 0x202A..=0x202E | 0x2066..=0x2069 | 0x200E | 0x200F | 0x200B | 0xFEFF)), "{n:?}");
        assert_eq!(gdm_core::name::sanitize(&n), n, "sanitize is idempotent");
    }
});
