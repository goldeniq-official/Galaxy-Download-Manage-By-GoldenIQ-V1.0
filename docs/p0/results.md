# P0 results

## G2: ការចែកចាយ Windows និង browser integration (S1a, S1b, S9)

ថ្ងៃ: 2026-11-09។ កូដ spike (throwaway, មិនចែកចាយ): `spikes/distribution/`។ Plan: `docs/p0/plan.md` (S1a, S1b, S9; ម្ចាស់បញ្ជាក់ 058e165)។
**ស្លាកលទ្ធផល:** រាល់លទ្ធផលខាងក្រោមជា **LOCAL VALIDATION** លើម៉ាស៊ីនម្ចាស់។ គ្មាន **REAL STORE CERTIFICATION** ទេ (មើល Store ខាងក្រោម)។

### បរិស្ថាន

| អ្វី | Version |
|---|---|
| Windows 11 Pro | 26H2, build **26300.9711** (registry ProductName សរសេរ "Windows 10 Pro" ជាចំណុចចាស់របស់ Windows) |
| Windows 10 | **មិនមាន** ម៉ាស៊ីន/VM (មើល "មិនទាន់ធ្វើ") |
| Chrome | 156.0.8078.80 (ដំឡើង per-user) |
| Edge | 157.0.4341.42 |
| Firefox | 159.0.1 (ដំឡើង per-user ក្នុង G2, ព្រោះ winget ជ្រើស MSIX ហើយ policy បដិសេធ) |
| Rust | rustc 1.97.1, cargo 1.97.1 |
| NSIS | 3.11 (zip portable, sha256 `C7D27F78…94FA1`, មិនដំឡើង) |
| Windows SDK | 10.0.26100.0 (makeappx, signtool) |
| Node / puppeteer-core | 24.22.1 / 24.43.1 |
| yt-dlp | 2026.10.26 |
| ម៉ាស៊ីន | គ្មានសិទ្ធិ admin; DPI 125% (120 dpi); Group Policy `AllowAllTrustedApps=0`, `AllowDevelopmentWithoutDevLicense=0` |

### អ្វីដែលបានសង់

- `bridge/` (Rust, `gdm-bridge.exe`): native messaging host stateless; framing 4-byte LE; បដិសេធ length > 256 KiB មុន allocate (B-03); origin allow-list (Chrome/Edge `allowed_origins` + ពិនិត្យ argv[1] ម្ដងទៀត; Firefox `allowed_extensions` + argv[2]) (B-01); protocol typed `deny_unknown_fields` គ្មាន field `dir`/`path` (B-01); URL ≤ 8 KiB, http(s) តែប៉ុណ្ណោះ; handoff មិនចាប់ផ្ដើមអ្វីដោយខ្លួនឯង (B-04)។ ក្នុង binary ដដែល: `register`/`unregister` (helper per-user), `fetch-component` (manifest minisign + sha256 + verify-on-held-handle, U-06)។
- `extension/`: MV3 manifest តែមួយសម្រាប់ Chrome/Edge (`service_worker`) និង Firefox (`scripts`), ID ថេរ `fccmejdfnnlgffbbcbakjpdodjjoeonh` តាម `key`; Firefox ID `gdm-spike@goldeniq.dev`។
- `check-host.cjs`: self-check protocol ១៤ ករណី។ `runner/browsers.cjs`: បើក browser ពិតជាមួយ profile ថ្មី + extension ហើយអាន log របស់ host។
- `nsis/installer.nsi` + `Khmer.nlf` + `Khmer.nsh` (S9), `lifecycle.ps1`, `screens.ps1`។ `msix/` (S1b)។

### លទ្ធផល check នីមួយៗ

