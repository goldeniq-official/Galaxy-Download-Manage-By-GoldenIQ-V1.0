# Content of the GDM blueprint (Khmer first). Sections are (id, number, title, html body).
import html as _h

from parity import PARITY_HTML
from design import DESIGN_HTML
from codereview import CODE_HTML
from council import COUNCIL_HTML


def code(s):
    return "<pre>" + _h.escape(s.strip("\n")) + "</pre>"


META = {
    "logo": "../logo/gdm-icon-256.png",
    "eyebrow": "Galaxy Download Manager · Blueprint · v1 · 2026-11-09",
    "title": "GDM: ផែនការស៊ីជម្រៅ មុនសរសេរកូដមួយបន្ទាត់",
    "byline": "GoldenIQ · ការសិក្សា IDM និង NDM · UX · ស្ថាបត្យកម្ម · សុវត្ថិភាព · សេនារីយ៉ូដេញដោលតាមមុខងារ · ផែនការអភិវឌ្ឍ",
}

S00 = r"""
<div class="kpis">
<div class="kpi"><b>68</b><span>មុខងារ IDM/NDM ក្នុង parity ledger (§04)</span></div>
<div class="kpi"><b>{{NSCEN}}</b><span>សេនារីយ៉ូដេញដោល តាម 17 មុខងារ (§12)</span></div>
<div class="kpi"><b>{{N_CRITICAL}}</b><span>critical ដែលការរចនាត្រូវបិទមុន MVP</span></div>
<div class="kpi"><b>0 $</b><span>ថ្លៃអ្នកប្រើ, គ្មានគណនី, គ្មាន telemetry</span></div>
<div class="kpi"><b>18</b><span>ចំណុចដែលម្ចាស់ត្រូវសម្រេច (§19)</span></div>
</div>
<p><strong>សេចក្ដីសម្រេចសំខាន់:</strong> GDM ជាកម្មវិធី Windows ដំបូង សរសេរដោយ Rust (engine) + Tauri v2 + React (UI)។ វាឥតគិតថ្លៃសម្រាប់អ្នកប្រើ ហើយប្រភពកូដអាចមើលបាន (source-available) ក្រោមអាជ្ញាប័ណ្ណកម្មសិទ្ធិរបស់ Golden IQ និង CHHIT Sungchhay (ម្ចាស់សម្រេច 2026-11-09)។ <strong>វាមិនមែនជា open source តាមនិយមន័យ OSI ទេ</strong>។ វារក្សាគ្រប់មុខងារ IDM និង NDM ដែលនៅមានន័យ ហើយកែចំណុចខ្សោយដែលអ្នកប្រើត្អូញត្អែរពិតប្រាកដ។</p>
<ul>
<li><strong>ល្បឿនដូច IDM:</strong> dynamic segmentation (កាត់ផ្នែកធំបំផុតពាក់កណ្ដាល) ប៉ុន្តែចាប់ផ្ដើមពី 2 connection ហើយបន្ថែមតែពេលល្បឿនកើនពិត។ ការផ្ទៀងផ្ទាត់ 206/Content-Range/If-Range និង write lease ការពារការលាយកំណែ; ពេលគ្មាន validator ខ្លាំង GDM ប្រាប់ថា "មិនបានផ្ទៀងផ្ទាត់" ជំនួសឱ្យការសន្មត (IDM ធ្លាក់ទៅ 1 connection ដោយស្ងាត់)។</li>
<li><strong>គ្មាន "stuck at 99%":</strong> ឯកសារ <code>.part</code> នៅក្នុងថតគោលដៅ ដូច្នេះគ្មានការ merge ឬផ្លាស់ទីពី temp ដូច IDM/NDM ទេ។</li>
<li><strong>ការភ្ជាប់ browser មានសុវត្ថិភាព:</strong> native messaging → named pipe ដែលមាន DACL។ គ្មាន localhost port (NDM បើក <code>ws://127.0.0.1:10007</code> គ្មាន token) និងគ្មាន kernel driver (IDM <code>idmwfp.sys</code>: CVE-2026-90493)។</li>
<li><strong>Update មិនរំខាន:</strong> ទាញនៅខាងក្រោយ ហើយដំឡើងតែពេលគ្មានការទាញយកសកម្ម។ មានហត្ថលេខាពីរស្រទាប់ (minisign + Authenticode) និងការពារ downgrade (IDM CVE-2025-56231: updater មិនពិនិត្យ TLS)។</li>
<li><strong>ឥតគិតថ្លៃពិត:</strong> គ្មាន serial ដូច្នេះគ្មាន "fake serial" ដែលសូម្បីអ្នកបង់លុយ IDM ក៏ជួបដែរ។ ផ្លូវចុះហត្ថលេខាឥតគិតថ្លៃគឺ Microsoft Store (MSIX ដែល Microsoft ចុះហត្ថលេខាឱ្យ)។ SignPath មិនអាចប្រើបានទេ ព្រោះវាតម្រូវ licence OSI (§15)។</li>
<li><strong>ខ្មែរមុន:</strong> installer និងផ្ទាំងបើកលើកដំបូងដាក់ខ្មែរលំដាប់ទីមួយ។ ពុម្ពអក្សរខ្មែរភ្ជាប់ក្នុងកម្មវិធី ហើយមានការពិនិត្យរូបភាព (visual test) ខ្មែរគ្រប់ផ្ទាំង។</li>
</ul>
<p class="note warn"><strong>អ្វីដែលមិនសន្យា:</strong> ទាញមាតិកាមាន DRM (Netflix, Widevine), ចាប់ការទាញយកពីគ្រប់កម្មវិធីដោយគ្មាន driver, និងធានាថា YouTube ដំណើរការរហូត (គេហទំព័រប្ដូររាល់សប្ដាហ៍: IDM ចេញ ~15 build ដើម្បីជួសជុល 403 តែមួយ)។</p>
<p class="note"><strong>តួនាទី:</strong> ឯកសារនេះជាផែនការរបស់ទីប្រឹក្សា មិនទាន់មានកូដទេ។ "ការពិនិត្យម្ដងមួយបន្ទាត់" ក្នុង §13 ជាកូដគំរូ (reference) ដែលបង្កើតឡើងដើម្បីកំណត់ invariant។ ការពិនិត្យបន្ទាត់ពិតប្រាកដ គឺជាច្រកឆ្លង (gate) មុន release លើកូដពិតតាម commit (ដូច ChatGPT បានរំលឹក)។</p>
"""

S01 = r"""
<p>ជំហានទី ១ នៃ The Algorithm: តម្រូវការនីមួយៗត្រូវមានឈ្មោះអ្នកទាមទារ ហេតុផល និងភស្តុតាង។ តម្រូវការដែលខ្សោយត្រូវបានកែ មិនមែនធ្វើតាមដោយងងឹតងងល់ទេ។</p>
<div class="tw"><table><thead><tr><th>#</th><th>តម្រូវការ</th><th>អ្នកទាមទារ</th><th>ហេតុផល / ភស្តុតាង</th><th>ការបកស្រាយចុងក្រោយ</th></tr></thead><tbody>
<tr><td>R1</td><td>មិនបាត់មុខងារ IDM/NDM មួយណាឡើយ</td><td>GoldenIQ</td><td>ធុញនឹង IDM (បង់លុយ) និង NDM (UX ខ្សោយ)</td><td>Parity ledger 68 ជួរ (§04)។ ជួរនីមួយៗមានស្ថានភាព: រក្សា / ល្អជាង / ជំនួសដោយសមមូល / ម្ចាស់សម្រេច។ គ្មានការលុបមុខងារដោយស្ងាត់ទេ។ មុខងារដែលវេទិការបស់វាស្លាប់ហើយ (IE, MMS, dial-up modem) ត្រូវការការយល់ព្រមពីម្ចាស់ (§19)។</td></tr>
<tr><td>R2</td><td>ខ្លាំងជាងគេ ទាំងមុខងារដែលគេល្អស្រាប់</td><td>GoldenIQ</td><td>—</td><td>រង្វាស់: ល្បឿនមធ្យម ≥ IDM −5% និងស្មើ ឬលឿនជាងលើ ≥ 3/5 URL តេស្ត, 0 ឯកសារខូចលើ hostile fixtures, ពេលពីចុចដល់ចាប់ផ្ដើមទាញ &lt; 300 ms (§17 exit criteria)។</td></tr>
<tr><td>R3</td><td>ឥតគិតថ្លៃ គ្មានការចុះឈ្មោះ ទាញហើយប្រើ</td><td>GoldenIQ</td><td>IDM: licence 1 ឆ្នាំឈប់ដំណើរការ, serial ត្រូវបិទ</td><td>គ្មានគណនី, គ្មាន activation, គ្មាន telemetry។ ផ្នែកដែលត្រូវ "ចុះឈ្មោះ" គឺខាងអ្នកអភិវឌ្ឍ (Microsoft Store, store extension) មិនមែនអ្នកប្រើទេ។</td></tr>
<tr><td>R4</td><td>"GitHub Open sauce ហើយចុះឈ្មោះ Open sauce ដើម្បីបាន certificate"</td><td>GoldenIQ</td><td>ចង់បាន certificate ឥតគិតថ្លៃ ដើម្បីកុំឱ្យ Windows ព្រមាន</td><td><strong>ផ្ទុយគ្នាជាមួយ R11</strong> (licence តឹងរឹង)។ SignPath Foundation និង Certum Open Source តម្រូវ licence OSI ប៉ុន្តែ licence ដែលហាមកែប្រែ និងហាមពាណិជ្ជកម្មមិនមែន OSI ទេ (OSD ចំណុច 1, 3, 6)។ ដូច្នេះបកស្រាយថ្មី: repo សាធារណៈលើ GitHub (source-available) + ចុះហត្ថលេខាតាម Microsoft Store MSIX (ឥតគិតថ្លៃ)។ Installer នៅលើ GitHub Releases នឹងគ្មានហត្ថលេខា រហូតដល់ម្ចាស់ទិញ certificate (D8)។ ផ្លូវចែកចាយបន្ថែមឥតគិតថ្លៃ: Microsoft Store (គណនីបុគ្គលឥតគិតថ្លៃតាំងពីកញ្ញា 2025; Store ចុះហត្ថលេខាឱ្យតែ MSIX ប៉ុណ្ណោះ, installer Win32 ត្រូវចុះហត្ថលេខាដោយយើង)។</td></tr>
<tr><td>R5</td><td>Rust + React + "..." តាមតម្រូវការ</td><td>GoldenIQ</td><td>—</td><td>Rust (engine, bridge), Tauri v2 (shell), React + TypeScript (UI), SQLite។ yt-dlp/ffmpeg ជា component ជម្រើស (opt-in)។ ហេតុផលដែលមិនប្រើ Electron: ទំហំ ~150 MB និង memory (Motrix ត្រូវគេត្អូញត្អែរ)។</td></tr>
<tr><td>R6</td><td>ខ្មែរ + អង់គ្លេស, ខ្មែរនៅមុខពេលដំឡើង</td><td>GoldenIQ</td><td>—</td><td>NSIS installer មានភាសាខ្មែរជាជម្រើសទីមួយ (NSIS គ្មាន Khmer.nlf ផ្លូវការ ដូច្នេះយើងសរសេរ <code>Khmer.nsh</code> ខ្លួនឯង)។ ផ្ទាំងជ្រើសភាសាក្នុង app ពេលបើកលើកដំបូងជាផ្លូវធានាបម្រុង។</td></tr>
<tr><td>R7</td><td>រចនាឆ្លាតតាមទំហំឧបករណ៍, ទាន់សម័យ, ស្ទីល Mazer Client, គ្មាន glow គ្មាន outline</td><td>GoldenIQ (CLAUDE.md ច្បាប់ 8)</td><td>—</td><td>§06: ទម្រង់ ៣ (compact/medium/wide) តាមទំហំ window, Per-Monitor DPI, design tokens ដកស្រង់ពី Mazer Client, state បង្ហាញដោយ fill មិនមែន outline។</td></tr>
<tr><td>R8</td><td>Auto-update ដែលស្រួលបួល</td><td>GoldenIQ</td><td>IDM update ប្រចាំសប្ដាហ៍ ហើយបិទមិនបាន</td><td>§07: ម៉ាស៊ីនស្ថានភាព update (state machine) ពេញលេញ។ មិនដែលរំខានការទាញយក។</td></tr>
<tr><td>R9</td><td>ពិនិត្យសុវត្ថិភាពគ្រប់សេនារីយ៉ូ ដេញតាមមុខងារនីមួយៗ រហូតដល់មួយបន្ទាត់</td><td>GoldenIQ</td><td>—</td><td>§10 threat model, §11 matrix S1–S15, §12 កាត {{NSCEN}}, §13 ការពិនិត្យកូដគំរូម្ដងមួយបន្ទាត់ លើអនុគមន៍គ្រោះថ្នាក់បំផុត ៤។</td></tr>
<tr><td>R11</td><td>Licence តឹងរឹង: ហាមកែប្រែ, ហាមពាណិជ្ជកម្ម, រាល់ការប្រើក្រៅពីការប្រើផ្ទាល់ខ្លួនត្រូវសុំការអនុញ្ញាត, រក្សាសិទ្ធិ Golden IQ &amp; CHHIT Sungchhay</td><td>GoldenIQ (2026-11-09)</td><td>ការពារស្នាដៃ និងម៉ាក</td><td>មានពីរកំណែ: (1) <code>LICENSE</code> ក្នុង repo ("GoldenIQ Restricted Personal-Use License", commit 683a56e, សរសេរដោយ session ផ្សេង) (2) <code>docs/licence-draft/</code> (អង់គ្លេស + ខ្មែរ) ដែលឆ្លង council ជុំ 4–5 (Work មិនរួមឯកសារអ្នកប្រើ, សិទ្ធិចម្លងចាំបាច់សម្រាប់ install/update/backup, §2A Store, cure 30 ថ្ងៃ, សិទ្ធិ consumer)។ ម្ចាស់ជ្រើសមួយ (D1)។ ផលវិបាក: (1) គ្មាន SignPath (2) dependency ត្រូវមាន licence permissive (MIT/Apache/BSD/ISC/Zlib/MPL-2.0/OFL) ប៉ុណ្ណោះ, ហាម GPL/AGPL/LGPL ដែល link ផ្ទាល់ (<code>cargo deny</code>) (3) ការប្រើដោយក្រុមហ៊ុន/សាលាត្រូវសុំការអនុញ្ញាត ដូច្នេះអាចបន្ថយអ្នកប្រើក្នុងការិយាល័យ (D16)។ គួរឱ្យមេធាវីកម្ពុជាពិនិត្យមុន release ទីមួយ។</td></tr>
<tr><td>R10</td><td>ជជែកដេញដោលជាមួយ ChatGPT រហូតដល់ផ្លូវកណ្ដាលដែលទុកចិត្តបាន</td><td>GoldenIQ (ច្បាប់ 6)</td><td>—</td><td>§18: council 7 ជុំ, ChatGPT ទទួលយកជាផ្លូវកណ្ដាល (99%)។ មានកំណត់ត្រាចំណុចដែលវាបន្ថែម ចំណុចដែលវាខុស និងចំណុចដែលឯកភាពចុងក្រោយ។</td></tr>
</tbody></table></div>
"""

