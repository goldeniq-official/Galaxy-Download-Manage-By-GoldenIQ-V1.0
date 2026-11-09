# §06 design system, extracted from Mazer Client (owner's preferred style) and re-coloured for GDM's logo.
import html as _h


def code(s):
    return "<pre>" + _h.escape(s.strip("\n")) + "</pre>"


DESIGN_HTML = r"""
<p>ប្រភព: Mazer Client (<code>src/styles/theme.css</code>, <code>motion.css</code>, <code>khmer.css</code>, <code>field.css</code>, <code>shell.css</code>) ដែលម្ចាស់ចូលចិត្ត។ យើងយក "DNA" របស់វាមកប្រើ ប្ដូរពណ៌ crimson ទៅជាពណ៌ខៀវ galaxy របស់ logo GDM ហើយ<strong>ទុកចោល glow ~20 កន្លែង និង border 32 កន្លែង</strong>ដែលមាននៅក្នុងកូដចាស់របស់ Mazer (មើល "មិនចម្លង" ខាងក្រោម)។</p>
<h3>DNA ដែលយកមក (ហេតុអ្វីបានជាមើលទៅ "ក្លាគ្រើម" និង "ពណ៌ប្រដេញ")</h3>
<ol>
<li><strong>ផ្ទៃខ្មៅស្រអាប់ ពណ៌ navy-violet ស្រាលៗ</strong> ដែលមានដុំពន្លឺពណ៌ alpha ទាប (radial pool) ៣ ជាន់ ធ្វើឱ្យគ្មានផ្ទៃណាមួយសំប៉ែត។ គែមផ្ទាំងមើលឃើញតាមរយៈការប្ដូរពន្លឺ មិនមែនតាមរយៈបន្ទាត់ទេ។ នេះជា "ពណ៌ប្រដេញ"។</li>
<li><strong>Accent ក្ដៅតែមួយ</strong> ជា gradient ៣ ពណ៌បង្វិល 135° (ភ្លឺនៅខាងលើឆ្វេង ងងឹតនៅខាងក្រោមស្ដាំ) មើលទៅដូចលោហៈបោះត្រា។ ប្រើតែសម្រាប់សកម្មភាពចម្បងមួយ និងស្ថានភាព "បានជ្រើស"។</li>
<li><strong>ពណ៌តាមន័យ</strong> (ជោគជ័យ, ព័ត៌មាន, ព្រមាន, កំហុស) ប្រើរាង gradient ៣ ពណ៌ដដែល។</li>
<li><strong>អក្សរក្រាស់ uppercase</strong> (Outfit 800–900, ចន្លោះអក្សរ .08–.14em) សម្រាប់ label និងប៊ូតុងអង់គ្លេស ហើយ body ស្ងាត់ 13 px។ <em>ចំពោះខ្មែរ: មិនដាក់ letter-spacing ទាល់តែសោះ</em> ព្រោះវាបំបែកជើងអក្សរ។</li>
<li><strong>បំបែកផ្នែកដោយ fill, blur និងស្រមោលខ្មៅ</strong>: chip ពណ៌ស alpha .04–.24, card មាន gradient ងងឹត, និង highlight 1 px ខាងលើ (inset ពណ៌ស alpha .02–.05 ជាពន្លឺផ្ទៃ មិនមែនបន្ទាត់ព័ទ្ធជុំវិញទេ)។</li>
<li><strong>Glass</strong> (blur 22–28 px, saturate 170–200%) សម្រាប់ chip, modal, menu និង tooltip។ <strong>Glass មិនផ្លាស់ទី</strong> ព្រោះ transform ក្រោម backdrop-filter ធ្វើឱ្យព្រិលក្នុង WebView2។ វាតែ fade និងប្ដូរ fill ប៉ុណ្ណោះ។</li>
<li><strong>State = ពន្លឺ fill, ពណ៌ icon និង fill មានពណ៌</strong>។ ជួរដែលបានជ្រើសមាន gradient wash ផ្ដេក + របារពណ៌ 3 px នៅគែម (ជាផ្ទៃ fill មិនមែនបន្ទាត់ព័ទ្ធជុំវិញ)។</li>
<li><strong>ជួរឈរមាតិកាផតចូល</strong> ក្នុង shell (ជ្រុងខាងលើឆ្វេងមូល 24 px, ស្រមោល inset ខ្មៅ) នៅចន្លោះ rail icon ដែលថ្លា។</li>
<li><strong>Control ទំហំថេរ</strong> (កំណត់តាមភាសាដែលវែងជាងគេ): ប៊ូតុង និង dialog មិនប្ដូរទំហំពេល <em>state</em> ប្ដូរ។ ទំហំត្រូវគណនាពីអត្ថបទខ្មែរ (line-height 1.5+, min-height មិនមែន height ថេរ) ដូច្នេះការប្ដូរភាសាអាចប្ដូរទំហំបាន តែការប្ដូរ state មិនអាចទេ។ លេខប្រើ <code>tabular-nums</code>។ ការប្ដូរប្រើ crossfade ដូច្នេះគ្មានអ្វីលោតទេ។</li>
<li><strong>"Dormant"</strong>: ផ្នែកដែលមិនទាន់អាចប្រើបានបង្ហាញជាពណ៌ប្រផេះ (<code>grayscale(1)</code> + token អក្សរ <code>--tx-muted</code>, មិនប្រើ brightness) តែនៅតែចុចបាន ហើយពេលចុច វាពន្យល់ថាហេតុអ្វី (ឧ. មុខងារវីដេអូមុនពេលទាញ component)។</li>
</ol>
<h3>Token (CSS variables) សម្រាប់ GDM</h3>
""" + code(r"""
:root {
  /* surfaces: Mazer's violet-black, shifted toward the logo's navy */
  --bg:        #04060c;   /* body */
  --shell:     #0f1322;   /* window shell under the colour pools */
  --sunk:      #060912;   /* sunken content column */
  --card-a:    #0c1020;  --card-b: #080b16;          /* card vertical gradient */
  --chip:      rgba(255,255,255,.08); --chip-hover: rgba(255,255,255,.16); --chip-press: rgba(255,255,255,.22);
  --field:     rgba(255,255,255,.04); --field-focus: rgba(255,255,255,.10);
  --focus-plate: rgba(255,255,255,.40);   /* measured 3.5:1 vs rest .04 on --sunk; white text on it 5.31:1 */

  /* the one hot accent: galaxy blue, 135deg three-stop (logo #1890f0 / #0048a8 / #003090) */
  /* every stop passes 4.5:1 with white text: #1f6fe0 4.76, #1660d0 5.80, #0b3a9c 10.0 (measured) */
  --accent-grad:        linear-gradient(135deg, #1f6fe0 0%, #1660d0 45%, #0b3a9c 100%);
  --accent-grad-hover:  linear-gradient(135deg, #1f6fe0 0%, #1f6fe0 45%, #1048b0 100%);  /* lighter, still >= 4.76 */
  --accent:             #3d9bff;   /* icon / text accent */
  --accent-wash:        rgba(61,155,255,.18);   /* selected chip fill */
  --lime:               #b5e61d;   /* logo lime: "complete" only, never decoration */

  /* semantic, same 135deg shape */
  --ok-grad:   linear-gradient(135deg, #b5e61d 0%, #7cc414 45%, #3f7a0b 100%);
  --info-grad: linear-gradient(135deg, #38bdf8 0%, #0284c7 45%, #0c4a6e 100%);
  --warn-grad: linear-gradient(135deg, #f59e0b 0%, #d97706 45%, #92400e 100%);
  --err-grad:  linear-gradient(135deg, #fb7185 0%, #e11d48 45%, #881337 100%);

  /* text */
  --tx-1: #ffffff; --tx-2: #e4e7f0; --tx-3: #c6cbd9; --tx-muted: #8a91a6; --tx-ph: rgba(255,255,255,.35);

  /* glass (Linux/no-blur: blur none, alpha ~.95) */
  --glass-card: linear-gradient(160deg, rgba(28,34,56,.55), rgba(10,12,22,.68));
  --glass-pop:  linear-gradient(180deg, rgba(34,40,62,.64), rgba(16,20,34,.74));
  --blur-card:  blur(28px) saturate(180%);  --blur-chip: blur(24px) saturate(200%);
  --scrim:      rgba(2,4,10,.6);

  /* depth = black only */
  --shadow-card: 0 18px 44px rgba(0,0,0,.5);  --shadow-modal: 0 30px 80px rgba(0,0,0,.55);
  --hi-top: inset 0 1px 0 rgba(255,255,255,.04);   /* surface light, not an outline */

  /* space, radius, motion: Mazer values */
  --space-1: 6px; --space-2: 10px; --space-3: 14px; --space-4: 18px; --space-5: 22px;
  --r-sm: 9px; --r-md: 12px; --r-lg: 14px; --r-xl: 20px; --r-pill: 999px;
  --motion-press: 90ms; --motion-hover: 150ms; --motion-quick: 180ms; --motion-base: 220ms; --motion-slow: 320ms;
  --ease-out: cubic-bezier(.16,1,.3,1); --ease-in: cubic-bezier(.4,0,1,1); --ease-in-out: cubic-bezier(.45,0,.35,1);

  /* z-index ladder (Mazer had literals; GDM makes it a scale) */
  --z-sticky: 10; --z-toast: 60; --z-modal: 100; --z-menu: 200; --z-titlebar: 300; --z-tooltip: 400;

  --f-latin: "Outfit"; --f-khmer: "GDM Khmer"; --f-mono: "JetBrains Mono";
  font-family: var(--f-latin), var(--f-khmer), system-ui, sans-serif;
}
body {
  background:
    radial-gradient(circle at 12% 18%, rgba(61,155,255,.10), transparent 32%),   /* blue pool */
    radial-gradient(circle at 86% 12%, rgba(139,92,246,.09), transparent 30%),   /* violet pool */
    radial-gradient(circle at 72% 92%, rgba(56,189,248,.06), transparent 34%),   /* sky pool (lime stays "complete only") */
    var(--bg);
}
""") + r"""
<h3>ពណ៌</h3>
<div class="grid2">
<div class="card"><div class="swatch"><i style="background:linear-gradient(135deg,#1f6fe0,#1660d0 45%,#0b3a9c)"></i>accent-grad: ប៊ូតុងចម្បង, ជួរដែលបានជ្រើស</div>
<div class="swatch"><i style="background:#b5e61d"></i>lime: "បញ្ចប់" ប៉ុណ្ណោះ (ពី logo)</div>
<div class="swatch"><i style="background:linear-gradient(135deg,#f59e0b,#d97706 45%,#92400e)"></i>warn: ផ្អាក, ត្រូវការ link</div>
<div class="swatch"><i style="background:linear-gradient(135deg,#fb7185,#e11d48 45%,#881337)"></i>err: បរាជ័យ, ឯកសារគ្រោះថ្នាក់</div></div>
<div class="card"><div class="swatch"><i style="background:#04060c"></i>bg</div><div class="swatch"><i style="background:#0f1322"></i>shell</div><div class="swatch"><i style="background:#060912"></i>sunk</div><div class="swatch"><i style="background:linear-gradient(#0c1020,#080b16)"></i>card</div></div>
</div>
<p>Contrast វាស់លើ stop ភ្លឺបំផុតនៃ gradient (មិនមែនពណ៌កណ្ដាល): អក្សរពណ៌សលើ <code>#1f6fe0</code> = 4.76:1 (ឆ្លង AA)។ ពណ៌ពី draft ដំបូង <code>#4aa8ff</code> មាន 2.52:1 ហើយមិនឆ្លង ដូច្នេះត្រូវបានដកចេញ (ChatGPT ជុំ 3)។ Theme ភ្លឺប្រើផ្ទៃ <code>#f4f6fb</code>/<code>#ffffff</code> ហើយ accent អក្សរ <code>#0b6fd6</code> (4.93:1)។ CI តេស្ត contrast គ្រប់ token pair ក្នុងករណីទាំងនេះ: theme ពីរ, forced-colors (High Contrast) និង "Transparency effects off" (glass ⇒ ផ្ទៃរឹង)។</p>
<h3>ការបង្ហាញស្ថានភាព (គ្មាន outline, គ្មាន glow)</h3>
<div class="tw"><table><thead><tr><th>State</th><th>ការធ្វើ</th></tr></thead><tbody>
<tr><td>Hover</td><td>Fill ភ្លឺឡើង (<code>--chip</code> .08 → .16) ឬ icon ពីពណ៌ muted ទៅជាពណ៌ស។ ប៊ូតុង gradient ប្ដូរទៅ token <code>--accent-grad-hover</code> ដែលវាស់ contrast រួច (មិនប្រើ <code>filter: brightness()</code> ព្រោះវាអាចបន្ថយ contrast) + លើក 1 px (មិនប្រើលើ glass)។</td></tr>
<tr><td>Press</td><td>ប៊ូតុង solid: <code>scale(.97)</code> រយៈ 90 ms។ Glass: fill .22 (មិនផ្លាស់ទី)។</td></tr>
<tr><td>Selected</td><td>Fill <code>--accent-wash</code> + icon accent (Phosphor weight <em>fill</em>)។ ជួរក្នុងបញ្ជី: gradient wash ផ្ដេក + របារ 3 px នៅគែមឆ្វេង។</td></tr>
<tr><td>Keyboard focus</td><td>(ត្រូវការការយល់ព្រមពីម្ចាស់ D6) បង្ហាញតែពេលប្រើ keyboard (<code>data-input="keyboard"</code>, គំរូពី Mazer <code>inputModality.ts</code>)។ Focus ត្រូវខុសពី hover និង selected (<strong>វាស់រួច</strong>): វត្ថុធម្មតា ⇒ <code>--focus-plate</code> ពណ៌ស .40 (3.5:1 ធៀប rest .04; អក្សរពណ៌សលើវា 5.31:1) + icon weight fill។ Fill .24 ដែលបានស្នើពីដំបូងទទួលបានត្រឹម 1.91:1 ហើយមិនឆ្លង 3:1 ទេ ដូច្នេះត្រូវដកចេញ (ChatGPT r5)។ វត្ថុដែល selected ⇒ fill accent solid <code>#1f6fe0</code> (3.6:1 ធៀប wash .18; អក្សរពណ៌ស 4.76:1)។ ប៊ូតុង gradient ⇒ <code>--accent-grad-hover</code> + អក្សរ underline 2 px (ផ្ទៃខាងក្នុង មិនមែន outline)។ Forced-colors ⇒ ប្រើ <code>Highlight</code> ជា fill (Windows គ្រប់គ្រង)។ គ្មាន ring ពណ៌ស។ Test: គ្រប់ component × state (rest, hover, selected, disabled) មាន focus ដែលមើលឃើញ (WCAG 2.4.7 AA) និងមិនត្រូវបាំង (2.4.11)។ ប៊ូតុង និងជួរមានគោលដៅ 2.4.13 AAA: contrast ≥ 3:1 លើ pixel ដដែលធៀប state មុន និងផ្ទៃអប្បបរមាតាម 2.4.13។ Field អនុវត្តតែច្បាប់ AA ដែលបានកត់ត្រាខាងក្រោម។ ការវាស់ពណ៌ខាងលើមិនទាន់ជាការបញ្ជាក់ conformance ពេញលេញទេ ត្រូវតេស្តលើ UI ពិត។</td></tr>
<tr><td>Field focus</td><td>Fill .04 → .10 + label ពណ៌ accent + caret ពណ៌ accent។ ការប្ដូរ fill នេះ (1.17:1) <strong>មិនដល់ 3:1</strong> ទេ។ វាឆ្លង WCAG 2.4.7 (AA, focus មើលឃើញ) តាមរយៈ label និង caret ពណ៌ accent តែមិនឆ្លង 2.4.13 (AAA)។ នេះជាការសម្រេចដោយដឹងខ្លួន ព្រោះ fill .40 ធ្វើឱ្យ field ធ្ងន់ពេកសម្រាប់ការវាយ។ ច្បាប់ដូចគ្នាទាំង keyboard និង mouse។</td></tr>
<tr><td>Disabled</td><td>Opacity .45។ ប៊ូតុងចម្បងក្លាយជា fill ប្រផេះ .08 និងអក្សរ .42 ("inert is grey")។</td></tr>
<tr><td>Dormant</td><td><code>grayscale(1)</code> + អក្សរ <code>--tx-muted</code> (#8a91a6, 6.3:1 លើ --sunk) ហើយនៅតែចុចបាន ដើម្បីពន្យល់មូលហេតុ។ មិនប្រើ <code>brightness()</code>។</td></tr>
<tr><td>Danger</td><td>អក្សរ <code>#f87171</code>, hover fill ក្រហម .14។ Modal លុបប្រើ glass ពណ៌ក្រហម។</td></tr>
</tbody></table></div>
<h3>ទម្រង់ responsive (ឆ្លាតតាមទំហំ window)</h3>
<p>GDM ខុសពី Mazer។ Mazer ប្រើ composition ថេរ 1190×745 ហើយ zoom webview ទាំងមូល។ ប៉ុន្តែ download manager ត្រូវការ window តូចផង (ដាក់ក្បែរ browser)។ ដូច្នេះ GDM ប្រើ <strong>container query តាមទំហំ window</strong> សម្រាប់ layout ហើយប្រើ <strong>zoom webview តាម DPI</strong> តាមគំនិត Mazer <code>window_zoom.rs</code> តែចំពោះអេក្រង់ធំខ្លាំងប៉ុណ្ណោះ (&gt; 1920 logical px)។</p>
<div class="tw"><table><thead><tr><th>ទម្រង់</th><th>ទទឹង window</th><th>រចនាសម្ព័ន្ធ</th></tr></thead><tbody>
<tr><td>Mini</td><td>360–479 (កម្ពស់ ≥ 140)</td><td>ផ្ទាំង progress តែមួយ (ជំនួស progress dialog របស់ IDM): ឈ្មោះ, របារ, ល្បឿន, ពេលនៅសល់, [ផ្អាក] [⋯]។ "Always on top" ជាជម្រើស។</td></tr>
<tr><td>Compact</td><td>480–759</td><td>បញ្ជីតែមួយជួរឈរ (ឈ្មោះ + របារ + ល្បឿន/ស្ថានភាពជាអក្សរតូច)។ Filter ជា tab ផ្ដេក (ទាំងអស់/កំពុងទាញ/បញ្ចប់/queue)។ ប៊ូតុង "+" នៅជ្រុង។ Details ជា sheet ពីក្រោម។</td></tr>
<tr><td>Medium</td><td>760–1179</td><td>Rail icon 64 px (category/queue) + តារាងមាន column (ឈ្មោះ, ទំហំ, ស្ថានភាព, ល្បឿន, ពេលនៅសល់)។ Details ជា drawer ពីស្ដាំ (ពេលចុច)។</td></tr>
<tr><td>Wide</td><td>≥ 1180</td><td>Rail ពង្រីក 220 px (ឈ្មោះ category ពីរភាសា, ចំនួន) + តារាង + details pane ស្ដាំ 340 px (segment bar, connection, URL, សកម្មភាព)។</td></tr>
</tbody></table></div>
<ul>
<li>Window លំនាំដើម 1100×700, អប្បបរមា 360×140 (mini)។ ចងចាំទំហំ/ទីតាំង (Tauri window-state) ហើយបើទីតាំងចាស់នៅក្រៅអេក្រង់ ⇒ ដាក់កណ្ដាល (Mazer smart-window-sizing)។</li>
<li>DPI: Per-Monitor V2 (Tauri default)។ Icon ជា vector។ Tray ICO ពហុទំហំ។ តេស្ត 100/125/150/200% និង monitor ពីរ។</li>
<li>Title bar ផ្ទាល់ខ្លួន (decorations: false) ដូច Mazer, ប៉ុន្តែប៊ូតុង min/max/close ប្រើ Windows snap layout (hover លើ maximize) តាម <code>WM_NCHITTEST HTMAXBUTTON</code>។ បើធ្វើមិនបាន ⇒ ប្រើ decorations ធម្មតា (spike P0)។</li>
<li>Touch/pen: គោលដៅប៉ះ ≥ 40 px ក្នុង compact។</li>
<li>Extension popup (360×520) ប្រើ token ដដែល ដូច្នេះមើលទៅជាផលិតផលតែមួយ។</li>
</ul>
<h3>Component សំខាន់ៗ</h3>
<div class="tw"><table><thead><tr><th>Component</th><th>Spec (ពី Mazer, ប្ដូរពណ៌)</th></tr></thead><tbody>
<tr><td>ប៊ូតុងចម្បង</td><td>កម្ពស់អប្បបរមា 40, r12, <code>--accent-grad</code>, 11 px/800/.1em uppercase (EN) ឬ 13 px/600 (KM), inset top highlight .14</td></tr>
<tr><td>ប៊ូតុងបន្ទាប់បន្សំ</td><td>កម្ពស់ 40, r12, fill ពណ៌ស .07 → .13 (hover)</td></tr>
<tr><td>Icon chip</td><td>36 មូល (header), 44 r12 (rail), glass chip + blur</td></tr>
<tr><td>Field</td><td>Fill .04, មិនដែលមាន border, ពេល focus fill .10 + label/caret accent, ប៊ូតុង clear មូល</td></tr>
<tr><td>Toggle</td><td>Track 40×24 pill ពណ៌ស .12។ ពេល on ⇒ accent-grad, thumb រំកិល 16 px រយៈ 180 ms</td></tr>
<tr><td>Tab segmented</td><td>Capsule accent រំកិលពីក្រោយអក្សរ (220 ms) <strong>គ្មាន</strong>ស្រមោលពណ៌ (Mazer មាន glow ត្រង់នេះ)</td></tr>
<tr><td>ជួរ download</td><td>កម្ពស់ 56 (wide) / 64 (compact, មាន 2 បន្ទាត់), r12, fill .04, hover .08, selected = wash + របារ 3 px។ Icon ប្រភេទឯកសារ, ឈ្មោះ (truncate grapheme), ស្លាក extension ពិត (N-02)</td></tr>
<tr><td>Progress bar</td><td>Track 4 px pill ពណ៌ស .10។ Fill <code>--accent-grad</code> ជាមួយ <code>background-size</code> ថេរតាម track (ពណ៌មិនរត់ពេលរបារវែង: ល្បិច Mazer)។ Transition width linear។ ពេលផ្អាក ⇒ warn-grad។ ពេលបញ្ចប់ ⇒ lime។ គ្មាន glow គ្មានចំណុចភ្លឺលោត</td></tr>
<tr><td>Segment bar (ថ្មី)</td><td>កម្ពស់ 10, r6។ ផ្នែកនីមួយៗជា block (connection សកម្ម = accent, ទាញរួច = accent 60%, ទំនេរ = ពណ៌ស .06)។ ចន្លោះ 1 px ជាផ្ទៃ bg (មិនមែនបន្ទាត់)។ Hover បង្ហាញ tooltip (offset, ល្បឿន)</td></tr>
<tr><td>Toast</td><td>ទទឹង min(380, 100vw−40), r16, ផ្ទៃ <code>#1a1f30→#11151f</code> + wash ពណ៌តាមប្រភេទពីឆ្វេង, badge 40 r12 gradient, សកម្មភាពមួយ, ស្រមោលខ្មៅតែប៉ុណ្ណោះ, ចូល 320 ms ចេញ 180 ms</td></tr>
<tr><td>Modal</td><td>Glass card r20, scrim blur 10, fade only, ទំហំថេរ, focus trap</td></tr>
<tr><td>Context menu / confirm</td><td>Popover glass ចង្អុល (clip-path shape), item កម្ពស់ 38 r9, confirm តូចនៅក្បែរប៊ូតុងលុប (មិនប្រើ modal ធំ)</td></tr>
<tr><td>Tooltip</td><td>Glass raised, 12 px/600, fade 180 ms, គ្មាន accent</td></tr>
<tr><td>Scrollbar</td><td>4 px, thumb <code>#27304a</code>, <code>scrollbar-gutter: stable</code></td></tr>
</tbody></table></div>
<h3>អក្សរ</h3>
<ul>
<li>Latin: Outfit 400–900 (OFL)។ Mono: JetBrains Mono សម្រាប់ទំហំ, ល្បឿន, URL, hash។ ពុម្ពទាំងអស់ភ្ជាប់ក្នុង app (CSP <code>font-src 'self'</code>) មិនទាញពី CDN។</li>
<li>Khmer: Kantumruy Pro ចុះឈ្មោះជា <code>"GDM Khmer"</code> ជាមួយ <code>unicode-range: U+1780-17FF, U+19E0-19FF, U+200B-200D, U+25CC</code> និង <code>size-adjust: 108%</code> (យកពី Mazer khmer.css)។ ដូច្នេះអក្សរឡាតាំង និងលេខក្នុងប្រយោគខ្មែរគូរដោយ Outfit។</li>
<li><code>html:lang(km)</code>: letter-spacing normal គ្រប់កន្លែង, line-height ≥ 1.5 (body) និង 1.55 (p/li/label)។ Uppercase មិនអនុវត្តលើខ្មែរ។</li>
<li>លេខ: លេខឡាតាំងជាលំនាំដើម (<code>-u-nu-latn</code>), <code>tabular-nums</code>។ ជម្រើស "លេខខ្មែរ" (០១២) ក្នុង Settings សម្រាប់ទំហំ និងកាលបរិច្ឆេទ។</li>
<li>ទំហំ: ចំណងជើង 20/700, label 13/600, body 13/500, caption 11.5 muted, លេខក្នុងតារាង 12.5 mono។</li>
</ul>
<h3>Icon</h3>
<p><strong>Phosphor Icons</strong> (MIT) ប្រើតែមួយបណ្ណាល័យ (Mazer ប្រើ FA + lucide ពីរ ហើយ agent ណែនាំឱ្យជ្រើសមួយ)។ ហេតុផល: Phosphor មាន weight <em>regular</em> និង <em>fill</em> ដែលមានរាងដូចគ្នា ដូច្នេះ state "selected" អាចប្ដូរពី stroke ទៅ fill បាន ដោយមិនបាច់ប្រើ outline។ ទំហំ 16/20/24, stroke 1.5 ថេរ (<code>vector-effect: non-scaling-stroke</code> ដូច <code>.mz-icon</code>)។ Icon ប្រភេទឯកសារ: Phosphor FileZip, FileVideo, FileAudio, FileDoc, AppWindow (exe) ដែលមានពណ៌តាមប្រភេទ។</p>
<h3>ចលនា</h3>
<ul><li>Token 6 duration និង 3 curve (Mazer)។ ផ្លាស់ទីតែ transform/opacity។ ចូលប្រើ ease-out ចេញប្រើ ease-in (180 ms)។ គ្មាន loop លើកលែងតែ spinner នៃការងារពិត។</li><li>Progress ប្រើ linear។ Reduce motion: ជម្រើសក្នុង app (<code>data-motion="reduce"</code>) + OS media query។</li></ul>
<h3>ផ្ទាំងចាប់ផ្ដើម និង first-run (យកពី Mazer)</h3>
<ul><li>Splash HTML inline ដែលបង្ហាញនៅ frame ដំបូង (&lt; 1 វិនាទី): logo + របារ progress ពិត។ <strong>គ្មាន</strong> "ព្រះអាទិត្យភ្លឺ" និងចំណុចលោត (glow ក្នុង Mazer splash)។</li><li>Onboarding: ផ្លូវទៅមុខតែងតែជាវត្ថុដែលភ្លឺជាងគេលើអេក្រង់ (ប៊ូតុង accent មួយ)។ ផ្នែកផ្សេងស្ថិតក្នុងស្ថានភាព "dormant"។</li><li>កំហុសបង្ហាញនៅក្រោមប៊ូតុង ដោយមិនរុញប៊ូតុង។ Empty state: ប៊ូតុង pill មួយ + ឃ្លាមួយ ("អូស link មកទីនេះ ឬចុច +")។</li></ul>
<h3>មិនចម្លងពី Mazer (បំពានច្បាប់ម្ចាស់ ឬមិនពាក់ព័ន្ធ)</h3>
<ul>
<li>Glow ទាំងអស់: ស្រមោលពណ៌ក្រោម nav marker, ប៊ូតុង LAUNCH, capsule tab, radio, halo "spotlight", ring "cue" នៃ CTA, ព្រះអាទិត្យ និងចំណុចលោតក្នុង splash, orb ពណ៌ក្នុង modal។</li>
<li>Border និង outline ព័ទ្ធជុំវិញ panel, ជួរ, ប៊ូតុង, field, radio ring និង unread-dot ring ក្នុង <code>shell.css:1446-2060</code>។ Outline ពណ៌ផ្កាឈូកក្នុង reduced-motion ក៏មិនចម្លងដែរ។</li>
<li>Ring keyboard ពណ៌ស (focus.css) ត្រូវជំនួសដោយ fill (D6)។</li>
<li>បណ្ណាល័យ icon ពីរ, ពណ៌ literal ដែលរាយប៉ាយ, z-index ad-hoc, Tailwind ដែលត្រូវ CSS ផ្សេងជាន់ពីលើដោយស្ងាត់ (GDM ប្រើ CSS variables + CSS modules ប៉ុណ្ណោះ)។</li>
<li>Noise ដែលមិនដំណើរការ (dead code)។ បើចង់បានរាង "គ្រើម" ពិត ⇒ grain SVG លើ pseudo-element ដាច់ដោយឡែក alpha .03 ហើយត្រូវតេស្ត performance លើ WebView2 មុន (ម្ចាស់មើលហើយសម្រេចក្នុង mockup P0)។</li>
</ul>
"""
