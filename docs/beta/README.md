# GDM v1.0 beta: ឧបករណ៍សម្រាប់អ្នកសាកល្បង / Tester kit

**គោលដៅ (§17 P2 exit):** អ្នកសាកល្បង ១០ នាក់ (ខ្មែរយ៉ាងតិច ៥) ប្រើ GDM ជំនួស IDM រយៈពេល ២ សប្ដាហ៍; ≥ 8/10 ឆ្លើយថា "GDM ជំនួស IDM បាន" ឬរបាយការណ៍ចន្លោះ (gap list) ទៅម្ចាស់។ ចម្លើយកត់ក្នុង `docs/beta/answers.md` (ម្នាក់មួយជួរ) ដោយគ្មានឈ្មោះពិត ឬ email (លេខ T01–T10 តែប៉ុណ្ណោះ)។

## ១. ការដំឡើង
1. ដំឡើង `Galaxy Download Manager_<version>_x64-setup.exe` (per-user, គ្មាន admin)។
2. Extension: Chrome/Edge › `chrome://extensions` › Developer mode › Load unpacked › `extension/dist/chrome` (រហូតដល់ store អនុម័ត)។
3. IDM: កុំលុប; បិទ browser integration របស់ IDM ក្នុងរយៈពេល beta (ដើម្បីកុំឱ្យចាប់ពីរ)។

## ២. កិច្ចការដែលត្រូវសាក (យ៉ាងតិចម្ដង)
| # | កិច្ចការ | ជួរ §04 |
|---|---|---|
| 1 | ទាញឯកសារធំពី browser (zip/iso/exe), ផ្អាក, បិទ GDM, បើកវិញ, បន្ត | F02/F03 |
| 2 | Download later ⇒ queue; កំណត់ម៉ោងចាប់ផ្ដើមយប់ + បិទម៉ាស៊ីនពេលបញ្ចប់ (សាក [បោះបង់]) | F08 |
| 3 | Sync queue លើឯកសារដែលផ្លាស់ប្ដូរ | F08 |
| 4 | Link ផុតកំណត់ ⇒ Refresh address | F06 |
| 5 | Add batch download (`img*.jpg` 1–20) | F10 |
| 6 | Import បញ្ជី `.ef2` ពី IDM (IDM › Tasks › Export) | F10 |
| 7 | `gdm.exe /d <URL> /n` ពី command prompt | F10 |
| 8 | Clipboard monitor: ចម្លង link ឯកសារ | F10 |
| 9 | Speed limit + quota; connection per server | F11 |
| 10 | Proxy (បើមាន): system ឬ manual | F05 |
| 11 | Site login (folder ដែលត្រូវការ password) | F16 |
| 12 | វីដេអូ: ដំឡើង component (យល់ព្រម) ⇒ បិទភ្ជាប់ទំព័រវីដេអូ ⇒ ជ្រើសគុណភាព | F07 |
| 13 | អូស link ពី browser ទម្លាក់លើផ្ទាំង GDM | §04 |
| 14 | ភាសាខ្មែរ: អក្សរកាត់/ពិបាកអាន? ពាក្យមិនច្បាស់? | F14 |

## ៣. សំណួរ (ចុងសប្ដាហ៍ទី ២)
1. **GDM ជំនួស IDM សម្រាប់អ្នកបានទេ?** បាទ/ចាស · ទេ · (ហេតុអ្វី?)
2. មុខងារ IDM ណាដែលអ្នកប្រើ ហើយ GDM ខ្វះ ឬធ្វើបានអាក្រក់ជាង? (gap)
3. ល្បឿនធៀប IDM: លឿនជាង · ស្មើ · យឺតជាង (URL/ទំហំ ប្រសិនបើដឹង)
4. កំហុស/crash/ឯកសារខូច? (ពេលវេលា, link ប្រសិនបើអាចចែករំលែកបាន)
5. ភាសាខ្មែរ: 1–5 និងពាក្យដែលគួរប្ដូរ
6. Windows (10/11), browser, ប្រើ proxy/VPN?

## ៤. ការរក្សាឯកជនភាព
កុំផ្ញើ cookie, ពាក្យសម្ងាត់ ឬ link ដែលមាន token។ ប៊ូតុង "ចម្លង diagnostic" ក្នុង GDM ដក token/query ចេញ (P-02)។

---

## English summary
Goal (§17 P2 exit): 10 testers (≥ 5 Khmer speakers) use GDM instead of IDM for 2 weeks; done when ≥ 8/10 answer "GDM replaces IDM", otherwise the gap list goes to the owner. Install the per-user installer and the unpacked extension, switch IDM's browser integration off during the beta, run the 14 tasks above at least once, then answer the 6 questions. Answers go to `docs/beta/answers.md` as T01–T10 rows without names or e-mail addresses. Never send cookies, passwords or tokenised links.