S02 = r"""
<p>IDM 6.43 Build 18 (4 វិច្ឆិកា 2026) ដោយ Tonec, Windows ប៉ុណ្ណោះ។ ប្រភព: internetdownloadmanager.com (FAQ, options, news), AMO reviews, AlternativeTo, NVD, Exploit-DB។ បញ្ជីមុខងារពេញលេញមាននៅក្នុង parity ledger (§04)។</p>
<h3>ចំណុចខ្លាំងដែលអ្នកប្រើសរសើរ (តាមលំដាប់)</h3>
<ol>
<li><strong>ល្បឿន:</strong> dynamic segmentation ប្រើ connection ឡើងវិញ ហើយកាត់ផ្នែកធំបំផុតពាក់កណ្ដាលពេលមាន connection ទំនេរ។</li>
<li><strong>"ចាប់បានគ្រប់យ៉ាង":</strong> ចាប់ពី browser, Ctrl+click បង្ខំឱ្យ IDM ទាញ, Alt+click រំលង, និងមានបញ្ជីលើកលែងតាមគេហទំព័រ។</li>
<li><strong>ចាប់វីដេអូ:</strong> ផ្ទាំង "Download this video" មានជម្រើសគុណភាព ហើយ mux ទៅ MP4/MKV។</li>
<li><strong>Resume និង Refresh download address:</strong> សំខាន់ខ្លាំងនៅតំបន់ដែលអ៊ីនធឺណិតមិនទៀង (អាស៊ីអាគ្នេយ៍, មជ្ឈិមបូព៌ា)។</li>
<li><strong>ផ្ទាំង Download File Info ចាប់ផ្ដើមទាញភ្លាម:</strong> ទាញខណៈអ្នកប្រើកំពុងជ្រើសថត ដូច្នេះលាក់ពេលរង់ចាំ។</li>
<li><strong>Queue/Scheduler ពេញលេញ:</strong> មាន synchronization queue, កំណត់ quota, និងបិទម៉ាស៊ីនពេលបញ្ចប់។</li>
<li><strong>ជួសជុលលឿនពេលគេហទំព័រប្ដូរ:</strong> ចេញ build ថ្មីក្នុងរយៈពេលប៉ុន្មានថ្ងៃ។ ភាសា 50+ រួមទាំង RTL។</li>
</ol>
<h3>ចំណុចខ្សោយ (មានភស្តុតាង)</h3>
<div class="tw"><table><thead><tr><th>ផ្នែក</th><th>បញ្ហា</th><th>GDM ដោះស្រាយដោយ</th></tr></thead><tbody>
<tr><td>Licence</td><td>Licence 1 ឆ្នាំឈប់ដំណើរការពេលផុតកំណត់។ Licence មួយជីវិតបាន update តែ 3 ឆ្នាំ។ Serial ត្រូវបិទពេលរកឃើញថាប្រើលើ PC ច្រើន ហើយអ្នកបង់លុយក៏ជួប "fake serial" ដែរ។</td><td>ឥតគិតថ្លៃសម្រាប់ការប្រើផ្ទាល់ខ្លួន, គ្មាន serial, គ្មាន phone-home។</td></tr>
<tr><td>Crack/malware</td><td>Crack របស់ IDM ត្រូវប្រើជានុយចែក infostealer។ Extension ក្លែង "IDM" មានអ្នកដំឡើង 200k+ ព្រោះ extension ពិតលាក់ពីការស្វែងរកក្នុង store។</td><td>ឥតគិតថ្លៃ ដូច្នេះគ្មានហេតុផលដើម្បី crack។ Store ID បង្ហាញក្នុង app និង README។</td></tr>
<tr><td>វីដេអូ</td><td>YouTube 403 (ចេញ ~15 build ដើម្បីជួសជុល), ផ្ទាំងវីដេអូបាត់, Shorts ដំណើរការ ~60%, CPU ខ្ពស់លើ YouTube។</td><td>Component extractor (yt-dlp) ដែល update ដាច់ពី app។</td></tr>
<tr><td>Extension</td><td>ខូចលើ Firefox fork និង Brave។ Extension ត្រូវបិទរាល់ពេលបើក browser។ ផ្ទាំងវីដេអូរំខាន។</td><td>ចុះ host manifest គ្រប់ browser, មានប៊ូតុង "ជួសជុលការភ្ជាប់", ផ្ទាំងវីដេអូបិទលំនាំដើម (បើកតាមគេហទំព័រ)។</td></tr>
<tr><td>Engine</td><td>Speed limiter កំណត់តែក្នុងមួយឯកសារ។ Temp folder ហើយ rebuild ធ្វើឱ្យ "stuck at 99%"។ ពេលខូចធ្លាក់ទៅ 1 connection។ គ្មាន hash verify។</td><td>Limiter សរុប + ក្នុង job + ក្នុង queue + តាមម៉ោង។ .part ក្នុងថតគោលដៅ។ ការផ្ទៀងផ្ទាត់ 206 យ៉ាងតឹងរឹង។ SHA-256។</td></tr>
<tr><td>UI</td><td>Win32 ចាស់ (toolbar skin "3D/Neon")។ Dark mode គ្របតែផ្នែកខ្លះ។ Settings migrate ដោយ export registry ដោយដៃ។</td><td>UI ថ្មីពេញលេញ, theme ពីរគ្រប់ផ្ទាំង, export settings ជា JSON។</td></tr>
<tr><td>Update</td><td>រំខានប្រចាំសប្ដាហ៍ ហើយ "cannot avoid"។ Update និង help តាម HTTP (EDB 51131)។</td><td>§07</td></tr>
<tr><td>ប្រព័ន្ធ</td><td>WFP kernel driver: BSOD និង CVE-2026-90493 (LPE)។ DLL ក្នុង Explorer ធ្វើឱ្យ Explorer crash។</td><td>គ្មាន driver, គ្មាន DLL injection។</td></tr>
<tr><td>Platform</td><td>Windows ប៉ុណ្ណោះ។ គ្មាន torrent, SFTP, HTTP/3។</td><td>Windows មុន, core ឯករាជ្យពី platform, SFTP/HTTP/3 ក្នុង Phase 3, torrent ជាជម្រើស Phase 4។</td></tr>
</tbody></table></div>
<h3>ប្រវត្តិសុវត្ថិភាព IDM</h3>
<div class="tw"><table><thead><tr><th>ID</th><th>កំណែ</th><th>បញ្ហា</th><th>មេរៀនសម្រាប់ GDM</th></tr></thead><tbody>
<tr><td>CVE-2026-90493</td><td>≤ 6.42 b63</td><td>Driver <code>idmwfp.sys</code> ឱ្យ user ធម្មតាកែ registry បាន (LPE, CVSS 8.8)។ បោះពុម្ព 2026-09-13 ហើយ vendor មិនឆ្លើយ។</td><td>មិនប្រើ kernel driver</td></tr>
<tr><td>CVE-2025-56231</td><td>≤ 6.42.41.1</td><td>Updater មិនពិនិត្យ TLS certificate (CWE-295, 9.1)</td><td>U-01, W-03</td></tr>
<tr><td>EDB 51131</td><td>6.41 b3</td><td>ទាញ CHM help តាម HTTP ហើយបើកភ្លាម ⇒ MITM RCE</td><td>គ្មានការទាញ code/help តាម HTTP</td></tr>
<tr><td>CVE-2020-23060 / 28964 / 37234</td><td>6.37–6.38</td><td>Stack overflow ក្នុង import <code>.ef2</code>, Search, Scheduler</td><td>Rust + កំណត់ប្រវែងគ្រប់ field (I-03)</td></tr>
<tr><td>CVE-2010-0995, CVE-2005-2210</td><td>ចាស់</td><td>Overflow ដោយ FTP URI និង URL វែង</td><td>Parser តឹងរឹង (B-03)</td></tr>
</tbody></table></div>
"""

S03 = r"""
<p>NDM ដោយ Javad Motallebi (អ្នកអភិវឌ្ឍម្នាក់): Windows v1.4 (~2022) និង macOS v1.3។ Extension Chrome 1.9.92 (វិច្ឆិកា 2023) មានអ្នកប្រើ ~700k។ <strong>ឈប់អភិវឌ្ឍន៍ហើយ</strong> (AlternativeTo សម្គាល់ថា discontinued)។ បិទ source, installer គ្មានហត្ថលេខា, និងមានតែភាសាអង់គ្លេស។ ភស្តុតាងខាងក្រោមខ្លះមកពីការអានកូដ extension Firefox 1.9.9 ដោយផ្ទាល់។</p>
<h3>ចំណុចខ្លាំង</h3>
<ul>
<li>ឥតគិតថ្លៃ គ្មានពាណិជ្ជកម្ម ហើយជា "IDM សម្រាប់ Mac"។ Installer តូចត្រឹម ~900 KB ហើយចាប់ផ្ដើមភ្លាម។</li>
<li>កំណត់ចំនួន connection និងល្បឿន <strong>ក្នុងពេលកំពុងទាញ</strong>។ Limiter មានទាំងក្នុងមួយ job និងសរុប។</li>
<li>Extension ផ្ញើ cookie, referrer និង <strong>POST body</strong> ដូច្នេះការទាញតាម form ដំណើរការ។</li>
<li>ផ្ទាំងមេឌៀបង្ហាញគុណភាព, bitrate, ទំហំ, រយៈពេល និង subtitle។ ដំណើរការលើគេហទំព័រច្រើន។</li>
<li>មានប៊ូតុង "Browsers" ដើម្បីដំឡើង extension និង tree ចែកជា Complete/Incomplete/តាមប្រភេទ។</li>
</ul>
<h3>ចំណុចខ្សោយ</h3>
<div class="tw"><table><thead><tr><th>ផ្នែក</th><th>បញ្ហា</th><th>GDM</th></tr></thead><tbody>
<tr><td>សុវត្ថិភាព</td><td><code>ws://127.0.0.1:10007</code> គ្មាន token។ Process ផ្សេងអាចយក port មុន ហើយទទួល cookie គ្រប់ការទាញយក។ គេហទំព័រអាចភ្ជាប់ WebSocket ទៅ localhost បាន (មិនទាន់សាកលើ Origin check)។</td><td>B-01, B-02: native messaging + pipe ដែលផ្ទៀងផ្ទាត់</td></tr>
<tr><td>ការចាប់</td><td>ចាប់ខុស (<code>compression-dictionary.dat</code> លើ Reddit, Synology Office ខូច)។ ប៊ូតុងបិទការចាប់លាក់។ Bypass ដោយ Delete key មិនមាននរណាដឹង។</td><td>C-01, C-04, popup ច្បាស់</td></tr>
<tr><td>ផ្ទាំងមេឌៀ</td><td>ទទឹង 210px លើគ្រប់ទំព័រ ("ugly"), លោតក្នុង player ទោះបិទ, បញ្ជីលើស 10,000 ជួរលើ live</td><td>បិទលំនាំដើម, ប៊ូតុងតូចបើកតាមគេហទំព័រ, កំណត់ 50 ធាតុ</td></tr>
<tr><td>មេឌៀ</td><td>គ្មាន YouTube (រំលង signatureCipher), គ្មាន DASH, HLS បានតែ .ts, វីដេអូគ្មានសំឡេង</td><td>F07: yt-dlp + ffmpeg mux MP4</td></tr>
<tr><td>Engine</td><td>Merge ឯកសារធំគាំង ហើយមិនប្រាប់ថាផ្នែកណាខូច។ SSL handshake error លើគេហទំព័រខ្លះ</td><td>.part តែមួយ (គ្មាន merge), rustls + platform verifier</td></tr>
<tr><td>ខ្វះមុខងារ</td><td>Scheduler, queue ពិតប្រាកដ, ថតតាម category, សកម្មភាពក្រោយបញ្ចប់, batch, clipboard, ស្វែងរកប្រវត្តិ, ភាសាផ្សេង</td><td>មានទាំងអស់ (§04)</td></tr>
<tr><td>ទំនុកចិត្ត</td><td>Installer គ្មានហត្ថលេខា ហើយ antivirus ចាប់ថាជាមេរោគ (ទំនងជាចាប់ខុស)។ គ្មាន update, គ្មាន issue tracker</td><td>Store MSIX, GitHub Issues, update ស្វ័យប្រវត្តិ</td></tr>
</tbody></table></div>
"""

S04 = r"""
<p>ច្បាប់: គ្មានមុខងារណាមួយត្រូវលុបដោយស្ងាត់ឡើយ។ <span class="tag keep">រក្សា</span> ដូចដើម · <span class="tag better">ល្អជាង</span> រក្សា ហើយកែចំណុចខ្សោយ · <span class="tag new">ថ្មី</span> មិនមានក្នុង IDM/NDM · <span class="tag swap">សមមូល</span> លទ្ធផលដូចគ្នាសម្រាប់អ្នកប្រើ តែប្រើវិធីផ្សេង (មានហេតុផល) · <span class="tag">ម្ចាស់</span> ត្រូវការការសម្រេចរបស់ម្ចាស់ (§19)។ P1–P4 = ដំណាក់កាលក្នុង §17។</p>
""" + "{{PARITY}}"

