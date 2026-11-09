# P1 ផែនការ UX, លំហូរ និង design (ក្រោយ G5)

ថ្ងៃ: 2026-11-10 · អ្នកសរសេរ: GoldenIQ (session G5) · ប្រភព: blueprint §05 (UX ៧, J1–J8), §06 (design), §09 (engine), `docs/evidence/G5.md`, `design/mockups/`។ ឯកសារនេះ**មិនប្ដូរ blueprint ទេ** វាជាការសិក្សា និងផែនការសម្រាប់ G6 (UI) ដែលផ្អែកលើអ្វីដែល G5 បានវាស់ និងរៀន។

## ១. គិតពីគោលការណ៍គ្រឹះ (first principles)

សួរថា "អ្នកប្រើពិតជាត្រូវការអ្វី?" ហើយដកអ្វីៗដែលមិនឆ្លើយសំណួរនោះចេញ:

| អ្វីដែលអ្នកប្រើត្រូវការ | ការពិតដែលវាស់បាន (G5) | អ្វីដែល UI ត្រូវធ្វើ |
|---|---|---|
| **ឯកសារត្រឹមត្រូវ** (byte ដូច server) | Engine មិនដែលដាក់ឈ្មោះចុងក្រោយលើឯកសារខុស (66 test, failpoint 17, kill ×1000) | បង្ហាញ "បានបញ្ចប់" តែពេល engine ថា Complete; បែងចែក **ផ្ទៀងផ្ទាត់រួច** (មាន hash) ពី **មិនបានផ្ទៀងផ្ទាត់** (គ្មាន ETag) ដោយពាក្យ មិនមែនដោយពណ៌ប៉ុណ្ណោះ |
| **លឿន** | Network engine ដល់ 124 MB/s; ពេល "done" ពឹងលើ disk (flush, ការអានឡើងវិញក្រោយ antivirus) | បង្ហាញដំណាក់កាល "កំពុងពិនិត្យជាមួយ antivirus និង SHA-256" ដើម្បីកុំឱ្យអ្នកប្រើគិតថាគាំងនៅ 100% |
| **មិនរំខាន** | Engine មានការសម្រេចចិត្តតែ ៥ ប្រភេទ (ខាងក្រោម) | គ្មាន modal ទេ: ការសម្រេចចិត្តនៅក្នុងជួរ (row) ជាប៊ូតុងតែមួយ (UX ១, ២៥ silent mode) |
| **សុវត្ថិភាពដោយមិនបាច់គិត** | MOTW ZoneId=3, EICAR ត្រូវរារាំង, junction/hard link ត្រូវបដិសេធ | ឯកសារគ្រោះថ្នាក់គ្មាន [បើក]; "គោលដៅមិនការពារ" ពន្យល់មួយឃ្លា និងស្នើថត NTFS |
| **មិនបាត់អ្វីទាំងអស់** | DB ខូច ⇒ ស្ដារពី backup + sidecar | ក្រោយស្ដារ: banner តែមួយ "បានស្ដារបញ្ជីពី backup ថ្ងៃ X; job Y ត្រូវបានរកឃើញឡើងវិញ" |

**អ្វីដែលដកចេញ (The Algorithm ជំហាន ២):** ផ្ទាំង "Complete dialog" របស់ IDM (ជំនួសដោយ toast), ផ្ទាំងព្រមាន "Are you sure?" សម្រាប់សកម្មភាពដែលត្រឡប់វិញបាន, ការបង្ហាញ error code ជាសារចម្បង (ទុកក្នុង details)។

## ២. ស្ថានភាព engine ⇒ UI (ផែនទីពិតពី G5)

ស្ថានភាពទាំងនេះយកពី `Fail`, `Protection` និង `jobs.state` ក្នុង `crates/gdm-core`។ ពាក្យខ្មែរ/អង់គ្លេសដូចក្នុង mockup ថ្មី `design/mockups/index.html?screen=states`។ ច្បាប់: ស្ថានភាពនីមួយៗ = **មូលហេតុមួយឃ្លា + សកម្មភាពមួយ**។

