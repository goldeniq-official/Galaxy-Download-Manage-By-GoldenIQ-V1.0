// SPDX-License-Identifier: RPL-1.5
// G6 UI checks on the real React UI (mock backend): every screen x km/en x dark/light.
//  - owner rule 8: no border/outline/ring/glow (design/check.cjs probes, shared with the P0 mockup check)
//  - X-03: Khmer ink never clipped (pixel probe), no horizontal clipping without a tooltip, nothing offscreen
//  - X-05: axe-core 0 serious/critical; keyboard focus visible by fill (D6)
//  - X-01: 30 XSS file names never execute; C-03: Enter within 400 ms never confirms Download Info
import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";

const require = createRequire(import.meta.url);
const P = require("../design/check.cjs");
const SHOTS = path.join("docs", "evidence", "g6-shots");
fs.mkdirSync(SHOTS, { recursive: true });

const SCREENS: Record<string, { q: string; w: number; h: number; then?: (p: Page) => Promise<void> }> = {
  list: { q: "scene=states", w: 1100, h: 700 },
  details: { q: "scene=states", w: 1280, h: 760, then: async (p) => p.locator(".row").first().click() },
  info: { q: "scene=info", w: 1100, h: 700 },
  batch: { q: "scene=batch", w: 1100, h: 700 },
  settings: { q: "scene=states", w: 1100, h: 700, then: async (p) => p.locator(".rail .nav").last().click() },
  onboarding1: { q: "scene=empty&onboarded=0", w: 1100, h: 700 },
  onboarding3: { q: "scene=empty&onboarded=0", w: 1100, h: 700, then: async (p) => { await p.locator(".btn-p").click(); await p.locator(".btn-p").click(); } },
  empty: { q: "scene=empty", w: 1100, h: 700 },
  update: { q: "scene=states&update=ready", w: 1100, h: 700 },
  compact: { q: "scene=states", w: 600, h: 600 },
  mini: { q: "scene=states", w: 380, h: 200 },
};

async function open(page: Page, q: string, w: number, h: number) {
  await page.setViewportSize({ width: w, height: h });
  await page.goto(`/?${q}`);
  await page.waitForSelector("#shell");
  await page.evaluate(() => document.fonts.ready);
  await page.waitForTimeout(700); // first tick + 400 ms Info arming
}

for (const [name, s] of Object.entries(SCREENS))
  for (const lang of ["km", "en"])
    for (const theme of ["dark", "light"])
      test(`${name} ${lang} ${theme}`, async ({ page, context }) => {
        await open(page, `${s.q}&lang=${lang}&theme=${theme}`, s.w, s.h);
        if (s.then) {
          await s.then(page);
          await page.waitForTimeout(400);
        }
        await page.mouse.move(2, 2);
        await page.screenshot({ path: path.join(SHOTS, `${name}-${lang}-${theme}.png`) });
        const rules: string[] = await page.evaluate(P.rulesProbe);
        expect(rules, "rule 8: no border/outline/ring/glow").toEqual([]);
        const lay = await page.evaluate(P.layoutProbe);
        expect(lay.hClip, "horizontal clipping without a tooltip").toEqual([]);
        expect(lay.offscreen, "text offscreen").toEqual([]);
        const diff = await context.newPage();
        const v = await P.clipRun(page, diff);
        expect(v, "Khmer/Latin ink clipped").toEqual([]);
        if (lang === "km") {
          const f = await P.fontRun(page);
          expect(f.fonts, "bundled Khmer font in use").toContain("Kantumruy Pro");
        }
        const axe = await new AxeBuilder({ page }).analyze();
        const serious = axe.violations.filter((x) => x.impact === "serious" || x.impact === "critical");
        expect(serious.map((x) => `${x.id}: ${x.nodes.map((n) => n.target.join(" ")).slice(0, 3).join(" | ")}`), "axe serious/critical").toEqual([]);
        if (theme === "dark" && s.w >= 1100) {
          const focus = await P.focusRun(page, diff);
          const bad = focus.filter((f: { ok: string }) => f.ok.startsWith("FAIL"));
          expect(bad.map((f: { label: string; px3: number; need: number; ok: string }) => `${f.label} ${f.px3}/${f.need} ${f.ok}`), "keyboard focus visible by fill (D6)").toEqual([]);
        }
      });

