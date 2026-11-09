# Neat Download Manager (NDM): feature inventory, user feedback, security

Researched 2026-11-09. Legend: **[V]** verified from the official site or from the Firefox extension code I downloaded and read myself. **[S]** reported by one or more third-party sources (cited). **[U]** unverified, conflicting or weak source.

## 0. Status snapshot

| Item | Value | Source |
|---|---|---|
| Developer | Javad Motallebi (one-person project, Iranian C++ dev; site template by Dima Group, dima.ir) | [V] https://www.neatdownloadmanager.com/index.php/en/about |
| Windows app | v1.4 (~900 KB Inno Setup installer, x86, machine-scope, admin needed). Sub-builds seen: 1.4.10 (Softpedia), 1.4.24 (VideoHelp, Nov 2022) | [V] official site; [S] https://wingetly.io/apps/javad-motallebi/neat-download-manager ; [U] build numbers |
| macOS app | v1.3 (~1.4 MB DMG, macOS 10.10+, Intel + Apple Silicon [S]) | [V] official site; [S] https://grokipedia.com/page/Neat_Download_Manager |
| Chrome/Edge extension | 1.9.92, MV3, updated 2023-11-21, ~700K users, 4.3★ (431) | [S] https://chromewebstore.google.com/detail/neatdownloadmanager-exten/cpcifbdmkopohnnofedkjghjiclmhdah , https://extscope.org/extension/cpcifbdmkopohnnofedkjghjiclmhdah |
| Firefox extension | 1.9.9, MV2, updated 2023-01-30, ~24.5K users, 4.6★ (172), "All Rights Reserved" | [V] https://addons.mozilla.org/en-US/firefox/addon/neatdownloadmanager-extension/ |
| Maintenance | No real app release since about 2022 (1.4). AlternativeTo marks it **discontinued**. Users in 2026: "gets no updates" | [S] https://alternativeto.net/software/neat-download-manager/about ; AMO review by Carl, Jun 2026 |
| License | Freeware, closed source. Claims that it is "open source" (kocpc.com.tw, one AlternativeTo user) look **wrong**: no source on the official site, GitHub repo `Neatdownloadmanager/Neat-Download-Manager` is now 404 and only ever showed a binary (`NeatDM_setup.exe`) | [V] 404 checked; [S] https://newreleases.io/project/github/Neatdownloadmanager/Neat-Download-Manager/release/NeatDM_setup.exe |
| Money model | Free. No ads, no Pro tier, no donation link found on the site | [V] official site (no donation/ads mentioned); [S] AlternativeTo tag "Ad-free" |
| Languages | **English only.** Chinese UI exists only as unofficial community patches. Users complain (Chinese, Taiwanese) | [S] AMO review Jul 2021; https://www.kocpc.com.tw/archives/566560 ; bilibili search summaries |
| Portable | No official portable build found | [U] |

## 1. Feature inventory

### 1.1 Download engine
- "Optimized Dynamic Segmentation Algorithm": splits the file into segments and re-splits as connections free up, to use the full bandwidth. [V] official site
- Protocols: HTTP, HTTPS, FTP. No BitTorrent, magnet, Metalink or SFTP. [V]
- **Max connections per download**, changeable **while the download runs**. Chinese sources say up to 32 connections. [V] official site; [U] the 32 cap
- **Bandwidth limit**, changeable live, per download and global (Options > Limit Bandwidth). kocpc tested it: a 1000 KB/s cap held. [V] official site; [S] kocpc
- Pause/resume. Resumes "crashed" downloads (after an app or system crash or a network loss). [V]
- **Renew expired downloads** (refresh the link of a dead download). [V] official site. Users want the original file name kept after renew/clone ([S] AMO review by TeainToi, Jan 2023).
- Segments go to a **temporary directory**, then merge on completion. Complaints: disk spike/freeze when merging big files, and no report of which temp part failed to merge ([S] bilibili summary; AMO review by Carl, 2026).
- HLS: downloads every .ts segment and merges them into **one .ts file**. No remux to MP4, so users must convert ([V] official site; [S] kocpc: X/Twitter and SoundCloud came out as .ts).
- Default User-Agent option (added in 1.2.12, Aug 2020). [S] Softpedia changelog via search snippet
- Proxy: HTTP and SOCKS (v4/v5 per the GitHub release text). Auth: Basic, Digest, NTLM for both HTTP and proxy. A user says it can work over Tor. [V] site; [S] AMO review by Carl
- Drag-and-drop links to add a download: **macOS only**. [V] official site

