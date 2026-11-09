# Mazer Client: design language extract (for reuse in a Tauri + React download manager)

Repo root: `C:\Users\yrgol\OneDrive\Documents\Code\MAZER-CLUB\PC APP\Mazer Client` (all paths below are relative to it).
Stack: Tauri v2, React 19, Tailwind v4 (`@import "tailwindcss"`, but nearly all real styling is plain unlayered CSS that beats utilities, see `src/styles/shell.css:4-14`), i18next.
Stylesheet entry: `src/App.css:4-16` imports, in order: tailwind, theme, motion, popover, khmer, shell, profiles, dormant, firstrun, platform, field, icon, focus.

---

## 1. Design tokens

### 1a. Brand + surface + fonts (`src/styles/theme.css:4-18`, Tailwind `@theme`)
```css
@theme {
  --color-brand-400: #fb7185;
  --color-brand-500: #e11d48;
  --color-brand-600: #be123c;
  --color-brand-900: #881337;

  --color-ui-base: #050505;
  --color-ui-shell: #0a0a0a;
  --color-ui-panel: #111111;
  --color-ui-border: #1f1f1f;
  --color-ui-border-hover: #333333;

  --font-sans: "Outfit", "Mazer Khmer", sans-serif;
  --font-mono: "JetBrains Mono", "Mazer Khmer", monospace;
}
```
Note: in practice the real surface colours are literals in shell.css (`#131318` shell, `#070709` main column, `#0c0c0e` cards), not these tokens.

### 1b. Glass tokens (`src/styles/theme.css:36-68`), with a Linux override (`:76-96`)
```css
:root {
  --glass-blur-card: blur(28px) saturate(180%);       /* modals, cards */
  --glass-blur-scrim: blur(12px) saturate(120%);      /* behind a modal */
  --glass-blur-chip: blur(24px) saturate(200%);       /* small round surfaces */
  --glass-blur-chip-hover: blur(16px) saturate(180%);
  --glass-blur-overlay: blur(30px) saturate(105%);

  --glass-card: linear-gradient(160deg, rgba(30,30,40,.55), rgba(12,12,16,.68));
  --glass-card-danger: linear-gradient(160deg, rgba(40,22,26,.72), rgba(12,12,16,.82));
  --glass-chip: rgba(255,255,255,.1);
  --glass-chip-hover: rgba(255,255,255,.18);
  --glass-scrim: rgba(4,4,6,.72);

  --glass-blur-popover: blur(22px) saturate(170%);
  --glass-popover: linear-gradient(180deg, rgba(34,34,46,.64), rgba(18,18,26,.74));
  --glass-popover-raised: linear-gradient(180deg, rgba(64,64,78,.66), rgba(46,46,58,.76)); /* tooltips */
}
/* Linux/WebKitGTK has no backdrop-filter: blur -> none, same hues, alpha raised to ~.94-.98 */
:root[data-platform="linux"] { --glass-blur-*: none; --glass-card: linear-gradient(160deg, rgba(38,38,50,.94), rgba(14,14,19,.96)); --glass-chip: rgba(255,255,255,.15); --glass-chip-hover: rgba(255,255,255,.24); --glass-scrim: rgba(4,4,6,.88); --glass-popover: linear-gradient(180deg, rgba(34,34,46,.97), rgba(20,20,28,.98)); --glass-popover-raised: linear-gradient(180deg,#3a3a45,#2f2f39); }
```
`data-platform` is written on `<html>` before first paint (`src/main.tsx:32`, `src/platform/platform.ts`).

### 1c. Spacing + radius (`src/styles/shell.css:37-44`)
```css
:root { --space-1: 6px; --space-2: 10px; --space-3: 14px; --space-4: 18px; --space-5: 22px; --panel-radius: 14px; }
```
Radius scale actually used: 8-9px (menu items, small inputs), 10-12px (buttons, nav chips 12px, launch button 12px), 14px (panels, popovers, rows), 16px (toasts), 18px (square tiles), 20px (modal card), 24px (main column top-left corner, `shell.css:197`), 999px (pills: window controls, CTA, tabs, toggle, clear button).
Content cap: `--content-max: 1400px` (`shell.css:228-236`).

### 1d. Motion tokens (`src/styles/motion.css:11-50`; guide `docs/motion/motion-guide.md`)
```css
:root {
  --motion-press: 90ms;   --motion-hover: 150ms;  --motion-quick: 180ms;
  --motion-base: 220ms;   --motion-slow: 320ms;   --motion-emphasis: 480ms;
  --motion-stagger: 30ms;
  --ease-out: cubic-bezier(0.16, 1, 0.3, 1);      /* house curve: enters, hover, press */
  --ease-in: cubic-bezier(0.4, 0, 1, 1);          /* exits only */
  --ease-in-out: cubic-bezier(0.45, 0, 0.35, 1);  /* A->B moves, indicators */
  --ease-spring: cubic-bezier(0.34, 1.56, 0.64, 1); /* tick/badge only */
  --motion-rise-small: 6px;
}
@theme { --default-transition-duration: 150ms; --default-transition-timing-function: cubic-bezier(0.16,1,0.3,1); }
```
The same numbers live in `src/lib/motion.ts`, kept equal by `motion.test.ts` (timers waiting on exits read them).

