# P0 exit (G4 / P0-D): design និង delivery baseline

ថ្ងៃ: 2026-11-09។ ស្ថានភាព: **P0 បិទ 2026-11-09**; ម្ចាស់អនុម័ត mockups + ឯកសារនេះ ទាំងកំណែក្រោយ G3។ G3 ចូល `results.md` (`75ccc68`) ⇒ ផ្នែក ៥ និង ៦ បានកែតាម G3; council បិទ P0 នៅផ្នែក ៧។

**លក្ខខណ្ឌមុន:** ពេលចាប់ផ្ដើម G4 មិនទាន់គ្រប់ទេ; ម្ចាស់ឱ្យធ្វើផ្នែក ១–៤ មុន (2026-11-09, "ល្មមធ្វើហើយអី")។ ឥឡូវគ្រប់ហើយ: G2 (council ២ ជុំ) និង G3 (council ២ ជុំ, `75ccc68`) នៅក្នុង `docs/p0/results.md`។ G3 បិទជា **ការវាយតម្លៃ** ដែល gate ល្បឿន durable ធៀប IDM **ធ្លាក់** ⇒ ផ្នែក ៥ (R12)។

## ១. Design tokens និង mockups

| អ្វី | Path |
|---|---|
| Design tokens (CSS variables, ងងឹត + ភ្លឺ + forced-colors) | `design/tokens.css` |
| Mockup ៥ ផ្ទាំង (ខ្មែរ/English, ភ្លឺ/ងងឹត តាម URL `?screen=&lang=&theme=`) | `design/mockups/index.html` |
| រូប (១០០%) ២០ សន្លឹក + high contrast ៥ សន្លឹក | `design/mockups/shots/<screen>-<km\|en>-<dark\|light>.png`, `…-en-highcontrast.png` |
| ទំព័រមើលរូបទាំងអស់ | `design/mockups/gallery.html` |
| Font ភ្ជាប់ក្នុង app (OFL-1.1) | `design/fonts/` (Kantumruy Pro khmer 400–700, Outfit 400–800, JetBrains Mono 400–500 ពី Fontsource 5.3.0) |
| Icon (Phosphor 2.1.1, MIT) | `design/mockups/icons.js` |
| Check ស្វ័យប្រវត្តិ | `design/check.cjs` ⇒ `docs/p0/design-check.json` (server តេស្ត bind 127.0.0.1 ហើយបម្រើតែ `design/` + `logo/`) |

ផ្ទាំងទាំង ៥ តាម §06: `main` (បញ្ជីមេ, ទម្រង់ medium 1100×700, rail 64 px), `details` (ទម្រង់ wide 1280×760, rail 220 px + details pane 340 px), `info` (ផ្ទាំង Download Info ជា modal), `settings`, `onboarding` (ជំហាន ២: ភ្ជាប់ browser)។

### លទ្ធផល check (រត់ចុងក្រោយ 2026-11-09)

ពាក្យបញ្ជា: `NODE_PATH=<playwright-core 1.65.0>/node_modules node design/check.cjs <ms-playwright/chromium-1228/chrome-win64/chrome.exe>` ⇒ **69/69 ok, exit 0** (រត់ម្ដងទៀតក្រោយការសម្រេច Q1/Q2): self-test ៩ + run ៦០ (៥ ផ្ទាំង × ២ ភាសា × ២ theme × deviceScaleFactor 1/1.25/1.5)។

| Check | របៀប | លទ្ធផល |
|---|---|---|
| Self-test (check ពិតជាចាប់កំហុសបាន) | Fixture ដែលដាក់កំហុសដោយចេតនា: outline, box-shadow មានពណ៌, inset ring 1 px, border-left 3 px, អក្សរខ្មែរកាត់ដោយប្រអប់ខ្លួនឯង និងដោយ ancestor, focus ដែលមិនប្ដូរអ្វី ⇒ ត្រូវតែ FAIL។ ការគ្រប់គ្រង (អក្សរមិនកាត់, focus fill) ⇒ ត្រូវតែ pass | ៩/៩ ដូចរំពឹង |
| Font ភ្ជាប់ពិត (គ្មាន fallback) | CDP `CSS.getPlatformFontsForNode` លើគ្រប់ node អក្សរ: font ពិតដែល Chromium គូរ ត្រូវតែជា Kantumruy Pro / Outfit / JetBrains Mono | ០ fallback |
| ខ្មែរមិនកាត់ (បញ្ឈរ) | Pixel: គូរអក្សរ បន្ទាប់មកគូរម្ដងទៀតដោយធ្វើឱ្យអក្សរថ្លា ⇒ ភាពខុសគ្នាគឺ ink ពិតរបស់អក្សរនោះ (មិនមែនរបស់ធាតុផ្សេង)។ ពេលនោះប្រអប់ `overflow:hidden/clip` ទាំងអស់ត្រូវដាក់ visible។ បើ ink ណាលើស ឬទាបជាងប្រអប់ ⇒ FAIL | ០ |
| កាត់ផ្ដេក | `scrollWidth > clientWidth`: អនុញ្ញាតតែ ellipsis ដែលមាន `title` ពេញ (អាចមើលអត្ថបទពេញ) | ០ (ellipsis មាន title) |
| គ្មាន glow/outline (ច្បាប់ ៨) | គ្រប់ធាតុ + `::before/::after`: border 0, outline none, box-shadow តែពណ៌ខ្មៅ និងគ្មាន inset ring, text-shadow none, គ្មាន drop-shadow; ទាំងធម្មតា និង forced-colors | ០ |
| Focus មើលឃើញ (D6) | ចុច Tab ពិត ហើយនៅ stop នីមួយៗ ថតរូបពេល focus, blur, ថតរូបពេល rest (មិនខូចពេល scroll), focus វិញ។ Pixel ដែលប្ដូរ contrast ≥ 3:1 ត្រូវ ≥ `4×(w+h)` (ផ្ទៃ perimeter 2 CSS px, WCAG 2.4.13)។ FAIL ពេល: មិនដល់, ធាតុ focus ចេញក្រៅអេក្រង់, គ្មាន tab stop, ឬ Tab ដល់ ៨០ | ឆ្លងគ្រប់ប៊ូតុង/ជួរ/tab/toggle (ក្រៅពី field មើលខាងក្រោម) |
| Windows high contrast | Chromium `forcedColors: active` | rules 0, focus ឆ្លងដូចគ្នា |

