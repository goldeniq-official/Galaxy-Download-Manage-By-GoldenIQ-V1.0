# §22 decision policy: how development sessions decide without asking the owner (owner, 2026-11-09:
# "I told you to copy them and study those two; stop asking me"). Also referenced from every /goal command.

AUTONOMY_EN = """DECISION POLICY (owner order 2026-11-09, binding, overrides any "ask the owner" / "owner approves" line below):
Do NOT ask the owner about scope, design, UX, defaults, wording, technical choices, phase moves or approval of your own documents. Decide yourself in this order and keep going:
 1. Owner rules (CLAUDE.md) and decisions already recorded in blueprint §19 (D1-D20) and git history. Never re-ask them.
 2. The blueprint text (sections, parity ledger §04, cards §12, §22 defaults table).
 3. Not covered -> copy what IDM 6.43 does (docs/research/idm.md). Where §02/§04 lists an IDM weakness, use the GDM fix from the blueprint; where NDM is better (docs/research/ndm.md), copy NDM. If the exact IDM default is unknown, install IDM 6.43 trial in a VM/sandbox and copy it.
 4. Still open -> choose the option you would mark "(Recommended)", or the smaller reversible one; run council if it is big or hard to undo.
 Record every such decision in docs/decisions.md (date, question, choice, source: rule/§/IDM/NDM/council) and continue. Phase moves (e.g. P1 -> P2) follow the same rule: move it, record why.
Ask the owner ONLY for: spending money; accounts, identity verification or signatures that need the owner's own login or ID (Store, SignPath, extension stores, KHQR image); git push / publishing; changing LICENSE text; deleting user data or files outside the repo. Collect these in ONE Khmer message at the end of the goal, never one by one, never as approval gates for your own work. A goal is done when its evidence and council review are recorded; owner approval is not a done-condition."""