### 1e. Typography sizes/weights (no tokens, recurring literals)
- Display title: `clamp(2.3rem, 4.1vw, 3.3rem)`, line-height .98 (`shell.css:835-853`).
- Big action labels: 17px / 900 / `letter-spacing: .14em` / uppercase (`shell.css:1150-1168`).
- Section headings: 14px / 900 / .14em / uppercase (`shell.css:1469-1477`).
- Buttons: 11px / 800-900 / .08-.12em / uppercase (`CreateSetupModal.css:183-199`, `SetupSettings.css:999-1015`).
- Body/menu: 12-13px / 500-600. Captions 11-11.5px, muted `#8c8f99`. Field text 12.5px.
- Numbers: `font-variant-numeric: tabular-nums` everywhere a number changes.
- Khmer line-height: body 1.5, p/li/label/dd/dt 1.55, never below 1.4 (`src/styles/khmer.css:97-107`).

### 1f. Z-index ladder (literal values)
resize handles 1000 (`shell.css:2068`), menus/confirm popovers 1000 (`SetupPage.css:367`, `SetupBody.css:1025`), toasts 60 (`SetupBody.css:2285`), shell sheen 60 (`shell.css:178`), accounts modal 100 (`AccountsModal.css:66`), lightbox 1200, window controls 1300 (`shell.css:465-468`, always above dialog scrims), splash 9999 (`index.html:61`), tooltip 2147483000 (`tooltip.css:35`). No token scale; worth making one in the new app.

---

## 2. Colour system

### Base palette
- Near-black, slightly cool/violet blacks: `#050506` body, `#131318` shell, `#070709` main/right column, `#0c0c0e`/`#0b0b0e`/`#09090c` cards.
- Brand crimson ramp (the "chasing" gradient), used for every primary action:
  `linear-gradient(135deg, #e11d48 0%, #be123c 45%, #881337 100%)` (LAUNCH, CTA, toast error badge; `shell.css:1042`, `firstrun.css:44`).
  A brighter sibling for in-dialog primaries/selected chips/toggles: `linear-gradient(135deg, #ff3856 0%, #ee1f39 50%, #c81028 100%)`, hover `#ff4a66/#f22c45/#d4172f` (`CreateSetupModal.css:232-243`, `SetupSettings.css:723-725`).
- Accent signal red: `#ef4444` (active nav icon, close-button hover, unread dot, red icons).
- Secondary ambient hues (never on controls, only in background light pools): violet `rgba(139,92,246,…)`, sky `rgba(56,189,248,…)`.
- Semantic three-stop gradients in the same 135deg shape (`SetupBody.css:2308-2345`):
  success `#10b981 → #059669 → #065f46`, info `#3b82f6 → #2563eb → #1e3a8a`, warn `#f59e0b → #d97706 → #92400e`, error = brand crimson. Running state green `#059669 → #047857 → #064e3b` (`shell.css:1068`).
- Text: `#ffffff` primary, `#f4f4f5/#e4e4e7` secondary, `#d4d4d8` tertiary, `#8c8f99`/`#71717a` muted, `rgba(255,255,255,.35)` placeholder. Error text `#fca5a5`/`#fecaca`; success text `#a7f3d0`.

### Layering ("ពណ៌ប្រដេញ": colour pools chasing each other across layers)
1. `body` (`shell.css:57-67`): three radial pools on `#050506`: red at 12%/18%, violet at 86%/12%, sky at 72%/92%, each fading to transparent by 30-34%.
2. `.app-shell` (`shell.css:113-122`): five more layers: red pool top-left, violet top-right, violet bottom-left, plus two diagonal linear washes (135deg pink `rgba(255,77,95,.055)` to 25%, 315deg violet to 28%) over `#131318`.
3. `.app-shell::before` sheen (`shell.css:173-184`): `linear-gradient(110deg, rgba(255,255,255,.055), transparent 15%, transparent 80%, rgba(139,92,246,.035))` over the whole window, plain alpha (screen blend broke under WebView2).
4. `.app-main` (`shell.css:186-204`): `#070709` with faint red pool 18%/8% and sky pool 92%/88%, `border-top-left-radius: 24px`, `box-shadow: inset 10px 10px 20px rgba(0,0,0,.2)` (sunken).
5. Right rail (`shell.css:346-362`): `#070709` + violet pool, inset top shadow.
6. Cards: dark vertical gradient (`#0c0c0f → #09090c`) + tiny inset top highlight `inset 0 1px 0 rgba(255,255,255,.018-.05)` + black drop shadow.
7. Glass chips/modals/popovers float on top (section 3).
Result: the header and left rail have no surface of their own (transparent), they sit on the shell glow; the content column is a darker sunken slab cut into it.

