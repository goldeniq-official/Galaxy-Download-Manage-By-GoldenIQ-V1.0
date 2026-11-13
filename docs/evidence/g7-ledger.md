# G7: §04 parity ledger, row by row (council G7 final #3)

Source: every row of `docs/blueprint/parity.py` whose phase is **P2** (26 rows, read 2026-11-13), the P1 rows
moved to P2 in G6 (`docs/decisions.md` 2026-11-10), and the P2 items of §17 that are not ledger rows.
"Done" = code + a test or measured evidence on this PC; "P3" = moved with reason/impact/workaround/trigger in
`docs/decisions.md`; "BLOCKED" = needs the owner's account, hardware or people.

## §04 P2 rows (26)

| # | Row (§04) | Status | Evidence |
|---|---|---|---|
| 1 | Connection per server (Exceptions), learn from 429 | Done | `g7_conns` (profile 2, learned cap), `g7_l02` (total cap 12/12) |
| 2 | Quota (MB per period), total and per queue | Done | `g7_queue` quota test |
| 3 | Refresh download address (R-01 protocol) | Done | `g7_refresh` core + app (seam fail ⇒ no resume, strong ⇒ resume, HTML ⇒ login) |
| 4 | FTP (PASV), FTPS | **P3** | decisions 2026-11-13 (browsers dropped FTP; trigger ≥ 3 requests) |
| 5 | Auth Basic / Digest / NTLM / Negotiate / Kerberos | Done: Basic, Digest, proxy NTLM/Negotiate (SSPI). **P3**: NTLM/Negotiate to an origin server | `gdm-auth` (RFC 7616 vectors, NTLM loopback), `g7_digest`, `g7_proxy` NTLM 3 legs on one socket; decisions 2026-11-10 |
| 6 | Site logins per folder (Credential Manager) | Done | `p01` round trip, `g7_logins` |
| 7 | Proxy: system, PAC, manual, SOCKS, bypass | Done (PAC naming SOCKS fails closed) | `g7_proxy` (6 scenarios), `g7_pac` (real PAC, SOCKS ⇒ Block), `proxy` unit |
| 8 | Proxy taken from the browser | **P3** | decisions 2026-11-13 |
| 9 | Page video panel (off, per site, badge; D5) | Done | `e2e-ext-video.log` 4/4 |
| 10 | Panel on selected links | Done (G6 content script, partial selections) | G6 `e2e-extension.log`; `content.ts` `containsNode(a, true)` |
| 11 | Cookie / referrer / POST | Done: cookie + referrer (G6). **P3**: POST replay | decisions 2026-11-13 (POST stays in the browser, never lost) |
| 12 | HLS/DASH, mux MP4/MKV, quality, subtitle | Done: HLS (GDM reader), DASH pairs via yt-dlp http formats, mux, quality popup. **P3**: subtitles; live and AES-128 HLS refused with a message | `e2e-media.log` (HLS ⇒ MKV, 1+1 streams), `gdm-media` 16 tests |
| 13 | YouTube (in the app via the component; clipboard) | Done | `clip::pick_video` test, UI test "copied video link" |
| 14 | Download queue, sync queue, unlimited queues | Done | `g7_queue`, `g7_sync` |
| 15 | N at once, retry, start/stop, days, periodic (DST) | Done | `sched` ×3 (Q-03), `g7_queue` |
| 16 | On finish: shutdown, sleep, hibernate, exit; 60 s countdown | Done | `g7_queue` q02 ×2 |
| 17 | Force terminate processes (only when chosen) | Done | `os::power` (`/t 0`, `/f` only with force), decisions 2026-11-10 |
| 18 | Open file / program when done (Q-01, UI only) | Done | `hook` ×2, `merge_settings` test, native confirmation |
| 19 | Wake to download (Task Scheduler, AC only) | Done (real wake = manual: firmware) | `q04_wake_task_ac_only_not_elevated` |
| 20 | Virus scan with the user's own scanner | Done | hook `scan` (same checks as Q-01) |
| 21 | Clipboard monitor (opt-in, I-01) | Done | `i01_excluded_clipboard_is_never_read` (real clipboard) |
| 22 | Command line /d /p /f /q /h /n /a /s (I-02) | Done | `e2e-cli.log` 8/8, `cli` unit |
| 23 | Batch download (I-04) | Done | `i04`, UI test |
| 24 | Import/export .ef2, txt, json + settings backup (I-03) | Done | `i03` ×3 (randomized), restore allowlist test |
| 25 | Toolbar customise + theme light/dark/density | Done: theme + density (G6), columns show/hide/order/sort (UI test). **P3**: choosing toolbar buttons | decisions 2026-11-13 |
| 26 | Event sounds (3 built-in + WAV) | Done | `merge_settings` test (WAV only via the picker), UI test |

## P1 rows moved to P2 in G6

| P1 # | Row | Status | Evidence |
|---|---|---|---|
| 8 | User-Agent global + per site | Done | `g7_p1rows` |
| 9 | Temp folder on another volume | **Requirement deleted** (waiver: `.part` stays in the destination, atomic rename) | decisions 2026-11-13 |
| 10 | `.sha256` beside the file | Done | `g7_p1rows` (good ⇒ verified, bad ⇒ HashMismatch) |
| 19 | Site pattern ⇒ folder, remember folder | Done | `g7_misc`, UI test |
| 21 | Drag links onto the window / drop basket | Done | UI tests (window drop, basket), basket capability |
| 24 | Drag a finished file out of a toast | **P3** (no native drag-out in Tauri v2) | decisions 2026-11-13 |
| 26 | Properties: edit address / folder / description | Done | UI test, `job_set_url`, `job_move` |
| 27 | Columns show/hide/order/sort | Done | UI test |
| 31 | No hard-coded strings, Khmer pixel checks, glossary reader | Done: eslint + Khmer clip probe (UI 157/157, WebView2 km). Glossary review by a Khmer reader = beta task 14 (BLOCKED-ON-OWNER) | `ui-tests.log`, `docs/beta/README.md` |

## §17 P2 items that are not ledger rows

| Item | Status | Evidence |
|---|---|---|
| Failure injection (ADR-1) | Done: one process kept; WebView recovery | `e2e-adr1.log` 11/11 |
| Cards F06, F07, F08, F10, F11 | Pass (see G7.md table) | tests above |
| Card F09 (site grabber) | Not in G7: the grabber rows are **P3** in §04 (Grabber rows) | `parity.py` |
| Microsoft Store (MSIX) | **BLOCKED-ON-OWNER** + recorded disposition | decisions 2026-11-13 |
| Speed gate ≥ IDM −5% on a protocol-valid machine | **BLOCKED** (hardware) + recorded waiver terms | decisions 2026-11-13 |
| Beta 10 testers (≥ 5 Khmer), 2 weeks | **BLOCKED-ON-OWNER**; kit ready; gap list = this file's P3/BLOCKED rows | `docs/beta/` |