**ដែនកំណត់ដែលត្រូវដឹង (council):** `deviceScaleFactor` ធ្វើត្រាប់ DPI ប៉ុណ្ណោះ មិនមែនការ scale របស់ Windows ក្នុង WebView2 ទេ ហើយ forced-colors ក៏ជាការធ្វើត្រាប់របស់ Chromium មិនមែន theme contrast ពិតរបស់ Windows ដែរ។ ទាំងពីរត្រូវសាកលើ app ពិតក្នុង P1 (R4, R5)។ "69/69" គឺជាភស្តុតាងលើ mockup មិនមែនជាការបញ្ជាក់ WCAG conformance ទេ។

**Field** (search, ឈ្មោះឯកសារ, ថត): តាមការសម្រេចរបស់ម្ចាស់ Q2 ឥឡូវ field ប្រើ plate .40 ពេល focus (ទាំង mouse និង keyboard) ហើយត្រូវឆ្លងច្បាប់ផ្ទៃដដែលនឹងប៊ូតុង។ មុនការសម្រេចនេះ ច្បាប់ §06 (fill .04→.10) ផ្ដល់ contrast ត្រឹម 1.2–2.6:1 ក្នុង theme ងងឹត។

### អ្វីដែល check រកឃើញ ហើយកែរួច

1. **ជើងខ្មែរជាន់គ្នាត្រូវកាត់ 1–1.5 px** (ឧ. "តន្ត្រី", "គ្មានបណ្ដាញ", "ក្នុងជួររង់ចាំ") នៅក្នុងអត្ថបទមួយបន្ទាត់ដែលមាន ellipsis។ Check ចាស់ (canvas measureText) មិនបានឃើញទេ ប៉ុន្តែ check pixel ឃើញ ⇒ ជួសជុលដោយ `padding-block: .2em` លើអត្ថបទមួយបន្ទាត់ដែលកាត់។
2. **"⇒" គូរដោយ Cambria Math** (មិនមែន font ភ្ជាប់) ⇒ ប្ដូរជា "·"។ **អក្សរខ្មែរក្នុងប្រអប់ mono គូរដោយ Leelawadee UI** ⇒ stack mono = JetBrains Mono, GDM Khmer។
3. Focus ធាតុ selected (2.8–3.0:1) និងប៊ូតុង/tab/toggle gradient (≈1:1) ⇒ token `--focus-strong` (fill ផ្ទុយ ស/ខ្មៅ)។ ការណែនាំ §06 ចាស់ "accent-grad-hover + underline" មិនដល់ 2.4.13 ទេ។
4. High contrast: ធាតុ Highlight រួចហើយបាត់ focus ⇒ ប្ដូរគូ `Highlight`/`HighlightText`; toggle off (GrayText ធៀប Highlight មិនធានា) ⇒ track ជិត Canvas + thumb CanvasText; field ⇒ wrapper Highlight ពេលវាយ។
5. Modal: Tab រត់ទៅខាងក្រោយ ⇒ `inert`។ Field ចុងក្រោយក្នុង Settings focus ជាប់គែម window ⇒ `scroll-padding-block`។ Ellipsis ស្ថានភាព/URL គ្មាន `title` ⇒ បន្ថែម។

### ការប្ដូរពី §06 ដែលម្ចាស់ត្រូវដឹង

- **ដក `--hi-top`** (inset highlight 1 px ខាងលើ card/ប៊ូតុង)។ ច្បាប់ ៨ ហាម keyline ហើយបន្ទាត់ 1 px នេះអាចត្រូវមើលឃើញជា keyline ដូច្នេះខ្ញុំដកវាចេញ ដើម្បីកុំឱ្យខុសច្បាប់។
- **Theme ភ្លឺ** មាន token ផ្ទាល់ខ្លួន។ Lime `#b5e61d` មាន contrast ត្រឹម 1.4:1 លើផ្ទៃស ⇒ របារ "បញ្ចប់" ក្នុង theme ភ្លឺប្រើ `#4f8c0b`។
- **Focus = fill ផ្ទុយ** សម្រាប់ធាតុ gradient និងធាតុ selected។
- **របារ 3 px នៅគែមឆ្វេងនៃជួរ selected** នៅតែរក្សាទុក ព្រោះ §06 ដែលម្ចាស់អនុម័តសរសេរវាច្បាស់ ("fill មិនមែនបន្ទាត់ព័ទ្ធជុំវិញ")។ Council ជុំ ១ ចាត់វាជា keyline ប៉ុន្តែក្នុងជុំ ២ បានយល់ព្រមថាវាមិនមែនជា outline ព័ទ្ធជុំវិញទេ។ ការពន្យល់របស់ខ្ញុំ: វាជា fill នៅគែមម្ខាង មិនមែនជា stroke ព័ទ្ធជុំវិញទេ។ មើលសំណួរ Q3។

### ការសម្រេចរបស់ម្ចាស់ (2026-11-09)

| # | សំណួរ | ចម្លើយ | អនុវត្ត |
|---|---|---|---|
| Q1 | ព្រំដែនក្នុង Windows high contrast | **អនុញ្ញាតតែក្នុង HC**: border `ButtonText` 1 px លើប៊ូតុង, field, toggle, tab; theme ភ្លឺ/ងងឹតនៅតែគ្មាន border | `design/tokens.css` (`@media (forced-colors: active)`); check អនុញ្ញាត border តែក្នុង run HC |
| Q2 | Field focus ក្នុង theme ងងឹតខ្សោយ (1.2–2.6:1) | **Plate .40** (ដូចប៊ូតុង) | `.in:focus-within`, `.search:focus-within`; field ឆ្លងច្បាប់ផ្ទៃ 2.4.13 ដូចធាតុផ្សេង |
| Q3 | របារ 3 px នៃជួរ selected | **រក្សា** (§06) | គ្មានការប្ដូរ |
| Q4 | Windows 10 22H2 x64 | **"ដំណើរការបាន, មិនសន្យា"**: VM តេស្តមួយដងក្នុង P1, កែ bug ធំ; ការគាំទ្រផ្លូវការ = Windows 11 x64 + arm64 | ផ្នែក ៣ |

