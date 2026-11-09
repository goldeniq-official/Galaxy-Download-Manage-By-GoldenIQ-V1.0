# GDM P0-A: បញ្ជីសេចក្ដីសម្រេច និងផែនការ spike

- Goal: G1 (P0-A) · កាលបរិច្ឆេទ: 2026-11-09 · ម្ចាស់: GoldenIQ
- ប្រភពពិត: `docs/GDM-Blueprint.html` (បង្កើតពី `docs/blueprint/*.py`) + `LICENSE` (RPL-1.5)
- ឯកសារនេះជាផែនការ មិនមែនកូដទេ។ Spike នៅក្នុង `spikes/` ជារបស់បោះចោល មិនដែលចេញជាមួយ release ឡើយ។ ករណីលើកលែងមួយ: fixture server (S4) រក្សាទុកក្នុង `fixtures/` សម្រាប់ test P1។

## ក. បញ្ជីសេចក្ដីសម្រេច D1–D19

ប្រភពចម្លើយ: "§19" = ម្ចាស់សម្រេចមុន (មានក្នុង blueprint/commit), "G1" = ម្ចាស់ឆ្លើយសំណួរផ្ទាល់ក្នុងវគ្គ G1 (2026-11-09)។ ការស្ងៀមមិនត្រូវបានរាប់ជាការយល់ព្រមទេ។

| # | សំណួរ | ចម្លើយម្ចាស់ | កាលបរិច្ឆេទ / ប្រភព | ការងារដែលវាដោះសោ |
|---|---|---|---|---|
| D1 | Licence | RPL-1.5 ដោយមិនកែប្រែ | 2026-11-09 / §19, commit a16dcac | ADR-5, header SPDX, `deny.toml` (G2), ពាក្យសុំ SignPath |
| D2 | បោះបង់ "Advanced integration" (WFP kernel driver)? | បោះបង់ | 2026-11-09 / G1 | Parity ledger §04; គ្មាន driver ⇒ គ្មាន kernel signing |
| D3 | បោះបង់ IE, MSN, AOL, Avant, Maxthon, IE click monitor, MMS? | បោះបង់ | 2026-11-09 / G1 | Extension គោលដៅតែ Chromium MV3 + Firefox |
| D4 | Dial-up ⇒ "រង់ចាំបណ្ដាញ + VPN/RAS (rasdial)" ក្នុង P3? | ជំនួស (P3) | 2026-11-09 / G1 | Card P3; មិនប៉ះ P0/P1 |
| D5 | ប៊ូតុងវីដេអូលើទំព័របិទលំនាំដើម ហើយសួរនៅ onboarding? | បាទ | 2026-11-09 / G1 | Extension UX (P2), onboarding |
| D6 | Keyboard focus = fill accent-soft + អក្សរដិត (គ្មាន outline)? | យល់ព្រម | 2026-11-09 / G1 | Design tokens, X-05, axe check |
| D7 | macOS ពន្យារទៅ P4? | ពន្យារ | 2026-11-09 / G1 | gdm-core រក្សាឯករាជ្យពី platform |
| D8 | ផ្លូវចុះហត្ថលេខា និងចែកចាយ | GitHub Releases + winget + SignPath; Store ជាជម្រើសពេលក្រោយ។ **បន្ថែម:** ទទួលយក NSIS គ្មាន Authenticode (sha256 + attestation) ជាផ្លូវសាធារណៈ មុន SignPath អនុម័ត ឬបើ SignPath បដិសេធ | 2026-11-09 / §19, commit 4031300 + G1 (ផ្នែក ឃ) | `release.yml`, ទំព័រ download ពន្យល់ SmartScreen (T-04), S8 |
| D9 | គណនី Microsoft Store បុគ្គល ឬក្រុមហ៊ុន? | ម្ចាស់ជ្រើសជម្រើសដែលសរសេរថា "Store ពន្យារ"។ ពេលចាប់ផ្ដើម Store: Golden IQ ជាឈ្មោះពាណិជ្ជកម្ម ⇒ policy 10.14 ទំនងតម្រូវគណនីក្រុមហ៊ុន (ការវិភាគ មិនមែនចម្លើយម្ចាស់) | 2026-11-09 / G1 | មិនរារាំង P0/P1 |
| D10 | YouTube តាម yt-dlp opt-in ក្នុង app (មិនក្នុង extension Chrome)? | បាទ ជាមួយ notice | 2026-11-09 / G1 | S7, ADR-4/6, M-07 |
| D11 | ឈ្មោះផ្លូវការ | "Galaxy Download Manager" | 2026-11-09 / ម្ចាស់ (blueprint + G1) | Identifier commit, ឈ្មោះ repo `galaxy-download-manager` |
| D12 | "GDM" តែក្នុង UI; ID = `GoldenIQ.GalaxyDownloadManager`? | យល់ព្រម | 2026-11-09 / G1 | `tauri.conf.json` identifier, winget ID។ (ឈ្មោះ native host ត្រូវជាអក្សរតូច ⇒ `com.goldeniq.gdm`, មើល S1a) |
| D13 | Logo: គូរ mark vector សម្រាប់ទំហំតូច | Mark vector: យល់ព្រម។ ទិសព្រួញ: នៅជាជម្រើសម្ចាស់ (មិនទាន់ជ្រើស) | 2026-11-09 / G1 | Icon (មិនរារាំង P0) |
| D14 | Parity "ផ្នែកខ្លះ" | ទទួលយក ២ ជួរដែលនៅសល់ (YouTube តាម app; ខ្មែរ+អង់គ្លេសមុន) | 2026-11-09 / §19, commit c707fe8 | Parity ledger |
| D15 | Hook .bat/.cmd ពេលបញ្ចប់ (កំណត់តែក្នុង UI, argument restriction)? | បាទ | 2026-11-09 / G1 | Q-01 (P2) |
| D16 | ប្រើក្នុងក្រុមហ៊ុន/សាលា/ស្ថាប័នដោយឥតគិតថ្លៃ? | បាទ | 2026-11-09 / §19 | §21 ចំណូល |
| D17 | (ក) ឈ្មោះ CHHIT Sungchhay (ខ) ស្ថានភាព Golden IQ (គ) អ្នកផ្ដល់ការអនុញ្ញាត | ម្ចាស់ជ្រើសជម្រើស "ឈ្មោះពាណិជ្ជកម្ម" ដែលសរសេរថា: "Golden IQ = ឈ្មោះពាណិជ្ជកម្មរបស់ CHHIT Sungchhay (មិនទាន់ចុះបញ្ជី); ម្ចាស់តែម្នាក់ផ្ដល់ការអនុញ្ញាត; Store ពន្យារ" ⇒ (ក) CHHIT Sungchhay (ខ) ឈ្មោះពាណិជ្ជកម្ម (គ) ម្ចាស់តែម្នាក់ | 2026-11-09 / G1 | NOTICE.md, Exhibit B, D9 |
| D18 | Dependency MPL-2.0 ដែលមិនកែប្រែ? | បាទ | 2026-11-09 / G1 | `deny.toml` (G2), THIRD-PARTY-NOTICES |
| D19 | OSI licence និងការហាមរកប្រាក់ផ្ទុយគ្នា | RPL-1.5 (គ្មាន commercial dual-licensing) | 2026-11-09 / §19 | §21 M4 |