### State without outline/glow (the intended pattern)
- Hover: fill brightens (`--glass-chip` .10 → .16/.18; solid `#121216 → #17171c`), or icon colour muted → white. Gradient buttons: `filter: brightness(1.1)` + `translateY(-1px)` (only on non-glass).
- Active/press: `scale(.97)` + `brightness(.96)` over 90ms on solid buttons; glass gets a lighter fill instead (`.version-select-btn:active` `.24`, `shell.css:1363`), because "glass does not move" (`motion.css:57-73`).
- Selected: tinted fill + coloured icon (`.nav-btn.active` `rgba(239,68,68,.22)` + `#ef4444`, `shell.css:601-605`); account row active = horizontal red gradient fill + 3px red bar on the left edge (`AccountRow.css:26-50`); tabs = sliding crimson capsule behind white text (`shell.css:2611-2626`).
- Disabled: opacity .4-.5; primary drops to grey fill `rgba(255,255,255,.08)` with text `.42` ("inert is grey", `CreateSetupModal.css:245-258`); toggle checked+disabled turns grey (`SetupSettings.css:739-751`).
- Focus: fields brighten fill only (`field.css:44-48`); menu items/buttons share the hover fill on keyboard focus; elsewhere a single white ring, keyboard-only (section 9).
- Asleep/locked ("dormant"): `filter: grayscale(1) brightness(.92) contrast(.9)` per element, waking with a staggered 480ms sweep (`src/styles/dormant.css:22-100`).
- Danger: red text `#f87171`, hover fill `rgba(239,68,68,.14)`; danger confirm button solid `rgba(239,68,68,.9)` → `#ef4444` (`SetupPage.css:446-505`).

### Violations of the owner's rules found (do NOT copy)
Glow (coloured box-shadow / drop-shadow / halo):
- `shell.css:626` active nav marker `box-shadow: 0 0 12px rgba(239,68,68,.6)`.
- `shell.css:1043-1053, 1069-1071, 1094, 1109` LAUNCH idle/running/fill coloured shadows (`rgba(190,18,60,.4-.5)`, green `.4`).
- `shell.css:2619`, `CreateSetupModal.css:511`, `SetupBody.css:53, 745`, `VersionModal.css:189` crimson capsule shadow `0 2px 10px rgba(238,31,57,.4)`.
- `AccountRow.css:94` active radio `0 0 10px rgba(239,68,68,.5)`; `LogView.css:195` same.
- `dormant.css:128-161` `.account-card--spotlight` breathing red halo + red border (live in `AccountSwitcherCard.tsx`; visible in `first-run-dormant` screenshot).
- `firstrun.css:88-103` CTA "cue" expanding red ring; its reduced-motion fallback is a pink outline (`firstrun.css:161-172`).
- `StoreView.css:133, 410, 417`, `ShopPage.css:116`, `ToolsPage.css:136, 598`, `SupportWidget.css:948`, `SetupSeal.css:53`, `SetupSettings.css:975` (3px red ring).
- Splash (`index.html:168-193` blurred breathing orange sun behind logo, `:270` fill glow, `:285-288` glowing pulsing leading dot).
- `AccountsModal.css:118-139` blurred red/violet "orbs" inside the modal card (soft coloured light; borderline, treat as glow).
Outlines/borders round things (32 `border: Npx solid|dashed` rules; most in older mockup-port code in `shell.css`):
- `shell.css:1452` setups panel, `:1483` New Setup button, `:1524` dashed empty card, `:1551` icon tile, `:1597` retry button, `:1624` account card (violet), `:1700/1716` offline form, `:1746` servers panel (sky), `:1793` server rows, `:2050-2059` gradient border on Mazer server row, `:454` unread dot ring, `:354` rail border-left, `:3335/3427` dashed add tiles.
- `AccountRow.css:86` radio ring (2px), `CreateSetupModal.css:1523`, `SupportWidget.css:1336/2067`.
- Hairline dividers (`border-bottom: 1px rgba(255,255,255,.08)` in modal headers, settings sidebar `border-right`) are separators, not outlines round a control; acceptable but optional.
- Text halos: `shell.css:850-852` title text-shadow is black (OK, depth only); `index.html:315` black (OK).
The newer code (glass chips, fields, toggles, menus, toasts, tooltips, the deleted UpdateButton) is the clean reference; the older mockup-port panels in `shell.css:1446-2060` are where the violations live.