| Q5 | ការប្ដូរ scope P1 ៣ ចំណុច (ផ្នែក ៣ ចុងក្រោយ) | **យល់ព្រមទាំង ៣** | P1 ត្រូវបានបង្កក |

**ម្ចាស់អនុម័ត mockups និងឯកសារនេះ (2026-11-09)។** P0 នៅបើកដដែល រហូតដល់ផ្នែក G3 ក្នុង `results.md` ឆ្លង council ហើយ council ពិនិត្យការបិទ P0 ត្រូវបានកត់ត្រា។

## ២. ច្បាប់ design ម្ចាស់ (D6)

គ្មាន glow, គ្មាន border/outline/ring នៅកន្លែងណាមួយ (check ស្វ័យប្រវត្តិ 0, self-test បញ្ជាក់ថា check ចាប់កំហុសទាំងនោះបាន)។ Focus ប្រើតែ fill/ពណ៌: `--focus-plate` (ធាតុធម្មតា), `--focus-strong` (gradient/selected), `Highlight`/`HighlightText` (high contrast)។ ពិនិត្យ focus រួចក្នុង theme ទាំងពីរ និង forced-colors (ការធ្វើត្រាប់)។ Check CSS មិនអាចបញ្ជាក់ "គ្មាន keyline ដែលមើលឃើញ" ពេញលេញទេ ⇒ ម្ចាស់មើលរូបក្នុង `gallery.html`។

## ៣. វិសាលភាព P1 ដែលបង្កក

### Windows និង architecture

| អ្វី | ស្នើ | ភស្តុតាង / ស្ថានភាព |
|---|---|---|
| Windows 11 x64 | **គាំទ្រ, តេស្ត** | G2 ឆ្លងលើ 26H2 build 26300.9711 |
| Windows 11 arm64 | **គាំទ្រ (§15 build x64 + arm64), provisional (preview)** រហូតដល់រត់លើឧបករណ៍ពិត | គ្មានម៉ាស៊ីន arm64 សាក ⇒ R3 |
| Windows 10 22H2 x64 | **ដំណើរការបាន, មិនសន្យា** (ម្ចាស់ Q4) | G2 #19 NOT RUN (គ្មាន VM)។ Microsoft បញ្ចប់ការគាំទ្រ Windows 10 ថ្ងៃ 2025-10-14 ហើយ ESU សម្រាប់អ្នកប្រើធម្មតាផុតនៅ 2026-10-13។ Persona ១ (laptop ចាស់) ប្រហែលនៅប្រើ Windows 10។ អនុសាសន៍: "ដំណើរការបាន, មិនសន្យា" + តេស្ត VM មួយដងក្នុង P1 |
| WebView2 | Evergreen runtime (មានស្រាប់ក្នុង Windows 11; NSIS ដំឡើងបើខ្វះ) | Tauri 2 default |
| Installer | **NSIS per-user** (ផ្លូវចម្បង G2) | G2 #4–#12 PASS |
| MSIX / Store | **ផ្លាស់ទៅ P2** (§17 P2 មាន Store រួចហើយ) | G2 #17 BLOCKED, D8/D9 Store ពេលក្រោយ ⇒ ដក exit criteria P1 "MSIX ឆ្លង Store certification" |

### ជួរ P1 ទាំង ៣២ នៃ §04 និង acceptance test