**ចំណុចបើកចំហ (មិនរារាំង P0):** D13 ទិសព្រួញ (ម្ចាស់ជ្រើសពេលក្រោយ); D9 ប្រភេទគណនី សម្រេចពេលចាប់ផ្ដើម Store។

### ភាពផ្ទុយក្នុង blueprint ដែល G1 រកឃើញ (កែក្នុង blueprint ពេលម្ចាស់យល់ព្រម)

1. §17 P1 exit criteria នៅតែតម្រូវ "MSIX ឆ្លង Store certification" ហើយ §15 ជំហាន 4 នៅមាន Store submission។ D8 ធ្វើឱ្យ Store ជាជម្រើស ⇒ លក្ខខណ្ឌ MSIX គួរផ្លាស់ទៅ "មុនដាក់ Store" មិនមែន P1 ទេ។
2. ជួរ "Microsoft Store (បុគ្គល)" ក្នុង §15 មិនត្រូវនឹងការពិតថា Golden IQ ជាឈ្មោះពាណិជ្ជកម្ម ⇒ policy 10.14 (D9)។
3. §07/§08 និយាយថា `latest.json` ចុះហត្ថលេខា។ តាមពិត Tauri updater ចុះហត្ថលេខា minisign លើ artifact នីមួយៗ ហើយ signature ស្ថិតក្នុង `latest.json` (មិនមែន manifest ទាំងមូលទេ)។ ដូច្នេះ freshness ត្រូវពឹងលើច្បាប់ "version ធំបំផុតដែលធ្លាប់ឃើញ" ក្នុង §07។
4. P0 ពេញលេញត្រូវការ ≈ 4 សប្ដាហ៍ មិនមែន 3 ទេ (ផ្នែក ខ)។

## ខ. Spike inventory (តាមលំដាប់ហានិភ័យ §17)

ការប៉ាន់ស្មានគិតជាថ្ងៃធ្វើការ មិនមែនការសន្យា។ លទ្ធផលនីមួយៗសរសេរក្នុង `docs/p0/S<n>.md` (ពាក្យបញ្ជា, version, លទ្ធផល, បរាជ័យ)។

**Gate មុនគេ (§17):** `cargo deny` ជាមួយ RPL-1.5 + Code signing policy + លំហូរ SignPath ជា G2 មិនមែន spike ក្នុងបញ្ជីនេះទេ។

**ការកែតាម Algorithm (ត្រូវការម្ចាស់យល់ព្រម):** S1 ដើមផ្ដោតលើ MSIX។ ប៉ុន្តែ D8 ចាត់ Store ជាជម្រើស ដូច្នេះហានិភ័យពិតនៃ P1 គឺ native messaging ពី **NSIS per-user**។ ស្នើ: S1a (NSIS) ជា spike ចម្បង; S1b (MSIX) ធ្វើត្រឹម probe ½ ថ្ងៃ ហើយ spike ពេញធ្វើមុនដាក់ Store។