| Engine | ពណ៌ (token) | ពាក្យខ្មែរ | សកម្មភាព | ពេលចុច |
|---|---|---|---|---|
| `downloading` | accent | កំពុងទាញ 63% | (ផ្អាក ក្នុង details) | — |
| `finalizing` | accent (bar 100%) | កំពុងពិនិត្យជាមួយ antivirus និង SHA-256 | គ្មាន | — |
| `complete` + hash ស្នើ/មាន `.sha256` | `--done` (lime) | បានបញ្ចប់ · ផ្ទៀងផ្ទាត់រួច | បើកថត | — |
| `complete` + `unverified` | `--done` + ឃ្លា | បានបញ្ចប់ · មិនបានផ្ទៀងផ្ទាត់ | ផ្ទៀងផ្ទាត់ hash | បិទភ្ជាប់ hash ⇒ `sha256_expected` |
| `unprotected` | `--warn` | បានបញ្ចប់ · ថត exFAT មិនការពារ | បើកថត (គ្មាន [បើក]) | — |
| `blocked` (`Fail::Blocked`) | `--err` | ត្រូវ antivirus រារាំង · មិនបានរក្សាទុក | ហេតុអ្វី? | ពន្យល់ + link ទៅ Windows Security |
| `paused` + `FileChanged`/`SizeChanged` | `--warn` | ឯកសារលើ server បានប្ដូរ · ផ្នែកចាស់នៅដដែល | ចាប់ផ្ដើមឡើងវិញ | `Unverified::Restart` |
| `paused` + `NeedsDecision` (គ្មាន validator) | `--warn` | បន្តដោយមិនអាចផ្ទៀងផ្ទាត់ ឬចាប់ផ្ដើមថ្មី? | ចាប់ផ្ដើមថ្មី (លំនាំដើម) | `Restart`; ជម្រើសទីពីរក្នុង details: `Continue` |
| `paused` + `NeedsDecision` (`.part` មិនមែនរបស់ job) | `--warn` | ឯកសារ .part ត្រូវបានប្ដូរ · មិនប៉ះ | ទាញម្ដងទៀតជាឈ្មោះថ្មី | job ថ្មី (ឈ្មោះ (1)) |
| `paused` + `DiskFull` | `--err` | Disk មិនគ្រប់ · ត្រូវការ X (រក្សា 1 GB) | ប្ដូរថត | ជ្រើសថត ⇒ job ផ្លាស់ |
| `paused` + `SizeExceeded` | `--warn` | ទំហំលើស 20 GB ដែលកំណត់ · បន្ត? | បន្ត | ដំឡើង cap |
| `failed` + `Forbidden` (private) | `--warn` | ទាញពីបណ្ដាញក្នុង 192.168.1.20? | អនុញ្ញាត | `Engine::allow_private` |
| `failed` + `Forbidden` (scheme/https→http) | `--err` | Server ព្យាយាមប្ដូរទៅ http មិនសុវត្ថិភាព | ព័ត៌មាន | — (គ្មាន bypass) |
| `failed` + `Tls` | `--err` | Certificate របស់ server មិនត្រឹមត្រូវ | ព័ត៌មាន | — (គ្មានប៊ូតុង "ignore", W-03) |
| `paused` + `Auth`/`Gone`/`Html` | `--warn`/`--err` | ត្រូវការ login ឬ link ផុតកំណត់ / ឯកសារលែងមាន / Server ផ្ញើទំព័រ HTML | យក link ថ្មី / បើកក្នុង browser | J5 Refresh |
| `failed` + `Timeout`/`TooManyRetries` | `--warn` | Server ឆ្លើយយឺតពេក · ព្យាយាម 10 ដងហើយ | ព្យាយាមម្ដងទៀត | `run` |
| `paused` + `Recovered` | `--warn` | បានរកឃើញឡើងវិញពីឯកសារ .part.gdm | បន្ត | seam check, ហើយសួរម្ដងទៀតបើ private |

**កែច្បាប់ពណ៌ (ច្បាស់ជាងមុន):** `--done` (lime) ប្រើ**តែ** Complete ដែលការពារ; Complete ដែលមិនការពារប្រើ `--warn` ព្រោះអ្នកប្រើត្រូវដឹងថាមិនដូចករណីធម្មតា។ ពណ៌មិនដែលជាសញ្ញាតែមួយទេ: ពាក្យតែងតែប្រាប់មូលហេតុ (a11y, ពិការភ្នែកពណ៌)។

## ៣. សេណារីយ៉ូ (ជំហានម្ដងមួយៗ, UI ⇄ engine)