| # | មុខងារ | Acceptance test (ត្រូវរត់បាន, មាន evidence) |
|---|---|---|
| 1 | Dynamic segmentation | Fixture 1 GiB: ចាប់ 2 connection, កើនដល់ N ពេលល្បឿនកើន; split តែផ្នែក ≥ 1 MiB (log); hash ត្រូវ |
| 2 | ពិនិត្យ seam/overlap | Hostile fixture `200-on-range`, Content-Range ខុស, gap/overlap, body ខ្លី, `416`, ETag ប្ដូរ ⇒ ផ្អាកជាមួយមូលហេតុ, មិនធ្លាក់ទៅ 1 connection ដោយស្ងាត់, hash មិនខុស (E-01, E-02) |
| 3 | Connection 1–32 live | ប្ដូរ 8⇒2⇒16 ពេលកំពុងទាញ ⇒ socket ពិតតាម (log) ក្នុង ≤ 2 វិ, hash ត្រូវ |
| 4 | Speed limiter | **P1: សរុប + job + toggle លើ tray** ⇒ វាស់ 60 វិ ក្នុង ±10% នៃកម្រិត។ ផ្នែក queue និងផែនការម៉ោង ⇒ P2 (ជាមួយ queue) |
| 5 | Resume ក្រោយ crash | `TerminateProcess` នៅចំណុចកំណត់ (មុន/ក្រោយ write, flush, DB commit, rename) 1000 ដង ⇒ 0 ឯកសារខូច (E-04)។ មិនមែនជាការធ្វើត្រាប់ដាច់ភ្លើងទេ (power loss មិនត្រូវបានសាក) |
| 6 | Validator ខ្លាំងពេល resume | Fixture ETag/Last-Modified ប្ដូរ ⇒ សួរ [ចាប់ផ្ដើមឡើងវិញ] [រក្សា]; គ្មាន validator ⇒ "មិនបានផ្ទៀងផ្ទាត់"; គ្មាន Complete ខុស (E-02) |
| 7 | HTTP/1.1, HTTP/2, HTTPS | Fixture h1 + h2 ⇒ ទាញបាន; badssl expired/wrong-host/self-signed ⇒ បរាជ័យ (W-03); CA ក្រុមហ៊ុនក្នុង Windows store ⇒ ទទួល (`rustls-platform-verifier`) |
| 8 | User-Agent | Fixture កត់ header: UA សកល និង UA តាម site ត្រូវតាមការកំណត់ |
| 9 | .part / temp folder | `.part` ក្នុងថតគោលដៅ ⇒ rename atomic; temp folder នៅ volume ផ្សេង ⇒ copy + flush + rename លើ volume គោលដៅ; disk ពេញ / ឯកសារត្រូវ lock / crash ពេល finalise ⇒ គ្មានឯកសារពាក់កណ្ដាលដែលមានឈ្មោះចុងក្រោយ; ជាន់ឈ្មោះ ⇒ ឈ្មោះថ្មី (N-04) |
| 10 | Hash verify | `.sha256`/`.sha512` ពី origin ដដែល ⇒ ✓; hash ខុស ⇒ "ផ្ទៀងផ្ទាត់មិនត្រូវ", មិនបង្ហាញ Complete |
| 11 | Chrome, Edge, Firefox, Opera, Brave, Vivaldi | Host manifest ចុះឈ្មោះគ្រប់ទីតាំង (B-07) ⇒ `sendNativeMessage` ពី browser នីមួយៗ ok (G2 បញ្ជាក់ Chrome/Edge/Firefox រួច; Opera/Brave/Vivaldi ត្រូវបន្ថែម); sender ក្លែង ⇒ បដិសេធ (B-01) |
| 12 | ចាប់តាមប្រភេទឯកសារ | 20 គេហទំព័រពិត (Reddit, Synology, Google Docs export) ⇒ 0 ចាប់ខុស (C-01); MIME + ទំហំអប្បបរមា |
| 13 | បញ្ជីលើកលែង site | "កុំចាប់គេហទំព័រនេះ" មួយចុចក្នុង popup ⇒ ការទាញបន្ទាប់ពី site នោះទុកឱ្យ browser |
| 14 | Ctrl+click / Alt+click | Ctrl ⇒ GDM ទាញ; Alt ⇒ browser ទាញ; rebind ក្នុង Settings (C-04); បង្ហាញក្នុង onboarding (mockup ✓) |
| 15 | Context menu ទាញ / ទាញ link ទាំងអស់ | ទំព័រ fixture 50 link ⇒ preview បញ្ជី ⇒ ជ្រើស ⇒ job ត្រឹមត្រូវ |
| 16 | Incognito | ទាញពី incognito ⇒ គ្មានដាននៅក្នុង DB, log, toast/notification history, ឯកសារ crash-recovery ក្រោយបញ្ចប់ (C-05) |
| 17 | Toggle ចាប់លើ icon extension | Popup: ស្ថានភាព, បិទការចាប់, ថ្មីៗ, គេហទំព័រនេះ; បិទ ⇒ 0 handoff |
| 18 | ប៊ូតុង "Browsers" ដំឡើង extension | Onboarding + Settings បង្ហាញ browser ដែលរកឃើញ ហើយស្ថានភាពប្ដូរជា "បានភ្ជាប់" ពេល hello (mockup ✓) |
| 19 | Category: ថត, ប្រភេទ, site pattern | .zip ⇒ ថត Compressed; site pattern ⇒ ថតកំណត់; "ចងចាំថត" ⇒ ប្រើលើកក្រោយ |
| 20 | Tree + ស្វែងរក FTS | 10 000 ជួរ ⇒ ស្វែងរកឈ្មោះ/URL (ខ្មែរ + ឡាតាំង) < 100 ms |
| 21 | Drag-drop link / drop basket | អូស link ពី browser ទៅ window ⇒ Download Info; basket តូចលើ desktop ទទួល drop |
| 22 | Download Info | ទាញភ្លាមពេលផ្ទាំងបើក (bytes > 0 មុនចុច Start); [បោះបង់] ⇒ លុប `.part` ដែលទាញទុកមុន; [ទាញពេលក្រោយ] ⇒ **បញ្ជី "ពេលក្រោយ" តែមួយ គ្មាន scheduler** (ចុចចាប់ផ្ដើមដោយដៃ; queue ពិត P2); ផ្ទាំងរួមពេលមានច្រើន (C-03); focus ចូល modal ពេលបើក ហើយត្រឡប់វិញពេលបិទ។ Preview zip ⇒ P3 |
| 23 | Progress: connection, segment bar, limiter | Details pane + ទម្រង់ mini (360×140) បង្ហាញ segment ពិត, ល្បឿន, limiter |
| 24 | Complete: បើក, ថត, អូសឯកសារចេញ | Toast មានសកម្មភាព; ឯកសារគ្រោះថ្នាក់គ្មាន [បើក]; អូសពី toast ទៅ Explorer ⇒ ចម្លង |
| 25 | Silent mode | 0 toast/dialog ក្នុងការទាញ 10 ឯកសារ; **ការសម្រេចដែលត្រូវការ (validator ប្ដូរ, login) មិនត្រូវលាក់ទេ** ⇒ បង្ហាញជាស្ថានភាពជួរ (គ្មាន modal) |
| 26 | Properties | កែ URL/ថត/description ⇒ រក្សាទុក; **login តាម site ⇒ P2** (ជាមួយ Credential Manager) |
| 27 | Column បង្ហាញ/លាក់/រៀប/តម្រៀប | ការកំណត់នៅដដែលក្រោយ restart |
| 28 | លុបជាមួយឯកសារ | ឯកសារទៅ Recycle Bin (មិនលុបជាស្ថាពរ) |
| 29 | Tray menu | limiter, ផ្អាកទាំងអស់, បើក/ចេញ; queue ⇒ P2 |
| 30 | Dark mode | ផ្ទាំងទាំងអស់មាន theme ទាំងពីរ; `design/check.cjs` ឆ្លងលើ UI ពិត (Playwright + WebView2) |
| 31 | ភាសា (ខ្មែរ + EN) | គ្មាន string hard-code (lint); check pixel ខ្មែរ 100/125/150% គ្មាន clip; អ្នកអានខ្មែរពិនិត្យ glossary |
| 32 | Update / licence / ចែករំលែក | Updater minisign: manifest ក្លែង/សញ្ញាខុស/version ចាស់ ⇒ បដិសេធ (U-01); About បង្ហាញ RPL Exhibit B + ប៊ូតុងចែករំលែក link |

### កាត critical/high ដែល P1 ត្រូវឆ្លង (exit criteria §17)

៣២ កាតក្នុង F01–F05, F12–F17 (ពីសរុប 96 កាត): B-01, B-02, B-03, B-04, B-09, C-01, C-02, E-01, E-02, E-03, E-04, N-01, N-02, N-03, N-04, N-05, N-09, W-01, W-02, W-03, D-01, U-01, U-02, U-03, U-06, X-01, X-02, T-01, P-01, S-01, S-02, S-03។ Test នីមួយៗយកពី field `test` នៃកាតក្នុង `docs/blueprint/scenarios.py`។

### ការប្ដូរ scope (ម្ចាស់យល់ព្រម Q5)