| # | Spike | Deliverable | Pass criterion (វាស់បាន) | Fallback / ផែនការ B | ថ្ងៃ | ហានិភ័យ | Cards §12 |
|---|---|---|---|---|---|---|---|
| S1a | Native messaging host ពី NSIS per-user | `spikes/s1-nmh/`: host Rust តូច (echo + ពិនិត្យ origin) + extension សាក។ ឈ្មោះ host ត្រូវជាអក្សរតូច (Chromium: អក្សរតូច លេខ `_` `.` តែប៉ុណ្ណោះ) ⇒ `com.goldeniq.gdm`។ Key HKCU `Software\Google\Chrome\NativeMessagingHosts\com.goldeniq.gdm`, `Software\Microsoft\Edge\NativeMessagingHosts\…`, `Software\Mozilla\NativeMessagingHosts\…` ដែល default value = path ពេញទៅ manifest (Firefox ប្រើ `allowed_extensions`) | Chrome, Edge, Firefox stable ទាំង ៣ ហៅ `connectNative` បាន 100/100; extension ID ផ្សេងត្រូវបដិសេធ 100/100; host⇒browser ≤ 1 MB (ដែនកំណត់ Chromium), browser⇒host មានដែនកំណត់ GDM (ឧ. 64 KiB) ⇒ លើសត្រូវបដិសេធ; header ខ្លី, length ប្រកាសធំគ្មាន payload, UTF-8 ខូច ⇒ host ចេញដោយគ្មាន crash; host មិនសរសេរអ្វីក្រៅ protocol ទៅ stdout; host ងាប់/disconnect/reconnect ដំណើរការ; uninstall លុប key ដែល installer ជាម្ចាស់ទាំងអស់ (`reg query` ទទេ) | Brave/Vivaldi/Opera: key តាម browser (B-07)។ បើ HKCU មិនដំណើរការ ⇒ HKLM តាម installer per-machine (មិនដោះស្រាយ policy ក្រុមហ៊ុនដែលហាម native messaging ⇒ ផ្លូវ clipboard/Add URL) | 2 | ខ្ពស់ | B-01, B-03, B-06, B-07, B-08, T-03 |
| S1b | MSIX probe (ជម្រើស; ទម្លាក់មុនគេបើយឺត) | MSIX sideload តាម `Add-AppxPackage -AllowUnsigned` (Windows 11) ឬ cert សាកក្នុងម៉ាស៊ីន ដែលដំឡើង host ដូចគ្នា | កត់ "browser ឃើញ host ពី MSIX: បាទ/ទេ" + schema extension ពិតដែលបានសាក + លទ្ធផលក្រោយ upgrade/uninstall។ លទ្ធផល = "sideload ដំណើរការ; Store eligibility មិនទាន់បញ្ជាក់" | Helper ដែលបើកពី package មិនរួចពី virtualization ដោយស្វ័យប្រវត្តិ ⇒ រក mechanism ផ្លូវការ (flexible virtualization) ឬបោះបង់ Store; សម្រេចពេល Store (D9) | 0.5 | មធ្យម (Store ជាជម្រើស) | B-06, T-03 |
| S2 | MV3 prepared handoff + ភាពត្រឹមត្រូវ request | `spikes/s2-mv3/`: Chromium: service worker + `downloads.onDeterminingFilename`/`onCreated`; Firefox: background script ដាច់ (គ្មាន `background.service_worker`, គ្មាន `onDeterminingFilename`)។ Ticket រក្សាទុក durable + idempotency key ⇒ host ack ⇒ cancel browser ⇒ activate ⇒ reconcile ពេលចាប់ផ្ដើម | 200 handoff មាន failure injection គ្រប់ transition (SW stop, host ងាប់, browser ទាញចប់មុន cancel) ⇒ 0 ស្ទួន, 0 បាត់: download នីមួយៗមានលទ្ធផលចុងក្រោយមួយ (browser ឬ GDM) ឬ "មិនច្បាស់" ដែលស្ដារបាន; ពេលចុចដល់ GDM ទទួល < 300 ms (median)។ Fixture S4: cookie, Referer, Authorization, redirect cross-origin (credential ត្រូវដក), range ក្រោយ handoff បរាជ័យ auth ⇒ refresh ⇒ hash ត្រូវ; POST/`blob:`/one-time URL ⇒ នៅក្នុង browser ជាមួយហេតុផលកត់ទុក; credential មិនដែលនៅក្នុង log | បើ ack មិនអាចធានា ⇒ ទុក download ក្នុង browser + ប៊ូតុង "ផ្ទេរទៅ GDM" សម្រាប់ GET ដែល replay បាន (បាត់ parity ចាប់ស្វ័យប្រវត្តិ ⇒ ម្ចាស់សម្រេច) | 2.5 | ខ្ពស់ | B-05, B-09, C-01, C-02, C-04, C-05, R-03, W-01 |
| S3 | Named pipe DACL + server verification | `spikes/s3-pipe/`: DACL = SID user បច្ចុប្បន្ន (ឬ logon SID ដើម្បីបំបែក session), មិនឱ្យ `FILE_GENERIC_WRITE` ដល់អ្នកផ្សេង (វារួម `FILE_CREATE_PIPE_INSTANCE`), `FILE_FLAG_FIRST_PIPE_INSTANCE`, `PIPE_REJECT_REMOTE_CLIENTS`; client បើកជាមួយ `SECURITY_SQOS_PRESENT \| SECURITY_IDENTIFICATION`, ពិនិត្យ server ដែលភ្ជាប់រួច (`GetNamedPipeServerProcessId` + កាន់ process handle ពេលពិនិត្យ image path/signature) មុនផ្ញើ cookie | User ផ្សេងភ្ជាប់មិនបាន; pipe squat មុន ⇒ engine ពិតបរាជ័យដោយបិទ (fail-closed) ហើយ client បដិសេធ 100%; instance ទីពីរដោយអ្នកវាយប្រហារពេល server ពិតកំពុងរត់ ⇒ បរាជ័យ; client user ផ្សេងផ្ញើពាក្យបញ្ជាមិនបាន; message ធំ/JSON ជ្រៅ ⇒ បដិសេធ (limit កត់ត្រា)។ **ព្រំដែន:** process user ដដែលដែលកែ install per-user មិនស្ថិតក្នុងវិសាលភាពការពារ (កត់ក្នុង threat model) | មុន SignPath ⇒ verify path ក្នុង `%LOCALAPPDATA%\Programs\…` + ACL ថត (ការធានាទាបជាង កត់ទុក) | 1.5 | ខ្ពស់ | B-02, B-03, B-08 |
| S4 | Hostile HTTP fixture server (**រក្សាទុក**) | `fixtures/hostile/`: hyper + raw TCP (សម្រាប់ framing ខុសដែល hyper មិនអនុញ្ញាត)។ ករណី: 200-on-range, 206 ខ្លី/វែង, Content-Range ខូច, range ជាន់, 416 ក្រោយបង្រួម, ETag ប្ដូរ, មាតិកាប្ដូរតែទំហំដដែល, validator ខ្សោយ/គ្មាន, Content-Length កុហក, body គ្មានទីបញ្ចប់, ដាច់ពាក់កណ្ដាល body, slowloris, redirect loop, redirect cross-origin (cookie), TLS ខូច, 429 + Retry-After, gzip លើ range, HTML ជំនួស binary, multipart/byteranges, auth (cookie/Referer/Authorization) | ករណីនីមួយៗមាន URL ដាច់, deterministic (seed), រត់ក្នុង CI Windows < 30 s; ករណីនីមួយៗមាន **oracle**: state transition រំពឹងទុក, ចំនួន retry, byte ដែលសរសេរ, ការបញ្ជូន credential, hash ឬកំហុសចុងក្រោយ; server log request ទាំងអស់ (trace ជាភស្តុតាង) | — (ជា harness សម្រាប់ P1) | 2 | មធ្យម | E-01, E-02, E-03, E-05, E-07, E-09, W-01, W-02, R-03 |
| S5 | Segmenter + write lease + benchmark | `spikes/s5-seg/`: actor segment map, lease មាន generation, split ពាក់កណ្ដាល, seek_write + preallocate, flush ⇒ commit, sidecar `.part.gdm` | 1 GB LAN fixture ⇒ ≥ 90% goodput (ផ្នែក ង), CPU < 10%, RAM engine < 60 MB; 1000 រង្វង់ `TerminateProcess` + resume ⇒ sha256 ត្រូវ 1000/1000; **failpoint កំណត់** នៅ: revoke lease, write មិនទាន់ចប់, data flush, sidecar flush/replace, rename ចុងក្រោយ ⇒ 0 ឯកសារខូច; write របស់ generation ចាស់ត្រូវចប់ ឬ cancel មុនផ្ដល់ផ្នែកទៅអ្នកផ្សេង; sidecar ដែលគេកែមិនអាចបង្វែរ write; fixtures S4 ទាំងអស់ ⇒ 0 ឯកសារខូច។ (TerminateProcess មិនមែនភស្តុតាងដាច់ភ្លើងទេ) | បើលឿនមិនគ្រប់ ⇒ profile (ETW/`cargo flamegraph`) ⇒ ប្ដូរ buffer/flush; បើ reqwest ជាចំណុចស្ទះ ⇒ hyper ផ្ទាល់ (មិនជួសជុល protocol persistence ខូចទេ) | 3 | ខ្ពស់ | E-01, E-02, E-04, E-06, E-08, L-01, L-02 |
| S6 | MOTW តាម IAttachmentExecute + exFAT/FAT | `spikes/s6-motw/`: Rust (`windows` crate) ហៅ `IAttachmentExecute::Save` ក្នុង STA worker (COM init) លើ NTFS, exFAT, FAT32 (VHD), SMB share | កត់ដាច់ៗ: HRESULT, ឯកសារនៅ/បាត់, ADS `Zone.Identifier` អានវិញ (ZoneId/HostUrl អាស្រ័យ policy មិនមែនសន្យា), ភស្តុតាង Defender; EICAR ⇒ state "ត្រូវបានរារាំង" (ទោះ quarantine មុន call); exFAT/FAT ⇒ "គោលដៅមិនការពារ" (មិនមែនជោគជ័យស្ងាត់); SMB ⇒ កត់តាម server; ឯកសារ executable មិនមានឈ្មោះចុងក្រោយ ឬប៊ូតុង "បើក" មុន attachment processing ចប់; rename រក្សា stream | បើ API បរាជ័យ ⇒ សរសេរ ADS ដោយផ្ទាល់ (NTFS ប៉ុណ្ណោះ; មិនស្មើ Attachment Execution Services; កត់ថាធានាតិចជាង) | 1.5 | មធ្យម | N-05, N-07, N-09, N-10 |
| S7 | yt-dlp policy proxy (ADR-6) + media gate តូច | `spikes/s7-ytdlp/`: proxy loopback port ចៃដន្យ, Proxy-Authorization ចៃដន្យក្នុង 1 run, network policy (resolve ⇒ ពិនិត្យ ⇒ connect ទៅ address ដែលបានពិនិត្យ, គ្មាន DNS ទីពីរ)។ yt-dlp: `-J --ignore-config --no-plugin-dirs` + option ថេរ + `--` មុន URL, env/cwd គ្រប់គ្រង, ដែនកំណត់ stdout/stderr/JSON depth/ពេល/child process (Job Object) | Request ទៅ 127.0.0.1, ::1, IPv4-mapped IPv6, link-local, ULA, LAN, 169.254.169.254, DNS rebinding, DNS មាន public+private, IP ទម្រង់ចម្លែក, CONNECT authority ចម្លែក ⇒ បដិសេធ 100%; policy ដូចគ្នាលើ URL មេឌៀដែល engine GDM ទាញ; argv injection (`--exec`, title មាន `-`) ⇒ 0 ជោគជ័យ; proxy បិទពេល run ចប់; **media gate:** ឯកសារ progressive 1 + វីដេអូ/សំឡេងដាច់ 1 ⇒ engine GDM ទាញ ⇒ ffmpeg mux (`-protocol_whitelist file,pipe`) ⇒ លេងបាន។ ការធានា = តែ request តាម proxy ក្នុង config ដែល pin (ProcMon/firewall សាកបង្ហាញតែឥរិយាបថ មិនមែន confinement) | Extractor ដែលរំលង proxy ⇒ បដិសេធមុនសកម្មភាពមិនសុវត្ថិភាព ⇒ "មិនគាំទ្រ" (M-08)។ WFP filter user-mode (មិនមែន kernel driver D2) ជាជម្រើសដែលអាចពិនិត្យនៅ P2 | 2.5 | ខ្ពស់ (assumption ADR-6) | M-01, M-02, M-03, M-05, M-06, M-08, W-02 |
| S8 | Update install + recover | `spikes/s8-update/`: Tauri updater (version pin) + minisign លើ artifact, NSIS passive, verify-to-execute: កាន់ handle `FILE_SHARE_READ` តែប៉ុណ្ណោះ (ហាម write/delete) ពី verify ដល់ launch, ថត staging ACL តឹង (ការពារ rename ថតមេ/reparse), រក្សា installer កំណែចាស់ដែលបាន verify | ប្ដូរ installer ឬថតមេក្រោយ verify ⇒ មិនរត់ (U-06) 100%; kill ពាក់កណ្ដាល install 20/20 ⇒ ជួសជុលបានដោយ installer កំណែចាស់ដែលរក្សាទុក + banner (U-04); install តែពេល idle, job សកម្មមិនដែលត្រូវ restart (U-02); manifest version ចាស់/ស្មើ ⇒ បដិសេធ | NSIS មិនមែន transaction rollback ⇒ បើ repair ពី installer ចាស់មិនដំណើរការ ⇒ ផែនការ B: ថតកម្មវិធីពីរ (A/B) ដែលប្ដូរ pointer | 2 | មធ្យម | U-01, U-02, U-04, U-06 |
| S9 | Khmer.nsh ក្នុង NSIS | `spikes/s9-nsis/Khmer.nsh` + Tauri bundler config (`languages`, `displayLanguageSelector`) + font ខ្មែរ (`SetFont`) | ពិនិត្យ **shaping** (ជើង, ស្រៈផ្សំ, កម្ពស់បន្ទាត់) មិនត្រឹម glyph: ប៊ូតុង, ផ្ទាំងជ្រើសភាសា, uninstall, repair/update, error dialog, path ខ្មែរ+ឡាតាំង; screenshot 100/150/200% DPI; ខ្មែរជាភាសាដែលបានជ្រើសដំបូង (មិនមែនត្រឹមនៅលើគេក្នុងបញ្ជី); អ្នកអានខ្មែរពិនិត្យ | Installer អង់គ្លេស + ផ្ទាំងជ្រើសភាសាក្នុង app ពេលបើកដំបូង (R6 ផ្លូវបម្រុង) | 0.5 | ទាប | T-02, X-03 |

