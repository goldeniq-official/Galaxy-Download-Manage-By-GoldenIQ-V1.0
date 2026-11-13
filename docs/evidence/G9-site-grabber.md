# G9 (P3): Site grabber (F09)

**ស្ថានភាព:** រួច (2026-11-13) — កាត G-01..G-05 ឆ្លងក្នុង test ដែល CI ដំណើរការ (core.yml `cargo test --workspace`, app.yml `pnpm test:ui`); ឯកសារខ្មែរ + អង់គ្លេស `docs/site-grabber.md` + README។ CI មិនទាន់រត់ ព្រោះម្ចាស់មិនទាន់ push (ច្បាប់ ២)។

## ១. លក្ខខណ្ឌមុន

- v1.0 evidence: `docs/evidence/G7.md` §៩ "Done-condition" (P2 26 ជួរ, council GO, speed gate = waiver ជាលក្ខខណ្ឌ release) ⇒ មាន ⇒ បន្ត។
- អានមុន: §17 P3 (`content.py:412`: "កាត F09 ឆ្លង; grabber ឆ្លង trap fixtures"), កាត §12 F09 (`scenarios.py` G-01..G-05), §04 ជួរ Grabber ២ (`parity.py:45-46`), IDM §1.4 (`docs/research/idm.md`), §10 W-01/W-02/X-01។

## ២. ជ្រើសមុខងារ

Site grabber (មើល `docs/decisions.md` 2026-11-13 G9)។ មូលហេតុ: មុខងារ P3 តែមួយដែលមានកាតផ្ទាល់ និងជាលក្ខខណ្ឌ exit P3។

## ៣. អ្វីដែលបានធ្វើ (commits លើ main)

| Commit | មាតិកា |
|---|---|
| `27aa470` | `crates/gdm-core/src/grab.rs` (crawl មានព្រំដែន, robots RFC 9309, ទំព័រជៀស, W-02, scanner link, mirror); `net::client_manual` (គ្មាន auto-redirect); `fsx::Dir::subdir` (បង្កើតថតតាម handle, បដិសេធ junction, កាន់គ្មាន SHARE_DELETE); fixture `/cal`, `/loop/`, `/robots.txt`; test `crates/gdm-core/tests/g9_grabber.rs` |
| `6dee1d0` | Tauri: `grab_run`, `grab_progress`, `grab_cancel`, `grab_pick_dir`, `grab_add` (+ permission/capability); ការកែ security review ២ ចំណុច |
| `d4758bf` | UI `src/Grabber.tsx` (wizard ៤ ជំហាន + លទ្ធផល), ម៉ឺនុយ កិច្ចការ ▾, api/mock/store, អត្ថបទ km/en, `tests/grabber.spec.ts` |
| (commit បិទ) | ការកែតាម council ចុងក្រោយ (redirect history/ច្បាប់ path/ជម្រៅគេហទំព័រផ្សេង, robots ដាច់ ⇒ unreachable, robots ក្នុងបញ្ជីជៀស, ថតលំនាំដើមតាម handle, stop "bytes") + test ៣; `docs/site-grabber.md` (ខ្មែរ + English), README, decisions, evidence នេះ, រូប `docs/evidence/g9-shots/` |

គ្មាន dependency ថ្មី (Cargo.lock / pnpm-lock មិនប្ដូរ)។

## ៤. កាត ⇒ test (គ្រប់ test ឆ្លង)

