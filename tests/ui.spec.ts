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
  for (const dpr of [1.25, 1.5])
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