S05 = r"""
<h3>អ្នកប្រើគោលដៅ (persona) ពីការសិក្សាពិភពលោក</h3>
<div class="grid2">
<div class="card"><h4>១. សិស្ស/និស្សិតកម្ពុជា</h4><p>ប្រើ hotspot ទូរស័ព្ទដែលមានកម្រិត data និង laptop ចាស់។ ទាញវគ្គសិក្សា វីដេអូ និង software។ <strong>ត្រូវការ:</strong> ភាសាខ្មែរ, limiter សរុប, resume ពេលអ៊ីនធឺណិតដាច់, ឥតគិតថ្លៃ, ប្រើ RAM តិច។</p></div>
<div class="card"><h4>២. បុគ្គលិកការិយាល័យ</h4><p>នៅក្រោយ proxy ក្រុមហ៊ុន (PAC, NTLM) និង antivirus តឹងរឹង។ <strong>ត្រូវការ:</strong> proxy ប្រព័ន្ធ, CA ក្រុមហ៊ុន, installer ដែលមានហត្ថលេខា, គ្មាន driver។</p></div>
<div class="card"><h4>៣. អ្នកទាញវីដេអូ/មេឌៀ</h4><p>ក្រុមដែលប្រើ IDM ច្រើនជាងគេ (វៀតណាម, ឥណ្ឌូនេស៊ី, អ៊ីរ៉ង់, អារ៉ាប់)។ <strong>ត្រូវការ:</strong> ជ្រើសគុណភាព, ឯកសារ MP4 មានសំឡេង, subtitle, ប៊ូតុងមិនរំខាន។</p></div>
<div class="card"><h4>៤. អ្នកប្រើជំនាញ</h4><p>មាន script ចាស់ដែលហៅ <code>idman /d</code>, ប្រើ queue យប់ និង sync queue។ <strong>ត្រូវការ:</strong> CLI ដែលត្រូវនឹង IDM, scheduler, បិទម៉ាស៊ីនពេលបញ្ចប់, hash verify។</p></div>
<div class="card"><h4>៥. អ្នកប្រើ keyboard/screen reader</h4><p><strong>ត្រូវការ:</strong> keyboard គ្រប់មុខងារ, ឈ្មោះប៊ូតុងពីរភាសា, focus មើលឃើញ។</p></div>
</div>
<h3>គោលការណ៍ UX ៧</h3>
<ol>
<li><strong>មិនរំខាន:</strong> គ្មាន modal ដែលអ្នកប្រើមិនបានស្នើ។ ការជូនដំណឹងប្រើ toast ឬ badge, មិនប្រើ popup។</li>
<li><strong>មិនបាត់ទិន្នន័យ:</strong> មិនសរសេរជាន់ដោយស្ងាត់, មិនលុបឯកសារពេល uninstall, លុបទៅ Recycle Bin។</li>
<li><strong>ពន្យល់ពេលបរាជ័យ:</strong> រាល់កំហុសមានមូលហេតុជាភាសាមនុស្ស និងសកម្មភាពមួយ (ឧ. "Server ផ្ញើទំព័រ HTML ⇒ ប្រហែលត្រូវ login ⇒ [បើកក្នុង browser]")។</li>
<li><strong>Default ល្អ, ជម្រើសនៅជ្រៅ:</strong> ដំឡើងហើយដំណើរការភ្លាម។ ជម្រើស IDM ទាំងអស់នៅក្នុង Settings › កម្រិតខ្ពស់។</li>
<li><strong>ឃើញតែអ្វីដែលពិត:</strong> segment bar ពិត, ល្បឿនពិត, ពេលនៅសល់ដែលបាន smooth (EMA 5 វិនាទី)។</li>
<li><strong>ខ្មែរជាភាសាដំបូង:</strong> អក្សរខ្មែរមិនត្រូវកាត់ជើង, មិនប្រើអត្ថបទបកប្រែដោយម៉ាស៊ីន, ពាក្យបច្ចេកទេសមានសទ្ទានុក្រម (glossary)។</li>
<li><strong>សុវត្ថិភាពដោយមិនបាច់គិត:</strong> MOTW, ការព្រមានឯកសារគ្រោះថ្នាក់ និងគ្មាន auto-open សុទ្ធតែកើតឡើងដោយស្វ័យប្រវត្តិ។</li>
</ol>
<h3>ដំណើរអ្នកប្រើ (journey) ជំហានម្ដងមួយៗ</h3>
<h4>J1 · ដំឡើងលើកដំបូង</h4>
<ol>
<li>ដំឡើងពី Microsoft Store (មានហត្ថលេខា, គ្មានការព្រមាន) ឬទាញ <code>GDM-Setup-x64.exe</code> ពី GitHub Releases / winget។ Installer GitHub គ្មានហត្ថលេខារហូតដល់ D8 ដូច្នេះ SmartScreen នឹងព្រមាន ហើយទំព័រ download ពន្យល់ពីរបៀបផ្ទៀងផ្ទាត់ sha256។</li>
<li>ផ្ទាំងជ្រើសភាសា: <strong>ខ្មែរ</strong> (ជ្រើសរួចជាលំនាំដើម) បន្ទាប់មក English។</li>
<li><strong>ផ្ទាំងអាជ្ញាប័ណ្ណ</strong> (ខ្មែរមុន): សង្ខេប ៤ ចំណុច (ឥតគិតថ្លៃសម្រាប់ការប្រើផ្ទាល់ខ្លួន · ហាមកែប្រែ ឬចែកចាយ · ហាមប្រើពាណិជ្ជកម្ម · ការប្រើផ្សេងទៀតសុំការអនុញ្ញាតពី Golden IQ) + link អត្ថបទពេញ ហើយត្រូវចុច checkbox "ខ្ញុំយល់ព្រម" (LICENSE §12(j))។ កំណែ Store: Microsoft បង្ហាញលក្ខខណ្ឌរបស់ខ្លួន ដូច្នេះ GDM បង្ហាញផ្ទាំងនេះពេលបើកលើកដំបូង។</li>
<li>ប្រភេទដំឡើង: "សម្រាប់ខ្ញុំ" (គ្មាន admin, ជាលំនាំដើម) ឬ "សម្រាប់អ្នកប្រើទាំងអស់"។</li>
<li>ចុចដំឡើង (~10 វិនាទី)។ ផ្ទាំងបញ្ចប់មាន checkbox "បើក GDM"។</li>
</ol>
<h4>J2 · បើកលើកដំបូង (onboarding ៣ ជំហាន, រំលងបាន)</h4>
<ol>
<li><strong>ភាសា និង theme:</strong> បង្ហាញភាសាដែលបានជ្រើសក្នុង installer ហើយអាចប្ដូរបាន។ Theme តាមប្រព័ន្ធ។</li>
<li><strong>ភ្ជាប់ browser:</strong> GDM រកឃើញ browser ដែលបានដំឡើង ហើយបង្ហាញប៊ូតុង "ដំឡើង extension" សម្រាប់ browser នីមួយៗ (បើកទំព័រ store ផ្ទាល់ដោយ ID ពិត)។ ស្ថានភាពប្ដូរជា "បានភ្ជាប់" ភ្លាមនៅពេល extension ផ្ញើ hello។</li>
<li><strong>ជម្រើស ៣ ដែលប៉ះឯកជនភាព/ការរំខាន:</strong> ផ្ទាំងវីដេអូលើគេហទំព័រ (បិទ/បើក), ការតាមដាន clipboard (បិទ/បើក), update (ស្វ័យប្រវត្តិ/ជូនដំណឹង/បិទ)។ ជម្រើសនីមួយៗមានឃ្លាពន្យល់មួយ។</li>
</ol>
<h4>J3 · ចាប់ពី browser (ផ្លូវសំខាន់បំផុត)</h4>
<ol>
<li>អ្នកប្រើចុច link ក្នុង browser។ Extension ពិនិត្យគោលការណ៍ចាប់ (C-01) ហើយផ្ញើ handoff ⇒ engine រក្សា job <em>prepared</em> ⇒ ack ⇒ extension cancel ការទាញក្នុង browser ⇒ រាយការណ៍លទ្ធផល ⇒ engine activate (B-05, B-09)។ ទាំងអស់កើតក្នុង &lt; 300 ms។</li>
<li>ផ្ទាំង <strong>Download Info</strong> លោតលើ monitor ដែល browser នៅ: ឈ្មោះ (កែបាន), ទំហំ, category (ជ្រើសដោយស្វ័យប្រវត្តិ), ថត, ប៊ូតុង [ចាប់ផ្ដើម] [ទាញពេលក្រោយ ▾ queue] [បោះបង់]។ Engine ចាប់ផ្ដើមទាញភ្លាមរួចហើយ (ដូច IDM) ដូច្នេះពេលចុចចាប់ផ្ដើម ការទាញបានដំណើរការមួយភាគរួចទៅហើយ។</li>
<li>Progress បង្ហាញក្នុងបញ្ជីមេ។ បើ window ត្រូវបិទ ⇒ toast តូចនៅជ្រុងអេក្រង់ (ជម្រើស)។ Segment bar និង connection នីមួយៗមើលបានក្នុង details pane។</li>
<li>បញ្ចប់ ⇒ toast "បានទាញ: name.zip [បើក] [បើកថត]"។ Toast នេះអាចអូសឯកសារចេញទៅកម្មវិធីផ្សេងបាន (ដូច IDM complete dialog)។ ឯកសារគ្រោះថ្នាក់មិនមានប៊ូតុង [បើក] ផ្ទាល់ទេ មានតែ [បើកថត]។</li>
</ol>
<h4>J4 · បន្ទាប់ពីបរាជ័យ</h4>
<p>ស្ថានភាពនីមួយៗមានពាក្យ និងសកម្មភាពច្បាស់: <em>ផ្អាក (គ្មានបណ្ដាញ)</em> ⇒ បន្តស្វ័យប្រវត្តិ · <em>ត្រូវការ link ថ្មី</em> ⇒ [Refresh] · <em>Server ប្ដូរឯកសារ</em> ⇒ [ចាប់ផ្ដើមឡើងវិញ] [រក្សាផ្នែកចាស់] · <em>ថតមិនអាចចូលបាន</em> ⇒ រង់ចាំ volume · <em>ត្រូវ antivirus រារាំង</em> ⇒ [ព័ត៌មាន]។ មិនបង្ហាញ "Error 0x..." តែម្នាក់ឯងទេ។ Code កំហុសមានក្នុង details សម្រាប់ផ្ញើ issue។</p>
<h4>J5 · Refresh download address</h4>
<p>ចុចកណ្ដុរស្ដាំ › Refresh ⇒ GDM បើកទំព័រប្រភពក្នុង browser ⇒ banner ក្នុង GDM "កំពុងរង់ចាំ link ថ្មីពីទំព័រនេះ (10:00)" ⇒ អ្នកប្រើចុចទាញម្ដងទៀត ⇒ GDM បង្ហាញ "រកឃើញ: ETag ដូចគ្នា ✓ ⇒ បន្តពី 63%" ⇒ [បន្ត]។ គ្មាន validator ខ្លាំង ⇒ "ទំនងជាឯកសារដដែល (មិនបានផ្ទៀងផ្ទាត់)" ⇒ [ចាប់ផ្ដើមថ្មី] (លំនាំដើម) [បន្ត (ហានិភ័យ)]។ មាន fallback [បិទភ្ជាប់ URL ថ្មី] (R-01)។</p>
<h4>J6 · វីដេអូ</h4>
<p>ផ្លូវ A (extension, គេហទំព័រដែលអនុញ្ញាត): រូបតំណាង extension បង្ហាញលេខមេឌៀ ⇒ popup បញ្ជីគុណភាព ⇒ ជ្រើស។ ផ្លូវ B (app): បិទភ្ជាប់ URL ទំព័រក្នុង [បន្ថែម] ⇒ GDM ពិនិត្យ component yt-dlp (ពេលដំបូង: "ត្រូវទាញ component មេឌៀ 18 MB, យល់ព្រម?") ⇒ បញ្ជីគុណភាព ⇒ MP4 មានសំឡេង។ DRM ⇒ សារច្បាស់ថាមិនគាំទ្រ។</p>
<h4>J7 · Queue និងកាលវិភាគ</h4>
<p>[ទាញពេលក្រោយ ▾] ⇒ ជ្រើស queue ⇒ ផ្ទាំង queue មាន: ចាប់ផ្ដើម/ឈប់ម៉ោង, ថ្ងៃ, ចំនួនឯកសារព្រមគ្នា, limiter, quota និង "ពេលបញ្ចប់: គ្មាន / បិទ GDM / sleep / hibernate / បិទម៉ាស៊ីន / បើកកម្មវិធី"។ សកម្មភាពដែលប៉ះប្រព័ន្ធមាន countdown 60 វិនាទី និងប៊ូតុងបោះបង់ (Q-02)។</p>
<h4>J8 · Update (មើល §07)</h4>
"""

