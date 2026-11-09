# §04 parity ledger: every IDM/NDM feature -> GDM status. 68 rows (S00 KPI reads this count).
T = {"keep": ("keep", "រក្សា"), "better": ("better", "ល្អជាង"), "new": ("new", "ថ្មី"),
     "swap": ("swap", "សមមូល"), "owner": ("", "ម្ចាស់"), "partial": ("swap", "ផ្នែកខ្លះ")}

ROWS = [
    # group, feature, IDM, NDM, GDM, status, phase
    ("Engine", "Dynamic segmentation", "✓", "✓", "ចាប់ពី 2 connection, បន្ថែមពេលល្បឿនកើន, កាតផ្នែកធំបំផុតពាក់កណ្ដាល ≥1 MiB", "better", "P1"),
    ("Engine", "ពិនិត្យ seam/overlap", "✓", "—", "Seam 4 KiB + 206/Content-Range/If-Range តឹងរឹង; ខូច ⇒ ផ្អាក និងប្រាប់ (មិនធ្លាក់ទៅ 1 connection ដោយស្ងាត់)", "better", "P1"),
    ("Engine", "ចំនួន connection 1–32, ប្ដូរពេលកំពុងទាញ", "✓", "✓ (live)", "Live ក្នុង details pane", "keep", "P1"),
    ("Engine", "Connection ក្នុងមួយ server (Exceptions)", "✓", "—", "Host profile ដោយដៃ + រៀនពី 429", "better", "P2"),
    ("Engine", "Quota (MB ក្នុងមួយម៉ោង)", "✓", "—", "សរុប និងក្នុង queue", "keep", "P2"),
    ("Engine", "Speed limiter", "ក្នុងឯកសារ", "job + សរុប live", "សរុប + queue + job + ផែនការម៉ោង + toggle លើ tray", "better", "P1"),
    ("Engine", "Resume ក្រោយ crash/ដាច់ភ្លើង", "✓", "✓", "Flush មុន commit (E-04)", "better", "P1"),
    ("Engine", "Refresh download address / renew", "✓", "✓ (ខ្សោយ)", "Protocol 5 ជំហាន + ticket + បិទភ្ជាប់ URL (R-01)", "better", "P2"),
    ("Engine", "មិនអើពើ mod-time ពេល resume", "✓", "—", "តក្កវិជ្ជា validator ខ្លាំង + សួរពេលប្ដូរ (E-02)", "swap", "P1"),
    ("Engine", "HTTP/1.1, HTTP/2, HTTPS", "✓", "✓", "reqwest + rustls", "keep", "P1"),
    ("Engine", "FTP (PASV), FTPS", "✓", "✓", "suppaftp; ព្រមាន plaintext", "keep", "P2"),
    ("Engine", "MMS", "✓", "—", "Protocol ត្រូវ deprecate 2003 (D3)", "owner", "—"),
    ("Engine", "SFTP, HTTP/3", "—", "—", "ពេល crate stable", "new", "P3"),
    ("Engine", "BitTorrent/magnet", "—", "—", "librqbit ជាជម្រើស", "new", "P4"),
    ("Engine", "User-Agent", "✓", "✓", "សកល + តាមគេហទំព័រ", "better", "P1"),
    ("Engine", "Temp folder ហើយ rebuild/merge", "✓", "✓", ".part ក្នុងថតគោលដៅ; ជម្រើស temp folder សម្រាប់អ្នកចង់បាន (ADR-3)", "better", "P1"),
    ("Engine", "Hash verify", "—", "—", "SHA-256/512, អាន .sha256 ពី origin ដដែល", "new", "P1"),
    ("Network", "Auth Basic/Digest/NTLM/Negotiate/Kerberos", "✓", "Basic/Digest/NTLM", "SSPI", "keep", "P2"),
    ("Network", "Site logins តាមថត", "✓", "—", "Credential Manager", "better", "P2"),
    ("Network", "Proxy: system, PAC, manual, SOCKS, bypass", "✓", "HTTP/SOCKS", "WinHTTP PAC + manual + SOCKS5", "keep", "P2"),
    ("Network", "Proxy ពី browser ដែលចាប់", "✓", "—", "Extension អាន proxy ដែល browser កំពុងប្រើ (<code>chrome.proxy.settings.get</code> / <code>browser.proxy.settings.get</code>, permission <code>proxy</code>: mode, PAC, rules) ហើយផ្ញើជាមួយ handoff ⇒ engine ប្រើ proxy ដដែល។ ពាក្យសម្ងាត់ proxy មិនអាចអានបាន ⇒ ស្នើបញ្ចូលក្នុង GDM ម្ដង (Credential Manager)។ ប្រភព: developer.chrome.com/docs/extensions/reference/api/proxy", "better", "P2"),
    ("Network", "Dial-up/RAS redial", "✓", "—", "រង់ចាំបណ្ដាញ + rasdial VPN (D4)", "owner", "P3"),
    ("Browser", "Chrome, Edge, Firefox, Opera, Brave, Vivaldi, fork", "✓", "✓ (ខ្វះ fork)", "Host manifest គ្រប់ទីតាំង (B-07)", "better", "P1"),
    ("Browser", "IE, MSN, AOL, Avant, Maxthon + IE click monitor", "✓", "—", "Browser ស្លាប់ហើយ (D3)", "owner", "—"),
    ("Browser", "Advanced integration (WFP driver)", "✓", "—", "គ្មាន driver; extension + clipboard + CLI (D2)", "owner", "—"),
    ("Browser", "ចាប់តាមប្រភេទឯកសារ", "✓", "✓", "Extension + MIME + ទំហំអប្បបរមា", "better", "P1"),
    ("Browser", "បញ្ជីលើកលែង site/address", "✓", "—", "+ 'កុំចាប់គេហទំព័រនេះ' មួយចុចក្នុង popup", "better", "P1"),
    ("Browser", "Ctrl+click បង្ខំ / Alt+click រំលង", "✓", "Delete (លាក់)", "Rebind បាន, បង្ហាញក្នុង onboarding (C-04)", "better", "P1"),
    ("Browser", "Context menu: ទាញ / ទាញ link ទាំងអស់", "✓", "ទាញ", "ទាំងពីរ + preview បញ្ជី", "keep", "P1"),
    ("Browser", "ផ្ទាំងវីដេអូលើទំព័រ", "✓", "✓ (រំខាន)", "បិទលំនាំដើម, បើកតាមគេហទំព័រ, badge លើ icon extension (D5)", "better", "P2"),
    ("Browser", "ផ្ទាំងលើ link ដែលជ្រើស", "✓", "—", "", "keep", "P2"),
    ("Browser", "ផ្ញើ cookie/referrer/POST", "✓", "✓", "Cookie ក្នុង jar តាម origin។ POST: extension សង្កេត <code>webRequest.onBeforeRequest</code> ជាមួយ <code>requestBody</code> (non-blocking, ប្រើបានក្នុង MV3) ហើយផ្គូរជាមួយ download ⇒ GDM ទាញដោយ POST ដដែល <strong>បន្ទាប់ពីអ្នកប្រើបញ្ជាក់</strong> ព្រោះ POST អាចធ្វើសកម្មភាពម្ដងទៀត (C-02)។ ករណីផ្គូរមិនបាន ⇒ ទុកឱ្យ browser។ ប្រភព: developer.chrome.com/docs/extensions/reference/api/webRequest", "better", "P2"),
    ("Browser", "Incognito", "✓", "?", "មិនរក្សាប្រវត្តិ (C-05)", "better", "P1"),
    ("Browser", "Toggle ចាប់លើ icon extension", "—", "✓ (លាក់)", "Popup: ស្ថានភាព, បិទការចាប់, ថ្មីៗ, គេហទំព័រនេះ", "better", "P1"),
    ("Browser", "ប៊ូតុង 'Browsers' ដំឡើង extension", "—", "✓", "ក្នុង onboarding + Settings ជាមួយស្ថានភាព", "better", "P1"),
    ("Media", "HLS/DASH, mux MP4/MKV, ជ្រើសគុណភាព, subtitle", "✓", "HLS→.ts", "Component yt-dlp + ffmpeg", "better", "P2"),
    ("Media", "YouTube", "✓ (ខូចញឹក)", "—", "ក្នុង app តាម component; clipboard monitor ចាប់ URL YouTube ដែលអ្នកប្រើចម្លង ហើយស្នើទាញ (មួយជំហានបន្ថែម)។ <em>ចន្លោះដែលយើងកែមិនបាន:</em> extension ក្នុង Chrome Web Store មិនអាចមានប៊ូតុងលើទំព័រ YouTube បានទេ ព្រោះ Program Policies ហាម 'unauthorized ... download ... of copyrighted content' (M-07)", "partial", "P2"),
    ("Media", "DRM", "—", "—", "មិនគាំទ្រ, សារច្បាស់", "keep", "—"),
    ("Grabber", "Wizard 4 ជំហាន, template, depth, filter, ទំហំ, ស្ទួន, link local, project, sync", "✓", "—", "+ trap limits, robots.txt", "better", "P3"),
    ("Grabber", "Login + ទំព័រជៀស, JS parsing", "✓", "—", "Login/ទំព័រជៀស: រក្សា។ JS: (1) static regex លំនាំដើម (2) ជម្រើស 'render JavaScript' (បិទលំនាំដើម, ដូច IDM 'use with caution'): ផ្ទុកទំព័រក្នុង WebView ដាច់ដែលលាក់ ហើយយក link ពី DOM។ Window នោះផ្ទុក URL ខាងក្រៅ ដូច្នេះហៅ IPC មិនបាន (Tauri: 'By default the API is only accessible to bundled code'; មិនដាក់ <code>remote</code>), គ្មាន download/popup, timeout, និងគោលការណ៍ W-02 (G-05)", "better", "P3"),
    ("Queue", "Download queue, sync queue, queue គ្មានកំណត់", "✓", "—", "Sync ប្រើ conditional request (Q-05)", "better", "P2"),
    ("Queue", "N ឯកសារព្រមគ្នា, retry, ម៉ោងចាប់ផ្ដើម/ឈប់, ថ្ងៃ, periodic", "✓", "—", "DST-safe (Q-03)", "keep", "P2"),
    ("Queue", "ពេលបញ្ចប់: បិទម៉ាស៊ីន, sleep, hibernate, ចេញពី app", "✓", "—", "Countdown 60 វិ", "better", "P2"),
    ("Queue", "Force terminate processes", "✓", "—", "តែពេលជ្រើសច្បាស់", "keep", "P2"),
    ("Queue", "បើកឯកសារ/កម្មវិធីពេលបញ្ចប់", "✓", "—", "កំណត់តែក្នុង UI, argv, គ្មាន shell (Q-01)", "keep", "P2"),
    ("Queue", "Hang up modem", "✓", "—", "ផ្ដាច់ VPN/RAS (D4)", "owner", "P3"),
    ("Queue", "Wake to download", "✓", "—", "Task Scheduler, AC only", "keep", "P2"),
    ("Category", "Category: ថត, ប្រភេទ, site pattern, ចងចាំថតចុងក្រោយ", "✓", "តែ view", "", "keep", "P1"),
    ("Category", "Tree: មិនទាន់ចប់/ចប់/queue/grabber", "✓", "✓", "+ ស្វែងរក FTS", "better", "P1"),
    ("Safety", "Virus scan ក្រោយទាញ (scanner ផ្ទាល់)", "✓", "—", "IAttachmentExecute (AV ប្រព័ន្ធ + MOTW) + scanner ផ្ទាល់ជាជម្រើស", "better", "P2"),
    ("Input", "Clipboard monitor", "✓ (បើកលំនាំដើម)", "—", "Opt-in, privacy (I-01)", "better", "P2"),
    ("Input", "Command line /d /p /f /q /h /n /a /s", "✓", "—", "Flag ដូចគ្នា + subcommand + JSON (I-02)", "better", "P2"),
    ("Input", "Batch download (pattern *)", "✓", "—", "Preview ចំនួន", "keep", "P2"),
    ("Input", "Import/export (.ef2, txt), settings backup", "✓ (registry ដោយដៃ)", "—", "Import .ef2/txt/json; export JSON រួម settings", "better", "P2"),
    ("Input", "Drag-drop link ទៅ window / drop basket", "✓", "Mac ប៉ុណ្ណោះ", "Windows + basket", "better", "P1"),
    ("Dialogs", "Download Info: ទាញភ្លាមពេលជ្រើសថត, Later+queue, Preview zip", "✓", "ផ្ទាំងសាមញ្ញ", "ផ្ទាំងរួមពេលមានច្រើន (C-03); preview zip P3", "better", "P1"),
    ("Dialogs", "Progress dialog: connection, segment bar, limiter", "✓", "—", "Details pane + ទម្រង់ mini", "better", "P1"),
    ("Dialogs", "Complete dialog: បើក, ថត, អូសឯកសារចេញ", "✓", "—", "Toast មានសកម្មភាព + dialog ជាជម្រើស", "better", "P1"),
    ("Dialogs", "Silent mode", "✓", "—", "", "keep", "P1"),
    ("Dialogs", "Properties: កែ URL/ថត/description/login", "✓", "—", "", "keep", "P1"),
    ("Main", "Column: បង្ហាញ/លាក់/រៀប/តម្រៀប", "✓", "តិច", "", "keep", "P1"),
    ("Main", "Toolbar customise + skin 3D/Neon", "✓", "—", "Toolbar customise + theme ភ្លឺ/ងងឹត/density", "swap", "P2"),
    ("Main", "លុបជាមួយឯកសារ", "✓", "?", "ទៅ Recycle Bin", "better", "P1"),
    ("Main", "Tray menu", "✓", "?", "+ limiter, ផ្អាកទាំងអស់, queue", "better", "P1"),
    ("Main", "សំឡេងតាមព្រឹត្តិការណ៍ (WAV ផ្ទាល់)", "✓", "—", "សំឡេង 3 ស្រាប់ + WAV ផ្ទាល់", "keep", "P2"),
    ("Main", "Dark mode", "ផ្នែកខ្លះ", "?", "គ្រប់ផ្ទាំង", "better", "P1"),
    ("Main", "ភាសា 50+", "✓", "EN", "ខ្មែរ + EN នៅ v1។ ប្រព័ន្ធ i18n (i18next) ទទួលភាសាថ្មីដោយមិនបាច់កែកូដ, Weblate ចាប់ពី P3។ <em>ចន្លោះ:</em> ចំនួនភាសាអាស្រ័យលើអ្នកបកប្រែពិត។ ការបកប្រែដោយម៉ាស៊ីនមិនត្រូវបានប្រើ (§05 គោលការណ៍ 6)", "partial", "P1→P3"),
    ("Main", "Update / licence / Tell a friend", "ប្រចាំសប្ដាហ៍ / បង់ / ✓", "គ្មាន / free / —", "§07 / ឥតគិតថ្លៃសម្រាប់ការប្រើផ្ទាល់ខ្លួន / ប៊ូតុងចែករំលែក link ក្នុង About", "better", "P1"),
]