1. MSIX + Store certification ⇒ ចេញពី P1 exit ទៅ P2 (G2 #17)។
2. Limiter ផ្នែក queue/ម៉ោង, tray queue, login ក្នុង Properties ⇒ P2 ព្រោះ queue និង site logins ជា P2 ក្នុង §04 រួចហើយ។ P1 "ទាញពេលក្រោយ" = បញ្ជីតែមួយគ្មាន scheduler។
3. TLS: `rustls-platform-verifier` (ជម្រើសតែមួយ, Windows cert store) មិនមែន `webpki-roots` (ផ្នែក ៤)។ Feature graph ពិតត្រូវផ្ទៀងផ្ទាត់ក្នុង P1។

## ៤. Dependency និង licence inventory

### Rust: `deny.toml` (cargo-deny 0.20.2)

Allow: MIT, Apache-2.0, BSD-2/3-Clause, ISC, Zlib, Unicode-3.0, OFL-1.1, MPL-2.0 (តាមបញ្ជីម្ចាស់; ខ្ញុំដក "Apache-2.0 WITH LLVM-exception" ចេញវិញ ព្រោះមិនមាន crate ណាត្រូវការ ហើយវាមិននៅក្នុងបញ្ជីម្ចាស់)។ អ្វីផ្សេងទៀតត្រូវបដិសេធ (GPL/AGPL/LGPL/SSPL និង licence មិនស្គាល់)។ Exception: crate GDM ខ្លួនឯង (RPL-1.5, តាមឈ្មោះ)។ Sources: crates.io តែប៉ុណ្ណោះ (គ្មាន git/registry ផ្សេង)។ Targets: x64 + arm64 Windows។

| Manifest | ពាក្យបញ្ជា | លទ្ធផល |
|---|---|---|
| `src-tauri/Cargo.toml` (app) | `cargo deny --manifest-path src-tauri/Cargo.toml --config deny.toml check` | **advisories ok, bans ok, licenses ok, sources ok**។ MIT 197, Apache-2.0 150, Unicode-3.0 19, Zlib 5, BSD-3 4, **MPL-2.0 5** (cssparser, cssparser-macros, dtoa-short, option-ext, selectors: មកពី Tauri ⇒ D18 ត្រូវការពិត)។ 0BSD/CC0/MIT-0/Unlicense លេចតែជាជម្រើស "OR" ⇒ ប្រើ MIT |
| Negative test (scratchpad crate ពឹងលើ `sequoia-openpgp`) | `check licenses` | **FAILED ដូចរំពឹង**: `LGPL-2.0-or-later` (buffered-reader …) ត្រូវបដិសេធ |
| `spikes/engine/Cargo.toml` (reqwest/rustls ដែល P1 នឹងប្រើ) | `check licenses` | **FAILED**: `webpki-roots` = **CDLA-Permissive-2.0** (ទិន្នន័យ CA ពី Mozilla) តាម `reqwest` feature `rustls-tls` (+ crate spike RPL-1.5 មិននៅក្នុង exception ព្រោះ spike មិនចែកចាយ) |
| `spikes/distribution/bridge/Cargo.toml` | `check licenses` | **FAILED**: `webpki-roots` (តាម `ureq`) + crate spike គ្មាន license field |

**ការសម្រេចដែលស្នើ:** មិនបន្ថែម CDLA-Permissive-2.0 ទេ។ P1 engine ប្រើ `reqwest` + `rustls-platform-verifier` (Windows cert store)។ វិធីនេះត្រូវនឹង §09 ("rustls + platform verifier") ហើយ CA ក្រុមហ៊ុនរបស់ persona ២ ក៏ដំណើរការផងដែរ។ G2 finding ៦ ស្នើរួចហើយថា bridge ពិតមិនត្រូវការ HTTP ដូច្នេះ `webpki-roots` ក្នុង bridge នឹងបាត់ទៅវិញដោយខ្លួនឯង។

**MPL-2.0 "មិនកែប្រែ":** cargo-deny មិនអាចបញ្ជាក់រឿងនេះបានទេ (council)។ ច្បាប់ P1: គ្មាន `[patch]`, path dependency ឬ vendored copy នៃ crate MPL (បច្ចុប្បន្ន 0) ហើយ `Cargo.lock` checksum ពី crates.io ត្រូវរក្សា។ ពេលចែកចាយ executable, MPL §3.2 តម្រូវឱ្យប្រាប់អ្នកទទួលពីរបៀបទទួល source ⇒ THIRD-PARTY-NOTICES ត្រូវមានឈ្មោះ crate MPL + version + link crates.io។

### npm: `pnpm licenses list` (pnpm 11.15.0)

| ប្រភេទ | លទ្ធផល |
|---|---|
| Production (ចូលក្នុង app) | react, react-dom, scheduler (MIT); @tauri-apps/api, @tauri-apps/plugin-opener (MIT OR Apache-2.0) ⇒ **ok** |
| Dev (មិនចែកចាយ) | MIT 13, Apache-2.0 2 (typescript, detect-libc), MPL-2.0 1 (lightningcss), ISC 1, BSD-3 1 ⇒ ok |
| Design assets ថ្មី | Kantumruy Pro, Outfit, JetBrains Mono (OFL-1.1, អត្ថបទ licence ក្នុង `design/fonts/OFL-*.txt`); Phosphor (MIT, `design/mockups/LICENSE-phosphor.txt`) |

ខ្ញុំប្រើ `pnpm licenses list` ដែលមានស្រាប់ក្នុង pnpm ជំនួស `license-checker` ព្រោះ repo ប្រើ pnpm ហើយ license-checker អាន `node_modules` ដែលជា symlink របស់ pnpm មិនបានល្អ។

### ផែនការ THIRD-PARTY-NOTICES (P1, ក្នុង release CI)

1. Rust: `cargo about generate` (មិនទាន់ដំឡើង: `cargo install cargo-about --locked`) ជាមួយ accepted list ដូច `deny.toml` ⇒ អត្ថបទ licence ពេញ + notice Apache §4 + MPL §3.2 (ទីតាំង source)។
2. npm: `pnpm licenses list --prod --json` ⇒ script តូចភ្ជាប់អត្ថបទ LICENSE ពី `node_modules`។
3. Font + icon: ចម្លង `OFL-*.txt` និង Phosphor MIT ដោយផ្ទាល់។
4. Component ដែលទាញពេលដំណើរការ (**មិន bundle**): yt-dlp (Unlicense; binary មានផ្នែក GPLv3+) ពី `github.com/yt-dlp/yt-dlp/releases` និង ffmpeg **BtbN FFmpeg-Builds `win64-lgpl`** ពី GitHub។ ទាំងពីរទាញទៅ `%LOCALAPPDATA%\GDM\components` តាម manifest ចុះហត្ថលេខា minisign + sha256 (G2 #10, #11 PASS) ហើយរត់ជា process ដាច់ (មិន link)។ THIRD-PARTY-NOTICES កត់ URL, version និង sha256 ប៉ុន្តែមិនចែកចាយ binary ទេ។ មុនបើកមុខងារវីដេអូ (P2) ត្រូវ pin artifact ពិតប្រាកដ និង audit licence/configure flags (គ្មាន `--enable-gpl`)។ ការរត់ជា process ដាច់មិនដោះស្រាយកាតព្វកិច្ចចែកចាយទាំងអស់ដោយខ្លួនឯងទេ (council)។

## ៥. ការប៉ាន់ស្មាន P1 ឡើងវិញ (ក្រោយ G3)

§17: P1 = **8 សប្ដាហ៍** (អ្នកអភិវឌ្ឍម្នាក់ពេញម៉ោង)។ ភស្តុតាង: `docs/p0/results.md` G2 + G3 (commit `75ccc68`)។

| ការប្ដូរ | សប្ដាហ៍ | ភស្តុតាង |
|---|---|---|
| MSIX/Store ចេញពី P1 | −1.0 | G2 #17 BLOCKED, D8/D9; §17 P1 មាន "installer NSIS + MSIX" និង "MSIX ឆ្លង Store certification" ⇒ ការងារ package + certification ចេញ (ចំនួន 1.0 ជាការប៉ាន់ស្មាន) |
| Installer ខ្មែរ: LangDLL font, បកប្រែ ~250 ឃ្លា, BOM, អ្នកអានពិនិត្យ | +0.5 | G2 #13, finding ៣–៥ |
| Upgrade ខណៈ host រត់ + update idle (U-02); ផ្លូវ update ពិត (plan B ថត A/B + pointer បាន PASS, NSIS in-place មិនបានសាក) | +0.8 | G2 finding ២; G3 S8 |
| ការតេស្តលើ Windows 10 VM និង arm64 (គ្មានឧបករណ៍) | +0.5 | G2 #19, ផ្នែក ៣ |
| TLS platform verifier ជំនួស webpki-roots | +0.2 | ផ្នែក ៤ |
| Opera/Brave/Vivaldi + browser fork (B-07) | +0.3 | ផ្នែក ៣ ជួរ 11 |
| ៣២ កាត critical/high (fuzz 24 ម៉ោង, kill ×1000, matrix MOTW) | +1.0 | §17 exit; ពេលវេលា test infrastructure/CI ដែលដាក់ឱ្យច្បាស់ថ្មី (មិនដឹងថា 8 សប្ដាហ៍ដើមរាប់វាឬអត់) |
| UI ពិត (component, keyboard, modal focus lifecycle, grid/tab semantics, check.cjs លើ WebView2) | 0 | Mockup មិនមែនជា component ទេ ⇒ មិនកាត់ពេល (council) |
| **ល្បឿន durable: gate ធៀប IDM ធ្លាក់** (geo-mean r 0.59–0.80 ទល់គោលដៅ ≥ 0.95; engine network 124.8 MB/s ឆ្លង) ⇒ writer ដាច់ + queue កំណត់ + checkpoint តាមពេល, profile flush, វាស់ម្ដងទៀតលើ NVMe ទំនេរ + LAN ពិត + iperf3 | **+1.5** (0.5–3.0) | G3 Benchmark + Fallback; plan S5 |
| Fallback G3 ដែលនៅសល់ (៥ ការងារ): (ក) ticket durable + failure injection + 200 handoff (S2) 0.3; (ខ) ACL ថតដំឡើង + pipe instance ច្រើន + Authenticode តាម handle (S3) 0.3; (គ) Job Object limits + allowlist extractor (S7) 0.2; (ឃ) Defender/AMSI ជាភស្តុតាង AV វិជ្ជមាន + ENOSPC (S6) 0.2; (ង) FAT/exFAT/SMB ពិត ក្នុង CI ≈0 (ពេលរង់ចាំឧបករណ៍ មិនមែនពេលអភិវឌ្ឍ) | **+1.0** (0.8–1.5) | G3 តារាង "មិនបានបង្ហាញ ⇒ fallback / P1" |
| កូដ spike គឺ throwaway; មានតែ fixture S4 (`spikes/engine/src/fixture.rs`) ដែលយកទៅ `fixtures/` | −0.3 | G3 ក្បាលផ្នែក |

**សរុប: ≈ 12.5 សប្ដាហ៍ (ចន្លោះ 11.3–14.5)** ទល់ 8 សប្ដាហ៍ក្នុង §17។ ចន្លោះ = ល្បឿន durable 0.5–3.0 និង fallback 0.8–1.5។ ការប៉ាន់ស្មាននេះគឺ adjustments លើ 8 សប្ដាហ៍ដើមប៉ុណ្ណោះ (blueprint មិនផ្ដល់ការបំបែក 8 សប្ដាហ៍) ហើយជា **conditional** លើ R12។ ហានិភ័យធំជាងគេ = ល្បឿន durable (R12): បើ writer ថ្មីនៅតែមិនដល់ ≥ IDM −5% លើ disk ធម្មតា ការប៉ាន់ស្មាននេះមិនគ្រប់ទេ។ ការប៉ាន់ស្មាននេះមិនមែនជាការសន្យាទេ; មានតែ exit criteria ទេដែលជាការសន្យា (§17)។ Internet (CDN ៥, observation): ស្មើ/លឿនជាង IDM 4/5 URL, geo-mean 2.30 ⇒ លក្ខខណ្ឌ "≥ 3/5 URL" ទំនងឆ្លង តែលក្ខខណ្ឌ "ល្បឿនមធ្យម ≥ IDM −5%" លើ LAN gate មិនទាន់ឆ្លង។

### Protocol gate ល្បឿន P1 (បង្កក)

§17 "ល្បឿនមធ្យម ≥ IDM −5%" នៅ P1 មានន័យថា protocol `docs/p0/plan.md` ចំណុច ង: LAN fixture 1 GB (S4) ជា gate; គូ GDM/IDM លំដាប់ AB/BA ចៃដន្យមានតុល្យភាព; warm-up 1 គូមិនរាប់; ≥ 5 គូក្នុងមួយ config; ratio គូ ⇒ median ក្នុងមួយ URL ⇒ geometric mean ≥ 0.95; MB/s ទសភាគ; GDM done = ចប់ durable (final sync + sha256 + MOTW + rename)។ Disk: NVMe ទំនេរ ≥ 25% (gate) និង HDD (រាយការណ៍)។ Internet (CDN ៥) = observation សម្រាប់ "≥ 3/5 URL"; IDM timeout ត្រូវកត់ តែមិនរាប់ជាជ័យជម្នះ GDM។

**Checkpoint សប្ដាហ៍ទី ២ នៃ P1** (ក្រោយ writer ថ្មី): វាស់តាម protocol ខាងលើ ⇒ ជ្រើសមួយ: (១) ឆ្លង ⇒ បន្ត; (២) ជិតឆ្លង ហើយមានមូលហេតុវាស់បាន ⇒ បន្តកែ និងកែកាលវិភាគ; (៣) មិនអាចឆ្លងដោយរក្សា "commit តែក្រោយ flush" ⇒ ម្ចាស់សម្រេចប្ដូរ acceptance។ មិនបន្ធូរ "commit តែក្រោយ flush" ដោយស្ងាត់ទេ។ Kill ×1000 ត្រូវរត់ម្ដងទៀតលើ engine ផលិតផល ក្រោយប្ដូរ writer។

## ៦. ហានិភ័យដែលនៅបើក

| # | ហានិភ័យ | ផលប៉ះពាល់ | ការធ្វើ / បិទពេល |
|---|---|---|---|
| R2 | Windows 10 មិនបានសាក; ស្ថានភាពគាំទ្រ | Persona ១ (laptop ចាស់) | VM តេស្តមួយដងក្នុង P1 (Q4) |
| R3 | arm64 គ្មានឧបករណ៍សាក | build ចេញ តែមិនបានរត់ពិត ⇒ arm64 = **provisional (preview)** រហូតដល់រត់លើឧបករណ៍ពិត | arm64 VM/ឧបករណ៍ក្នុង P1 |
| R4 | Check HC ជាការធ្វើត្រាប់ Chromium | Windows HC ពិត (WebView2) អាចខុស | សាក theme contrast ពិត ៤ (Aquatic, Desert, Dusk, Night sky) លើ UI ពិតក្នុង P1 |
| R5 | Mockup ជា HTML; scale ធ្វើត្រាប់ដោយ deviceScaleFactor | Windows scaling/WebView2 zoom និងទំហំ window អប្បបរមាមិនទាន់សាក | រត់ `check.cjs` លើ app ពិតក្នុង P1 CI + សាក 100/125/150% ពិត |
| R6 | អត្ថបទខ្មែរក្នុង mockup និង installer (G2 #15 "រង់ចាំម្ចាស់") មិនទាន់មានអ្នកអានខ្មែរពិនិត្យ | §05 គោលការណ៍ ៦ (គ្មានការបកប្រែម៉ាស៊ីន) | **G2 #15 ពន្យារទៅ P1** (មិនរារាំង P0): អ្នកអានខ្មែរពិនិត្យ glossary + `docs/p0/g2-screens/` មុន v0.9 |
| R8 | MSIX native messaging មិនទាន់បញ្ជាក់ | ផ្លូវ Store (P2) អាចត្រូវការផែនការ B | Spike ពេញមុន Store (G2 plan S1b) |
| R9 | `webpki-roots` / CDLA | P1 build បរាជ័យ cargo-deny បើប្រើ reqwest default | Platform verifier (ផ្នែក ៤) |
| R10 | Key ចុះហត្ថលេខា updater / extension store account / migration DB | បាត់ key ឬ account ⇒ update មិនបាន (U-03, S-03) | Drill ស្ដារ key + checklist release មុន v0.9 |
| R11 | `src-tauri/Cargo.toml` `authors` មាន "CHHIT Sungchhay" ក្បែរ "Golden IQ" | ច្បាប់ ៥: author ត្រូវជា GoldenIQ ប៉ុណ្ណោះ (§14/§15 ក៏កត់ © Golden IQ និង CHHIT Sungchhay) | ម្ចាស់បញ្ជាក់ថាជាឈ្មោះម្ចាស់ខ្លួនឯង ឬត្រូវដក (ខ្ញុំមិនបានកែ) |
| R12 | ល្បឿន durable ចាញ់ IDM (r 0.59–0.80) លើម៉ាស៊ីននេះ; ផ្នែកមួយនៃភាពខុសគ្នាមិនទាន់ពន្យល់ (8 conn ថេរនៅតែចាញ់លើ C:) | P1 exit "≥ IDM −5%" អាចមិនឆ្លង ⇒ P1 យឺត; ឬ **មិនអាចឆ្លងបានទាល់តែសោះ** លើ disk ខ្លះ ព្រោះ final sync ជាតម្លៃដែល writer ថ្មីលុបមិនបាន | សប្ដាហ៍ទី ១–២ នៃ P1: writer ដាច់ + checkpoint + profile; វាស់លើ NVMe ទំនេរ ≥ 25% និង LAN ពិត។ checkpoint សប្ដាហ៍ទី ២ (ផ្នែក ៥); រក្សា invariant "commit តែក្រោយ flush"; "fast finish មិន durable" ដូច IDM មិនបានយក (ជាសេចក្ដីសម្រេចផលិតផល មិនមែនការកែ bug) |
| R13 | ភស្តុតាង G3 ជា local validation (គ្មាន admin): FAT/exFAT/SMB, account ទីពីរ, DNS rebinding server ពិត, extractor ដែលរំលង proxy មិនបាន confine | ចំណុចសុវត្ថិភាពខ្លះនៅលើអំណះអំណាងរចនាសម្ព័ន្ធ | រត់ `check.ps1 s6 <path>` / S3 លើ volume/account ពិតក្នុង P1 CI; M-08 "មិនគាំទ្រ"; WFP user-mode ពិនិត្យ P2 |
| R14 | Resume គ្មាន SHA-256 ដែលដឹងមុន: seam check មើលមិនឃើញការខូចខាងក្នុង prefix ដែល commit | ឯកសារអាចខូចដោយស្ងាត់ក្រោយ resume | UI បែងចែក "ទាញចប់ durable" ពី "ផ្ទៀងប្រភពរួច" (តែពេលមាន hash); test validator ខ្សោយ/គ្មាន, content ប្ដូរ, sidecar ខូច |
| R15 | M-08 "មិនគាំទ្រ" មិនមែនការ confine; Job Object មិនមែន network confinement; proxy បញ្ជូនតែ read ដំបូង (គ្មាន POST body) | Extractor អាចរំលង proxy | Allowlist extractor + បដិសេធផ្លូវ network ក្រៅ proxy មុនរត់; test POST/extractor មិនគាំទ្រ ⇒ បដិសេធ |
| R16 | `.part` sparse មិនកក់ទំហំ ⇒ ENOSPC អាចកើតគ្រប់ដំណាក់កាល | ខូច sidecar/metadata ឬ finalise ពាក់កណ្ដាល | Test ENOSPC ពេលទាញ, checkpoint, ដក sparse, hash, finalise, និងទាញច្រើនព្រមគ្នា ⇒ state កំហុសត្រឹមត្រូវ, 0 ឯកសារខូច។ Kill test = process crash មិនមែនដាច់ភ្លើង |

## ៧. Council (G4 review, 2026-11-09)

ChatGPT (Codex CLI, read-only) ពីរជុំ។ ខ្ញុំជាអ្នកដឹកនាំ ហើយបានផ្ទៀងផ្ទាត់គ្រប់ការអះអាងជាមួយកូដ ឬការរត់ពិត។

- **ទទួលយក និងកែរួច:** ផ្ទៃ 2.4.13 ត្រូវជា `4×(w+h)` មិនមែន `2×(w+h)`; "visible<2.4.13" ត្រូវតែ FAIL; check `title` (`el.title !== undefined` តែងតែពិត); clip ដោយ ancestor; DPR ≠ Windows scaling; font fallback; field/toggle ក្នុង forced-colors; cargo-deny ត្រូវរត់គ្រប់ check; MPL §3.2; P1 "ពេលក្រោយ", silent mode, temp volume ផ្សេង, incognito, kill test; ដកការកាត់ −0.5 សប្ដាហ៍។ Check pixel ថ្មីបានរកឃើញការកាត់ជើងខ្មែរពិត ដែល check ចាស់មើលមិនឃើញ។
- **Codex ដកចេញវិញនៅជុំ ២:** របារ 3 px មិនមែនជា outline ព័ទ្ធជុំវិញទេ (§06 អនុម័តរួច) ⇒ រក្សាទុក ហើយសួរម្ចាស់ (Q3)។
- **កែការរចនាតាមជុំ ២:** pixel clip check ឥឡូវបែងចែក ink របស់អក្សរនីមួយៗ (គូរ/ថ្លា) និងមាន fixture ដែលដឹងចម្លើយ; ការវាស់ field ហៅថា "heuristic" មិនហៅថា "AA verified" ទេ។
- **ទម្លាក់:** គ្មាន។ ការរកឃើញទាំងអស់មានភស្តុតាង។

### P0 exit council (ក្រោយ G3, 2026-11-09)

ChatGPT (Codex CLI, read-only) ពីរជុំ លើ G3 ពេញ, G2 council, និងផ្នែក ៣, ៥, ៦។ ខ្ញុំជាអ្នកដឹកនាំ ហើយផ្ទៀងផ្ទាត់ជាមួយ `results.md`, `plan.md` និង blueprint §17។

- **ការឯកភាព:** បិទ P0 ជា **ការវាយតម្លៃ** ដែល gate ល្បឿន durable ធ្លាក់; មិនត្រូវការ performance engineering បន្ថែមក្នុង P0 ទេ (plan S5 អនុញ្ញាត fallback "profile ⇒ ប្ដូរ buffer/flush")។ លុះត្រាម្ចាស់អនុម័ត exit.md ក្រោយ G3 (ការប៉ាន់ស្មាន 12.5 សប្ដាហ៍, protocol gate, ការពន្យារ G2 #15)។
- **ទទួលយក និងកែរួច:** កំហុសនព្វន្ធចន្លោះ (11.5 ⇒ 11.3–14.5); បំបែក fallback G3 ជា ៥ ការងារ; protocol gate ល្បឿនបង្កក + checkpoint សប្ដាហ៍ទី ២ ដែលមានលទ្ធផល ៣; R12 រួមលទ្ធភាព "មិនអាចឆ្លង"; kill ×1000 លើ engine ផលិតផល; R14 (integrity គ្មាន hash), R15 (M-08), R16 (ENOSPC/sparse); arm64 = provisional; G2 #15 ពន្យារដោយច្បាស់។
- **Codex ដកចេញវិញនៅជុំ ២:** MSIX −1.0 "ត្រូវការហេតុផល" (§17 P1 មាន MSIX + Store certification ពិត); "+1.0 test ជាការរាប់ពីរដង" ⇒ ហៅថា allowance ដែលដាក់ឱ្យច្បាស់ថ្មី។
- **ទម្លាក់:** បំបែក 8 សប្ដាហ៍ដើម (blueprint មានតែចំនួនសរុប) ⇒ estimate ជា adjustments-only ហើយ conditional; Codex យល់ព្រម។
- **Codex ខុស/មិនបានឃើញ:** "DPI មិនទាន់មាន" (G2 #14 PASS 100–175%, `2c955eb`); Codex មិនបានឃើញបញ្ជី ៣២ ជួរ (មិនបានផ្ញើ) ⇒ ពិនិត្យរួចក្នុង council G4 មុន។

**ស្ថានភាព P0: បិទ (2026-11-09)។** ម្ចាស់អនុម័តកំណែក្រោយ G3 (`9865f78`): ការប៉ាន់ស្មាន ≈12.5 សប្ដាហ៍, protocol gate ល្បឿន, ការពន្យារ G2 #15។