---

## 3. Texture / "rough" feel

- Noise: `body` declares an SVG `feTurbulence` grain (`shell.css:56`, `baseFrequency .8, numOctaves 3, opacity .03`), but line 57's `background:` shorthand overrides it, so the grain never renders. The "raw" feel does not come from noise.
- What actually makes it feel rough/raw:
  - Heavy, wide-tracked, uppercase 900-weight Outfit on almost every label (LAUNCH GAME, SETUPS, SERVERS, buttons).
  - The hero picture is deliberately un-blurred with hard voxel edges, graded `filter: saturate(1.22) contrast(1.16)` (`shell.css:761-813`), and darkened only where text sits (three neutral scrims, `shell.css:927-977`: "darken where the type is, nowhere else").
  - Deep, slightly purple blacks with stacked low-alpha colour pools (red/violet/sky), so edges of panels read as light changing rather than lines.
  - Saturated 3-stop diagonal gradients (light top-left to dark bottom-right) on primary pills, like stamped metal.
  - Brushed-steel grey gradient for a sleeping primary (`dormant.css:76-90`: `#4a4a52 → #34343b → #25252b`).
- Glass (frosted): `backdrop-filter` with high saturation (180-200%) so the page shows through as colour, not shapes. Used on: nav chips, window controls, version pill, settings gear, tiles, modal cards, popovers, tooltips.
  - Popovers/tooltips are one clipped outline (card + pointer) via `clip-path: shape(...)` on a `::after`; shadow from a separate `::before` (`src/styles/popover.css:28-147`, `src/styles/tooltip.css:88-110`). Fallback without `shape()` = plain rounded glass (`popover.css:149-164`).
  - Rule: glass never moves (transform under backdrop-filter smears in WebView2), so glass fades and changes fill only (`motion.css:57-73`).
- SVG filters: a skin outline filter for the 3D character's keyboard focus (`.skin-outline-defs`, `shell.css:2538`), app-specific.
- Background images: `/hero-backdrop.webp` (hero + splash), wordmark PNG used as a mask for the splash glint (`index.html:209-243`).

---

## 4. Layout & responsiveness

- Window (`src-tauri/tauri.conf.json:13-29`): 1190x745 default and minimum, `decorations: false` (custom title bar), `visible: false` until ready, `center`, `preventOverflow`, `theme: Dark`, `dragDropEnabled: false`. macOS: decorations on + overlay title bar, traffic lights kept, own controls hidden (`src/styles/platform.css:22-43`).
- Window memory: `tauri-plugin-window-state` with `SIZE|POSITION|MAXIMIZED` only; off-screen saved position falls back to `center()` (`docs/superpowers/specs/2026-07-25-smart-window-sizing-design.md`). Older spec numbers (1160x715) are stale.
- DPI / zoom (`src-tauri/src/window_zoom.rs:1-40`): the page is only laid out between 1190x745 and ~1600x852 logical px. Smaller work areas (1366x768 laptop, 125% scaling) lower the min to the work area and zoom the webview down; windows larger than ~1600x852 are zoomed up (a maximized 1080p window = a 1641x882 page). Webview zoom, not CSS `zoom`, so `vw`/media queries/pointer coords stay honest. Worth copying.
- Shell grid (`shell.css:113-161`): `grid-template-columns: 72px minmax(410px, 1fr) 365px; grid-template-rows: 60px 1fr;` full window both axes, `min-height: 620px`. Header spans all columns (transparent, logo left, round chip buttons + window-controls pill right). Left = 72px icon rail (44px rounded-square nav chips, social links at bottom, settings last). Middle = sunken main column, padding 24, gap 24, content capped 1400px and centred. Right = rail of cards (account, servers, ad).
- Breakpoints: `min-width:1600px` right rail 401px (`:377`), `max-width:1080px` rail 68 / right 280 / body min 700 (`:383-391`), `max-height:720px` min-height 560 and tighter hero (`:393, :1436`), `max-height:800px` compact server rows (`:1813`).
- Full-screen rule (`docs/superpowers/specs/2026-08-02-full-screen-responsive-layout-design.md`): shell always fills the window; only content caps. Widths chosen from Steam survey data; verified by screenshots at 1160x715 → 3840x2160.
- Fixed-size controls never reflow: LAUNCH is exactly 300x56 in all five states, crossfaded layers, fixed grid columns for metrics, a Playwright test samples the box every 100ms (`shell.css:979-997`).
- Modal height fixed per window (`clamp(floor, 56vh, 620px)`), list absorbs changes; list shows 3.5 rows so it obviously scrolls (`AccountsModal.css:21-64, 79-116`).
- Grids: square tiles with `scrollbar-gutter: stable` so nothing shifts when the scrollbar appears (`src/styles/profiles.css:4-22`).
- Undecorated resize: invisible 8px edge/corner hit zones `.resize-handle-*` (`shell.css:2068-2140`).

