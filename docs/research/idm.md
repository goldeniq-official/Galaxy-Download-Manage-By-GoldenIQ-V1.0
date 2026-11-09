# Internet Download Manager (IDM): feature inventory and user analysis

Research date: 2026-11-09. Subject: IDM by Tonec (Tonec FZE / Tonec Inc.), Windows only. Latest build is **6.43 Build 18 (Nov 04, 2026)** ([changelog](https://www.internetdownloadmanager.com/news.html)).

Labels used below:
- **[OFF]**: confirmed on an official Tonec page (cited).
- **[3P]**: confirmed by a third-party source (cited).
- **[UNVERIFIED]**: from general product knowledge or screenshots I could not open. Check it against a real install before relying on it.

Notes on sources:
- reddit.com cannot be reached by this research agent. No Reddit quotes are included, and none have been made up.
- VideoHelp reviews, Trustpilot, voz.vn, Kaskus and Zhihu were not reachable or had no IDM content. User quotes come from AlternativeTo, Mozilla AMO and other forums instead.

---

## 0. Release cadence (context)

- **Build rate:** about 10 builds in 6.43 (Jun to Nov 2026) and about 35 builds in 6.42 (Nov 2023 to May 2026).
- **What the builds fix:** almost all of them fix video detection, the "download panel missing" bug, **403 Forbidden** errors, TS to MP4 conversion, or Explorer crashes. The 403 fixes alone appear in about 15 builds. [OFF] [news](https://www.internetdownloadmanager.com/news.html)
- **Lesson for the competitor:** the work never ends because sites keep changing. A competitor needs an updatable site-extractor layer (yt-dlp-style), not hard-coded logic.
- **Notable recent changes:** [OFF] news
  - 6.42.42 (Jul 2025): Chrome extension moved to **Manifest V3**.
  - 6.42.1 (Nov 2023): Windows ARM browser integration.
  - 6.43.1 (Jun 2026): MP4 offered where only TS was available.
  - 6.43.10 (Aug 2026): "Improved HTTP/2 downloading".
  - 6.43.18 (Nov 2026): faster video conversion. Also fixed "antivirus screen protection hiding the panel".

---

## 1. Feature inventory

### 1.1 Download engine

| Feature | What it does / how users interact | Source |
|---|---|---|
| **Dynamic segmentation** | Splits the file *while* downloading. When a connection frees up, it takes half of the largest remaining segment, so every connection stays busy. Tonec markets this as "up to 8 times" faster. It reuses open connections, so it skips repeated connect and login steps. | [OFF] [features2](https://www.internetdownloadmanager.com/features2.html) |
| **Overlap integrity check** | Downloads extra bytes at the edges of each part and compares them with the neighbouring part. On a mismatch it re-downloads that part. If several connections mismatch, it falls back to **one connection for the whole file**. | [OFF] [problems9](https://www.internetdownloadmanager.com/register/new_faq/problems9.html) |
| **Max connections per download** | Global setting in Options → Connection. The values are 1/2/4/8/16/24/32, and the default is 8. | values and default [UNVERIFIED] |
| **Per-server connection exceptions** | Options → Connection → "Exceptions" table sets a fixed number of connections per server, for example 1 for FTP servers that allow only 1 or 2. | [OFF] [options](https://www.internetdownloadmanager.com/support/using_idm/options.html) |
| **Download limits (quota)** | Fair-use caps such as "40 MB per hour" or "150 MB every 4 hours". IDM pauses at the cap and resumes later on its own. Available globally and per queue. | [OFF] options, [scheduler](https://www.internetdownloadmanager.com/support/idm-scheduler/idm_scheduler.html) |
| **Speed Limiter** | Opened from Downloads → Speed Limiter → Settings / Turn on / Turn off, or from the **tray icon right-click menu**. The limit is in KB/s and applies **per file, not to the total** ("total = number_of_downloads × set_speed"). Option "Always turn on Speed Limiter on IDM startup". Tonec warns against values under 10 KB/s because servers may drop the connection. The progress dialog also has a per-download limiter tab [UNVERIFIED]. | [OFF] [functions11](https://www.internetdownloadmanager.com/register/new_faq/functions11.html), [3P] [itstillworks](https://itstillworks.com/limit-transfer-rate-internet-download-manager-32908.html) |
| **Resume** | Continues after dropped connections, shutdowns or power loss. The "Resume capability: Yes/No" field comes from the server's range support. A FAQ covers resume failing even when "Yes" is shown. | [OFF] features2, [problems3](https://www.internetdownloadmanager.com/register/new_faq/problems3.html) |
| **Refresh download address** | For expired or one-time links: right-click → "Refresh download address". IDM opens the page where the link was found. The user clicks the download again and IDM catches the new link. If IDM is unsure, it asks "is this the link I'm waiting for?". The download then resumes on the new URL. | [OFF] [sites2_3](https://www.internetdownloadmanager.com/register/new_faq/sites2_3.html) |
| **Ignore modification-time changes on resume** | Checkbox in Options → Downloads. It stops a resume from restarting at zero when the server changes the file timestamp. | [OFF] options |
| **Protocols** | HTTP, HTTPS, FTP, MMS. HTTP/2 is supported (6.43.10 notes say "improved HTTP/2 downloading"). | [OFF] features2, news |
| **HTTP/3 / QUIC** | Never mentioned on any official page. **[UNVERIFIED]**: likely not supported. | — |
| **FTP PASV** | "Use FTP in PASV" option (Proxy tab). | [OFF] options |
| **User-Agent** | "User agent for manually added downloads" textbox. Downloads caught from a browser reuse that browser's UA. Cookies, referrer and POST data are also passed from the browser (see privacy policy). | [OFF] options, [privacy](https://www.internetdownloadmanager.com/privacy.html) |
| **Temporary folder** | Options → Save To → "Temporary Directory". Parts are stored there and assembled or moved to the target at the end. The default is under `%APPDATA%\IDM\DwnlData` [UNVERIFIED path]. Tonec documents that a file can "stay in temp folder" when the target is unreachable. | [OFF] options, [temp FAQ](https://www.internetdownloadmanager.com/register/new_faq/IDM-stores-file-in-temporary-folder.html) |
| **Rebuilding temp files** | A known slow step, plus a "temporary files cannot be rebuilt" error (FAQ items problems5/problems8). | [OFF] [functions FAQ index](https://www.internetdownloadmanager.com/register/new_faq/functions_main.cgi) |
| **Not supported** | No torrents ("for legal reasons", [functions4](https://www.internetdownloadmanager.com/register/new_faq/functions4.html)). No SFTP ([AlternativeTo](https://alternativeto.net/software/internet-download-manager/about)). No macOS, Linux, Android or iOS ([functions2](https://www.internetdownloadmanager.com/register/new_faq/functions2.html)). No RTSP (planned) and no DRM streams ([video panel FAQ](https://www.internetdownloadmanager.com/register/new_faq/video-download-panel-missing.html)). | [OFF] |

### 1.2 Browser integration

| Feature | Detail | Source |
|---|---|---|
| **Browsers** | Edge, Chrome, Firefox, Opera, IE, Safari (old), MSN Explorer, AOL, Avant, Maxthon and others. Chromium forks such as Brave and Vivaldi also work. "Add Browser…" in Options → General lets the user point to any browser exe. | [OFF] features2, options |
| **Extension** | "IDM Integration Module": Chrome Web Store id `ngpampappnmepgilojfohadhhmbhlaek`, which is **hidden from store search**. A Firefox AMO version also exists, rated 4.6/5 from 3,843 reviews. Tonec says every other "IDM" extension in the store is fake. | [OFF] [chrome_extension](https://www.internetdownloadmanager.com/register/new_faq/chrome_extension.html), [AMO](https://addons.mozilla.org/en-US/firefox/addon/tonec-idm-integration-module/reviews/) |
| **Extension permissions** | Read/change all site data (detect links, parse page source for media), tabs/activity (place panel over players), privacy settings (read only), "communicate with cooperating websites" (self-check against Tonec site), **native messaging** (hand downloads to IDMan.exe), manage extensions (detect conflicting extensions), downloads. | [OFF] [chrome_perms](https://www.internetdownloadmanager.com/register/new_faq/chrome_perms.html) |
| **Incognito** | Not on by default. The user must tick "Allow in incognito". | [OFF] chrome_extension |
| **Basic vs Advanced integration** | Advanced mode catches downloads from **any application**, not just browsers. It appears to use a signed WFP kernel driver, `idmwfp.sys` [3P] [freefixer](https://www.freefixer.com/library/file/idmwfp.sys-288453). Basic mode works through browser extensions only. Options → General. | [OFF] options, features2 |
| **IE-based click monitor** | "Run module for click monitoring for IE-based browsers" (IEMonitor.exe). | [OFF] options |
| **Catch rules** | Options → File Types holds the list of extensions IDM takes over (long default list of archives, exe, media, iso, etc.). | [OFF] options |
| **Sites exception list** | "Don't start downloading automatically from the following sites", with wildcards such as `*.tonec.com`, edited through "Edit list…". There is also a separate **address exceptions** list. | [OFF] options |
| **Special keys** | Ctrl+click **forces** IDM. Alt+click **bypasses** IDM. Both are changeable via Options → General → "Keys". | [OFF] [starting](https://www.internetdownloadmanager.com/support/using_idm/starting.html) |
| **Context menu** | "Download with IDM" on a link. "Download all links with IDM" on selected text or the whole page ("Download All" feature). | [OFF] starting, features2 |
| **Download panel ("Download this video")** | Floating button over web players, with a quality dropdown (360p … 4K/8K, audio-only). Options → General → "Customize IDM Download panels in browsers" → Edit. That dialog has an **Exceptions** list (domain, `*` wildcard) and "Don't start downloading from web-players automatically". | [OFF] [video18](https://www.internetdownloadmanager.com/register/new_faq/video18.html), [bi18](https://internetdownloadmanager.com/register/new_faq/bi18.html), [video14](https://aaa.internetdownloadmanager.com/register/new_faq/video14.html) |
| **Panel requests** | Users ask for "disabled by default, remember per-site toggle" (AMO, davehyun, Aug 2026). | [3P] AMO |
| **Selected-link panel** | A panel can also appear over selected links (bi18). | [OFF] |
| **Cookie / credential handoff** | The extension passes "URL, cookies, encrypted credentials, query parameters, post data" to IDM. Tonec states it is sent only to the origin site. | [OFF] [privacy](https://www.internetdownloadmanager.com/privacy.html) |

### 1.3 Video grabber

- **How it works:** detects HTTP media, HLS/DASH/TS streams and separate audio and video tracks, then muxes them to MP4 or MKV. Since 6.43, TS is converted to MP4.
- **Ultra HD:** 4K and 8K recognition since 6.42.56/58.
- **History:** "Download FLV video" dates from the Flash era. [OFF] news
- **MKV vs MP4:** MKV playback complaints led to an MP4 option ([bi28](https://www.internetdownloadmanager.com/register/new_faq/bi28.html)).
- **Limits:**
  - No private or protected YouTube.
  - No DRM. A user complains "IDM cannot download this protected stream for legal reasons" ([AlternativeTo](https://alternativeto.net/software/internet-download-manager/about)).
  - Google Drive video through the panel is unsupported. Tonec says to use Drive's own download icon instead ([GDrive FAQ](https://www.internetdownloadmanager.com/register/new_faq/download-from-google-drive.html)).
  - YouTube Shorts work only partly. One user estimated "about 60%" (AMO, Oct 2026).

### 1.4 Site Grabber / spider ([intro](https://www.internetdownloadmanager.com/support/idm-grabber/idm_grabber.html), [wizard](https://www.internetdownloadmanager.com/support/idm-grabber/grabber_wizard.html), [settings](https://www.internetdownloadmanager.com/support/idm-grabber/grabber_settings.html)) [OFF]

The grabber runs as a **4-step wizard**:

1. **Start page:** start URL, login and password. Under Advanced: manual login page (opens a browser to sign in) and **logout pages to avoid**. Also the project name and a **template**: all images, all video, all audio, complete site, or user templates via "Project → Save current settings as a template".
2. **Save location:**
   - By category folder, all into one category, or **keep original relative subfolders**.
   - **Convert links to local** (offline browsing).
   - Overwrite existing, or append `_2`.
   - "Add checked files to IDM" main list.
3. **Explore filters:**
   - Depth on this site and on other sites.
   - Ignore popups, don't explore parent directories, explore all subdomains of the main domain.
   - **JavaScript parsing** ("use with caution").
   - Include/exclude path and domain filters with `*`.
4. **File filters:**
   - Type lists, comma-separated, `*` allowed, plus a `<start page>` token.
   - Search on this site only.
   - **Hide duplicates** (same name and size).
   - Auto-start downloading all matches.
   - Under Advanced: path and domain include/exclude, and **min/max file size**.

Other grabber behaviour:
- **Settings:** number of files to explore, number of simultaneous downloads, link text used as the description, "Look up files in IE cache before downloading" (on by default).
- **Projects:** saved under the "Grabber projects" node in the categories tree. Right-click offers Open, Schedule and Delete.
- **Scheduling:** run once, stop at a time, or repeat to sync. The grabber has its own scheduler.

### 1.5 Scheduler and queues ([scheduler](https://www.internetdownloadmanager.com/support/idm-scheduler/idm_scheduler.html), [queues](https://www.internetdownloadmanager.com/support/idm-scheduler/idm_queues.html)) [OFF]

- **Queue types:**
  - **Main download queue:** items leave the queue when done. Icon: 3 yellow files.
  - **Main synchronization queue:** keeps its files and re-downloads them if they change on the server. Icon: 3 green files.
  - Unlimited custom queues of either type (2-file icons). A clock overlay means the queue is scheduled.
  - The type can be switched only for custom queues.
- **Per queue:**
  - Start Now. Start at a set time. **Days of week / daily** [UNVERIFIED; the page does not say].
  - Stop at a set time.
  - Periodic sync every N minutes or hours, until the stop time or the end of the day.
  - **N files at the same time.**
  - Retry count per failed file. By default a failed file moves to the end of the queue. If it is the only file left, IDM retries every 30 s forever.
  - Download quota.
- **On completion:** hang up the modem, **turn off the computer**, force processes to terminate, or **open a file/program** (a .bat for several actions). These do not run if the user stopped the queue manually. Hibernate and sleep, plus a "wake to download" option, are covered in FAQ functions22 ([index](https://www.internetdownloadmanager.com/register/new_faq/functions_main.cgi)). Exit IDM when done is also an option [UNVERIFIED].
- **UI:**
  - The Scheduler dialog has two tabs, "Schedule" and "Files in the queue". Files can be reordered by drag and drop between queues.
  - Toolbar Start Queue / Stop Queue buttons, with dropdowns for the other queues.
  - Downloads menu submenus, and a right-click menu on queues in the categories tree.
  - "Show queue selection panel" when choosing Download Later.
  - Changes to queue contents save immediately. Other changes save on Apply, on switching queues, or on closing the dialog.

### 1.6 Categories and automatic folders

- **Default categories:** General, Compressed, Documents, Music, Programs, Video. The page names Music, Video, Programs and Documents [OFF]; Compressed and General are [UNVERIFIED].
- **Category settings:** each category has a name, a **default folder**, **file types**, and optionally **site patterns** (wildcard domains). Categories can be added, edited and deleted.
- **Tree filters:** the tree also shows "Unfinished", "Finished", "Queues" and "Grabber projects" nodes [UNVERIFIED for Unfinished/Finished; OFF for Grabber projects].
- **Remember last folder:** "Change folder for … category on last selected" makes a folder changed in the Download File Info dialog the new default.
- [OFF] [main dialog](https://www.internetdownloadmanager.com/support/using_idm/using_idm.html), options.

### 1.7 Proxy / SOCKS [OFF] options

- **Get System:** copies proxy settings from the browser.
- **Configuration:** an auto-config (PAC) script, or a manual proxy per protocol (HTTP, HTTPS, FTP, SOCKS).
- **Follow the browser:** "Use proxy/socks from a browser's request" uses the proxy of the browser the download came from.
- **Bypass list:** wildcard sites that skip the proxy.
- **Login:** proxy username and password.
- **Also supported:** Microsoft ISA and FTP proxies. A VPN FAQ exists ([configure-VPN](https://www.internetdownloadmanager.com/register/new_faq/configure-VPN-in-IDM.html)).

### 1.8 Authentication and login manager

- **Protocols:** Basic, Negotiate, NTLM, Kerberos ([features2](https://www.internetdownloadmanager.com/features2.html)).
- **Site Logins tab:** list of URL + user + password, with **different logins per directory** of the same site. A "New" button adds entries.
- **Add URL dialog:** optional "Use authorization" with user and password. Credentials can also be edited in Properties.
- [OFF] options, main dialog.

### 1.9 Dial-up / VPN tab [OFF] options

- Pick a Windows RAS connection, user, password and "Save password".
- Redial count (0 = endless) and redial interval.
- Legacy feature, but VPN users still rely on it.

### 1.10 Antivirus scan after download [OFF] options, [features2](https://www.internetdownloadmanager.com/features2.html)

- Options → Downloads → Virus checking → Edit: path to the scanner exe plus command-line parameters, using the `[File]` placeholder.
- Known scanners (Avast, AVG, McAfee, Norton, Spybot) are pre-filled. With no parameters, IDM appends the full path.
- **Competitor note:** Windows' built-in AMSI or Mark-of-the-Web zone tagging would be the modern equivalent. **[UNVERIFIED]** whether IDM writes the MotW `Zone.Identifier`.

### 1.11 Clipboard monitoring [OFF] starting, options

- On by default (Options → General → system integration).
- When a URL with a monitored extension is copied, the Download File Info dialog pops up. OK starts the download.

### 1.12 Command line ([official](https://www.internetdownloadmanager.com/support/command_line.html)) [OFF]

```
idman /s                                   start the scheduler queue
idman /d URL [/p path] [/f filename] [/q] [/h] [/n] [/a]
  /d URL   download URL
  /p path  local folder
  /f name  local file name
  /q       exit IDM after successful download (only first running copy)
  /h       hang up connection after successful download
  /n       silent mode (no questions)
  /a       add to queue only, don't start
```

- `/a /h /n /q /f /p` work only together with `/d`.
- Tonec **forbids third-party store extensions from calling the command line** and may block them.
- **Gaps:** no JSON/RPC API, no headers/cookies/referer switches, no list output.
- **Silent install:** `/skipdlgs` per FAQ functions21 [UNVERIFIED switch name].

### 1.13 Dialogs

- **Download File Info dialog** (on capture) [OFF] [starting](https://www.internetdownloadmanager.com/support/using_idm/starting.html):
  - Shows URL, category (with "+" to add one), **Save As** path, "Remember this path for X category", description, and file size [UNVERIFIED field list].
  - Buttons: **Start Download**, **Download Later** (with queue picker), **Cancel**, and **Preview** (zip contents before download).
  - Option: "Start download immediately while showing Download File Info dialog". IDM starts fetching while the user is still choosing a folder, which hides latency.
- **Download progress dialog** [mostly UNVERIFIED; the official page does not detail it]:
  - **Tabs:** Download status (URL, status, size, downloaded, speed, time left, resume capability), **Speed Limiter**, and **Options on completion** (show complete dialog, hang up, shut down, exit).
  - **Buttons:** Pause/Resume, Cancel, and "Show details >>". Details show the **per-connection list** (start position, bytes, status) and a **segment bar** with each connection coloured.
  - **Start view:** normal, minimized, or "Don't show" [OFF options].
- **Download Complete dialog** [OFF] [completeD](https://www.internetdownloadmanager.com/support/using_idm/completeD.html):
  - Buttons: Open, Open with… [UNVERIFIED], Open folder [UNVERIFIED], Close.
  - "Don't show this dialog again", which can be re-enabled in Options → Downloads.
  - A **drag icon** to drag the file into another app (for example, drop onto a player to queue it without replacing the playlist).
- **Silent mode:** turn off the start dialog and the complete dialog, and set progress to "Don't show". Files then save straight to the category folder [OFF options].
- **Properties dialog:** opens on double-click of a list item. Shows read-only info, plus editable save path, **URL**, description and login, and an Open button [OFF [properties](https://www.internetdownloadmanager.com/support/using_idm/properties.html)].
- **Add URL dialog:** has URL history, which can be cleared (FAQ functions16), and an optional login [OFF].
- **Batch download:** Tasks → Add batch download. A URL pattern with `*` replaced by a numeric (with zero-padding width) or letter range, from-to. Also a batch from clipboard or text [UNVERIFIED, the label is not on any official page reached].
- **Find:** Downloads → Find. The search box had CVE-2020-28964 ([intruder](https://cvemon.intruder.io/cves/CVE-2020-28964)).

### 1.14 Main window, toolbar, list [OFF] [main dialog](https://www.internetdownloadmanager.com/support/using_idm/using_idm.html)

- **Toolbar:** Add URL, Resume, Stop, Stop All, Delete, Delete Completed, Options, Scheduler, Start Queue ▾, Stop Queue ▾, Grabber, Tell a Friend.
  - Buttons are enabled or disabled based on the selection.
  - Right-click the toolbar for 4 looks (large/small × 3D/Neon), downloadable or custom skins, and "Customize…" to add, remove and reorder buttons.
- **List:** columns for file name, Q (queue icon), size, status (% or "Complete"), time left, transfer rate, last try date, description.
  - Sort by clicking a header. Show, hide and reorder columns via drag or a Columns dialog.
  - The context menu mirrors the Downloads menu: Open, Open with, Open folder, Move/rename, Redownload, Resume, Stop, **Refresh download address**, Remove, Add to/Delete from queue, Properties [partly UNVERIFIED].
- **Menus:**
  - Tasks: Add URL, Add batch download, **Export / Import**, Run site grabber, **Show drop target**, Exit.
  - File, Downloads, View (Hide categories, Arrange files, **Dark Mode support**, Language), Help (About, Check for updates, Registration, Tutorials).
  - [View/Dark mode: 3P [neowin 6.37b8](https://www.neowin.net/software/internet-download-manager-idm-637-build-8-beta/). Rest partly UNVERIFIED.]
- **Delete with file:** an option to delete files from disk when removing entries (FAQ functions15) [OFF index].
- **Tray icon:** menu with Speed Limiter, queue start/stop, show IDM, Exit [OFF for the Speed Limiter item, rest UNVERIFIED].

### 1.15 Drag and drop / basket [OFF] starting, features2

- Drag links onto the main window or onto the **drop target** floating basket. Restore the basket via Tasks → Show drop target.
- Drag completed files **out** of IDM, from the list or the complete dialog.

### 1.16 Import / export

- **Import:** Tasks → Import → From text file ([import_downloads](https://idm.internetdownloadmanager.com/support/import_downloads.html)).
- **Export/import format:** the proprietary **`.ef2`** file (CVE-2020-23060 was in this parser).
- **Settings migration:** the official advice is to **export registry key `HKCU\Software\DownloadManager`** and copy the temp folder and category folders across ([functions17](https://www.internetdownloadmanager.com/register/new_faq/functions17.html)). There is no built-in settings backup.

### 1.17 Sounds [OFF] options

- Each event (download complete, failed, queue start/stop, etc.) gets an optional WAV file.
- Checkbox to enable each one, plus Browse and Play buttons.

### 1.18 Windows integration

- Autostart with Windows, a tray icon and Explorer integration. Recent builds fixed **Windows Explorer crashes** (6.42.18, 6.43.5), which implies an in-process shell extension DLL [OFF news].
- Network-app hooks: a legacy IDM DLL "may crash several network applications" ([idmmbc](https://www.internetdownloadmanager.com/support/idmmbc.html)).
- The WFP driver `idmwfp.sys`. A Windows 11 BSOD has been attributed to it ([MS Q&A](https://learn.microsoft.com/en-us/answers/questions/4149979/av-w-(null)-idmwfp-unknown-function-crash-code-win)).
- Controlled Folder Access conflicts: users must allow IDMan.exe ([temp FAQ](https://www.internetdownloadmanager.com/register/new_faq/IDM-stores-file-in-temporary-folder.html)).
- Windows ARM integration since 6.42.1.

### 1.19 Languages

- 50+ language packs ([Wikipedia](https://en.wikipedia.org/wiki/Internet_Download_Manager)), switched via View → Language. A FAQ covers switching back to English (functions13).
- Non-English installs **download missing CHM help files over HTTP**, which is the RCE vector in section 4.

### 1.20 Updates / "Quick Update"

- Checks about weekly, lists what changed and asks before updating. Help → Check for updates.
- Tonec says updates **cannot be avoided** ([functions19](https://www.internetdownloadmanager.com/register/new_faq/functions19.html)).
- Updates are disabled for cracked copies ([download3](https://www.internetdownloadmanager.com/download3.html)).

### 1.21 Licensing / registration

- 30-day trial.
- One-year licence: about **$11.95** (list $14.95). Lifetime licence: about **$24.95** (list $29.95), shown as a "limited-time" discount that never seems to end ([download3](https://www.internetdownloadmanager.com/download3.html)).
- Extra PCs are $19.95 each for lifetime. The local-currency page (KHR) shows multi-PC prices about 20% lower ([buy](https://secure.internetdownloadmanager.com/buy_idm.html), [register8](https://www.internetdownloadmanager.com/register/new_faq/register8.html)).
- **Lifetime licence:** works forever on the version you have, with 3 years of guaranteed updates (purchases since Apr 2016), then a paid "update subscription" ([purchase7](https://cn.internetdownloadmanager.com/register/new_faq/purchase7.html)).
- **One-year licence:** **stops working** after a year.
- **Per-computer:** the licence is tied to hardware. **Multi-PC use is detected and the serial is "blocked automatically"** ([register8](https://www.internetdownloadmanager.com/register/new_faq/register8.html)).
- The installer warns when IDM is "used on other computers".

### 1.22 Misc

- Install wizard that tests the connection.
- Firewall configuration FAQ.
- Portable mode: none officially [UNVERIFIED].
- "Tell a friend" button.
- Description field per download.
- 64-bit: FAQ functions3.
- High CPU/RAM of IDMan.exe and IDMGrHlp.exe (the grabber helper) is a known FAQ topic (problems11).

---

## 2. Strengths users praise

1. **Speed (top reason).**
   - "Fastest download manager I've ever used. Excellent and easy to understand interface" (Ituaf, Mar 2024).
   - "When we talk about speed then no software can beat IDM." (ZainAslam760, 2021).
   - "I think this is the best download manager for windows" (Goswami Dipgiri, 2022).
   - All from [AlternativeTo](https://alternativeto.net/software/internet-download-manager/about), rated 4.1/5 from 18 ratings.
   - Comparison blogs put IDM ahead of FDM and JDownloader on throughput ([speed-drain](https://speed-drain.com/blog/idm-vs-fdm-vs-jdownloader-best-download-manager-comparison/), [fullidm](https://fullidm.app/blog/idm-vs-alternatives/); both low-trust, figures unverified).
2. **"It just catches everything".** Browser integration and one-click capture. AlternativeTo's summary lists browser integration (Chrome, Firefox) as a top pro. Techjockey (4.5/5, 15 reviews) praises the integration ([techjockey](https://www.techjockey.com/reviews/internet-download-manager)).
3. **Video grabbing from almost any site.** "Great for downloads from videos of any kind all over internet" (exyron, Aug 2024, AlternativeTo). Comparison articles say IDM has the "strongest YouTube and video grabbing" versus FDM, whose "YouTube video download feature is weak" ([speed-drain](https://speed-drain.com/blog/idm-vs-fdm-vs-jdownloader-best-download-manager-comparison/)).
4. **Reliable resume and refresh of expired links.** Especially valued on unstable connections in South and Southeast Asia, Iran and the Middle East. These regions dominate the crack and serial searches; the evidence is search-result volume in Vietnamese, Persian, Indonesian, Arabic and Chinese (see section 3 sources).
5. **Low resource use and a simple, stable UI** ("easy to understand interface"). Plus a Softpedia 5/5 in 2011 and 2022 ([Wikipedia](https://en.wikipedia.org/wiki/Internet_Download_Manager)).
6. **Privacy:** no ads and no telemetry claimed ("pretty good privacy policy", azoac 2022). The policy says IDM "WILL NOT COLLECT any information about you" ([privacy](https://www.internetdownloadmanager.com/privacy.html)).
7. **Very frequent fixes** when sites break: 403 and panel fixes within days ([news](https://www.internetdownloadmanager.com/news.html)).
8. **Multilingual** (50+ languages), with RTL support for Arabic and Persian.

---

## 3. Weaknesses and complaints (specific)

| Area | Evidence |
|---|---|
| **Price and licence model** | Shareware with a 30-day trial. **The one-year licence stops IDM working** after expiry. The lifetime licence includes only 3 years of updates, then a paid subscription ([purchase7](https://cn.internetdownloadmanager.com/register/new_faq/purchase7.html)). Licensed per PC with **automatic serial blocking on multi-PC detection** ([register8](https://www.internetdownloadmanager.com/register/new_faq/register8.html)). The permanent "limited-time" discount ($24.95 instead of $29.95) is a dark pattern. AlternativeTo users cite "cost over time" ([AT](https://alternativeto.net/software/internet-download-manager/about)). Cost is the main reason for mass piracy, and paying is hard where cards don't work internationally (Iran sanctions; inferred, [UNVERIFIED]). |
| **"Fake serial" popup** | "Internet Download Manager has been registered with a fake Serial Number or the Serial Number has been blocked", after which IDM exits ([tomsguide thread](https://forums.tomsguide.com/threads/idm-has-been-registered-with-a-fake-serial-key-idm-is-exiting.347152/latest)). **Paying users also hit it.** Tonec's FAQ answer for buyers is to exit, reboot, reinstall, and "probably a cracked version was used before" ([register7](https://www.internetdownloadmanager.com/register/new_faq/register7.html)). Users who **reinstall Windows often** can get blocked; Tonec advises keeping the same computer name ([register8](https://www.internetdownloadmanager.com/register/new_faq/register8.html)). A Firefox user thought the browser was at fault ([Mozilla support](https://support.mozilla.org/es/questions/937657)). Vietnamese guides on this exist ([fptshop](https://fptshop.com.vn/tin-tuc/thu-thuat/ly-do-idm-bi-fake-serial-va-cach-fix-loi-nhanh-gon-60227), [mytour](https://mytour.vn/en/blog/bai-viet/fixing-idm-fake-serial-number-issue-for-download-problems.html)). |
| **Cracked copies and malware** | Cracked IDM is a lure in infostealer campaigns. CloudSEK reported a Pakistan-based syndicate using IDM cracks ([ITPro](https://www.itpro.com/security/malware/warning-issued-as-new-pakistan-based-malware-group-hits-millions-globally)). ANY.RUN verdicts mark fake 6.42 builds as malicious ([any.run](https://any.run/report/29312e10e16f19ab4806ceaf1d7443f2bb452b19b4a21b03f24a574aec14085a/d15b4a69-ed18-4377-857d-6240476a1610)). A **fake "Internet Download Manager" Chrome extension had 200k+ installs** of adware, in the store since 2019 ([GizChina/BleepingComputer, 2022](https://www.gizchina.com/2022/08/28/internet-download-manager-chrome-extension-is-dangerous-for-your-pc/)). The real extension being *hidden from store search* makes fakes easy. Tonec also states cracks "always contain viruses" ([download3](https://www.internetdownloadmanager.com/download3.html)). |
| **YouTube / streaming breakage** | Constant: about 15 builds fix "403 Forbidden", many fix a "missing panel", TS stalling at 99%, separate A/V tracks, MKV desync, only 360p offered, or "video" replacing the filename ([news](https://www.internetdownloadmanager.com/news.html)). Tonec's own fix list for a missing panel includes "sign in and out of Google", "try another ISP or hotspot", "Ctrl+F5 several times" ([video-panel FAQ](https://www.internetdownloadmanager.com/register/new_faq/video-download-panel-missing.html)). Shorts work about 60% of the time (AMO 2026). |
| **Extension problems (MV3, forks)** | MV3 migration in Jul 2025 (6.42.42), then a required extension approval (6.42.45). AMO 2026 reviews: "It absolutely breaks on Firefox forks including Nightly and Developer"; "button doesn't appear… error status"; "not work on 6.43.7"; **high CPU on YouTube** with 6.43.1; broke on Firefox 150 and Brave ([AMO](https://addons.mozilla.org/en-US/firefox/addon/tonec-idm-integration-module/reviews/)). Chrome FAQ topics: extension disabled on each launch, "cannot launch IDM", Chrome crashes, sites hanging, unwanted downloads when opening sites ([Chrome FAQ](https://www.internetdownloadmanager.com/register/new_faq/Chrome-browser-integration.html)). |
| **Intrusive download panel** | Users want it off by default and a per-site memory (AMO, Aug 2026). An auto-download-on-play behaviour needed its own off switch ([video14](https://aaa.internetdownloadmanager.com/register/new_faq/video14.html)). |
| **Windows only** | No macOS, Linux or mobile ([functions2](https://www.internetdownloadmanager.com/register/new_faq/functions2.html)). |
| **Missing protocols** | No torrent ([functions4](https://www.internetdownloadmanager.com/register/new_faq/functions4.html)). No SFTP ([AT](https://alternativeto.net/software/internet-download-manager/about)). No RTSP or DRM. HTTP/3 not documented [UNVERIFIED]. |
| **Dated UI** | AlternativeTo users cite an "outdated interface" and "UI limitations". It is a Win32 UI with 2000s-era toolbar skins ("3D/Neon"). Softpedia in 2014 criticised no major version since 2010 ([Wikipedia](https://en.wikipedia.org/wiki/Internet_Download_Manager)). Still on 6.x as of 2026. |
| **Dark mode** | Exists since **6.37** (View → Dark Mode support; follows the Windows app mode), but only covers "most frequently used dialogs" and "the majority of" the UI ([neowin](https://www.neowin.net/software/internet-download-manager-idm-637-build-8-beta/), [features2](https://www.internetdownloadmanager.com/features2.html)). So it is partial. |
| **High-DPI** | No IDM-specific evidence found. **[UNVERIFIED]**. Common Win32 bitmap-toolbar blur is likely. Test it. |
| **File corruption** | An official FAQ exists. IDM blames antivirus (ESET named), proxies, "accelerators" and cracks. The workaround is to **drop to 1 connection** ([problems9](https://www.internetdownloadmanager.com/register/new_faq/problems9.html)). There is **no hash verification** (MD5/SHA-256 check) [UNVERIFIED that none exists]. |
| **Stuck at 99% / rebuilding / temp folder** | FAQ items for "stuck at 99%", "long time rebuilding temp files", "temp files cannot be rebuilt", and "file stays in temporary folder" when the target drive is missing or Controlled Folder Access blocks it ([index](https://www.internetdownloadmanager.com/register/new_faq/functions_main.cgi)). The temp-then-move design costs double disk I/O and needs double space [design inference]. |
| **"File size unknown" / no resume** | Happens when the server has no Content-Length or range support, or when sessions expire (Dropbox large files) ([Dropbox forum](https://www.dropboxforum.com/discussions/101001012/problems-when-using-idm-to-download-a-big-single-file-on-dropbox/623329), [hovatek](https://www.hovatek.com/forum/thread-152.html)). Signed or session-bound links often fail to refresh. |
| **Google Drive / cookie-auth** | The panel is unsupported for Drive video. Restricted files fail. IDM sometimes doesn't catch Drive downloads (extension or exception issues; check `*.usercontent.google.com` is not excepted) ([GDrive FAQ](https://www.internetdownloadmanager.com/register/new_faq/download-from-google-drive.html)). "Small HTML file downloaded instead of the archive" is a FAQ item, a classic lost-cookie/auth symptom (problems6). |
| **Crashes / system stability** | Explorer crashes (fixed 6.42.18, 6.43.5). A BSOD in `idmwfp.sys` ([MS Q&A](https://learn.microsoft.com/en-us/answers/questions/4149979/av-w-(null)-idmwfp-unknown-function-crash-code-win)). An old DLL crashing network apps ([idmmbc](https://www.internetdownloadmanager.com/support/idmmbc.html)). High CPU/RAM (FAQ problems11). Network monitors show traffic continuing after pause (FAQ problems13). |
| **Update nags** | About weekly prompts. Tonec: "cannot avoid frequent updates" ([functions19](https://www.internetdownloadmanager.com/register/new_faq/functions19.html)). Each update can trip antivirus or firewall rules because the exe changed ([problems9](https://www.internetdownloadmanager.com/register/new_faq/problems9.html)). |
| **Speed limiter is per-file only** | No global bandwidth cap ([functions11](https://www.internetdownloadmanager.com/register/new_faq/functions11.html)). |
| **No settings backup** | Migration means exporting a registry key by hand ([functions17](https://www.internetdownloadmanager.com/register/new_faq/functions17.html)). |
| **Privacy / telemetry** | No evidence of telemetry. The extension reads all pages and gets cookies and "encrypted credentials". Serial and multi-PC checks phone home, implied by automatic blocking ([register8](https://www.internetdownloadmanager.com/register/new_faq/register8.html)). Update and help traffic over **plain HTTP** (see section 4). |
| **Antivirus friction** | Kaspersky "ssl_connect error 5", Controlled Folder Access prompts, and Chrome flagging the install as malicious ([FAQ index](https://www.internetdownloadmanager.com/register/new_faq/functions_main.cgi), [Chrome FAQ](https://www.internetdownloadmanager.com/register/new_faq/Chrome-browser-integration.html)). |

---

## 4. Security history and attack surface

### Known CVEs and exploits

| ID | Version | Issue | Source |
|---|---|---|---|
| CVE-2005-2210 | 4.05 | Stack overflow via a long URL leading to remote code execution (EDB 1091) | [Tenable](https://www.tenable.com/cve/CVE-2005-2210) |
| EDB 32465 | 4.0.5 | File parsing buffer overflow | [EDB](https://www.exploit-db.com/exploits/32465/) |
| EDB 32824 | 5.15 b3 | **Language file** parsing buffer overflow (PoC) | [EDB](https://www.exploit-db.com/exploits/32824) |
| CVE-2010-0995 | < 5.19 | Stack overflow via a **crafted FTP URI** leading to RCE | [NVD](https://nvd.nist.gov/vuln/detail/CVE-2010-0995) |
| EDB 37668 | ? | "OLE Automation Array RCE". Probably the IE bug CVE-2014-6332 reached via IDM [UNVERIFIED mapping] | [EDB](https://www.exploit-db.com/exploits/37668/) |
| EDB 37608 | ? | "Find Download" crash PoC | [EDB](https://exploit-db.com/exploits/37608) |
| CVE-2020-28964 (EDB 48397) | 6.37.11.1 | Stack overflow in the **Search** function; local privilege escalation | [intruder](https://cvemon.intruder.io/cves/CVE-2020-28964) |
| CVE-2020-23060 | 6.37.11.1 | Stack overflow in **Export/Import** via a crafted `.ef2` file; CVSS 7.1 | [intruder](https://cvemon.intruder.io/cves/CVE-2020-23060) |
| CVE-2020-37234 (EDB 49083) | 6.38.12 | **Scheduler** "open file" field overflow (>5,000 bytes); local DoS; CVSS 6.2; published by NVD 2026-05-16 | [NVD](https://nvd.nist.gov/vuln/detail/cve-2020-37234) |
| EDB 51131 (no CVE) | 6.41 b3 (2022/2023) | **Man-in-the-middle RCE**: non-English installs fetch missing `.chm` help over **plain HTTP** and open it automatically. The author notes **update checks are also over HTTP**. | [EDB](https://www.exploit-db.com/exploits/51131) |

No CVE from 2023 to 2026 was found in the search. That does not prove there are none, since NVD was not queried directly.

### Attack surface (for the competitor's threat model)

- **Browser to app channel:** Chrome/Firefox native messaging, with a `nativeMessaging` permission ([chrome_perms](https://www.internetdownloadmanager.com/register/new_faq/chrome_perms.html)).
  - The extension passes cookies, POST data and "encrypted credentials" ([privacy](https://www.internetdownloadmanager.com/privacy.html)).
  - The native host must validate the origin extension ID and treat every field as untrusted: path traversal in filenames, oversized fields (IDM's history is overflow bugs), CRLF in headers.
- **Command line:** `IDMan.exe /d URL /p /f /n` is a silent download-anywhere primitive. Tonec forbids third-party extensions from calling it, which shows they know it is abusable ([command_line](https://www.internetdownloadmanager.com/support/command_line.html)).
  - Risk: a silent drop of an exe into the Startup folder via `/p`.
- **Kernel driver:** `idmwfp.sys` (WFP callout, signed Tonec) for "advanced integration". It has caused BSODs. A kernel driver is a large privileged surface ([freefixer](https://www.freefixer.com/library/file/idmwfp.sys-288453), [MS Q&A](https://learn.microsoft.com/en-us/answers/questions/4149979/av-w-(null)-idmwfp-unknown-function-crash-code-win)).
- **In-process DLLs** in Explorer, IE and other network apps. These caused Explorer crashes ([news](https://www.internetdownloadmanager.com/news.html), [idmmbc](https://www.internetdownloadmanager.com/support/idmmbc.html)).
- **Registry:** all settings, the download list, **site logins and proxy credentials** live under `HKCU\Software\DownloadManager` ([functions17](https://www.internetdownloadmanager.com/register/new_faq/functions17.html)). How the stored passwords are encrypted is unknown [UNVERIFIED]. Any same-user process can read or alter them.
- **Parsers:** `.ef2` import, language files, scheduler fields, the grabber's **JavaScript parsing** of untrusted pages, and the HTML/zip "Preview".
- **Post-download actions:** "open file when done" and the antivirus command line with `[File]` substitution. Quote properly to avoid argument injection.
- **Updates and help over HTTP** (EDB 51131). Use HTTPS plus signed updates.
- **Licensing DRM** that phones home and blocks serials.
- **Store impersonation:** the real extension is hidden from search, which lets fakes thrive.

---

## 5. UX details

### Copy exactly

1. **Ctrl+click forces, Alt+click bypasses**, both rebindable. File-type list plus wildcard site exceptions ([starting](https://www.internetdownloadmanager.com/support/using_idm/starting.html)).
2. **Download File Info dialog starts fetching immediately** while the user picks a folder (the default option). It offers Start / Download Later (+ queue picker) / Cancel, picks a category by file type, and has "remember this path for category".
3. **Refresh download address flow.** Open the source page, wait for a new link, ask if unsure, then resume. This is IDM's killer feature for expiring links ([sites2_3](https://www.internetdownloadmanager.com/register/new_faq/sites2_3.html)).
4. **Dynamic segmentation, half of the largest remaining piece.** Plus the overlap check and the auto-fallback to a single connection on mismatch ([features2](https://www.internetdownloadmanager.com/features2.html), [problems9](https://www.internetdownloadmanager.com/register/new_faq/problems9.html)).
5. **Floating "Download this video" panel** with a quality list. Per-site exceptions. "Don't auto-download from players".
6. **Category = folder + file types + site patterns.** Auto-sorting into category folders.
7. **Queues:** main download queue plus a **synchronization queue** (re-check for changed files). N files at a time per queue. Start/stop times. Retries. Quota. Post actions: shutdown, run program, or exit.
8. **Progress dialog "Show details"** with a per-connection table and coloured segment bar. Users love *seeing* the connections [UNVERIFIED layout].
9. **Complete dialog** with Open / Open folder / drag-the-file-out icon and "don't show again".
10. **Silent mode** = three toggles. Drop-target basket. Clipboard capture. "Download all links" from a selection.
11. **Per-server connection limits.** Site logins per directory. Proxy "use the browser's proxy".
12. **Delete-from-disk option** when removing a list entry. **Ignore mod-time changes on resume.**
13. **Grabber wizard:** 4 steps, templates, depth plus domain filters, size filters, hide duplicates, convert links to local, saved projects in the tree, scheduled sync.
14. **Command-line flags** `/d /p /f /a /n /q /s` (keep them compatible so existing scripts work).

### Fix

1. **Licensing:** free and open source removes the fake-serial and blocking class of pain completely. Do not ship a "phone home" of any kind.
2. **Speed limiter:** add a **global** cap plus per-download caps, a schedule (for example, slow by day, full by night) and a **tray quick toggle**.
3. **Panel:** off by default or unobtrusive, with **per-site remember** (an AMO request), keyboard shortcut and no CPU drain on YouTube.
4. **Extension:** publish under a findable name with the store id linked from the app. Support Firefox forks, Brave and Vivaldi. Self-diagnose "cannot launch app" with a one-click repair. Allow incognito clearly.
5. **Streams:** use a pluggable, frequently updated extractor (yt-dlp) so 403 and site changes don't need app releases. Add HTTP/3, SFTP, magnet/torrent (optional) and RTSP.
6. **Integrity:** optional hash verification (paste SHA-256 or auto-detect `.sha256`). Clear error states instead of a silent fallback to one connection.
7. **Temp handling:** download into the **target folder** as `name.part` (or preallocate) so there is no slow move or rebuild and no "stuck in temp". Show where the parts live.
8. **Auth and cookies:** reuse the browser's live session (cookies, headers via the extension) on resume and refresh, so Google Drive, Dropbox and signed URLs keep working.
9. **Settings:** export/import of settings plus the list as one JSON file. Store passwords in **Windows Credential Manager/DPAPI**, not plain registry.
10. **UI:** full dark mode across every dialog, modern Per-Monitor-V2 high-DPI, vector icons, no glow or skin gimmicks, accessible keyboard navigation.
11. **Updates:** HTTPS, signed, optional, silent background. Allow pinning a version. Keep site extractors as data updates.
12. **No kernel driver and no DLL injection into Explorer.** Use extensions plus native messaging only. Optionally catch downloads from other apps via a protocol handler and clipboard.
13. **Security:** strict native-messaging origin checks, filename sanitisation, CLI hardening (no silent writes to autostart paths), quoted post-download commands, and set **Mark-of-the-Web** on downloaded files.
14. **Cross-platform** (Windows first, but don't block macOS/Linux) and a **local API/RPC** for scripting, beyond the minimal CLI.
