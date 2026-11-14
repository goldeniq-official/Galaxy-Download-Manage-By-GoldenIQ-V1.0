# Galaxy Download Manager (GDM)

Galaxy Download Manager is a desktop download manager for Windows. It downloads files over multiple connections, resumes interrupted transfers, and gives a file its final name only after every byte has been verified. It is free for individuals and organisations alike, and contains no advertising, no telemetry and no account system. The interface is available in Khmer and English.

**Current release:** 1.0.0 · [Release notes and known limitations](docs/release/notes-1.0.0.md) · Officially supported platform: Windows 11 x64

## Features

**Downloading**
- Multi-connection transfers: 2 connections at start, scaling to 8 by default and up to 32, adjustable while a download is running.
- Automatic resume after network loss, application exit or a system restart.
- Total and per-file speed limits, file categories, and search in Khmer and Latin script.

**Integrity**
- Every completed file is verified with SHA-256 before it receives its final name.
- Partial and unverified data is never presented as a finished file. This is tested against 1,000 forced process terminations and deliberately faulty servers, with no corrupted file published.

**Security**
- Mark-of-the-Web is applied and the system antivirus scans each completed file.
- Potentially dangerous file types are never opened automatically.
- Downloads from the local network require confirmation; invalid TLS certificates stop the transfer, with no override.
- No local network port is opened. Browser extensions communicate with the application through native messaging only.
- Application updates are cryptographically signed, checked once a day, installed only while GDM is idle, and can be turned off.

**Browser integration**
- Extensions for Google Chrome, Microsoft Edge and Firefox. Brave, Vivaldi and Opera are registered but not yet tested.
- <kbd>Ctrl</kbd>+click sends a link to GDM; <kbd>Alt</kbd>+click leaves it to the browser.
- Context menu entries "Download with GDM" and "Download all links", and a per-site exclusion.

**Site grabber** (Tasks menu)
- Explores a website and lists the images, videos or documents that match your selection. It only reads pages, skips logout and delete links, and enforces limits against crawler traps. See [docs/site-grabber.md](docs/site-grabber.md).

## Installation

1. Download `GDM-Setup_1.0.0_windows-x64.exe` (Windows on Arm: `GDM-Setup_1.0.0_windows-arm64.exe`) from the [official GitHub Releases page](https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0/releases) or the official website.
2. Verify the file before running it:
   - compare its SHA-256 hash with `SHA256SUMS.txt` from the same release, and
   - confirm its build provenance:
     ```
     gh attestation verify GDM-Setup_1.0.0_windows-x64.exe --repo goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0
     ```
3. Run the installer. It installs for the current user and does not require administrator rights. Release 1.0.0 does not yet carry an Authenticode signature, so Windows SmartScreen may display a warning; select **More info › Run anyway** only after the verification above succeeds.
4. On the final page of Setup, select **Add** next to each detected browser and confirm in the browser's extension store. GDM shows **Connected** when the extension is linked. The same option is available later under **Settings › Browser**.

The file `GDM_…-setup.exe` in each release is the underlying NSIS installer, intended for winget and silent installation (`/S`). An uninstaller is included and removes the browser registration and autostart entries.

## Official sources

GDM is distributed only through the channels below. A build, extension or store listing obtained elsewhere is not an official release, and the project accepts no responsibility for it.

- Source code and releases: https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0
- Website: https://gdm.goldeniq.app
- Browser extensions: only the store listings linked from inside the GDM application.

## Code signing policy

Release builds are produced from this repository by GitHub Actions and published with SHA-256 checksums and build-provenance attestations. The full code signing policy, including team roles, is published at https://gdm.goldeniq.app/code-signing-policy.

## Privacy

GDM collects no usage data and sends nothing to its authors. It connects to the network only to perform the downloads you request, to check for updates (which can be turned off), and to fetch optional components after you have agreed. Full details: [PRIVACY.md](PRIVACY.md) · https://gdm.goldeniq.app/privacy

## Licence and intellectual property

Galaxy Download Manager is copyright © 2026 Golden IQ and Chhit Sungchhay (the Licensor). The source code is published under the **Reciprocal Public License 1.5** (SPDX: `RPL-1.5`), an OSI-approved open source licence. The licence text is in [LICENSE](LICENSE); its application to this project is set out in [NOTICE.md](NOTICE.md). The Licensor offers GDM under no other licence. The summary below is for convenience only; [LICENSE](LICENSE) is the binding text.

**Use.** Anyone may download, install and use the official release free of charge, including businesses, schools and public institutions.

**Modification.** Any person or organisation that modifies GDM and Deploys the result accepts the following obligations. Under RPL-1.5, Deploy includes internal use within a business or organisation, not only public distribution (§1.2). The only exceptions are Research and Personal Use.
- The complete source code of the modified version, including any components required to install and run it, must be published under RPL-1.5 within one month of Deployment and kept available for at least twelve months after Deployment ends (§1.12, §6.1). No fee may be charged for that source beyond the actual cost of copying and distribution.
- The modified version must carry a distinct title (§6.0). It may not be named "Galaxy Download Manager" or "GDM".
- Every modification must be documented, the modified version must state that it is derived from Galaxy Download Manager, and the names of the Licensor must appear in its source and in its user-visible copyright notices (§6.2, §6.4(d)).
- Existing copyright notices, change notices and licence text may not be modified or removed without the Licensor's written permission (§6.2).
- Everyone who creates or contributes to a modified version grants the Licensor a worldwide, royalty-free licence to use and distribute those changes (§6.0).

**Trademarks.** The licence grants no rights to the Licensor's trademarks (§5.0). The names "Galaxy Download Manager", "GDM" (for download software) and "Golden IQ", and the Galaxy Download Manager logo, may not be used for any modified version, website, store listing or product, or in any way that suggests official status or endorsement, without the Licensor's written permission. Truthful statements such as "based on Galaxy Download Manager" remain permitted. See [TRADEMARKS.md](TRADEMARKS.md).

**Enforcement.** A use that does not comply with RPL-1.5 or the trademark policy is not licensed. The Licensor will act against infringing builds, websites, store listings and extensions. Failure to act in one case does not waive the right to act in another (§13.4). To report a suspected infringement or request trademark permission, open an issue titled "Trademark request" in this repository.

**Third-party components** remain under their own licences, listed in [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md). Optional external tools that GDM downloads at the user's request, such as yt-dlp and FFmpeg, are separate programs under their own licences and are not part of the installer.

## Supporting the project

GDM is free for everyone. Financial support is voluntary: it unlocks no feature, grants no priority in support or development decisions, and is not the purchase of a product or service. Contributions can be made by scanning the KHQR code below with any banking app that supports KHQR; the same code is shown on the application's About page.

<img src="src/assets/khqr.png" alt="KHQR code for supporting Galaxy Download Manager" width="220">

Organisations that require deployment assistance or training: [docs/services.md](docs/services.md). Sponsors: [docs/sponsors.md](docs/sponsors.md).

## Contributing and security

- Contributions are accepted under RPL-1.5. See [CONTRIBUTING.md](CONTRIBUTING.md).
- Security vulnerabilities must be reported privately as described in [SECURITY.md](SECURITY.md), never in a public issue.

---

Galaxy Download Manager — Copyright (c) 2026 Golden IQ and Chhit Sungchhay. Licensed under the Reciprocal Public License 1.5. https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0