| # | Check | Route | លទ្ធផល | ភស្តុតាង |
|---|---|---|---|---|
| 1 | Host protocol: ping/ack, origin បរទេស 100/100 បដិសេធ (exit 3), Firefox ID បរទេស, field `dir` បដិសេធ, length 4 GB បដិសេធមុន allocate, `[`×100k បដិសេធគ្មាន crash, nesting 50k ក្នុង object ដែលមាន tag បដិសេធ (field ទាំងអស់ជា scalar មានប្រភេទ ដូច្នេះ serde បដិសេធនៅ `[` ទីមួយ ដោយមិនចូលជ្រៅ), URL > 8 KiB, `file://`, header ខ្លី, length គ្មាន payload, UTF-8 ខូច, 50 frames = 50 replies; stdout ត្រូវអានអស់ទាំងស្រុងជា frame ពេញ (គ្មាន byte ក្រៅ frame) | — | **PASS** 15/15 | `g2-logs/checks.log`, `g2-logs/rerun-after-council.log` |
| 2 | Browser ឃើញ host (helper per-user, HKCU) | helper | **PASS** Chrome, Edge, Firefox | `g2-logs/browsers-summary.log` [helper-all] |
| 3 | Key ណាដែល browser នីមួយៗអាន (ចុះឈ្មោះម្ដង key មួយ) | helper | Chrome អានតែ `HKCU\Software\Google\Chrome\NativeMessagingHosts\<name>`; **Edge អាន key របស់ខ្លួន `HKCU\Software\Microsoft\Edge\NativeMessagingHosts\<name>` ហើយក៏អាន key Chrome ជា fallback**; Firefox អានតែ `HKCU\Software\Mozilla\NativeMessagingHosts\<name>`។ Default value = path ពេញទៅ manifest JSON | `g2-logs/browsers-summary.log` [only-chrome/only-edge/only-firefox] |
| 4 | ក្នុង browser ពិត extension ផ្ទៀង **reply នីមួយៗ** (ok/type/nonce/ver) នៃ `sendNativeMessage` 100/100, `connectNative` 2/2 (មាន timeout + onDisconnect), និង `ok:false` សម្រាប់ field `dir` ហើយរាយការណ៍ទៅ host ជា ACK (`-all100`, `-ports2`, `-dirrej`); runner PASS លុះត្រាគ្រប់ ACK + 102 PING, exit ≠ 0 ពេល FAIL | NSIS | **PASS** ទាំង ៣ browser លើ 0.1.0 និង 0.2.0 (runner exit 0) | `g2-logs/rerun-after-council.log`, `g2-logs/browsers-summary.log` [rerun-0.1.0, rerun-0.2.0] |
| 5 | NSIS per-user install (`/S`, គ្មាន admin) | NSIS | **PASS** exit 0, 0.67 s; ដំឡើងក្នុង `%LOCALAPPDATA%\Programs\GDM-Spike`, key ទាំង ៣ + uninstall key (HKCU) | `g2-logs/nsis-steps.log` |
| 6 | Launch ពី browser បន្ទាប់ install | NSIS | **PASS** log បង្ហាញ `exe=…\Programs\GDM-Spike\gdm-bridge.exe` (312/312 បន្ទាត់) | `g2-logs/nsis-steps.log` (uniq -c នៃ host.log) |
| 7 | Upgrade 0.1.0 → 0.2.0 (`/S` ពីលើ, browser បិទ) | NSIS | **PASS** DisplayVersion 0.2.0; browser ទាំង ៣ ហៅ host 0.2.0 ពី path ដដែល។ Installer **ចុះឈ្មោះ key ឡើងវិញ** នៅ path ដដែល។ Upgrade ខណៈ host កំពុងរត់ មិនទាន់សាក (P1, U-02) | `g2-logs/rerun-after-council.log` |
| 8 | Uninstall (`/S`) និងគ្មាន key/file កំព្រា | NSIS | **PASS មានលក្ខខណ្ឌ** key NMH ៣ + uninstall key = គ្មាន; `Programs\GDM-Spike`, `GDM\nmh`, `GDM\components` = គ្មាន; ឯកសារ sentinel ក្នុង `%LOCALAPPDATA%\GDM` (ទិន្នន័យដែល installer មិនមែនជាម្ចាស់) **នៅរស់**។ **កំព្រាមួយ:** uninstall នីមួយៗទុក `%TEMP%\~nsuN.tmp\Un.exe` (~64 KB) ព្រោះ NSIS មិនអាចលុប exe ដែលកំពុងរត់ ហើយគ្មាន admin សម្រាប់ delete-on-reboot ⇒ ត្រូវកែក្នុង P1 | `g2-logs/rerun-after-council.log` |
| 9 | T-01: DLL ក្លែង ២០ ឈ្មោះ (version, uxtheme, userenv, cryptbase, dwmapi, comctl32, …) នៅក្បែរ setup.exe | NSIS | **PARTIAL** install silent exit 0 ហើយ UI interactive បើកធម្មតា។ នេះបង្ហាញតែថា installer មិន crash ទេ មិនមែនភស្តុតាងថាគ្មាន DLL ណាត្រូវ load ពីថតនោះ (គ្មានភស្តុតាង module-load) ⇒ P1: ProcMon ឬ DLL សាកដែលកត់ត្រាការ load | ពាក្យបញ្ជាក្នុង transcript G2 (ដាក់ junk DLL ហើយរត់ `/S`) |
| 10 | Component: yt-dlp ទាញក្រោយ install ទៅ `%LOCALAPPDATA%\GDM\components` | NSIS | **PASS** manifest signature OK; sha256 `f5389d36…7a7d` ត្រូវនឹង `SHA2-256SUMS` ផ្លូវការ; write ពេលកាន់ handle ត្រូវរារាំង; `yt-dlp --version` = 2026.10.26 | `g2-logs/nsis-steps.log` |
| 11 | Component អវិជ្ជមាន: manifest ដែលគេកែ ⇒ បដិសេធ; sha256 ខុស ⇒ បដិសេធ ហើយ **គ្មានឯកសារនៅសល់** | — | **PASS** (bug ដែលរកឃើញ ហើយកែរួច: ពីមុនឯកសារខុស hash នៅសល់ ព្រោះ handle ដែលហាម delete នៅបើក; ឥឡូវ verify លើ `.part` មុន rename ហើយ verify ម្ដងទៀតលើ handle ដែលកាន់) | `g2-logs/checks.log`, `bridge/src/main.rs` `fetch` |
| 12 | S9 Khmer installer: welcome, folder, installing/finish, uninstall confirm/done | NSIS | **PASS** shaping ត្រឹមត្រូវ (ជើង, ស្រៈផ្សំ: ស្វាគមន៍, ក្នុង, ត្រូវការ, កុំព្យូទ័រ), ចំណងជើង/អត្ថបទ/ប៊ូតុងគ្មានកាត់; uninstaller ចាំភាសាខ្មែរ (`InstallerLanguage=1107` ក្នុង HKCU)។ **បញ្ហាដែលរកឃើញ ហើយកែរួច:** ប៊ូតុង "បង្ហាញព័ត៌មានលម្អិត" ត្រូវកាត់ ⇒ ប្ដូរជា "លម្អិត"; `installer.nsi` គ្មាន BOM ⇒ mojibake ⇒ ដាក់ BOM | `g2-screens/dpi-125/` |
| 13 | S9 ផ្ទាំងជ្រើសភាសា (LangDLL) | NSIS | **PARTIAL ⇒ plan B ត្រូវបានសាក ហើយ PASS** ខ្មែរត្រូវបានជ្រើសដំបូង; នៅ 175% "ខ្មែរ" អានបានច្បាស់ ប៉ុន្តែនៅ 100–125% តូច ព្រោះ LangDLL មិនទទួល font។ **Plan B** (`-DNOLANGDLG`: គ្មានផ្ទាំង, ខ្មែរលំនាំដើម, ប្ដូរភាសាក្នុង app តាម R6): install/uninstall ជាខ្មែរទាំងស្រុង គ្មានកាត់ | `g2-screens/dpi-*/01-language.png`, `g2-screens/nolang-125/` |
| 14 | S9 DPI | NSIS | **PASS 100% / 125% / 150% / 175%**: គ្រប់ទំព័រ install + uninstall គ្មានអក្សរខ្មែរកាត់ (window 585×501 → 999×768)។ **200%: មិនអាចសាកលើអេក្រង់នេះ** (Windows កំណត់ត្រឹម 175%; step 4 = 168 dpi)។ ប្ដូរ scale តាម `SPI_SETLOGICALDPIOVERRIDE` ដោយគ្មាន sign out ហើយស្ដារ 125% របស់ម្ចាស់វិញ (ផ្ទៀង: system dpi = 120)។ កំហុសខ្ញុំ: ការរត់ដំបូងស្ដារទៅ step 0 = 100% ខុស ⇒ កែភ្លាមក្នុងការរត់បន្ទាប់ | `g2-logs/dpi.log`, `g2-screens/dpi-100…175/` |
| 15 | S9 អ្នកអានខ្មែរពិនិត្យ | NSIS | **រង់ចាំម្ចាស់** ពិនិត្យរូប `g2-screens/` (shaping ពិនិត្យដោយ Claude ហើយមើលទៅត្រឹមត្រូវ ប៉ុន្តែមិនជំនួសអ្នកអានខ្មែរ) | — |
| 16 | S1b MSIX: build + ចុះហត្ថលេខា | MSIX | **PASS** `makeappx pack` (schema ឆ្លង) ជាមួយ `desktop6:RegistryWriteVirtualization=disabled`, `desktop6:FileSystemWriteVirtualization=disabled`, `rescap:unvirtualizedResources`, `uap3` appExecutionAlias `gdm-bridge.exe`, `uap10:Parameters="register"`; `signtool` ជាមួយ cert self-signed `CN=GoldenIQ Spike` | `msix/AppxManifest.template.xml`, `msix/build.ps1` |
| 17 | S1b MSIX: install | MSIX | **BLOCKED (លើម៉ាស៊ីន/config នេះ)** (1) cert ក្នុង CurrentUser\TrustedPeople: `0x800B0109` root មិនទុកចិត្ត ⇒ ទំនងត្រូវ LocalMachine\TrustedPeople (admin) ប៉ុន្តែមិនទាន់បានសាក; (2) package គ្មានហត្ថលេខា (publisher OID) + `-AllowUnsigned`: `0x80073D2B` "an unsigned package cannot include Executable activations"; (3) policy `AllowAllTrustedApps=0` បដិសេធ MSIX របស់ Firefox ពី winget (`0x80073CFF`)។ សេចក្ដីសន្និដ្ឋានទាំងនេះសម្រាប់ config នេះប៉ុណ្ណោះ | `g2-logs/msix.log` |
| 18 | S1b MSIX: browser ឃើញ host, upgrade, uninstall, component ក្នុង package | MSIX | **NOT RUN ⇒ plan B ត្រូវបានសាក ហើយ PASS**: goal ជំហាន 2 "if that fails, try a separate per-user installer helper" = `gdm-bridge register` (#2, #3) និង NSIS (#4–#8, #10)។ MSIX ពេញលេញធ្វើមុន Store (D9) | — |
| 19 | Windows 10: install/launch/upgrade/uninstall | NSIS | **NOT RUN** គ្មាន Windows 10 | — |
| 20 | Store submission (D9) | Store | **N/A**: goal ជំហាន 6 ធ្វើតែ "if the owner has a Store account"; D9 = Store ពន្យារ, គ្មានគណនី ⇒ គ្មាន REAL STORE CERTIFICATION | plan.md D9 |

សំណល់ដែលកើតពីការព្យាយាម MSIX (មិនមែនពី NSIS): Windows Shell បង្កើត `HKCU\Software\Microsoft\Windows\CurrentVersion\CloudStore\…\windows.data.apps.appmetadata$goldeniq.galaxydownloadmanager.spike` ពេល `Add-AppxPackage` បរាជ័យ។ វាជាទិន្នន័យរបស់ Windows ដែលខ្ញុំមិនបានលុប។

### ផ្លូវដែលជ្រើស

**NSIS per-user ជាផ្លូវចម្បង** (ត្រូវនឹង D8)។ Installer ចុះឈ្មោះ host ក្នុង HKCU key ទាំង ៣ ហើយ uninstaller លុបវាវិញ។ លើ Windows 11 ផ្លូវនេះត្រូវបានបង្ហាញថាដំណើរការ (#1–#8, #10–#12) ជាមួយចំណុចខ្វះ: #8 (`Un.exe` នៅ %TEMP%), #9 (PARTIAL), #13 (PARTIAL ⇒ plan B PASS), #14 (200% មិនមានលើអេក្រង់នេះ), #15 (រង់ចាំម្ចាស់), #19 (NOT RUN)។ ការចែកចាយតាម GitHub Releases / winget / Tauri updater ពិតប្រាកដ **មិនទាន់បានបង្ហាញ** ក្នុង G2 ទេ (វាជាការងារ P1/release.yml)។
**MSIX:** package build បាន ប៉ុន្តែការដំឡើង ដូច្នេះក៏ការមើលថាតើ browser ឃើញ host ឬទេ មិនអាចសាកលើម៉ាស៊ីននេះបានទេ។ តាម plan S1b វាជា probe ½ ថ្ងៃ ហើយ spike ពេញធ្វើមុនដាក់ Store (D9)។

**ស្ថានភាព G2: មិនទាន់រួចរាល់ពេញលេញ។** MSIX មាន plan B ដែលបានសាក (#18) ហើយ Store = N/A (#20)។ នៅសល់: **Windows 10 (#19)** គ្មានលទ្ធផល, **200% DPI** មិនអាចសាកលើអេក្រង់នេះ, **អ្នកអានខ្មែរ (#15)** ⇒ ត្រូវការម្ចាស់: ផ្ដល់ម៉ាស៊ីន/VM Windows 10 (និងអេក្រង់ដែលគាំទ្រ 200%) ឬយល់ព្រមពន្យារ ហើយពិនិត្យរូបខ្មែរ។

### អ្នកម្ចាស់ update តាមផ្លូវ

| Route | App | Components (yt-dlp/ffmpeg) |
|---|---|---|
| NSIS / GitHub Releases / winget (ចម្បង) | Tauri updater + minisign; installer ថ្មី `/S` ពីលើកំណែចាស់ ដែលចុះឈ្មោះ key ឡើងវិញនៅ path ដដែល (#7, ពេល browser បិទ) | `components.json` ចុះហត្ថលេខា minisign + sha256 + verify-on-handle (#10, #11) |
| MSIX / Store (ក្រោយ) | Microsoft Store; updater ក្នុង app បិទ | ដូចគ្នា (មិនទាន់សាកក្នុង package: #18) |

### ការរកឃើញសម្រាប់ P1

1. **Edge អាន key Chrome ជា fallback។** P1 នៅតែត្រូវសរសេរ key Edge ផ្ទាល់ ដើម្បីកុំពឹងលើឥរិយាបថនោះ ហើយ uninstall ត្រូវលុបទាំងពីរ។
2. **Upgrade ខណៈ host កំពុងរត់** (browser កាន់ `connectNative`) មិនទាន់សាកទេ។ NSIS `File` អាចបរាជ័យលើ exe ដែលកំពុងប្រើ ⇒ P1/S8 ត្រូវសាក (U-02: install តែពេល idle)។
3. **LangDLL** មិនទទួល font ⇒ ក្នុង P1 សាក `LangDLL` ដែលកែ (fork តូច) ឬលុបផ្ទាំងជ្រើសភាសា ហើយប្រើខ្មែរលំនាំដើម + ប្ដូរភាសាក្នុង app (plan S9 fallback, R6)។
4. **ឯកសារ NSIS ត្រូវមាន UTF-8 BOM** (`.nsi`, `.nsh`) បើមិនដូច្នោះ អក្សរខ្មែរខូច។ Tauri bundler បង្កើត template ខ្លួនឯង ⇒ ពិនិត្យក្នុង P1។
5. Khmer.nlf/.nsh ក្នុង spike បកប្រែតែទំព័រដែលប្រើ; P1 ត្រូវបកប្រែពេញ (ប្រហែល ២៥០ ឃ្លា) ហើយឱ្យអ្នកអានខ្មែរពិនិត្យ។
6. Spike `gdm-bridge.exe` ជា release ទំហំ 1.96 MB (ureq + rustls + minisign នៅក្នុង binary ដដែល)។ P1 bridge ពិតមិនត្រូវការ HTTP/minisign ទេ ⇒ ដាក់ `fetch-component` នៅក្នុង app មិនមែនក្នុង bridge។
7. **Uninstaller ទុក `%TEMP%\~nsuN.tmp\Un.exe`** (#8)។ P1: សម្អាតថតនោះពេលបើក installer/app លើកក្រោយ ឬប្រើ uninstall ក្នុងកន្លែង (`_?=`) ពី app រួចលុប `uninstall.exe` តាមវិធីផ្សេង។
8. **U-06 មិនទាន់ពេញលេញ** (council): `Command::new(path)` resolve path ម្ដងទៀតក្រោយ verify លើ handle ⇒ ថតមេ/ancestor junction អាចត្រូវប្ដូរ។ ការពិនិត្យ reparse ធ្វើតែលើថត `components` ម្ដងប៉ុណ្ណោះ។ G3/S8 ជាម្ចាស់ U-06។
9. **ភាពជាម្ចាស់នៃការចុះឈ្មោះ** (council): `register` សរសេរជាន់ ហើយ `unregister` លុបដោយមិនពិនិត្យថា manifest/exe ជារបស់ install នេះឬទេ ⇒ install A, install B នៅ path ផ្សេង, uninstall A ⇒ B បាត់ integration។ P1: unregister តែពេល path ក្នុង manifest = `$INSTDIR`។
10. **Component fetch** (council): `.part` ឈ្មោះអាចទាយបាន ហើយ `File::create` truncate/follow link ⇒ P1 ប្រើ `create_new` + ឈ្មោះចៃដន្យ, ដែនកំណត់ទំហំ, timeout លើ child, លុប `.part` លើគ្រប់ផ្លូវកំហុស, និងចាត់ exit ≠ 0 របស់ child ជាបរាជ័យ។
11. Installer ឥឡូវ Abort ពេល `register` បរាជ័យ (exit ≠ 0) ប៉ុន្តែ `unregister` នៅតែរាយការណ៍ "absent" សម្រាប់គ្រប់កំហុស ⇒ P1។

### មិនទាន់ធ្វើ / ត្រូវការម្ចាស់

- **Windows 10** (#19): ត្រូវការ VM ឬម៉ាស៊ីន Windows 10។ ពាក្យបញ្ជាដដែល (ផ្នែក "របៀបរត់ឡើងវិញ") រត់បានដោយគ្មានការកែ។
- **MSIX install** (#17, #18): ត្រូវរត់ជា admin មួយដង ហើយទទួលយកថា policy របស់ម៉ាស៊ីនប្ដូរ: `Import-Certificate out\spike.cer -CertStoreLocation Cert:\LocalMachine\TrustedPeople` និង `AllowAllTrustedApps=1`។ ខ្ញុំមិនធ្វើវាទេ (គ្មាន admin ហើយវាប្ដូរ security policy របស់ម៉ាស៊ីនម្ចាស់)។ តាម plan S1b អាចពន្យាររហូតដល់ Store (D9)។
- **DPI 200%** (#14): អេក្រង់នេះឈប់ត្រឹម 175%។ ត្រូវការអេក្រង់ធំជាង ឬ VM ដែលមានគុណភាពបង្ហាញខ្ពស់។
- **អ្នកអានខ្មែរ** (#15): ម្ចាស់ពិនិត្យ `docs/p0/g2-screens/`។

### របៀបរត់ឡើងវិញ

```
cd spikes/distribution
cargo build --release --manifest-path bridge/Cargo.toml        # env GDM_SPIKE_PUBKEY=$(cat out/spike.pub), GDM_SPIKE_VER=0.1.0
node check-host.cjs bridge/target/release/gdm-bridge.exe
bridge/target/release/gdm-bridge.exe register <exe> [--only chrome|edge|firefox]
node runner/browsers.cjs <label> [chrome edge firefox]
tools/nsis-3.11/makensis.exe -DVERSION=0.1.0 -DBRIDGE=<exe> -DOUT=<setup.exe> <abs path>/nsis/installer.nsi
out/GDM-Spike-Setup-0.1.0.exe /S ; out/GDM-Spike-Setup-0.2.0.exe /S ; "%LOCALAPPDATA%\Programs\GDM-Spike\uninstall.exe" /S
gdm-bridge.exe fetch-component out/components.json out/components.json.minisig
powershell -File nsis/screens.ps1 -Out docs/p0/g2-screens/dpi-125   [-Setup GDM-Spike-Setup-0.1.0-nolang.exe -NoLang]
powershell -File nsis/dpi.ps1 -Out docs/p0/g2-screens -Steps 2 -Restore 1   (one step per run; restores the owner's 125%)
powershell -File msix/build.ps1 -Version 0.1.0.0 ; Add-AppxPackage out/gdm-spike-0.1.0.0.msix
```

`out/` និង `tools/` មិនត្រូវ commit (`.gitignore`)។ Log សំខាន់ៗ ត្រូវបានចម្លងទៅ `docs/p0/g2-logs/` (`checks.log`, `browsers-summary.log`, `nsis-steps.log`, `msix.log`, `rerun-after-council.log`, `dpi.log`)។ `nsis-steps.log` ជាការរត់មុន council (runner ចាស់ ដែល PASS មិនតម្រូវ ports/dir); លទ្ធផលដែលប្រើក្នុងតារាងគឺ `rerun-after-council.log`។

### Council (ChatGPT/Codex, read-only, 2 ជុំ, 2026-11-09)

**Codex បានបន្ថែម (ផ្ទៀងផ្ទាត់ជាមួយកូដ ហើយពិតទាំងអស់):** runner សម្រេច PASS ដោយមិនតម្រូវ ports/dir ហើយ exit 0 ជានិច្ច; `RMDir /r $LOCALAPPDATA\GDM` អាចលុបទិន្នន័យដែលមិនមែនជារបស់ installer; លទ្ធផល `nsExec` មិនត្រូវបានពិនិត្យ; #9 ខ្លាំងពេក (គ្មានភស្តុតាង module-load); #7 អះអាងខុសថា "មិនបាច់ចុះឈ្មោះឡើងវិញ"; #8 ពិនិត្យតែ `Un_*.exe`; check-host មិនតម្រូវ stdout អានអស់; U-06 path re-resolution; ភាពជាម្ចាស់នៃការចុះឈ្មោះ; `.part` ដែលទាយបាន។
**ការសម្រេច (Claude ជាអ្នកដឹកនាំ):** កែក្នុង spike ហើយរត់ check ដែលរងផលប៉ះពាល់ឡើងវិញ: harness (#4), uninstall មានតែអ្វីដែលខ្លួនជាម្ចាស់ + sentinel (#8), Abort ពេល register បរាជ័យ, check-host (#1), កែស្លាក #7 #8 #9 #17។ ការរត់ឡើងវិញក៏បានរកឃើញ `Un.exe` នៅ %TEMP% (#8) ដែលការពិនិត្យមុនបានខកខាន។ ចំណុចផ្សេងទៀតត្រូវបានកត់ទុកជា P1 (ការរកឃើញ 7–11)។
**ការឯកភាព:** Codex យល់ព្រមថាធាតុ P1 មិនរារាំងការបញ្ជាក់ផ្លូវ NSIS ទេ (S8 ជាម្ចាស់ U-06/U-02) ហើយផ្ដល់យោបល់កុំពង្រីកការ Abort ទៅជា rollback engineering។ **ទម្លាក់:** ការរត់ឡើងវិញទាំងស្រុង (រត់តែ check ដែលរងផលប៉ះពាល់)។
**សេចក្ដីសន្និដ្ឋានរួម:** ផ្លូវ NSIS per-user ត្រូវបានបង្ហាញលើ Windows 11 ប៉ុន្តែ G2 **មិនទាន់រួចរាល់ពេញលេញ** រហូតដល់ Windows 10, DPI, អ្នកអានខ្មែរ មានលទ្ធផល ឬម្ចាស់យល់ព្រមពន្យារ។


## G3: ភាពត្រឹមត្រូវ engine និងព្រំដែនសុវត្ថិភាព (S2–S8) + benchmark ធៀប IDM 6.43

ថ្ងៃ: 2026-11-09។ កូដ spike (throwaway, មិនចែកចាយ): `spikes/engine/` (crate តែមួយ, binary មួយក្នុងមួយ spike, `src/fixture.rs` = S4 fixture ដែល P1 យកទៅ `fixtures/`)។ Plan: `docs/p0/plan.md` (ម្ចាស់បញ្ជាក់ 058e165)។ សេចក្ដីសម្រេចដែល session ធ្វើ: `docs/decisions.md`។
**ស្លាកលទ្ធផល:** រាល់លទ្ធផលជា **LOCAL VALIDATION** លើម៉ាស៊ីនម្ចាស់ (គ្មាន admin)។ "LAN" = loopback fixture ដែលកំណត់ល្បឿន មិនមែន NIC ពិត។ `TerminateProcess` បង្ហាញតែការ recover ពី process crash មិនមែនដាច់ភ្លើងទេ។

**សេចក្ដីសន្និដ្ឋានរួម:** G3 បិទជា **ការវាយតម្លៃ** ដែលមាន **gate មួយធ្លាក់**។ S2–S8 មាន check ដែលរត់ឡើងវិញបាន ហើយ PASS ទាំងអស់។ ចំណុចដែលមិនអាចសាកនៅទីនេះមាន fallback កត់ទុក។ Gate ល្បឿនផលិតផល (durable) ធៀប IDM **ធ្លាក់** លើម៉ាស៊ីននេះ (មើល "Benchmark")។ Gate 90% goodput ឆ្លងតែផ្លូវ network របស់ engine លើ loopback ប៉ុណ្ណោះ។

### បរិស្ថាន

| អ្វី | Version / ស្ថានភាព |
|---|---|
| Windows 11 Pro | build 26300; គ្មានសិទ្ធិ admin |
| CPU / RAM | Intel i5-12400F (12 threads) / 31.8 GB |
| C: | ADATA LEGEND 710 NVMe (DRAM-less), **ពេញ 97%** (ទំនេរ 16.8 GB), **0% idle** ដោយ I/O ផ្ទៃក្រោយ ~40 MB/s ដែលមិនមែនរបស់ GDM/IDM |
| D: | WD10EZEX 7200 rpm HDD, ទំនេរ 76%, idle |
| Network | Ethernet Intel I219-V 1 Gbps; Internet ≈ 9–12 MB/s ក្នុង 1 connection (curl, 2026-11-09 16:5x) |
| AV | Defender real-time **បិទ**; Bitdefender ដំឡើង (ដូចគ្នាទាំង GDM និង IDM) |
| IDM | 6.43 build 10 (`IDMan.exe` FileVersion 6.43.10.3, sha256 `6C047F10…9E0254EF5`), connection លំនាំដើម 8 (គ្មាន override ក្នុង registry) |
| Rust | rustc / cargo 1.97.1; tokio 1.54.1, reqwest 0.12.28 (hyper 1.12.0, rustls 0.23.46), windows 0.61.3, minisign 0.7.9 / minisign-verify 0.2.5, sha2 0.10.9, serde_json 1.0.154 |
| yt-dlp | 2026.10.26 (sha256 `f5389d36…7a7d` = `SHA2-256SUMS` ផ្លូវការ) |
| ffmpeg / ffprobe | N-128691-gcc583787bb-20261108 (sha256 `0d6723a6…0b8553` / `c070df72…466b6e`) |
| iperf3 | មិនមាន (LAN gate ប្រើ fixture ដែលដឹង cap ច្បាស់) |

### លទ្ធផល check នីមួយៗ

ពាក្យបញ្ជា: `powershell -File spikes/engine/check.ps1 <spike>` (build ទៅ `%LOCALAPPDATA%\gdm-spike-target`, exit 0 = PASS គ្រប់បន្ទាត់)។ Log ពេញ: `docs/p0/g3-logs/<spike>.txt`។

| Spike | Check | លទ្ធផល | អ្វីដែលបានបង្ហាញ | មិនបានបង្ហាញ ⇒ fallback / P1 |
|---|---|---|---|---|
| S4 hostile fixture | `check.ps1 s4` | **PASS 18/18** (9 s) | តាម wire ដោយ TCP ឆៅ: 206 ត្រឹមត្រូវ, If-Range ខុស ⇒ 200, 416, 200 មិនអើពើ Range, 206 រួច 200 ពាក់កណ្ដាល, ETag ប្ដូរ, body គ្មានទីបញ្ចប់, លើស/ខ្វះ, Content-Range ខុស, slowloris, slow body, redirect ឆ្លង origin, 429 + Retry-After, text/html, multipart/byteranges, log request + header | ផែនទី "fixture ⇒ លទ្ធផល engine" នៅក្នុង S5 (ខាងក្រោម) |
| S5 segmenter | `check.ps1 s5` (30 រង្វង់) និង `check.ps1 s5 1000` | **PASS 36/36**; 1000 រង្វង់: **PASS 36/36** (41 នាទី) | Probe `Range: bytes=0-` + `Accept-Encoding: identity`; ramp ពី 2, +1/2 s ពេលល្បឿនកើន ≥ 10%, cap 8; split ពាក់កណ្ដាល ≥ 1 MiB; If-Range ជាមួយ strong ETag; fixture S4 ទាំងអស់ ⇒ ឯកសារត្រឹមត្រូវ ឬ state កំហុសត្រឹមត្រូវ (0 ឯកសារខូច); redirect ឆ្លង origin (port/host) ⇒ គ្មាន Cookie/Authorization + control same-origin; failpoint កំណត់ ៦ (`lease_revoke`, `after_ack`, `mid_write`, `after_sync`, `after_sidecar_tmp`, `after_commit`) ⇒ byte ដែល commit ត្រូវលើ disk + resume sha ត្រូវ; crash មុន rename ចុងក្រោយ ⇒ resume ស្នើតែ 320 KiB; sidecar ខូចរចនាសម្ព័ន្ធ ⇒ ចោល; **sidecar រចនាសម្ព័ន្ធត្រឹមត្រូវតែកុហក** ⇒ seam check ចាប់ ⇒ sha ត្រូវ; **kill 1000 ដង: 0 byte ខូចលើ disk, 0 progress ថយ, 25/25 ឯកសារ sha256 ត្រូវ** | Gate ល្បឿន durable ធ្លាក់ (Benchmark)។ **ដែនកំណត់ (NOTE):** seam check ពិនិត្យតែព្រំដែន segment; ការខូចខាងក្នុង prefix ដែល commit មិនត្រូវបានរកឃើញ ⇒ ត្រូវការ SHA-256 ដែលដឹងមុន។ Write ចាស់មិនអាចសរសេរក្រោយ split: worker សរសេរ និងដោះស្រាយ Shrink ក្នុង loop តែមួយតាមលំដាប់ ⇒ ack តែពេល cursor < mid ហើយ clamp គ្រប់ write ក្រោយ (អំណះអំណាងរចនាសម្ព័ន្ធ + failpoint, council យល់ព្រម) |
| S6 finalisation | `check.ps1 s6` | **PASS 8/8** | Byte count ≠ total ⇒ បរាជ័យ; sha256 ខុស ⇒ បរាជ័យ; `IAttachmentExecute::Save` S_OK លើ NTFS ⇒ `Zone.Identifier` ZoneId=3 (+HostUrl តាម policy) នៅរស់ក្រោយ rename; rename តាម handle `ReplaceIfExists=false` ⇒ ឯកសារអ្នកប្រើមិនត្រូវជំនួស (0x800700B7); EICAR ⇒ **"ត្រូវបានរារាំង"** គ្មានឈ្មោះចុងក្រោយ; FS ឈ្មោះ exFAT ⇒ **"គោលដៅមិនការពារ"** | **FAT/exFAT ពិត និង SMB: មិនបានសាក** (គ្មាន admin សម្រាប់ VHD, គ្មាន USB/share) ⇒ `s6 <path>` រត់លើ volume ពិតពេលមាន។ **AV:** Bitdefender មិនផ្ដល់ error code ទេ (វាស់: write/read តាម handle ជោគជ័យ, លុបដោយស្ងាត់ ~1.5 s ក្រោយ close) ⇒ "ត្រូវបានរារាំង" = "សង្ស័យ security software" ពេលឯកសារបាត់ ហើយ probe ស្លូតនៅរស់; ឯកសារដែល process ផ្សេងលុបក៏ត្រូវចាត់ដូចគ្នា (NOTE) ⇒ P1: Defender MpThreatDetection / AMSI ជាភស្តុតាងវិជ្ជមាន |
| S3 named pipe | `check.ps1 s3` | **PASS 19/19** (រត់ ៣ ដង ដូចគ្នា) | DACL អានវិញ = `D:P(A;;FA;;;<SID user>)` (protected, ACE តែមួយ); principal ផ្សេង (anonymous token, និង token ដែល SID user ជា deny-only) បើក/បង្កើត instance ⇒ ERROR_ACCESS_DENIED, control: principal ដដែលបើក pipe DACL លំនាំដើមបាន; instance ទី ២ ដោយ user ដដែល ⇒ 231; server ឃើញ client ត្រឹម SecurityIdentification (SQOS); message 100 KiB ⇒ បដិសេធ; JSON ជ្រៅ 5000 ⇒ បដិសេធ (recursion limit 128) ហើយ server នៅរស់; `\\127.0.0.1\pipe\…` ⇒ បដិសេធ (control: pipe គ្មាន flag ចូលបាន); **pipe squat មុន ⇒ server ពិតបរាជ័យដោយបិទ (FAILCLOSED 5), client បដិសេធ 50/50, squatter ទទួល 0 message**, control: client មិនផ្ទៀង លេចធ្លាយទៅ squatter | Account ទីពីរពិត / session ផ្សេង: មិនបានសាក (ត្រូវការ admin)។ Client ផ្ទៀងតែ image path (fallback មុន SignPath); **ACL ថតដំឡើង មិនទាន់ implement** (P1); Authenticode ក្រោយ SignPath ត្រូវចងនឹង image របស់ process handle ដែលកាន់ មិនមែន path។ `nMaxInstances=1` ក្នុង spike; P1 ត្រូវការ instance ច្រើន ⇒ client verify ជាអ្នកការពារ។ Same-user នៅក្រៅ threat model |
| S2 handoff (ខាង host) | `check.ps1 s2` | **PASS 18/18** | Origin ត្រឹមត្រូវ + cookie ក្នុង scope ⇒ ទទួល, stdout មានតែ frame, គ្មានឯកសារសរសេរ; origin ផ្សេង, message 10 MB (> 64 KiB មុនអាន body), header ខ្លី/កាត់, UTF-8/JSON ខូច, cookie ក្រៅ scope (hostOnly, site ផ្សេង, Secure ទៅ http, Path), header injection, `file://`, userinfo, ticket ផុត ឬ > 60 s, field មិនស្គាល់ (ឧ. `path`) ⇒ បដិសេធ | ខាង browser ពិត (MV3, `chrome.cookies`) = G2 S1a (Chrome/Edge/Firefox ពិត, `sendNativeMessage` 100/100 + `connectNative` 2/2, `docs/p0/g2-logs/`)។ **Ticket durable + failure injection + 200 handoff: មិនទាន់ធ្វើ** ⇒ P1។ "Cookie មិនរក្សាទុក" = គោលការណ៍ app (log, ticket, diagnostics) ត្រូវផ្ទៀងក្នុង P1 |
| S7 yt-dlp proxy + media gate | `check.ps1 s7` | **PASS 26/26** | គ្មាន/ខុស Proxy-Authorization ⇒ 407; SSRF matrix 23/23 បដិសេធ (GET + CONNECT); CONNECT authority ចម្លែក ⇒ បដិសេធ; yt-dlp `-J --ignore-config --no-plugin-dirs` + `--` ⇒ URL មេឌៀ; redirect ទៅ loopback ⇒ **proxy REFUSED លើ connection ផ្ទាល់ខ្លួន (403 Forbidden by GDM policy)** + control redirect ទៅទំព័រដែលអនុញ្ញាតដំណើរការ; `--exec=…` ក្រោយ `--` ⇒ គ្មាន command រត់; engine: literal loopback, `localtest.me` (DNS សាធារណៈ ⇒ 127.0.0.1), redirect hop ទៅ loopback ⇒ Forbidden មុន connect (canary 0) + control; DNS answer public+private ⇒ បដិសេធទាំងមូល; វីដេអូ+សំឡេងដាច់ ⇒ engine ទាញ ⇒ ffmpeg mux (`-protocol_whitelist file,pipe`) ⇒ ffprobe video,audio; progressive ⇒ video,audio; ffmpeg បដិសេធ http input និង playlist ដែលចង្អុល http; proxy បិទក្រោយ run | **ការធានា ADR-6 = តែ request ដែលគោរព `--proxy`**; extractor ដែលរំលង proxy មិនត្រូវបាន confine (គ្មាន WFP, គ្មាន admin) ⇒ "មិនគាំទ្រ" (M-08) / WFP user-mode ពិនិត្យ P2។ DNS rebinding server ពិតមិនបានរត់ (រចនាសម្ព័ន្ធ: lookup មួយក្នុងមួយ connection ហើយ connect តែទៅ address ដែលបានពិនិត្យ)។ Job Object limits មិនក្នុង spike ⇒ P1។ Proxy មិនបញ្ជូន request body លើស read ដំបូង (yt-dlp `-J` មានតែ GET) |
| S8 update + recover | `check.ps1 s8` | **PASS 12/12** | 1.1.0 ចុះហត្ថលេខា ⇒ ដំឡើង; byte ប្ដូរ ⇒ sha256 បដិសេធ; sha256 ក្លែងត្រូវគ្នា ⇒ minisign បដិសេធ; key ផ្សេង ⇒ បដិសេធ; កំណែចាស់ / ដូចគ្នា ⇒ បដិសេធ; កាន់ handle (share READ) ⇒ overwrite/delete/rename ឯកសារ staged ត្រូវរារាំង; installed bytes = verified bytes (កញ្ចប់អានចូល memory តាម handle ដែលកាន់, verify + extract ពី bytes ដដែល); crash នៅជំហាននីមួយៗ ⇒ 1.0.0 ឬ 1.1.0 ដែល verify ពេញ; **kill ចៃដន្យ 20/20 ⇒ launcher ចាប់ផ្ដើម version ដែល verify** (8 × 1.0.0, 12 × 1.1.0); ឯកសារ version ថ្មីខូច ⇒ ត្រឡប់ 1.0.0 + flag "Update បរាជ័យ" | **ផ្លូវដែលបាន PASS = plan B (ថត A/B + pointer)** ត្រូវបានទទួលយកជាផ្លូវ P0; NSIS passive in-place មិនមែនផ្លូវដែលបានសាក។ "Install តែពេល idle" (U-02), ACL staging/reparse: P1 |

### Bug ដែលរកឃើញ និងកែក្នុង G3

1. **Fixture pacer** គេងរាល់ chunk 256 KiB ហើយ timer Windows 15.6 ms ⇒ cap ~17 MB/s ក្នុងមួយ connection (ផ្ដល់អត្ថប្រយោជន៍ IDM 8 conn) ⇒ គេងតែពេលលឿនមុន ≥ 20 ms + credit 20 ms (1 conn 125.6 MB/s, 8 conn 124 MB/s)។
2. **Fixture response 200** (request គ្មាន Range) មិនជាប់ cap; trace បង្ហាញថា connection ទី ១ របស់ IDM គ្មាន Range ⇒ run LAN ដំបូងលំអៀង ហើយត្រូវបោះចោល (`bench/lan-baseline-*` ទុកជាភស្តុតាងបញ្ហាតែប៉ុណ្ណោះ)។ Trace ក៏បង្ហាញ IDM ទាញ overlap 64 byte នៅព្រំដែន segment។
3. **Zero-fill NTFS**: write នៅកណ្ដាលឯកសារក្រោយ `set_len` ⇒ NTFS zero-fill ចន្លោះ (4 conn 12.6 s ទល់ 8.6 s) ⇒ `.part` sparse, ដក flag ពេល finalise (sparse មិនកក់ទំហំ ⇒ P1 ពិនិត្យទំហំទំនេរមុន)។
4. **HTTP/2**: segment ក្លាយជា stream លើ TCP តែមួយ (CDN: segment 3, TCP 1, 9.9 MB/s) ⇒ segmenter ប្រើ HTTP/1.1 ដូច IDM។
5. **Partial write**: ចំនួន byte ពី `seek_write` មិនត្រូវបានពិនិត្យ ⇒ loop រហូតអស់។
6. **Proxy S7**: request ទី ២ លើ connection keep-alive ទៅ upstream ចាស់ដោយមិនពិនិត្យ (PASS មុននោះទទេ) ⇒ request មួយក្នុង connection មួយ + `Connection: close`។
7. **Engine S7**: ពិនិត្យ ហើយ reqwest resolve ម្ដងទៀត, redirect មិនពិនិត្យ ⇒ resolver ដែលពិនិត្យ + ពិនិត្យ literal នៅរាល់ hop។
8. **Probe offset**: ពេល seam check ផ្លាស់ byte ខ្វះដំបូង, body របស់ probe អាចត្រូវសរសេរខុសទីតាំង ⇒ ផ្ដល់ probe លុះត្រា offset ដូចគ្នា។
9. **Sidecar ចុងក្រោយ** មិនត្រូវបានសរសេរ ⇒ crash មុន rename ទាញ window ចុងក្រោយម្ដងទៀត ⇒ សរសេរ "commit ទាំងអស់" ក្រោយ final sync។
10. **Probe ពេល commit គ្រប់** ស្នើ `bytes=0-` ⇒ ស្នើតែ byte ចុងក្រោយ។
11. **Harness**: typeperf ទុកតម្លៃទទេចុងក្រោយ ⇒ parse បរាជ័យ ⇒ រំលងតម្លៃមិនមែនលេខ។

### Benchmark ធៀប IDM 6.43 (plan ង)

Harness: `spikes/engine/bench.ps1` (គូ GDM/IDM, លំដាប់ AB/BA ចៃដន្យដែលមានតុល្យភាព seed 20261109, warm-up 1 គូមិនរាប់, restart IDM រាល់ run, `ipconfig /flushdns`, CPU = typeperf ម៉ាស៊ីនទាំងមូល 1 s, RAM = Private Bytes peak, connection = `Get-NetTCPConnection`), របាយការណ៍ `python -I spikes/engine/bench_report.py <csv…>`, URL: `docs/p0/bench-urls.md`, ទិន្នន័យឆៅ: `docs/p0/bench/*.csv` + `*-log.txt`។
**ចំណុចបញ្ចប់:** GDM `done_s` = ចប់ពិត (final sync + sha256 + IAttachmentExecute + rename); IDM = ពេលឯកសារមានទំហំពេញ ហើយបើក exclusive បាន (IDM មិន flush)។ ដូច្នេះ GDM `transfer_s` ទល់ IDM done ជាការប្រៀបធៀបផ្ទេរ, ហើយ GDM done ទល់ IDM done ជាការប្រៀបធៀបផលិតផល (ចំណុចបញ្ចប់មិនស្មើគ្នា: GDM durable)។ MB/s = ទសភាគ (bytes ÷ done_s)។

**LAN gate (loopback fixture S4, link 1 Gbit/s = 125 MB/s; `lan1g` = cap សរុប, `lan1g-percon` = + 25 MB/s ក្នុងមួយ connection)** — config: c/d = disk C:/D:, a = លំនាំដើមទល់លំនាំដើម, b = GDM 8 ថេរ ទល់ IDM លំនាំដើម 8:

| Config | URL | GDM MB/s median (worst) | IDM MB/s median (worst) | median r | សេចក្ដីសម្រេច | conn GDM / IDM | CPU ម៉ាស៊ីន GDM / IDM % | RAM GDM MB |
|---|---|---|---|---|---|---|---|---|
| C: a | lan1g | 62.4 (55.3) | 101.8 (73.2) | 0.598 | យឺតជាង | 3–6 / 8 | 12.0 / 14.5 | 20.1 |
| C: a | lan1g-percon | 55.9 (46.1) | 100.5 (63.4) | 0.580 | យឺតជាង | 3–6 / 8 | 9.2 / 11.6 | 20.0 |
| C: b | lan1g | 84.5 (82.3) | 109.0 (106.7) | 0.769 | យឺតជាង | 8 / 8–9 | 7.8 / 11.9 | 26.6 |
| C: b | lan1g-percon | 78.2 (50.5) | 104.2 (62.7) | 0.822 | យឺតជាង | 8 / 8 | 10.8 / 14.8 | 26.9 |
| D: a | lan1g | 65.5 (61.8) | 110.6 (108.9) | 0.588 | យឺតជាង | 3–6 / 8 | 7.4 / 13.1 | 20.1 |
| D: a | lan1g-percon | 64.9 (63.5) | 106.1 (102.7) | 0.613 | យឺតជាង | 5 / 8 | 8.3 / 13.4 | 17.5 |
| D: b | lan1g | 67.2 (61.4) | 110.1 (109.3) | 0.613 | យឺតជាង | 8 / 8 | 7.4 / 12.0 | 26.5 |
| D: b | lan1g-percon | 68.4 (67.4) | 104.4 (102.2) | 0.653 | យឺតជាង | 8–9 / 8 | 6.1 / 14.3 | 26.7 |

Geometric mean នៃ median(r): C: a 0.589, C: b 0.795, D: a 0.600, D: b 0.633 (គោលដៅ ≥ 0.95) ⇒ **ធ្លាក់**។ Ratio គូនីមួយៗ: CSV / `bench_report.py`។

**Engine តែប៉ុណ្ណោះ (D:, commit តាមកាលកំណត់បិទ `GDM_FLUSH_MB=100000`, មិនមែនឥរិយាបថផលិតផល):** `lan1g` transfer 8.59–8.94 s = **120–125 MB/s (median 124.8 MB/s ≥ 112.5) ⇒ gate 90% ឆ្លងសម្រាប់ផ្លូវ network របស់ engine** លើ paced loopback (មិនមែន LAN ពិត)។ Done 12.7–17.2 s ព្រោះ final sync រុញ ~1 GB ពី cache ទៅ HDD។ IDM done 9.7–10.0 s។

**CPU < 10% / RAM < 60 MB:** RAM engine ឆ្លង (11–27 MB)។ CPU ម៉ាស៊ីនទាំងមូល median 6–12% (12 threads, typeperf) ⇒ ឆ្លងលើ D:, នៅព្រំដែនលើ C: config a (12.0%); IDM 11–15%។

**អ្វីដែលវាស់ ពីមូលហេតុល្បឿន durable (contributors ដែលបានវាស់ មិនមែនការពន្យល់ពេញ):** (1) `FlushFileBuffers` លើឯកសារ ធ្វើឱ្យ write ផ្សេងរង់ចាំ: fixed=1 គ្មាន flush 8.75 s, flush 8 MiB/2 s 15.8 s, flush តាមពេល 2 s 14.1 s; (2) ល្បឿន durable របស់ disk ខ្លួនឯង (python, 1 MiB, fsync រាល់ 8 MiB): D: 99 MB/s (គ្មាន fsync 177), C: 83–137 MB/s (រវល់); (3) ramp ឈប់នៅ 3–6 conn ព្រោះល្បឿនសរុបលែងកើន (disk-bound); (4) `FILE_FLAG_WRITE_THROUGH`: C: fixed=4 10.7 s តែ D: 17–18 s ⇒ បិទលំនាំដើម។ សូម្បីតែ 8 conn ថេរ GDM នៅតែចាញ់លើ C: ⇒ ផ្នែកមួយនៃភាពខុសគ្នានៅមិនទាន់ពន្យល់ពេញ។

**Fallback (P1, plan S5 "profile ⇒ ប្ដូរ buffer/flush"):** រក្សា invariant "commit តែក្រោយ flush"; សាក writer ដាច់ដែលមាន queue កំណត់ + checkpoint តាមពេលជាមួយ budget re-download ក្រោយ crash; វាស់ពេល flush, ចំនួន checkpoint, byte ទាញឡើងវិញ; រត់ម្ដងទៀតលើ NVMe ដែលទំនេរ ≥ 25% និង LAN ពិតមានម៉ាស៊ីនទីពីរ + iperf3។ "Fast finish (មិន durable)" ដូច IDM = ជម្រើសផលិតផលដែលមិនបានយក។

**Internet (CDN ៥, observations តែប៉ុណ្ណោះ):** config ក (លំនាំដើមទល់លំនាំដើម), disk D:, `docs/p0/bench/public-a.csv`, log `public-log.txt`; config ខ មិនរត់លើ Internet (docs/decisions.md)។

**config pa**

| URL | GDM MB/s median (worst) | IDM MB/s median (worst) | r per pair | median r | verdict | conns GDM/IDM | CPU avg GDM/IDM % | GDM priv MB peak | sha ok |
|---|---|---|---|---|---|---|---|---|---|
| cloudflare | 11.7 (11.0) | 5.1 (0.6) | 17.22 1.15 2.15 2.33 18.62 | 2.332 | faster | 3 / 2/8 | 8.6 / 11.0 | 12.4 | NO |
| cloudfront | 11.3 (10.8) | 11.2 (10.8) | 1.02 1.09 0.95 0.97 1.04 | 1.015 | equal | 4 / 8 | 5.4 / 8.5 | 13.3 | yes |
| fastly | 10.3 (8.2) | 0.8 (0.4) | 5.46 27.25 12.28 5.55 26.45 | 12.277 | faster | 3 / 2/8 | 5.9 / 6.1 | 12.1 | NO |
| github | 9.1 (8.9) | 10.4 (9.7) | 0.88 0.82 0.92 1.01 0.87 | 0.883 | slower | 4 / 8 | 5.6 / 8.4 | 12.0 | yes |
| google | 11.1 (10.8) | 4.6 (1.0) | 2.50 1.40 10.80 4.76 1.05 | 2.497 | faster | 3 / 8 | 8.0 / 14.8 | 13.1 | yes |

equal-or-faster on 4/5 URLs; geometric mean of median(r) = 2.298 (target >= 0.95)

កំណត់សម្គាល់: ផ្លូវ Internet ឈប់ត្រឹម ~11–12 MB/s (≈100 Mbit/s) ⇒ GDM ពេញ link ដោយ 3–4 connection; ពិន្ទុ "faster" មកពី IDM ជាប់ (cloudflare 2/5, fastly 2/5 timeout 600 s = sha missing ⇒ "sha ok NO" ជារបស់ IDM, GDM sha ត្រូវគ្រប់ run) មិនមែន GDM លឿនជាង link; github GDM យឺតជាង (r 0.883)។ Run ដែលបោះបង់ (HTTP/2 មុន `http1_only`, IDM ជាប់ 3600 s) ទុកជា `public-*-aborted-*`; log IDM ជាប់ `bench/idm-stall-cloudflare.log`។ លទ្ធផលនេះជា observation (ង.4) មិនមែន gate។

### អ្វីដែលមិនទាន់ធ្វើ / ត្រូវការម្ចាស់

- FAT/exFAT/SMB ពិត (S6), account ទីពីរពិត (S3): ត្រូវការ admin ឬ USB/share ⇒ អាចរត់ `s6 <path>` / S3 ពេលមាន។
- Benchmark លើ LAN ពិត (ម៉ាស៊ីនទីពីរ) និង disk ដែលមិនពេញ: P1។
- ការជ្រើស "fast finish មិន durable" ដើម្បីស្មើ IDM: មិនបានយក; ប្រសិនម្ចាស់ចង់បាន ត្រូវជាសេចក្ដីសម្រេចផលិតផល។

### របៀបរត់ឡើងវិញ

```
powershell -File spikes/engine/check.ps1 s2|s3|s4|s5|s6|s7|s8      (s5 1000 = 1000 kill rounds, ~41 min)
spikes/engine/s4 serve 8790   (LAN fixture; GDM_FIXTURE_TRACE=1 prints each request + Range)
powershell -File spikes/engine/bench.ps1 -UrlFile spikes/engine/bench-lan.txt -Out lan.csv [-Fixed 8] -Root D:\gdm-bench
python -I spikes/engine/bench_report.py docs/p0/bench/lan-d-a.csv ...
```

S7 ត្រូវការ `%LOCALAPPDATA%\gdm-spike-tools\{yt-dlp,ffmpeg,ffprobe}.exe` (sha256 ខាងលើ)។

### Council (ChatGPT/Codex, read-only, 2 ជុំ, 2026-11-09)

**ជុំ ១ (request changes):** Codex រកឃើញ: S7 PASS ទទេ (redirect) និង re-resolve; S5 1000 រង្វង់មិនទាន់រត់; sidecar រចនាសម្ព័ន្ធត្រឹមត្រូវតែកុហក; partial write; final rename; EICAR "បាត់" មិនមែនភស្តុតាង AV; FS matrix មិនពេញ; S3/S8/S2 ពាក្យខ្លាំងពេក; ការវាយតម្លៃ benchmark។
**ការសម្រេច (Claude ជាអ្នកដឹកនាំ):** ផ្ទៀងគ្រប់ចំណុចជាមួយកូដ/ការវាស់។ ពិត ហើយបានកែ: S7 (ធ្ងន់ជាងគេនិយាយ: request ទី ២ ត្រូវបញ្ជូនដោយមិនពិនិត្យ), partial write, seam check (+ bug probe offset ដែលរកឃើញពេលធ្វើ), sidecar ចុងក្រោយ, ច្បាប់ AV, 1000 រង្វង់។ **ទម្លាក់:** harness ចាក់សោ write សម្រាប់ "write មិនទាន់ចប់ពេល revoke" (អំណះអំណាងរចនាសម្ព័ន្ធគ្រប់គ្រាន់, Codex យល់ព្រមជុំ ២); ការពន្យល់ parent rename S8 (កញ្ចប់ verify/extract ពី bytes ក្នុង memory)។
**ជុំ ២:** Codex យល់ព្រម: អំណះអំណាង worker តាមលំដាប់, S8 bytes ក្នុង memory, ការកែ partial write / sidecar ចុងក្រោយ / probe guard, resolver ដែលពិនិត្យ (rebinding), ការរក្សា durability។ ចំណុចដែលនៅសល់ ហើយបានអនុវត្ត: បង្រួមការអះអាង seam check (+ NOTE interior corruption ដែលបង្ហាញ sha ខុស), ពាក្យ "សង្ស័យ security software", gate 90% តែលើ paced loopback, ពន្យល់ល្បឿនជា contributors, 1000 រង្វង់មុនបិទ (រួចហើយ)។
**ការឯកភាព:** G3 បិទជា **ការវាយតម្លៃ** ដែល gate ល្បឿនផលិតផលធ្លាក់ មិនមែន "PASS ទាំងស្រុង"; fallback P1 ទទួលយកតែកន្លែងដែល plan អនុញ្ញាត។
