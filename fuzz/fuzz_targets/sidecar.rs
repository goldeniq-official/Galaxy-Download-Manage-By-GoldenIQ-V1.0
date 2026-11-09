// SPDX-License-Identifier: RPL-1.5
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Some(sc) = gdm_core::sidecar::decode(data) {
        // anything accepted is well-formed: contiguous cover of [0, total), committed inside each segment
        let mut prev = 0;
        for [a, e, d] in &sc.segs {
            assert!(*a == prev && a < e && a <= d && d <= e);
            prev = *e;
        }
        assert_eq!(prev, sc.total);
        assert_eq!(gdm_core::name::sanitize(&sc.name), sc.name);
    }
});
