// SPDX-License-Identifier: RPL-1.5
//! HTTP authentication helpers: Digest (RFC 7616) for origin logins and
//! Windows SSPI NTLM/Negotiate tokens for GDM's own proxy relay.

mod digest;
pub use digest::*;

#[cfg(windows)]
mod sspi;
#[cfg(windows)]
pub use sspi::*;

use base64::Engine as _;

pub fn b64(data: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(data)
}

pub fn unb64(s: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::STANDARD.decode(s.trim()).map_err(|_| "bad-base64".into())
}

/// `"<package> <base64>"` for a `Proxy-Authorization` header.
pub fn proxy_header_value(package: &str, token: &[u8]) -> String {
    format!("{package} {}", b64(token))
}

/// Server token from `Proxy-Authenticate: <package> <b64>` (one header value,
/// possibly a comma list of schemes). `None` when the package is absent or
/// offered without a token (the initial bare challenge).
pub fn parse_proxy_challenge(proxy_authenticate: &str, package: &str) -> Option<Vec<u8>> {
    proxy_authenticate.split(',').find_map(|part| {
        let mut it = part.split_whitespace();
        let scheme = it.next()?;
        if !scheme.eq_ignore_ascii_case(package) {
            return None;
        }
        unb64(it.next()?).ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn b64_roundtrip_and_proxy_header() {
        assert_eq!(b64(b"hello"), "aGVsbG8=");
        assert_eq!(unb64("aGVsbG8=").unwrap(), b"hello");
        assert!(unb64("!!").is_err());
        assert_eq!(proxy_header_value("NTLM", b"hello"), "NTLM aGVsbG8=");
        let h = "Basic realm=\"p\", NTLM aGVsbG8=";
        assert_eq!(parse_proxy_challenge(h, "ntlm").unwrap(), b"hello");
        assert_eq!(parse_proxy_challenge("Negotiate", "Negotiate"), None);
        assert_eq!(parse_proxy_challenge(h, "Negotiate"), None);
    }
}
