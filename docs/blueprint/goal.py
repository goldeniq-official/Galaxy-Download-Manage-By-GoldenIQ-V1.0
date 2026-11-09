"""§21 income plan and §22 goal page (copyable /goal commands for the developer)."""
import html as _h

MONEY_HTML = r"""
<p><strong>គោលការណ៍ (ម្ចាស់សម្រេច 2026-11-09):</strong> GDM ឥតគិតថ្លៃពិតសម្រាប់គ្រប់គ្នា រួមទាំងក្រុមហ៊ុន សាលា និងស្ថាប័នរដ្ឋ (LICENSE §2: Personal Use + Internal Use)។ ក្នុង app គ្មានពាណិជ្ជកម្ម គ្មានការតាមដាន គ្មាន activation (R3)។ ចំណូលមកពីអ្វីៗនៅជុំវិញ app មិនមែនពី app ខ្លួនឯងទេ។</p>
<div class="tw"><table><thead><tr><th>#</th><th>ប្រភពចំណូល</th><th>ច្បាប់អនុវត្ត</th><th>ពេលចាប់ផ្ដើម</th></tr></thead><tbody>
<tr><td>M1</td><td><strong>KHQR "គាំទ្រ GDM"</strong> (Bakong, ចូល ABA ផ្ទាល់)</td><td>នៅលើ README, គេហទំព័រ និងទំព័រ About (រូប QR ភ្ជាប់ក្នុង app, គ្មាន network)។ ការបរិច្ចាគជាការស្ម័គ្រចិត្ត មិនផ្ដល់មុខងារបន្ថែម ឬអាទិភាពទេ។ ផ្ទៀងផ្ទាត់លក្ខខណ្ឌគណនី ABA បង្កាន់ដៃ និងពន្ធ។</td><td>ឥឡូវ (ម្ចាស់ផ្ដល់រូប KHQR)</td></tr>
<tr><td>M2</td><td><strong>សេវាបង់ថ្លៃសម្រាប់ស្ថាប័ន</strong>: ដំឡើងច្រើនគ្រឿង, កំណត់ setting, បង្ហាត់, support, មុខងារតាមសំណូមពរ</td><td>App នៅតែឥតគិតថ្លៃ។ គិតថ្លៃតែពេលវេលា និងជំនាញ។ មុខងារតាមសំណូមពរត្រូវមានកិច្ចសន្យាជាលាយលក្ខណ៍អក្សរ: កម្មសិទ្ធិ (Golden IQ & CHHIT Sungchhay), ការបោះពុម្ពចូល product ផ្លូវការ (មិនសន្យាគ្រប់ករណី), លក្ខខណ្ឌទទួល, ការថែទាំ។ គ្មាន fork ឯកជន។</td><td>ពេល v0.9 beta</td></tr>
<tr><td>M3</td><td><strong>អ្នកឧបត្ថម្ភ</strong> (ISP, ក្រុមហ៊ុនទូរស័ព្ទ, hosting, ធនាគារ)</td><td>តែលើគេហទំព័រ និង README ប៉ុណ្ណោះ <strong>មិនមែនក្នុង app</strong> (logo បង់លុយក្នុង app = ពាណិជ្ជកម្ម)។ ដាក់ស្លាក "ឧបត្ថម្ភ"។ កិច្ចសន្យាហាមអ្នកឧបត្ថម្ភមិនឱ្យមានឥទ្ធិពលលើ default, ការទាញយក, លទ្ធផលស្វែងរក ឬទិន្នន័យអ្នកប្រើ។</td><td>ពេលមានអ្នកប្រើច្រើន</td></tr>
<tr><td>M4</td><td><strong>អាជ្ញាប័ណ្ណចែកចាយ</strong>: ហាង PC / OEM ដែលដំឡើងជាមុន ឬភ្ជាប់ GDM ក្នុង image</td><td>LICENSE §3 ហាមការចែកចាយ ⇒ ត្រូវសុំការអនុញ្ញាត §6 (អាចគិតថ្លៃ)។ ជាងដែលដំឡើង Official Build ពី Official Channel លើម៉ាស៊ីនអតិថិជនតាមសំណើ ឥតគិតថ្លៃ (LICENSE §2(e))។</td><td>ពេលមានសំណើ</td></tr>
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
<p class="note"><strong>Council (2 ជុំ, 2026-11-09):</strong> ChatGPT បានបន្ថែម: logo ឧបត្ថម្ភក្នុង app គឺជាពាណិជ្ជកម្ម; LICENSE ត្រូវមានសិទ្ធិជាងដំឡើង (§2(e)) និងប្រយោគ "Personal/Internal Use ឥតគិតថ្លៃជានិច្ច" (§6); មុខងារតាមសំណូមពរត្រូវមានកិច្ចសន្យា។ វាបានបង្ហាញ URL policy AdSense ផ្ទាល់ ដែល Claude ផ្ទៀងផ្ទាត់រួច។ ការអះអាងថា Payoneer ទទួលការបរិច្ចាគបាន មិនទាន់ផ្ទៀងផ្ទាត់ ⇒ ពន្យារ។</p>
"""

