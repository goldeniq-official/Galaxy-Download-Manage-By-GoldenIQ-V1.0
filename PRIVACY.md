# គោលការណ៍ឯកជនភាព / Privacy policy

**ខ្មែរ** · [English](#english)

GDM គ្មាន telemetry គ្មាន account គ្មាន ID អ្នកប្រើ និងគ្មានការផ្សាយពាណិជ្ជកម្មទេ។

| អ្វី | កន្លែងរក្សា | ចេញពីកុំព្យូទ័រ? |
|---|---|---|
| បញ្ជីការទាញយក (link, ឈ្មោះ, ថត, ស្ថានភាព, SHA-256) | `%LOCALAPPDATA%\GoldenIQ.GalaxyDownloadManager\gdm.db` + backup ៧ ថ្ងៃ | ទេ |
| ការកំណត់ | `…\settings.json` ក្នុងថតដដែល | ទេ |
| Cookie/Authorization ដែល extension ផ្ញើសម្រាប់ link មួយ | memory តែប៉ុណ្ណោះ, ផ្ញើតែទៅ origin របស់ link នោះ | តែទៅ site ដែលអ្នកកំពុងទាញ |
| ការទាញក្នុង private/incognito window | Extension មិនដំណើរការក្នុង private window ទាល់តែសោះ | — |

**Request ដែល GDM ផ្ញើដោយខ្លួនឯង:** តែការពិនិត្យ update ប៉ុណ្ណោះ: `GET latest.json` និង `latest.json.sig` ពី GitHub (រាល់ថ្ងៃ ពេល update = ស្វ័យប្រវត្តិ ឬជូនដំណឹង)។ GitHub ឃើញ IP របស់អ្នក និងថាអ្នកប្រើ GDM ដូចការទស្សនាគេហទំព័រធម្មតា។ បិទបានក្នុង ការកំណត់ › ការធ្វើបច្ចុប្បន្នភាព › បិទ (គ្មាន request ទៀតទេ)។

**Extension** ផ្ញើទៅ GDM ក្នុងកុំព្យូទ័រតែ: link, referrer, ឈ្មោះស្នើ, ទំហំ, ប្រភេទ (mime) និង cookie របស់ link នោះ តាម native messaging (គ្មាន network)។ វាមិនអានមាតិកាទំព័រក្រៅពី `<a href>` ទេ។

**ការលុប:** ពេលលុប GDM ប្រអប់ "លុបបញ្ជីការទាញយក និងការកំណត់" លុបថតខាងលើ។ ឯកសារដែលអ្នកបានទាញមិនដែលត្រូវលុបទេ។ ការលុប job ពីបញ្ជីមិនមែនជា secure erase ទេ។

ទំនាក់ទំនង: GitHub issues ក្នុង repository នេះ។

<a id="english"></a>
## English

GDM has no telemetry, no account, no user ID and no ads.

- **Stored on your PC only:** the download list (links, names, folders, states, SHA-256) in `%LOCALAPPDATA%\GoldenIQ.GalaxyDownloadManager\gdm.db` with 7 daily backups, and `settings.json` next to it.
- **Cookies/Authorization** that the extension passes for a link live in memory only and are sent only to that link's own origin.
- **Private windows:** the extension does not run in private/incognito windows at all.
- **The only request GDM makes by itself** is the update check: `GET latest.json` and `latest.json.sig` from GitHub, daily when updates are Automatic or Notify. GitHub sees your IP address and that you use GDM, as with any website visit. Turn it off in Settings › Updates › Off (then GDM makes no request at all).
- **The extension** sends GDM, through native messaging on this PC (no network), only: link, referrer, suggested name, size, mime type and that link's cookie. It reads nothing from pages except `<a href>` links.
- **Uninstall:** the box "Delete the application data" removes the folder above. Files you downloaded are never deleted. Removing a job from the list is not a secure erase.

Contact: GitHub issues in this repository.
