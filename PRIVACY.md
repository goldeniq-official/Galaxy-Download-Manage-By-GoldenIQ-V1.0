# គោលការណ៍ឯកជនភាព / Privacy policy

**ខ្មែរ** · [English](#english)

GDM គ្មាន telemetry គ្មាន account គ្មាន ID អ្នកប្រើ និងគ្មានការផ្សាយពាណិជ្ជកម្មទេ។

| អ្វី | កន្លែងរក្សា | ចេញពីកុំព្យូទ័រ? |
|---|---|---|
| បញ្ជីការទាញយក (link, ឈ្មោះ, ថត, ស្ថានភាព, SHA-256, queue) | `%LOCALAPPDATA%\GoldenIQ.GalaxyDownloadManager\gdm.db` + backup ៧ ថ្ងៃ | ទេ |
| ការកំណត់ (រួមទាំង queue, proxy, ឈ្មោះអ្នកប្រើ site login, ផ្លូវកម្មវិធីដែលអ្នកជ្រើសឱ្យរត់ពេលបញ្ចប់) | `…\settings.json` ក្នុងថតដដែល | ទេ |
| ពាក្យសម្ងាត់ site login និង proxy | Windows Credential Manager (`GDM/site/…`, `GDM/proxy`) តែប៉ុណ្ណោះ; មិនដែលក្នុង settings, DB ឬឯកសារ export | ផ្ញើតែទៅ site/proxy ដែលអ្នកកំណត់វាសម្រាប់ |
| Cookie/Authorization ដែល extension ផ្ញើសម្រាប់ link មួយ | memory តែប៉ុណ្ណោះ, ផ្ញើតែទៅ origin របស់ link នោះ | តែទៅ site ដែលអ្នកកំពុងទាញ |
| Clipboard (ពេលអ្នកបើក "មើល clipboard", បិទលំនាំដើម) | មិនរក្សាទុក: GDM អានតែអត្ថបទ ≤ 8 KiB ហើយយកតែពេលវាជា link http(s) មួយដែលមានប្រភេទឯកសារដែលចាប់ (ឬទំព័រវីដេអូ YouTube ពេលមាន component វីដេអូ)។ មាតិកាដែលកម្មវិធីសម្គាល់ថា "កុំយកទៅ clipboard history" (ឧ. password manager) មិនត្រូវអានទាល់តែសោះ | ទេ |
| Component វីដេអូ (yt-dlp, FFmpeg) ពេលអ្នកយល់ព្រម | `…\components` | ទាញពី GitHub release ផ្លូវការរបស់គម្រោងទាំងនោះ |
| Task Scheduler "GDM wake <queue>" (តែពេលអ្នកបើក "ដាស់កុំព្យូទ័រ") | Windows Task Scheduler របស់អ្នក | ទេ |
| ការទាញក្នុង private/incognito window | Extension មិនដំណើរការក្នុង private window ទាល់តែសោះ | — |

**Request ដែល GDM ផ្ញើដោយខ្លួនឯង** (ក្រៅពីការទាញដែលអ្នកចាប់ផ្ដើម):
- ការពិនិត្យ update: `GET latest.json` និង `latest.json.sig` ពី GitHub (រាល់ថ្ងៃ ពេល update = ស្វ័យប្រវត្តិ ឬជូនដំណឹង)។ បិទបានក្នុង ការកំណត់ › ការធ្វើបច្ចុប្បន្នភាព › បិទ។
- `<ឯកសារ>.sha256` ពី server ដដែលនៃការទាញនីមួយៗ ដើម្បីផ្ទៀងផ្ទាត់ (បិទបានក្នុងការកំណត់)។
- Queue sync: request តូចមួយ (If-None-Match / If-Modified-Since) ទៅ server នៃឯកសារក្នុង sync queue តាមម៉ោងដែលអ្នកកំណត់។
- ពេលអ្នកយល់ព្រមដំឡើង component វីដេអូ: `components.json` (+ `.sig`) ពី GitHub របស់ GDM ហើយឯកសារពី release របស់ yt-dlp និង BtbN FFmpeg។
- ពេលអ្នកបិទភ្ជាប់ទំព័រវីដេអូ: yt-dlp ទៅមើលទំព័រនោះតាម relay ក្នុង GDM ដោយគ្មាន cookie ឬពាក្យសម្ងាត់ណាមួយ។
- Refresh address: GDM បើកទំព័រដើមក្នុង browser របស់អ្នក។

GitHub និង server ដែលអ្នកទាញពី ឃើញ IP របស់អ្នក ដូចការទស្សនាគេហទំព័រធម្មតា។ GDM ផ្ញើ User-Agent `GalaxyDownloadManager/<កំណែ>` លុះត្រាអ្នកកំណត់ផ្សេង។ ពេលប្រើ proxy, request ទាំងអស់ឆ្លង proxy ដែលអ្នកកំណត់។

**Extension** ផ្ញើទៅ GDM ក្នុងកុំព្យូទ័រតែ: link, referrer, ឈ្មោះស្នើ, ទំហំ, ប្រភេទ (mime) និង cookie របស់ link នោះ តាម native messaging (គ្មាន network)។ វាអានទំព័រតែ `<a href>` និង (តែលើគេហទំព័រដែលអ្នកបើក "បង្ហាញវីដេអូ") ចំនួន `<video>` ដើម្បីបង្ហាញលេខលើ icon។ បញ្ជីគេហទំព័រដែលអ្នកបិទ/បើក រក្សាក្នុង storage របស់ extension លើកុំព្យូទ័រនេះ។

**Export:** ឯកសារ JSON ដែលអ្នក export មានបញ្ជីការទាញ និងការកំណត់ ប៉ុន្តែគ្មានពាក្យសម្ងាត់ គ្មានកម្មវិធីដែលរត់ពេលបញ្ចប់ និងគ្មានផ្លូវថត។

**ការលុប:** ពេលលុប GDM ប្រអប់ "លុបបញ្ជីការទាញយក និងការកំណត់" លុបថតខាងលើ (រួមទាំង component)។ ពាក្យសម្ងាត់ក្នុង Credential Manager លុបបានក្នុង ការកំណត់ › Site logins ឬ Windows Credential Manager។ ឯកសារដែលអ្នកបានទាញមិនដែលត្រូវលុបទេ។ ការលុប job ពីបញ្ជីមិនមែនជា secure erase ទេ។

ទំនាក់ទំនង: GitHub issues ក្នុង repository នេះ។

<a id="english"></a>
## English

GDM has no telemetry, no account, no user ID and no ads.

- **Stored on your PC only:** the download list (links, names, folders, states, SHA-256, queues) in `%LOCALAPPDATA%\GoldenIQ.GalaxyDownloadManager\gdm.db` with 7 daily backups, and `settings.json` next to it (queues, proxy, site-login user names, the program you chose to run when a queue finishes).
- **Site-login and proxy passwords** live only in Windows Credential Manager (`GDM/site/…`, `GDM/proxy`), never in the settings, the database or an export, and are sent only to the site folder or proxy you set them for.
- **Cookies/Authorization** that the extension passes for a link live in memory only and are sent only to that link's own origin.
- **Clipboard monitor** (off by default): GDM reads at most 8 KiB of text and keeps it only when it is a single http(s) link with a caught file type (or a YouTube video page when the video component is installed). Content that its program marks as "do not keep in clipboard history" (password managers) is never read. Nothing is stored or logged.
- **Private windows:** the extension does not run in private/incognito windows at all.
- **Requests GDM makes by itself** (besides the downloads you start): the update check (`latest.json` + `.sig` from GitHub, daily when updates are Automatic or Notify; Settings › Updates › Off stops it); `<file>.sha256` from the same server as a download, to verify it (can be turned off); sync-queue checks (one small conditional request per file at the times you set); after you agree to install the video components, `components.json` (+ `.sig`) from GDM's GitHub and the files from the yt-dlp and BtbN FFmpeg releases; when you paste a video page, yt-dlp reads that page through GDM's relay without any cookie or password; Refresh address opens the original page in your browser. GitHub and the servers you download from see your IP address, as with any website visit. GDM sends the User-Agent `GalaxyDownloadManager/<version>` unless you set another. With a proxy, all requests go through the proxy you set.
- **Wake to download** (off by default) creates a Windows Task Scheduler task "GDM wake <queue>" on your PC.
- **The extension** sends GDM, through native messaging on this PC (no network), only: link, referrer, suggested name, size, mime type and that link's cookie. It reads nothing from pages except `<a href>` links and, only on sites where you turned on "Show videos", the number of `<video>` elements for the icon badge. Your site lists stay in the extension's storage on this PC.
- **Export:** an exported JSON file holds the list and the settings, but no passwords, no program to run and no folder paths.
- **Uninstall:** the box "Delete the application data" removes the folder above (including the components). Passwords in Credential Manager can be removed in Settings › Site logins or in Windows Credential Manager. Files you downloaded are never deleted. Removing a job from the list is not a secure erase.

Contact: GitHub issues in this repository.
