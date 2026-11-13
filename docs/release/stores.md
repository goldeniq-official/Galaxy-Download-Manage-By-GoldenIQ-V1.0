# ដាក់ GDM និង extension ជាសាធារណៈ

ឯកសារនេះសម្រាប់ម្ចាស់ (GoldenIQ)។ ជំហានទាំងអស់ខាងក្រោមត្រូវការគណនី ឬសិទ្ធិរបស់ម្ចាស់ ដូច្នេះ Claude ធ្វើជំនួសមិនបាន។ ធ្វើតាមលំដាប់: **ID ហាងត្រូវតែមាននៅក្នុង app មុនពេល app ចេញ** ព្រោះ bridge ទទួលតែ extension ដែល ID របស់វាមាននៅក្នុងបញ្ជី។

## ០. មុនគេ (ម្ដងតែប៉ុណ្ណោះ)

1. `git push` (មានតែម្ចាស់ប៉ុណ្ណោះ)។ បន្ទាប់មកមើល GitHub → Actions ថា `core`, `app` និង `scorecard` ឆ្លងទាំងអស់។ នេះជាលើកដំបូងដែល CI ដំណើរការ ដូច្នេះ Linux/macOS អាចមានកំហុសដែលត្រូវកែ។
2. GitHub → Settings → Environments → បង្កើត **release** ហើយដាក់ secrets `TAURI_SIGNING_PRIVATE_KEY` និង `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (ពី `D:\gdm-secrets`)។ បន្ទាប់មកយក `D:\gdm-secrets` ទៅទុកក្រៅកុំព្យូទ័រ។

## ១. ចុះឈ្មោះ extension ក្នុងហាង (មិនទាន់ publish ទេ) ដើម្បីបាន ID

ឯកសារ zip: `pnpm ext` រួច zip **ខ្លឹមសារ** នៃ `extension/dist/chrome` (មិនមែន folder ទេ: `manifest.json` ត្រូវនៅ root) ហើយលុប field `"key"` ចេញពី manifest ក្នុង zip។ `release.yml` ធ្វើបែបនេះដោយខ្លួនឯង (`gdm-extension-chrome-<v>.zip`)។

| ហាង | គណនី | ID |
|---|---|---|
| Chrome Web Store (Chrome, Brave, Vivaldi, Opera) | https://chrome.google.com/webstore/devconsole: ថ្លៃចុះឈ្មោះ $5 ម្ដង, ត្រូវមាន 2-Step Verification | Upload zip → item ថ្មីបង្ហាញ ID ៣២ អក្សរភ្លាម (មិនទាន់ submit) |
| Microsoft Edge Add-ons | https://partner.microsoft.com/dashboard/microsoftedge: ឥតគិតថ្លៃ | Create new extension → upload zip ដដែល → Extension identity បង្ហាញ ID |
| Firefox (AMO) | https://addons.mozilla.org/developers/: ឥតគិតថ្លៃ | ID ថេរ `gdm@goldeniq.dev` (មាននៅក្នុង manifest រួចហើយ) មិនបាច់ធ្វើអ្វីទេ |

បន្ទាប់មក ប្រាប់ ID ទាំងពីរមក Claude (ឬដាក់ខ្លួនឯង) ក្នុង `crates/gdm-bridge/src/lib.rs`:

```rust
pub const CWS_ID: &str = "<ID Chrome Web Store>";
pub const EDGE_ID: &str = "<ID Edge Add-ons>";
```

កន្លែងតែមួយនេះប្ដូរ ៣ យ៉ាងក្នុងពេលតែមួយ: bridge ទទួល extension ពីហាង, ប៊ូតុង "ដំឡើង extension" ក្នុង GDM និងប៊ូតុង "បន្ថែម" ក្នុង GDM Setup នាំទៅទំព័រ GDM ផ្ទាល់ (មុននេះវាបើកទំព័រស្វែងរក)។

## ២. ចេញ app (GitHub Releases)

GitHub → Actions → **version** → Run (មើល [README.md](README.md))។ Draft release មាន៖
- `GDM-Setup_<v>_x64.exe` / `_arm64.exe`: **ឯកសារដែលមនុស្សទាញ** (Setup ដែលរចនាដោយ GDM, មាន NSIS នៅខាងក្នុង)
- `GDM_<v>_<arch>-setup.exe` + `.sig`: NSIS ដដែល សម្រាប់ updater និង winget (`/S`)
- `gdm-extension-chrome-<v>.zip`, `gdm-extension-firefox-<v>.zip`: សម្រាប់ upload ទៅហាង
- `latest.json`, `SHA256SUMS.txt`, attestation

សាក `GDM-Setup` លើកុំព្យូទ័រស្អាតមួយ មុនពេលចុច Publish។ ពេលមាន SignPath: ចុះហត្ថលេខា NSIS **មុន** វាត្រូវបានដាក់ចូលក្នុង Setup ហើយបន្ទាប់មកចុះហត្ថលេខា Setup (ត្រូវប្ដូរ `release.yml` ពេលនោះ)។

## ៣. Submit extension

ចម្លងអត្ថបទខាងក្រោមទៅក្នុង form ហាងនីមួយៗ (ជាភាសាអង់គ្លេស ព្រោះអ្នកពិនិត្យអានអង់គ្លេស)។

**Single purpose:** Hands downloads from the browser to the Galaxy Download Manager desktop app on the same computer.

**Permission justifications:**
- `downloads`: see downloads the browser starts so GDM can take them over, and cancel the browser's own copy only after GDM has accepted it.
- `nativeMessaging`: the only channel to the GDM desktop app on this computer. There is no server.
- `contextMenus`: the "Download with GDM" and "Download all links with GDM" menu items.
- `storage`: the user's own settings (catching on/off, sites not to catch).
- `cookies`: a download that needs the user's login sends that site's cookie to GDM on this computer, as the browser itself would. It is never sent anywhere else.
- `webRequest`: read the method and referrer of a download (e.g. POST forms), and notice video streams (HLS/DASH) on the page the user is viewing.
- Host permissions (all http/https sites): downloads and videos can come from any site; the content script lists a page's links for "Download all links" and finds video players.

**Remote code:** No. All code is in the package.

**Data:** nothing is sent to the developer or any server. Links, file names and a site's cookie go only to the GDM app on the same computer (native messaging). Privacy policy URL: https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0/blob/main/PRIVACY.md

**Notes for the reviewer:** The extension works with the free Galaxy Download Manager desktop app (Windows). Install it from https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0/releases/latest, open it, then click a download link: GDM shows its download window. Without the app the popup says "GDM is not connected" and the browser downloads normally.

**Listing:** ឈ្មោះ និងការពិពណ៌នាមកពី `extension/static/_locales` (អង់គ្លេស + ខ្មែរ) រួចហើយ។ ត្រូវការ screenshot យ៉ាងតិច ១ (1280×800) និង Chrome: រូប promo តូច 440×280។

**Firefox (AMO):** upload `gdm-extension-firefox-<v>.zip`។ កូដត្រូវបាន transpile ពី TypeScript ដូច្នេះ AMO ត្រូវការ source code: upload zip នៃ repo (ឬ `extension/` + `package.json` + `pnpm-lock.yaml`) ហើយសរសេរ build instructions: `pnpm install --frozen-lockfile && pnpm ext` (Node 22+, pnpm)។

លំដាប់: Chrome Web Store → Edge → Firefox។ ក្រោយហាងនីមួយៗអនុម័ត សាក extension ពីហាង ជាមួយ GDM ដែលបានចេញ ថា popup បង្ហាញ "បានភ្ជាប់"។ Chrome អាចពិនិត្យយូរ (ច្រើនថ្ងៃ) ព្រោះ extension សុំសិទ្ធិលើគ្រប់គេហទំព័រ និង cookies។

## ៤. ក្រោយមក

- Key updater (`TAURI_SIGNING_PRIVATE_KEY`) ឥឡូវនៅក្នុង env នៃជំហាន build ដែល `build.rs` របស់ dependency ណាមួយអានបាន។ ក្រោយ CI រត់ជោគជ័យលើកដំបូង: build ដោយគ្មាន key ហើយចុះហត្ថលេខាក្នុងជំហានដាច់ដោយឡែក (`tauri signer sign`)។ មិនទាន់ប្ដូរ ព្រោះវាប៉ះរបៀបបង្កើត `.sig` និង `.app.tar.gz` របស់ macOS ហើយ CI មិនទាន់ធ្លាប់រត់សោះ។

- winget: ដាក់ manifest ទៅ `microsoft/winget-pkgs` ដោយប្រើ `GDM_<v>_x64-setup.exe` (NSIS, `/S`)។
- macOS: Apple Developer ID ($99/ឆ្នាំ) សម្រាប់ notarization (រហូតដល់ពេលនោះ ad-hoc signed ហើយ Gatekeeper នឹងព្រមាន)។
- ល្បឿនធៀប IDM: វាស់លើ NVMe ទំនេរ ≥ 25% មុន v1.0 (មិនទាន់វាស់បានលើម៉ាស៊ីននេះ)។