1. **ចាប់ពី browser ⇒ ទាញ ⇒ Complete:** extension handoff ⇒ `Engine::add` (ឈ្មោះពី Download Info; Content-Disposition តាម `name::file_name`) ⇒ `run` ⇒ progress ⇒ `finalizing` (បង្ហាញ ≥ 300 ms ដើម្បីកុំភ្លឹប) ⇒ toast "បានទាញ: name [បើក] [បើកថត]"។
2. **ដាច់បណ្ដាញ ⇒ ត្រឡប់មក:** engine retry ផ្ទាល់ខ្លួន (backoff 1→60 s, 10 ដង) ⇒ ជួរ "ផ្អាក (គ្មានបណ្ដាញ) · នឹងបន្តដោយខ្លួនឯង"; បិទ app ⇒ បើកវិញ ⇒ resume ពី checkpoint (≤ 8 MiB ឬ 2 s ដែលបាត់)។
3. **Server ប្ដូរឯកសារ:** `FileChanged` ⇒ ជួរ warn + [ចាប់ផ្ដើមឡើងវិញ]; ផ្នែកចាស់នៅដដែលរហូតអ្នកប្រើសម្រេច។
4. **Antivirus រារាំង:** `blocked` ⇒ គ្មានឈ្មោះចុងក្រោយ, គ្មាន toast "បានទាញ"; toast ខុសប្រភេទ "ត្រូវ antivirus រារាំង" (មិនលាក់ទោះ silent mode)។
5. **Disk ពេញមុនចាប់ផ្ដើម:** `DiskFull` មុន preallocate ⇒ Download Info បង្ហាញទំហំខ្វះភ្លាម (មិនរង់ចាំបរាជ័យពាក់កណ្ដាល)។
6. **USB exFAT:** Complete ⇒ `unprotected` ⇒ ឃ្លាពន្យល់ + គ្មាន [បើក] សម្រាប់ .exe; Category អាចកំណត់ "ប្រភេទគ្រោះថ្នាក់ទៅថត NTFS ជានិច្ច" (N-09)។
7. **Crash ពេលបញ្ចប់:** engine reconcile ពី intent ⇒ Complete ដោយមិនទាញស្ទួន; UI មិនត្រូវការធ្វើអ្វីទេ។
8. **DB ខូច:** `Engine::open` report ⇒ banner មួយ (មិនមែន modal) "បានស្ដារពី backup …"។

## ៤. ការដំឡើងដំបូង និងការបើកដំបូង (J1/J2 កែលម្អ)

គោលដៅ: **ពីចុច Setup ដល់ការទាញដំបូង < 60 វិនាទី** ហើយ ≤ 5 ការចុច។

| ជំហាន | អ្វីដែលបង្ហាញ | ហេតុផល |
|---|---|---|
| Installer ១ | ភាសា: ខ្មែរ (ជ្រើសរួច), English | R6, G2 S9 (LangDLL plan B បានសាក) |
| Installer ២ | "សម្រាប់ខ្ញុំ (គ្មាន admin)" ជាលំនាំដើម + ថត `%LOCALAPPDATA%\Programs\GDM` | ADR, G2 per-user |
| Installer ៣ | Progress ~10 s + ផ្ទាំងបញ្ចប់: [បើក GDM] (ជ្រើសរួច), link LICENSE/NOTICE (គ្មាន checkbox) | J1 |
| បើកដំបូង ១ | ភាសា + theme (តាមប្រព័ន្ធ) មួយផ្ទាំង | J2.1 |
| បើកដំបូង ២ | ភ្ជាប់ browser (mockup onboarding) | J2.2 |
| បើកដំបូង ៣ | ជម្រើស ៣ (វីដេអូ, clipboard, update) + **ថតរក្សាទុក** (Downloads ពិនិត្យថា NTFS) | J2.3 + G5 N-09 |
| ចប់ | Empty state: ប៊ូតុង pill មួយ "អូស link មកទីនេះ ឬចុច +" | §06 |

**បន្ថែមពី G5:** នៅជំហាន ៣ ពិនិត្យថត Downloads: បើ volume មិនមែន NTFS ឬទំនេរ < 2 GB ⇒ ឃ្លាព្រមានមួយ (មិនរារាំង)។ ការពិនិត្យនេះប្រើ `fsx::fs_name` / `free_space` ដែលមានរួច។

