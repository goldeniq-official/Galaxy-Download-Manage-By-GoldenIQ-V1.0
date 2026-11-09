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
| 12 | S9 Khmer installer: welcome, folder, installing/finish, uninstall confirm/done | NSIS | **PASS** shaping ត្រឹមត្រូវ (ជើង, ស្រៈផ្សំ: ស្វាគមន៍, ក្នុង, ត្រូវការ, កុំព្យូទ័រ), ចំណងជើង/អត្ថបទ/ប៊ូតុងគ្មានកាត់; uninstaller ចាំភាសាខ្មែរ (`InstallerLanguage=1107` ក្នុង HKCU)។ **បញ្ហាដែលរកឃើញ ហើយកែរួច:** ប៊ូតុង "បង្ហាញព័ត៌មានលម្អិត" ត្រូវកាត់ ⇒ ប្ដូរជា "លម្អិត"; `installer.nsi` គ្មាន BOM ⇒ mojibake ⇒ ដាក់ BOM | `docs/p0/g2-screens/02…06*.png` (125%) |
| 13 | S9 ផ្ទាំងជ្រើសភាសា (LangDLL) | NSIS | **PARTIAL** ខ្មែរត្រូវបានជ្រើសដំបូង ហើយ label បង្ហាញខ្មែរបាន ប៉ុន្តែអក្សរ "ខ្មែរ" ក្នុង combobox តូច ហើយចង្អៀត; LangDLL មិនទទួល font ដែលបញ្ជូនបន្ទាប់បញ្ជីភាសា (MUI2 ក៏មិនបញ្ជូន font ដែរ) | `docs/p0/g2-screens/01-language.png` |
| 14 | S9 DPI 100% / 150% / 200% | NSIS | **NOT RUN** ប្ដូរ DPI ត្រូវ sign out; បានសាកតែ 125% | — |
| 15 | S9 អ្នកអានខ្មែរពិនិត្យ | NSIS | **រង់ចាំម្ចាស់** ពិនិត្យរូប `g2-screens/` | — |
| 16 | S1b MSIX: build + ចុះហត្ថលេខា | MSIX | **PASS** `makeappx pack` (schema ឆ្លង) ជាមួយ `desktop6:RegistryWriteVirtualization=disabled`, `desktop6:FileSystemWriteVirtualization=disabled`, `rescap:unvirtualizedResources`, `uap3` appExecutionAlias `gdm-bridge.exe`, `uap10:Parameters="register"`; `signtool` ជាមួយ cert self-signed `CN=GoldenIQ Spike` | `msix/AppxManifest.template.xml`, `msix/build.ps1` |
| 17 | S1b MSIX: install | MSIX | **BLOCKED (លើម៉ាស៊ីន/config នេះ)** (1) cert ក្នុង CurrentUser\TrustedPeople: `0x800B0109` root មិនទុកចិត្ត ⇒ ទំនងត្រូវ LocalMachine\TrustedPeople (admin) ប៉ុន្តែមិនទាន់បានសាក; (2) package គ្មានហត្ថលេខា (publisher OID) + `-AllowUnsigned`: `0x80073D2B` "an unsigned package cannot include Executable activations"; (3) policy `AllowAllTrustedApps=0` បដិសេធ MSIX របស់ Firefox ពី winget (`0x80073CFF`)។ សេចក្ដីសន្និដ្ឋានទាំងនេះសម្រាប់ config នេះប៉ុណ្ណោះ | `g2-logs/msix.log` |
| 18 | S1b MSIX: browser ឃើញ host, upgrade, uninstall, component ក្នុង package | MSIX | **NOT RUN** (អាស្រ័យលើ #17) | — |
| 19 | Windows 10: install/launch/upgrade/uninstall | NSIS | **NOT RUN** គ្មាន Windows 10 | — |
| 20 | Store submission (D9) | Store | **NOT RUN** D9: Store ពន្យារ, គ្មានគណនី ⇒ គ្មាន REAL STORE CERTIFICATION | plan.md D9 |

សំណល់ដែលកើតពីការព្យាយាម MSIX (មិនមែនពី NSIS): Windows Shell បង្កើត `HKCU\Software\Microsoft\Windows\CurrentVersion\CloudStore\…\windows.data.apps.appmetadata$goldeniq.galaxydownloadmanager.spike` ពេល `Add-AppxPackage` បរាជ័យ។ វាជាទិន្នន័យរបស់ Windows ដែលខ្ញុំមិនបានលុប។

### ផ្លូវដែលជ្រើស

**NSIS per-user ជាផ្លូវចម្បង** (ត្រូវនឹង D8)។ Installer ចុះឈ្មោះ host ក្នុង HKCU key ទាំង ៣ ហើយ uninstaller លុបវាវិញ។ លើ Windows 11 ផ្លូវនេះត្រូវបានបង្ហាញថាដំណើរការ (#1–#8, #10–#12) ជាមួយចំណុចខ្វះ: #8 (`Un.exe` នៅ %TEMP%), #9 និង #13 (PARTIAL), #14, #15, #19 (NOT RUN)។ ការចែកចាយតាម GitHub Releases / winget / Tauri updater ពិតប្រាកដ **មិនទាន់បានបង្ហាញ** ក្នុង G2 ទេ (វាជាការងារ P1/release.yml)។
**MSIX:** package build បាន ប៉ុន្តែការដំឡើង ដូច្នេះក៏ការមើលថាតើ browser ឃើញ host ឬទេ មិនអាចសាកលើម៉ាស៊ីននេះបានទេ។ តាម plan S1b វាជា probe ½ ថ្ងៃ ហើយ spike ពេញធ្វើមុនដាក់ Store (D9)។

**ស្ថានភាព G2: មិនទាន់រួចរាល់ពេញលេញ។** Windows 10, DPI 100/150/200%, អ្នកអានខ្មែរ និង MSIX install មិនទាន់មានលទ្ធផល ⇒ ត្រូវការម្ចាស់សម្រេច: ផ្ដល់ម៉ាស៊ីន/VM Windows 10 ឬយល់ព្រមពន្យារ (មើល "មិនទាន់ធ្វើ")។

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
- **DPI 100/150/200%** (#14) និង **អ្នកអានខ្មែរ** (#15)។

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
powershell -File nsis/screens.ps1 -Out docs/p0/g2-screens
powershell -File msix/build.ps1 -Version 0.1.0.0 ; Add-AppxPackage out/gdm-spike-0.1.0.0.msix
```

`out/` និង `tools/` មិនត្រូវ commit (`.gitignore`)។ Log សំខាន់ៗ ត្រូវបានចម្លងទៅ `docs/p0/g2-logs/` (`checks.log`, `browsers-summary.log`, `nsis-steps.log`, `msix.log`, `rerun-after-council.log`)។ `nsis-steps.log` ជាការរត់មុន council (runner ចាស់ ដែល PASS មិនតម្រូវ ports/dir); លទ្ធផលដែលប្រើក្នុងតារាងគឺ `rerun-after-council.log`។

### Council (ChatGPT/Codex, read-only, 2 ជុំ, 2026-11-09)

**Codex បានបន្ថែម (ផ្ទៀងផ្ទាត់ជាមួយកូដ ហើយពិតទាំងអស់):** runner សម្រេច PASS ដោយមិនតម្រូវ ports/dir ហើយ exit 0 ជានិច្ច; `RMDir /r $LOCALAPPDATA\GDM` អាចលុបទិន្នន័យដែលមិនមែនជារបស់ installer; លទ្ធផល `nsExec` មិនត្រូវបានពិនិត្យ; #9 ខ្លាំងពេក (គ្មានភស្តុតាង module-load); #7 អះអាងខុសថា "មិនបាច់ចុះឈ្មោះឡើងវិញ"; #8 ពិនិត្យតែ `Un_*.exe`; check-host មិនតម្រូវ stdout អានអស់; U-06 path re-resolution; ភាពជាម្ចាស់នៃការចុះឈ្មោះ; `.part` ដែលទាយបាន។
**ការសម្រេច (Claude ជាអ្នកដឹកនាំ):** កែក្នុង spike ហើយរត់ check ដែលរងផលប៉ះពាល់ឡើងវិញ: harness (#4), uninstall មានតែអ្វីដែលខ្លួនជាម្ចាស់ + sentinel (#8), Abort ពេល register បរាជ័យ, check-host (#1), កែស្លាក #7 #8 #9 #17។ ការរត់ឡើងវិញក៏បានរកឃើញ `Un.exe` នៅ %TEMP% (#8) ដែលការពិនិត្យមុនបានខកខាន។ ចំណុចផ្សេងទៀតត្រូវបានកត់ទុកជា P1 (ការរកឃើញ 7–11)។
**ការឯកភាព:** Codex យល់ព្រមថាធាតុ P1 មិនរារាំងការបញ្ជាក់ផ្លូវ NSIS ទេ (S8 ជាម្ចាស់ U-06/U-02) ហើយផ្ដល់យោបល់កុំពង្រីកការ Abort ទៅជា rollback engineering។ **ទម្លាក់:** ការរត់ឡើងវិញទាំងស្រុង (រត់តែ check ដែលរងផលប៉ះពាល់)។
**សេចក្ដីសន្និដ្ឋានរួម:** ផ្លូវ NSIS per-user ត្រូវបានបង្ហាញលើ Windows 11 ប៉ុន្តែ G2 **មិនទាន់រួចរាល់ពេញលេញ** រហូតដល់ Windows 10, DPI, អ្នកអានខ្មែរ មានលទ្ធផល ឬម្ចាស់យល់ព្រមពន្យារ។
