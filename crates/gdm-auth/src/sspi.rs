// SPDX-License-Identifier: RPL-1.5
//! SSPI client tokens (NTLM / Negotiate) with the logged-on user's default
//! credentials. Only for GDM's own proxy relay, which keeps every leg of the
//! handshake on one TCP connection. No password ever passes through here.

use windows::core::{HRESULT, PCWSTR};
use windows::Win32::Foundation::{SEC_E_OK, SEC_I_COMPLETE_AND_CONTINUE, SEC_I_COMPLETE_NEEDED, SEC_I_CONTINUE_NEEDED};
use windows::Win32::Security::Authentication::Identity::*;
use windows::Win32::Security::Credentials::SecHandle;

#[derive(Debug, PartialEq, Eq)]
pub enum Step {
    /// Send this token to the server and feed its answer to the next `step`.
    Continue(Vec<u8>),
    /// Handshake finished on this side; send the token if there is one.
    Done(Option<Vec<u8>>),
}

/// Credential + context handles shared by the client and the test server.
pub(crate) struct Handles {
    pub(crate) cred: SecHandle,
    pub(crate) ctx: SecHandle,
    pub(crate) has_ctx: bool,
}

impl Handles {
    pub(crate) fn acquire(package: &str, usage: SECPKG_CRED) -> Result<Self, String> {
        let pkg = wide(package);
        let mut cred = SecHandle::default();
        unsafe { AcquireCredentialsHandleW(PCWSTR::null(), PCWSTR(pkg.as_ptr()), usage, None, None, None, None, &mut cred, None) }
            .map_err(|e| format!("sspi-acquire-{:#x}", e.code().0))?;
        Ok(Self { cred, ctx: SecHandle::default(), has_ctx: false })
    }

    pub(crate) fn package(&self) -> Result<String, String> {
        if !self.has_ctx {
            return Err("no-context".into());
        }
        // NEGOTIATION_INFO names the real package under Negotiate; plain NTLM
        // contexts only answer PACKAGE_INFO.
        unsafe {
            let mut neg = SecPkgContext_NegotiationInfoW::default();
            let mut info = std::ptr::null_mut();
            if QueryContextAttributesW(&self.ctx, SECPKG_ATTR_NEGOTIATION_INFO, &mut neg as *mut _ as _).is_ok() {
                info = neg.PackageInfo;
            } else {
                let mut p = SecPkgContext_PackageInfoW::default();
                if QueryContextAttributesW(&self.ctx, SECPKG_ATTR_PACKAGE_INFO, &mut p as *mut _ as _).is_ok() {
                    info = p.PackageInfo;
                }
            }
            if info.is_null() {
                return Err("sspi-query-package".into());
            }
            let name = (*info).Name;
            let s = if name.is_null() { String::new() } else { PCWSTR(name).to_string().unwrap_or_default() };
            let _ = FreeContextBuffer(info as _);
            Ok(s)
        }
    }
}

impl Drop for Handles {
    fn drop(&mut self) {
        unsafe {
            if self.has_ctx {
                let _ = DeleteSecurityContext(&self.ctx);
            }
            let _ = FreeCredentialsHandle(&self.cred);
        }
    }
}

pub(crate) fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

/// Runs one ISC/ASC call: builds the in/out token buffers, copies the output
/// token, and maps the status to a `Step`.
pub(crate) fn run(
    h: &mut Handles,
    input: Option<&[u8]>,
    call: impl FnOnce(Option<*const SecHandle>, Option<*const SecBufferDesc>, *mut SecHandle, *mut SecBufferDesc) -> HRESULT,
) -> Result<Step, String> {
    let t = input.unwrap_or_default();
    let mut in_buf = SecBuffer {
        cbBuffer: u32::try_from(t.len()).map_err(|_| "token-too-big")?,
        BufferType: SECBUFFER_TOKEN,
        pvBuffer: t.as_ptr() as _, // SSPI only reads input tokens
    };
    let in_desc = SecBufferDesc { ulVersion: SECBUFFER_VERSION, cBuffers: 1, pBuffers: &mut in_buf };
    let mut out_buf = SecBuffer { cbBuffer: 0, BufferType: SECBUFFER_TOKEN, pvBuffer: std::ptr::null_mut() };
    let mut out_desc = SecBufferDesc { ulVersion: SECBUFFER_VERSION, cBuffers: 1, pBuffers: &mut out_buf };
    let old = h.has_ctx.then_some(&h.ctx as *const _);
    let mut st = call(old, input.map(|_| &in_desc as *const _), &mut h.ctx, &mut out_desc);
    if st == SEC_I_COMPLETE_NEEDED || st == SEC_I_COMPLETE_AND_CONTINUE {
        if unsafe { CompleteAuthToken(&h.ctx, &out_desc) }.is_err() {
            st = HRESULT(-1);
        } else {
            st = if st == SEC_I_COMPLETE_NEEDED { SEC_E_OK } else { SEC_I_CONTINUE_NEEDED };
        }
        h.has_ctx = true;
    }
    let token = if out_buf.pvBuffer.is_null() {
        Vec::new()
    } else {
        let v = unsafe { std::slice::from_raw_parts(out_buf.pvBuffer as *const u8, out_buf.cbBuffer as usize) }.to_vec();
        let _ = unsafe { FreeContextBuffer(out_buf.pvBuffer) };
        v
    };
    match st {
        SEC_I_CONTINUE_NEEDED => {
            h.has_ctx = true;
            Ok(Step::Continue(token))
        }
        SEC_E_OK => {
            h.has_ctx = true;
            Ok(Step::Done((!token.is_empty()).then_some(token)))
        }
        e => Err(format!("sspi-{:#010x}", e.0)),
    }
}