test("X-01: 30 XSS file names render as text and never run", async ({ page }) => {
  await open(page, "scene=xss", 1280, 760);
  for (const r of await page.locator(".row").all()) await r.click();
  await page.locator(".row").first().click();
  await page.waitForTimeout(300);
  expect(await page.evaluate(() => (window as unknown as { __xss?: number }).__xss)).toBeUndefined();
  expect(await page.locator("#root img[src='x'], #root iframe, #root svg[onload], #root script").count()).toBe(0);
  await expect(page.locator(".row").first()).toContainText("<img src=x onerror=");
});

test("C-03: Enter right after Download Info opens never confirms; Start works after 400 ms", async ({ page }) => {
  await page.setViewportSize({ width: 1100, height: 700 });
  await page.goto("/?scene=info");
  await page.waitForSelector(".modal");
  await page.locator(".modal .btn-p").focus();
  await page.keyboard.press("Enter");
  const early = await page.evaluate(() => (window as unknown as { __calls: [string][] }).__calls.filter((c) => c[0] === "info_confirm").length);
  expect(early).toBe(0);
  await page.waitForTimeout(500);
  await page.locator(".modal .btn-p").click();
  const later = await page.evaluate(() => (window as unknown as { __calls: [string][] }).__calls.filter((c) => c[0] === "info_confirm").length);
  expect(later).toBe(1);
  await expect(page.locator(".modal")).toHaveCount(0);
  await expect(page.locator(".list")).toBeVisible();
});

test("C-03: five downloads at once = one dialog with five rows", async ({ page }) => {
  await open(page, "scene=batch", 1100, 700);
  await expect(page.locator(".modal")).toHaveCount(1);
  await expect(page.locator(".modal .check")).toHaveCount(5);
});

// owner 2026-11-13: nothing may push the layout. Notices go to the fixed footer, dialogs hang from a fixed top,
// a dialog error has a reserved line; the list and the dialog title must not move.
test("a download can be removed from its right-click menu, and all completed ones at once (IDM Delete / Delete completed)", async ({ page }) => {
  await open(page, "scene=states&lang=en", 1100, 700);
  const done = await page.evaluate(() => document.querySelectorAll(".row .st.done").length);
  await page.locator(".row").nth(3).click({ button: "right" });
  await page.locator(".menu .mi", { hasText: /^Remove from list$/ }).click();
  await expect.poll(async () => (await calls(page, "job_remove")).length).toBe(1);
  await page.locator(".row").nth(3).click({ button: "right" });
  await page.locator(".menu .mi", { hasText: "Remove all completed" }).click();
  await expect.poll(async () => (await calls(page, "job_remove")).length).toBeGreaterThan(1);
  expect(done).toBeGreaterThan(0);
});

test("no layout shift: a dialog error and an update notice move nothing", async ({ page }) => {
  await open(page, "scene=states&lang=en", 1100, 700);
  await page.locator(".top .btn-p").click();
  const title = page.locator(".modal h2");
  const btn = page.locator(".modal .btn-p");
  const before = [await title.boundingBox(), await btn.boundingBox()];
  await page.locator(".modal input").fill("ftp://x");
  await page.keyboard.press("Enter");
  await expect(page.locator(".modal .err-line")).toContainText("valid link");
  expect([await title.boundingBox(), await btn.boundingBox()]).toEqual(before);
  await open(page, "scene=states&lang=en&update=ready", 1100, 700);
  const list = await page.locator(".list").boundingBox();
  await open(page, "scene=states&lang=en", 1100, 700);
  expect(await page.locator(".list").boundingBox()).toEqual(list);
});

test("a YouTube page link added by hand opens the quality list instead of downloading the HTML page", async ({ page }) => {
  await open(page, "scene=states&lang=en", 1100, 700);
  await page.locator(".top .btn-p").click();
  await page.locator(".modal input").fill("https://www.youtube.com/watch?v=abc");
  await page.keyboard.press("Enter");
  await expect(page.locator(".modal h2")).toHaveText("Add video");
  await expect.poll(async () => (await calls(page, "media_probe")).length).toBe(1);
  expect(await calls(page, "add_url")).toHaveLength(0);
});

