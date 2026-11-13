# សេវាបង់ថ្លៃសម្រាប់ស្ថាប័ន និងហាង PC / Paid services for organisations and PC shops

**ខ្មែរ** · [English](#english)

> ឯកសារនេះពន្យល់គោលការណ៍ (blueprint §21 M2, M4)។ វាមិនមែនជាកិច្ចសន្យាទេ។ សេវានីមួយៗធ្វើតាមកិច្ចសន្យាជាលាយលក្ខណ៍អក្សរ ដែលអានជាមួយ [LICENSE](../LICENSE) (RPL-1.5)។ ចំពោះសិទ្ធិលើ software ប្រសិនបើមានភាពខុសគ្នា LICENSE មានអានុភាពជាង។

## ខ្លឹមសារ

- **GDM ឥតគិតថ្លៃសម្រាប់អ្នកទាំងអស់គ្នា** រួមទាំងក្រុមហ៊ុន សាលា និងស្ថាប័នរដ្ឋ។ អ្នកមិនចាំបាច់ទិញអ្វីដើម្បីប្រើ ឬចែកចាយ GDM កំណែដើម (មិនកែ) ទេ។
- យើង**មិនលក់អាជ្ញាប័ណ្ណ**ទេ។ GDM មានតែ RPL-1.5 ប៉ុណ្ណោះ ([NOTICE.md](../NOTICE.md)): គ្មាន licence ពាណិជ្ជកម្ម គ្មាន dual-licensing។ នេះជាគោលការណ៍គម្រោង ហើយលក្ខខណ្ឌ SignPath Foundation ក៏មិនទទួលគម្រោងដែលមាន commercial dual-licensing ដែរ។
- ថ្លៃសេវាគិតលើ**ពេលវេលា និងជំនាញ**ប៉ុណ្ណោះ។ ការបង់ថ្លៃសេវាមិនផ្ដល់សិទ្ធិផ្សេងលើ software ក្រៅពី RPL-1.5 ទេ។

## សេវាដែលផ្ដល់

| សេវា | អ្វីដែលរួមបញ្ចូល |
|---|---|
| ដំឡើងច្រើនគ្រឿង | ដំឡើង GDM និង extension លើកុំព្យូទ័រស្ថាប័ន, ពិនិត្យ SHA-256 និង attestation |
| កំណត់ setting | queue, កម្រិតល្បឿន, proxy, ថត, category តាមតម្រូវការស្ថាប័ន (ការកំណត់ធម្មតា មិនមែនការកែកូដ) |
| បង្ហាត់ | វគ្គបង្ហាត់ប្រើ GDM ជាភាសាខ្មែរ ឬអង់គ្លេស |
| Support | ជំនួយតាមរយៈពេល និងម៉ោងឆ្លើយតបដែលសរសេរក្នុងកិច្ចសន្យា |
| មុខងារតាមសំណូមពរ | សរសេរមុខងារថ្មីតាមកិច្ចសន្យា (លក្ខខណ្ឌខាងក្រោម) |

## លក្ខខណ្ឌមុខងារតាមសំណូមពរ

1. **កិច្ចសន្យាជាលាយលក្ខណ៍អក្សរ** មុនចាប់ផ្ដើម: ភាគីដែលចុះកិច្ចសន្យា, អ្វីដែលត្រូវប្រគល់, តម្លៃ និងការបង់តាមដំណាក់កាល។
2. **កម្មសិទ្ធិ:** កូដ GDM ដែលសរសេរក្នុងសេវានេះ ជាកម្មសិទ្ធិបញ្ញារបស់ Golden IQ និង CHHIT Sungchhay ហើយចេញផ្សាយក្រោម RPL-1.5 ដូចកូដ GDM ផ្សេងទៀត។ ឯកសារ ទិន្នន័យ ឬកូដដែលអតិថិជនផ្ដល់ នៅតែជារបស់អតិថិជន។ ការបង់ថ្លៃខ្លួនឯងមិនផ្ទេរ ឬផ្ដល់កម្មសិទ្ធិទេ ដូច្នេះកិច្ចសន្យាត្រូវសរសេរវាឱ្យច្បាស់។
3. **កូដត្រូវបោះពុម្ព:** កូដនៃមុខងារនេះត្រូវដាក់ជាសាធារណៈក្រោម RPL-1.5 ពេលប្រគល់ (មិនមែនរង់ចាំ release ផ្លូវការទេ) ឬមុននោះ ពេល RPL-1.5 តម្រូវ: ការប្រើសាកល្បង ឬប្រើការងារក្នុងស្ថាប័នមុនពេលប្រគល់ ក៏អាចជា Deploy ដែលមានកំណត់ពេលបោះពុម្ព និងប្រកាសតាម LICENSE ដែរ (លើកលែងតែ Research)។ អ្វីដែលបោះពុម្ពគឺ Source Code និង Required Components ពេញលេញ (§1.12, §1.16) មិនមែនតែ patch ទេ។ យើង**មិនផ្ដល់សេវា fork ឯកជន ឬ fork សម្ងាត់**ទេ ហើយមិនសន្យាថាកូដនៅជាការសម្ងាត់ ឬផ្ដល់សិទ្ធិផ្ដាច់មុខទេ។ (នេះជាដែនកំណត់នៃសេវារបស់យើង មិនមែនការដកសិទ្ធិដែល RPL-1.5 ផ្ដល់ដល់អ្នកណាម្នាក់ទេ។)
4. **ការចូលក្នុង GDM ផ្លូវការមិនធានាទេ:** ការទទួលការងារ (acceptance) ក្នុងកិច្ចសន្យា ខុសពីការដាក់មុខងារចូលក្នុង GDM ផ្លូវការ។ ការដាក់ចូលផ្លូវការជាការសម្រេចរបស់អ្នកថែទាំ (សុវត្ថិភាព, ភាពត្រឹមត្រូវ, ចំណាប់អារម្មណ៍អ្នកប្រើទូទៅ)។
5. **ការទទួលការងារ:** លក្ខខណ្ឌវាស់បាន (test, ឧទាហរណ៍) និងរយៈពេលពិនិត្យ សរសេរក្នុងកិច្ចសន្យា។
6. **ការថែទាំ:** រយៈពេលជួសជុលកំហុស និងការថែទាំក្រោយប្រគល់ សរសេរក្នុងកិច្ចសន្យា។ ក្រោយរយៈពេលនោះ ការថែទាំគឺជាសេវាថ្មី។
7. **បើអតិថិជនកែកូដខ្លួនឯង ហើយប្រើក្នុងស្ថាប័ន:** RPL-1.5 ចាត់ទុកការប្រើក្នុងស្ថាប័នជា "Deploy" (§1.2)។ អ្នកដែល Deploy កំណែកែត្រូវបោះពុម្ពកូដក្នុងរយៈពេល ១ ខែ រក្សាវាឱ្យទាញបាន ១២ ខែក្រោយឈប់ប្រើ (§6.1), ប្រកាសទីតាំងកូដ (§6.4(c)) និងប្រើឈ្មោះផ្សេង (§6.0, [TRADEMARKS.md](../TRADEMARKS.md))។ កិច្ចសន្យាត្រូវកំណត់ថាអ្នកណាជាអ្នកធ្វើការងារទាំងនេះ ប៉ុន្តែការផ្ទេរការងារតាមកិច្ចសន្យា មិនដោះលែងអ្នក Deploy ពីកាតព្វកិច្ច licence ទេ។

## ការធានា និង support (RPL-1.5 §6.5)

ការសន្យា support ឬ warranty ធ្វើដោយអ្នកផ្ដល់សេវាដែលមានឈ្មោះក្នុងកិច្ចសន្យាតែប៉ុណ្ណោះ។ អ្នករួមចំណែកផ្សេងទៀតមិនមែនជាភាគីទេ។ តាម §6.5 អ្នកផ្ដល់សេវានោះទទួលខុសត្រូវ (indemnify) Licensor និងអ្នករួមចំណែក ចំពោះការទាមទារដែលកើតពីការសន្យារបស់ខ្លួន រួមទាំងថ្លៃមេធាវី និងចំណាយពាក់ព័ន្ធ។ យើងមិនសន្យាថាគ្រប់ការទាញយកនឹងជោគជ័យទេ។

## ហាង PC និង OEM

- **ការដំឡើង GDM កំណែដើម (មិនកែ) លើកុំព្យូទ័រដែលអ្នកលក់ គឺអនុញ្ញាតដោយឥតគិតថ្លៃរួចហើយ** ក្រោម RPL-1.5។ មិនចាំបាច់សុំការអនុញ្ញាត ឬទិញអ្វីទេ។
- អ្នកអាចគិតថ្លៃលើ**សេវាដំឡើង កំណត់ setting និង support** របស់អ្នក។ GDM មិនតម្រូវឱ្យទិញអាជ្ញាប័ណ្ណទេ។ សូមពិពណ៌នាថ្លៃរបស់អ្នកឱ្យត្រឹមត្រូវ ជាសេវាដំឡើង កំណត់ setting ឬ support (ឧ. "សេវាដំឡើង GDM") ហើយកុំបង្ហាញថាការបង់ថ្លៃទិញសិទ្ធិ software បន្ថែម ឬការគាំទ្រផ្លូវការ។
- រក្សា LICENSE, NOTICE និងប្រយោគ attribution (Exhibit B) ដែលបង្ហាញក្នុងទំព័រ About ឱ្យនៅដដែល។
- ប្រសិនបើអ្នកបន្ថែម script, wrapper ឬកែ GDM នោះវាអាចក្លាយជា Extension ដែលមានកាតព្វកិច្ចខាងលើ (ចំណុច 7)។ Image ដែលដំឡើង GDM កំណែដើមដោយការកំណត់ធម្មតា មិនតម្រូវឱ្យប្ដូរឈ្មោះទេ។
- កុំបង្ហាញថាជាដៃគូផ្លូវការ ឬមានវិញ្ញាបនបត្រពី GDM លុះត្រាមានកិច្ចព្រមព្រៀងជាលាយលក្ខណ៍អក្សរ។ RPL-1.5 មិនផ្ដល់សិទ្ធិពាណិជ្ជសញ្ញាទេ (§5.0)។ ការសរសេរពិតដូចជា "កុំព្យូទ័រនេះមាន Galaxy Download Manager (ឥតគិតថ្លៃ)" អនុញ្ញាត។
- ការសន្យា support ឬ warranty របស់ហាង ស្ថិតក្រោម §6.5 ដូចខាងលើ។

## ទំនាក់ទំនង

បើកសំណើតាម GitHub issues ក្នុង repository នេះ (ដាក់ស្លាក "services")។ កុំដាក់ព័ត៌មានសម្ងាត់ក្នុង issue សាធារណៈ។

---

<a id="english"></a>
## English

> This page explains the policy (blueprint §21 M2, M4). It is not a contract. Each service is done under a written agreement read together with [LICENSE](../LICENSE) (RPL-1.5). Where they differ about software rights, LICENSE prevails.

### In short

- **GDM is free for everyone**, including businesses, schools and government. You never need to buy anything to use or redistribute the unmodified GDM.
- We **do not sell licences**. GDM is offered only under RPL-1.5 ([NOTICE.md](../NOTICE.md)): no commercial licence and no dual licensing. This is project policy, and SignPath Foundation's eligibility terms also exclude projects with commercial dual licensing.
- Fees pay for **time and skill** only. Paying for a service grants no software rights beyond RPL-1.5.

### Services

| Service | What it covers |
|---|---|
| Multi-PC deployment | install GDM and the extension on an organisation's PCs, check SHA-256 and attestations |
| Configuration | queues, speed limits, proxy, folders, categories for the organisation (normal settings, not code changes) |
| Training | GDM training in Khmer or English |
| Support | help for the term and response times written in the agreement |
| Commissioned features | new features written under an agreement (terms below) |

### Terms for commissioned features

1. **A written agreement** before work starts: the contracting parties, the deliverables, the price and payment milestones.
2. **Ownership:** GDM code written under the service is the intellectual property of Golden IQ and CHHIT Sungchhay and is released under RPL-1.5 like the rest of GDM. Material, data or code the client supplies stays the client's. Payment by itself does not transfer or grant ownership, so the agreement states it explicitly.
3. **The code is published:** the feature's source is made public under RPL-1.5 at delivery, not when an official release happens, or earlier when RPL-1.5 requires it: a pilot or operational use inside an organisation before delivery can already be a Deploy with the licence's publication and notification deadlines (Research is the exception). What is published is the full Source Code and Required Components (§1.12, §1.16), not only a patch. We **do not offer private or confidential forks as a service**, and we do not promise confidentiality or exclusivity for the code. (This limits our service; it does not take away anyone's rights under RPL-1.5.)
4. **Inclusion in official GDM is not guaranteed:** acceptance of the deliverable under the agreement is separate from merging the feature into official GDM, which is the maintainers' decision (security, correctness, value to all users).
5. **Acceptance:** measurable criteria (tests, examples) and a review period are written in the agreement.
6. **Maintenance:** the defect-correction and maintenance term after delivery is written in the agreement. After it, maintenance is a new service.
7. **If the client modifies GDM and uses it inside the organisation:** RPL-1.5 counts internal organisational use as "Deploy" (§1.2). Whoever deploys a modified version must publish its source within one month and keep it available for 12 months after deployment ends (§6.1), announce where it is (§6.4(c)) and use a distinct title (§6.0, [TRADEMARKS.md](../TRADEMARKS.md)). The agreement names who does this work, but assigning the work by contract does not release the deployer from its licence duties.

### Warranty and support (RPL-1.5 §6.5)

Support or warranty commitments are made only by the provider named in the agreement; other contributors are not parties. Under §6.5 that provider indemnifies the Licensor and contributors against claims arising from its own commitments, including attorney fees, costs and related expenses. We do not promise that every download will succeed.

### PC shops and OEMs

- **Installing the unmodified GDM on PCs you sell is already allowed, free of charge,** under RPL-1.5. No permission or purchase is needed.
- You may charge for **your installation, configuration and support services**. GDM requires no licence purchase. Describe your charges accurately as installation, configuration or support services (for example "GDM installation service"), and do not imply that payment purchases additional software rights or official endorsement.
- Keep LICENSE, NOTICE and the attribution notice (Exhibit B, shown on the About page) intact.
- Adding scripts or wrappers, or changing GDM, can make an Extension with the duties above (term 7). An image that installs the unmodified GDM with normal settings needs no renaming.
- Do not present yourself as an official partner or certified by GDM without a written agreement. RPL-1.5 grants no trademark rights (§5.0). Truthful statements such as "this PC includes Galaxy Download Manager (free)" are fine.
- A shop's own support or warranty promises fall under §6.5 as above.

### Contact

Open a request in this repository's GitHub issues (label "services"). Do not put confidential details in a public issue.