---

## 5. Component patterns

| Component | Size / radius | Rest | Hover / active / selected / disabled | Ref |
|---|---|---|---|---|
| Hero primary (LAUNCH) | 300x56, r12 | crimson 135deg 3-stop, 17px/900/.14em uppercase, play icon | hover brightness 1.1 + lift 1px; press scale .97; preparing `#120507` track with crimson fill growing left→right (`background-size` pinned to 300x56 so the ramp stays put; width transition 220ms linear); running green; stopping stone grey; 44px stop segment on right separated by a 1px white .18 hairline | `shell.css:1023-1315` |
| First-run CTA pill | h52, r999, pad 0 30 0 26 | crimson gradient, 13px/900/.14em, `inset 0 1px 0 rgba(255,255,255,.14)` top highlight | brightness 1.1 + lift; press .97 | `firstrun.css:31-72` |
| Dialog button (secondary) | h40, r12, pad 0 16 | `rgba(255,255,255,.07)`, 11px/800/.08em uppercase, icon .7 white | hover/focus `.13`; press translateY 1px; disabled .45 | `CreateSetupModal.css:183-226` |
| Dialog button (primary) | same | bright crimson `#ff3856→#ee1f39→#c81028` | hover lighter stops; disabled = grey fill, text .42 | `CreateSetupModal.css:231-258` |
| Footer pair | 2-col grid, gap 10, pad 13 12, r14 | glass .07 / crimson primary, 11px/900/.12em | same | `SetupSettings.css:992-1035` |
| Danger confirm | h34, r10 | `rgba(239,68,68,.9)`; "Keep" = white .10 | hover `#ef4444` / `.16` | `SetupPage.css:483-520` |
| Ghost / icon chip | 36 round (header), 44 r12 (rail), 28 round (window ctrl) | `--glass-chip` + blur, icon `#71717a`/`#8c8c99` | hover fill .16-.18 + icon white; close hover solid `#ef4444`; active rail = red tint + red icon + 4px red bar on rail edge | `shell.css:424-627` |
| Glass pill (version) | h56, r12, pad 0 24 | white .10 + chip blur, no border, 17px/900 | hover .18, press .24 (no movement) | `shell.css:1335-1423` |
| Text field | shape per place (capsule or r12) | `rgba(255,255,255,.04)`, 12.5px white, placeholder .35, no border ever | focus = fill .07, nothing drawn; round 20px clear button .10 → .18 | `src/styles/field.css` |
| Toggle | track 40x24 r999, thumb 18 | track white .12, thumb white + small black shadow | on = crimson gradient track, thumb slides 16px (180ms ease-in-out); keyboard focus = track .20 / brightness 1.15; disabled .5, checked-disabled grey | `SetupSettings.css:688-751` |
| Tabs (segmented) | rail r999, tab h34 pad 0 16 | tab text white .6, 12px/800/.08em uppercase | sliding crimson capsule behind active (transform+width 220ms ease-in-out); hover/active text white | `shell.css:2569-2692` |
| Modal / dialog | max-w 512, r20, pad 24, gap 18 | glass gradient `rgba(24,24,34,.45)→rgba(10,10,16,.52)` + `blur(24px) saturate(180%)`, shadow `0 30px 80px rgba(0,0,0,.55)`, inset top highlight; scrim `rgba(0,0,0,.25)` + `blur(10px)` | fades only (backdrop 180ms, card 220ms); exits 180ms ease-in then unmount; window controls stay above scrim inside a solid "notch" | `AccountsModal.css:66-145`, `shell.css:458-528` |
| Danger modal | — | `--glass-card-danger` (red-tinted glass) | — | `theme.css:51`, `storage/DeleteEverythingModal.css` |
| Toast | w min(380, 100vw-40), r16, min-h 64, top 76 right 20, newest on top | dark violet card `#22222e→#16161e` washed from left in the kind's colour (`linear-gradient(100deg, rgb(tone/.3), rgb(tone/.1) 42%, transparent 78%)`), 40px r12 badge filled with the kind's 3-stop gradient, title in tone colour 14/700, body 12.5; black shadow only | in `mz-toast-in` 320ms ease-out, out 180ms ease-in; action button = tone gradient | `SetupBody.css:2275-2420` |
| Inline notice | r12, pad 10 10 10 14 | tinted fill: good `rgba(16,185,129,.12)`/`#a7f3d0`, error `rgba(239,68,68,.12)`/`#fecaca` | rise-in 6px | `SetupBody.css:387-470` |
| Progress (splash) | track 3px r999 white .12 | fill `linear-gradient(90deg,#ef4444,#ff6a3d 60%,#ffc46b)`, `background-size` pinned to track (red early, gold near done), width 520ms ease-out | has a glow + pulsing dot (violation, drop it) | `index.html:245-301` |
| List row (account) | h60, r14, pad 10 14, gap 8 between rows | white .06 | hover .12 + lift 1px; active red horizontal gradient + 3px red left bar | `AccountRow.css:4-50` |
| Server row (old style) | min-h 64, r11, 38px icon | dark gradient + 1px border (violation) | hover: an overlay fades in (95% dark + blur) with action buttons, row content blurs to .08 opacity | `shell.css:1786-2045` |
| Tile (square) | 140px default, r18, `container-type: size` | `--glass-chip` + blur, picture r12 at 86% | hover/keyboard focus `--glass-chip-hover`; badge top-right 20px | `src/styles/profiles.css:24-100` |
| Context menu | w240, pad 6, r14, gap 2 | frosted pointed glass (popover.css), black shadow `0 18px 44px rgba(0,0,0,.5)` | item h38 r9 pad 0 12, 13px/600 white .85, icon .5; hover/focus white .08; danger `#f87171`, hover red .14; divider 1px white .07; disabled .45; inline success note green tint | `SetupPage.css:367-470`, `popover.css` |
| Confirm popover | pad 14, r14 | same pointed glass; title 13/800, body 12/500 .62, 2-col action grid | — | `SetupBody.css:1025-1065` |
| Tooltip | max-w min(320, 100vw-16), pad 7 11 8 | one-piece frosted glass incl. arrow (`--glass-popover-raised`), 12px/600, no outline, no accent; card variant with heading/subheading/rule/dl rows (tabular nums) | fades 180ms, no travel | `src/styles/tooltip.css` |
| Scrollbar | 4px, thumb `#27272a` r4, track transparent, no arrows | hover `#3f3f46`; on glass, thumb white .16 → .28 | applied globally, not only `.custom-scroll` | `shell.css:70-111`, `AccountsModal.css:269-280` |
| Update button (deleted 2026-10-31, still a good pattern) | h36 r999 pad 0 16 0 14, 13px/800 | `--glass-chip` + blur, arrow-up icon | tone `ready` = `rgba(225,29,72,.2)` + brand-400 icon, hover .32; `required` = solid brand-500 → brand-600; keyboard focus = hover fill; "no line around it in any state, no pulsing ring (owner 2026-10-19)" | `git show c8f5147b^:src/components/shell/UpdateButton.css` |
| Splash | full window, z 9999 | inline in `index.html` (paints on first frame, before React): drifting backdrop (24s), red/violet veil that lifts as progress grows (`opacity: calc(1 - var(--p)*.42)`), vignette, 440px wordmark with a masked glint sweep, 360px meter, uppercase status + tabular % at .2em tracking, version bottom, own drag strip + window controls; leaves by "walking in" (world scale 1.14, stage rises, fade 220ms) | `index.html:17-508`, `src/splash/` |
| Empty/skeleton | — | `.hero-blank` rounded bars `linear-gradient(90deg, white .16, white .09)` where text will go | — | `firstrun.css:133-158` |

