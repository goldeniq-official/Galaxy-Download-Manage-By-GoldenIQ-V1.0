G8 is workable. The main traps are treating “no private forks” as a licence restriction, assuming commissioned work automatically belongs to GDM, and promising confidentiality that conflicts with RPL source publication. No files were changed.

**1. Missing KHQR: ship a complete text-only state**

Until the owner supplies and verifies GDM’s QR:

- README: include the bilingual support wording, with **no image link**. An HTML comment can record the intended future path.
- About: show the same optional-support explanation and “KHQR support is not available yet.” Render no image, payment button, or empty QR frame.
- Keep the future asset path in one implementation setting, initially absent/disabled. Later activation should require adding the verified asset and updating README in the same change.
- Do not use the other organisation’s QR, a sample payment QR, or a remote QR generator.

Suggested common wording:

> GDM is free for everyone, including businesses, schools and government organisations. Financial support is optional. It unlocks no features and gives no priority in support, issue handling or feature decisions.

Use one bundled PNG, for example `public/gdm-support-khqr.png`, once supplied. About references its local asset URL; README references the repository file. Preserve the existing CSP. On image failure, hide the image and retain the explanatory text.

Verify that the owner supplies a reusable merchant/payment QR rather than an expiring transaction QR. Display the verified recipient name and currency beside it so users can check their banking app.

**2. RPL and commercial wording**

