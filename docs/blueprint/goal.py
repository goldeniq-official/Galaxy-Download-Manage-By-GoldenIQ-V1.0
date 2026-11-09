"""§21 income plan, and the separate "goal" tab: copyable /goal commands with done-checkboxes and prerequisites."""
import html as _h
import json as _json

from autonomy import AUTONOMY_EN

MONEY_HTML = r"""
<p><strong>គោលការណ៍ (ម្ចាស់សម្រេច 2026-11-09):</strong> GDM ឥតគិតថ្លៃពិតសម្រាប់គ្រប់គ្នា រួមទាំងក្រុមហ៊ុន សាលា និងស្ថាប័នរដ្ឋ (LICENSE: RPL-1.5, ប្រើដោយសេរីសម្រាប់គ្រប់គ្នា)។ ក្នុង app គ្មានពាណិជ្ជកម្ម គ្មានការតាមដាន គ្មាន activation (R3)។ ចំណូលមកពីអ្វីៗនៅជុំវិញ app មិនមែនពី app ខ្លួនឯងទេ។</p>
<div class="tw"><table><thead><tr><th>#</th><th>ប្រភពចំណូល</th><th>ច្បាប់អនុវត្ត</th><th>ពេលចាប់ផ្ដើម</th></tr></thead><tbody>
<tr><td>M1</td><td><strong>KHQR "គាំទ្រ GDM"</strong> (Bakong, ចូល ABA ផ្ទាល់)</td><td>នៅលើ README, គេហទំព័រ និងទំព័រ About (រូប QR ភ្ជាប់ក្នុង app, គ្មាន network)។ ការបរិច្ចាគជាការស្ម័គ្រចិត្ត មិនផ្ដល់មុខងារបន្ថែម ឬអាទិភាពទេ។ ផ្ទៀងផ្ទាត់លក្ខខណ្ឌគណនី ABA បង្កាន់ដៃ និងពន្ធ។</td><td>ឥឡូវ (ម្ចាស់ផ្ដល់រូប KHQR)</td></tr>
<tr><td>M2</td><td><strong>សេវាបង់ថ្លៃសម្រាប់ស្ថាប័ន</strong>: ដំឡើងច្រើនគ្រឿង, កំណត់ setting, បង្ហាត់, support, មុខងារតាមសំណូមពរ</td><td>App នៅតែឥតគិតថ្លៃ។ គិតថ្លៃតែពេលវេលា និងជំនាញ។ មុខងារតាមសំណូមពរត្រូវមានកិច្ចសន្យាជាលាយលក្ខណ៍អក្សរ: កម្មសិទ្ធិ (Golden IQ & CHHIT Sungchhay), ការបោះពុម្ពចូល product ផ្លូវការ (មិនសន្យាគ្រប់ករណី), លក្ខខណ្ឌទទួល, ការថែទាំ។ គ្មាន fork ឯកជន។</td><td>ពេល v0.9 beta</td></tr>
<tr><td>M3</td><td><strong>អ្នកឧបត្ថម្ភ</strong> (ISP, ក្រុមហ៊ុនទូរស័ព្ទ, hosting, ធនាគារ)</td><td>តែលើគេហទំព័រ និង README ប៉ុណ្ណោះ <strong>មិនមែនក្នុង app</strong> (logo បង់លុយក្នុង app = ពាណិជ្ជកម្ម)។ ដាក់ស្លាក "ឧបត្ថម្ភ"។ កិច្ចសន្យាហាមអ្នកឧបត្ថម្ភមិនឱ្យមានឥទ្ធិពលលើ default, ការទាញយក, លទ្ធផលស្វែងរក ឬទិន្នន័យអ្នកប្រើ។</td><td>ពេលមានអ្នកប្រើច្រើន</td></tr>
<tr><td>M4</td><td><strong>សេវាដំឡើងសម្រាប់ហាង PC / OEM</strong> (ការដំឡើង, image, support)</td><td><strong>មិនលក់អាជ្ញាប័ណ្ណ</strong>: SignPath ហាម commercial dual-licensing ហើយ RPL-1.5 អនុញ្ញាតការចែកចាយកំណែដើមដោយសេរីរួចហើយ។ គំរូអាជីវកម្មដែលជ្រើស: គិតថ្លៃលើសេវាដំឡើង និង support (RPL §6.5 ក៏កំណត់កាតព្វកិច្ច indemnity ពេលផ្ដល់ warranty/support)។</td><td>ពេលមានសំណើ</td></tr>
<tr><td>M5</td><td><strong>YouTube ខ្មែរ</strong> (មេរៀនប្រើ download manager)</td><td>ជាការផ្សព្វផ្សាយមុន ចំណូលក្រោយ។ ហាមវីដេអូពីការទាញវីដេអូ YouTube/streaming ឬរំលងការការពារ។ ការប្រៀបធៀបល្បឿនត្រូវមានការវាស់ (§09)។</td><td>ពេល v1.0</td></tr>
</tbody></table></div>
<h3>អ្វីដែលបោះបង់ ឬពន្យារ</h3>
<ul>
<li><strong>បោះបង់: ពាណិជ្ជកម្មក្នុង app, affiliate/VPN, bundleware</strong>។ បំផ្លាញទំនុកចិត្ត ហើយ antivirus អាចចាត់ GDM ជា adware។ Google AdSense/AdMob មិនគាំទ្រ app កុំព្យូទ័រទេ។</li>
<li><strong>បោះបង់: AdSense លើគេហទំព័រ GDM</strong>។ Google Publisher Policies ("Enabling dishonest behavior") ហាម "pages that assist or enable users to download streaming videos if prohibited by the content provider" ហើយអាចផ្អាកគណនីបាន (ផ្ទៀងផ្ទាត់លើទំព័រផ្ទាល់ <a href="https://support.google.com/adsense/answer/10502938">support.google.com/adsense/answer/10502938</a>, 2026-11-09)។ GDM មាន yt-dlp opt-in (D10) ដូច្នេះហានិភ័យខ្ពស់។ ពិចារណាឡើងវិញតែក្រោយអាន policy ថ្មី។</li>
<li><strong>ពន្យារ: Payoneer សម្រាប់អ្នកបរទេស</strong>។ ផ្លូវដកចូល ABA មានតែប្រភពភាគីទីបី ហើយមិនទាន់ដឹងថាវាទទួលការបរិច្ចាគបានទេ។ PayPal នៅកម្ពុជាទទួល/ដកលុយមិនបាន។</li>
<li><strong>ពន្យារ: "Supporter pack" ក្នុង Microsoft Store</strong> (theme ប៉ុណ្ណោះ, មុខងារនៅឥតគិតថ្លៃ)។ មិនទាន់ឃើញឯកសារផ្លូវការថា Partner Center ផ្ញើលុយមកកម្ពុជា។</li>
</ul>
<h3>កាតព្វកិច្ចដែលងាយភ្លេច</h3>
<ul>
<li>ថវិកាប្រចាំឆ្នាំ: domain, hosting, Chrome Web Store $5, certificate (D8), ថ្លៃទទួលលុយពីបរទេស (ABA)។</li>
<li>គេហទំព័រ និងការទូទាត់ប្រមូលទិន្នន័យខ្លះ (log, ធនាគារ) ⇒ <code>PRIVACY.md</code> ត្រូវសរសេរឱ្យត្រូវ ទោះ app គ្មាន telemetry។</li>
<li>ពន្ធ ការចុះបញ្ជីអាជីវកម្ម និង invoice: សួរគណនេយ្យករកម្ពុជា (ពាក់ព័ន្ធ D17)។</li>
<li>គណនី domain, Store, extension, signing key: បើក MFA, មានផ្លូវ recover, និងអ្នកជំនួសពេលអាសន្ន។</li>
</ul>
<p class="note"><strong>Council (2 ជុំ, 2026-11-09):</strong> ChatGPT បានបន្ថែម: logo ឧបត្ថម្ភក្នុង app គឺជាពាណិជ្ជកម្ម; LICENSE ត្រូវមានសិទ្ធិជាងដំឡើង (§2(e)) និងប្រយោគ "Personal/Internal Use ឥតគិតថ្លៃជានិច្ច" (§6); មុខងារតាមសំណូមពរត្រូវមានកិច្ចសន្យា។ វាបានបង្ហាញ URL policy AdSense ផ្ទាល់ ដែល Claude ផ្ទៀងផ្ទាត់រួច។ ការអះអាងថា Payoneer ទទួលការបរិច្ចាគបាន មិនទាន់ផ្ទៀងផ្ទាត់ ⇒ ពន្យារ។ ផ្ទាំង <strong>goal</strong> (ខាងលើទំព័រ) មានពាក្យបញ្ជា /goal ទាំងអស់។</p>
"""