test("settings is one dialog: Esc closes it, the section rows switch inside a fixed box", async ({ page }) => {
  await open(page, "scene=states&lang=en", 1100, 700);
  await mainNav(page).last().click();
  const box = await page.locator(".setdlg").boundingBox();
  await page.locator(".setnav .nav").nth(1).click();
  await expect(page.locator(".sethead h2")).toHaveText("Downloads");
  expect(await page.locator(".setdlg").boundingBox()).toEqual(box);
  await page.keyboard.press("Escape");
  await expect(page.locator(".setdlg")).toHaveCount(0);
  await expect(page.locator("#shell")).not.toHaveAttribute("inert", "");
});

test("X-05: keyboard list: arrows select, Delete removes, Ctrl+N opens Add", async ({ page }) => {
  await open(page, "scene=states", 1100, 700);
  await page.locator(".row").first().focus();
  await page.keyboard.press("ArrowDown");
  await expect(page.locator(".row.is-selected")).toHaveCount(1);
  await expect(page.locator(".row").nth(1)).toHaveAttribute("aria-selected", "true");
  await page.keyboard.press("Delete");
  const del = await page.evaluate(() => (window as unknown as { __calls: [string][] }).__calls.filter((c) => c[0] === "job_remove").length);
  expect(del).toBe(1);
  await page.keyboard.press("Control+n");
  await expect(page.locator(".modal")).toHaveCount(1);
});

test("D-03 / P1 #20: 10 000 rows stay virtual and search answers fast", async ({ page }) => {
  await open(page, "scene=big", 1100, 700);
  expect(await page.locator(".row").count()).toBeLessThan(60);
  const t0 = Date.now();
  await page.locator(".search input").fill("គណិតវិទ្យា");
  await expect(page.locator(".foot")).toContainText("104");
  expect(Date.now() - t0).toBeLessThan(1000);
});

// X-03/X-04 + exit.md R4/R5: Windows scaling 125/150 % (deviceScaleFactor) and high contrast (forced-colors)
for (const name of ["list", "info", "settings", "onboarding1"])
  for (const dpr of [1.25, 1.5, 2])
    for (const lang of ["km", "en"])
      test(`dpi ${dpr * 100}% ${name} ${lang}`, async ({ browser }) => {
        const s = SCREENS[name];
        const ctx = await browser.newContext({ viewport: { width: s.w, height: s.h }, deviceScaleFactor: dpr, reducedMotion: "reduce", baseURL: "http://127.0.0.1:1431" });
        const page = await ctx.newPage();
        await open(page, `${s.q}&lang=${lang}&theme=dark`, s.w, s.h);
        if (s.then) {
          await s.then(page);
          await page.waitForTimeout(400);
        }
        await page.mouse.move(2, 2);
        if (lang === "km") await page.screenshot({ path: path.join(SHOTS, `dpi${dpr * 100}-${name}-${lang}.png`) });
        expect(await page.evaluate(P.rulesProbe), "rule 8").toEqual([]);
        const lay = await page.evaluate(P.layoutProbe);
        expect([...lay.hClip, ...lay.offscreen], "layout at this scale").toEqual([]);
        expect(await P.clipRun(page, await ctx.newPage()), "Khmer/Latin ink clipped at this scale").toEqual([]);
        await ctx.close();
      });

for (const name of ["list", "info", "settings"])
  for (const lang of ["km", "en"])
    test(`high contrast ${name} ${lang}`, async ({ page, context }) => {
      const s = SCREENS[name];
      await page.emulateMedia({ forcedColors: "active" });
      await open(page, `${s.q}&lang=${lang}&theme=dark`, s.w, s.h);
      if (s.then) {
        await s.then(page);
        await page.waitForTimeout(400);
      }
      await page.mouse.move(2, 2);
      if (lang === "en") await page.screenshot({ path: path.join(SHOTS, `hc-${name}-${lang}.png`) });
      expect(await page.evaluate(P.rulesProbe, true), "rule 8 (system borders allowed in HC only, owner Q1)").toEqual([]);
      const focus = await P.focusRun(page, await context.newPage());
      const bad = focus.filter((f: { ok: string }) => f.ok.startsWith("FAIL"));
      expect(bad.map((f: { label: string; ok: string }) => `${f.label} ${f.ok}`), "focus visible in high contrast").toEqual([]);
    });