| កាត | Test (`crates/gdm-core/tests/g9_grabber.rs`) | អ្វីដែលបញ្ជាក់ |
|---|---|---|
| G-01 trap | `g01_robots_cut_short_means_disallowed`, `g01_redirects_meet_path_filters_and_other_site_depth`, `g01_calendar_query_trap_stops_at_query_variant_cap`, `g01_page_budget_stops_the_crawl`, `g01_path_loop_guard`, `g01_depth_time_and_byte_budgets`, `g01_per_host_spacing_and_cancel`, `g01_hard_ceilings`, `g01_robots_disallow_is_never_requested`, `g01_robots_rfc9309_matching` | ប្រតិទិនគ្មានទីបញ្ចប់ ឈប់ត្រង់ query-variant / pages; `/loop/a/a/` ឈប់មុន segment ដដែល ៣ ដង; ជម្រៅ, បៃ, ពេល ឈប់ដោយមានមូលហេតុ; ≥ 240 ms រវាងសំណើទៅ host ដដែល (robots ក៏រាប់); Stop ក្នុងពេលរង់ចាំ < 3 s; ទំព័រ ≤ 5 000 ទោះវាយ 1 000 000; robots Disallow មិនដែលស្នើ; RFC 9309 longest-match/Allow tie/`*`/`$`/group merge/percent |
| G-02 ទំព័រជៀស | `g02_avoided_robots_txt_is_never_requested`, `g02_logged_in_crawl_never_requests_avoided_pages`, `g02_redirect_into_an_avoided_page_is_not_followed`, `g02_robots_redirect_into_an_avoided_page_is_not_followed`, `g02_avoid_matching` | crawl មាន login: `/logout`, `/LogOut`, `%6Cogout`, `delete`, `unsubscribe`, redirect ទៅ `/logout`, robots redirect ទៅ `/logout` ⇒ ០ សំណើ; គ្រប់សំណើជា GET; form មិនដែលតាម |
| G-03 traversal | `g03_mirror_segments_table`, `g03_mirror_folders_are_held_and_junctions_refused`, `g03_results_become_paused_jobs_in_mirror_folders` | `%2e%2e`, separator encoded, NUL, CON, `c:\`, ចំណុចចុង, decode ម្ដង, port, ខ្មែរ, ប្រវែង; junction ក្នុងដើមឈើ ⇒ បដិសេធ ហើយគ្មានអ្វីត្រូវបង្កើតខាងក្រៅ; ថតកាន់ ⇒ rename មិនបាន; ឯកសារក្លាយជា job ផ្អាកក្នុងថត mirror |
| G-04 SSRF | `g04_private_and_foreign_links_are_skipped`, `g04_untyped_private_start_is_refused`, `g04_credentials_stay_on_their_origin` | 192.168.1.1, [fd00::1], 10.0.0.1, redirect ទៅ 10.0.0.2, host ផ្សេង ⇒ រំលង ហើយមិនដែលទាក់ទង; domain filter អនុញ្ញាតឱ្យរាយ; start private មិនបានវាយ ⇒ មិនតភ្ជាប់; Authorization មិនទៅ origin ផ្សេង (port ផ្សេង) |
| G-05 JS | `g05_static_js_scan_never_executes`, `g05_malformed_markup_does_not_panic`, `filters` | តែ literal (`https://…/clip.mp4`, `/files/doc.pdf`); string គណនា/template/escape/eval មិនតាម; JS បិទ ⇒ មិនស្កេន script; markup ខូច/ខ្មែរបន្ទាប់ `&` មិន panic |

### លទ្ធផល

- `cargo test -p gdm-core --test g9_grabber` ⇒ **24 passed** (log: `docs/evidence/g9-logs/cargo-g9.txt`)។
- `cargo test -p gdm-core` (ទាំងអស់, ក្រោយការកែ council) ⇒ គ្រប់ suite ok (kill/hostile/network/... មិនខូច; `g9-logs/cargo-core-all.txt`)។
- `cargo test -p galaxy-download-manager --lib` ⇒ 30 passed។
- `cargo clippy -p gdm-core -p gdm-fixture -p galaxy-download-manager --all-targets -- -D warnings` ⇒ ស្អាត; `cargo fmt --check` ⇒ ស្អាត។
- UI: `npx tsc --noEmit` និង `eslint src tests` ⇒ ស្អាត (លើ worktree HEAD ស្អាត); `playwright test tests/grabber.spec.ts` ⇒ **22 passed** លើ worktree HEAD ស្អាត (`g9-logs/playwright-g9.txt`; ជំហាន ១–៤ + លទ្ធផល × km/en × dark/light: ច្បាប់ ៨ គ្មាន border/outline/glow, អក្សរខ្មែរមិនកាត់, axe គ្មាន serious/critical, focus មើលឃើញដោយ fill; ឥរិយាបថ: តែឯកសារធីក, លាក់ស្ទួន, គ្មាន path/header ពី WebView, URL មិនមែន http មិនទៅមុខ)។ រូប: `docs/evidence/g9-shots/*-km-*.png`។
- Versions: rustc/cargo 1.97.1, node 24.22.1, Playwright 1.65.0 (Chromium 1228)។

### បរាជ័យដែលបានជួប (និងការកែ)

- test ប្រតិទិនដំបូងបញ្ចប់ "done" ព្រោះជម្រៅ 3 ឈប់មុន (ពិតជា guard មួយ) ⇒ test កំណត់ជម្រៅ 20 ហើយ query variant 10 ដើម្បីបញ្ជាក់ guard នោះផ្ទាល់។
- G-03 "`..` ⇒ `_..`": WHATWG URL ដោះ `%2e%2e` រួចហើយ, ហើយ Windows កាត់ចំណុចចុង ⇒ ប្ដូរទៅ `_dot`/`_dotdot` (decisions)។
- Security review (plugin) លើ `27aa470`: (1) slice `rest[..12]` អាចកាត់ char ខ្មែរ ⇒ panic ⇒ ស្កេនជាបៃ + test; (2) robots.txt redirect មិនឆ្លងការពិនិត្យទំព័រជៀស/scope ⇒ បន្ថែម + fixture `robots.redirect` + test។
- Playwright: ប៊ូតុង `aria-disabled` មិនអាចចុច (រង់ចាំរហូត timeout) ⇒ test ពិនិត្យ attribute ជំនួស។
- IDM 6.43 កំពុងរត់: ក្រោយ Playwright ពិនិត្យ window IDMan ⇒ គ្មានផ្ទាំងលោតសល់។