# Shared header for every command. Project rules in CLAUDE.md (no push, authorship, Khmer replies, no glow/outline,
# The Algorithm, council) load by themselves, so they are referenced, not repeated.
HEAD = ("Project: Galaxy Download Manager (GDM), repo root = this folder. Source of truth: docs/GDM-Blueprint.html "
        "(generated from docs/blueprint/*.py) + LICENSE. Follow CLAUDE.md rules. Evidence goes in docs/p0/ (P0) or "
        "docs/evidence/<goal>.md (later goals): commands run, versions, results, failures. Never mark done without it.\n\n" + AUTONOMY_EN)

# (id, phase, Khmer title, Khmer summary, prerequisites, can run in parallel with, command body)
GOALS = [
    ("G1", "P0", "បញ្ជីសេចក្ដីសម្រេច និងផែនការ spike",
     "បង្កើត docs/p0/plan.md: សេចក្ដីសម្រេច D1–D18 ដែលម្ចាស់ឆ្លើយពិត, spike ៩ មាន pass/fallback, ម្ចាស់ update តាមប្រភេទ install, protocol វាស់ល្បឿន, រៀបឈ្មោះ/identifier តាម D11/D12។",
     [], [],
     """GOAL G1 (P0-A): decision register and P0 spike plan.
Read first: §01 requirements R1-R11, §07 update, §08 architecture + ADR-1..6, §09 engine targets, §15 release, §17 plan, §19 decisions D1-D18.
Do:
1. Prerequisite check: owner decisions D1-D20 are recorded in §19 (all answered 2026-11-09); do not re-ask.
2. Write docs/p0/plan.md (Khmer, technical terms in English):
   a. Decision register D1-D18: question, owner's answer, date, which work it unblocks.
   b. Spike inventory, in the §17 risk order: S1 MSIX + native messaging host visible to Chrome/Edge/Firefox; S2 MV3 prepared handoff; S3 named pipe with DACL + server verification; S4 hostile HTTP fixture server; S5 segmenter + write lease + speed benchmark; S6 MOTW via IAttachmentExecute + exFAT/FAT targets; S7 yt-dlp policy proxy (ADR-6); S8 update install + recover (U-04, U-06); S9 Khmer.nsh in NSIS. For each: deliverable, pass criterion (measurable), fallback/plan B, estimate, risk, related cards (§12).
   c. Update ownership per install type: Store MSIX = Microsoft Store updates, in-app updater disabled; NSIS/GitHub = Tauri updater + minisign; components (yt-dlp/ffmpeg) = signed components.json for both.
   d. Owner's answer: is unsigned GitHub NSIS acceptable as the public fallback if Store fails (D8)?
   e. Benchmark protocol vs IDM 6.43: fixed machine, wired link, bandwidth recorded, cache cleared, same 5 public URLs on different CDNs + 1 GB LAN fixture, 5 runs each, report median and worst case, connection counts logged. Targets from §09: >=90% line rate on LAN, CPU <10% (4 cores), engine RAM <60 MB, mean >= IDM -5% and equal/faster on >=3/5 URLs.
   f. Repo layout proposal: Cargo workspace (crates/gdm-core, crates/gdm-bridge, src-tauri app), extension/, fixtures/, spikes/ (throwaway, never shipped), docs/.
3. Only if the owner confirmed D11/D12: align identifiers (package.json name, Cargo package name, tauri.conf.json productName "Galaxy Download Manager", identifier per D12) in one small commit. Otherwise list them as pending.
Done when: docs/p0/plan.md is committed on main and council has reviewed it (record what Codex added/got wrong)."""),

    ("G2", "P0", "Spike ការចែកចាយ Windows និង browser",
     "MSIX + native messaging ឱ្យ browser ទាំង ៣ ឃើញ, install/upgrade/uninstall លើ Win10/11, component ទាញក្រោយ install, NSIS + Khmer.nsh ជា plan B។",
     ["G1"], ["G3"],
     """GOAL G2 (P0-B): prove Windows distribution and browser integration (spikes S1, S9 + plan B).
Read first: docs/p0/plan.md (spike S1, S9 rows), §08 (gdm-bridge.exe, installer row), §15 (Store, certificates, release.yml), cards F01 (B-01 B-02 B-03 B-04 B-09), F13 (U-06), F15 (T-01), F17 (S-01 S-02 S-03).
Prerequisite check: docs/p0/plan.md exists; if not, stop and say G1 is not done.
Do (throwaway code under spikes/distribution/, never shipped):
1. Minimal gdm-bridge native messaging host (stdin/stdout framing, origin allow-list, size limit, stateless) + minimal MV3 extension that calls it.
2. Package as MSIX. Make the native messaging host registration visible to Chrome, Edge and Firefox: try desktop:Extension / unvirtualized registry first; if that fails, try a separate per-user installer helper. Record exactly which manifest/registry keys each browser reads.
3. On Windows 10 and Windows 11 (record exact builds): install, launch from browser, upgrade to a higher version, uninstall; confirm no orphan registry keys or files (T-01).
4. Download a component (yt-dlp) after install into %LOCALAPPDATA%\\GDM\\components with sha256 check from a signed manifest; confirm MSIX does not virtualize it in a way that breaks execution.
5. Plan B: NSIS per-user installer with Khmer.nsh (Khmer text renders, no clipping), same browser checks.
6. If the owner has a Store account (D9): submit a private/hidden test package and record certification feedback. Label every result clearly as LOCAL VALIDATION or REAL STORE CERTIFICATION.
Record in docs/p0/results.md (section G2): steps, versions, screenshots paths, pass/fail per check, chosen route, update owner per route.
Done when: every distribution check has a recorded result (success or tested plan B), the chosen route is written down, and council has reviewed the results."""),

    ("G3", "P0", "Spike ភាពត្រឹមត្រូវ engine និងសុវត្ថិភាព",
     "Hostile fixtures, segmenter + lease, MOTW, named pipe DACL, MV3 handoff, yt-dlp policy proxy, update recover, វាស់ល្បឿនធៀប IDM។",
     ["G1"], ["G2"],
     """GOAL G3 (P0-C): prove engine correctness and security boundaries (spikes S2-S8).
Read first: docs/p0/plan.md, §09 engine (probe, dynamic segmentation, write lease, seam check, flush/commit, limiter, finalisation, server-response table), §10 threat model + the 12 invariants, ADR-2/3/6, cards F03 (E-01..E-04), F04 (N-01..N-05, N-09), F05 (W-01..W-03), F01 (B-02), F07 (M-01..M-04, M-08), F13 (U-01..U-03, U-06).
Prerequisite check: docs/p0/plan.md exists; if not, stop.
Do (throwaway code under spikes/engine/, each spike leaves ONE runnable check):
1. S4 hostile HTTP fixture server (Rust, reusable later in fixtures/): 200 ignoring Range, ETag change mid-download, infinite body, slowloris, cross-origin redirect (credentials must be stripped), 429 + Retry-After, text/html instead of binary, multipart/byteranges.
2. S5 segmenter: probe GET Range bytes=0- with Accept-Encoding identity; start 2 connections; add one every 2 s only if total speed rises >=10% (default cap 8, max 32); split largest remaining half (>=1 MiB); single actor owns the segment map; write leases with generation + boundary ack; .part preallocated with set_len, 1 MiB buffers, commit offset only after flush (8 MiB or 2 s).
3. S6 finalisation: byte count = total, optional SHA-256, IAttachmentExecute from Rust (MOTW + AV), blocked => "ត្រូវបានរារាំង", untaggable target (exFAT/FAT) => "គោលដៅមិនការពារ", rename by handle without replace.
4. S3 named pipe server with explicit DACL (current user only) + client verifies the server process; reject squatted pipe.
5. S2 MV3 prepared handoff: extension passes URL + needed cookies once, never stored; host validates origin and size.
6. S7 yt-dlp policy proxy: yt-dlp -J extract only, through a loopback proxy with random port and per-run Proxy-Authorization; GDM engine downloads streams; ffmpeg mux only with -protocol_whitelist file,pipe.
7. S8 update install + recover: staged update, verify sha256 + minisign, refuse downgrade, kill during install => old version still starts (U-04, U-06).
8. Run the benchmark protocol from plan.md vs IDM 6.43.
Record in docs/p0/results.md (section G3): each spike's check command, output, pass/fail, fallback.
Done when: all spikes S2-S8 have repeatable results or a documented fallback, the benchmark report exists, and council has reviewed them."""),

    ("G4", "P0", "Design + baseline MVP (បិទ P0)",
     "Design tokens + ផ្ទាំង ៥ ជាខ្មែរ/អង់គ្លេស ពន្លឺ/ងងឹត, focus ដោយ fill (D6), បង្កក scope P1, បញ្ជី licence dependency, ប៉ាន់ស្មានថ្មី។",
     ["G2", "G3"], [],
     """GOAL G4 (P0-D): approve the MVP design and delivery baseline; this closes P0.
Read first: §05 UX + personas, §06 design system, §04 parity ledger (P1 rows), §14 rules, ADR-5 (licence policy for dependencies), docs/p0/results.md.
Prerequisite check: G2 and G3 sections in docs/p0/results.md are complete and council-reviewed; if not, stop.
Do:
1. Design tokens (CSS variables) and the 5 screen mockups from §06 (main list, download info, details, settings, onboarding) in Khmer and English, light and dark. Khmer font bundled, no clipping at 100/125/150% scaling.
2. Owner design rules: no glow and no outline/border/ring anywhere; keyboard focus shown only by fill/colour/brightness change (D6). Check focus visibility in both themes and Windows high-contrast.
3. Freeze the P1 scope: list every P1 row of §04 with its acceptance test; supported Windows versions and architectures (x64 + arm64 per §15).
4. Dependency and licence inventory: add deny.toml (cargo-deny) allowing MIT/Apache-2.0/BSD/ISC/Zlib/Unicode/OFL + unmodified MPL-2.0 (D18), denying linked GPL/AGPL/LGPL/SSPL; npm licence check; plan THIRD-PARTY-NOTICES generation (cargo about + license-checker); record yt-dlp/ffmpeg (BtbN win64-lgpl) as downloaded-at-runtime, not bundled.
5. Revise the P1 estimate from spike evidence.
Write docs/p0/exit.md: mockup paths, frozen P1 acceptance list, dependency inventory result, revised estimate, open risks.
Done when: the mockups and docs/p0/exit.md are committed and the P0 exit council review is recorded (no owner approval gate; decide by the DECISION POLICY)."""),

    ("G5", "P1", "Core ទាញយក HTTP",
     "crates/gdm-core: engine HTTP/HTTPS (segment, resume, integrity, limiter), .part/rename/MOTW, SQLite + crash recovery។ Test មុនកូដ។",
     ["G4"], [],
     """GOAL G5 (P1-A): deliver the persistent HTTP download core.
Read first: docs/p0/exit.md (frozen scope), §08 stack and ADR-1/3, §09 engine, §10 invariants, cards F03, F04, F05, F11, F12 (all critical/high: E-01..E-04, N-01..N-05, N-09, W-01..W-03, D-01) and their test columns in §12.
Prerequisite check: docs/p0/exit.md exists and the P0 exit council review is recorded; if not, stop.
Do:
1. crates/gdm-core as a library independent of Tauri (so it can become gdm-engine.exe later, ADR-1): tokio, reqwest (hyper 1, HTTP/1.1 + HTTP/2), rustls + rustls-platform-verifier, url, rusqlite (bundled, WAL), keyring, windows-rs. No TLS bypass, no https->http, no credentials across origins.
2. Port the proven spike designs (not the throwaway code as-is): probe, dynamic segmentation, write leases, server-response table from §09, 3-level token-bucket limiter (global -> queue -> job), finalisation with IAttachmentExecute, .part + sidecar name.part.gdm (<=64 KB, untrusted) per ADR-3.
3. SQLite schema (jobs, segments, history) with migrations and daily backup; progress recorded <= flushed data.
4. Tests first for every critical/high card above; move the hostile fixture server into fixtures/ and run it in CI.
5. Crash test: 1000 cycles of kill -9 during download -> 0 corrupt or silently wrong files.
6. Throughput test against the §09 targets using the agreed protocol.
Evidence in docs/evidence/G5.md.
Done when: all listed card tests pass in CI, hostile fixtures pass, 1000 kill cycles give 0 corrupt files, targets are measured (met or explained), and there is no open critical integrity defect."""),

    ("G6", "P1", "Beta v0.9 ពេញលេញ",
     "UI React, extension Chrome/Edge/Firefox, gdm-bridge, tray/toast, ស្វែងរក, ខ្មែរ/អង់គ្លេស, updater, NSIS + MSIX, CI ពេញ។",
     ["G5"], [],
     """GOAL G6 (P1-B): deliver the integrated v0.9 public beta (not published by you).
Read first: docs/p0/exit.md, §05 flows, §06 design, §07 update state machine, §08 (UI, gdm.exe, gdm-bridge, extension, installer rows), §15 CI list, cards F01, F02, F12, F13, F14, F15, F16, F17.
Prerequisite check: docs/evidence/G5.md shows G5 done; if not, stop.
Do:
1. UI: React 19 + TypeScript + Vite, i18next (km first, en), TanStack Virtual for big lists, Zustand, CSS variables only (no UI kit). Screens: list, details, settings, Download Info, onboarding (D5: page video button off by default, asked here). Tray, toast, categories, FTS search, two themes. No glow, no outline; focus by fill (D6).
2. Tauri v2 with plugins updater, single-instance, notification, autostart, window-state only; never shell/fs/http plugins; minimal capabilities (X-01, X-02); strict CSP.
3. gdm-bridge.exe (~400 lines Rust, no big async runtime) + extension (TypeScript, MV3 Chrome/Edge, Firefox build; WXT or Vite): capture, Alt/Ctrl click, context menu, popup. Chrome build does not run on YouTube (M-07).
4. Updater per §07 state machine for the GitHub build; disabled in the MSIX build (Store updates).
5. Installers: NSIS per-user with Khmer.nsh, MSIX on the route chosen in G2.
6. CI per §15: fmt, clippy -D warnings, test, cargo-deny, cargo-audit, npm audit, ESLint (react/no-danger), grep for danger_accept_invalid, Playwright UI tests in Khmer and English, hostile fixtures, weekly fuzz, OpenSSF Scorecard; actions pinned by SHA, permissions read-all.
7. README (Khmer first, says "not open source"), PRIVACY.md, SECURITY.md, CONTRIBUTING.md, THIRD-PARTY-NOTICES.
Evidence in docs/evidence/G6.md.
Done when: every P1 exit criterion in §17 has evidence (cards above pass; MSIX native messaging works or tested plan B; speed gate per protocol; Khmer screenshots without clipping; axe 0 serious), and draft release notes with known limitations are ready for the owner. The owner pushes and publishes."""),

    ("G7", "P2", "v1.0 parity ជាមួយ IDM",
     "គ្រប់ជួរ P2 ក្នុង §04: queue, scheduler, refresh address, batch, clipboard, CLI, import/export, proxy, វីដេអូ, failure-injection។",
     ["G6"], [],
     """GOAL G7 (P2): deliver the agreed v1.0 IDM-parity scope.
Read first: §04 parity ledger (P2 rows + owner waivers), §02 IDM study, ADR-1, ADR-4, ADR-6, cards F06 (R-01), F07 (M-01..M-04, M-08), F08 (Q-01, Q-02), F10 (I-01, I-02), F11, F16.
Prerequisite check: v0.9 beta evidence in docs/evidence/G6.md; if not, stop.
Do, for every P2 row in §04 (owner waivers already recorded in §19, do not ask again): queues + sync queue + scheduler + post actions (D15: .bat/.cmd hook only set in UI, with argument restriction, never from extension/CLI/import), refresh address protocol, batch, clipboard, IDM CLI flags (gdm.exe /d ... over the same pipe), import .ef2/txt, export JSON, proxy PAC/NTLM, site logins in Credential Manager, per-server limits, quota, sounds, drop basket, video via opt-in yt-dlp/ffmpeg components with quality popup (D10 notice, no DRM), failure-injection tests (crash WebView, crash core) to settle ADR-1.
Then: beta with 10 testers (>=5 Khmer) for 2 weeks; record answers.
Evidence in docs/evidence/G7.md.
Done when: every P2 row is done or has a waiver already recorded in §19/docs/decisions.md, cards F06-F11 pass, THIRD-PARTY-NOTICES and PRIVACY.md are complete and accurate, and >=8/10 testers say GDM replaces IDM (or the gap list is reported to the owner)."""),

    ("G8", "ចំណូល", "KHQR, សេវា, អ្នកឧបត្ថម្ភ (§21)",
     "ដាក់ KHQR ក្នុង README និង About (រូបភ្ជាប់, គ្មាន network), សរសេរ docs/services.md និង docs/sponsors.md។ ធ្វើបានពេលណាក៏បាន ក្រោយម្ចាស់ផ្ដល់រូប KHQR។",
     [], ["G1", "G2", "G3", "G4", "G5", "G6", "G7"],
     """GOAL G8 (income): set up the income plan in §21 without touching the free-for-everyone promise.
Read first: §21 income plan, LICENSE (RPL-1.5) and NOTICE.md, SignPath terms (no commercial dual-licensing), memory/rule: no ads, tracking, activation or payment SDK in the app.
Prerequisite check: the owner has given the KHQR image (the only owner question here; ask once in Khmer if missing, do everything else first).
Do:
1. README: short voluntary "គាំទ្រ GDM / Support GDM" block with the KHQR image, Khmer first, saying support is optional and unlocks nothing.
2. App About page (if the app exists yet): same KHQR as a bundled image, no network call, no tracking; otherwise leave a task note for G6.
3. docs/services.md (Khmer + English): paid deployment, configuration, training, support, commissioned features; written terms on ownership (Golden IQ & CHHIT Sungchhay), inclusion in the official product (not guaranteed), acceptance, maintenance; no private forks.
4. docs/sponsors.md: website/README placements only, labelled "sponsored", sponsors never influence defaults, downloads, search results or user data.
5. PC shop/OEM note: redistribution of unmodified GDM is already allowed by RPL-1.5; offer paid installation/support services only, never paid licences (SignPath bans commercial dual-licensing).
Done when: the files are committed, wording matches "free for everyone" and LICENSE, and council has reviewed them."""),

    ("G9", "P3", "មុខងារ P3 ម្ដងមួយ (គំរូ)",
     "ចម្លង ប្ដូរ <NAME> ទៅមុខងារមួយ: site grabber, zip preview, SFTP, HTTP/3, HLS native, portable, gdm://, Weblate, VPN/rasdial។",
     ["G7"], [],
     """GOAL G9 (P3, one feature per goal): implement the P3 feature <NAME>.
Choose one: site grabber (4-step wizard, F09 G-01 G-02, trap fixtures) | zip preview | SFTP | HTTP/3 (only when reqwest support is stable) | native HLS (plain or AES-128, ADR-4) | portable mode | gdm:// (ADR-2 restricted form: http(s) URL only, opens the Add dialog, no path/cookie/auto-start) | Weblate community translation | VPN/rasdial before queue (D4).
Read first: §17 P3, the feature's cards in §12, §10 invariants.
Prerequisite check: v1.0 evidence in docs/evidence/G7.md; if not, stop.
Do: tests first from the feature's cards; keep the change inside the existing architecture; Khmer + English UI text; no glow/outline.
Evidence in docs/evidence/G9-<NAME>.md.
Done when: its cards pass in CI and the feature is documented in Khmer and English."""),
]