// ---------- G7: queues + scheduler, Tasks (batch, import, export, video, drop target), refresh address, settings ----------
// Same probes as above on every new screen (km/en x dark/light); Khmer screenshots go to docs/evidence/g7-shots.
const G7SHOTS = path.join("docs", "evidence", "g7-shots");
fs.mkdirSync(G7SHOTS, { recursive: true });
const mainNav = (p: Page) => p.locator(".body > nav.rail > .nav");
const tasks = async (p: Page, i: number) => {
  await p.locator(".top .tb[aria-haspopup]").last().click();
  await p.locator(".menu .mi").nth(i).click();
};
const setSec = (i: number, more?: (p: Page) => Promise<void>) => async (p: Page) => {
  await mainNav(p).last().click();
  await p.locator(".setnav .nav").nth(i).click();
  if (more) await more(p);
};
const calls = (p: Page, cmd: string) =>
  p.evaluate((c) => ((window as unknown as { __calls?: [string, Record<string, unknown>][] }).__calls ?? []).filter((x) => x[0] === c).map((x) => x[1]), cmd);

const G7: Record<string, { q: string; w: number; h: number; then?: (p: Page) => Promise<void> }> = {
  queues: { q: "scene=states", w: 1280, h: 760, then: async (p) => mainNav(p).nth(10).click() },
  rowmenu: { q: "scene=states", w: 1100, h: 700, then: async (p) => p.locator(".row").nth(6).click({ button: "right" }) },
  tasks: { q: "scene=states", w: 1100, h: 700, then: async (p) => p.locator(".top .tb[aria-haspopup]").last().click() },
  qmenu: { q: "scene=states", w: 1100, h: 700, then: async (p) => p.locator(".top .tb[aria-haspopup]").first().click() },
  batch: {
    q: "scene=states", w: 1100, h: 700,
    then: async (p) => {
      await tasks(p, 0);
      await p.locator(".modal input").nth(0).fill("https://cdn.example.org/photos/img-*.jpg");
      await p.locator(".modal input").nth(2).fill("20000");
      await p.locator(".modal input").nth(3).fill("5");
      await p.waitForSelector(".modal .check");
    },
  },
  import: { q: "scene=states", w: 1100, h: 700, then: async (p) => tasks(p, 2) },
  refresh: {
    q: "scene=states", w: 1100, h: 700,
    then: async (p) => {
      await p.locator(".row").nth(10).click({ button: "right" });
      await p.locator(".menu .mi", { hasText: /Refresh download address|យកអាសយដ្ឋានទាញយកថ្មី/ }).click();
      await p.waitForSelector(".modal .btn-p", { timeout: 5000 });
    },
  },
  video: {
    q: "scene=states", w: 1100, h: 700,
    then: async (p) => {
      await tasks(p, 1);
      await p.locator(".modal input").fill("https://video.example.org/watch?v=abc");
      await p.locator(".modal .fld-row .btn").click();
      await p.waitForSelector(".modal [role=radio]");
    },
  },
  countdown: { q: "scene=states&countdown=1", w: 1100, h: 700, then: async (p) => void (await p.waitForSelector(".cd")) },
  "set-connection": { q: "scene=states", w: 1100, h: 700, then: setSec(2) },
  "set-downloads": { q: "scene=states", w: 1100, h: 700, then: setSec(1) },
  "set-queues": { q: "scene=states", w: 1100, h: 700, then: setSec(3) },
  "set-queue-files": { q: "scene=states", w: 1100, h: 700, then: setSec(3, async (p) => p.locator(".sbody .tabs .tab").nth(1).click()) },
  "set-proxy": {
    q: "scene=states", w: 1100, h: 700,
    then: setSec(4, async (p) => {
      await p.locator(".sbody .tabs").first().locator(".tab").nth(2).click();
      await p.locator(".sbody .tabs").nth(1).locator(".tab").nth(1).click();
    }),
  },
  "set-logins": { q: "scene=states", w: 1100, h: 700, then: setSec(5) },
  "set-sounds": { q: "scene=states", w: 1100, h: 700, then: setSec(6) },
  "set-video": { q: "scene=states&comp=0", w: 1100, h: 700, then: setSec(7) },
  "set-about": { q: "scene=states", w: 1100, h: 700, then: async (p) => { await mainNav(p).last().click(); await p.locator(".setnav .nav").last().click(); } },
};