**លំដាប់អនុវត្ត:** តាមបញ្ជី §17 ខាងលើ ប៉ុន្តែ S7 និង S8 អាចចាប់ផ្ដើមស្របគ្នាក្រោយ S4 (មិនពឹង S5)។

**ពេលវេលា:** spike 18 ថ្ងៃ + integration/rerun 2 ថ្ងៃ = 20 ថ្ងៃ (≈ 4 សប្ដាហ៍, លើស §17 ១ សប្ដាហ៍)។ បើត្រូវរក្សា 3 សប្ដាហ៍ ⇒ ទម្លាក់ S1b មុន ហើយរត់ការសាក 1000 រង្វង់ (S5) និង 200 handoff (S2) ជាការងារពេលយប់ ដាច់ពីការសរសេរ spike។

**ក្រៅ P0 (ទៅ P1 cards មិនមែន spike):** ឈ្មោះឯកសារអាក្រក់/ADS syntax/device name/reparse/disk full (N-01..N-10); update ស្របគ្នា winget + in-app, app ពីរ instance, migration បរាជ័យ (U-02, D-02); media matrix ពេញ DASH/HLS (P2/P3)។

## គ. ម្ចាស់ update តាមប្រភេទដំឡើង

| ប្រភេទ | អ្នក update app | Updater ក្នុង app | Component (yt-dlp/ffmpeg) |
|---|---|---|---|
| NSIS / GitHub Releases (ផ្លូវចម្បង, D8) | Tauri updater: `latest.json` មាន minisign signature របស់ artifact (+ Authenticode ពេល SignPath អនុម័ត) | បើក (state machine §07) | `components.json` ដែល GDM ចុះហត្ថលេខា (sha256 pin) |
| winget | winget ក៏ update បាន; Tauri updater នៅតែបើក (installer ដូចគ្នា) | បើក | ដូចខាងលើ |
| MSIX / Microsoft Store (ជម្រើស, D8/D9) | Microsoft Store | **បិទ** (build flag); Settings › Update បង្ហាញ "Update តាម Microsoft Store" | ដូចខាងលើ (ដូចគ្នាទាំងពីរ) |