G0_ITEMS = [
    ("g0-d11", "D11/D12: យល់ព្រមឈ្មោះ \"Galaxy Download Manager\" និង package ID GoldenIQ.GalaxyDownloadManager (មុនបង្កើត repo ឬ identifier)"),
    ("g0-d9", "D9/D17: Golden IQ ជានីតិបុគ្គល ឬឈ្មោះពាណិជ្ជកម្ម? គណនី Store បុគ្គល ឬក្រុមហ៊ុន? នរណាផ្ដល់ការអនុញ្ញាតបាន?"),
    ("g0-rest", "D2–D8, D10, D13–D15, D18: ឆ្លើយ \"យល់ព្រម\" ឬ \"ប្ដូរ\" ម្ដងមួយៗ (§19)"),
    ("g0-repo", "បង្កើត repo GitHub galaxy-download-manager ហើយ push ដោយខ្លួនឯង"),
    ("g0-khqr", "ផ្ញើរូប KHQR (សម្រាប់ G8)"),
]


def _chk(key, label):
    return ('<button class="chk" type="button" role="checkbox" aria-checked="false" data-key="%s" aria-label="%s">'
            '<span aria-hidden="true">✓</span></button>' % (key, _h.escape(label)))


def goal_panel():
    g0 = "".join('<li class="g0i">%s<span>%s</span></li>' % (_chk(k, t), _h.escape(t)) for k, t in G0_ITEMS)
    cards = []
    for gid, phase, title, summary, pre, par, body in GOALS:
        cmd = HEAD + "\n\n" + body
        pre_txt = ("ត្រូវការ: " + ", ".join(pre)) if pre else "ចាប់ផ្ដើមបានភ្លាម"
        par_txt = (" · ធ្វើស្របគ្នាបានជាមួយ " + ", ".join(par)) if par and gid != "G8" else (" · ធ្វើស្របគ្នាបានជាមួយ goal ណាក៏បាន" if gid == "G8" else "")
        cards.append(
            '<article class="gc" data-goal="%s" data-pre="%s">'
            '<div class="gh">%s<div class="gt"><span class="gid">%s · %s</span><h3>%s</h3></div><span class="badge"></span></div>'
            '<p>%s</p><p class="gm">%s%s</p>'
            '<details><summary>មើលពាក្យបញ្ជាពេញ</summary><pre id="cmd-%s">%s</pre></details>'
            '<div class="ga"><button class="copy" type="button" data-for="cmd-%s">ចម្លង /goal</button><span class="when"></span></div>'
            '</article>'
            % (gid, " ".join(pre), _chk(gid, "%s រួចហើយ" % gid), gid, _h.escape(phase), _h.escape(title),
               _h.escape(summary), _h.escape(pre_txt), _h.escape(par_txt), gid, _h.escape(cmd), gid))
    return ('<div class="gbar"><div><strong id="gprog">0/%d</strong> goal រួច · '
            '<span id="gnext"></span></div><button class="pill" type="button" id="ghide" aria-pressed="false">លាក់ goal ដែលរួច</button></div>'
            '<p class="note warn"><strong>ពាក្យបញ្ជានីមួយៗមានច្បាប់សម្រេចដោយខ្លួនឯង (§22):</strong> session មិនសួរម្ចាស់ពីការរចនា វិសាលភាព ឬការអនុម័តទេ។ វាចម្លង IDM/NDM ហើយកត់ក្នុង docs/decisions.md។ វាសួរតែរឿងលុយ គណនី/អត្តសញ្ញាណ push និង LICENSE ជាសារតែមួយនៅចុង goal។</p>'
            '<p class="byline">ចុច "ចម្លង /goal" ហើយបិទភ្ជាប់ក្នុង session Claude Code ថ្មី។ ពេលធ្វើរួច ចុចប្រអប់ ✓ ។ '
            'Goal ដែលមិនទាន់ដល់វេន (goal មុនមិនទាន់ ✓) នឹងសួរបញ្ជាក់មុនចម្លង ដើម្បីកុំឱ្យជាន់គ្នា។ ស្ថានភាពរក្សាទុកក្នុង browser នេះ។</p>'
            '<div class="card g0"><h3>G0 · ម្ចាស់ត្រូវធ្វើមុន (មិនមែន /goal)</h3><ul class="g0l">%s</ul></div>'
            '<div class="glist">%s</div>' % (len(GOALS), g0, "".join(cards)))