S07 = r"""
<p>គោលដៅ: អ្នកប្រើមិនដែលបាត់ progress និងមិនត្រូវបានរំខាន។ ពេលមាន update សុវត្ថិភាព វាត្រូវដំឡើងនៅឱកាស idle ដំបូងបន្ទាប់ពីទាញរួច (ប្រសិនបើ update ស្វ័យប្រវត្តិត្រូវបានបើក ហើយមានអ៊ីនធឺណិត)។</p>
<h3>ការកំណត់ (Settings › Update)</h3>
<ul>
<li><strong>ស្វ័យប្រវត្តិ</strong> (លំនាំដើម): ពិនិត្យ 1 ដងក្នុងមួយថ្ងៃ និងពេលបើក app (មិនលឿនជាង 6 ម៉ោងម្ដង)។ ទាញនៅខាងក្រោយ ហើយដំឡើងពេល idle ឬពេលបិទ app។</li>
<li><strong>ជូនដំណឹង:</strong> ពិនិត្យ និងបង្ហាញ badge, តែមិនទាញ។</li>
<li><strong>បិទ:</strong> មិនពិនិត្យទាល់តែសោះ (គ្មាន request ទៅ GitHub)។ ប៊ូតុង "ពិនិត្យឥឡូវ" ក្នុង About នៅតែមាន។</li>
<li>Channel: Stable (លំនាំដើម) / Beta។ ផ្អាកការទាញ update លើបណ្ដាញ metered (លំនាំដើម)។</li>
<li><strong>កំណែ Store (MSIX):</strong> Microsoft Store ជាអ្នក update។ Updater ក្នុង app ត្រូវបិទ ហើយ Settings › Update បង្ហាញតែ "Update តាម Microsoft Store"។ State machine ខាងក្រោមអនុវត្តចំពោះកំណែ GitHub ប៉ុណ្ណោះ (component yt-dlp/ffmpeg update ដូចគ្នាទាំងពីរ)។</li>
</ul>
<h3>ម៉ាស៊ីនស្ថានភាព (state machine)</h3>
<div class="tw"><table><thead><tr><th>ស្ថានភាព</th><th>អ្វីដែលអ្នកប្រើឃើញ</th><th>ការផ្លាស់ប្ដូរ</th></tr></thead><tbody>
<tr><td>Idle</td><td>គ្មានអ្វីសោះ</td><td>Timer ឬការចាប់ផ្ដើម ⇒ Checking</td></tr>
<tr><td>Checking</td><td>គ្មានអ្វីសោះ (ស្ងាត់)</td><td>មិនមាន ⇒ Idle · មាន ⇒ Verify manifest (សញ្ញា, version &gt; ធំបំផុតដែលធ្លាប់ឃើញ, មិនទាន់ផុតកំណត់) ⇒ Downloading · បរាជ័យ ⇒ Idle + log (ព្យាយាមម្ដងទៀតក្នុង 6 ម៉ោង)</td></tr>
<tr><td>Downloading</td><td>ចំណុចតូចលើរូបតំណាង Settings</td><td>Limiter 20% នៃបណ្ដាញ, ផ្អាកពេល metered ⇒ Verifying</td></tr>
<tr><td>Verifying</td><td>—</td><td>sha256 + minisign + Authenticode ⇒ Ready · បរាជ័យ ⇒ លុបឯកសារ, Idle, log "update ត្រូវបដិសេធ"</td></tr>
<tr><td>Ready</td><td>ប៊ូតុងតូចក្នុង sidebar "Update 1.4.0 រួចរាល់ · ពេលបិទ GDM" (ចុចមើល changelog ពីរភាសា)</td><td>Job សកម្ម = 0 និង idle 2 នាទី ⇒ ដំឡើងស្ងាត់ (បើកជម្រើស "ដំឡើងពេល idle") · អ្នកប្រើចុច "Restart ឥឡូវ" ⇒ Installing · បិទ app ⇒ Installing</td></tr>
<tr><td>Installing</td><td>Toast "កំពុង update…" (~5 វិនាទី)</td><td>Engine checkpoint ⇒ បិទ ⇒ NSIS passive ⇒ បើកវិញ ⇒ resume job ដែលបានផ្អាក ⇒ toast "បាន update ទៅ 1.4.0 [មើលអ្វីថ្មី]"</td></tr>
<tr><td>Failed install</td><td>បើកកំណែចាស់ធម្មតា + banner "Update បរាជ័យ [ព្យាយាមម្ដងទៀត] [ទាញពី GitHub]"</td><td>ក្រោយ 2 ដង ⇒ ផ្អាក auto, ទុកឱ្យអ្នកប្រើសម្រេច</td></tr>
</tbody></table></div>
<h3>ច្បាប់មិនត្រូវបំពាន</h3>
<ul>
<li>មិនដែល restart ពេលមាន job សកម្ម, mux, hash, ឬ grabber កំពុងរត់ (U-02)។ ការបិទ window មិនមែនជាការបិទ app ទេ (app នៅក្នុង tray)។ "ពេលបិទ GDM" មានន័យថាចេញពី tray ឬ shutdown។</li>
<li>Update សុវត្ថិភាព (<code>"security": true</code> ក្នុង manifest): ប៊ូតុងប្ដូរពណ៌ (fill មិនមែន glow) និងមានឃ្លា "មានការជួសជុលសុវត្ថិភាព"។ នៅតែមិនបង្ខំ restart។</li>
<li>Component (yt-dlp, ffmpeg) update ដាច់ពី app តាម manifest ដែល GDM ចុះហត្ថលេខា។ វាអាច update ពេលមិនមានការទាញវីដេអូ ដោយមិនបាច់ restart app។</li>
<li>Downgrade ត្រូវបានបដិសេធ។ បើអ្នកប្រើចង់ត្រឡប់ទៅកំណែចាស់ ⇒ ដំឡើងដោយដៃពី GitHub ⇒ DB ធ្វើ migration ចុះក្រោម ឬបើកជា read-only (D-02)។</li>
<li>Changelog ពីរភាសា ខ្លី និងសរសេរពីទស្សនៈអ្នកប្រើ (មិនមែនបញ្ជី commit)។</li>
</ul>
"""

S08 = r"""
<figure><svg viewBox="0 0 900 420" role="img" aria-label="ស្ថាបត្យកម្ម GDM">
<style>.b{fill:var(--surface2)}.t{fill:var(--fg);font:14px var(--f-body)}.s{fill:var(--muted);font:12px var(--f-body)}.l{stroke:var(--accent);stroke-width:2;fill:none}.d{stroke:var(--muted);stroke-width:1.5;stroke-dasharray:5 5;fill:none}</style>
<rect class="b" x="20" y="30" width="200" height="110" rx="12"/><text class="t" x="36" y="58">Browser</text><text class="s" x="36" y="82">MV3 extension (Chrome/Edge)</text><text class="s" x="36" y="102">Firefox extension</text><text class="s" x="36" y="122">គ្មានសិទ្ធិ path/hook</text>
<rect class="b" x="270" y="30" width="170" height="110" rx="12"/><text class="t" x="286" y="58">gdm-bridge.exe</text><text class="s" x="286" y="82">native messaging host</text><text class="s" x="286" y="102">ពិនិត្យ origin, ទំហំ</text><text class="s" x="286" y="122">stateless</text>
<rect class="b" x="490" y="20" width="390" height="250" rx="12"/><text class="t" x="506" y="48">gdm.exe (Tauri v2, per-user, unelevated)</text>
<rect x="506" y="62" width="170" height="90" rx="10" fill="var(--bg)"/><text class="t" x="520" y="88">UI (React)</text><text class="s" x="520" y="110">WebView2, CSP តឹងរឹង</text><text class="s" x="520" y="130">command យក ID</text>
<rect x="694" y="62" width="170" height="190" rx="10" fill="var(--bg)"/><text class="t" x="708" y="88">gdm-core (Rust)</text><text class="s" x="708" y="110">scheduler · segmenter</text><text class="s" x="708" y="130">network policy</text><text class="s" x="708" y="150">file writer · MOTW</text><text class="s" x="708" y="170">updater · components</text><text class="s" x="708" y="190">pipe server (DACL)</text><text class="s" x="708" y="230">ឯករាជ្យពី Tauri</text>
<rect x="506" y="170" width="170" height="82" rx="10" fill="var(--bg)"/><text class="t" x="520" y="196">SQLite (WAL)</text><text class="s" x="520" y="218">jobs · segments · history</text><text class="s" x="520" y="238">backup ប្រចាំថ្ងៃ</text>
<rect class="b" x="490" y="300" width="180" height="100" rx="12"/><text class="t" x="506" y="328">Components</text><text class="s" x="506" y="350">yt-dlp · ffmpeg</text><text class="s" x="506" y="370">sha256 pin, argv, Job Object</text>
<rect class="b" x="700" y="300" width="180" height="100" rx="12"/><text class="t" x="716" y="328">Windows</text><text class="s" x="716" y="350">IAttachmentExecute (AV)</text><text class="s" x="716" y="370">Credential Manager</text>
<rect class="b" x="20" y="300" width="200" height="100" rx="12"/><text class="t" x="36" y="328">CLI: gdm.exe /d …</text><text class="s" x="36" y="350">ផ្ញើតាម pipe ដូចគ្នា</text><text class="s" x="36" y="370">flag IDM ទាំង 8</text>
<rect class="b" x="270" y="300" width="170" height="100" rx="12"/><text class="t" x="286" y="328">GitHub Releases</text><text class="s" x="286" y="350">latest.json + sig</text><text class="s" x="286" y="370">components.json</text>
<path class="l" d="M220 85 H270"/><path class="l" d="M440 85 H490"/><path class="l" d="M676 107 H694"/><path class="l" d="M676 211 H694"/>
<path class="l" d="M220 350 C 360 280, 600 290, 760 252"/><path class="d" d="M440 350 H490"/><path class="d" d="M440 330 C 600 270, 700 270, 760 252"/><path class="l" d="M670 350 H700"/>
</svg><figcaption>ព្រំដែនទុកចិត្ត: Browser ↔ bridge (native messaging) · bridge ↔ core (named pipe ផ្ទៀងផ្ទាត់) · UI ↔ core (Tauri IPC, capability) · core ↔ Internet (network policy) · core ↔ components (process ដាច់)។</figcaption></figure>
<h3>សមាសធាតុ</h3>
<div class="tw"><table><thead><tr><th>ផ្នែក</th><th>បច្ចេកវិទ្យា</th><th>ហេតុផល</th></tr></thead><tbody>
<tr><td>gdm-core</td><td>Rust, tokio, reqwest (hyper 1, HTTP/1.1 + HTTP/2), rustls + rustls-platform-verifier, suppaftp (FTP/FTPS), url, rusqlite (bundled, WAL), keyring, windows-rs</td><td>Crate ឯករាជ្យពី Tauri (ChatGPT ជុំ 2): អាចបំបែកជា <code>gdm-engine.exe</code> នៅពេលក្រោយបាន ដោយមិនចាំបាច់សរសេរឡើងវិញ</td></tr>
<tr><td>gdm.exe</td><td>Tauri v2, plugins: updater, single-instance, notification, autostart, window-state។ <strong>មិនប្រើ</strong> plugin shell/fs/http</td><td>Capability តិចបំផុត (D3, X-02)</td></tr>
<tr><td>UI</td><td>React 19 + TypeScript + Vite, i18next (km, en), TanStack Virtual (បញ្ជីធំ), Zustand (state), CSS variables (មិនប្រើ UI kit)</td><td>ស្រាល, ងាយគ្រប់គ្រង token តាម Mazer Client</td></tr>
<tr><td>gdm-bridge.exe</td><td>Rust ~400 បន្ទាត់, គ្មាន async runtime ធំ</td><td>Binary តូច (ផ្ទៃវាយប្រហារតូច), ចុះហត្ថលេខា</td></tr>
<tr><td>Extension</td><td>TypeScript, MV3 (Chrome/Edge), MV3 Firefox (webRequest អាចប្រើបាន), build តែមួយ (WXT ឬ Vite)</td><td>Code តែមួយ ចេញ build តាម store; build Chrome មិនរត់លើ YouTube (M-07)</td></tr>
<tr><td>Installer</td><td>NSIS (Tauri bundler) + Khmer.nsh, per-user លំនាំដើម (GitHub); MSIX (Microsoft Store)</td><td>WiX/MSI មិនគាំទ្រខ្មែរ</td></tr>
</tbody></table></div>
<h3>ការសម្រេចស្ថាបត្យកម្ម (ADR សង្ខេប)</h3>
<ol>
<li><strong>ADR-1 · Process តែមួយសម្រាប់ MVP:</strong> engine រត់ក្នុង gdm.exe នៅលើ tokio runtime ផ្ទាល់ខ្លួន។ ការបិទ window មិនបិទ app។ <em>ច្រកឆ្លង:</em> នៅ Phase 2 ធ្វើ failure-injection test (crash WebView, crash core)។ បើ WebView2 crash ធ្វើឱ្យការទាញយកឈប់ ⇒ បំបែកជា <code>gdm-engine.exe</code> ដែលចាប់ផ្ដើមពេលត្រូវការ (មិនមែន service)។ គ្មាន Windows service ឡើយ ព្រោះ service រត់ជា SYSTEM ហើយធំជាងតម្រូវការ។</li>
<li><strong>ADR-2 · គ្មាន port localhost:</strong> browser ⇒ native messaging ⇒ named pipe។ <code>gdm://</code> URL scheme ត្រូវពន្យារទៅ Phase 3។ បើធ្វើ វាមានទម្រង់កំណត់: ទទួលតែ URL http(s) ហើយបើកផ្ទាំង "បន្ថែម" ឱ្យអ្នកប្រើបញ្ជាក់ ដោយគ្មាន path, cookie ឬ auto-start។</li>
<li><strong>ADR-3 · ឯកសារ .part ក្នុងថតគោលដៅ:</strong> .part មានតែ payload ឆៅ។ Metadata សម្រាប់ស្ដារស្ថិតក្នុង sidecar <code>name.part.gdm</code> (≤ 64 KB, untrusted)។ មានជម្រើស "ថតបណ្ដោះអាសន្ន" សម្រាប់អ្នកដែលចង់បានដូច IDM (បិទលំនាំដើម)។</li>
<li><strong>ADR-4 · មេឌៀតាម component:</strong> yt-dlp/ffmpeg មិនភ្ជាប់ក្នុង installer។ អ្នកប្រើទាញវាតាមការយល់ព្រម ហើយ GDM ផ្ទៀងផ្ទាត់ hash ដែលចុះហត្ថលេខា។ HLS ធម្មតា (មិនអ៊ិនគ្រីប ឬ AES-128) នឹងធ្វើក្នុង Rust ផ្ទាល់នៅ Phase 3 ដើម្បីកាត់បន្ថយការពឹងលើ component។</li>
<li><strong>ADR-6 · Component ក្រោមគោលការណ៍ GDM:</strong> yt-dlp រត់ជា extract-only (<code>-J</code>) តាម policy proxy ក្នុង GDM (loopback, port ចៃដន្យ, Proxy-Authorization ចៃដន្យក្នុងមួយ run)។ Engine GDM ទាញ stream ដោយខ្លួនឯង។ ffmpeg ធ្វើតែ mux ឯកសារក្នុងម៉ាស៊ីន (<code>-protocol_whitelist file,pipe</code>)។ ប្រតិបត្តិការណាដែលមិនអាចដាក់ក្រោមគោលការណ៍នេះបាន ⇒ មិនគាំទ្រ (M-08)។ Proxy នេះមិនមែនជា API ទេ ព្រោះវាមិនមានពាក្យបញ្ជា ហើយបិទពេល run បញ្ចប់។</li>
<li><strong>ADR-5 · Licence កម្មសិទ្ធិ source-available (ម្ចាស់សម្រេច):</strong> <code>LICENSE</code> អនុញ្ញាតតែការប្រើផ្ទាល់ខ្លួន និងការអានកូដ។ ការកែប្រែ ចែកចាយ និងប្រើពាណិជ្ជកម្មត្រូវការការអនុញ្ញាតជាលាយលក្ខណ៍អក្សរពី Golden IQ។ Dependency: licence permissive (MIT/Apache-2.0/BSD/ISC/Zlib/Unicode/OFL) + <strong>MPL-2.0 ដែលមិនកែប្រែ</strong> ជាករណីលើកលែង (D18) ព្រោះ Tauri ខ្លួនឯងប្រើ crate MPL (<code>cssparser</code>, <code>selectors</code>; ត្រូវផ្ទៀងផ្ទាត់ដោយ <code>cargo deny</code> ក្នុង P0)។ MPL ជា copyleft កម្រិតឯកសារ: ត្រូវផ្ដល់ source នៃឯកសារ MPL ទាំងនោះ (link crates.io + notice)។ ហាម GPL/AGPL/LGPL ដែល link ផ្ទាល់។ yt-dlp (Unlicense; binary មាន GPLv3+) និង ffmpeg ត្រូវទាញពី upstream ហើយរត់ជា process ដាច់។ ការបំបែក process មិនមែនជាការលើកលែង copyleft ជាសកលទេ ដូច្នេះមេធាវីត្រូវពិនិត្យ។</li>
</ol>
<h3>ទិន្នន័យ (SQLite)</h3>
""" + code(r"""
jobs(id TEXT PK, url TEXT, final_url TEXT, referrer TEXT, name TEXT, dir_id INT, category_id INT,
     queue_id INT NULL, state TEXT, size INT NULL, done INT, etag TEXT NULL, last_modified TEXT NULL,
     validator_kind TEXT, sha256_expected TEXT NULL, created_at INT, finished_at INT NULL,
     error_code TEXT NULL, flags INT)                  -- flags: incognito, from_browser, media
segments(job_id TEXT, start INT, end_incl INT, written INT, PRIMARY KEY(job_id,start))
names(dir_id INT, name_ci TEXT, job_id TEXT, UNIQUE(dir_id,name_ci))   -- N-04 reserve
queues(id, name, kind 'download'|'sync', max_parallel, schedule_json, limit_bps, quota_json, on_done)
categories(id, name_km, name_en, dir, exts, site_patterns)
host_profiles(host TEXT PK, max_conn INT, no_range INT, last_429 INT)
refresh_tickets(id TEXT PK, job_id TEXT, tab_hint TEXT, expires_at INT)
history_fts USING fts5(name, url, description, content='jobs')
""")