# Each goal: (id, title, who/when, command text). Project rules in CLAUDE.md (no push, authorship, Khmer replies,
# no glow/outline, The Algorithm, council) apply automatically, so commands only carry what is specific to the goal.
GOALS = [
    ("G1", "P0-A · បញ្ជីសេចក្ដីសម្រេច និងផែនការ spike", "បន្ទាប់ពីម្ចាស់ឆ្លើយ G0",
     "/goal Read docs/GDM-Blueprint.html (§17 P0, §19, §07, §09, §15). Write docs/p0/plan.md (Khmer): (1) decision register D1-D18 with the owner's actual answer for each (never assume silence = yes; ask the owner for any missing one); (2) inventory of the 9 P0 spikes, each with deliverable, pass criterion, fallback and estimate; (3) update ownership per install type (Store MSIX vs NSIS/GitHub); (4) whether the owner accepts unsigned GitHub as the public fallback; (5) a repeatable benchmark protocol vs IDM (controlled bandwidth, connection count, cache state, repeated runs, median and worst case). Done when the file is committed on main, council has reviewed it, and the owner has confirmed the register."),
    ("G2", "P0-B · Spike ការចែកចាយ Windows និង browser", "ក្រោយ G1",
     "/goal Execute the distribution spikes from docs/p0/plan.md as throwaway prototypes under spikes/: MSIX package whose native messaging host is visible to Chrome, Edge and Firefox (desktop:Extension / unvirtualized registry, or a separate installer helper); install, launch, upgrade and uninstall on Windows 10 and 11; a component (yt-dlp) downloaded after install; NSIS fallback with Khmer.nsh. Record every result in docs/p0/results.md with exact steps, versions and pass/fail, and say plainly which results are local validation and which are real Store certification. Done when every distribution spike has a recorded result (success or tested plan B), the chosen route and update owner are written down, and council has reviewed the results."),
    ("G3", "P0-C · Spike ភាពត្រឹមត្រូវ engine និងព្រំដែនសុវត្ថិភាព", "ស្របពេល ឬក្រោយ G2",
     "/goal Execute the engine and security spikes from docs/p0/plan.md under spikes/: hostile HTTP fixture server (200-on-range, ETag change, infinite body, slowloris, cross-origin redirect, 429); segmenter with write lease and 206/Content-Range/If-Range checks; .part in the target folder and MOTW via IAttachmentExecute from Rust; named pipe with DACL and server verification; MV3 prepared handoff; yt-dlp policy proxy (ADR-6); update install + recover (U-04, U-06); speed benchmark vs IDM using the agreed protocol. Each spike leaves one runnable check. Record results in docs/p0/results.md. Done when all of them have repeatable results or a documented fallback, and council has reviewed them."),
    ("G4", "P0-D · Design និង baseline MVP (បិទ P0)", "ក្រោយ G2 + G3",
     "/goal Produce design tokens and the 5 screen mockups from §06 in Khmer and English, light and dark, with keyboard focus shown by fill/colour change only (D6: no outline, no glow). Freeze the MVP scope and the supported Windows versions/architectures, run cargo-deny / licence inventory for every dependency and downloaded binary (MPL-2.0 unmodified allowed, no linked GPL/AGPL/LGPL), and revise the P1 estimate from the spike evidence. Done when the owner approves the mockups and the frozen P1 acceptance list in docs/p0/exit.md, and the P0 exit council review is recorded."),
    ("G5", "P1-A · Core ទាញយក HTTP ដែលរក្សាទុកបាន", "ក្រោយ G4",
     "/goal Implement the GDM download core in Rust per §08-§09 and docs/p0/exit.md: HTTP/HTTPS engine with dynamic segmentation, resume, integrity checks and global + per-job limiter; .part/rename/MOTW finalisation; SQLite persistence and crash recovery. Tests first for every critical/high card in F01-F05. Done when those tests pass in CI, the hostile fixtures pass, 1000 kill -9 cycles produce 0 corrupt files, and there is no open critical integrity defect."),
    ("G6", "P1-B · Beta v0.9 ពេញលេញ", "ក្រោយ G5",
     "/goal Complete the P1 MVP on top of the core: React UI (list, details, settings), Download Info, toast, tray, Chrome/Edge/Firefox extension (capture, Alt/Ctrl, context menu, popup), categories, FTS search, Khmer/English, two themes, updater, NSIS + MSIX installers, full CI from §15. Done when every P1 exit criterion in §17 has evidence (cards F12-F17 pass, MSIX native messaging works or the tested plan B, speed gate per the benchmark protocol, Khmer screenshots without clipping, axe 0 serious), and draft release notes with known limitations are ready for the owner. Do not publish or push; the owner releases."),
    ("G7", "P2 · v1.0 parity ជាមួយ IDM", "ក្រោយ beta v0.9",
     "/goal Implement only the owner-approved P2 rows of the parity ledger (§04): queues, scheduler, post actions, refresh address, batch, clipboard, IDM CLI flags, import .ef2/txt, export JSON, proxy PAC/NTLM, site logins, per-server limits, quota, sounds, drop basket, video via opt-in yt-dlp/ffmpeg components, failure-injection tests. Done when every P2 row is done or has an explicit owner waiver, cards F06-F11 pass, THIRD-PARTY-NOTICES and PRIVACY.md are complete, and the beta result (>=8/10 testers say it replaces IDM) is recorded."),
    ("G8", "ចំណូល · KHQR និងសេវា (§21)", "ពេលណាក៏បាន (ម្ចាស់ផ្ដល់រូប KHQR)",
     "/goal Set up the income plan in §21 of the blueprint: add the owner's KHQR image and a short voluntary 'Support GDM' block to README (Khmer first) and to the app's About page as a bundled image with no network call; write docs/services.md (paid deployment, configuration, training, commissioned features with written terms on ownership and maintenance); write docs/sponsors.md (website/README only, labelled, no influence on defaults, downloads or data). No ads, trackers or payment SDKs in the app. Done when the files are committed, the wording matches 'free for everyone', and the owner approves."),
    ("G9", "P3 · មុខងារម្ដងមួយ (គំរូ)", "ក្រោយ v1.0",
     "/goal Implement the P3 feature <NAME> from §17 (one feature per goal: site grabber, zip preview, SFTP, HTTP/3, native HLS, portable mode, gdm://, Weblate translation, VPN/rasdial). Start from its scenario cards in §12, write the tests first, and keep the change inside the existing architecture. Done when its cards pass in CI and the feature is documented in Khmer and English."),
]