ច្បាប់: minisign secret ក្នុង GitHub Environment "release" ដែលតម្រូវ reviewer = ម្ចាស់ (§15)។ Downgrade ត្រូវបដិសេធ។ Component update មិនបាច់ restart app។

## ឃ. D8: NSIS គ្មានហត្ថលេខាជាផ្លូវសាធារណៈ

**ចម្លើយម្ចាស់ (2026-11-09): ទទួលយក។** មុន SignPath អនុម័ត (ឬបើបដិសេធ) release លើ GitHub ជា NSIS គ្មាន Authenticode មាន sha256 + GitHub attestation (`actions/attest-build-provenance`)។ ទំព័រ download និង README ពន្យល់ការព្រមាន SmartScreen (T-04)។ Tauri updater នៅតែការពារ artifact ដោយ minisign ទោះគ្មាន Authenticode។ មិនត្រូវការ Store ជាផ្លូវបម្រុងសម្រាប់ចម្លើយនេះទេ។

## ង. Protocol វាស់ល្បឿនធៀប IDM 6.43

1. **ម៉ាស៊ីនថេរ:** ម៉ាស៊ីនតែមួយគ្រប់ run; កត់ CPU, RAM, disk (NVMe/SATA), Windows build, driver NIC, commit GDM, IDM build + hash installer + ការកំណត់។ Defender real-time scan កត់ស្ថានភាព (ដូចគ្នាទាំងពីរ)។ លទ្ធផលពាក់ព័ន្ធតែ build IDM នោះប៉ុណ្ណោះ។
2. **ខ្សែ (wired):** Ethernet តែប៉ុណ្ណោះ, Wi-Fi បិទ, VPN/proxy បិទ។
3. **កត់ bandwidth:** មុនរាល់ batch, វាស់ TCP goodput ដោយ `iperf3` លើផ្លូវ និង server ដូចគ្នា (LAN) និង speedtest (Internet); កត់ម៉ោង។ ឯកតា: MB/s ទសភាគ (90% នៃ 1 Gbit/s = 112.5 MB/s) ហើយមិនលាយ MiB/s។
4. **Cache:** លុបឯកសារគោលដៅ, disk ទំនេរ ≥ 3× ទំហំ, `ipconfig /flushdns`, restart app រវាង run, warm-up 1 run មិនរាប់។ (DNS flush មិនសម្អាត cache CDN/resolver ខាងលើទេ ⇒ ហេតុនេះលទ្ធផល CDN ជាការសង្កេត)។
5. **URL:** **LAN fixture 1 GB (S4) ជា gate ល្បឿន**។ ឯកសារសាធារណៈ 5 លើ CDN ផ្សេងគ្នា (ឧ. Cloudflare, Fastly, Akamai, CloudFront, GitHub Releases), 200 MB–1 GB, គាំទ្រ Range ⇒ បញ្ជី URL + sha256 ក្នុង `docs/p0/bench-urls.md` មុន run ដំបូង មិនប្ដូរក្នុងរង្វង់វាស់មួយ។
6. **ការកំណត់ ២ ប្រភេទ:** (ក) default ទល់ default (ឥរិយាបថផលិតផល) (ខ) connection ថេរ 8 ទាំងពីរ (ប្រសិទ្ធភាព implementation)។ គ្មាន speed limit។
7. **Runs:** URL នីមួយៗ 5 គូ GDM/IDM, លំដាប់ AB/BA ចៃដន្យដែលមានតុល្យភាព (seed កត់ទុក)។
8. **កត់ត្រា:** ពេលផ្ទេរចប់ និងពេល "ចប់ពិត" (flush + rename + attachment processing) ដាច់គ្នា, hash វាស់ដាច់; sha256 ផ្ទៀងផ្ទាត់; **ចំនួន connection** (GDM log; IDM `netstat -ano`; LAN = server request log ព្រោះ netstat មិនរាប់ HTTP/2 stream); CPU = ម៉ាស៊ីនទាំងមូល normalize (typeperf 1 s, មធ្យម + peak); RAM: engine Private Bytes និងផលិតផលទាំងមូល (WebView2) ដាច់គ្នា។
9. **របាយការណ៍:** URL នីមួយៗ: median និង worst case, ratio GDM/IDM ទាំង 5 គូ; ទិន្នន័យឆៅ CSV ក្នុង `docs/p0/bench/`។ លទ្ធផលជា descriptive (5 run មិនគ្រប់សម្រាប់អះអាងថាលឿនជាងជាទូទៅ)។
10. **គោលដៅ (§09, R2), ច្បាប់សម្រេចប្រកាសមុន:** LAN ≥ 90% goodput; CPU < 10% (4 core); RAM engine < 60 MB។ Ratio គូ r = ល្បឿន GDM / ល្បឿន IDM; ក្នុង URL មួយ median(r) ≥ 1.02 = លឿនជាង, 0.98–1.02 = ស្មើ (tolerance វិស្វកម្ម), < 0.98 = យឺតជាង ⇒ ត្រូវ "ស្មើ ឬលឿនជាង" លើ ≥ 3/5 URL; geometric mean នៃ median(r) ទាំង 5 ≥ 0.95 (= IDM −5%)។

