**NO-GO as written, for two documentation fixes.** The income model itself is compatible with free use, RPL-1.5 and the owner’s app restrictions.

1. **PC-shop wording imposes a restriction the licence does not impose.** “Never sell a ‘GDM licence’ or say the customer bought GDM” reads as a binding downstream prohibition. RPL §§3.1 and 6.0 permit distribution and prohibit adding restrictions on recipients’ rights; §6.1 specifically caps charges for source supplied under that section. Royalty-free rights do not establish a general ban on paid redistribution. Replace both language versions with:

   > GDM requires no licence purchase. Describe your charges accurately as installation, configuration or support services, and do not imply that payment purchases additional software rights or official endorsement.

   Keep “**we** do not sell licences” as your own business policy. [RPL-1.5](https://opensource.org/license/RPL-1.5)

2. **“At delivery” must not postpone obligations triggered by earlier deployment.** Organisational pilots or operational use can constitute Deploy before contractual delivery; internal Research is expressly excepted. Add in both languages:

   > We publish the source at delivery, or earlier when required by RPL-1.5. Earlier deployment remains subject to the licence’s source-publication and notification deadlines.

   Publication must cover the applicable **Source Code and Required Components**, rather than merely a feature patch. Contractually assigning compliance work does not release a deployer from its licence obligations. [RPL §§1.2, 1.12, 1.16, 6.1 and 6.4(c)](https://opensource.org/license/RPL-1.5)

Other findings, without additional blockers:

- **Free for everyone:** paid labour and commissioned development do not contradict free software access. Voluntary support gets no priority; separately contracted support can have response times.
- **NOTICE / SignPath:** no alternative licence is offered. SignPath’s current eligibility terms explicitly exclude commercial dual licensing; they do not prohibit these service fees or sponsorships. RPL §5.0 itself permits alternative licensing by eligible rights holders, so attribute your restriction to project policy and SignPath eligibility—not RPL. [SignPath terms](https://signpath.org/terms)
- **§6.5:** your summary is materially sound. Actual agreements should expressly include the indemnity’s attorney fees, costs and related expenses. “LICENSE prevails” should mean preserving software rights, without accidentally cancelling separately agreed service commitments. [RPL §6.5](https://opensource.org/license/RPL-1.5)
- **App restrictions:** the supplied change adds a bundled image and text, with no payment SDK, activation, sponsor advertising or tracking mechanism. Sponsor exclusions are strong.
- **Privacy precision:** replace “GDM never learns who paid” with “The app does not collect or receive payment confirmations.” The recipient may also be a maintainer; distinguish the app from its operators. “No external request” is more precise than “no request.”
- **Evidence limits:** the actual CSP is `img-src 'self' data:`, not only `'self'`. Your test checks external requests during the observed browser session; it does not independently establish packaged-app behaviour. I did not rerun your reported checks.
- **Languages:** I found no substantive mismatch in the supplied new Khmer/English policy text.

I concede your three declined recommendations: recipient name/currency can await the owner while no QR ships; neither a tax disclaimer nor an in-app refund procedure is necessary for this text-only build. Confirming the actual recipient and currency remains appropriate before publishing the real QR.

**Must-fix list: revise the PC-shop prohibition and qualify the delivery publication deadline, in both languages.** After those edits, GO on the reviewed G8 scope.