## ៥. ប្លង់ និងច្បាប់ design (កែតម្រូវ)

1. **ជួរឈរ "ស្ថានភាព"** មានអត្ថបទមួយបន្ទាត់ (ellipsis + title) និង**សកម្មភាពតែមួយ**។ អត្ថបទពេញ + ជម្រើសទីពីរ នៅក្នុង details pane (មិនមែន popover)។ Mockup states បង្ហាញថាឃ្លាខ្មែរវែងជាងអង់គ្លេសប្រហែល 1.3× ⇒ ទទឹងជួរឈរ ≥ 280 px នៅ 1100 px។
2. **គ្មាន glow / គ្មាន outline** (ច្បាប់ម្ចាស់ ៨): ស្ថានភាពបង្ហាញដោយពណ៌អក្សរ និង fill របស់ bar ប៉ុណ្ណោះ; focus = `--focus-plate` fill។ Check ស្វ័យប្រវត្តិ `design/check.cjs` ឥឡូវរួមផ្ទាំង `states`។
3. **លំដាប់អាទិភាពក្នុងបញ្ជី:** ជួរដែលត្រូវការការសម្រេចចិត្ត (warn/err) ឡើងខាងលើ "កំពុងទាញ" ក្នុងទិដ្ឋភាព "ទាំងអស់" តែពេលអ្នកប្រើតម្រៀបតាមស្ថានភាព (លំនាំដើមនៅតាមពេលបន្ថែម ដូច IDM)។
4. **លេខ:** `tabular-nums`; ល្បឿន EMA 5 s; "នៅសល់" មិនបង្ហាញពេល `finalizing` (បង្ហាញ "—")។
5. **ពណ៌ type icon** (`.ft.*`) នៅដដែល; ពណ៌ស្ថានភាពមិនប្រើពណ៌ type icon ដើម្បីកុំច្រឡំ។

## ៦. ផែនការបន្ទាប់ (G6 និងបន្ទាប់)

| លំដាប់ | ការងារ | ហេតុផល / ភស្តុតាង |
|---|---|---|
| 1 | **API progress + ពាក្យបញ្ជា** ក្នុង gdm-core: channel event (bytes, ល្បឿន, segment map, state) និង pause/cancel/connections live | G5 មានតែ `run` ⇒ UI មិនអាចបង្ហាញ progress ឬផ្អាកបាន (ចន្លោះដែលរកឃើញ) |
| 2 | Tauri IPC ⇒ React list/details/Download Info តាម mockup | G6 goal |
| 3 | Writer/checkpoint profile (checkpoint សប្ដាហ៍ទី ២, exit.md ផ្នែក ៥) | G5 វាស់: disk durable ជាដែនកំណត់; ការអានឡើងវិញក្រោយ Save លើ HDD ~6 s/GiB |
| 4 | Defender/AMSI ជាភស្តុតាង AV វិជ្ជមាន (R13 fallback) | "Blocked" បច្ចុប្បន្នផ្អែកលើការលុប + probe |
| 5 | E-08 sleep/Wi-Fi (medium), W-05 proxy ប្រព័ន្ធ (medium) | មិននៅក្នុង G5 |
| 6 | CI ពិតលើ GitHub (ម្ចាស់ push) ⇒ N-09 exFAT/SMB, fuzz, kill ×1000 nightly | `.github/workflows/core.yml` ត្រៀមរួច |

## ៧. ចំណុចខ្វះខាតដែលនៅបើក (ត្រង់ៗ)

- CI មិនទាន់រត់ (ត្រូវ push ដោយម្ចាស់)។ N-09 ពិត និង fuzz 24 h មិនទាន់រត់។
- Power loss មិនត្រូវបានសាក (TerminateProcess តែប៉ុណ្ណោះ)។
- Gate ល្បឿន "≥ IDM −5%" មិនឆ្លងលើម៉ាស៊ីននេះ (មើល `docs/evidence/G5.md` ផ្នែក ៥): GDM ធានា durable + hash + MOTW ក្រោយ Save, IDM មិន flush ទេ។ ការសម្រេចប្ដូរ acceptance ជារបស់ម្ចាស់ (exit.md checkpoint សប្ដាហ៍ទី ២)។
