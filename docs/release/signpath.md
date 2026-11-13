# SignPath Foundation: ការត្រៀមដាក់ពាក្យ

ឯកសារនេះសម្រាប់ម្ចាស់ (GoldenIQ)។ ប្រភព៖ https://signpath.org/terms និង form https://signpath.org/apply (អានផ្ទាល់ 2026-11-14)។ គេហទំព័រ GDM នៅ `Code\Web\Nextjs\Galaxy Download Manage WEB v1.0` (repo ផ្ទាល់ខ្លួន, commit `aa2f10a`)។

## ១. លក្ខខណ្ឌ SignPath និងស្ថានភាព

| លក្ខខណ្ឌ (signpath.org/terms) | ស្ថានភាព |
|---|---|
| គ្មាន malware / PUP | ✅ |
| អាជ្ញាប័ណ្ណ OSI គ្រប់ផ្នែក គ្មាន dual-licensing ពាណិជ្ជកម្ម | ✅ RPL-1.5 (OSI); គ្មានកំណែបង់ប្រាក់ |
| គ្មានកូដ proprietary ក្នុងឯកសារដែលចុះហត្ថលេខា | ✅ yt-dlp/FFmpeg មិនមានក្នុង installer (ទាញពេលអ្នកប្រើយល់ព្រម); WebView2 = system library |
| ថែទាំសកម្ម | ✅ commit ជាប្រចាំ |
| **ចេញផ្សាយរួចក្នុងទម្រង់ដែលនឹងចុះហត្ថលេខា** | ⏳ ម្ចាស់ត្រូវ publish release `v0.9.0` សាធារណៈលើ GitHub (draft មិនរាប់) |
| ពិពណ៌នាមុខងារលើទំព័រទាញយក | ✅ `/download` ផ្នែក "What you are downloading" |
| គម្រោងរបស់ខ្លួន, binary របស់ខ្លួន | ✅ build ពី repo នេះតែប៉ុណ្ណោះ |
| មិនមែនឧបករណ៍ hacking | ✅ (site grabber អានតែប៉ុណ្ណោះ, ជៀស logout/delete) |
| ឯកជនភាព: ការផ្ញើទិន្នន័យត្រូវមាន policy និងបិទបាន | ✅ PRIVACY.md + `/privacy`; update check បិទបាន; component វីដេអូសួរមុន |
| ប្រកាសការប្ដូរប្រព័ន្ធ | ✅ `/download` "What GDM changes on your computer" |
| មាន uninstall | ✅ NSIS uninstaller (ដក registry native messaging + autostart) |
| **MFA គ្រប់សមាជិក (GitHub + SignPath)** | ⏳ ម្ចាស់: បើក 2FA លើ GitHub `goldeniq-official` (ហើយលើ SignPath ពេលទទួលគណនី) |
| តួនាទី Author / Reviewer / Approver | ✅ `/code-signing-policy`: GoldenIQ កាន់ទាំងបី (គម្រោងមនុស្សម្នាក់ សរសេរត្រង់ៗ) |
| Build ផ្ទៀងបាន + approve ដោយដៃ | ✅ GitHub Actions `release.yml`; ⏳ ការភ្ជាប់ SignPath ធ្វើក្រោយទទួលយក (ផ្នែក ៤) |
| ទំព័រ Code signing policy លើ homepage | ✅ `https://gdm.goldeniq.xyz/code-signing-policy`; ពាក្យ "Code signing policy" នៅលើ home, download និង header គ្រប់ទំព័រ |
| ប្រយោគ "Free code signing provided by SignPath.io, certificate by SignPath Foundation" | ✅ មាន ជាមួយស្ថានភាពពិត (`SIGNING_PROGRAM` ក្នុង `src/content/release.ts`: planned → pending → active) ដើម្បីមិនអះអាងមុនពេលទទួលយក (council 2026-11-14) |
| Metadata: ProductName = ឈ្មោះគម្រោង, ProductVersion ដូចគ្នា | ✅ `gdm.exe` (Tauri), **ថ្មី** `gdm-bridge.exe` និង `gdm-setup.exe` (build.rs បង្កើត VERSIONINFO ពី version workspace; ផ្ទៀងលើ build: "Galaxy Download Manager 0.9.0") |

## ២. អ្វីដែលម្ចាស់ត្រូវធ្វើ (តាមលំដាប់)