S09 = r"""
<h3>Dynamic segmentation (ល្អជាង IDM)</h3>
<ol>
<li><strong>Probe:</strong> GET <code>Range: bytes=0-</code> ជាមួយ <code>Accept-Encoding: identity</code>។ ឆ្លើយ 206 + Content-Range មាន total ⇒ គាំទ្រ range។ ឆ្លើយ 200 ⇒ មួយ connection។ រក្សា ETag ខ្លាំង ឬ Last-Modified។ Connection probe នេះបន្តទាញពី byte 0 (មិនខាត request)។</li>
<li><strong>ចាប់ផ្ដើម 2 connection:</strong> ផ្នែកទី 2 ចាប់ពីពាក់កណ្ដាល។ រៀងរាល់ 2 វិនាទី: បើល្បឿនសរុបកើន ≥ 10% ពីការបន្ថែមចុងក្រោយ ហើយ connection &lt; កម្រិត (លំនាំដើម 8, អតិបរមា 32, per-host profile) ⇒ បន្ថែមមួយ។ បើមិនកើន ⇒ ឈប់បន្ថែម (ការពារ server និង router)។</li>
<li><strong>Split:</strong> ពេល connection មួយទំនេរ ⇒ actor យកផ្នែកដែលនៅសល់ធំបំផុត ហើយកាត់ពាក់កណ្ដាល (ពាក់កណ្ដាលចុង ≥ 1 MiB)។ Actor តែមួយជាម្ចាស់ segment map (E-06)។</li>
<li><strong>Write lease:</strong> worker នីមួយៗកាន់ lease [start,end) មាន generation។ Split = actor បង្រួម lease ហើយរង់ចាំ ack ព្រំដែន មុនផ្ដល់ផ្នែកថ្មី (E-06)។</li>
<li><strong>Seam check (ការប៉ាន់ស្មាន):</strong> connection ថ្មីចាប់ផ្ដើមមុនព្រំដែន 4 KiB ហើយប្រៀបធៀប byte ដែលជាន់គ្នា។ មិនត្រូវ ⇒ ឯកសារប្ដូរ ⇒ ផ្អាក និងប្រាប់។ ត្រូវ ⇒ មិនមែនជាភស្តុតាងថាឯកសារទាំងមូលដដែលទេ។ ការធានាពិតមកពី validator ខ្លាំង ឬ hash (E-02)។</li>
<li><strong>សរសេរ:</strong> <code>seek_write</code> ទៅ .part ដែលបាន preallocate (<code>set_len</code>)។ Buffer 1 MiB ក្នុង connection។ Flush (8 MiB ឬ 2 វិនាទី) ⇒ ទើប commit offset (E-04)។</li>
<li><strong>Limiter:</strong> token bucket ៣ កម្រិត (សរុប → queue → job) ដែលចែកស្មើតាម connection។</li>
<li><strong>បញ្ចប់:</strong> ពិនិត្យ byte = total ⇒ SHA-256 (បើស្នើ ឬមាន <code>.sha256</code> ពី origin ដដែល) ⇒ IAttachmentExecute (MOTW + AV): បដិសេធ ⇒ "ត្រូវបានរារាំង"; tag មិនបាន ⇒ "គោលដៅមិនការពារ" (N-05, N-09) ⇒ rename ដោយ handle (no-replace) ⇒ Complete ⇒ toast។</li>
</ol>
<h3>ការដោះស្រាយចម្លើយពី server</h3>
<div class="tw"><table><thead><tr><th>ចម្លើយ</th><th>សកម្មភាព</th></tr></thead><tbody>
<tr><td>206 ត្រូវ</td><td>សរសេរ</td></tr>
<tr><td>200 លើ range</td><td>បញ្ឈប់ worker ទាំងអស់, សម្គាល់ host no-range, ចាប់ផ្ដើម stream តាមលំដាប់ពី 0 (E-01)</td></tr>
<tr><td>200 ក្រោម If-Range</td><td>ឯកសារប្ដូរ ⇒ ផ្អាក ហើយសួរ</td></tr>
<tr><td>416</td><td>ទំហំប្ដូរ ⇒ ផ្អាក ហើយសួរ</td></tr>
<tr><td>401/403</td><td>"ត្រូវការ login ឬ link ផុតកំណត់" ⇒ Refresh</td></tr>
<tr><td>404/410</td><td>"ឯកសារលែងមាន" ⇒ Refresh ឬលុប</td></tr>
<tr><td>429/503</td><td>Retry-After, កាត់ connection ពាក់កណ្ដាល</td></tr>
<tr><td>5xx ផ្សេង</td><td>Backoff 1→60 វិនាទី, 10 ដង, បន្ទាប់មកផ្អាក</td></tr>
<tr><td>text/html តូចជំនួស binary</td><td>"Server ផ្ញើទំព័រ HTML" (R-03)</td></tr>
<tr><td>multipart/byteranges</td><td>បដិសេធ (មិនបានស្នើ multi-range)</td></tr>
</tbody></table></div>
<h3>គោលដៅល្បឿន (វាស់ក្នុង Phase 1)</h3>
<ul><li>ឯកសារ 1 GB លើ fixture LAN 1 Gbps ⇒ ≥ 90% នៃ line rate។ CPU &lt; 10% (4 core), RAM engine &lt; 60 MB។</li><li>ប្រៀបធៀបជាមួយ IDM 6.43 លើ URL សាធារណៈ ៥ (CDN ផ្សេងគ្នា) ⇒ ល្បឿនមធ្យម ≥ IDM −5%។</li><li>ពេលចាប់ផ្ដើម app &lt; 1 វិនាទី (cold start ពី SSD)។</li></ul>
"""

S10 = r"""
<h3>ទ្រព្យសម្បត្តិដែលត្រូវការពារ (តាមលំដាប់)</h3>
<ol><li>ម៉ាស៊ីនអ្នកប្រើ (code execution, persistence)</li><li>Cookie/session browser ដែលឆ្លងកាត់ handoff</li><li>ឯកសារអ្នកប្រើ (ការសរសេរជាន់, ការលុប)</li><li>ឯកសារដែលទាញ (integrity: មិនខូចដោយស្ងាត់)</li><li>ពាក្យសម្ងាត់ site/proxy</li><li>ឯកជនភាព: ប្រវត្តិ URL និង clipboard</li><li>Release channel និងកេរ្តិ៍ឈ្មោះ GoldenIQ</li></ol>
<h3>អ្នកវាយប្រហារ</h3>
<div class="tw"><table><thead><tr><th>អ្នកវាយ</th><th>សមត្ថភាព</th><th>ក្នុងវិសាលភាព?</th></tr></thead><tbody>
<tr><td>A1 Server/គេហទំព័រអាក្រក់</td><td>គ្រប់គ្រង header, body, redirect, ឈ្មោះ, manifest</td><td>បាទ (ចម្បង)</td></tr>
<tr><td>A2 បណ្ដាញ (Wi-Fi សាធារណៈ)</td><td>MITM, DNS</td><td>បាទ</td></tr>
<tr><td>A3 Extension ផ្សេង ឬ extension យើងត្រូវប្លន់</td><td>ផ្ញើ message ទៅ host</td><td>បាទ</td></tr>
<tr><td>A4 Process ក្នុង user ដដែល</td><td>អានឯកសារ user, ក្លែង pipe, ហៅ CLI, ប្ដូរឯកសារ</td><td>មួយផ្នែក: កាត់បន្ថយ pipe squatting និងការប្ដូរឯកសារធម្មតា ហើយមិនឱ្យ GDM ក្លាយជាជំហានឡើងសិទ្ធិ។ <strong>មិនអះអាងថាផ្ទៀងផ្ទាត់អត្តសញ្ញាណបានប្រឆាំង malware ក្នុង user ដដែលទេ</strong></td></tr>
<tr><td>A5 Supply chain</td><td>Dependency, CI, store account, website</td><td>បាទ</td></tr>
<tr><td>A6 អ្នកប្រើដែលត្រូវបោក</td><td>ចុច link phishing</td><td>បាទ (UI មិនបោក)</td></tr>
<tr><td>A7 Admin/SYSTEM malware</td><td>គ្រប់យ៉ាង</td><td>ទេ</td></tr>
</tbody></table></div>
<h3>STRIDE តាមព្រំដែន</h3>
<div class="tw"><table><thead><tr><th>ព្រំដែន</th><th>S</th><th>T</th><th>R</th><th>I</th><th>D</th><th>E</th></tr></thead><tbody>
<tr><td>Extension → bridge</td><td>B-01 B-08</td><td>B-04</td><td>log handoff</td><td>C-05</td><td>B-03</td><td>B-01</td></tr>
<tr><td>Bridge → core pipe</td><td>B-02</td><td>B-02</td><td>—</td><td>B-02</td><td>B-03</td><td>—</td></tr>
<tr><td>UI → core (IPC)</td><td>X-01</td><td>X-01</td><td>—</td><td>—</td><td>D-03</td><td>X-02 Q-01</td></tr>
<tr><td>Core → Internet</td><td>W-03</td><td>E-01 E-02</td><td>—</td><td>W-01 P-02</td><td>E-03 E-07</td><td>W-02</td></tr>
<tr><td>Core → filesystem</td><td>N-02</td><td>N-03 N-04</td><td>—</td><td>—</td><td>E-03</td><td>N-01 N-05</td></tr>
<tr><td>Core → components</td><td>M-05</td><td>M-02</td><td>—</td><td>—</td><td>M-03</td><td>M-01</td></tr>
<tr><td>Release → user</td><td>S-01 S-04</td><td>U-01</td><td>attestation</td><td>U-05</td><td>U-03</td><td>T-01</td></tr>
</tbody></table></div>
<h3>ច្បាប់ invariant ១២ (មិនត្រូវបំពានក្នុងកូដណាក៏ដោយ)</h3>
<ol>
<li>គ្មានការសរសេរឯកសារក្រៅថតដែលអ្នកប្រើជ្រើស ឬថត category។ ករណីលើកលែង: ថតដែល GDM ជាម្ចាស់ក្នុង <code>%LOCALAPPDATA%\GDM\</code> (DB, update staging, components, <code>private\</code> សម្រាប់ incognito)។</li>
<li>គ្មាន shell សម្រាប់ input ដែលមិនគួរទុកចិត្ត។ Process ខាងក្រៅរត់តែតាម argv។ ករណីលើកលែងតែមួយ: hook .bat ដែលអ្នកប្រើកំណត់ផ្ទាល់ (D15) មាន argument restriction។</li>
<li>គ្មាន TLS bypass និងគ្មាន https→http។</li>
<li>Credential មិនឆ្លងទៅ origin ផ្សេង។</li>
<li>Byte មិនត្រូវសរសេរ ប្រសិនបើ status/Content-Range/validator មិនត្រូវ។</li>
<li>Progress ដែលកត់ត្រា ≤ data ដែល flush រួច។</li>
<li>គ្មានការសរសេរជាន់ដោយស្ងាត់ និងគ្មានការលុបឯកសារអ្នកប្រើ (ទៅ Recycle Bin)។ ករណីលើកលែង: ឯកសារ incognito ដែលមិនទាន់បញ្ចប់ក្នុង <code>private\</code> ត្រូវលុបជាអចិន្ត្រៃយ៍ (C-07)។</li>
<li>គ្រប់ឯកសារបញ្ចប់មាន MOTW ឬមានស្ថានភាព "គោលដៅមិនការពារ" ដែលច្បាស់ និងគ្មានប៊ូតុងបើកផ្ទាល់។ ឯកសារដែល AV បដិសេធមិនដែល Complete។</li>
<li>គ្មាន auto-open ឯកសារប្រភេទគ្រោះថ្នាក់។</li>
<li>Extension, CLI និង import មិនអាចកំណត់ hook, setting ឬ path គ្រោះថ្នាក់បានទេ។</li>
<li>Update app ត្រូវបានផ្ទៀងផ្ទាត់ដោយ minisign (កំណែ GitHub) ឬដោយ Store (កំណែ MSIX)។ Component ត្រូវផ្ទៀងផ្ទាត់ hash ពី manifest ដែលមានហត្ថលេខា មុនរត់។ Authenticode ត្រូវផ្ទៀងផ្ទាត់ពេលមាន certificate (D8)។ Installer GitHub ដែលគ្មានហត្ថលេខា ជាហានិភ័យដែលបានកត់ត្រា (sha256 + attestation)។</li>
<li>មិនមានការផ្ញើទិន្នន័យចេញពីម៉ាស៊ីន លើកលែងតែ: request ទាញយក (រួមទាំង HEAD, redirect, .sha256, refresh) ដែលអ្នកប្រើចាប់ផ្ដើម, extractor យ៉ាងដែលអ្នកប្រើស្នើ, update check (បិទបាន), ការទាញ component (យល់ព្រម), និង crash report តែពេលអ្នកប្រើចុចផ្ញើ បន្ទាប់ពីបានមើល preview។</li>
</ol>
"""

