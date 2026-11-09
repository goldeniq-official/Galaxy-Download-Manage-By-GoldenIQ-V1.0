# Tech research: Rust/Tauri v2 download manager (Windows-first, Khmer + English)

Date: 2026-11-09. "(unverified)" = from memory or a single secondary source, not checked against the primary page.

## 1. Open-source competitors

| Project | Stack | Licence | Learn from it |
|---|---|---|---|
| Free Download Manager | C++/Qt | **Proprietary now**; GPL only for v2.5-3.9.7, later source not published (GPL dispute) | Not a real OSS competitor. Users mainly know it; its 2023 Linux-installer supply-chain compromise (Kaspersky report, unverified detail) is a lesson for signed builds. https://en.wikipedia.org/wiki/Free_Download_Manager , https://portableapps.com/node/53308 |
| AB Download Manager | Kotlin (Compose Desktop, unverified) | Apache-2.0 | Closest modern rival: clean UI, browser ext in separate repo, added HLS (non-encrypted) capture from browser, ships via winget/Scoop/Homebrew. Heavy JVM runtime is the main weak point we beat with Rust. https://github.com/amir1376/ab-download-manager , https://itsfoss.com/news/ab-download-manager/ |
| Gopeed | Go core + Flutter UI | GPL-3.0 | Core/UI split (Go lib + REST API), JS extension system for site scripts, HTTP + BitTorrent + magnet, also runs headless/web. https://github.com/GopeedLab/gopeed , https://pkg.go.dev/github.com/GopeedLab/gopeed@v1.5.7 |
| Motrix | Electron + aria2 | MIT (aria2 is GPL-2.0+) | Popular UI over aria2 RPC; last real release 1.8.19 (2023) = largely unmaintained; complaints: Electron size, stalls, slow updates. A Tauri rewrite fork "Motrix Next" exists (MIT). https://github.com/agalwood/Motrix , https://aur.archlinux.org/packages/motrix |
| Xtreme Download Manager (XDM) | Java (v7) → C#/.NET (v8) | GPL-2.0 (unverified) | Video capture from browser (HLS/DASH via ffmpeg), browser ext + local HTTP; v8 rewrite stalled for long periods. https://github.com/subhra74/xdm |
| Persepolis | Python/Qt GUI over aria2 | GPL-3.0 (unverified; browser ext GPL-2.0+) | Uses aria2 + yt-dlp; scheduler, queues. https://github.com/persepolisdm/persepolis |
| aria2 | C++ CLI/daemon | GPL-2.0+ with OpenSSL exception (unverified detail) | Reference engine: segmented HTTP/FTP/BT/Metalink, JSON-RPC/WebSocket. RPC is a known attack surface (see §5). https://github.com/aria2/aria2 |
| uGet | C/GTK | LGPL-2.1 (unverified) | Queues, clipboard monitor, optional aria2 plugin; mostly dormant. https://ugetdm.com/ |
| Hitomi Downloader | Python, Windows | Licence unclear - check repo LICENSE (unverified) | Site-specific scrapers for image/video sites; shows demand for "gallery" downloads but carries legal/ToS risk. https://github.com/KurtBestor/Hitomi-Downloader |
| yt-dlp | Python | Unlicense (public domain) (unverified, but long-standing) | De-facto site extractor; updates very often (sites break weekly), so ship as a self-updatable sidecar, not baked in. https://github.com/yt-dlp/yt-dlp |

Common user complaints across the category (from reviews/issues, general): bloated runtimes (JVM/Electron), browser extension breaking after Chrome MV3, SmartScreen/AV false positives on unsigned builds, video sites breaking, ads/bundleware (FDM, IDM clones), no portable mode.

## 2. Free code signing

**SignPath Foundation (free for OSS)** - https://signpath.org/terms , https://about.signpath.io/product/open-source
- OSI licence for all components, **no commercial dual-licensing**, **no proprietary components** (system libs ok); no malware/PUP; no "hacking tools"; actively maintained; **already released** in the form to be signed.
- Team must own the repo; sign **only your own built artifacts** (upstream binaries can be bundled unsigned, e.g. ffmpeg/yt-dlp sidecars stay unsigned by you).
- MFA for all members on SignPath + GitHub; roles Authors/Reviewers/Approvers; **manual approval per release**.
- Builds must be **verifiable from source**: origin verification records repo, branch, commit, CI job URL (GitHub Actions integration). https://docs.signpath.io/origin-verification
- Home page needs a section titled "Code signing policy" (attribution, team roles, privacy policy). Privacy: any data sent off-device must be disclosed at install and switchable off. Must provide uninstall.
- Certificate subject/publisher = **"SignPath Foundation"**, not you. It is an OV-type cert on an HSM; SmartScreen reputation is shared across many projects but still built per file/cert (unverified how fast).
- Ownership note: publisher shown in UAC will be "SignPath Foundation" - does not change copyright/authorship (GoldenIQ stays author in LICENSE/metadata).