### 1.2 Main window / organisation
- Left tree: All Downloads / Complete / Incomplete, plus **by file type** (Video, Document, Compressed, Music, Program and so on). [V] site; [S] kocpc
- List columns: name, size, %, speed, time left. [S] kocpc
- Toolbar has a **"Browsers"** button that opens the extension install pages (Chrome, Firefox, Edge). [S] kocpc
- Options dialog (known contents): max connections per download, download directory, temp directory, bandwidth limit, proxy, default user-agent, **list of file extensions to take over from the browser**, and a "show media panel" toggle (the app pushes `ShowPanelFox=0/1` to the extension). [S] kocpc, https://rottenwifi.com/neat-download-manager-for-windows/ ; [V] the ShowPanel flag is in the extension code
- Categories do **not** go to separate folders: everything lands in one directory. [S] bilibili/Chinese reviews
- Queue: one source says it has queueing and prioritisation (SourceForge), others say it has no queues or rules. **[U] Likely only a simple concurrent list.**
- **Scheduler: none.** Grokipedia and others say it has no scheduled downloads. One tips site claims a scheduler. [S]/[U]: treat as absent.
- Post-download actions (shutdown, sleep, antivirus scan): none found. [S] geekchamp (low quality); [U]
- Clipboard monitoring: no evidence found. [U]
- Batch/pattern download, site grabber: none. Users ask for "batch download and sort" (AMO review, May 2023). [S]
- Dark mode: only an AlternativeTo tag; likely the macOS build follows the system appearance. [U]
- UI typos and odd English. [S] AlternativeTo review by JohnDangerbrooks

### 1.3 Browser extension (read from the code of Firefox 1.9.9)
- Browsers: Chrome, Edge, Firefox. Opera, Brave, Vivaldi and Arc work through the Chrome store. Orion (macOS) does **not** work. Safari: no extension found [U]. [V] AMO; [S] https://orionfeedback.org/d/10813-orion-browser-currently-does-not-natively-support-integration-with-ndm-neat-download-manager
- **Catching**: uses `webRequest`/`webRequestBlocking` to intercept responses whose extension or MIME type is on the list (ZIP, RAR, EXE, MSI, DMG, MPKG, PDF, EPUB, MKV, AVI, WMV, FLV, WEBM, MP4, AAC, WAV, WMA and more). It **cancels the browser download** and sends it to NDM with method, URL, Referer, Origin, **cookies** (`chrome.cookies.getAll`), Content-Type, size and **POST body** (so POST-form downloads work). [V]
- **Bypass**: hold **Delete or Backspace** while clicking (keyCode 46/8). [V]
- **Toolbar icon toggle**: clicking it turns the "Download catcher" on or off (red "Off" badge). It is not discoverable: users keep asking for exactly this toggle (AMO 2021–2023), and kocpc says "clicking the icon does nothing". [V] code; [S] reviews
- Context menu: "Download by NeatDownloadManager" on links and images. [V]
- **Media panel**: a fixed 210 px floating box injected into every page (z-index 1e8, own close button), titled "NeatDownloadManager Video/Audio Panel". It lists found media as "MP4 File HQ/LQ", "TS File <bitrate Kbps> <resolution>", "Subtitles File", "2 Files" (separate video + audio streams), with size and duration. [V] content script
- HLS: parses master playlists (`EXT-X-STREAM-INF` → one row per variant with BANDWIDTH and RESOLUTION), `EXTINF` durations, `EXT-X-BYTERANGE`. [V]
- **DASH/MPD: not supported** (no MPD code; a user asked for it in Jun 2022). [V] code; [S] AMO review by nikgrc
- YouTube: the code maps YouTube itags (144p–2160p, 60fps) and reads `formats`/`adaptiveFormats`, but **skips any stream with `signatureCipher`**, which is now nearly all of them. The store listing says plainly: "Can't download YouTube videos and encrypted media files". [V] code + listing
- Facebook (`hd_src`/`sd_src`) and Vimeo special-cased. Subtitles: VTT, SRT, DFXP, YouTube `timedtext`. [V]
- No DRM or encrypted media (by design). [V] listing
- Privacy statement: collects no data. [V] listing