S11 = r"""
<p>Reviewer ត្រូវរត់សេនារីយ៉ូ S1–S15 លើមុខងារនីមួយៗ។ លេខក្នុងក្រឡាគឺចំនួនកាតក្នុង §12 ដែលសេនារីយ៉ូនោះរកឃើញ។ សញ្ញា "·" មិនមានន័យថាមានសុវត្ថិភាពទេ គ្រាន់តែមិនទាន់មានកាត ហើយត្រូវរត់ម្ដងទៀតនៅពេលមានកូដពិត។ តារាងនេះរាប់ដោយស្វ័យប្រវត្តិពីកាត ដូច្នេះលេខមិនអាចខុសពីកាតបានទេ។</p>
{{MATRIX}}
"""

S12 = r"""
<p>{{NSCEN}} កាត: critical {{N_CRITICAL}} · high {{N_HIGH}} · medium {{N_MEDIUM}} · low {{N_LOW}}។ កាតនីមួយៗក្លាយជា test ក្នុង repo (ឈ្មោះ test = ID កាត) មុនពេលមុខងារនោះត្រូវចាត់ទុកថារួចរាល់។ ចុចលើកាតដើម្បីមើលលម្អិត។</p>
{{CARDS}}
"""

S14 = r"""
<div class="tw"><table><thead><tr><th>ផ្នែក</th><th>ក្បួន/ច្បាប់</th><th>អ្វីដែល GDM ធ្វើ</th></tr></thead><tbody>
<tr><td>Licence កូដ</td><td>អាជ្ញាប័ណ្ណកម្មសិទ្ធិ source-available v1.0 (<code>LICENSE</code>); ច្បាប់កម្ពុជាស្ដីពីសិទ្ធិអ្នកនិពន្ធ 2003; អនុសញ្ញាប៊ែន (កម្ពុជាចាប់ពី 9 មីនា 2022); TRIPS; GitHub ToS D.5 (fork ក្នុង GitHub មិនអាចហាមបាន)</td><td>រក្សាសិទ្ធិ Golden IQ និង CHHIT Sungchhay; header <code>SPDX-License-Identifier: LicenseRef-GDM-SAL-1.0</code> ក្នុងឯកសារកូដ; <code>cargo deny</code> ហាម GPL/AGPL/LGPL/SSPL ដែល link ផ្ទាល់; <code>THIRD-PARTY-NOTICES</code> បង្កើតដោយ <code>cargo about</code> + license-checker; <code>cargo deny</code> ហាម licence មិនត្រូវគ្នា</td></tr>
<tr><td>Component</td><td>yt-dlp (Unlicense; binary មាន GPLv3+), ffmpeg (LGPL-2.1+/GPL តាម build)</td><td>GDM ទាញ binary ពី release របស់ upstream (URL + sha256 ក្នុង manifest ចុះហត្ថលេខា) ដូច្នេះ GDM មិនចែកចាយ binary ទាំងនោះទេ។ yt-dlp: <code>github.com/yt-dlp/yt-dlp/releases</code>។ FFmpeg មិនចេញ binary Windows ផ្លូវការទេ (មានតែ source)។ ដូច្នេះជ្រើស <strong>BtbN FFmpeg-Builds, វ៉ារ្យ៉ង់ <code>win64-lgpl</code></strong> (GitHub, ffmpeg.org ផ្ដល់ link) ហើយត្រូវ audit configure flags។ បើថ្ងៃក្រោយ mirror ⇒ ត្រូវភ្ជាប់ Corresponding Source តាមវិធីដែល GPL/LGPL កំណត់។ កត់ build និងកំណែពិតប្រាកដក្នុង THIRD-PARTY-NOTICES។ លក្ខខណ្ឌគេហទំព័រ (ToS) ត្រូវពិនិត្យដាច់ដោយឡែក: component opt-in មិនផ្ដល់សិទ្ធិ copyright ទេ</td></tr>
<tr><td>Copyright និង DRM</td><td>DMCA 17 U.S.C. §1201; WIPO Copyright Treaty Art. 11; ច្បាប់កម្ពុជាស្ដីពីសិទ្ធិអ្នកនិពន្ធ និងសិទ្ធិប្រហាក់ប្រហែល (2003)</td><td>គ្មានការរំលង DRM; សារក្នុង app; Terms: អ្នកប្រើទទួលខុសត្រូវលើមាតិកាដែលទាញ</td></tr>
<tr><td>Platform policy</td><td>Chrome Web Store Program Policies; Edge Add-ons, AMO policies; YouTube ToS</td><td>Build Chrome មិនរត់លើ YouTube (M-07); មិនសន្យាការអនុម័ត; extension សុំ permission តិចបំផុត</td></tr>
<tr><td>ឯកជនភាព</td><td>GDPR Art. 5, 6, 25 (អនុវត្តពេលផ្ដល់ជូនអ្នកប្រើ EU); ePrivacy Art. 5(3); ច្បាប់កម្ពុជាស្ដីពីការការពារទិន្នន័យផ្ទាល់ខ្លួន (សេចក្ដីព្រាងចុងក្រោយ 23 មិថុនា 2025; ស្ថានភាពវិច្ឆិកា 2026 មិនទាន់ផ្ទៀងផ្ទាត់); ច្បាប់ពាណិជ្ជកម្មអេឡិចត្រូនិក 2019</td><td>គ្មាន telemetry, គ្មានគណនី; Privacy policy ពីរភាសា (តម្រូវដោយ Microsoft Store)</td></tr>
<tr><td>ពាណិជ្ជសញ្ញា</td><td>ការប្រើឈ្មោះអ្នកដទៃតាមការពិត (nominative fair use)</td><td>ប្រើឈ្មោះ IDM/NDM តែក្នុងតារាងប្រៀបធៀបដែលពិត; មិនចម្លង icon/UI របស់គេ; ពិនិត្យឈ្មោះ "GDM" (ជាន់ GNOME Display Manager: D12)</td></tr>
<tr><td>Logo ដែលបង្កើតដោយ AI</td><td>US Copyright Office: ផ្នែកដែល AI បង្កើតសុទ្ធមិនអាចចុះ copyright បាន (Thaler v. Perlmutter, 2025)</td><td>ការពារម៉ាកតាម trademark និងការកែប្រែដោយមនុស្ស (vector redraw) មិនពឹងលើ copyright នៃរូប AI (§16)</td></tr>
<tr><td>សុវត្ថិភាពកម្មវិធី</td><td>OWASP ASVS 5.0, OWASP Top 10:2025, CWE Top 25, NIST SSDF SP 800-218, Microsoft SDL</td><td>Threat model §10, កាត §12 ភ្ជាប់ CWE, gate ក្នុង CI</td></tr>
<tr><td>Supply chain</td><td>SLSA v1.0 Build L2→L3, OpenSSF Scorecard, Sigstore attestations</td><td>§15</td></tr>
<tr><td>HTTP</td><td>RFC 9110 (semantics, Range, If-Range), RFC 9112, RFC 6265 (cookie), RFC 6266 + 8187 (Content-Disposition), RFC 6585 (429), RFC 9530 (Digest), RFC 8996 (TLS 1.0/1.1 deprecated)</td><td>§09, §13</td></tr>
<tr><td>A11y និងភាសា</td><td>WCAG 2.2 AA, EN 301 549; Unicode Khmer (UAX #14 line break, UTS #39 confusables); BCP 47 (<code>km-KH</code>)</td><td>X-03, X-05; i18n</td></tr>
<tr><td>Version</td><td>Semantic Versioning 2.0; Conventional Commits; Keep a Changelog</td><td>tag <code>v1.2.3</code> ⇒ release CI</td></tr>
</tbody></table></div>
"""

S15 = r"""
<h3>Repository</h3>
<ul>
<li>GitHub (org ឬ account GoldenIQ), repo សាធារណៈ (source-available)។ README មានពីរភាសា (ខ្មែរមុន) ហើយបញ្ជាក់ច្បាស់ថា "មិនមែន open source"។ ឯកសារត្រូវមាន: LICENSE + LICENSE-km.md (© Golden IQ &amp; CHHIT Sungchhay), SECURITY.md (GitHub private vulnerability reporting), CONTRIBUTING.md (issue/សំណើ = អាជ្ញាប័ណ្ណមិនផ្តាច់មុខ; កូដ/ការបកប្រែ = ត្រូវចុះលក្ខខណ្ឌអ្នកចូលរួមចំណែកដាច់ដោយឡែក, LICENSE draft §4), PRIVACY.md, THIRD-PARTY-NOTICES។</li>
<li>ការការពារ branch <code>main</code>: តម្រូវ PR, CI ត្រូវឆ្លង, history ត្រង់ (linear), tag ត្រូវចុះហត្ថលេខា។ ម្ចាស់ជាអ្នក push តែម្នាក់ (ច្បាប់ម្ចាស់)។</li>
<li>CI (GitHub Actions, pin SHA, <code>permissions: read-all</code>): fmt, clippy <code>-D warnings</code>, test, cargo-deny, cargo-audit, npm audit, ESLint (<code>react/no-danger</code>), grep <code>danger_accept_invalid</code>, Playwright (UI ខ្មែរ/អង់គ្លេស), hostile HTTP fixtures, fuzz (ប្រចាំសប្ដាហ៍), OpenSSF Scorecard។</li>
</ul>
<h3>ការចេញ release (ម្ចាស់ចុចតែ ២ ដង)</h3>
<ol>
<li>ម្ចាស់ push tag <code>v1.2.0</code> (ម្ចាស់ជាអ្នក push ដោយខ្លួនឯង)។</li>
<li>Workflow <code>release.yml</code> build x64 + arm64 ⇒ (1) MSIX សម្រាប់ Microsoft Store (Store ចុះហត្ថលេខា) (2) NSIS installer + portable zip សម្រាប់ GitHub (គ្មាន Authenticode រហូតដល់ D8; មាន sha256 + attestation)។</li>
<li>Tauri updater ចុះហត្ថលេខា minisign (secret ក្នុង GitHub Environment "release" ដែលតម្រូវ reviewer = ម្ចាស់) ⇒ <code>latest.json</code> + attestation (<code>actions/attest-build-provenance</code>)។</li>
<li>GitHub Release ជា draft ⇒ ម្ចាស់ពិនិត្យ ⇒ Publish ⇒ action ផ្ញើ PR ទៅ winget-pkgs ដោយស្វ័យប្រវត្តិ។ Microsoft Store ⇒ submission MSIX (ពិនិត្យដោយ MS ~1–3 ថ្ងៃ)។ Build MSIX បិទ updater ក្នុង app ព្រោះ Store ជាអ្នក update។</li>
</ol>
<h3>Certificate និងការចុះឈ្មោះ (ខាងម្ចាស់ មិនមែនអ្នកប្រើ)</h3>
<div class="tw"><table><thead><tr><th>ផ្លូវ</th><th>តម្លៃ</th><th>អ្វីដែលទទួល</th><th>អនុសាសន៍</th></tr></thead><tbody>
<tr><td>SignPath Foundation</td><td>0 $</td><td>Authenticode</td><td><strong>មិនអាច</strong>: តម្រូវ licence OSI ទាំងស្រុង (R11)</td></tr>
<tr><td>Microsoft Store (បុគ្គល)</td><td>0 $ (ផ្ទៀងផ្ទាត់អត្តសញ្ញាណ)</td><td>MSIX ដែល Store ចុះហត្ថលេខា និង update។ Listing Win32 តម្រូវ installer ដែលយើងចុះហត្ថលេខាខ្លួនឯង ដូច្នេះមិនអាចប្រើបាន។ <strong>ហានិភ័យ:</strong> MSIX virtualize ការសរសេរ registry ដូច្នេះ browser ប្រហែលជាមើលមិនឃើញ native messaging host ក្នុង HKCU។ Tauri ក៏មិនបង្កើត MSIX ស្រាប់ដែរ (spike P0-9)</td><td>បាទ: ផ្លូវចុះហត្ថលេខាឥតគិតថ្លៃតែមួយ</td></tr>
<tr><td>Certum Open Source</td><td>~€69</td><td>Publisher = ឈ្មោះម្ចាស់</td><td>ទំនងជាមិនអាច: សម្រាប់គម្រោង open source (ត្រូវផ្ទៀងផ្ទាត់ជាមួយ Certum)</td></tr>
<tr><td>OV certificate ពាណិជ្ជកម្ម (Certum Standard, Sectigo ...)</td><td>ប្រហែល $100–300/ឆ្នាំ (ត្រូវផ្ទៀងផ្ទាត់តម្លៃ)</td><td>Authenticode ឈ្មោះ Golden IQ, installer GitHub គ្មានការព្រមាន (ពេល reputation កើន)</td><td>D8: ពេលមានថវិកា</td></tr>
<tr><td>Azure Artifact Signing</td><td>$9.99/ខែ</td><td>—</td><td>មិនអាច (បុគ្គលតែ US/Canada)</td></tr>
<tr><td>Chrome Web Store</td><td>$5 ម្ដង</td><td>បង្ហោះ extension</td><td>ចាំបាច់ (ថ្លៃតែមួយដងនេះ មិនមែនថ្លៃអ្នកប្រើ)</td></tr>
<tr><td>Edge Add-ons, Firefox AMO</td><td>0 $</td><td>បង្ហោះ extension</td><td>បាទ</td></tr>
<tr><td>Apple notarization</td><td>$99/ឆ្នាំ</td><td>macOS</td><td>Phase 4 (D7)</td></tr>
</tbody></table></div>
<p class="note">SmartScreen: EV លែងរំលង SmartScreen ទៀតហើយ (ចាប់ពីឆ្នាំ 2024)។ Reputation កើនតាម certificate និង hash ដូច្នេះ release ដំបូងៗអាចនៅមានការព្រមាន។ ទំព័រ download ត្រូវពន្យល់រឿងនេះ និងបង្ហាញ sha256។</p>
"""

