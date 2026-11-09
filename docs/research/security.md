# Threat model research: download manager (Rust/Tauri v2 + React, browser extension, local engine)

Date: 2026-11-09. Web research, defensive only.
Legend: **[V]** = checked against a search result / advisory page in this session. **[M]** = from prior knowledge, not re-checked this session; verify before citing. **[U]** = found only on aggregators or conflicting; treat as unverified.

---

## Part 1: Real incidents and CVEs in download managers and similar tools

| # | Product / CVE | Root cause | Lesson for us |
|---|---|---|---|
| 1 | **Free Download Manager (Linux), 2020-2022, disclosed Sept 2023** [V] | FDM's site had a vulnerable script; attackers changed the download page so it *sometimes* redirected Linux users to `deb.fdmpkg[.]org`. The .deb carried a malicious `postinst` that dropped a Bew-family backdoor (`crond`) and a credential stealer. It ran for about 3 years unnoticed. | Distribution is part of the attack surface. Sign releases, publish hashes and provenance (SLSA), serve downloads only from a pinned origin (GitHub Releases), make the website static, and watch for "selective" redirects (targeted at some users, so most people never see them). https://securelist.com/backdoored-free-download-manager-linux-malware/110465/ (Kaspersky primary, [M] URL), https://www.bleepingcomputer.com/news/security/free-download-manager-site-redirected-linux-users-to-malware-for-years/ , https://securityaffairs.com/150851/malware/free-download-manager-supply-chain-attack.html |
| 2 | **FDM CVE-2014-2087** [U] | Old FDM bug (listed by Fortinet; details not checked). | Old native parsers get bugs. Use a memory-safe engine (Rust) for parsing. https://fortiguard.fortinet.com/encyclopedia/endpoint-vuln/2177 |
| 3 | **IDM CVE-2025-56231** [U] (CWE-295, CVSS 9.1 per aggregator) | Updater does not validate the TLS certificate, so a network attacker can serve a fake update. | The update channel must use TLS **and** a signature check that does not depend on TLS. https://www.wiz.io/vulnerability-database/cve/cve-2025-56231 |
| 4 | **IDM CVE-2026-90493** [U] (kernel driver `idmwfp.sys`, vendor did not respond) | Bug in a WFP kernel driver used to catch browser traffic. | Never ship a kernel driver to catch downloads. A browser extension plus native messaging gives the same result with far less risk. https://app.opencve.io/cve/CVE-2026-90493 |
| 5 | **IDM CVE-2020-28964 / CVE-2020-23060 / CVE-2020-37234; CVE-2010-0995; CVE-2005-2210** [U] | Stack buffer overflows (Search box, Scheduler field, a long URL, FTP). | Classic C/C++ memory bugs. Rust removes most of them. Still put length limits on every URL, header and filename (CWE-120/121). https://app.opencve.io/cve/CVE-2020-28964 , https://vuldb.com/cve/CVE-2005-2210 , https://fortiguard.fortinet.com/encyclopedia/endpoint-vuln/2120 |
| 6 | **aria2 / Motrix RPC exposure** [V partial] | aria2 JSON-RPC (default port 6800; Motrix uses 16800 [M]) can `aria2.changeGlobalOption` / `addUri` with `dir`, and options such as `--on-download-complete` run a program. With no `rpc-secret`, `--rpc-listen-all` or a forwarded port, anyone who can reach the port can write files anywhere and can often run code. `--rpc-allow-origin-all` adds `Access-Control-Allow-Origin: *`, so any web page can read the replies. I found **no aria2-specific DNS rebinding CVE** (the risk is inferred from the same pattern in Transmission). Motrix release notes say it binds to loopback and turned off permissive CORS, but it still lets the user set an empty secret on purpose. | A local engine API is a remote attack surface: any website the user opens can send it requests. Use a random per-install secret, bind to loopback only, check `Host`, never send `ACAO: *`, and never let the API change hook commands or the download root. https://salivity.github.io/aria2/article/what-are-the-risks-of-exposing-aria2-rpc-to-the-internet , https://newreleases.io/project/github/agalwood/Motrix/release/v2.0.0-beta.28 , https://aria2.github.io/manual/en/html/aria2c.html#rpc-interface [M] |
| 7 | **Transmission CVE-2018-5702** [V] | DNS rebinding against localhost RPC: the server did not check `Host`, so the attacker's domain resolved to 127.0.0.1 and POST to `/transmission/rpc` worked (a session ID was not enough). | Closest example to our local API. Check that `Host` is in an allowlist (`127.0.0.1:port`, `localhost:port`). https://www.exploit-db.com/exploits/43665 , https://access.redhat.com/security/cve/CVE-2018-5702 |
| 8 | **qBittorrent CVE-2024-51774** [V] (CWE-295, NVD 8.1; fixed in 5.0.1) | `DownloadManager` called `ignoreSslErrors` on every download (torrents, RSS, search plugins, the Python installer on Windows). Maintainers first called it theoretical. | Never turn off certificate checks anywhere in code, not even "for one CDN". Add a CI check that fails the build on `danger_accept_invalid_certs`. https://access.redhat.com/security/cve/CVE-2024-51774 , https://www.wiz.io/vulnerability-database/cve/cve-2024-51774 |
| 9 | **qBittorrent WebUI** (several older CSRF/XSS and auth-bypass bugs) [M] | A web UI on a LAN port, with weak defaults (local auth bypass on by default). | Do not ship a web UI unless it is needed. If one exists: off by default, loopback only, auth required. |
| 10 | **yt-dlp CVE-2024-38519 / GHSA-79w7-vh3h-8g4j** [V] (CVSS 7.8, fixed 2024.07.01) | The file extension came from the server and was not checked, so attackers could create arbitrary files (path traversal on Windows), for example drop a `yt-dlp.conf` with `--exec`, which leads to RCE. Same bug in youtube-dl (GHSA-22fp-mf44-f2mq). | Never trust a server-supplied **extension** either, not only the name. Keep an allowlist of extensions. Never write files the app itself reads (config, plugins, DLLs) into the download folder. https://advisories.gitlab.com/pypi/yt-dlp/CVE-2024-38519/ , https://db.gcve.eu/vuln/ghsa-79w7-vh3h-8g4j |
| 11 | **yt-dlp CVE-2023-40581 → CVE-2024-22423 → CVE-2026-26331** [V/U] | `--exec` built a shell command from metadata (title etc.). The escaping for Windows `cmd` was not enough (`"`→`""`, then `%` env expansion, then the default `{}` placeholder). | Escaping for `cmd.exe` keeps failing. **Never pass downloaded data through a shell.** Use `std::process::Command` with an argv array, and on Windows never call `.bat`/`.cmd` (see the Rust BatBadBut issue CVE-2024-24576 [M]). https://feedly.com/cve/vendors/yt-dlp_project , https://www.cvelogic.com/vendor/yt-dlp_project |
| 12 | **yt-dlp CVE-2023-35934 / GHSA-v8mc-9377-rwjj** [V] (fixed 2023.07.06) | Cookies were sent to other hosts after a redirect, and to fragment hosts that differ from the manifest host. | Scope cookies per request URL with a real cookie jar (RFC 6265). Remove `Cookie`/`Authorization` when the origin changes. https://www.suse.com/security/cve/CVE-2023-35934 , https://access.redhat.com/security/cve/CVE-2023-35934 |
| 13 | **GNU wget CVE-2016-4971** [V] | After an HTTP→FTP redirect, wget used the name **the redirect target** gave, so an attacker could write any file (for example `.bash_profile`). | Take the filename from the user's **original** URL or a validated Content-Disposition, never from the redirect target alone. Never follow redirects to another scheme (http→ftp/file). https://nvd.nist.gov/vuln/detail/CVE-2016-4971 |
| 14 | **GNU wget CVE-2024-38428** [V] (≤1.24.5) | A `;` in userinfo made wget parse the host wrongly (`http://us;er@host`). | Use one strict URL parser (WHATWG `url` crate), and use the *same* parse result for the policy check and for the connection. Reject userinfo in URLs from the browser. https://jfrog.com/blog/cve-2024-38428-wget-vuln-all-you-need-to-know/ |
| 15 | **curl CVE-2018-1000007 & CVE-2022-27776** [V] | `Authorization` / custom auth headers were kept on a redirect to another host, then (the 2022 fix) to the same host on another port or scheme. | Origin = scheme + host + port. Remove credentials whenever the origin changes. The `reqwest` crate removes them on cross-host redirects [M]; test that it does, including port and scheme changes. https://curl.se/docs/CVE-2022-27776.html , https://osv.dev/vulnerability/CURL-CVE-2018-1000007 |
| 16 | **curl CVE-2023-38545** (SOCKS5 heap overflow with a long hostname) [M] | A buffer overflow in proxy code. | Proxy code is attack surface too. Keep dependencies updated (cargo-audit). https://curl.se/docs/CVE-2023-38545.html |
| 17 | **JDownloader 2010 Click'n'Load RCE (`JDExternInterface`)**; **2015 zip-slip in auto-extract** [V] | A local HTTP listener (port 9666) that any web page could call; the extractor followed `../` entries. | The browser to local app bridge needs authentication. Auto-extract needs zip-slip protection. https://www.exploit-db.com/exploits/33615 , https://seclists.org/bugtraq/2010/Feb/89 |
| 18 | **XDM, uGet** [V: none found] | No CVEs found in NVD-indexed aggregators (CVE-2006-2964 "Xtreme Downloads" is a different PHP product). | No CVE does not mean safe. Still, these are useful as a feature reference only. |
| 19 | **Tauri CVE-2023-31134 / GHSA-4wm2-cwcf-wwvp** [V] (1.x) | An open redirect moved a window to an external site, which then had IPC access (isolation bypass). | Block navigation to anything except the app origin (`on_navigation`), and never give capabilities to remote URLs. https://www.cvelogic.com/cve/CVE-2023-31134 |
| 20 | **Tauri CVE-2024-35222 / GHSA-57fm-592m-34r7** [V] (1.6.7, 2.0.0-beta.20) | Iframes could call IPC even in isolation mode. Fixed with `__TAURI_INVOKE_KEY__`. | Use Tauri ≥ the patched 2.x. Set CSP `frame-src 'none'`. Never show remote content (site previews) in an iframe inside the main window. https://vulnerability.cert.dk/vuln/cve-2024-35222 |
| 21 | **tauri-plugin-shell CVE-2025-31477** [V] (<2.2.1, CWE-20) | The `open` scope check did not work, so `file://`, `smb://` and `nfs://` reached the OS handler, which leads to RCE if JS is compromised. | "Open file" and "Open folder" must not take a raw string from JS. Use the `opener` plugin with a fixed path scope, or a Rust command that takes a **download ID** and resolves the path itself. https://nvd.nist.gov/vuln/detail/CVE-2025-31477 , https://security.snyk.io/vuln/SNYK-RUST-TAURIPLUGINSHELL-9697751 |
| 22 | **Tauri fs scope glob GHSA-q9wv-22m9-vhqh** [V] | Glob patterns in the scope allowed more paths than intended (Windows bypass). | Keep fs/asset scope as small as possible. Prefer Rust commands over the generic fs plugin. https://github.com/tauri-apps/tauri/security/advisories/GHSA-q9wv-22m9-vhqh |
| 23 | **Tauri asset protocol wildcard** (misconfiguration, not a CVE) [V] | `assetProtocol.scope: ["**"]` lets any script in the renderer read any file. | Turn the asset protocol off, or scope it to a thumbnails cache only. https://snyk.io/articles/5-common-security-misconfigurations |
| 24 | **Electron**: CVE-2018-1000136 (`<webview>` re-enables nodeIntegration), CVE-2022-29247 (contextIsolation bypass via IPC) [M] | The renderer got more privilege than intended. | The same lesson applies to Tauri: treat the WebView as untrusted, and authorise in Rust. https://www.electronjs.org/docs/latest/tutorial/security |
| 25 | **7-Zip CVE-2025-0411** [V] (fixed 24.09, CISA KEV) | MOTW (`Zone.Identifier`) was not copied to files from nested archives, so SmartScreen and Office Protected View did not run. | If we extract archives, copy MOTW to every extracted file. Our own downloads must write MOTW. https://nvd.nist.gov/vuln/detail/CVE-2025-0411 |
| 26 | **Windows MOTW bypass CVE-2022-41091 / CVE-2022-44698** [M] | Windows itself missed MOTW on some file types. | Write MOTW **and** also scan with `IAttachmentExecute` (it calls AV and SmartScreen). |
| 27 | **Browser extension takeovers**: The Great Suspender (sold in 2020, pulled in 2021) [V]; Cyberhaven + ≥16 other extensions (Dec 2024, a phished developer's Chrome Web Store account pushed a malicious update that stole cookies and tokens) [V] | Store credentials or ownership changed hands, and auto-update sent the malware to every user. | Turn on 2FA/hardware keys on the CWS/AMO accounts, keep publish rights to one person, publish from CI with reviewed source, give the extension the fewest permissions, and **the native host must not trust the extension blindly** (it should check messages as if they could come from an attacker). https://www.bleepingcomputer.com/news/security/the-great-suspender-chrome-extensions-fall-from-grace/ , https://securityaffairs.com/172491/hacking/chrome-browser-extensions-compromise.html , https://www.securityweek.com/cyberhaven-chrome-extension-hack-linked-to-widening-supply-chain-campaign/ |

---

## Part 2 and 3: Attack classes, mitigations, standards

Standards key: ASVS 5.0 chapters [V]: V1 Encoding & Sanitization, V3 Web Frontend Security, V5 File Handling (5.1 docs, 5.2 upload/content, 5.3 storage, 5.4 download), V12 Secure Communication (12.1 general TLS, 12.2 external HTTPS, 12.3 service-to-service), V13 Configuration, V15 Secure Coding & Architecture, V14 Data Protection, V11 Cryptography [M for chapters other than V3/V5/V12; check https://github.com/OWASP/ASVS/tree/v5.0.0]. OWASP Top 10 2025 [V]: A01 Broken Access Control (now includes SSRF), A02 Security Misconfiguration, A03 Software Supply Chain Failures, A04 Cryptographic Failures, A05 Injection, A06 Insecure Design, A07 Authentication Failures, A08 Software or Data Integrity Failures, A09 Logging & Alerting Failures, A10 Mishandling of Exceptional Conditions (the order of A02/A04/A07-A09 is [M]; https://owasp.org/Top10/2025/).

### A. Filenames and paths

**A1. Path traversal from Content-Disposition / URL / redirect / server extension**
- Attack: `filename="../../AppData/Roaming/Microsoft/Windows/Start Menu/Programs/Startup/x.exe"`, `filename*=UTF-8''..%2F..%2F`, absolute `C:\`, UNC `\\host\share`, `\\?\` and `\\.\` device paths, `/` and `\` mixed, drive-relative `C:x`.
- Mitigations: parse Content-Disposition according to RFC 6266 (and RFC 8187 for `filename*`), and **take only the final path segment** (RFC 6266 §4.3: recipients must not use directory info) [V-knowledge M]. Then sanitise: remove `/ \ : * ? " < > |`, control characters U+0000-U+001F and U+007F, and trailing dots and spaces. Join to the user-chosen dir, then **canonicalise and check that the parent is the download root** (do not rely on string prefix checks). Never use the name from the redirect target alone (wget CVE-2016-4971).
- Standards: CWE-22, CWE-23, CWE-36, CWE-73; ASVS V5.3 (file storage: user-supplied filenames are validated/ignored), V5.4; Top 10 A01; RFC 6266, RFC 8187.

**A2. Extension spoofing: RTLO, homoglyphs, double extension, extension from server**
- Attack: `invoice\u202Efdp.exe` shows as `invoiceexe.pdf`; `report.pdf.exe` with hidden extensions; Cyrillic `а`; the server picks `.exe`/`.lnk`/`.url`/`.hta`/`.scr`/`.conf` (yt-dlp CVE-2024-38519).
- Mitigations: remove bidi controls (U+202A-202E, U+2066-2069, U+200E/200F) and zero-width characters (U+200B-200D, U+FEFF); NFC-normalise. In the UI show the **real extension** in its own column. Mark dangerous types (a list similar to Windows "Unsafe" / Chromium's `download_file_types.asciipb` [M]) with a warning and **never auto-open** them. If the extension from the URL, Content-Type and Content-Disposition disagree, flag it.
- Standards: CWE-451 (UI misrepresentation), CWE-176 (Unicode encoding), CWE-434-style unrestricted types; ASVS V5.2/V5.4; UTS #39 (confusables) [M].

**A3. Windows reserved names and Alternate Data Streams**
- Attack: `CON`, `PRN`, `AUX`, `NUL`, `COM0-9`, `LPT0-9` (also with an extension, `nul.txt`, and the superscript variants `COM¹²³`, `LPT¹²³` [M]), `CONIN$`/`CONOUT$`; `file.txt:evil.exe` or `file:Zone.Identifier` writes an ADS (it can overwrite MOTW!); `::$DATA`; 8.3 short names (`PROGRA~1`).
- Mitigations: since `:` is already removed in A1, ADS is blocked; prefix reserved stems with `_`. Tests must include every item above.
- Standards: CWE-67 (Windows device names), CWE-69 (ADS), CWE-41 (path equivalence).

**A4. Long paths**
- Attack: names longer than 255 UTF-16 units or paths longer than MAX_PATH 260 cause truncation, so the check passes on one name and the write lands on another; or the app crashes (A10).
- Mitigations: cap the component at 200 UTF-16 units (cut at a grapheme boundary, keep the extension); use `\\?\` verbatim paths inside the engine only after canonicalising; handle errors without crashing.
- Standards: CWE-170 / CWE-130, Top 10 2025 A10.

**A5. Symlink/junction races in temp/part dir, TOCTOU on rename**
- Attack: another local process (or a planted junction in a world-writable dir) swaps `file.part` or its parent dir for a symlink/junction to `C:\Windows\...` or another user's profile between the check and the write/rename. On Windows, `MoveFileEx` follows a junction in the parent path.
- Mitigations: keep `.part` files **in the destination dir** (same volume, user-owned), never in `%TEMP%` shared paths. Create with `create_new(true)` (O_EXCL / `CREATE_NEW`); on Windows open with `FILE_FLAG_OPEN_REPARSE_POINT` and reject reparse points; keep the handle open and finish with **handle-based rename** (`SetFileInformationByHandle(FileRenameInfoEx)` with `FILE_RENAME_FLAG_POSIX_SEMANTICS`) [M], and do not re-resolve the path. Never rename over an existing file without asking (no silent overwrite). The engine runs **unelevated**, so it is never a privilege-escalation target.
- Standards: CWE-59 (link following), CWE-367 (TOCTOU), CWE-377 (insecure temp file), CWE-379.

**A6. Zip-slip and archive bombs if auto-extract exists**
- Attack: entries with `../`, absolute paths, symlink entries pointing outside, then a second entry written through the link (JDownloader 2015); 42.zip-style ratio bombs; no MOTW on extracted files (7-Zip CVE-2025-0411).
- Mitigations: **delete the feature first** (Algorithm step 2: let the OS or 7-Zip extract). If it is kept: apply A1 sanitising to every entry, refuse symlink/hardlink entries, cap total size, file count and ratio (for example 100:1), extract into a fresh dir, and copy MOTW to every output.
- Standards: CWE-22, CWE-409 (decompression bomb), CWE-59; ASVS V5.2 (archive checks: max uncompressed size, file count) [M].

**A7. Mark-of-the-Web**
- Attack: a file written without `Zone.Identifier` skips SmartScreen, Office Protected View and the "blocked" check on macros.
- Mitigations: on Windows write `:Zone.Identifier` with `[ZoneTransfer] ZoneId=3`, plus `ReferrerUrl`/`HostUrl` (strip credentials and query tokens for privacy). Better: call `IAttachmentExecute::Save()` with the source URL. It writes MOTW, calls the registered AV scanner and applies policy, which is what browsers do [M]. Do it **before** the final rename is visible. On macOS set `com.apple.quarantine` (if ever ported); on Linux there is no equivalent (only optional `user.xdg.origin.url`).
- Standards: Microsoft SDL; https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-iattachmentexecute [M]; CWE-693.

**A8. Auto-open after download**
- Attack: a drive-by download becomes execution with one click (or none).
- Mitigations: off by default. Never auto-open types on the dangerous list. "Open" goes through `ShellExecute` on a **path resolved by Rust from the download ID**, never a string from the WebView (tauri-plugin-shell CVE-2025-31477). Never open a `file://`/`smb://`/custom-scheme URL.
- Standards: CWE-494, CWE-829; Top 10 A06.

### B. Network

**B1. SSRF / private-IP redirect / DNS rebinding to local services**
- Attack: the extension (or a malicious page that triggers the extension) sends `http://192.168.1.1/cgi-bin/...`, `http://169.254.169.254/` or `http://127.0.0.1:<engine-port>/`, or a public host that redirects there, or DNS that rebinds to 127.0.0.1 after the first check. The downloader becomes a confused deputy that hits the router or the engine's own API.
- Mitigations: by default, block loopback, RFC 1918, link-local, CGNAT 100.64/10, ULA fc00::/7, `::ffff:` mapped addresses, 0.0.0.0 and multicast, **for every hop of a redirect**. Resolve DNS once, check the IP, then **connect to that IP** (custom `reqwest` resolver / `connect_to`) so a rebind cannot change it. Show a confirm dialog when the user really wants a LAN download. Allow only the `http`/`https`(/`ftp`?) schemes; no `file:`, `data:`, `gopher:`. Limit redirect hops to 10 or fewer, and do not allow https→http downgrade.
- Standards: CWE-918 (SSRF), CWE-350, CWE-601; Top 10 2025 A01 (SSRF merged); ASVS V1.3 / V15 SSRF requirements [M]; OWASP SSRF Prevention Cheat Sheet https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html

**B2. Credentials and cookies forwarded to other hosts on redirect / segment hosts**
- Attack: the extension hands over browser cookies and `Authorization`; a redirect or a CDN segment host receives them (yt-dlp CVE-2023-35934, curl CVE-2018-1000007 / CVE-2022-27776).
- Mitigations: keep cookies in a per-download RFC 6265 jar scoped by domain/path/secure, never as a raw `Cookie:` header copied to every request. Remove `Authorization`, `Proxy-Authorization`, `Cookie` and custom auth headers when the **origin (scheme, host, port)** changes. Never send `Secure` cookies over http. Ask the extension only for the cookies of that URL (`chrome.cookies.getAll({url})`), at the moment of the download, and do not store them on disk.
- Standards: CWE-200, CWE-522, CWE-359; RFC 6265; RFC 9110 §15.4 (redirects) [M]; ASVS V12.

**B3. Proxy and site credential storage**
- Attack: plaintext passwords in `settings.json` / SQLite are read by malware or appear in backups and bug reports.
- Mitigations: use the OS keychain (`keyring` crate: Windows Credential Manager, which uses DPAPI under the hood; macOS Keychain; Secret Service). If using DPAPI directly, `CryptProtectData` with optional entropy is user-scoped; it protects against other users and offline disk theft, **not** against malware running as the same user (document this). Never log credentials; redact `user:pass@` from URLs in logs and in the UI.
- Standards: CWE-256, CWE-312, CWE-532 (sensitive info in logs); ASVS V13 (secrets management) / V14 [M]; Top 10 A04.

**B4. TLS downgrade / disabled cert checks**
- Mitigations: use `rustls` with the platform verifier (`rustls-platform-verifier`) [M] so that corporate CAs work without a "skip check" button. **No per-download "ignore certificate" option**, or if the owner insists, make it per-host, shown in red, and never usable by the updater or the extension bridge. Allow TLS 1.2+ only. No https→http redirect without asking the user. Use a CI grep that fails on `danger_accept_invalid_certs` / `danger_accept_invalid_hostnames` (lesson from qBittorrent CVE-2024-51774 and IDM CVE-2025-56231).
- Standards: CWE-295, CWE-757; ASVS V12.1/12.2; Top 10 A04; RFC 8996 (deprecates TLS 1.0/1.1) [M].

**B5. HTTP response splitting / header injection**
- Attack: a URL, cookie or referrer supplied by the extension contains CR/LF, so it injects request headers, or a server header with CR/LF breaks our own parsing or logs.
- Mitigations: build headers with `http::HeaderValue`, which rejects CR, LF and NUL [M]. Reject URLs with control characters before parsing. Do not hand-write HTTP/1.1. Treat server headers as untrusted when displayed (log injection).
- Standards: CWE-113, CWE-93, CWE-117 (log neutralisation); RFC 9110 §5.5 (field values: CR/LF/NUL invalid), RFC 9112 (message framing, Content-Length vs Transfer-Encoding) [M].

**B6. Decompression bombs (Content-Encoding)**
- Attack: `Content-Encoding: gzip`/`br` with a 1000:1 ratio, or nested encodings.
- Mitigations: for file downloads send `Accept-Encoding: identity` (we want the bytes as sent; this also keeps Range offsets correct). If decoding is ever on (site grabber HTML), cap output bytes and ratio.
- Standards: CWE-409, CWE-400.

**B7. Disk fill / infinite stream / slowloris server**
- Attack: no `Content-Length` and an endless body; `Content-Length` lies; very many tiny segments; very slow trickle that holds connections.
- Mitigations: preallocate only up to the declared size after a free-space check (keep a reserve, for example 1 GB or 5%); stop when bytes received exceed the declared length; user-set max size for unknown-length downloads (default cap with a prompt); idle and total timeouts; cap connections per host and in total; cap redirect count and header size.
- Standards: CWE-400, CWE-770, CWE-1284; Top 10 2025 A10.

**B8. Wrong Range responses and content changing between segments (integrity)**
- Attack and bug cases: the server answers `200` to a Range request (the whole file arrives in each segment); `206` with a `Content-Range` that is not the range asked for; `416`; a multipart/byteranges reply; the file changes on the server between segments or on resume, so the result is a silent mix of two versions; a CDN edge with a different ETag.
- Mitigations (RFC 9110 §14 and §13.1.5) [M section numbers]:
  - Accept segment data only if the status is `206` **and** `Content-Range` equals exactly the requested `bytes a-b/total` **and** `total` equals the size first seen. If the reply is `200`, fall back to a single connection (or stop) and never write the body at that offset.
  - Store the strong `ETag` (or `Last-Modified` if no ETag) from the first reply; send `If-Range: <etag>` on every segment and resume. Under If-Range, if the server answers `200`, the file changed, so restart. Weak ETags (`W/`) must not be used with If-Range. Without either validator, do not resume over a long gap without asking.
  - Optional: `Want-Repr-Digest` / `Repr-Digest` (RFC 9530) when servers support it [M].
- Standards: CWE-345 (insufficient verification of data authenticity), CWE-353 (missing integrity check), CWE-354; RFC 9110 (§13.1.5 If-Range, §14.4 Content-Range, §15.3.7 206), RFC 9530.

**B9. Checksum verification**
- Mitigations: let the user paste SHA-256/SHA-512 (and auto-read `.sha256` / `SHA256SUMS` files next to the URL only on the **same origin**, as a hint). Compare in constant time (not secret, but it costs nothing). Show MD5/SHA-1 as "legacy, collisions possible". Hashing happens after the final assembly, before the file is marked complete.
- Standards: CWE-354, CWE-328 (weak hash); SLSA (provenance) for our own releases.

### C. App, update and build chain

**C1. Update channel hijack (Tauri updater)**
- Mitigations: use `tauri-plugin-updater` with a minisign key pair; the public key is in `tauri.conf.json`, the private key and password only in a CI secret (never in the repo) [M]. Updater endpoints over HTTPS only (v2 refuses http in release builds unless `dangerousInsecureTransportProtocol`) [M]. Pin the endpoint to GitHub Releases or our own static host. Keep an offline backup of the private key (losing it means every user must reinstall by hand). Plan key rotation. Add Authenticode code signing for Windows (SmartScreen reputation), so users can verify the installer. Do not let the update manifest choose the install path or arguments.
- Standards: CWE-494 (download of code without integrity check), CWE-345; Top 10 2025 A03 / A08; SLSA v1.0 Build L2-L3 (provenance via `actions/attest-build-provenance`) https://slsa.dev/spec/v1.0/levels ; https://v2.tauri.app/plugin/updater/

**C2. DLL search-order hijacking (installer and app)**
- Attack: the installer runs from `Downloads\`, which also holds a planted `version.dll`, `cryptbase.dll`, `uxtheme.dll` and so on; the app loads a DLL from CWD or from the download folder.
- Mitigations: call `SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32)` early; keep the NSIS installer up to date (NSIS has had fixes for this [M]); never set the CWD of child processes to the download dir unless needed; never load plugins from user-writable paths; install per-machine to `Program Files` (admin-only write) or per-user with that risk written down. WebView2 bootstrapper: use the signed Evergreen bootstrapper.
- Standards: CWE-427 (uncontrolled search path), CWE-426; Microsoft "Dynamic-Link Library Security" https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-security ; Microsoft SDL.

**C3. Supply chain of our own code**
- Mitigations: `Cargo.lock` + `package-lock.json` committed; `cargo audit`/`cargo deny`, `npm audit` in CI; pin GitHub Actions to SHAs; branch protection; signed tags; reproducible builds where possible; OpenSSF Scorecard workflow (checks: Branch-Protection, Pinned-Dependencies, Token-Permissions, Signed-Releases, Dangerous-Workflow, SAST, Fuzzing, Vulnerabilities) https://github.com/ossf/scorecard ; SECURITY.md with a private reporting channel (GitHub private advisories). Fuzz the filename sanitiser, Content-Disposition and Content-Range parsers (`cargo fuzz`).
- Standards: Top 10 2025 A03; SLSA; OpenSSF Scorecard; CWE-1104 (unmaintained third-party components), CWE-1357.

### D. Browser bridge and local API

**D1. Native messaging host abuse by other extensions**
- Attack: another installed extension (or a takeover of ours, see Cyberhaven) talks to our host and asks it to download to any path, add hooks, or read the history.
- Mitigations: the Chrome manifest `allowed_origins` lists **only our exact extension IDs** (no wildcards are allowed there anyway); Firefox `allowed_extensions` [M]. Chrome passes the caller's origin as the first argument to the host; check it. Keep the protocol small and typed: `add_download {url, referrer, cookies-for-url, suggested_name}`; there are **no** commands for setting the path, options, hooks or reading history. The user confirms each download in the app's own window (or a trusted-sites list set in the app, not by the extension). Cap message size (Chrome limits host→browser to 1 MB, browser→host 64 MiB [M]). Write the host manifest to HKCU with an absolute path to a signed binary.
- Standards: CWE-940 (improper verification of source of a communication channel), CWE-346 (origin validation); Top 10 A01; https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging

**D2. Local HTTP API CSRF / DNS rebinding (if the engine has one)**
- Mitigations: **prefer no HTTP listener at all** (native messaging + Tauri IPC cover it; Algorithm step 2). If one is needed: bind `127.0.0.1` only (not `0.0.0.0`); random port; random 256-bit token in a user-only-readable file, sent as a header (not a cookie and not in the query); reject requests whose `Host` is not `127.0.0.1:<port>`/`localhost:<port>` (stops DNS rebinding, Transmission CVE-2018-5702); reject any `Origin` that is not ours; no CORS `*`; JSON only with `Content-Type: application/json` required (blocks simple-form CSRF); Chrome's Private Network Access (preflight to localhost) helps but is not enough on its own [M]. Consider a named pipe with a per-user ACL instead.
- Standards: CWE-352 (CSRF), CWE-350, CWE-1385 (missing origin validation in WebSockets); ASVS V3 (CORS, Origin checks) / V4 [M]; Top 10 A01.

**D3. Tauri IPC command exposure (capabilities)**
- Mitigations: Tauri v2 capabilities per window, the fewest permissions; no `fs:default` / `shell:allow-execute` / `opener` with broad scope; custom Rust commands take IDs, not paths/URLs/commands; **validate every argument in Rust** (the WebView is untrusted); no `remote` URLs in any capability; block navigation away from `tauri://localhost`; consider the isolation pattern; strict CSP (`default-src 'self'; script-src 'self'; object-src 'none'; frame-src 'none'; connect-src ipc: http://ipc.localhost`) [M exact CSP for v2]; asset protocol off or narrowly scoped; turn devtools off in release.
- Standards: CWE-749 (exposed dangerous method), CWE-284, CWE-862; Top 10 A01/A02; https://v2.tauri.app/security/capabilities/ , https://v2.tauri.app/security/csp/

**D4. XSS in the WebView via filename/title/URL/server-error rendering, leading to IPC abuse**
- Attack: a filename such as `<img src=x onerror=__TAURI__.core.invoke(...)>.zip`, a page title from the site grabber, an HTTP reason phrase, an RSS/feed field. In Tauri, XSS can reach every IPC command the window is allowed, which leads to RCE if a command can run or open things.
- Mitigations: React escapes text by default; **ban `dangerouslySetInnerHTML`** (ESLint `react/no-danger`) and `href={userUrl}` without checking the scheme (`javascript:`); render URLs as text; no Markdown/HTML rendering of server data; the CSP above (no inline script); remove `withGlobalTauri` (no `window.__TAURI__` global) [M]; the IPC surface stays small (D3), so XSS cannot do much.
- Standards: CWE-79, CWE-116; ASVS V1 (output encoding), V3 (CSP); Top 10 2025 A05.

### E. Features with privacy and behaviour risks

**E1. Clipboard monitor**
- Risk: reads passwords, 2FA codes, private links; can be a GDPR issue if it is ever sent anywhere.
- Mitigations: off by default (opt-in); process it locally only; match only `http(s)://` URLs with a known downloadable extension; never store clipboard text that did not match; respect the Windows `ExcludeClipboardContentFromMonitorProcessing` / `CanIncludeInClipboardHistory` formats that password managers set [M]; show an indicator while it is on.
- Standards: CWE-359 (exposure of private personal information), GDPR Art. 5(1)(c) data minimisation, Art. 25 privacy by design.

**E2. Telemetry and privacy law**
- GDPR (EU users, extraterritorial Art. 3(2) for an app offered to the EU): telemetry needs a lawful basis (consent is safest), data minimisation, no URLs/filenames/IPs in telemetry, a privacy notice, and rights to delete. ePrivacy Art. 5(3) covers storing or reading information on the user's device [M]. Best option: **no telemetry**. Crash reports opt-in, with a preview of what is sent.
- **Cambodia** [V]: no comprehensive data protection law in force as of the end of 2025. The Ministry of Post and Telecommunications (MPTC) released a draft Law on Personal Data Protection (final draft dated 23 June 2025, published July 2025), with adoption targeted for late 2025 / early 2026; Rajah & Tann's Q4 2025 review still lists it as a draft. Current status in Nov 2026 is **unverified**; check mptc.gov.kh. Related existing laws [M]: the E-Commerce Law (2019) and Consumer Protection Law (2019) have some data clauses; the Cybercrime Law is still in draft [U]. Sources: https://www.hoganlovells.com/en/publications/cambodia-moves-to-enact-comprehensive-data-privacy-law , https://data.opendevelopmentcambodia.net/laws_record/draft-law-on-personal-data-protection , https://www.rajahtannasia.com/viewpoints/regional-round-up-cambodia-q4-2025-year-in-review-edition/ , https://www.icnl.org/resources/civic-freedom-monitor/cambodia
- Standards: ASVS V14 (data protection) [M]; CWE-359.

**E3. Site grabber / recursive download**
- Attack: infinite crawler traps (calendar links), links to other domains or to private IPs (B1), `robots.txt` ignored (legal/ToS issue), huge HTML pages, `file:` or `javascript:` links, path traversal from URL paths when mirroring a site into directories (`/../`, `%2e%2e`, Windows names in path segments).
- Mitigations: same host by default; caps on depth, page count, total bytes and time; the B1 IP check for every link; respect `robots.txt` by default; map URL paths to local paths by **percent-decoding once and then applying the A1 sanitiser to each segment**; never render grabbed HTML in the app WebView.
- Standards: CWE-674 / CWE-834 (excessive iteration), CWE-918, CWE-22.

**E4. Scheduler and post-download commands (shutdown, "run program after")**
- Attack: a command template filled with downloaded data (`{filename}`) runs in a shell, which gives injection (yt-dlp `--exec` chain); the local API or the extension sets the command (aria2 `on-download-complete`); a scheduled task elevated as SYSTEM.
- Mitigations: **Algorithm step 2: delete "run any command"**; keep a fixed list of actions (shutdown, sleep, hibernate, quit app, open folder) as Rust enums. If "run program" is kept: set only through the app's own settings UI (never via IPC from the extension or the local API), stored as a program path plus argv list, run with `Command::new(exe).args([...])` with no shell; refuse `.bat`/`.cmd` (BatBadBut CVE-2024-24576 [M]); downloaded values passed only as separate argv entries. Shutdown shows a cancellable 60-second countdown. Scheduled tasks run as the user, never elevated.
- Standards: CWE-78 (OS command injection), CWE-88 (argument injection), CWE-77; Top 10 2025 A05; ASVS V1 (injection prevention).

---

## Part 4: Top 12 decisions this research supports (for the plan)

1. No kernel driver, no LAN web UI, no generic "run command" action, no auto-extract in v1 (delete first).
2. Browser bridge = native messaging only, `allowed_origins` pinned, typed and minimal protocol, user confirms each download.
3. If a localhost listener must exist: loopback + token + Host check + Origin check, no CORS `*`.
4. One filename sanitiser in Rust (A1-A4), fuzzed, with a test table of every nasty input.
5. `.part` file in the destination dir, `create_new`, no reparse points, handle-based rename, no silent overwrite.
6. MOTW via `IAttachmentExecute` on every completed file.
7. SSRF guard: block private ranges on every hop, pin the resolved IP, http(s) only, no downgrade.
8. Per-origin credential stripping + RFC 6265 cookie jar; cookies fetched per URL and never stored.
9. Range integrity: 206 + exact Content-Range + strong ETag via If-Range, else restart.
10. rustls + platform verifier; no "ignore cert" path; CI grep for `danger_*`.
11. Signed Tauri updates (minisign key in CI secret, backup offline) + Authenticode + SLSA provenance + Scorecard.
12. Tauri: minimal capabilities, IDs not paths in commands, strict CSP, no `dangerouslySetInnerHTML`, asset protocol off.
