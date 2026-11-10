# Security policy / គោលការណ៍សុវត្ថិភាព

**រាយការណ៍ចំណុចខ្សោយដោយឯកជន** តាម GitHub *Private vulnerability reporting* (ផ្ទាំង Security › Report a vulnerability) ក្នុង repository នេះ។ សូមកុំបើក issue សាធារណៈ។ យើងឆ្លើយក្នុង 7 ថ្ងៃ ហើយចេញការជួសជុលតាម update ដែលចុះហត្ថលេខា។

**Report vulnerabilities privately** through GitHub *Private vulnerability reporting* (Security tab › Report a vulnerability) in this repository. Please do not open a public issue. We answer within 7 days and ship fixes through signed updates.

## Supported versions

| Version | Supported |
|---|---|
| 0.9.x (public beta) | yes |
| older | no |

## In scope

The app (`gdm.exe`), the native messaging host (`gdm-bridge.exe`), the browser extension, the installer and the update channel. Threat model and test cards: `docs/GDM-Blueprint.html` §10–§12.

## Out of scope (by design, documented)

- Malware already running as the same Windows user (it can read the same files GDM can; Chromium's native messaging FAQ). GDM still verifies the pipe server image and never takes a path, a hook or an auto-start from the browser.
- Self-built or modified versions not published by this project.

## How releases are protected

Updates: a minisign-signed manifest (`latest.json`, key compiled into the app, no downgrade, expiry) checked by GDM before Tauri's updater, which verifies the installer's own minisign signature. Releases carry SHA-256 sums and GitHub build-provenance attestations. Authenticode signing via SignPath is planned once the project is accepted (D8).