assert len(ROWS) == 68, len(ROWS)


def _row(r):
    g, f, i, n, d, st, ph = r
    cls, label = T[st]
    return "<tr><td>%s</td><td>%s</td><td>%s</td><td>%s</td><td>%s</td><td><span class=\"tag %s\">%s</span></td><td>%s</td></tr>" % (
        g, f, i, n, d, cls, label, ph)


counts = {}
for r in ROWS:
    counts[r[5]] = counts.get(r[5], 0) + 1

PARITY_HTML = ('<p>សរុប %d: រក្សា %d · ល្អជាង %d · ថ្មី %d · សមមូល %d · ម្ចាស់សម្រេច %d។</p>'
               % (len(ROWS), counts.get("keep", 0), counts.get("better", 0), counts.get("new", 0), counts.get("swap", 0), counts.get("owner", 0))
               + '<p>ផ្នែកខ្លះ (មានចន្លោះសមត្ថភាពដែលសរសេរច្បាស់ ម្ចាស់ទទួលយក 2026-11-09, D14): %d។ ក្រុមដែលរួមក្នុងជួរតែមួយ នៅតែមាន acceptance test ដាច់ៗសម្រាប់មុខងាររងនីមួយៗ។</p>' % counts.get("partial", 0)
               + '<div class="tw"><table><thead><tr><th>ក្រុម</th><th>មុខងារ</th><th>IDM</th><th>NDM</th><th>GDM</th><th>ស្ថានភាព</th><th>ដំណាក់</th></tr></thead><tbody>'
               + "".join(_row(r) for r in ROWS) + "</tbody></table></div>")
