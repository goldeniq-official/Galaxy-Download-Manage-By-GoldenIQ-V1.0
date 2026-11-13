**(1) is sound at the transport layer, but I would not close it yet.** The weakest remaining points are plain HTTP forwarding, PAC fidelity, authentication scope, and crash consistency. This is a design review; I have not verified the implementation or your reported tests.

I concede the corrected M-02 mitigation addresses the specific CVE: the upstream advisory identifies **2024.07.01** as the patched version. Keep that as a security floor, pin an exact reviewed build, and explicitly disable external configuration/plugin loading; an empty working directory alone does not exclude user configuration. I also concede that withholding credentials from yt-dlp and making GDM fetch streams substantially narrows M-08. Traffic capture establishes observed behavior, however, rather than enforcing containment. [yt-dlp advisory](https://github.com/yt-dlp/yt-dlp/security/advisories/GHSA-79w7-vh3h-8g4j)

**1. Relay: approve the principle, tighten the contract.**

- **IP CONNECT with hostname TLS is valid.** The upstream opens a byte tunnel to the validated IP; reqwest retains the original URL hostname for SNI and certificate verification. Use `[IPv6]:port` authority syntax. SOCKS5 explicitly supports IPv4 and IPv6 address types, so domain-free requests are also valid. Test certificate mismatch rejection through both routes, not just the recorded CONNECT authority. [HTTP CONNECT](https://www.rfc-editor.org/rfc/rfc9110.html#section-9.3.6), [SOCKS5 addressing](https://www.rfc-editor.org/rfc/rfc1928.html#section-5)
- **Plain HTTP is the largest unspecified branch.** If you forward `GET http://hostname/path` to the upstream HTTP proxy, you have handed name resolution back to it. Specify that the relay first CONNECTs to the validated IP even for HTTP, then sends an origin-form request with the original `Host`. Some proxies will refuse CONNECT to port 80; record that as unsupported compatibility, rather than silently reverting to ordinary forwarding.
- **PAC per host is a deliberate compatibility restriction.** Microsoft explicitly requires per-URL evaluation because results can differ by URL. A CONNECT relay cannot see the encrypted path, so `https://host/` cannot reproduce arbitrary URL-dependent PAC behavior. Include nondefault ports in the synthetic URL. For engine requests, evaluate the actual URL before connecting and partition tunnel reuse by selected route; for yt-dlp, document the available authority-only behavior. [WinHttpGetProxyForUrl](https://learn.microsoft.com/windows/win32/api/winhttp/nf-winhttp-winhttpgetproxyforurl)
- Validate every candidate IP actually attempted, including retries, redirects and address-family fallback. Strip the local relay credential before upstream forwarding; upstream credentials must be separately scoped. Reject malformed authorities and impose an explicit CONNECT port policy. RFC 9110 specifically warns about unrestricted tunnels to services such as SMTP. [CONNECT security considerations](https://www.rfc-editor.org/rfc/rfc9110.html#section-9.3.6)

“Proxy refused IP” also overstates what a generic 403 proves. Prefer “proxy refused connection to the validated IP.” The less-safe setting should explicitly say it disables GDM’s destination-IP guarantee, and should never activate automatically.

**2. Authentication: your fallback is defensible; the reuse premise is not.**

`pool_max_idle_per_host=1` limits **idle** connections. It does not reserve a socket or guarantee authentication affinity. A successful fixture is useful regression evidence, but cannot establish a guarantee under server closes, incomplete response bodies or reconnects. For NTLM, own one HTTP/1.1 connection and restart authentication from the beginning whenever it changes—or ship the stated unsupported outcome. [reqwest documentation](https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html#method.pool_max_idle_per_host)

Origin authentication **can traverse the relay** inside TLS; the relay simply cannot perform it. Also, `Negotiate` does not mean “Kerberos only.” Check the selected SSPI package and reject NTLM fallback when operating in Kerberos-only mode. Bind origin credentials to the intended hostname/SPN and explicit authorization; a hostile server’s 401 must not automatically solicit ambient Windows credentials. Process final authentication tokens, including those returned with successful responses. [RFC 4559](https://www.rfc-editor.org/info/rfc4559/)

I reject “Digest ≈80 lines” as a planning assumption. The hash formula is small; challenge parsing, quoted commas, nonce counts under concurrency, stale nonces, algorithm selection, request-target construction and credential scope are the work. A deliberately limited MD5/SHA-256, `qop=auth` implementation is reasonable if unsupported variants fail explicitly and published vectors pass. [RFC 7616](https://www.rfc-editor.org/rfc/rfc7616.html)

**6. Pipe verification: useful provenance, not user intent.**

A different executable does not pass merely by changing its command-line image name, assuming you query the process through the OS. Exact executable comparison is stronger than “inside the install directory.” Nevertheless:

- Another process can launch the genuine installed `gdm.exe` with attacker-selected CLI arguments. Your check will correctly authenticate GDM, while proving nothing about who requested the download. Therefore `origin="cli"` must remain a restricted input category, not privileged consent.
- Writable installation files or directories undermine path-based executable trust. Canonical path equality does not establish trusted contents.
- Explicitly reject remote pipe clients. Use restricted client impersonation quality of service so a squatting pipe server cannot impersonate the client before verification. Windows documents both remote named-pipe access and client control of impersonation level. [Named pipes](https://learn.microsoft.com/windows/win32/ipc/named-pipes), [impersonation controls](https://learn.microsoft.com/en-us/windows/win32/ipc/impersonating-a-named-pipe-client)

I accept excluding same-user injection from this threat model. You do not need injection to demonstrate the genuine-CLI-launch limitation.

Your held-directory-handle approach is strong, assuming child creation really remains relative to it. Extend argv detection to `/p`, `/q`, `/h` and other supported switches; handle `--background` explicitly. Bind `/q` to the server-created job and a newly created background-instance ownership token, so simultaneous CLI launches cannot accidentally claim an existing instance.

**Changes I would make in 3–7:**

- **Scheduler:** put durable occurrence claims alongside queue/job state in SQLite. A settings claim and SQLite enqueue cannot commit atomically: claim first can lose a run; enqueue first can duplicate it. Define overnight windows, clock rollback, timezone changes and schedule-edit identity explicitly. I accept your stated DST coverage, but cannot independently confirm it.
- **Finish actions:** the final recheck is necessary, but closing the lock before the power call leaves a race. Commit a “finish action executing” state under the lock that prevents new work from starting until cancellation or call completion. Count hooks and postprocessing if “finished” promises those are complete. Your recorder trait and visible cancellation are good choices.
- **Hook:** persist the consumed event **before** spawning to obtain at-most-once launch, and acknowledge that a crash can then lose the hook. For scripts, “final expanded argv” needs a precise definition: `cmd.exe` performs further parsing and environment expansion. Prefer passing downloaded metadata through a data file; otherwise reject shell expansion syntax and invoke a trusted absolute `cmd.exe` with AutoRun and delayed expansion disabled. Window-label authorization also does not protect against compromise of that authorized renderer. [cmd parsing and switches](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cmd)
- **Clipboard:** check exclusion formats and read text while holding the same `OpenClipboard` session. Reading the `CanIncludeInClipboardHistory` DWORD necessarily requires reading that metadata; the promise should be “before reading text.” Bound allocation size before scanning for termination, and match YouTube by parsed hostname boundary. Your privacy interpretation is conservative and reasonable; Microsoft defines those markers for history/sync rather than universal monitoring consent. [Clipboard formats](https://learn.microsoft.com/en-gb/windows/desktop/dataxchg/clipboard-formats)

I fully concede (8): explicit prerequisites and `BLOCKED-ON-OWNER` are the correct treatment of unavailable evidence.

Approve IP-based tunneling, conditional on explicit plain-HTTP routing and PAC limitations.  
Require owned connections for NTLM, credential scoping, and Digest conformance tests.  
Keep CLI provenance checks, but treat genuine CLI invocation as untrusted input.  
Make scheduler claims atomic; close finish-action races and document lost-hook semantics.  
Proceed with the tester kit and retain BLOCKED labels until prerequisite evidence arrives.