---

## 6. Icons
- Primary: Font Awesome 7 (solid, regular, brands) via `@fortawesome/react-fontawesome` and a local wrapper `src/lib/FontAwesomeIcon.tsx`; FA stylesheet imported first (`src/main.tsx:8`). Filled glyphs, sized by font-size (rail 16px, buttons 9-14px).
- Secondary: `lucide-react` (stroke). Normalised by `.mz-icon` (`src/styles/icon.css:20-30`): `width/height: round(1.2em, 2px)`, `stroke-width: 1.35px` fixed with `vector-effect: non-scaling-stroke`, so lucide sits at FA's visual size. In the rail lucide gets 18px/2px stroke to match solid FA weight (`shell.css:595-599`). Toast badge icons 20px stroke 2.4.
- Social icons: monochrome `#71717a`, brand colour on hover (`shell.css:254-299`).
- Recommendation for the new app: pick one (lucide alone with the `.mz-icon` rule is simplest).

## 7. Typography
- Latin: Outfit 300-900 (`@fontsource/outfit`, `src/main.tsx:9-15`). Mono: JetBrains Mono 400/700/800 for figures/metrics/file names.
- Khmer: Kantumruy Pro registered as `"Mazer Khmer"` restricted by `unicode-range: U+1780-17FF, U+19E0-19FF, U+200B-200D, U+25CC`, `size-adjust: 108%`, weights 300-700 (700 file serves 700-900) (`src/styles/khmer.css:26-76`). Every stack is `"Outfit", "Mazer Khmer", …`, so English/digits in Khmer text are always drawn by Outfit and Khmer letters only by Kantumruy.
- Khmer mode (`html:lang(km)`): all letter-spacing forced to normal (tracked uppercase would split clusters) except elements marked `lang="en"` (wordmark, codes); line-height 1.5 / 1.55 (`khmer.css:91-107`).
- Numerals: Western digits in both languages via `-u-nu-latn` in every Intl formatter (`src/i18n/format.ts:16-31`); `tabular-nums` on changing numbers.
- Fonts bundled locally (CSP `font-src 'self' data:`), splash text waits for Outfit (`.fonts-ready`) to avoid a font swap.