test("G8 §21 M1: About shows the voluntary support block and loads nothing from the network", async ({ page }) => {
  const outside: string[] = [];
  page.on("request", (r) => { if (!r.url().startsWith("http://127.0.0.1:1431/") && !r.url().startsWith("data:")) outside.push(r.url()); });
  await open(page, "scene=states&lang=en&theme=dark", 1100, 700);
  await mainNav(page).last().click();
  await page.locator(".setnav .nav").last().click();
  await expect(page.getByText("Support GDM")).toBeVisible();
  await expect(page.getByText(/unlocks no feature/)).toBeVisible();
  await expect(page.locator("img[alt=KHQR]")).toBeVisible();
  expect(outside).toEqual([]);
});

for (const [name, s] of Object.entries(G7))
  for (const lang of ["km", "en"])
    for (const theme of ["dark", "light"])
      test(`g7 ${name} ${lang} ${theme}`, async ({ page, context }) => {
        await open(page, `${s.q}&lang=${lang}&theme=${theme}`, s.w, s.h);
        if (s.then) await s.then(page);
        await page.waitForTimeout(400);
        await page.mouse.move(2, 2);
        if (lang === "km") await page.screenshot({ path: path.join(G7SHOTS, `${name}-km-${theme}.png`) });
        expect(await page.evaluate(P.rulesProbe), "rule 8: no border/outline/ring/glow").toEqual([]);
        const lay = await page.evaluate(P.layoutProbe);
        expect(lay.hClip, "horizontal clipping without a tooltip").toEqual([]);
        expect(lay.offscreen, "text offscreen").toEqual([]);
        const diff = await context.newPage();
        expect(await P.clipRun(page, diff), "Khmer/Latin ink clipped").toEqual([]);
        if (lang === "km") expect((await P.fontRun(page)).fonts, "bundled Khmer font in use").toContain("Kantumruy Pro");
        const axe = await new AxeBuilder({ page }).analyze();
        const serious = axe.violations.filter((x) => x.impact === "serious" || x.impact === "critical");
        expect(serious.map((x) => `${x.id}: ${x.nodes.map((n) => n.target.join(" ")).slice(0, 3).join(" | ")}`), "axe serious/critical").toEqual([]);
        if (theme === "dark") {
          const focus = await P.focusRun(page, diff);
          const bad = focus.filter((f: { ok: string }) => f.ok.startsWith("FAIL"));
          expect(bad.map((f: { label: string; px3: number; need: number; ok: string }) => `${f.label} ${f.px3}/${f.need} ${f.ok}`), "keyboard focus visible by fill (D6)").toEqual([]);
        }
      });

test("G7 batch: more than 10 000 needs the confirmation (I-04)", async ({ page }) => {
  await open(page, "scene=states&lang=en", 1100, 700);
  await tasks(page, 0);
  await page.locator(".modal input").nth(0).fill("https://cdn.example.org/img-*.jpg");
  await page.locator(".modal input").nth(2).fill("20000");
  await expect(page.locator(".modal .lab")).toContainText("20,000");
  const add = page.locator(".modal .btn-p");
  await expect(add).toHaveAttribute("aria-disabled", "true");
  await add.click({ force: true });
  expect(await calls(page, "batch_add")).toHaveLength(0);
  await page.locator(".modal .check").click();
  await expect(add).toHaveAttribute("aria-disabled", "false");
  await add.click();
  await expect(page.locator(".modal")).toHaveCount(0);
  const c = await calls(page, "batch_add");
  expect(c).toHaveLength(1);
  expect(c[0]).toMatchObject({ confirmed: true, queue: "main", b: { pattern: "https://cdn.example.org/img-*.jpg", from: "1", to: "20000" } });
});