### 1.4 macOS vs Windows
- Mac 1.3 vs Windows 1.4 (Mac lags behind). Drag-and-drop only on Mac. Mac asks for system permissions on first run. Otherwise the same features. [V]/[S]

## 2. Strengths users praise
- **Free alternative to IDM, "IDM for Mac"** ("It's just IDM for mac. wow loved it" – AMO, Nov 2022; "Meilleur alternative de l'IDM" – Chrome 2026; gHacks: "surprisingly good free alternative"). https://addons.mozilla.org/en-US/firefox/addon/neatdownloadmanager-extension/reviews/?page=4 , https://www.ghacks.net/2021/02/27/neat-download-manager-is-an-impressive-downloader-that-also-supports-media-grabbing/
- **Tiny and light**: 900 KB installer, low CPU and RAM (AlternativeTo, MacUpdate 4.7★ per Grokipedia, Techjockey).
- **Fast**: "gets up to a gig and holds steady… gigabit" (AMO, Aug 2024); Thai "ดาวน์โหลดได้เร็วมาก" (very fast, Jan 2025); kocpc speed test.
- **Simple, zero-config**: "free, simple, well functioning" (AMO, Apr 2024); "Simple and straightforward" (2026).
- **Media catching on many sites**, "better than cat-catch" (Chrome review, Oct 2026); Facebook and adult sites work (kocpc).
- **Per-download live speed and connection control** (kocpc, Taiwan).
- Worldwide base: Arabic "جداً مفيد" (very useful), Indonesian "bagus, pertahankan" (good, keep it up), Chinese "简单轻巧但是非常牛逼" (simple, light and great), Persian users ("mahdi": "best download manager"). Source: extscope reviews.
- No Khmer- or Vietnamese-specific threads found (Vietnamese AMO pages only mirror the reviews above).

## 3. Weaknesses and complaints
**Extension / UX**
- Media panel is intrusive and ugly: "that download from this site thing is ugly af… something more simple and smaller" (Chrome, 2026); "I don't like the panel… option to remove it" (AMO, 2022); it pops up in the video player even when disabled (AMO, 2023). Users write custom CSS to shrink it.
- No obvious on/off or per-site toggle; it **grabs every download**: "I only want it to take over when I actively call it" (AMO, 2022), plus at least 4 more requests for a toggle.
- **False catches**: Reddit loads trigger `compression-dictionary.dat` downloads (Chrome, Oct 2026); unwanted automatic downloads (Grokipedia); breaks Synology Office online docs (AMO, 2025); YouTube community page keeps refreshing (AMO, 2021).
- Extension does not launch NDM if the app is not running; FDM's does (AMO, 2021, Chinese).
- Live streams: the panel list grows past 10,000 entries (AMO, 2022).
- File names: the name gets wiped and replaced by an auto name (Chrome, 2026); subtitle file name does not match the video (AMO, 2022); no rename before download (AMO, 2023).
- Download dialog: the file name field is dark and low-contrast (Chrome, 2026).
- Works only on Mac/Windows; synced browsers without NDM still get the extension (AMO, 2020).

**Media**
- No YouTube, no DASH/MPD, no DRM. HLS gives .ts only (no MP4). Instagram failed (kocpc). Video without sound (AMO, 2021: probably split video + audio streams not merged). "only 720p not 1080p" (AMO, 2020). Video catching was weak on Chromium vs Firefox (gHacks 2021, Grokipedia). "Tried on one stream and it didn't work" (AMO, 2026). Breaks on forks: "Not working on Iceraven… needs maintenance" (AMO, Oct 2026).

**Engine**
- SSL handshake errors on some HTTPS sites (AMO, 2022). Big files: merge stage is slow and fragile, with no report of which part failed. Sporadic bugs that need a reinstall (Grokipedia).

