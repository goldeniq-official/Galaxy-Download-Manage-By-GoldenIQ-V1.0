# Galaxy Download Manager 1.0.0

**ខ្មែរ** · [English](#english)

## អ្វីថ្មី

- **ទាញលឿន និងត្រឹមត្រូវ:** ការតភ្ជាប់ច្រើន (ចាប់ពី 2 ដល់ 8, អតិបរមា 32, ប្ដូរបានពេលកំពុងទាញ), បន្តក្រោយដាច់បណ្ដាញ ឬបិទកុំព្យូទ័រ, SHA-256 គ្រប់ឯកសារ; ក្នុង test (kill process 1000 ដង, server អាក្រក់) គ្មានឯកសារខុសណាមួយទទួលឈ្មោះចុងក្រោយទេ។
- **សុវត្ថិភាព:** Mark-of-the-Web + ការពិនិត្យ antivirus, ឯកសារគ្រោះថ្នាក់មិនបើកដោយផ្ទាល់, ការទាញពីបណ្ដាញក្នុង (LAN) សួរមុន, certificate ខុស ⇒ ឈប់ (គ្មានប៊ូតុងរំលង)។
- **Browser:** extension Chrome/Edge (និង Brave, Vivaldi, Opera) និង Firefox, <kbd>Ctrl</kbd>+ចុច = GDM, <kbd>Alt</kbd>+ចុច = browser, ម៉ឺនុយ "ទាញដោយ GDM" / "ទាញ link ទាំងអស់", popup "កុំចាប់លើគេហទំព័រនេះ"។
- **ផ្ទាំង:** បញ្ជី (រាប់ម៉ឺនជួរបាន), ព័ត៌មានលម្អិត + segment bar, ផ្ទាំង Download Info ដូច IDM (ទាញភ្លាមខណៈអ្នកជ្រើស), ការកំណត់, ការណែនាំលើកដំបូង ៣ ជំហាន, ស្វែងរកខ្មែរ/ឡាតាំង, theme ភ្លឺ/ងងឹត, tray, កំណត់ល្បឿនសរុប និងក្នុងមួយឯកសារ។
- **Update** ចុះហត្ថលេខា ពិនិត្យរាល់ថ្ងៃ ដំឡើងតែពេល GDM ទំនេរ (បិទបាន)។

## ដែនកំណត់ដែលដឹងរួច

1. **គ្មានហត្ថលេខា Authenticode** (SignPath ទទួលយកក្រោយ release សាធារណៈ) ⇒ SmartScreen អាចព្រមាន។ ផ្ទៀងផ្ទាត់ `SHA256SUMS.txt` និង `gh attestation verify`។
2. **ល្បឿនធៀប IDM មិនទាន់បញ្ជាក់:** GDM មិនអះអាងថាលឿនស្មើ IDM ទេ។ លើម៉ាស៊ីនសាក (ក្រៅលក្ខខណ្ឌ protocol) GDM យឺតជាង IDM (geo-mean 0.53–0.66 លើ LAN 1 Gbit/s) ការវាស់របស់យើងបង្ហាញថាពេលបន្ថែមភាគច្រើនមកពីការ flush ទៅ disk និងការអាន SHA-256 ក្រោយ antivirus (IDM មិនធ្វើទាំងពីរ)។ មើល `docs/evidence/G5.md` ផ្នែក ៥។
3. Browser ដែលបានសាក: Chromium និង Edge។ Opera/Brave/Vivaldi ចុះឈ្មោះ តែមិនទាន់សាក។ Extension មិនទាន់នៅក្នុង Chrome Web Store / Edge Add-ons / AMO ទេ (ម្ចាស់បោះពុម្ព)។ Build Firefox មិនទាន់សាកក្នុង Firefox ពិតក្នុង G6 (native messaging Firefox បានបង្ហាញក្នុង G2)។
4. មិនទាន់មាន: User-Agent តាម site, ថតបណ្ដោះអាសន្ន, ទាញ `.sha256` ស្វ័យប្រវត្តិ, site pattern ក្នុង category, បង្ហាញ/លាក់ column, កែ URL ក្នុង Properties, video grabber, queue + scheduler, site login, proxy PAC/NTLM, CLI `/d`, តាមដាន clipboard, drop basket, អូសឯកសារចេញពី toast, ផ្ទាំង mini "នៅលើគេជានិច្ច", ផ្ទៀងផ្ទាត់ hash ដោយបិទភ្ជាប់ (P2)។
5. ប្ដូរឈ្មោះ ឬ category ក្នុងផ្ទាំង Download Info ចាប់ផ្ដើមការទាញឡើងវិញ (link ដែលប្រើបានតែម្ដងអាចបរាជ័យ)។
6. បើ GDM ឆ្លើយយឺតជាង 3 វិ browser ក៏បន្តទាញដែរ ⇒ អាចមានឯកសារពីរ (ក្នុង test kill មិនបាត់ទេ)។
7. បន្ថយចំនួនការតភ្ជាប់ពេលកំពុងទាញ មានប្រសិទ្ធភាពពេល segment បច្ចុប្បន្នបញ្ចប់។
8. Update ដំឡើងជំនួសកំណែចាស់ (NSIS) គ្មានការត្រឡប់ទៅកំណែមុនដោយស្វ័យប្រវត្តិ; មិនទាន់កំណត់ល្បឿន update 20% ឬផ្អាកលើ metered។
9. Windows 10 និង arm64 មិនទាន់សាក; ការគាំទ្រផ្លូវការ = Windows 11 x64។ ការដាច់ភ្លើងមិនទាន់សាក (សាកតែ kill process ×1000)។

<a id="english"></a>
## English

### New

- **Fast and correct downloads:** several connections (2 → 8, up to 32, changeable while downloading), resume after network loss or a restart, SHA-256 for every file; in testing (1000 process kills, hostile servers) no wrong file ever got its final name.
- **Safety:** Mark-of-the-Web + antivirus check, dangerous files never open directly, local-network (LAN) downloads ask first, bad certificates stop (no bypass button).
- **Browsers:** extensions for Chrome/Edge (also Brave, Vivaldi, Opera) and Firefox, <kbd>Ctrl</kbd>+click = GDM, <kbd>Alt</kbd>+click = browser, "Download with GDM" / "Download all links" menus, popup "Don't catch on this site".
- **Screens:** list (tens of thousands of rows), details with a segment bar, an IDM-style Download Info dialog (downloading starts while you choose), settings, 3-step first run, Khmer/Latin search, light/dark themes, tray, total and per-file speed limits.
- **Signed updates**, checked daily, installed only when GDM is idle (can be turned off).

### Known limitations

1. **No Authenticode signature** yet (SignPath accepts projects after a public release): SmartScreen may warn. Check `SHA256SUMS.txt` and `gh attestation verify`.
2. **Speed vs IDM not proven:** GDM makes no IDM-parity claim. On the test PC (outside the benchmark protocol's disk requirement) GDM is slower than IDM (geo-mean 0.53–0.66 on a 1 Gbit/s LAN) our measurements attribute most of the extra time to flushing to disk and re-reading for SHA-256 after the antivirus check (IDM does neither). See `docs/evidence/G5.md` §5.
3. Tested browsers: Chromium and Edge. Opera/Brave/Vivaldi are registered but untested. The extension is not yet in the Chrome Web Store / Edge Add-ons / AMO (the owner publishes it). The Firefox build was not exercised in a real Firefox in G6 (Firefox native messaging was proven in G2).
4. Not yet: per-site User-Agent, temporary folder, automatic `.sha256` check, category site patterns, column show/hide, editing a URL in Properties, video grabber, queues + scheduler, site logins, PAC/NTLM proxy, `/d` CLI, clipboard watching, drop basket, dragging a file out of a toast, always-on-top mini window, paste-a-hash verification (P2).
5. Renaming or re-categorising in the Download Info dialog restarts the download (single-use links may fail).
6. If GDM answers the browser later than 3 s, the browser keeps downloading too: you may get two copies (none was lost at the tested crash points).
7. Lowering the connection count while downloading takes effect as the current segments finish.
8. Updates install in place (NSIS); no automatic rollback to the previous version; the 20 % update bandwidth cap and metered-network pause are not in this release yet.
9. Windows 10 and arm64 are untested; supported: Windows 11 x64. Power loss is untested (process kill ×1000 only).