S16 = r"""
<div class="grid2">
<div class="card"><h4>ឯកសារដែលបានទទួល</h4><p><code>png_style_glossy_3d_isometric_cartoon_icon_of_the.png</code> ទំហំ 3762×3762 RGBA, 7.4 MB (ផ្ទៃថ្លា)។ <code>13221261-…jpg</code> ទំហំ 3762×3762 ផ្ទៃ navy <code>#001848</code>។ រូបរាង: គូបពិភពលោកប្រភេទ voxel (ខៀវ/បៃតងក្រូចឆ្មារ/ស) ព័ទ្ធដោយ swoosh ព្រួញ និងមានគ្រាប់ពពុះ។</p></div>
<div class="card"><h4>ចំណុចល្អ</h4><p>ពណ៌ច្បាស់ ស្គាល់ភ្លាមនៅទំហំធំ ហើយសមនឹងឈ្មោះ "Galaxy"។ ពណ៌ខៀវ <code>#1890f0</code> និងបៃតង <code>#b5e61d</code> យកមកធ្វើ accent របស់ UI បាន។</p></div>
<div class="card"><h4>បញ្ហា</h4><ul>
<li>នៅ 16–32 px (tray, taskbar, extension) ពពុះ និងព័ត៌មានលម្អិតក្លាយជាចំណុចរញ៉េរញ៉ៃ (បានសាកបង្រួម 32 px)។</li>
<li>Bounding box ដល់គែម (3732 px) គ្មាន padding ទេ។ Windows icon ត្រូវការគែម ~6%។</li>
<li>ព្រួញមិនចង្អុលចុះក្រោម ដូច្នេះមិនបង្ហាញន័យ "download" ច្បាស់។</li>
<li>Gloss និង highlight ក្នុង logo ជាការរចនារបស់ម្ចាស់ (អនុញ្ញាតក្នុង logo)។ ប៉ុន្តែកំណែតូចក្នុង UI គួរតែ flat ដើម្បីស្របនឹងច្បាប់ "គ្មាន glow"។</li>
<li>គូប voxel បៃតង/ស មើលទៅស្រដៀងស្ទីល Minecraft បន្តិច ហានិភ័យទាប តែគួរដឹង។ រូបបង្កើតដោយ AI ទំនងជាមិនអាចចុះ copyright (§14)។</li>
</ul></div>
<div class="card"><h4>អនុសាសន៍</h4><ul>
<li>រក្សា logo 3D សម្រាប់ទំហំ ≥ 128 px (installer, Store, About, splash)។</li>
<li>គូរឡើងវិញជា vector "mark" សាមញ្ញសម្រាប់ ≤ 48 px: គូប + swoosh ក្រាស់ គ្មានពពុះ។ មានកំណែ monochrome សម្រាប់ tray (ពណ៌ស/ខ្មៅតាម theme taskbar)។</li>
<li>ស្នើជម្រើស: swoosh បង្វិលឱ្យចុងព្រួញចង្អុលចុះ ↓ (ម្ចាស់សម្រេច D13)។</li>
</ul></div>
</div>
<h3>បញ្ជី asset ដែលត្រូវបង្កើត</h3>
<div class="tw"><table><thead><tr><th>កន្លែង</th><th>ទំហំ</th><th>ប្រភព</th></tr></thead><tbody>
<tr><td>Windows <code>icon.ico</code></td><td>16, 20, 24, 32, 40, 48, 64, 256</td><td>≤48: mark vector · 64/256: 3D</td></tr>
<tr><td>Tauri <code>icons/</code></td><td>32, 128, 128@2x, icon.png 512, Square*Logo (Store)</td><td><code>tauri icon</code> ពី PNG 1024 មាន padding</td></tr>
<tr><td>Tray</td><td>16, 20, 24, 32 (ពណ៌ស និងខ្មៅ) + badge "កំពុងទាញ"</td><td>mark monochrome</td></tr>
<tr><td>Extension</td><td>16, 32, 48, 128 + ស្ថានភាពបិទ (ប្រផេះ)</td><td>mark</td></tr>
<tr><td>NSIS</td><td>header 150×57, sidebar 164×314 (BMP)</td><td>JPG navy + logo</td></tr>
<tr><td>Store / GitHub</td><td>Store tiles 300×300, 1080×1080 hero; social preview 1280×640</td><td>JPG navy</td></tr>
<tr><td>Website favicon</td><td>SVG + 32 PNG + 180 apple-touch</td><td>mark</td></tr>
</tbody></table></div>
<p>PNG 7.4 MB ត្រូវ optimize (oxipng) និងរក្សាប្រភពដើមក្នុង <code>logo/source/</code>។ ក្នុង repo ប្រើតែ PNG 1024 ដែលបាន optimize (≈ 1 MB)។</p>
"""

S17 = r"""
<p>ការប៉ាន់ស្មាន: អ្នកអភិវឌ្ឍម្នាក់ (ម្ចាស់ + Claude Code) ធ្វើការពេញម៉ោង។ ពេលវេលាជាការប៉ាន់ស្មាន មិនមែនការសន្យាទេ។ អ្វីដែលជាការសន្យាគឺ exit criteria: ដំណាក់កាលមួយមិនត្រូវបានចាត់ទុកថារួចរាល់ ទាល់តែគ្រប់លក្ខខណ្ឌទាំងអស់ត្រូវបានបំពេញ។</p>
<div class="tw"><table><thead><tr><th>ដំណាក់</th><th>រយៈពេល</th><th>ខ្លឹមសារ</th><th>Exit criteria (ត្រូវវាស់បាន)</th></tr></thead><tbody>
<tr><td>P0 · Spike</td><td>3 សប្ដាហ៍</td><td><strong>Gate Store (មុនគេ):</strong> គណនី (D9), MSIX + native messaging (browser មើលឃើញ host, launch, upgrade, uninstall, Windows 10/11), certification ដែលមាន component ទាញពេលក្រោយ; បើបរាជ័យ ⇒ ផែនការ B: GitHub គ្មានហត្ថលេខា + D8។ Prototype បោះចោលបាន: MV3 prepared handoff; named pipe DACL + ផ្ទៀងផ្ទាត់ server; IAttachmentExecute ពី Rust; Khmer.nsh ក្នុង NSIS; MSIX + native messaging host ពី package (P0-9: <code>desktop:Extension</code>/unvirtualized registry ឬ installer helper ដាច់); policy proxy សម្រាប់ yt-dlp (ADR-6); protocol ដំឡើង update + recover (U-04, U-06); hostile HTTP fixture server (200-on-range, ETag ប្ដូរ, infinite, slowloris, redirect cross-origin, 429); segmenter + ការវាស់ល្បឿនធៀប IDM; design tokens + 5 screen mockups</td><td>Spike ទាំង 9 មានលទ្ធផលកត់ត្រា (ជោគជ័យ ឬ ផែនការ B); ម្ចាស់សម្រេច D1–D18; council review លទ្ធផល P0</td></tr>
<tr><td>P1 · MVP</td><td>8 សប្ដាហ៍</td><td>Engine HTTP/HTTPS (segment, resume, integrity, limiter សរុប+job), .part/rename/MOTW, SQLite, UI ៣ ទម្រង់ (list, details, settings), ផ្ទាំង Download Info, toast, tray, extension Chrome/Edge/Firefox (ចាប់, Alt/Ctrl, context menu, popup), categories, ស្វែងរក FTS, Khmer/English, theme ពីរ, updater, installer NSIS + MSIX, CI ពេញ</td><td>កាត critical/high ទាំងអស់ក្នុង F01–F05, F12–F17 ត្រូវមាន test ហើយ test ឆ្លង; MSIX ឆ្លង Store certification ហើយ native messaging ដំណើរការពី MSIX (ឬមានផែនការ B ដែលបានសាក); 0 ឯកសារខូចលើ 1000 រង្វង់ kill -9; hostile fixtures ឆ្លងទាំងអស់; ល្បឿនមធ្យម ≥ IDM −5% និងស្មើ/លឿនជាងលើ ≥ 3/5 URL (§09); screenshot ខ្មែរគ្មាន clip; axe 0 serious; v0.9 public beta</td></tr>
<tr><td>P2 · IDM parity</td><td>10 សប្ដាហ៍</td><td>Queue + sync queue + scheduler + post actions, refresh address protocol, batch, clipboard, CLI flag IDM, import .ef2/txt, export JSON, proxy PAC/NTLM, site logins, per-server limits, quota, sounds, drop basket, video (component yt-dlp/ffmpeg, popup quality, ប៊ូតុងទំព័រ opt-in), Microsoft Store, failure-injection test (ADR-1)</td><td>Parity ledger: ជួរ P1/P2 ទាំងអស់ "រួច" ឬមានការយល់ព្រមពីម្ចាស់; កាត F06–F11 ឆ្លង; beta tester ១០ នាក់ (ខ្មែរ ≥ 5) ប្រើ 2 សប្ដាហ៍ ⇒ ≥ 8/10 ថា "ជំនួស IDM បាន"; v1.0</td></tr>
<tr><td>P3 · លើសគេ</td><td>8 សប្ដាហ៍</td><td>Site grabber (wizard 4 ជំហាន), preview zip, SFTP, HTTP/3 (ពេល reqwest stable), HLS native, portable mode, gdm:// (ទម្រង់កំណត់), ការបកប្រែតាមសហគមន៍ (Weblate), VPN/rasdial (បើ D4)</td><td>កាត F09 ឆ្លង; grabber ឆ្លង trap fixtures; v1.x</td></tr>
<tr><td>P4 · ជម្រើស</td><td>—</td><td>macOS (notarization $99/ឆ្នាំ), BitTorrent (librqbit), Linux (Flathub)</td><td>ម្ចាស់សម្រេចតាមតម្រូវការអ្នកប្រើពិត</td></tr>
</tbody></table></div>
<h3>ការងារមុនសរសេរកូដ (សប្ដាហ៍នេះ)</h3>
<ol><li>ម្ចាស់អានឯកសារនេះ ហើយសម្រេច D1–D18 (§19)។ D9, D17, D18 ជាលក្ខខណ្ឌមុនចែកចាយតាម Store; D1/D17 ជាលក្ខខណ្ឌមុនបោះពុម្ព licence។</li><li>បង្កើត repo GitHub <code>galaxy-download-manager</code> (ម្ចាស់បង្កើត និង push)។</li><li>ចាប់ផ្ដើម P0 spike ទាំង 9 តាមលំដាប់ហានិភ័យ: MSIX + native messaging ⇒ MV3 prepared handoff ⇒ pipe ⇒ fixtures ⇒ segmenter + lease ⇒ MOTW/exFAT ⇒ policy proxy yt-dlp ⇒ update install/recover ⇒ Khmer NSIS។</li></ol>
"""