pub struct SspiClient {
    h: Handles,
    target: Vec<u16>,
    done: bool,
}

impl SspiClient {
    /// `package` is "Negotiate" or "NTLM"; `target_spn` e.g. "HTTP/proxy.corp.local".
    pub fn new(package: &str, target_spn: &str) -> Result<Self, String> {
        Ok(Self { h: Handles::acquire(package, SECPKG_CRED_OUTBOUND)?, target: wide(target_spn), done: false })
    }

    /// First call with `None`; then with each token the proxy sent back.
    pub fn step(&mut self, server_token: Option<&[u8]>) -> Result<Step, String> {
        if self.done {
            return Err("already-done".into());
        }
        if self.h.has_ctx != server_token.is_some() {
            return Err("bad-step".into());
        }
        let (cred, target) = (self.h.cred, self.target.as_ptr());
        let step = run(&mut self.h, server_token, |old, input, new, out| {
            let mut attrs = 0u32;
            let flags = ISC_REQ_ALLOCATE_MEMORY | ISC_REQ_CONNECTION;
            unsafe {
                InitializeSecurityContextW(
                    Some(&cred),
                    old,
                    Some(target),
                    flags,
                    0,
                    SECURITY_NATIVE_DREP,
                    input,
                    0,
                    Some(new),
                    Some(out),
                    &mut attrs,
                    None,
                )
            }
        });
        self.done = matches!(step, Ok(Step::Done(_)) | Err(_));
        step
    }

    /// Package the context actually uses ("NTLM", "Kerberos", ...), so a
    /// Kerberos-only mode can refuse an NTLM fallback. Valid after the first step.
    pub fn package_used(&self) -> String {
        self.h.package().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test-only server side (AcceptSecurityContext, current user).
    struct Server(Handles);

    impl Server {
        fn step(&mut self, token: &[u8]) -> Result<Step, String> {
            let cred = self.0.cred;
            run(&mut self.0, Some(token), |old, input, new, out| {
                let mut attrs = 0u32;
                unsafe {
                    AcceptSecurityContext(
                        Some(&cred),
                        old,
                        input,
                        ASC_REQ_ALLOCATE_MEMORY | ASC_REQ_CONNECTION,
                        SECURITY_NATIVE_DREP,
                        Some(new),
                        Some(out),
                        &mut attrs,
                        None,
                    )
                }
            })
        }
    }

    /// Runs the exchange to the end; returns legs (tokens sent) and both package names.
    fn handshake(package: &str, target: &str) -> Result<(usize, String, String), String> {
        let mut c = SspiClient::new(package, target)?;
        let mut s = Server(Handles::acquire(package, SECPKG_CRED_INBOUND)?);
        let mut legs = 0;
        let mut to_server = match c.step(None)? {
            Step::Continue(t) => t,
            Step::Done(_) => return Err("client done at once".into()),
        };
        loop {
            legs += 1;
            match s.step(&to_server)? {
                Step::Done(None) => break,
                Step::Done(Some(_)) => {
                    legs += 1; // mutual-auth token back to the client
                    break;
                }
                Step::Continue(t) => {
                    legs += 1;
                    match c.step(Some(&t))? {
                        Step::Continue(t) | Step::Done(Some(t)) => to_server = t,
                        Step::Done(None) => break,
                    }
                }
            }
        }
        Ok((legs, c.package_used(), s.0.package()?))
    }

    #[test]
    fn ntlm_loopback_three_legs() {
        let (legs, client_pkg, server_pkg) = handshake("NTLM", "HTTP/localhost").unwrap();
        assert_eq!(legs, 3);
        assert_eq!(client_pkg, "NTLM");
        assert_eq!(server_pkg, "NTLM");
    }

    #[test]
    fn negotiate_loopback_succeeds_or_fails_cleanly() {
        match handshake("Negotiate", "HTTP/localhost") {
            Ok((legs, c, s)) => println!("Negotiate OK: {legs} legs, client={c}, server={s}"),
            Err(e) => println!("Negotiate Err: {e}"),
        }
    }

    #[test]
    fn misuse_is_refused() {
        let mut c = SspiClient::new("NTLM", "HTTP/x").unwrap();
        assert_eq!(c.step(Some(b"x")), Err("bad-step".into()));
        assert!(matches!(c.step(None), Ok(Step::Continue(_))));
        assert_eq!(c.step(None), Err("bad-step".into()));
        assert!(c.step(Some(b"garbage")).is_err());
        assert_eq!(c.step(Some(b"x")), Err("already-done".into()));
        assert!(SspiClient::new("NoSuchPackage", "HTTP/x").is_err());
    }
}
