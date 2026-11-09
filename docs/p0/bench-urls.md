# បញ្ជី URL សម្រាប់ benchmark (plan.md ង.5)

បញ្ជីនេះកំណត់មុន run វាស់ដំបូង ហើយមិនប្ដូរក្នុងរង្វង់វាស់មួយ។ ឯកសារប្រភពដែល harness អាន: `spikes/engine/bench-lan.txt` និង `spikes/engine/bench-public.txt`។ sha256 ចាក់សោដោយគូ warm-up (GDM និង IDM ត្រូវដូចគ្នា) ហើយកត់ក្នុង `docs/p0/results.md` G3។

## LAN gate (loopback fixture S4, `s4 serve 8790`)

| id | URL | bytes | ចំណាំ |
|---|---|---|---|
| lan1g | `http://127.0.0.1:8790/file?size=1073741824&seed=7&trate=125000000` | 1073741824 | សរុប 125 MB/s = link 1 Gbit/s; gate ≥ 112.5 MB/s |
| lan1g-percon | `http://127.0.0.1:8790/file?size=1073741824&seed=8&rate=25000000&trate=125000000` | 1073741824 | link ដដែល + server កំណត់ 25 MB/s ក្នុងមួយ connection |

## Internet (CDN ផ្សេងគ្នា 5)

ពិនិត្យ 2026-11-09: ទំហំ = Content-Length, `Accept-Ranges: bytes` ទាំង 5។

| id | CDN | URL | bytes |
|---|---|---|---|
| cloudflare | Cloudflare (download.blender.org) | https://download.blender.org/release/Blender4.2/blender-4.2.0-windows-x64.zip | 383542405 |
| fastly | Fastly (cdn.kernel.org) | https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.12.tar.gz | 239929282 |
| cloudfront | CloudFront (download.jetbrains.com) | https://download.jetbrains.com/python/pycharm-community-2024.2.4.exe | 489747472 |
| github | GitHub Releases | https://github.com/ollama/ollama/releases/download/v0.3.14/OllamaSetup.exe | 696149360 |
| google | Google (dl.google.com) | https://dl.google.com/android/repository/emulator-windows_x64-12414864.zip | 419372875 |

គ្មាន Akamai ក្នុងបញ្ជី (plan ឧទាហរណ៍) ⇒ ជំនួសដោយ Google CDN ព្រោះមិនបានរក URL Akamai ដែលស្ថិរ និងគាំទ្រ Range។