1. `git push` repo GDM (ហើយ CI ឆ្លង)។
2. Publish release `v0.9.0` សាធារណៈ (GitHub → Actions → version / release; សាក `GDM-Setup` លើកុំព្យូទ័រស្អាតមុន)។
3. ដាក់គេហទំព័រ: Cloudflare → Workers → បង្កើត Worker `gdm-web` ពី repo គេហទំព័រ (Workers Builds: `npm run build`, deploy `npx wrangler deploy`) → Custom domain `gdm.goldeniq.xyz`។ (repo គេហទំព័រមិនទាន់មាន remote: បង្កើត repo GitHub មួយ ហើយ push ដោយខ្លួនឯង)។
4. ក្នុង repo គេហទំព័រ `src/content/release.ts`: `published: true`, `date` = ថ្ងៃ release ពិត។ (Claude ធ្វើបានពេលម្ចាស់ប្រាប់ថា release ចេញហើយ)។
5. បើក 2FA លើ GitHub។
6. ទុកពេលឱ្យមាន reputation (អ្នកប្រើពិត ចំនួនទាញ stars issues)។ SignPath មិនបោះពុម្ពលេខកំណត់ទេ តែវាសួរ "Reputation" ជាកាតព្វកិច្ច។
7. ដាក់ពាក្យតាម form ខាងក្រោម។ ពេលដាក់រួច: `SIGNING_PROGRAM = "pending"`។ ពេលទទួលយក និង release ដំបូងបានចុះហត្ថលេខា: `"active"` និង `authenticode: true` សម្រាប់ release នោះ។

## ៣. ចម្លើយ form (ចម្លងដាក់ផ្ទាល់)

- **Project Name:** Galaxy Download Manager
- **Repository URL:** https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0
- **Homepage URL:** https://gdm.goldeniq.xyz
- **Download URL:** https://gdm.goldeniq.xyz/download
- **Privacy Policy URL:** https://gdm.goldeniq.xyz/privacy
- **Wikipedia URL:** (ទទេ)
- **Tagline:** A free, open source download manager for Windows that resumes downloads and verifies every file.
- **Category:** Utilities (ឬជិតបំផុតក្នុងបញ្ជី)
- **Description:** Galaxy Download Manager (GDM) is a desktop download manager for Windows. It downloads files over several connections, resumes after network loss or a restart, and only gives a file its final name once every byte is verified with SHA-256 and checked by the user's antivirus. Browser extensions for Chromium browsers and Firefox hand downloads to the app through native messaging. It is free for everyone, has no ads, no telemetry and no account, and its interface is in Khmer and English.
- **Reputation:** (សរសេរតែការពិត ពេលដាក់) ឧ. GitHub stars/downloads ពី release, ការប្រើក្នុងសហគមន៍ខ្មែរ, វីដេអូ YouTube @GoldenIQ, ការចុះផ្សាយ។ កុំដាក់លេខដែលមិនទាន់មាន។
- **Maintainer Type:** Individual
- **Build System:** GitHub Actions
- **First / Last Name, Email:** ព័ត៌មានផ្ទាល់ខ្លួនរបស់ម្ចាស់ (ម្ចាស់បំពេញ)
- **Primary Discovery Channel:** ChatGPT (ឬ Other)

## ៤. ក្រោយទទួលយក: ការប្ដូរ `release.yml`

លំដាប់ចុះហត្ថលេខា (ឯកសារខាងក្នុងមុន ព្រោះ Setup ផ្ទុក NSIS ហើយ NSIS ផ្ទុក exe):

1. Job build: `tauri build --no-bundle` → upload `gdm.exe` + `gdm-bridge.exe` ជា artifact → SignPath GitHub action (`signpath/github-action-submit-signing-request`) ចុះហត្ថលេខា (approve ដោយដៃ)។
2. យក exe ដែលបានចុះហត្ថលេខា → `tauri bundle` (NSIS) → ចុះហត្ថលេខា NSIS installer។ Uninstaller ដែល NSIS បង្កើតខាងក្នុង ត្រូវការ signing ពេល build (Tauri `bundle.windows.signCommand`): **ត្រូវសួរ SignPath ពេល onboarding** ថាធ្វើតាមរបៀបណា។
3. `GDM_SETUP_PAYLOAD` = NSIS ដែលបានចុះហត្ថលេខា → build `gdm-setup` → ចុះហត្ថលេខា Setup ចុងក្រោយ។
4. បន្ទាប់ពីហត្ថលេខាចុងក្រោយទើប: minisign (`.sig`, `latest.json`), `SHA256SUMS.txt`, attestation។

SignPath សម្រាប់ OSS តម្រូវ runner របស់ GitHub (មិនមែន self-hosted) ហើយ artifact ត្រូវ upload ក្នុង workflow ដដែល។ Artifact configuration ត្រូវកំណត់ ProductName "Galaxy Download Manager" និង version ជា restriction។ macOS (Apple notarization) និង Linux មិនពាក់ព័ន្ធ SignPath ទេ។