**Certum Open Source Code Signing** - not free: ~EUR 69 (set incl. card reader; later renewals cheaper, unverified), issued to an individual; shop says trusted by Microsoft/SmartScreen. Requires identity docs. https://shop.certum.eu/open-source-code-signing.html (older forum reports doubt kernel/driver use only - irrelevant for apps).

**Azure Artifact Signing (ex-Trusted Signing)** - ~US$9.99/month (Basic), billing from account creation; **individuals only USA/Canada**, orgs USA/Canada/EU/UK (needs 3-yr verifiable business history for orgs, unverified). Not available to a Cambodia-based individual. https://azure.microsoft.com/pricing/details/artifact-signing/ , https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options

**SmartScreen**: EV no longer bypasses SmartScreen (since ~Aug 2024 EV and OV treated equally). Reputation is earned by downloads over time per file hash and per certificate; a new cert/renewal may restart it. Signing still helps (reputation accrues to cert across versions; unsigned resets every build). https://learn.microsoft.com/windows/apps/package-and-deploy/smartscreen-reputation , https://textslashplain.com/2024/11/15/best-practices-for-smartscreen-apprep/

**Microsoft Store**: MSIX/Win32 apps in the Store are signed by Microsoft and avoid SmartScreen - free route (see §3).

**macOS**: notarization needs Apple Developer Program, US$99/year; no free OSS option. Unsigned = Gatekeeper "right-click Open"/`xattr` workaround.

**GitHub artifact attestations**: free on public repos (`actions/attest`, formerly `attest-build-provenance`), Sigstore-backed, gives **SLSA v1.0 Build L2**; L3 with reusable workflows. Verify with `gh attestation verify`. Does not affect SmartScreen, but proves binaries came from the repo. https://docs.github.com/actions/security-guides/using-artifact-attestations-to-establish-provenance-for-builds , https://github.com/actions/attest-build-provenance

**Truly free options in practice**: (1) SignPath Foundation, (2) Microsoft Store distribution (Store signs), (3) attestations for provenance. "Open source certificate" from Certum is cheap, not free. No other CA currently gives free OSS certs (unverified, none found).

## 3. Distribution with no registration (for users)

- **GitHub Releases**: primary; also hosts Tauri updater `latest.json`.
- **winget**: PR manifest to microsoft/winget-pkgs (free, automated validation; needs stable installer URL + SHA256; `wingetcreate`/`winget-releaser` action). https://github.com/microsoft/winget-pkgs
- **Scoop**: own bucket repo or PR to `extras` bucket; good for portable zip. https://github.com/ScoopInstaller/Extras
- **Chocolatey**: free community repo, human moderation (slow), needs account. https://community.chocolatey.org/
- **Microsoft Store**: **free for individual developers worldwide since Sept 2025** (ID + selfie verification, no card); company account still US$99 one-time. Store signs MSIX / can list unsigned-hosted Win32 EXE/MSI (Win32 listing requires your own signed installer, unverified). https://blogs.windows.com/windowsdeveloper/2025/09/10/free-developer-registration-for-individual-developers-on-microsoft-store/
- **Flathub**: free; Linux only (later).
- **Tauri v2 auto-update**: `tauri-plugin-updater`; `tauri signer generate` → minisign keypair; pubkey in `tauri.conf.json`, private key as CI secret `TAURI_SIGNING_PRIVATE_KEY` (+ `_PASSWORD`); `tauri-action` builds, writes `.sig` and `latest.json`; endpoint `https://github.com/<owner>/<repo>/releases/latest/download/latest.json`. Losing the private key = no more updates for existing installs - back it up. This signature is separate from Authenticode. https://v2.tauri.app/plugin/updater/

## 4. Rust building blocks