test("G7 batch: a pattern without * is refused with a message", async ({ page }) => {
  await open(page, "scene=states&lang=en", 1100, 700);
  await tasks(page, 0);
  await page.locator(".modal input").nth(0).fill("https://cdn.example.org/img.jpg");
  await expect(page.locator(".modal .err-line")).toContainText("exactly one *");
});

test("G7 refresh address: Start over is the default button, Resume disabled when not strong", async ({ page }) => {
  await open(page, "scene=states&lang=en", 1100, 700);
  await page.locator(".row").nth(10).click({ button: "right" });
  await page.locator(".menu .mi", { hasText: "Refresh download address" }).click();
  expect(await calls(page, "refresh_start")).not.toHaveLength(0);
  await expect(page.locator(".modal .btn-p")).toHaveText("Start over", { timeout: 5000 });
  await expect(page.locator(".modal .btn-p")).toBeFocused();
  await expect(page.locator(".modal button", { hasText: /^Resume$/ })).toBeDisabled();
  await expect(page.locator(".modal button", { hasText: "Continue (risk)" })).toBeVisible();
  await expect(page.locator(".modal")).toContainText("not verified");
  await page.keyboard.press("Enter");
  await expect(page.locator(".modal")).toHaveCount(0);
  expect(await calls(page, "refresh_decide")).toEqual([{ id: expect.any(String), choice: "restart" }]);
});

test("G7 refresh address: a web page instead of the file says login needed", async ({ page }) => {
  await open(page, "scene=states&lang=en&refresh=html", 1100, 700);
  await page.locator(".row").nth(10).click({ button: "right" });
  await page.locator(".menu .mi", { hasText: "Refresh download address" }).click();
  await expect(page.locator(".modal .err-t")).toContainText("login needed", { timeout: 5000 });
  await expect(page.locator(".modal button", { hasText: "Start over" })).toHaveCount(0);
});

test("G7 countdown banner: Cancel calls queue_cancel_action", async ({ page }) => {
  await open(page, "scene=states&lang=en&countdown=1", 1100, 700);
  await expect(page.locator(".cd")).toContainText(/Shutting down in \d+ s/);
  await page.locator(".cd button").click();
  await expect(page.locator(".cd")).toHaveCount(0);
  expect(await calls(page, "queue_cancel_action")).toEqual([{ id: "main" }]);
});

test("countdown: it takes no focus, and Esc cancels it", async ({ page }) => {
  await open(page, "scene=states&lang=en&countdown=1", 1100, 700);
  await expect(page.locator(".cd")).toBeVisible();
  expect(await page.evaluate(() => !!document.activeElement?.closest(".cd"))).toBe(false);
  await page.keyboard.press("Escape");
  await expect(page.locator(".cd")).toHaveCount(0);
  expect(await calls(page, "queue_cancel_action")).toEqual([{ id: "main" }]);
});

test("G7 video: DRM is refused with a clear message; a page with formats lists the qualities", async ({ page }) => {
  await open(page, "scene=states&lang=en", 1100, 700);
  await tasks(page, 1);
  await page.locator(".modal input").fill("https://video.example.org/drm/movie");
  await page.keyboard.press("Enter");
  await expect(page.locator(".modal .err-line")).toContainText("protected (DRM)");
  await page.locator(".modal input").fill("https://video.example.org/watch?v=abc");
  await page.keyboard.press("Enter");
  await expect(page.locator(".modal [role=radio]")).toHaveCount(4);
  await page.locator(".modal [role=radio]").nth(1).click();
  await page.locator(".modal .btn-p").click();
  expect(await calls(page, "media_start")).toEqual([{ token: "t-1", choice: 1 }]);
});

test("G7 video: components missing => D10 notice, install, then probe again", async ({ page }) => {
  await open(page, "scene=states&lang=en&comp=0", 1100, 700);
  await tasks(page, 1);
  await page.locator(".modal input").fill("https://video.example.org/watch?v=abc");
  await page.keyboard.press("Enter");
  // the helpers set themselves up (no click); the D10 notice is checked on Settings > Video (g7 set-video)
  await expect(page.locator(".modal [role=radio]")).toHaveCount(4, { timeout: 5000 });
  expect(await calls(page, "components_install")).toHaveLength(1);
});

