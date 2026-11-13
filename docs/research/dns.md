# DNS សាធារណៈ (Google, Cloudflare, Quad9 ...) ក្នុង GDM: ការសិក្សា និងការវាស់

កាលបរិច្ឆេទ: 2026-11-14 · សំណួររបស់ម្ចាស់: "យក DNS ឥតគិតថ្លៃរបស់ Google, Cloudflare ... មកប្រើជាមួយ GDM បានទេ? ជួយឲ្យទាញលឿនជាងមុនទេ?"

## ចម្លើយខ្លី

- **ប្រើបាន** (ទាំង DNS ធម្មតា និង DNS-over-HTTPS) ប៉ុន្តែ **មិនធ្វើឲ្យការទាញលឿនជាងមុនទេ** (វាស់ខាងក្រោម)។
- DNS ត្រូវការតែម្ដងមុនភ្ជាប់ (~30-45 ms)។ ល្បឿនទាញកំណត់ដោយ bandwidth, ចំនួន connection (GDM ចែកជាផ្នែកដូច IDM) និង server។
- GDM ប្រើ DNS របស់ប្រព័ន្ធ ដូច្នេះ **អ្នកប្រើដែលចង់បាន DNS សាធារណៈ ឬ DNS ដែលបំលែងកូដ (DoH) គ្រាន់តែកំណត់ក្នុង Windows/macOS/Linux ហើយ GDM ប្រើតាមដោយស្វ័យប្រវត្តិ** គ្មានកូដបន្ថែម គ្មានការកំណត់ក្នុង GDM។
  - Windows 11: Settings › Network & internet › (Ethernet/Wi-Fi) › DNS server assignment › Edit › 1.1.1.1 / 8.8.8.8 + "DNS over HTTPS: On (automatic template)"។
  - macOS: profile DoH របស់អ្នកផ្ដល់ (Cloudflare 1.1.1.1 app, ឬ `.mobileconfig`)។ Linux: `systemd-resolved` `DNSOverTLS=yes`។

## អ្វីដែលបានវាស់ (កុំព្យូទ័រម្ចាស់, កម្ពុជា, DNS ប្រព័ន្ធ = 1.1.1.1 + 8.8.8.8)

| វាស់ | លទ្ធផល |
|---|---|
| រយៈពេល lookup (warm) | system / Cloudflare / Google / Quad9: 32-45 ms ទាំងអស់; lookup ដំបូង (cold) github.com: 1063 ms ម្ដង |
| DNS ផ្សេងគ្នា ផ្ដល់ CDN edge ផ្សេងគ្នា? | **បាទ**: dl.google.com ⇒ 142.250.x (system, Cloudflare) ទល់ 142.251.x (Google, Quad9); TCP connect 54-74 ms ទល់ 34 ms |
| ល្បឿនទាញពិត (ឯកសារ 82 MB ពី dl.google.com, connection មួយ, ២ ដងក្នុងមួយ edge) | system 12.0 / 13.2 MB/s · Google 13.6 / 12.9 · Cloudflare 13.1 / 12.4 · Quad9 14.0 / 12.8 MB/s ⇒ **ខុសគ្នាក្នុងកម្រិត noise**; តែ TTFB ខុស ~0.1 s |

សេចក្ដីសន្និដ្ឋាន: edge ដែល RTT ទាបជាង មិនបានផ្ដល់ throughput ខ្ពស់ជាងលើបណ្ដាញដែលកំណត់ដោយ bandwidth។ ការចំណេញតែ ~0.1 s នៅដើមការទាញ មិនសមនឹងហានិភ័យខាងក្រោម។

## ហេតុអ្វីមិនដាក់ DNS សាធារណៈក្នុង GDM ដោយស្វ័យប្រវត្តិ (council Codex r1 + ការវាស់)

1. **សុវត្ថិភាព:** ក្រុមហ៊ុន/សាលា/ISP និង DNS ដូចជា Quad9 បិទ domain មេរោគ (NXDOMAIN)។ បើ GDM ប្ដូរទៅ DNS ផ្សេងពេល DNS ប្រព័ន្ធបដិសេធ វានឹងទាញមេរោគរំលងតម្រងនោះ។
2. **បណ្ដាញក្រុមហ៊ុន / VPN (split-horizon):** ឈ្មោះខាងក្នុងរកមិនឃើញ ឬទៅ address ខុស តាម DNS សាធារណៈ។
3. **ឯកជនភាព:** ផ្ញើឈ្មោះគ្រប់គេហទំព័រទៅ Google/Cloudflare ទោះអ្នកប្រើមិនបានជ្រើស។ ពេលប្រើ proxy (ឧ. ដើម្បីលាក់អត្តសញ្ញាណ) វាលេចធ្លាយ។
4. **CDN:** Cloudflare មិនផ្ញើ EDNS Client Subnet ⇒ ពេលខ្លះផ្ដល់ edge ឆ្ងាយជាង DNS របស់ ISP។ ការប្ដូរដោយស្វ័យប្រវត្តិអាចធ្វើឲ្យយឺតជាង មិនមែនលឿនជាង។
5. **Captive portal (Wi-Fi សណ្ឋាគារ):** DoH ត្រូវបានរាំង ⇒ ការទាញជាប់រង់ចាំ។
6. **កូដ:** DoH ត្រឹមត្រូវតាម RFC 8484 ត្រូវការ DNS wire parser (ឬ crate ធំ hickory) ដែលជាផ្ទៃវាយប្រហារថ្មីលើទិន្នន័យពីបណ្ដាញ ដើម្បីអត្ថប្រយោជន៍ដែលវាស់មិនឃើញ។

## អ្វីដែល GDM មានរួចហើយ ដែលសំខាន់ជាង DNS

- ការពារ DNS rebinding / SSRF: resolve ម្ដង ពិនិត្យគ្រប់ address ហើយភ្ជាប់តែ address ដែលបានពិនិត្យ (`crates/gdm-core/src/net.rs`, W-02)។
- Happy Eyeballs IPv4/IPv6 និងសាកល្បង address បន្ទាប់ពេលមួយភ្ជាប់មិនបាន (hyper-util connector)។
- គ្រប់ request ផ្នែកផ្ញើ `If-Range` ⇒ បើ connection ផ្សេងទៅដល់ CDN edge ដែលមានកំណែឯកសារផ្សេង GDM ដឹង (FileChanged) មិនបញ្ចូលទិន្នន័យខុសទេ។
- Proxy: relay ក្នុង GDM resolve ហើយពិនិត្យ address មុនផ្ញើទៅ proxy (ADR-6, decisions G7)។

## ពេលណាត្រូវពិចារណាម្ដងទៀត

បើមានអ្នកប្រើពិត (issue) ដែល ISP បំផ្លាញ DNS ហើយ OS របស់គេគ្មាន DoH ⇒ ជម្រើស "Secure DNS" ដែលអ្នកប្រើបើកដោយខ្លួនឯង: អ្នកផ្ដល់តែមួយ, RFC 8484 wire format, គ្មាន fallback ស្ងាត់, បិទពេលប្រើ proxy, និង PRIVACY.md ថ្មី។ មុនសាងសង់: វាស់លើបណ្ដាញអ្នកប្រើនោះ។
