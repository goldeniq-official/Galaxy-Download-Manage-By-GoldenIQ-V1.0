// SPDX-License-Identifier: RPL-1.5
// B-03: the bridge's message decoder never panics, and anything it accepts respects every limit
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(m) = gdm_bridge::parse(data) {
        assert!(m.validate().is_ok());
        if let gdm_bridge::Msg::Handoff { url, name, cookie, .. } = &m {
            assert!(url.len() <= gdm_bridge::MAX_URL && (url.starts_with("http://") || url.starts_with("https://")));
            assert!(name.as_deref().is_none_or(|n| n.len() <= gdm_bridge::MAX_NAME));
            assert!(cookie.as_deref().is_none_or(|c| c.len() <= gdm_bridge::MAX_COOKIE));
        }
    }
});