## ៥. Council (ច្បាប់ ៦)

- **ផែនការ r1** (`codex exec -s read-only`): Codex ស្នើ module ៦, dependency `scraper`/`regex`/robots crate, persistence project, concurrency 4, redirect ដោយដៃ, credential តាម origin, typed តែ host ដែលវាយ, `.. ⇒ _..` ជាមួយ handle តាមកម្រិត, ការធ្វើ test ច្បាស់។
- **r2 (ជជែក)**: Codex ទទួល file មួយ, គ្មាន dependency, crawl តាមលំដាប់, Basic login, ការកាត់វិសាលភាព; លក្ខខណ្ឌ: កាន់ handle ថតមេរហូត add_many (+ test junction), robots group merge/`*` fallback/percent + bootstrap redirect, ការប្រើបៃមានព្រំដែន (មិនអះអាង "មិនដែលទាញ"), test collision + `<base>` + `srcset` មានក្បៀស ⇒ **ធ្វើទាំងអស់**។
- ការសម្រេច (lead): scanner/robots សរសេរខ្លួនឯង (គ្មាន supply-chain ថ្មី), parser ពិនិត្យដោយ table/fixture test; ទម្លាក់: persistence project + schedule/sync, convert links, size filter, JS render (ផ្លាស់ទៅក្រោយ, កត់ក្នុង decisions)។
- **ពិនិត្យចុងក្រោយ**: មើល §៧។

## ៦. ការកាត់ (Algorithm ជំហាន ២) — ចន្លោះ parity ដែលនៅសល់

ជួរ §04 "Wizard 4 ជំហាន, template, depth, filter, ទំហំ, ស្ទួន, link local, project, sync": **រួច** wizard, template, depth, filter (domain/path/type), ស្ទួន + trap limits + robots.txt។ **ផ្លាស់ទៅក្រោយ**: ទំហំ, link local, project, sync។ ជួរ "Login + ទំព័រជៀស, JS parsing": **រួច** login (Basic), ទំព័រជៀស, JS static; **ផ្លាស់ទៅក្រោយ**: render JS ក្នុង WebView ដាច់, login តាម browser/cookie។ Trigger និងមូលហេតុ: `docs/decisions.md`។

## ៧. Council ពិនិត្យចុងក្រោយ

- **r1: NO-GO** (Codex, ៧ ចំណុច): (1) https→http លើ hop ទី ១ (history មិនមាន URL បច្ចុប្បន្ន); (2) robots.txt ដាច់ពាក់កណ្ដាល ⇒ អនុញ្ញាតទាំងអស់; (3) robots.txt ស្នើទោះនៅក្នុងបញ្ជីជៀស; (4) redirect រំលង no-parent/include/exclude; (5) redirect ទៅគេហទំព័រផ្សេងរំលង other_depth; (6) budget បៃអាចលើសមួយ chunk; (7) `create_dir_all` មុនការពិនិត្យ junction។
- ការសម្រេច (lead, ផ្ទៀងនឹងកូដ): ១–៥ និង ៧ ពិត ⇒ កែទាំងអស់ + test ៣ (`g01_robots_cut_short_means_disallowed`, `g02_avoided_robots_txt_is_never_requested`, `g01_redirects_meet_path_filters_and_other_site_depth`); ៦ = ពិដាន ponytail ដែលកត់ (budget ជាលក្ខខណ្ឌឈប់ មិនមែន cap បៃពិតប្រាកដ; លើសបានយ៉ាងច្រើនមួយ chunk)។
- **r2: GO** — Codex ទទួល ១–៥, ៧ និង ៦ ជាពិដានកត់ទុក។ ចំណុចតូច ២: stop "done" ពេល budget អស់លើទំព័រចុង ⇒ **កែ** ("bytes"); ខ្សែ redirect ក្រៅ→ផ្ទះ→ក្រៅ រក្សាចំនួនដើម ⇒ ឈប់មុនមួយជំហាន (មិនដែលហួស) ⇒ កត់ `ponytail:` ក្នុងកូដ។
- Codex មិនអាចដំណើរការ test (read-only); test ទាំងអស់ដំណើរការដោយ lead (§៤)។

## ៨. របស់ម្ចាស់ (ច្បាប់: សួរតែម្ដងនៅចុង)

- `git push` (CI core.yml + app.yml នឹងដំណើរការ test កាត G-01..G-05 និង UI spec)។ ពាក្យបញ្ជា: `git push origin main`។
- គ្មានអ្វីផ្សេងទៀត: គ្មានលុយ គណនី ឬ LICENSE ក្នុង G9។