## 8. Motion
- Tokens above; rules in `docs/motion/motion-guide.md`: only transform/opacity move; enter ease-out, exit ease-in and shorter (always 180ms); nobody waits for motion; nothing loops except work indicators; list properties, never `transition: all`; reuse `mz-*` keyframes (`motion.css:75-126`: view-in, dialog-in/out, pop-in/out, rise-in, fade-in/out, all opacity-only except rise-in 6px).
- Expand/collapse with `grid-template-rows: 0fr → 1fr` (`shell.css:642-658`, account panel `:1645-1662`).
- Progress widths are `linear` (eased bars look like stutter).
- Reduced motion: in-app setting writes `data-motion="reduce"` on `<html>`, plus the OS media query; both set all animation/transition durations to .01ms, except `.fa-spin`, `.animate-spin`, `[data-motion="essential"]` (spinners keep turning) (`motion.css:128-160`).

## 9. i18n / a11y
- Languages en + km; initial guess = Khmer if region KH, timezone `Asia/Phnom_Penh`, or a `km`/`-KH` browser language (`src/i18n/languages.ts:46-57`); fallback en; `<html lang>` updated.
- Focus without rings for pointer users: `installInputModality()` writes `data-input="pointer|keyboard"`; only Tab/arrows/Home/End/Page keys switch to keyboard, Escape/Enter/typing do not (`src/lib/inputModality.ts`). One global ring only in keyboard mode: `outline: 2px solid rgba(255,255,255,.85); outline-offset: 2px` (inside at -3px where clipped); text fields never get a ring (fill brightens instead); switch ring on track, slider ring on thumb (`src/styles/focus.css`). Component focus styles use `:focus-visible:where([data-input="keyboard"] *)`. Per CLAUDE.md rule 8 this white keyboard ring is the one ring that exists; ask the owner before copying it (or replace with fill-only focus like menu items/buttons already do).
- Focus trap with Tab cycling (`installTabCycle`, `src/lib/useFocusTrap`), outside-press closes dialogs (`installOutsideClose`), file drop guard (`src/main.tsx:34-42`).
- Progress a11y: LAUNCH becomes `role="progressbar"` with `aria-valuetext`, `aria-live="polite"` only on stage change, Escape never stops, stop is a separate labelled button (launch spec, `docs/superpowers/specs/2026-08-02-launch-button-progress-design.md` §Accessibility).
- `forced-colors` handled for decorative plates (`shell.css:2525`).
- `user-select: none` on body; splash text not selectable.

## 10. UX flows worth copying
- Startup: inline HTML splash that paints on frame one, real staged progress (`--p`), window hidden until ready, then "walk into" the app (`index.html`, `src/splash/bootSequence.ts`).
- First run in two steps (`docs/superpowers/specs/2026-09-24-first-run-two-steps-design.md`): step 1 the middle column is one red pill + one line; step 2 the app is present but "asleep" (silver greyscale), with blanks where content will appear, and one pill that fills it. Pressing a locked thing never opens a dialog; it focuses the pill (cue). The way forward is always the brightest thing on screen.
- Dormant, not walled: gated elements stay readable and pressable, pressing explains why (`dormant.css:4-35`); colour sweeps in with a stagger on unlock.
- Updates: download + stage silently in the background; restart by itself at a harmless moment (after the game quits, minimized/tray a minute, or not in front for 10 min); never under active work (commit `c8f5147b`, `docs/install-uninstall-ux.md:113`). Updates that need the user live in Settings > About with one status line + one button (`src/components/shell/settings/AboutTab.tsx:46-120`). Earlier variant: a header pill that appears only when a click can restart immediately; a small pointed panel only if the restart is refused.
- Errors: under the primary button, without moving it (`shell.css:1005-1021`); toasts top-right with one next action; inline failed card with Retry (`shell.css:1578-1610`).
- Empty states: centred pill + one-line caption; skeleton bars for missing text.
- Confirmations: small pointed glass popover beside the bin (title, body, Keep / Delete grid) instead of a modal; big destructive actions get a red-tinted danger modal with the real size shown.
- Fixed-geometry rule: controls and dialogs never change size while their state changes.
- Installer: per-user, no UAC, terms before install, update keeps data (`docs/install-uninstall-ux.md`).

