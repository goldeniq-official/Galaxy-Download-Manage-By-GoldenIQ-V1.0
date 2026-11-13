// SPDX-License-Identifier: RPL-1.5
// G9 site grabber wizard (F09) on the real React UI (mock backend): the 4 steps and the results list, km/en x
// dark/light, with the same probes as G6/G7 (owner rule 8 no border/outline/glow, X-03 Khmer ink, X-05 axe +
// visible focus). Behaviour: the folder never comes from the WebView (X-01), duplicates hidden by default,
// unticked files are not added.
import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";

const require = createRequire(import.meta.url);
const P = require("../design/check.cjs");
const SHOTS = path.join("docs", "evidence", "g9-shots");
fs.mkdirSync(SHOTS, { recursive: true });

async function open(page: Page, q: string) {
  await page.setViewportSize({ width: 1100, height: 700 });
  await page.goto(`/?${q}`);
  await page.waitForSelector("#shell");
  await page.evaluate(() => document.fonts.ready);
  await page.waitForTimeout(700);
}
const calls = (p: Page, cmd: string) =>
  p.evaluate((c) => ((window as unknown as { __calls?: [string, Record<string, unknown>][] }).__calls ?? []).filter((x) => x[0] === c).map((x) => x[1]), cmd);

async function wizard(p: Page) {
  await p.locator(".top .chip[aria-haspopup]").first().click();
  await p.locator(".menu .mi").last().click();
  await p.locator(".modal input").first().fill("https://example.com/gallery/");
}
const next = (p: Page) => p.locator(".modal .mfoot .btn-p").click();

const STEPS: Record<string, (p: Page) => Promise<void>> = {
  "step1": async (p) => wizard(p),
  "step2": async (p) => (await wizard(p), await next(p)),
  "step3": async (p) => (await wizard(p), await next(p), await next(p)),
  "step4": async (p) => (await wizard(p), await next(p), await next(p), await next(p)),
  "results": async (p) => {
    await wizard(p);
    for (let i = 0; i < 4; i++) await next(p);
    await p.waitForSelector(".modal .grab-list .check", { timeout: 5000 });
  },
};

for (const [name, go] of Object.entries(STEPS))
  for (const lang of ["km", "en"])
    for (const theme of ["dark", "light"])
      test(`g9 grabber ${name} ${lang} ${theme}`, async ({ page, context }) => {
        await open(page, `scene=states&lang=${lang}&theme=${theme}`);
        await go(page);
        await page.waitForTimeout(400);
        await page.mouse.move(2, 2);
        if (lang === "km") await page.screenshot({ path: path.join(SHOTS, `${name}-km-${theme}.png`) });
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

test("g9 grabber: ticked files only, duplicates hidden, no path from the WebView, login stays a plain value", async ({ page }) => {
  await open(page, "scene=states&lang=en&theme=dark");
  await wizard(page);
  await page.locator(".modal input").nth(1).fill("alice");
  await page.locator(".modal input[type=password]").fill("s3cret");
  for (let i = 0; i < 4; i++) await next(page);
  await page.waitForSelector(".modal .grab-list .check");
  const run = await calls(page, "grab_run");
  expect(run).toHaveLength(1);
  expect(run[0].user).toBe("alice");
  expect((run[0].cfg as Record<string, unknown>).creds, "no headers from the WebView").toBeUndefined();
  // 4 found, 1 duplicate hidden => 3 shown; untick one => 2 added
  await expect(page.locator(".modal .grab-list .check")).toHaveCount(3);
  await page.locator(".modal .grab-list .check").nth(1).click();
  await page.locator(".modal .mfoot .btn-p").click();
  const add = await calls(page, "grab_add");
  expect(add).toHaveLength(1);
  const files = add[0].files as { name: string }[];
  expect(files.map((f) => f.name)).toEqual(["photo-1.jpg", "album.zip"]);
  expect(Object.keys(add[0]).sort()).toEqual(["files", "keep", "start", "startUrl"]);
  expect(add[0].start, "added paused unless 'start right away' is ticked").toBe(false);
});

test("g9 grabber: a non-http start address cannot go on", async ({ page }) => {
  await open(page, "scene=states&lang=km&theme=dark");
  await page.locator(".top .chip[aria-haspopup]").first().click();
  await page.locator(".menu .mi").last().click();
  await page.locator(".modal input").first().fill("file:///C:/Windows/");
  await expect(page.locator(".modal .mfoot .btn-p")).toHaveAttribute("aria-disabled", "true");
  await expect(page.locator(".modal .err-line")).not.toBeEmpty();
});