test("G7 import: a preview comes first, nothing is added before Add (I-03)", async ({ page }) => {
  await open(page, "scene=states&lang=en", 1100, 700);
  await tasks(page, 2);
  await expect(page.locator(".modal")).toContainText("3 downloads found");
  await expect(page.locator(".modal .imp-row")).toHaveCount(3);
  expect(await calls(page, "import_commit")).toHaveLength(0);
  await page.locator(".modal .btn-p").click();
  await expect(page.locator(".modal")).toHaveCount(0);
  expect(await calls(page, "import_commit")).toEqual([{ queue: null, restoreSettings: false }]);
});

test("G7 export: the saved file name is shown", async ({ page }) => {
  await open(page, "scene=states&lang=en", 1100, 700);
  await tasks(page, 3);
  await expect(page.locator(".toast")).toContainText("gdm-export.json");
});

test("G7 Download later puts the file in the chosen queue", async ({ page }) => {
  await open(page, "scene=info&lang=en", 1100, 700);
  await page.locator(".modal select").nth(1).selectOption("q-1");
  await page.locator(".modal .btn-s", { hasText: "Download later" }).click();
  await expect(page.locator(".modal")).toHaveCount(0);
  await expect.poll(async () => (await calls(page, "queue_put")).length).toBe(1);
  const c = await calls(page, "info_confirm");
  expect(c[0]).toMatchObject({ later: true });
  expect(await calls(page, "queue_put")).toEqual([{ ids: [c[0].id], queue: "q-1" }]);
});

test("G7 queue: the queue node lists files in order, the context menu removes, reorder re-puts", async ({ page }) => {
  await open(page, "scene=states&lang=en", 1280, 760);
  await mainNav(page).nth(10).click();
  await expect(page.locator(".row")).toHaveCount(3);
  await expect(page.locator(".row").first()).toContainText("dataset-2026-q3.zip");
  await page.locator(".row").first().click({ button: "right" });
  await page.locator(".menu .mi", { hasText: "Remove from queue" }).click();
  await expect(page.locator(".row")).toHaveCount(2);
  await setSec(3, async (p) => p.locator(".sbody .tabs .tab").nth(1).click())(page);
  await expect(page.locator(".sbody .hl-row")).toHaveCount(2);
  await page.locator(".sbody .hl-row").first().locator("button").nth(1).click();
  await expect(page.locator(".sbody .hl-row").first()).toContainText("report-q3.pdf");
  const put = await calls(page, "queue_put");
  expect(put.slice(-2).map((x) => x.queue)).toEqual([null, "main"]);
});

test("G7 scheduler: Tasks > Scheduler opens queue settings; new queue; program only via the native picker", async ({ page }) => {
  await open(page, "scene=states&lang=en", 1100, 700);
  await tasks(page, 4);
  await expect(page.locator(".sethead h2")).toHaveText("Queues and scheduler");
  await page.locator(".sbody button", { hasText: "Choose program" }).click();
  await expect.poll(async () => (await calls(page, "hook_set")).length).toBe(1);
  expect((await calls(page, "hook_set"))[0]).toMatchObject({ queue: "main", pick: true, clear: false });
  await page.locator(".sbody button", { hasText: "New queue" }).click();
  await page.locator(".sbody .col-opt input").first().fill("Night");
  await page.locator(".sbody button", { hasText: "Create" }).click();
  await expect(page.locator(".sbody select").first()).toHaveValue("q-2");
  const set = await calls(page, "settings_set");
  const qs = (set[set.length - 1].patch as { queues: { id: string; name: string }[] }).queues;
  expect(qs.map((q) => q.id)).toEqual(["main", "sync", "q-1", "q-2"]);
  expect(qs[3].name).toBe("Night");
});