---

## 11. Design DNA (what makes it distinctive)
1. Near-black, faintly violet surfaces (`#050506`, `#131318`, `#070709`) lit by stacked low-alpha radial pools of red, violet and sky, never flat.
2. One hot accent: crimson in a 135deg three-stop gradient (`#e11d48 → #be123c → #881337`), reserved for the single primary action and "selected".
3. Semantic colours reuse the same 3-stop diagonal shape (green, blue, amber, crimson).
4. Heavy uppercase type: Outfit 800-900 with .08-.14em tracking for labels and actions; quiet 12-13px 500 body.
5. Separation by fill, blur and black shadow, not lines: white-alpha chips (.04-.24), dark gradient cards, inset 1px top highlight.
6. Frosted glass (blur 22-28px, saturate 170-200%) for chips, modals, menus and tooltips; pointed popovers cut as one shape.
7. Glass never moves: it fades and changes fill; only solid gradient buttons lift 1px and press to .97.
8. State shown as fill brightness, icon colour and tinted fills; selected rows get a gradient wash plus a short colour bar.
9. A content column sunk into the shell (rounded 24px top-left, inset shadow) between a transparent icon rail and a card rail.
10. Fixed-geometry controls (300x56 LAUNCH, fixed-height modal), tabular numbers, layered crossfades, so nothing jumps.
11. Webview zoom keeps one composed layout from 1190x745 to 4K.
12. "Dormant" greyscale for locked UI, with a staggered colour sweep on unlock.
13. Khmer done properly: separate Khmer face by unicode-range, size-adjust 108%, no tracking, line-height 1.5+, Western digits.
14. One motion vocabulary (6 durations, 4 curves), with a reduce-motion switch in the app.
15. Hand-written comments explain every visual decision with measurements; worth keeping as a habit.

## Do NOT copy
- Every glow listed in section 2 (nav marker glow, LAUNCH coloured shadows, capsule shadows, radio glow, spotlight halo, CTA ring cue, splash sun/fill/dot glow, store drop-shadows, modal colour orbs).
- Every border/outline round panels, rows, buttons and fields in the older `shell.css` panels (setups panel, New Setup, servers panel/rows, account card, offline form, dashed empty cards, radio ring, unread-dot ring).
- The reduced-motion pink outline fallback on the CTA cue (`firstrun.css:161-172`).
- The dead noise `background-image` on body (`shell.css:56`); if grain is wanted, put it on a separate pseudo-element.
- Two icon libraries at once; pick one.
- Literal colours scattered instead of tokens; ad-hoc z-index values; Tailwind utilities that are silently overridden by unlayered CSS (`shell.css:4-14`).
- Minecraft-specific bits (voxel backdrop, skin viewer, ping bars, ad rail).
- The global white keyboard ring is a ring; confirm with the owner before reusing it.

---

## Screenshots viewed
- `tests/visual/app-shell.spec.ts-snapshots/app-shell-default-win32.png` (1280x720): transparent header with pixel-art red wordmark, round glass chips (support, bell) and a pill of window controls; left rail of rounded-square glass chips with the active Play chip tinted red; sunken main column holding the hero card (sunset Angkor-in-voxels backdrop, huge white "Survival" title, translucent "FABRIC 1.20.4" glass pill, crimson LAUNCH GAME, glass gear) and a SETUPS panel with a glass square tile; right rail with account card, SERVERS card, ad box. Visible thin borders on the setups panel, account and server cards (the violations).
- `tests/visual/accounts-modal.spec.ts-snapshots/accounts-modal-win32.png`: frosted card over a blurred page, red tinted corner light (orb), title chip with red icon, rows as white-alpha fills; active row red gradient wash + red bar on the left, radio dot with red glow; two wide uppercase footer buttons in grey glass.
- `tests/visual/onboarding.spec.ts-snapshots/first-run-dormant-win32.png`: empty dark middle with one crimson `+ ADD AN ACCOUNT` pill and a grey two-line caption; rail icons dimmed; account card with red border + glow ("No account yet") (violation); news card in violet gradient.
- `test-results/legal-about-km-1280.png`: Settings as a large frosted dialog over the app, Khmer breadcrumb title, left list of sections with icon + Khmer label, active section red-tinted fill; right a rounded panel of label/value rows (muted labels left, bold white values right); round glass close button. Khmer and Latin mix cleanly at matching visual size.

## Side note for the owner (CLAUDE.md rule 5)
`git log` shows commits in this repo authored as `MazerDev <mazerclub@gmail.com>` (e.g. `c8f5147b`) and with `Co-Authored-By: Claude` lines. Not changed (read-only task, and history must not be rewritten); flagged only.