## ច. Repo layout ស្នើ

```
Cargo.toml            # workspace: members = crates/*, src-tauri; exclude = spikes/*
crates/gdm-core/      # engine Rust ឯករាជ្យពី Tauri (ADR-1)
crates/gdm-bridge/    # native messaging host (~400 បន្ទាត់)
src-tauri/            # gdm.exe (Tauri v2 shell)
src/                  # UI React + TypeScript
extension/            # MV3 Chrome/Edge + Firefox
fixtures/             # hostile HTTP server (S4, រក្សាទុក) + data សាក
spikes/               # បោះចោល មិនដែល ship (exclude ពី workspace និង bundle)
docs/                 # blueprint, p0, evidence
```

បង្កើតថតនីមួយៗតែពេលមានកូដពិតដាក់ (G2/P1) មិនមែនឥឡូវទេ (Algorithm ជំហាន ២: មិនបង្កើតថតទទេ)។

## ឆ. Identifier (D11/D12 យល់ព្រម)

| កន្លែង | ចាស់ | ថ្មី |
|---|---|---|
| `package.json` name | galaxy-download-manage | galaxy-download-manager |
| `src-tauri/Cargo.toml` package / lib | galaxy-download-manage / galaxy_download_manage_lib | galaxy-download-manager / galaxy_download_manager_lib |
| `tauri.conf.json` productName / window title | galaxy-download-manage | Galaxy Download Manager |
| `tauri.conf.json` identifier | xyz.goldeniq | GoldenIQ.GalaxyDownloadManager |