**App / product**
- English only, with typos. Only 3–4 top-level views; no search in history [U]; no scheduler, no real queues, no per-category folders, no post-actions, no batch.
- **Abandoned**: no app update since about 2022; Chrome extension since Nov 2023; no public tracker. Contact by email only.
- **Trust**: closed source, **unsigned** Windows installer; AV heuristics flag it (Gridinsoft "Malware.Win32.Patcher.cc"; Norton warning per AlternativeTo 2024; Defender "Phonzy.A!ml" on a temp segment file, Zotero forum). Most likely false positives, but users lose trust. https://forums.zotero.org/discussion/comment/470425/

## 4. Security
- **Transport: local WebSocket, not native messaging.** [V] The extension opens `ws://127.0.0.1:10007/download` with subprotocol `neatextension.v1`. The Firefox manifest also declares `nativeMessaging` but never uses it. The app replies with `waiting`/`nowaiting`/`ShowPanelFox=…`. The payload is HTTP-header-like text: URL, Referer, Origin, **Cookie header**, UA, POST body.
- Risks (inferred from the design, not tested):
  1. **No authentication token** in the handshake. Any local process can pose as the app by binding port 10007 first (port squatting) and **receive the user's cookies** for every caught download, i.e. session hijack.
  2. Web pages can open `ws://127.0.0.1` too. Unless the app checks the `Origin` and subprotocol, any website could push downloads into NDM (drive-by download into the Downloads folder). [U] Not tested whether NDM checks Origin.
  3. Plain `ws://` on loopback: fine on the wire, but there is no integrity binding to the extension ID.
  4. Wide extension permissions: `<all_urls>`, cookies, webRequestBlocking, and a content script in every frame at document_start. The code is minified/obfuscated (extscope rates it HIGH risk).
  5. Unsigned Windows binary, admin installer, no auto-update channel, so fixes cannot reach users.
- No public CVEs found for NDM. [S] search found none
- Fix in our app: native messaging (Chrome/Firefox/Edge) as the primary channel, or a localhost channel with a per-install random secret, an Origin allowlist (extension origins only) and a random port. Sign the binaries. Open source. Minimal permissions.

## 5. UX: keep vs fix
**Keep**: tiny footprint and instant start; zero-config defaults; live change of connections and speed per download; transparent takeover of browser downloads, including POST and cookies; Delete-key bypass (but make it discoverable and configurable); the right-click menu item; the media list with quality, bitrate, size and duration; subtitles grabbing; HLS variant choice; renew expired link; "Browsers" button that installs extensions; Complete/Incomplete/type tree; proxy with NTLM; Mac drag-and-drop (bring it to Windows too).

**Fix**: small, unobtrusive, dismissable media button with a per-site "never on this site" setting and a global toggle in the popup; real popup UI on the toolbar icon (state, recent catches, pause catching); auto-launch the app from the extension (native messaging host); exclusion lists (URL patterns, min size, MIME) to stop false catches like `compression-dictionary.dat`; mux HLS/DASH to MP4 (ffmpeg) and merge split video + audio; DASH support; keep user file names and rename before start; cap and dedupe the media list on live streams; clear merge-error reporting with retry per segment; streaming merge (write in place, no final spike); localisation (incl. Khmer, Chinese, Persian, Arabic RTL); dark/light theme; history search; scheduler; queues; per-category folders; post-actions; batch/clipboard; signed builds and auto-update; public issue tracker.

## 6. IDM features NDM lacks
Scheduler (start/stop times, shutdown when done) · multiple queues with sync limits · per-category save folders and rules · clipboard monitoring · batch download from a wildcard pattern · Site Grabber/spider · "Refresh download address" from the page (NDM has renew, but weaker) · YouTube and broad video grabbing incl. DASH and audio/video merge · per-site exceptions and a browser-monitoring config (key modifiers, exception lists) · command-line interface · import/export of lists · antivirus scan after download · sound and notification options · 40+ UI languages · regular updates and support. (IDM is Windows-only and paid; NDM's edge is free + macOS.) Sources: https://mirror5.internetdownloadmanager.com/support/options.html ; Grokipedia comparison.

## Source notes
geekchamp.com and similar SEO pages contradict the official site (e.g. "no bandwidth control"), so they were not trusted. VideoHelp, Softpedia, gHacks and TheWindowsClub blocked fetching (403/captcha); I used their search snippets only. Reddit: no NDM threads surfaced in search.
