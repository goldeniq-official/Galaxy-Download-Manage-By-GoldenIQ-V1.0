# ការចេញកំណែ GDM (version)

GDM មានលេខកំណែតែមួយគត់ ដែលរក្សាទុកក្នុង `package.json` និង `[workspace.package]` នៃ `Cargo.toml`។ កម្មវិធី, bridge, setup, extension, installer និង User-Agent ទាំងអស់យកលេខពីទីនោះ។ គ្មាននរណាកែលេខដោយដៃទេ។ មានតែ `scripts/version.mjs` ប៉ុណ្ណោះដែលប្ដូរវា។

## ស្តង់ដារលេខ: Semantic Versioning 2.0.0 (semver.org)

លេខមានទម្រង់ `MAJOR.MINOR.PATCH`។ ប្រភេទនៃការផ្លាស់ប្ដូរ ជាអ្នកកំណត់ថាលេខមួយណាឡើង មិនមែនចំនួន commit ទេ៖

| ប្រភេទ (`kind`) | ពេលណា | ឧទាហរណ៍ | រង់ចាំក្រោយកំណែមុន |
|---|---|---|---|
| **patch** | ជួសជុលតែប៉ុណ្ណោះ: គ្មានមុខងារថ្មី, គ្មានការកំណត់ថ្មី, មិនដកអ្វីចេញ | 1.0.0 → 1.0.1 | ២ ថ្ងៃ (បើ `"security": true` ចេញបានភ្លាម) |
| **minor** | មានមុខងារ ឬការកំណត់ថ្មី ហើយការកំណត់, ការទាញយក និង extension ចាស់នៅដំណើរការដដែល | 1.0.0 → 1.1.0 | ១៤ ថ្ងៃ |
| **major** | បំបែករបស់ចាស់: ការកំណត់/database/extension ចាស់លែងដំណើរការ ឬឈប់គាំទ្រប្រព័ន្ធប្រតិបត្តិការណាមួយ | 1.0.0 → 2.0.0 | ៩០ ថ្ងៃ |

ពេលលេខខាងឆ្វេងឡើង លេខខាងស្ដាំត្រលប់ទៅ 0 វិញ (1.0.3 → 1.1.0)។ បើមានការផ្លាស់ប្ដូរច្រើនប្រភេទក្នុងកំណែតែមួយ យកប្រភេទដែលធំជាងគេ។

## ចេញកំណែថ្មី (ចុចតែម្ដង)

1. សរសេរ release notes ពីរឯកសារ៖
   - `docs/release/next.json`: `{ "kind": "patch", "security": false, "km": "...", "en": "..." }`
   - `docs/release/next.md`
2. GitHub → Actions → **version** → Run workflow (branch `main`)។
3. workflow អាន `kind` ពី `next.json` ហើយគណនាលេខបន្ទាប់ដោយខ្លួនឯង។ វាបង្កើត commit `release vX.Y.Z` និង tag រួចដំណើរការ `release.yml`។ `release.yml` build ទាំង ៦ គោលដៅ ហើយបង្កើត release ជា **draft**។
4. ម្ចាស់ពិនិត្យ draft ហើយចុច **Publish**។ ពេល publish រួច `site.yml` ចាប់ផ្ដើម build គេហទំព័រឡើងវិញ។ គេហទំព័របង្ហាញលេខកំណែ និង link ទាញយកនៃ release ដែលទើប publish ដោយខ្លួនឯង។

## ហេតុអ្វីលេខមិនឡើងលឿនពេក

- លេខប្ដូរតែពេលចុចប៊ូតុង **version** ប៉ុណ្ណោះ។ commit, push ឬ build នីមួយៗ មិនប្ដូរលេខទេ។
- workflow បដិសេធ ពេលគ្មានអ្វីថ្មីតាំងពីការចេញលើកមុន ឬពេលរយៈពេលរង់ចាំក្នុងតារាងខាងលើមិនទាន់គ្រប់។
- ការចុចលើកដំបូងចេញ 1.0.0 ដូចដែលមានស្រាប់ ដោយមិនបូកលេខ។ សម្រាប់កំណែនេះ ឯកសារ `notes-1.0.0.*` មានរួចហើយ។

## គេហទំព័រ (gdm.goldeniq.app)

នៅពេល build គេហទំព័រអានកំណែចុងក្រោយពី `github.com/<repo>/releases/latest` (Web repo `scripts/latest-release.mjs`)។ ដូច្នេះលេខកំណែ និង link ទាញយក តែងតែត្រូវនឹង release ដែលបាន publish។ ម្ចាស់រៀបចំតែម្ដងគត់៖
1. Cloudflare → Workers & Pages → `gdm-web` → Settings → Builds → **Deploy Hooks** → បង្កើត hook (branch `main`) ហើយចម្លង URL។
2. GitHub repo GDM → Settings → Secrets and variables → Actions → New repository secret ឈ្មោះ `SITE_DEPLOY_HOOK`, value = URL នោះ។