ធ្វើក្នុង commit តូចមួយដាច់ពីឯកសារនេះ។

## Council review (ChatGPT/Codex, 2026-11-09, 2 ជុំ, read-only)

**អ្វីដែល Codex បន្ថែម (ផ្ទៀងផ្ទាត់ហើយ ទទួលយក):** ឈ្មោះ native host ត្រូវជាអក្សរតូច (ID D12 ប្រើមិនបាន ⇒ `com.goldeniq.gdm`); ដែនកំណត់ message ពីរទិស (1 MB host⇒browser); Firefox គ្មាន `background.service_worker` និង `onDeterminingFilename` ⇒ ផ្លូវ Firefox ដាច់; handoff ត្រូវមាន ticket durable + failure injection គ្រប់ transition; `FILE_GENERIC_WRITE` រួម `FILE_CREATE_PIPE_INSTANCE` ⇒ សាក instance ទីពីរ, SQOS identification, ព្រំដែន threat same-user; `latest.json` មិនចុះហត្ថលេខាទាំងមូល; `IAttachmentExecute::Save` មិនសន្យា ZoneId/HostUrl; ADS+AMSI មិនស្មើ Attachment Execution Services; handle `FILE_SHARE_READ` (exclusive នឹងរារាំង launch ខ្លួនឯង); NSIS មិនមែន rollback transaction; ពិនិត្យ shaping ខ្មែរ និងភាសាដែលជ្រើសដំបូង; failpoint កំណត់ក្នុង S5; ការសាក request ដែលមាន auth; media gate តូចសម្រាប់ ADR-6; SSRF matrix (IPv6, rebinding, DNS ទីពីរ) និង flag yt-dlp (`--ignore-config`, `--no-plugin-dirs`, `--`); protocol វាស់: LAN ជា gate, ឯកតា, លំដាប់ចៃដន្យ, geometric mean, ratio គូ, config ពីរប្រភេទ, pin build IDM។