GOAL_PANEL = goal_panel()
GOAL_COUNT = len(GOALS)

# Fill-only states everywhere: owner rule bans glow and outline.
GOAL_CSS = r"""
.tabs{grid-column:1/-1;display:flex;gap:8px;flex-wrap:wrap;margin-top:-20px}
.tabs button,.pill{font:inherit;font-size:15px;font-weight:600;padding:8px 20px;border-radius:99px;border:0;background:var(--surface);color:var(--muted);cursor:pointer}
.tabs button[aria-selected="true"],.pill[aria-pressed="true"]{background:var(--accent);color:#fff}
.tabs button:hover,.tabs button:focus-visible,.pill:hover,.pill:focus-visible{background:var(--surface2);color:var(--fg);outline:none}
.tabs button[aria-selected="true"]:hover,.tabs button[aria-selected="true"]:focus-visible{background:var(--accent);color:#fff;filter:brightness(1.1)}
.pill{font-size:13px;padding:5px 14px;font-weight:500}
#goalpanel{grid-column:1/-1;display:grid;gap:16px;min-width:0}
body.tab-goal .wrap>nav,body.tab-goal .wrap>main{display:none}
body:not(.tab-goal) #goalpanel{display:none}
.gbar{display:flex;flex-wrap:wrap;gap:12px;align-items:center;justify-content:space-between;background:var(--surface);border-radius:12px;padding:12px 16px}
#gprog{font-size:20px}
.glist{display:grid;grid-template-columns:repeat(auto-fill,minmax(min(100%,380px),1fr));gap:14px}
.gc{background:var(--surface);border-radius:12px;padding:14px 16px;display:grid;gap:8px;align-content:start;min-width:0}
.gc.done{opacity:.55}.gc.done.hide{display:none}
.gc.next{background:color-mix(in srgb,var(--accent) 16%,var(--surface))}
.gh{display:flex;gap:12px;align-items:flex-start}
.gt{flex:1;min-width:0}.gt h3{margin:0;font-size:17px}
.gid{font-family:var(--f-mono);font-size:12px;color:var(--accent)}
.gm{color:var(--muted);font-size:13px}
.badge{font-family:var(--f-mono);font-size:11px;padding:2px 9px;border-radius:99px;white-space:nowrap}
.badge:empty{display:none}
.badge.b-next{background:var(--accent);color:#fff}.badge.b-wait{background:var(--surface2);color:var(--muted)}.badge.b-done{background:var(--ok);color:#fff}
.gc details summary{cursor:pointer;color:var(--accent);font-size:14px}
.gc pre{white-space:pre-wrap;overflow-wrap:anywhere;max-height:420px;overflow:auto;margin-top:8px}
.ga{display:flex;gap:10px;align-items:center;flex-wrap:wrap}
.when{color:var(--muted);font-size:12px}
button.copy{font:inherit;font-size:13px;padding:5px 16px;border-radius:99px;border:0;background:var(--accent-soft);color:var(--fg);cursor:pointer}
button.copy:hover,button.copy:focus-visible{background:var(--accent);color:#fff;outline:none}
.chk{flex:none;width:30px;height:30px;border-radius:8px;border:0;background:var(--surface2);color:transparent;font-size:18px;line-height:30px;padding:0;cursor:pointer}
.chk[aria-checked="true"]{background:var(--ok);color:#fff}
.chk:hover,.chk:focus-visible{background:var(--accent-soft);color:var(--muted);outline:none}
.chk[aria-checked="true"]:hover,.chk[aria-checked="true"]:focus-visible{background:var(--ok);color:#fff;filter:brightness(1.12)}
.g0l{list-style:none;padding:0;display:grid;gap:8px}
.g0i{display:flex;gap:10px;align-items:center}
.g0i.done span{text-decoration:line-through;color:var(--muted)}
"""

