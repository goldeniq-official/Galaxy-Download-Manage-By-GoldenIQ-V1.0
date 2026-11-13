# G7 gap report to the owner (G7 done-condition: "≥ 8/10 testers … or the gap list is reported to the owner")

Date: 2026-11-13. Recipient: GoldenIQ (owner). Delivered in the closing message of the G7 session and kept here.

**Why there are no tester answers:** the 2-week beta with 10 real people (≥ 5 Khmer) can only start after the
owner publishes a beta build (push + release, owner-only) and recruits the testers. No answer has been invented:
`docs/beta/answers.md` stays empty until real testers answer. The tester kit is ready (`docs/beta/README.md`).

## Gap list: where GDM v1.0 does not (yet) match IDM 6.43

| # | Gap | Impact for an IDM user | Workaround today | Recorded in |
|---|---|---|---|---|
| 1 | FTP / FTPS downloads | `ftp://` links are refused | another FTP client | decisions 2026-11-13 (P3) |
| 2 | Proxy taken from the browser's own proxy setting | a browser-only proxy (extension VPN) is not followed | set the same proxy in GDM (manual/system) | decisions 2026-11-13 (P3) |
| 3 | POST downloads replayed by GDM | form-POST downloads stay in the browser | none needed: the browser keeps them (never lost) | decisions 2026-11-13 (P3) |
| 4 | Subtitles for video | no subtitle files | yt-dlp by hand | decisions 2026-11-13 (P3) |
| 5 | NTLM/Negotiate login to an intranet web server (proxy NTLM works) | intranet sites with Windows login fail | Basic/Digest site login, or the browser | decisions 2026-11-10 (P3) |
| 6 | Live streams and AES-128 encrypted HLS | refused with a message | none in v1.0 | decisions 2026-11-13 |
| 7 | Drag a finished file out of a toast | no drag-out | [Open] / [Open folder] | decisions 2026-11-13 (P3) |
| 8 | Choosing which toolbar buttons show | fixed toolbar | Tasks menu, columns, theme, density | decisions 2026-11-13 (P3) |
| 9 | PAC scripts that name SOCKS (WinHTTP limit) | such routes are refused, not silently direct | set SOCKS manually | decisions 2026-11-13 |
| 10 | Speed parity with IDM not measured on a protocol-valid machine (G5 runs: 0.53–0.66 of IDM on this full disk) | no speed-parity claim allowed | owner NVMe run before v1.0 | decisions 2026-11-10/13 |
| 11 | Microsoft Store (MSIX) | no Store listing | NSIS installer + GitHub Releases | decisions 2026-11-13 |
| 12 | Video component install needs the owner-signed `components.json` | until published, the video install button fails | — | G7.md §8 |
| 13 | Site grabber (§04 Grabber rows) | not in v1.0 | — | §04: P3 |

## What the owner must do for the beta measurement
Push `main`, publish a beta build and `components.json` (+ `.sig`), give the kit to 10 testers (≥ 5 Khmer) for
2 weeks, then their answers go into `docs/beta/answers.md`; ≥ 8/10 "replaces IDM" closes the beta line, otherwise
the gaps they name are added to this list.