AUTONOMY_HTML = r"""
<p class="note warn"><strong>បញ្ជាម្ចាស់ (2026-11-09):</strong> "ខ្ញុំប្រាប់ឱ្យចម្លងតាម ហើយសិក្សាពីអាពីរហ្នឹង" ⇒ session អភិវឌ្ឍ<strong>មិនត្រូវសួរម្ចាស់</strong>ពីវិសាលភាព ការរចនា UX លំនាំដើម ពាក្យ ជម្រើសបច្ចេកទេស ការផ្លាស់ដំណាក់កាល ឬការអនុម័តឯកសាររបស់ខ្លួនទៀតទេ។ ច្បាប់នេះត្រូវបានដាក់នៅក្បាលពាក្យបញ្ជា /goal គ្រប់មួយ។</p>
<h3>លំដាប់នៃការសម្រេច (ឈប់នៅជំហានដំបូងដែលមានចម្លើយ)</h3>
<ol>
<li><strong>ច្បាប់ម្ចាស់</strong> (CLAUDE.md) និងសេចក្ដីសម្រេចដែលបានកត់ត្រារួចក្នុង §19 (D1–D20) និងក្នុង git។ មិនត្រូវសួរម្ដងទៀតទេ។</li>
<li><strong>Blueprint</strong>: ផ្នែកនីមួយៗ, parity ledger §04, កាត §12, និងតារាងលំនាំដើមខាងក្រោម។</li>
<li><strong>ចម្លង IDM 6.43</strong> (<code>docs/research/idm.md</code>)។ បើ §02/§04 ចុះថាជាចំណុចខ្សោយរបស់ IDM ⇒ ប្រើដំណោះស្រាយ GDM ក្នុង blueprint។ បើ NDM ល្អជាង (<code>docs/research/ndm.md</code>) ⇒ ចម្លង NDM។ បើមិនដឹងលំនាំដើមពិតរបស់ IDM ⇒ ដំឡើង IDM 6.43 trial ក្នុង VM ហើយចម្លង។</li>
<li><strong>នៅតែមិនច្បាស់</strong> ⇒ ជ្រើសជម្រើសដែលខ្លួនឯងនឹងដាក់ថា "(Recommended)" ឬជម្រើសដែលតូចជាង និងអាចត្រឡប់វិញបាន។ រឿងធំ ឬពិបាកត្រឡប់ ⇒ ប្រើ council។</li>
</ol>
<p>រាល់ការសម្រេចបែបនេះត្រូវកត់ក្នុង <code>docs/decisions.md</code> (កាលបរិច្ឆេទ, សំណួរ, ជម្រើស, ប្រភព: ច្បាប់/§/IDM/NDM/council) ហើយបន្តការងារភ្លាម។ ការផ្លាស់មុខងារពីដំណាក់មួយទៅដំណាក់មួយ (ឧ. P1 → P2) ក៏អនុវត្តច្បាប់ដដែល: ផ្លាស់ ហើយកត់ហេតុផល។</p>
<h3>រឿងដែលត្រូវសួរម្ចាស់ (តែប៉ុណ្ណេះ)</h3>
<ul>
<li>ការចំណាយលុយ។</li>
<li>គណនី ការផ្ទៀងផ្ទាត់អត្តសញ្ញាណ ឬហត្ថលេខា ដែលត្រូវការ login ឬអត្តសញ្ញាណបណ្ណរបស់ម្ចាស់ផ្ទាល់ (Store, SignPath, store extension, រូប KHQR)។</li>
<li><code>git push</code> និងការបោះពុម្ពផ្សាយ។</li>
<li>ការប្ដូរអត្ថបទ LICENSE។</li>
<li>ការលុបទិន្នន័យអ្នកប្រើ ឬឯកសារនៅក្រៅ repo។</li>
</ul>
<p>សំណួរទាំងនេះត្រូវប្រមូលជា<strong>សារខ្មែរតែមួយ</strong> នៅចុង goal មិនមែនសួរម្ដងមួយៗទេ ហើយមិនដែលប្រើជាច្រកអនុម័តការងាររបស់ខ្លួនឯងឡើយ។ Goal ចាត់ទុកថារួចរាល់ ពេលមានភស្តុតាង និងការពិនិត្យរបស់ council ហើយ។ ការអនុម័តពីម្ចាស់មិនមែនជាលក្ខខណ្ឌបញ្ចប់ទេ។</p>
<h3>លំនាំដើម (defaults) ដែលបានសម្រេចរួច</h3>
<div class="tw"><table><thead><tr><th>ធាតុ</th><th>IDM 6.43</th><th>GDM</th><th>ប្រភព</th></tr></thead><tbody>
<tr><td>Connection ក្នុងមួយការទាញ</td><td>8 (ជម្រើស 1/2/4/8/16/24/32)</td><td>ចាប់ពី 2, បន្ថែមពេលល្បឿនកើន, កម្រិត 8, អតិបរមា 32; ជម្រើសដូច IDM</td><td>§09; IDM default មិនទាន់ផ្ទៀងលើការដំឡើងពិត ⇒ ចម្លងពី IDM trial</td></tr>
<tr><td>Category លំនាំដើម</td><td>General, Compressed, Documents, Music, Programs, Video</td><td>ដូច IDM (ឈ្មោះខ្មែរ/អង់គ្លេស), ថតរងក្នុង Downloads, បញ្ជី extension ចម្លងពី IDM</td><td>idm.md §1.6</td></tr>
<tr><td>បញ្ជី extension ដែលចាប់</td><td>បញ្ជីវែង (archive, exe, media, iso ...)</td><td>ចម្លងបញ្ជី IDM ពិតពី IDM trial + ទំហំអប្បបរមា 1 MB + បញ្ជីលើកលែង C-01</td><td>idm.md §1.2, C-01</td></tr>
<tr><td>Key បង្ខំ / រំលង</td><td>Ctrl+click / Alt+click</td><td>ដូច IDM, កែបាន</td><td>idm.md §1.2</td></tr>
<tr><td>ផ្ទាំង Download Info</td><td>បង្ហាញ, ចាប់ផ្ដើមទាញភ្លាមពេលកំពុងជ្រើស</td><td>ដូច IDM + ផ្ទាំងរួមពេលចាប់ច្រើន (C-03)</td><td>idm.md §1.13</td></tr>
<tr><td>ពេលបញ្ចប់</td><td>Complete dialog</td><td>Toast មានសកម្មភាព; dialog ជាជម្រើស</td><td>§05 J3</td></tr>
<tr><td>Speed limiter</td><td>បិទ (ក្នុងមួយឯកសារ)</td><td>បិទ; ពេលបើក មានសរុប + queue + job</td><td>§04</td></tr>
<tr><td>Clipboard monitor</td><td>បើក</td><td>សួរនៅ onboarding (ឯកជនភាព)</td><td>I-01</td></tr>
<tr><td>ផ្ទាំងវីដេអូលើទំព័រ</td><td>បើក</td><td>សួរនៅ onboarding (D5)</td><td>D5</td></tr>
<tr><td>Retry ពេលបរាជ័យ</td><td>ដាក់ចុង queue; ឯកសារចុងក្រោយ retry រាល់ 30 វិ រហូត</td><td>Backoff 1→60 វិ, 10 ដង, បន្ទាប់មកផ្អាក និងបង្ហាញមូលហេតុ</td><td>§09</td></tr>
<tr><td>Download "ពេលក្រោយ" ក្នុង P1</td><td>Queue + scheduler</td><td>P1: បញ្ជីតែមួយ; queue/scheduler ពេញនៅ P2 (docs/p0/exit.md, ម្ចាស់យល់ព្រម 2026-11-09)</td><td>exit.md</td></tr>
<tr><td>Update</td><td>រំខានប្រចាំសប្ដាហ៍</td><td>ពិនិត្យរាល់ថ្ងៃ, ទាញនៅខាងក្រោយ, ដំឡើងពេល idle</td><td>§07</td></tr>
<tr><td>Theme</td><td>Dark mode មួយផ្នែក</td><td>តាមប្រព័ន្ធ, គ្រប់ផ្ទាំង</td><td>§06</td></tr>
<tr><td>ភាសា</td><td>តាម installer</td><td>ខ្មែរជាជម្រើសទីមួយ ហើយជាលំនាំដើម</td><td>R6</td></tr>
</tbody></table></div>
"""