GOAL_JS = r"""
(function(){
  var KEY='gdm-goals-v1',st={};try{st=JSON.parse(localStorage.getItem(KEY)||'{}')}catch(e){}
  var save=function(){try{localStorage.setItem(KEY,JSON.stringify(st))}catch(e){}};
  var ORDER=__ORDER__;
  var tabs=document.querySelectorAll('.tabs button');
  function show(t){document.body.classList.toggle('tab-goal',t==='goal');
    tabs.forEach(function(b){b.setAttribute('aria-selected',b.dataset.tab===t?'true':'false')});
    try{localStorage.setItem('gdm-tab',t)}catch(e){}}
  tabs.forEach(function(b){b.addEventListener('click',function(){show(b.dataset.tab);
    history.replaceState(null,'',b.dataset.tab==='goal'?'#goal':location.pathname);window.scrollTo(0,0)})});
  var t0=location.hash==='#goal'?'goal':null;if(!t0){try{t0=localStorage.getItem('gdm-tab')}catch(e){}}
  show(t0==='goal'?'goal':'plan');
  var done=function(k){return !!(st[k]&&st[k].done)};
  function render(){
    var n=0,next=null;
    document.querySelectorAll('.chk').forEach(function(c){c.setAttribute('aria-checked',done(c.dataset.key)?'true':'false');
      var li=c.closest('.g0i');if(li)li.classList.toggle('done',done(c.dataset.key))});
    var hide=document.getElementById('ghide').getAttribute('aria-pressed')==='true';
    ORDER.forEach(function(id){var card=document.querySelector('.gc[data-goal="'+id+'"]');
      var miss=(card.dataset.pre||'').split(' ').filter(Boolean).filter(function(p){return !done(p)});
      var d=done(id),b=card.querySelector('.badge');if(d)n++;
      card.classList.toggle('done',d);card.classList.toggle('hide',d&&hide);card.classList.remove('next');
      b.className='badge';b.textContent='';
      if(d){b.classList.add('b-done');b.textContent='រួច'}
      else if(miss.length){b.classList.add('b-wait');b.textContent='រង់ចាំ '+miss.join(', ')}
      else if(!next&&id!=='G8'){next=id;card.classList.add('next');b.classList.add('b-next');b.textContent='បន្ទាប់'}
      else{b.classList.add('b-next');b.textContent='ធ្វើបាន'}
      card.querySelector('.when').textContent=st[id]&&st[id].copied?'បានចម្លង '+st[id].copied:''});
    document.getElementById('gprog').textContent=n+'/'+ORDER.length;
    document.getElementById('gnext').textContent=next?'បន្ទាប់: '+next:(n===ORDER.length?'រួចទាំងអស់':'');
    document.querySelector('.tabs .tc').textContent=n+'/'+ORDER.length}
  document.querySelectorAll('.chk').forEach(function(c){c.addEventListener('click',function(){
    var k=c.dataset.key;st[k]=st[k]||{};st[k].done=!st[k].done;
    if(st[k].done)st[k].at=new Date().toLocaleString();save();render()})});
  document.getElementById('ghide').addEventListener('click',function(){var x=this;
    x.setAttribute('aria-pressed',x.getAttribute('aria-pressed')==='true'?'false':'true');render()});
  document.querySelectorAll('.gc button.copy').forEach(function(x){x.addEventListener('click',function(){
    var card=x.closest('.gc'),id=card.dataset.goal;
    var miss=(card.dataset.pre||'').split(' ').filter(Boolean).filter(function(p){return !done(p)});
    if(done(id)&&!confirm(id+' បានធ្វើរួចហើយ។ ចម្លងម្ដងទៀតមែនទេ?'))return;
    if(miss.length&&!confirm(id+' ត្រូវការ '+miss.join(', ')+' ធ្វើរួចជាមុន។ ចម្លងទោះយ៉ាងណាមែនទេ?'))return;
    var t=document.getElementById(x.dataset.for).textContent;
    var ok=function(){st[id]=st[id]||{};st[id].copied=new Date().toLocaleString();save();render();
      x.textContent='ចម្លងរួច';setTimeout(function(){x.textContent='ចម្លង /goal'},1500)};
    var fail=function(){alert('ចម្លងមិនបាន។ បើក "មើលពាក្យបញ្ជាពេញ" ហើយចម្លងដោយដៃ។')};
    if(navigator.clipboard)navigator.clipboard.writeText(t).then(ok,fail);else fail()})});
  render();
})();
""".replace("__ORDER__", _json.dumps([g[0] for g in GOALS]))

TABS_HTML = ('<div class="tabs" role="tablist"><button type="button" role="tab" data-tab="plan" aria-selected="true">ផែនការ</button>'
             '<button type="button" role="tab" data-tab="goal" aria-selected="false">goal <span class="tc">0/%d</span></button></div>'
             '<div id="goalpanel" role="tabpanel">%s</div>' % (GOAL_COUNT, GOAL_PANEL))