**អ្វីដែល Codex ខុស ឬហួសវិសាលភាព (lead សម្រេច):** ស្នើប្ដូរលំដាប់ spike (S7/S8 មុន) ⇒ មិនប្ដូរ (លំដាប់ §17 ជាបញ្ជាម្ចាស់) តែអនុញ្ញាតរត់ស្របគ្នាក្រោយ S4។ Media matrix ពេញ (DASH/HLS), POST replay ពេញ, update serialization និង destination safety ជា spike ⇒ ផ្ទេរទៅ P1/P2។ Confidence interval លើ 5 run ⇒ មិនធ្វើ។ Codex យល់ព្រមទាំងអស់នេះនៅជុំ ២។

**ចំណុចដែល Codex កែ lead (ជុំ ២):** ច្បាប់ "range ជាន់គ្នា ±2%" របស់ lead គ្មាន denominator ហើយបោះបង់ការផ្គូ (NIST paired comparison) ⇒ ជំនួសដោយ ratio គូ និងច្បាប់ 0.98–1.02 ដែលប្រកាសមុន (ង.10)។

**អ្វីដែលទាំងពីរខកខាន ហើយ lead បន្ថែម:** P0 លើសពេល 3 សប្ដាហ៍ ⇒ ម្ចាស់ត្រូវដឹង; ភាពផ្ទុយ §17/§15 ជាមួយ D8 (MSIX ក្នុង P1 exit criteria)។

Prompt និងចម្លើយ council ទុកក្នុង scratchpad វគ្គ (មិនបញ្ចូល repo ព្រោះជាការពិភាក្សា មិនមែនភស្តុតាង spike)។

## ការបញ្ជាក់របស់ម្ចាស់

- [x] ម្ចាស់បញ្ជាក់បញ្ជីសេចក្ដីសម្រេច D1–D19 (ផ្នែក ក), 2026-11-09: ចម្លើយផ្ទាល់ "បាទ/ចាស បញ្ជាក់" ចំពោះសំណួរដែលរាយបញ្ជីទាំងមូល។
- [x] **D20 សម្រេច (2026-11-09): ក** = រក្សា RPL-1.5 (D1) ហើយបន្ថែម `TRADEMARKS.md` ដែលហាមកំណែកែប្រើឈ្មោះ "Galaxy Download Manager", "GDM" និង logo (គំរូ Firefox/Rust)។ LICENSE មិនប្ដូរទេ។ ការងារបន្ត: សរសេរ `TRADEMARKS.md` + យោងពី NOTICE.md/README ក្នុង G2 (licence gate)។
- [x] ម្ចាស់យល់ព្រម S1 ⇒ S1a/S1b និង P0 ≈ 4 សប្ដាហ៍ពេញ, 2026-11-09។

### គោលបំណង licence របស់ម្ចាស់ (2026-11-09, ពាក្យម្ចាស់សង្ខេប)

1. អ្នកប្រើទាំងអស់ប្រើបានដោយសេរី គ្មានកំហិតអ្វីទាំងអស់។
2. កូដបើកចំហ ប៉ុន្តែមិនមែន "ចេះតែយក" ទេ។ ការកែ ឬទាញយកកូដទៅប្រើជាលក្ខណៈឯកជន ធ្វើបាន។
3. ហាមយកកំណែកែទៅចេញជាសាធារណៈ ដើម្បីបង្ខូចបង្កាច់ ឬយកឈ្នះលើកំណែដើម ("លក់ឆ្នាំងបាយខ្លួនឯង")។ ការចែកចាយកំណែកែត្រូវសុំការអនុញ្ញាតពីម្ចាស់។

**ការផ្ទៀងផ្ទាត់ជាមួយ D1/D19 (RPL-1.5):** ចំណុច 1 និង 2 ត្រូវគ្នានឹង RPL-1.5។ ចំពោះចំណុច 3 RPL-1.5 ការពារបានមួយផ្នែក: §6.0 កំណែកែត្រូវប្ដូរឈ្មោះ, §5.0 មិនផ្ដល់សិទ្ធិពាណិជ្ជសញ្ញា, §6.2/§6.4(d) ត្រូវរក្សាឈ្មោះអ្នកនិពន្ធ, §6.1 កំណែកែដែល Deploy ត្រូវបោះពុម្ពកូដ។ តែ licence OSI ណាក៏ដោយ មិនអាច**ហាម**ការចែកចាយកំណែកែបានទេ (OSD 3 "Derived Works")។ SignPath តម្រូវ licence OSI។ ⇒ **ម្ចាស់ត្រូវសម្រេចមួយចំណុច (D20)**, ឆ្លើយក្នុងសារបិទ G1។
