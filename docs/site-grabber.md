# ឧបករណ៍ចាប់ពីគេហទំព័រ (Site grabber)

[English below](#english)

ឧបករណ៍នេះរុករកគេហទំព័រមួយ ចាប់ពីទំព័រដែលអ្នកវាយ ហើយរាយឯកសារដែលត្រូវនឹងប្រភេទដែលអ្នកចង់បាន (រូបភាព វីដេអូ សំឡេង ឯកសារ ...)។ អ្នកធីកឯកសារដែលចង់បាន ហើយ GDM បន្ថែមវាទៅបញ្ជីទាញយក។ ឯកសារមិនត្រូវបានទាញទេ រហូតដល់អ្នកចុច "បន្ថែម"។

**បើក:** ម៉ឺនុយ **កិច្ចការ ▾ → ចាប់ពីគេហទំព័រ**។

## ជំហានទាំង ៤

1. **ទំព័រចាប់ផ្ដើម:** អាសយដ្ឋាន http ឬ https; គំរូ (ឯកសារទាំងអស់, រូបភាព, វីដេអូ, សំឡេង, ឯកសារការិយាល័យ, ឯកសារបង្ហាប់); ឈ្មោះចូល និងពាក្យសម្ងាត់ (បើគេហទំព័រសុំ: ផ្ញើទៅតែគេហទំព័រចាប់ផ្ដើមប៉ុណ្ណោះ, មិនរក្សាទុក); ទំព័រត្រូវជៀស។
2. **រក្សាទុកក្នុង:** ថត (ជ្រើសតាមប្រអប់ Windows), "រក្សារចនាសម្ព័ន្ធថតរបស់គេហទំព័រ", "ចាប់ផ្ដើមទាញភ្លាម" (បើមិនធីក ឯកសារត្រូវបានបន្ថែមដោយផ្អាក)។
3. **ការរុករក:** ជម្រៅលើគេហទំព័រនេះ (លំនាំដើម 3) និងលើគេហទំព័រផ្សេង (លំនាំដើម 0), គេហទំព័រផ្សេងដែលអនុញ្ញាត, subdomain, កុំរុករកថតមេ (លំនាំដើម បើក), path ដែលចង់/មិនចង់, គោរព robots.txt (លំនាំដើម បើក), រក link ក្នុង JavaScript (លំនាំដើម បិទ), វិនាទីរវាងទំព័រ (លំនាំដើម 1)។
4. **ប្រភេទឯកសារ:** ឧ. `*.jpg, *.png`; លាក់ឯកសារស្ទួន (ឈ្មោះដូចគ្នា)។

បន្ទាប់មក GDM បង្ហាញដំណើរការ (ចុច **ឈប់** បានគ្រប់ពេល) ហើយបង្ហាញឯកសារដែលរកឃើញជាមួយប្រអប់ធីក។

## សុវត្ថិភាព

- **អានតែប៉ុណ្ណោះ:** ឧបករណ៍ប្រើ GET តែប៉ុណ្ណោះ មិនបំពេញ ឬផ្ញើ form ទេ។ Link ដែលមាន logout, sign out, delete, remove ឬ unsubscribe មិនត្រូវបានបើកឡើយ (ក៏ពិនិត្យពេល redirect ដែរ)។
- **មិនដំណើរការ script:** ទំព័រត្រូវបានអានជាអត្ថបទ មិន render ក្នុង app ទេ។ ពេលបើក "JavaScript" GDM គ្រាន់តែរកអាសយដ្ឋានដែលសរសេរត្រង់ៗក្នុង script។
- **បណ្ដាញក្នុង:** link ទៅអាសយដ្ឋានក្នុង (192.168.x, 10.x, ...) ត្រូវរំលង លើកលែងតែ host ដែលអ្នកវាយខ្លួនឯង។ គេហទំព័រផ្សេងត្រូវរំលង លើកលែងតែវានៅក្នុងបញ្ជី "គេហទំព័រផ្សេងដែលអនុញ្ញាត"។
- **ដែនកំណត់ (ការពារអន្ទាក់ crawler):** 1 000 ទំព័រ (អតិបរមា 5 000), 5 000 សំណើ, ទំព័រ 100 MB សរុប (2 MB ក្នុងមួយទំព័រ), 15 នាទី, 50 000 អាសយដ្ឋាន, 50 កំណែ query ក្នុងមួយ path, path ដែលដដែលម្ដងហើយម្ដងទៀត (≥ 3 ដង) ត្រូវរំលង, មួយទំព័រក្នុងមួយវិនាទីក្នុងមួយគេហទំព័រ (ឬយឺតជាងនេះ បើ robots.txt សុំ)។
- **ថត:** ពេល "រក្សារចនាសម្ព័ន្ធ" ឈ្មោះថតនីមួយៗត្រូវបានសម្អាត (`..`, `:`, `\`, ឈ្មោះ CON ...) ហើយ junction/symbolic link ក្នុងថតត្រូវបដិសេធ ដូច្នេះគ្មានឯកសារណាត្រូវសរសេរក្រៅថតដែលអ្នកជ្រើសឡើយ។

## អ្វីដែលមិនទាន់មាន (កំណែនេះ)

រក្សា project ទុកបើកម្ដងទៀត, កាលវិភាគ/sync របស់ grabber, បម្លែង link ទៅជា local (មើលក្រៅបណ្ដាញ), តម្រងទំហំឯកសារ, ការ render JavaScript, login តាម cookie/Digest។ (កត់ក្នុង `docs/decisions.md`)

---

<a id="english"></a>
# Site grabber (English)

The site grabber explores a website from the page you type and lists the files that match the types you want (images, video, audio, documents ...). You tick the ones you want and GDM adds them to the download list. Nothing is downloaded until you press **Add**.

**Open it:** **Tasks ▾ → Run site grabber**.

## The 4 steps

1. **Start page:** an http or https address; a template (all files, images, video, audio, documents, archives); login name and password (only if the site asks: sent to the start page's own site only, never stored); pages to avoid.
2. **Save to:** the folder (chosen in the Windows folder dialog), "Keep the site's folder structure", "Start downloading right away" (otherwise the files are added paused).
3. **Explore:** depth on this site (default 3) and on other sites (default 0), other sites allowed, subdomains, do not explore parent folders (default on), only/never these paths, respect robots.txt (default on), look for links in JavaScript (default off), seconds between pages (default 1).
4. **File types:** e.g. `*.jpg, *.png`; hide duplicates (same name).

GDM then shows its progress (press **Stop** at any time) and the files it found, with check boxes.

## Safety

- **Read only:** GET requests only; forms are never filled or sent. Links containing logout, sign out, delete, remove or unsubscribe are never opened (redirects are checked too).
- **No scripts run:** pages are read as text and never rendered inside the app. With "JavaScript" on, GDM only picks up addresses written literally in the script text.
- **Local network:** links to private addresses (192.168.x, 10.x, ...) are skipped, except the host you typed yourself. Other sites are skipped unless they are in "Other sites allowed".
- **Limits (crawler traps):** 1 000 pages (5 000 maximum), 5 000 requests, 100 MB of pages in total (2 MB per page), 15 minutes, 50 000 addresses, 50 query variants per path, repeated path segments (3 times or more) skipped, one page per second per site (slower if robots.txt asks).
- **Folders:** with "Keep the site's folder structure" every folder name is cleaned (`..`, `:`, `\`, names like CON ...) and junctions or symbolic links in the tree are refused, so nothing is written outside the folder you chose.

## Not in this version

Saved projects to reopen, grabber schedule/sync, converting links for offline browsing, file size filters, rendering JavaScript, cookie/Digest logins. (Recorded in `docs/decisions.md`.)