test("G7 site login: the password is sent once and never kept in the form", async ({ page }) => {
  await open(page, "scene=states&lang=en", 1100, 700);
  await setSec(5)(page);
  await page.locator(".sbody input").nth(0).fill("https://dl.example.com/members/");
  await page.locator(".sbody input").nth(1).fill("me");
  await page.locator(".sbody input[type=password]").fill("s3cret");
  await page.locator(".sbody button", { hasText: "Save login" }).click();
  await expect(page.locator(".sbody .hl-row")).toHaveCount(2);
  await expect(page.locator(".sbody input[type=password]")).toHaveValue("");
  expect(await calls(page, "login_set")).toEqual([{ prefix: "https://dl.example.com/members/", user: "me", password: "s3cret" }]);
});

test("G7 P1 #21: several links dropped on the main window each go to add_url", async ({ page }) => {
  await open(page, "scene=states&lang=en", 1100, 700);
  await page.evaluate(() => {
    const dt = new DataTransfer();
    dt.setData("text/uri-list", "# comment\r\nhttps://a.example.org/1.zip\r\nhttps://a.example.org/2.zip\r\njavascript:alert(1)");
    document.querySelector(".main")!.dispatchEvent(new DragEvent("drop", { dataTransfer: dt, bubbles: true, cancelable: true }));
  });
  await expect.poll(async () => (await calls(page, "add_url")).map((c) => c.url)).toEqual(["https://a.example.org/1.zip", "https://a.example.org/2.zip"]);
});

test("G7 P1 #27: a column header sorts, its menu hides a column; both are saved in settings.columns", async ({ page }) => {
  await open(page, "scene=states&lang=en", 1100, 700);
  await page.locator(".cols .colh", { hasText: "Size" }).click();
  await expect(page.locator(".cols [aria-sort=ascending]")).toHaveCount(1);
  let set = await calls(page, "settings_set");
  expect((set[set.length - 1].patch as { columns: string[] }).columns).toEqual(["name", "size", "status", "speed", "eta", "sort:size:asc"]);
  await expect(page.locator(".row").first()).toContainText("backup-nas.7z"); // unknown size first
  await page.locator(".cols .colh", { hasText: "Speed" }).click({ button: "right" });
  await page.locator(".menu .mi", { hasText: /^Speed$/ }).click();
  await expect(page.locator(".cols .colh", { hasText: "Speed" })).toHaveCount(0);
  set = await calls(page, "settings_set");
  expect((set[set.length - 1].patch as { columns: string[] }).columns).toEqual(["name", "size", "status", "eta", "sort:size:asc"]);
});

test("G7 P1 #19/#26: remember a folder for the category; change the address of a stopped download", async ({ page }) => {
  await open(page, "scene=info&lang=en", 1100, 700);
  await page.locator(".modal button", { hasText: "Remember a folder for" }).click();
  await expect(page.locator(".modal .url").first()).toHaveText("D:\\Lectures");
  expect(await calls(page, "category_pick_dir")).toEqual([{ id: expect.any(String), clear: false }]);
  await page.locator(".modal .btn-g", { hasText: "Cancel" }).click();
  await page.locator(".row").nth(10).click({ button: "right" });
  await page.locator(".menu .mi", { hasText: "Change address" }).click();
  await page.locator(".modal input").fill("https://dl.example.org/new/report-q3.pdf");
  await page.locator(".modal .btn-p").click();
  await expect(page.locator(".modal")).toHaveCount(0);
  expect(await calls(page, "job_set_url")).toEqual([{ id: expect.any(String), url: "https://dl.example.org/new/report-q3.pdf" }]);
});

test("G7 copied video link: a toast asks first, Choose quality opens the quality list, nothing starts by itself", async ({ page }) => {
  await open(page, "scene=states&lang=en&videolink=1", 1100, 700);
  await expect(page.locator(".toast")).toContainText("Download this video?");
  expect(await calls(page, "media_probe")).toHaveLength(0);
  await page.locator(".toast button", { hasText: "Choose quality" }).click();
  await expect(page.locator(".modal [role=radio]")).toHaveCount(4);
  expect(await calls(page, "media_probe")).toEqual([{ url: "https://video.example.org/watch?v=abc" }]);
  expect(await calls(page, "media_start")).toHaveLength(0);
});