S19 = r"""
<p>ការសម្រេចទាំងនេះជារបស់ម្ចាស់។ D14–D15 ត្រូវបានបន្ថែមក្រោយ council ជុំទី ៣ ហើយ D16–D18 បន្ថែមក្រោយម្ចាស់សម្រេចលើ licence។ ទីប្រឹក្សាផ្ដល់អនុសាសន៍ ហើយការងារ P0 អាចចាប់ផ្ដើមបានមុនមានចម្លើយ លើកលែងតែ D11/D12 (ឈ្មោះ repo)។</p>
<div class="tw"><table><thead><tr><th>#</th><th>សំណួរ</th><th>អនុសាសន៍</th><th>ហេតុផល</th></tr></thead><tbody>
<tr><td>D1</td><td>Licence: គោលការណ៍សម្រេចរួច (តឹងរឹង, ប្រើផ្ទាល់ខ្លួន)។ អត្ថបទណាត្រូវប្រើ: <code>LICENSE</code> ដែល commit រួច ឬ draft ក្នុង <code>docs/licence-draft/</code>?</td><td>Draft (ឬបញ្ចូលភាពខ្លាំងរបស់វាទៅក្នុង LICENSE ដែលមានស្រាប់)</td><td>LICENSE ដែលមានស្រាប់ហាម "using it inside a business" ហើយកំណត់ "Software" រួមទាំងរូបភាព/ឈ្មោះ ប៉ុន្តែគ្មាន §2A សម្រាប់ Microsoft Store (Developer Agreement §4(i)) និងគ្មានការលើកលែងសិទ្ធិ consumer។ ទាំងពីរត្រូវការមេធាវីកម្ពុជាពិនិត្យ។ ឈ្មោះក្នុង LICENSE ដែលមានស្រាប់: "GoldenIQ &amp; CHHIT SUNGCHHAY", "Galaxy Download Manage" (D11, D17)។</td></tr>
<tr><td>D2</td><td>បោះបង់ "Advanced integration" របស់ IDM (WFP kernel driver ចាប់ពីគ្រប់កម្មវិធី)?</td><td>បោះបង់</td><td>CVE-2026-90493, BSOD។ សមមូល: extension + clipboard + CLI + Add URL (ប្រហែល 95% នៃករណីប្រើ)។</td></tr>
<tr><td>D3</td><td>បោះបង់ browser/protocol ដែលស្លាប់ហើយ: IE, MSN, AOL, Avant, Maxthon, IE click monitor, MMS?</td><td>បោះបង់</td><td>IE ចូលនិវត្តន៍ មិថុនា 2022; MMS ត្រូវ Microsoft deprecate ឆ្នាំ 2003។</td></tr>
<tr><td>D4</td><td>Dial-up redial: ជំនួសដោយ "រង់ចាំបណ្ដាញ + ភ្ជាប់ VPN/RAS (rasdial) មុន queue" (P3) ឬបោះបង់?</td><td>ជំនួស (P3)</td><td>Modem ស្លាប់ ប៉ុន្តែ VPN ដែលប្រើ RAS នៅមាន។</td></tr>
<tr><td>D5</td><td>ប៊ូតុងវីដេអូលើទំព័រ: បិទលំនាំដើម ហើយសួរនៅ onboarding?</td><td>បាទ</td><td>NDM/IDM ត្រូវត្អូញត្អែរថារំខាន; ChatGPT និងខ្ញុំឯកភាព។</td></tr>
<tr><td>D6</td><td>Keyboard focus: ច្បាប់ម្ចាស់ហាម outline។ ស្នើ focus = ផ្ទៃ fill ប្ដូរពណ៌ (accent-soft) + អក្សរដិត។ យល់ព្រម?</td><td>យល់ព្រម</td><td>WCAG 2.4.7 តម្រូវឱ្យ focus មើលឃើញ, មិនតម្រូវឱ្យជា outline ទេ។ ក្រុមអ្នកប្រើ keyboard ត្រូវការ។</td></tr>
<tr><td>D7</td><td>macOS: ពន្យារទៅ P4 (Apple $99/ឆ្នាំ)?</td><td>ពន្យារ</td><td>NDM ពេញនិយមលើ Mac។ Core ឯករាជ្យពី platform ដូច្នេះមិនបិទផ្លូវ។</td></tr>
<tr><td>D8</td><td>Installer នៅលើ GitHub: ទុកគ្មានហត្ថលេខា (SmartScreen ព្រមាន) ឬទិញ OV certificate ឈ្មោះ Golden IQ?</td><td>Store MSIX ជាផ្លូវចម្បង; GitHub គ្មានហត្ថលេខាសិន; ទិញ cert ពេលមានថវិកា</td><td>Licence តឹងរឹងធ្វើឱ្យ SignPath មិនអាចប្រើបាន។ អ្នកប្រើភាគច្រើនដំឡើងពី Store (គ្មានការព្រមាន)។</td></tr>
<tr><td>D9</td><td>Microsoft Store: គណនីបុគ្គល (ឥតគិតថ្លៃ, publisher = ឈ្មោះផ្ទាល់ខ្លួន) ឬគណនីក្រុមហ៊ុន (ថ្លៃម្ដង + ផ្ទៀងផ្ទាត់)?</td><td>ម្ចាស់សម្រេចតាម policy</td><td>Store policy 10.14: គណនីក្រុមហ៊ុនតម្រូវមិនត្រឹមតែសម្រាប់នីតិបុគ្គលដែលបានចុះបញ្ជីប៉ុណ្ណោះទេ ថែមទាំងសម្រាប់អ្នកដែលធ្វើសកម្មភាពក្នុងអាជីវកម្ម/វិជ្ជាជីវៈរបស់ខ្លួន និងឈ្មោះដែលត្រូវបានយល់ថាជាអាជីវកម្មផងដែរ។ ការបោះពុម្ពជា "Golden IQ" ទំនងជាត្រូវការគណនីក្រុមហ៊ុន។ គណនីបុគ្គលសមស្របតែពេលបោះពុម្ពជាឈ្មោះផ្ទាល់ខ្លួន ហើយមិនមែនជាអាជីវកម្ម។ Store ត្រូវបានចាត់ទុកថាអនុម័ត តែក្រោយ D18 (ខ) ត្រូវបានយល់ព្រម។</td></tr>
<tr><td>D10</td><td>YouTube តាម component yt-dlp opt-in ក្នុង app (មិនមែនក្នុង extension Chrome)?</td><td>បាទ ជាមួយ notice</td><td>ជាតម្រូវការអ្នកប្រើខ្លាំង; ហានិភ័យច្បាប់កាត់បន្ថយដោយ component opt-in, គ្មាន DRM, គ្មាននៅក្នុង extension store។</td></tr>
<tr><td>D11</td><td>ឈ្មោះផ្លូវការ: "Galaxy Download Manage" (ដូចបានសរសេរ) ឬ "Galaxy Download <strong>Manager</strong>"?</td><td>Manager</td><td>"Manage" ទំនងជាកំហុសវាយ។ ឯកសារនេះប្រើ "Manager" រហូតដល់ម្ចាស់សម្រេច។</td></tr>
<tr><td>D12</td><td>អក្សរកាត់ "GDM" ជាន់ GNOME Display Manager (Linux)។ ប្រើ "GDM" តែក្នុង UI, ID package ប្រើ <code>GoldenIQ.GalaxyDownloadManager</code>?</td><td>បាទ</td><td>ជៀសការច្រឡំក្នុង winget/Flathub/ការស្វែងរក។</td></tr>
<tr><td>D13</td><td>Logo: គូរ mark vector សម្រាប់ទំហំតូច ហើយព្រួញចង្អុលចុះ?</td><td>Mark: បាទ · ព្រួញ: ម្ចាស់ជ្រើស</td><td>§16</td></tr>
<tr><td>D14</td><td>ទទួលយក parity "ផ្នែកខ្លះ" ៥ ជួរ (§04): POST ក្នុង Chrome ទុកឱ្យ browser, JS link តែ static, proxy តាម browser ⇒ proxy ប្រព័ន្ធ, YouTube ក្នុង app មិនមែនក្នុង extension Chrome, ភាសា 2 (មិនមែន 50+)?</td><td>យល់ព្រម ជាមួយផែនការ</td><td>ជួរនីមួយៗមានចន្លោះសមត្ថភាពដែលសរសេរច្បាស់ និងផែនការ (Firefox POST, សហគមន៍បកប្រែ P3)។ ការបិទចន្លោះទាំងអស់មានតម្លៃខ្ពស់ពេក ឬប៉ះសុវត្ថិភាព/គោលការណ៍ store។</td></tr>
<tr><td>D15</td><td>Hook "បើកកម្មវិធីពេលបញ្ចប់" អនុញ្ញាត .bat/.cmd ជាករណីលើកលែងដែលអ្នកប្រើទុកចិត្ត?</td><td>បាទ (ដើម្បីរក្សា parity IDM)</td><td>IDM ណែនាំ .bat សម្រាប់សកម្មភាពច្រើន។ ការកំណត់អាចធ្វើបានតែក្នុង UI, មាន argument restriction (Q-01)។ បើម្ចាស់ចង់បានសុវត្ថិភាពជាង ⇒ .exe ប៉ុណ្ណោះ។</td></tr>
<tr><td>D16</td><td>Licence ត្រូវការការអនុញ្ញាតសម្រាប់ការប្រើក្នុងក្រុមហ៊ុន/សាលា/ស្ថាប័ន។ ចង់អនុញ្ញាតការប្រើ <em>ក្នុងស្ថាប័ន ដោយមិនកែប្រែ និងមិនលក់</em> ដោយស្វ័យប្រវត្តិទេ?</td><td>ពិចារណាអនុញ្ញាត</td><td>បុគ្គលិកការិយាល័យជា persona ទី ២ (§05)។ ការប្រើកម្មវិធីដើម្បីទាញឯកសារការងារមិនមែនជាការលក់កូដទេ។ ការហាមអាចធ្វើឱ្យពួកគេនៅតែប្រើ IDM។ ការសម្រេចនេះជារបស់ម្ចាស់។</td></tr>
<tr><td>D17</td><td>(ក) "CHHIT Sungchhay" (នាមត្រកូលអក្សរធំ) ត្រឹមត្រូវ? មានឈ្មោះជាអក្សរខ្មែរដែលចង់ដាក់ទេ? (ខ) "Golden IQ" ជានីតិបុគ្គលដែលបានចុះបញ្ជី ឬជាឈ្មោះពាណិជ្ជកម្មរបស់ CHHIT Sungchhay? (គ) នរណាមានសិទ្ធិផ្ដល់ការអនុញ្ញាតជំនួសម្ចាស់ទាំងពីរ?</td><td>ម្ចាស់បញ្ជាក់</td><td>អាជ្ញាប័ណ្ណត្រូវកំណត់ភាគីកិច្ចសន្យាឱ្យច្បាស់ (ChatGPT ជុំ 4)។ ខ្ញុំមិនស្មានឈ្មោះ ឬស្ថានភាពច្បាប់ទេ។</td></tr>
<tr><td>D18</td><td>(ក) អនុញ្ញាត dependency MPL-2.0 ដែលមិនកែប្រែ ជាករណីលើកលែង? (ខ) ច្បាប់ចម្លងពី Microsoft Store ត្រូវគោរព Usage Rules របស់ Microsoft (Developer Agreement §4(i)) ដែលអាចទូលាយជាង licence (ឧ. ការប្រើក្នុងអង្គការ)។ យល់ព្រម (LICENSE §2A) ឬមិនបោះពុម្ពលើ Store?</td><td>(ក) បាទ (ខ) យល់ព្រម</td><td>(ក) បើគ្មាន MPL មិនអាចប្រើ Tauri បានទេ។ (ខ) Store ជាផ្លូវចុះហត្ថលេខាឥតគិតថ្លៃតែមួយ។ បើបដិសេធ ⇒ GitHub គ្មានហត្ថលេខា ឬត្រូវទិញ cert (D8)។</td></tr>
</tbody></table></div>
"""

S20 = r"""
<h3>Coverage</h3>
<ul>
<li><strong>IDM:</strong> មុខងារ 22 ក្រុមពីឯកសារផ្លូវការ, CVE 4 + EDB 6 និង CVE ថ្មី 2 ដែលខ្ញុំបានផ្ទៀងផ្ទាត់ផ្ទាល់ (CVE-2025-56231 តាម Wiz, CVE-2026-90493 តាម OpenCVE)។</li>
<li><strong>NDM:</strong> ឯកសារផ្លូវការ, កូដ extension Firefox 1.9.9 (អានផ្ទាល់), និង review AMO/Chrome។</li>
<li><strong>សុវត្ថិភាព:</strong> ឧប្បត្តិហេតុពិត 27 ករណី, ថ្នាក់វាយប្រហារ 30 និងកាត {{NSCEN}}។</li>
</ul>
<h3>ទំនុកចិត្ត</h3>
<div class="tw"><table><thead><tr><th>ការអះអាង</th><th>កម្រិត</th></tr></thead><tbody>
<tr><td>បញ្ជីមុខងារ IDM (ឯកសារផ្លូវការ)</td><td>ខ្ពស់។ ចំណុចខ្លះនៃ progress dialog និងតម្លៃលំនាំដើម connection មិនទាន់ផ្ទៀងផ្ទាត់ លើកំណែដែលបានដំឡើងពិត</td></tr>
<tr><td>ស្ថាបត្យកម្ម និងគោលការណ៍សុវត្ថិភាព</td><td>ខ្ពស់។ ChatGPT ទទួលយកជាផ្លូវកណ្ដាល (99%) ក្រោយ 7 ជុំ (§18) ហើយមានភស្តុតាងពី CVE ពិត។ ការទទួលយកនេះគឺលើការរចនា មិនមែនលើកូដពិតទេ</td></tr>
<tr><td>MV3 handoff, pipe, Khmer NSIS, policy proxy yt-dlp, update recover, MSIX</td><td>មធ្យម។ ត្រូវការ spike P0 ដើម្បីបញ្ជាក់</td></tr>
<tr><td>រយៈពេល P1–P3</td><td>ទាប-មធ្យម។ ជាការប៉ាន់ស្មាន</td></tr>
<tr><td>ស្ថានភាពច្បាប់ PDP កម្ពុជា</td><td>ទាប។ មិនទាន់ផ្ទៀងផ្ទាត់សម្រាប់ខែវិច្ឆិកា 2026</td></tr>
</tbody></table></div>
<h3>ដែនកំណត់</h3>
<ul><li>Reddit, Trustpilot និង VideoHelp បានបិទមិនឱ្យភ្នាក់ងារស្រាវជ្រាវចូល ដូច្នេះមតិអ្នកប្រើមកពី AMO, Chrome Web Store, AlternativeTo, វេទិកា និង FAQ ផ្លូវការ។ មិនមានការបង្កើតសម្រង់ (quote) ក្លែងក្លាយទេ។</li><li>មិនបានដំឡើង IDM/NDM ដើម្បីវាស់ផ្ទាល់ទេ។ P0 មានការងារវាស់ល្បឿនធៀប IDM។</li><li>ឯកសារស្រាវជ្រាវដើម (អង់គ្លេស មាន URL ប្រភពគ្រប់ចំណុច) រក្សាទុកក្នុង <code>docs/research/</code>។</li></ul>
"""

SECTIONS = [
    ("s00", "00", "សង្ខេប", S00),
    ("s01", "01", "តម្រូវការ (ជំហាន ១: ធ្វើឱ្យឆ្លាតជាងមុន)", S01),
    ("s02", "02", "ការសិក្សា IDM", S02),
    ("s03", "03", "ការសិក្សា NDM", S03),
    ("s04", "04", "Parity ledger: គ្មានមុខងារបាត់", S04.replace("{{PARITY}}", PARITY_HTML)),
    ("s05", "05", "បទពិសោធន៍អ្នកប្រើ និងដំណើរការ", S05 + "<p>មើល §07។</p>"),
    ("s06", "06", "ប្រព័ន្ធរចនា និង responsive", DESIGN_HTML),
    ("s07", "07", "Auto-update", S07),
    ("s08", "08", "ស្ថាបត្យកម្ម", S08),
    ("s09", "09", "Engine ទាញយក", S09),
    ("s10", "10", "Threat model", S10),
    ("s11", "11", "សេនារីយ៉ូដេញដោល: តារាង S1–S15", S11),
    ("s12", "12", "សេនារីយ៉ូតាមមុខងារ និងដំណោះស្រាយ", S12),
    ("s13", "13", "ការពិនិត្យកូដគំរូម្ដងមួយបន្ទាត់", CODE_HTML),
    ("s14", "14", "ក្បួនច្បាប់អន្តរជាតិ", S14),
    ("s15", "15", "Source-available, certificate, release", S15),
    ("s16", "16", "Logo និង asset", S16),
    ("s17", "17", "ផែនការអភិវឌ្ឍ និង exit criteria", S17),
    ("s18", "18", "Council: Claude × ChatGPT", COUNCIL_HTML),
    ("s19", "19", "ម្ចាស់ត្រូវសម្រេច (18)", S19),
    ("s20", "20", "Coverage, ទំនុកចិត្ត, ដែនកំណត់", S20),
]