- **HTTP**: `reqwest` (on hyper 1.x) - HTTP/1.1 + HTTP/2 stable, range requests trivial. **HTTP/3 still unstable** (`--cfg reqwest_unstable`, quinn+h3); skip HTTP/3 for v1. https://github.com/seanmonstar/reqwest/releases . For segmented downloads one client + N concurrent `Range` requests is enough; existing crates (e.g. `http-downloader`, `trauma`) are small/immature (unverified) - write own ~300-line segmenter.
- **TLS**: reqwest 0.13 defaults to rustls (unverified); use **`rustls-platform-verifier`** (uses Windows CertVerify / system store) so corporate MITM proxies and enterprise roots work; `native-tls` (SChannel) is the alternative. Proxies: reqwest reads env proxies only - read Windows system proxy (WinINet / registry, or `sysproxy` crate) yourself; PAC/WPAD needs extra work (unverified crate options).
- **HLS/DASH**: delegate to **yt-dlp + ffmpeg sidecars**, downloaded on demand (not bundled) to keep installer small and licences clean. yt-dlp = Unlicense; ffmpeg = LGPL-2.1+ (or GPL if built with x264 etc. - yt-dlp's FFmpeg-Builds offer both). Running as separate processes = "mere aggregation", no licence infection of our code; must still ship licence text + source offer/link for ffmpeg if we redistribute it. Native HLS (m3u8-rs + AES-128 decrypt) possible later.
- **BitTorrent**: `librqbit` (rqbit) - Apache-2.0 per Cargo.toml, embeddable, DHT, magnets, HTTP API; v1 protocol focus. https://github.com/ikatson/rqbit . Torrent support increases AV/ISP/ToS concerns - consider later phase.
- **SQLite**: `rusqlite` (sync, `bundled` feature, simplest) vs `sqlx` (async, compile-time checks). For a desktop app with one writer, rusqlite + WAL in a blocking thread is enough. Tauri also has `tauri-plugin-sql` (sqlx).
- **Preallocation**: Windows - `File::set_len` (fast on NTFS, valid-data-length zero-fill happens lazily on write), optional `FSCTL_SET_SPARSE` (sparse) via `windows` crate; `SetFileValidData` needs SeManageVolumePrivilege (admin) and leaks old disk data - don't use. Write segments with `seek_write` (FileExt) to avoid locking.
- **Mark-of-the-Web**: **yes, write it** - browsers do (Chrome/Edge/Firefox via `IAttachmentExecute::Save`), and a download manager that strips MotW becomes a malware-delivery bypass (SmartScreen, Office Protected View won't fire). Preferred: `IAttachmentExecute` (`SetSource(url)`, `SetReferrer`, `SetLocalPath`, `SetClientGuid`, `Save()`), which writes `Zone.Identifier` (ZoneId=3, HostUrl, ReferrerUrl) honouring policy, and **invokes registered antivirus scanners** (via Attachment Manager; AV hook behaviour depends on policy, unverified). Fallback: write `file:Zone.Identifier` ADS directly. Skip HostUrl for private mode/option. https://learn.microsoft.com/windows/win32/api/shobjidl_core/nn-shobjidl_core-iattachmentexecute , https://textslashplain.com/2022/12/02/mark-of-the-web-additional-guidance , https://chromium.googlesource.com/chromium/src/+/4b7f2d40ed547f844c3b42b712f035e90357e1e5/content/common/quarantine/quarantine_win.cc

## 5. Browser integration (2026)

- **MV3 interception**: `chrome.downloads.onCreated` / `onDeterminingFilename` → read URL/filename/referrer/mime → `chrome.downloads.cancel()` + `erase()` → hand off. Only **one** extension can own `onDeterminingFilename` (conflicts with IDM/other managers). Cookies/headers: request `cookies` permission + host permissions to forward session cookies (sensitive - minimise). Service worker can be killed; a `connectNative` port keeps it alive while open (unverified exact rule). Race: cancel may fail if file already finished - handle. Also offer context-menu "Download with ..." which is more reliable. Firefox supports same `downloads` API (+ `webRequest` blocking still allowed in Firefox).
- **Native messaging (recommended)**: host manifest registered under `HKCU\Software\Google\Chrome\NativeMessagingHosts\<name>` (Edge: `...\Microsoft\Edge\...`, Firefox: `HKCU\Software\Mozilla\NativeMessagingHosts`). `allowed_origins` must list exact `chrome-extension://<id>/` (no wildcards) - browser enforces caller identity; Firefox uses `allowed_extensions`. No open port. https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging
- **Local HTTP/WebSocket server risks**: any website can hit `127.0.0.1` - CSRF (simple POSTs), **DNS rebinding** (attacker domain resolves to 127.0.0.1, bypasses same-origin), WebSocket has no CORS. Precedents: Transmission RPC DNS-rebinding RCE (CVE-2018-5702), exposed aria2 RPC → arbitrary file write → code exec (`--dir` + `--on-download-complete`), Zoom localhost server (2019). If a local server is unavoidable: random port + per-install secret token, check `Host` header == `127.0.0.1:<port>`, check `Origin` allowlist, never expose a "set save dir / run hook" command. https://nvd.nist.gov/vuln/detail/CVE-2018-5702 , https://salivity.github.io/aria2/article/what-are-the-risks-of-exposing-aria2-rpc-to-the-internet
- **Store fees**: Chrome Web Store US$5 one-time (https://developer.chrome.com/docs/webstore/register); Edge Add-ons **free** (Partner Center MSA) (https://learn.microsoft.com/microsoft-edge/extensions/publish/create-dev-account); Firefox AMO free (signing mandatory, self-distribution allowed) (unverified via search, long-standing).

## 6. Installer i18n and Khmer

- **Tauri NSIS**: `bundle.windows.nsis.languages` (array; first entry = fallback), `displayLanguageSelector: true`, `customLanguageFiles {lang: path.nsh}`; key must be a **valid NSIS language**. Selector lists languages from the array (order of display, unverified). https://docs.rs/tauri-utils/2.9.3/tauri_utils/config/struct.NsisConfig.html
- **NSIS has no official Khmer.nlf** (VLC hit "Khmer.nlf can't be opened" in 2020; community ~95% translation exists; MakeLangId lacks Khmer; codepage 1780 warning). So Khmer in the installer requires adding Khmer.nlf/.nsh to the NSIS install used by Tauri (or a custom `template`). Unicode NSIS 3 can render Khmer text if the font is present (Windows 8+ ship "Khmer UI"/"Leelawadee UI"). Feasible but extra work; test. https://nsis-dev.github.io/NSIS-Forums/html/t-365664.html , https://mailman.videolan.org/pipermail/vlc-devel/2020-April/132986.html
- **WiX/MSI**: MSI tables use ANSI codepages; Khmer has none → Khmer MSI UI effectively unsupported (unverified, strong prior). Use NSIS.
- Pragmatic choice: installer in English (+ Khmer if Khmer.nlf works); app UI language picker on first run, Khmer default.
- **Fonts in WebView2**: bundle **Kantumruy Pro** (OFL, variable, Google Fonts) and/or **Noto Sans Khmer** (OFL) locally (offline, no CDN). Khmer needs line-height ~1.6-1.8 (stacked coeng subscripts) - avoid fixed-height single-line clipping. https://fonts.google.com/specimen/Kantumruy+Pro , https://fonts.google.com/noto/specimen/Noto+Sans+Khmer
- **Line breaking**: Khmer uses no spaces; Chromium/WebView2 uses ICU dictionary-based line breaking for Khmer (works for most text); translators can insert U+200B (ZWSP) at phrase boundaries for buttons/labels. `Intl.Segmenter('km', {granularity:'word'})` available for truncation logic.
- **Numbers**: `Intl.NumberFormat('km-KH')` gives Latin digits by default; Khmer digits via `km-KH-u-nu-khmr` (០១២...). Recommend Latin digits for sizes/speeds (readability, copy-paste) with option; dates via `Intl.DateTimeFormat('km-KH')`.

## 7. Icon sets

| Set | Licence | Notes |
|---|---|---|
| Lucide | ISC | ~1.5k stroke icons, tree-shakeable `lucide-react`; most common in React. https://lucide.dev/license |
| Phosphor | MIT | 6 weights incl. fill/duotone - good for "state via fill" (owner bans outlines/glow). https://phosphoricons.com |
| Tabler | MIT | ~5k+ stroke icons, some filled. https://tabler.io/icons |
| Fluent UI System Icons | MIT | Native Windows 11 look, regular+filled pairs. https://github.com/microsoft/fluentui-system-icons |
| Material Symbols | Apache-2.0 | Variable font (fill/weight axes); heavier, Android look. https://fonts.google.com/icons |

Fit for a Windows-first app: Fluent (native feel) or Phosphor (filled variants). All permissive; keep licence text in third-party notices.