G0_HTML = """<div class="card"><h3>G0 · ម្ចាស់ត្រូវឆ្លើយមុន (មិនមែន /goal)</h3><ul>
<li><strong>D11/D12</strong>: ឈ្មោះ "Galaxy Download Manager" និង package ID <code>GoldenIQ.GalaxyDownloadManager</code> (ត្រូវការមុនបង្កើត repo ឬ ID ណាមួយ)។</li>
<li><strong>D9/D17</strong>: Golden IQ ជានីតិបុគ្គល ឬឈ្មោះពាណិជ្ជកម្ម? គណនី Store បុគ្គល ឬក្រុមហ៊ុន? នរណាផ្ដល់ការអនុញ្ញាតបាន? (ត្រូវការមុន submit Store)។</li>
<li><strong>D2–D8, D10, D13–D15, D18</strong>: យល់ព្រមតាមអនុសាសន៍ ឬប្ដូរ (§19)។ ភាពស្ងៀមស្ងាត់មិនមែនជាការយល់ព្រម។</li>
<li>បង្កើត repo GitHub <code>galaxy-download-manager</code> ហើយ push ដោយខ្លួនឯង (agent មិន push ទេ)។</li>
</ul></div>"""


def goal_html():
    cards = "".join(
        '<div class="card"><h3>%s · %s</h3><p class="byline">%s</p><pre id="cmd-%s">%s</pre>'
        '<button class="copy" type="button" data-for="cmd-%s">ចម្លង</button></div>'
        % (gid, _h.escape(t), _h.escape(when), gid, _h.escape(cmd), gid)
        for gid, t, when, cmd in GOALS)
    return ('<p>ពាក្យបញ្ជា <code>/goal</code> សម្រាប់អ្នកអភិវឌ្ឍ (Claude Code)។ ចុច "ចម្លង" ហើយបិទភ្ជាប់ក្នុង session ថ្មី '
            'ម្ដងមួយ តាមលំដាប់។ ច្បាប់ក្នុង CLAUDE.md (មិន push, អ្នកនិពន្ធ GoldenIQ, ឆ្លើយខ្មែរ, គ្មាន glow/outline, '
            'The Algorithm, council) អនុវត្តដោយស្វ័យប្រវត្តិ ដូច្នេះពាក្យបញ្ជាមិនបាច់សរសេរម្ដងទៀតទេ។ '
            'goal នីមួយៗបញ្ចប់តែពេលមានភស្តុតាង (test, លទ្ធផលកត់ត្រា) មិនមែនដោយការអះអាងទេ។</p>'
            + G0_HTML + cards)


GOAL_HTML = goal_html()