The recommendations below follow the actual repository [LICENSE](../../../LICENSE) and [NOTICE.md](../../../NOTICE.md). RPL-1.5’s publication, attribution and support provisions are also in the [OSI licence text](https://opensource.org/license/RPL-1.5).

| Topic | Risk and recommended treatment |
|---|---|
| **Commission ownership** | Payment does not itself transfer copyright. Identify the legal contracting parties; establish whether “Golden IQ” is an entity or trading name. Obtain written assignments from the actual authors to the intended owners, subject to applicable law. Exclude client materials, pre-existing work and third-party components from blanket ownership claims. |
| **Client rights** | State that delivered GDM code remains under RPL-1.5. Copyright ownership does not remove recipients’ licence rights. Do not promise exclusivity, prohibit redistribution, or require an NDA covering code that must become public. |
| **Client deployment** | Explain that organisational production use can trigger obligations even without external redistribution. For client-created Extensions, identify who publishes source and notices; merely assigning ownership to GDM does not safely eliminate the deploying client’s responsibilities. |
| **Publication deadline** | Source for each deployed version must be publicly available within one month, remain available throughout deployment, and for at least **12 months after deployment ceases**. Separately provide the community notification required by §6.4(c). A private client repository is insufficient. |
| **Source scope** | Include the source and required build/install materials. Review deployment scripts and configuration for “Required Components”; keep credentials and personal data out of deliverables intended for publication. |
| **“No private forks”** | Write: “We do not offer confidential or proprietary GDM forks as a service.” Do not write: “Customers may not fork GDM.” The service offering can be limited; recipients’ RPL rights cannot be contracted away. |
| **Official inclusion** | Separate acceptance of commissioned deliverables from acceptance into the official product. Specify deliverables, acceptance criteria, review period, payment milestones, defect correction, maintenance term and termination. Official inclusion remains a separate maintainer decision. |

**Can Golden IQ deliver a custom build to one client before public release?** Generally yes, provided it is an RPL delivery with the applicable obligations honoured. Delivery is distribution; do not start the compliance clock only when the client enters production or the official release happens.

The safest operational policy is to publish the exact source and a searchable release notice **at delivery**, even if the official stable binary release comes later. Avoid promising a confidential release window or assuming the Licensor’s ownership exempts contributed portions from their existing licence. A separately maintained client variant also needs review under the distinct-title requirement.

For paid support, recommended wording is:

> Fees cover the services described in this agreement. GDM remains available without charge under RPL-1.5. Service commitments are made solely by [identified provider]; other contributors are not parties to this agreement and make no service commitments.

Section 6.5 is more than a disclaimer: the provider assumes indemnity obligations protecting the Licensor and Contributors against claims arising from its warranty/support/liability commitments, including associated legal costs. Do not promise unlimited indemnity, guaranteed download success, or obligations on behalf of all contributors. Have the actual provider review this exposure before signing organisational contracts.

For PC shops/OEMs:

- Invoice **“GDM installation and configuration service”** or **“GDM support service”**; explicitly state that GDM itself is free.
- Preserve the licence, notices and Exhibit B attribution in the relevant attribution displays and documentation.
- Unmodified redistribution does not trigger renaming merely because a shop installs it. Extensions are broader than edited application binaries: wrappers, runtime alterations and required scripts deserve review.
- Do not imply official partnership, certification or trademark permission. RPL does not grant trademark rights.
- Apply §6.5 to the shop’s support/warranty promises.

For voluntary payments:

> This is a voluntary contribution to support GDM development. It purchases no product, feature, service or priority. We do not represent that contributions are tax-deductible.

Avoid “charitable donation,” “tax receipt,” “investment,” or promised returns without an established legal basis. An ordinary payment acknowledgement must not imply charitable status. Publish a contact and procedure for mistaken or duplicate payments; avoid an absolute “no refunds under any circumstances.” Verify Cambodian accounting, tax treatment and bank/merchant-account conditions before collecting payments. Those local conclusions are not established by this review.

Keep Exhibit A’s standard wording. Its reference to another licence does not establish that GDM actually offers one; the existing NOTICE clarifies the policy. Paid services should grant no alternative software licence. SignPath’s conditions expressly exclude commercial dual licensing and proprietary components; eligibility for particular custom builds still requires confirmation against its release/signing conditions. [SignPath Foundation terms](https://signpath.org/terms)

**3. Sponsor policy: make the boundary contractual**

Recommended core language:

> Sponsorship buys only the acknowledgement placement, duration and format expressly stated in the agreement, on GDM’s website and/or README. Every acknowledgement is labelled “Sponsored.” Sponsorship provides no placement in the application, installer, updater, notifications or download workflow.

Also explicitly prohibit:

- Sponsor names, logos, promotions or paid links in About and other application surfaces.
- Influence over defaults, search results, download sources, rankings, recommendations, updates or security decisions.
- Bundled offers, redirects, affiliate identifiers, tracking pixels, remote advertising scripts or access to user data.
- Exclusivity or veto rights over product decisions, criticism, competitors or security disclosures.
- Claims of official endorsement, partnership or unrestricted trademark rights.

Use locally hosted sponsor artwork and direct, non-tracking links. Reserve the right to reject or remove acknowledgements and terminate sponsorship; specify what happens to prepaid fees. Any commissioned development should use a separate services agreement and cannot buy guaranteed product inclusion.

**4. Concrete completion checks**

Before calling the documentation and implementation done:

- **Promise consistency:** README, About, services, sponsors and OEM notes all say free for everyone; distinguish voluntary contributions from purchased service commitments.
- **No extra licence restrictions:** no anti-fork, redistribution, confidentiality or exclusivity clause contradicts RPL.
- **Contract readiness:** legal provider identified; both intended owners’ authority established; author assignments, acceptance, maintenance and §6.5 exposure addressed.
- **Release compliance:** exact source versions, required materials, modification notices, attribution, public notification and retention responsibility documented.
- **Missing-image behaviour:** no broken Markdown image or application image request; the unrelated QR is absent from repository and packaged assets.
- **Later QR activation:** owner approval, decoded payment details, recipient name, currency, reusability and scan readability verified.
- **Application behaviour:** About works offline; opening it causes no new network requests; CSP remains intact; no payment SDK, analytics or payment verification is introduced.
- **Localisation/accessibility:** Khmer-first README; equivalent Khmer/English meaning; useful alt text; no untranslated keys; usable small-window layout and keyboard navigation.
- **Sponsor/OEM boundaries:** sponsorship never enters application assets; redistribution retains notices; service invoices do not describe licence purchases.
- **Verification claims:** run the normal build/type checks when implementation occurs, and inspect the packaged application. This read-only review did not build or test it.

G8 can be complete as **income-policy documentation plus a deliberate “KHQR unavailable” state**. Payment collection remains pending until the owner supplies and verifies the correct QR.