// GDM P0 design check: owner rule 8 (no glow / no outline), D6 focus visibility, Khmer clipping, bundled font.
// Run: NODE_PATH=<dir with playwright-core> node design/check.cjs [chrome.exe]
// First runs self-test fixtures with known answers (the check must catch planted faults), then the mockups.
// Writes screenshots to design/mockups/shots and a JSON report to docs/p0/design-check.json. Exit 1 on any failure.
// Limits (see docs/p0/exit.md): deviceScaleFactor emulates DPI, not Windows scaling in WebView2; forced-colors is
// Chromium's emulation, not a real Windows contrast theme; the field focus number is a project heuristic, not a WCAG verdict.
const http = require("http"), fs = require("fs"), path = require("path");

const ROOT = path.resolve(__dirname, "..");
const SHOTS = path.join(__dirname, "mockups", "shots");
const SCREENS = { main: [1100, 700], details: [1280, 760], info: [1100, 700], settings: [1100, 700], onboarding: [1100, 700], states: [1100, 700] };
const LANGS = ["km", "en"], THEMES = ["dark", "light"], SCALES = [1, 1.25, 1.5];
const FONTS_OK = ["Kantumruy Pro", "Outfit", "JetBrains Mono"];   // platform names carry the weight, e.g. "Kantumruy Pro SemiBold"
const MIME = { ".html": "text/html", ".css": "text/css", ".js": "text/javascript", ".woff2": "font/woff2", ".png": "image/png" };

// Local test server: loopback only, and only design/ + logo/ (no path escape, no symlink escape).
const SERVED = ["design", "logo"].map(d => fs.realpathSync(path.join(ROOT, d)));
const server = http.createServer((q, r) => {
  let f;
  try { f = fs.realpathSync(path.join(ROOT, decodeURIComponent(q.url.split("?")[0]))); } catch { f = null; }
  const inside = f && SERVED.some(d => { const rel = path.relative(d, f); return rel && !rel.startsWith("..") && !path.isAbsolute(rel); });
  if (!inside || !fs.statSync(f).isFile()) { r.writeHead(404); return r.end(); }
  r.writeHead(200, { "content-type": MIME[path.extname(f)] || "application/octet-stream" }); fs.createReadStream(f).pipe(r);
});

// --- rule 8: CSS-level probe (every element + pseudo) -----------------------
function rulesProbe(allowHcBorders) {
  const bad = [];
  const black = s => /^rgba?\(0, 0, 0(, [\d.]+)?\)$/.test(s.trim());
  const shadowOk = v => v === "none" || v.split(/,(?![^(]*\))/).every(p => { const c = p.match(/rgba?\([^)]*\)/); return c && black(c[0]); });
  for (const el of document.querySelectorAll("*")) for (const pseudo of [null, "::before", "::after"]) {
    const cs = getComputedStyle(el, pseudo);
    if (pseudo && cs.content === "none") continue;
    const id = (el.id ? "#" + el.id : el.tagName.toLowerCase() + (typeof el.className === "string" && el.className.trim() ? "." + el.className.trim().split(/\s+/).join(".") : "")) + (pseudo || "");
    if (!allowHcBorders) for (const s of ["Top", "Right", "Bottom", "Left"]) if (parseFloat(cs["border" + s + "Width"]) > 0 && cs["border" + s + "Style"] !== "none") bad.push(id + " border-" + s);
    if (cs.outlineStyle !== "none" && parseFloat(cs.outlineWidth) > 0) bad.push(id + " outline");
    if (/(^|, )(inset )?(rgba?\([^)]*\) )?0px 0px 0px [\d.]+px( inset)?/.test(cs.boxShadow) && /inset/.test(cs.boxShadow)) bad.push(id + " inset ring");
    else if (!shadowOk(cs.boxShadow)) bad.push(id + " coloured box-shadow");
    if (cs.textShadow !== "none") bad.push(id + " text-shadow");
    if (/drop-shadow/.test(cs.filter)) bad.push(id + " drop-shadow");
  }
  return [...new Set(bad)];
}

// --- horizontal overflow / offscreen (DOM) ----------------------------------
function layoutProbe() {
  const out = { hClip: [], ellipsis: 0, offscreen: [] };
  for (const el of document.querySelectorAll("body *")) {
    const own = [...el.childNodes].filter(n => n.nodeType === 3 && n.textContent.trim()).map(n => n.textContent).join("");
    if (!own || el.closest("[inert]")) continue;   // under a modal: not visible content
    const cs = getComputedStyle(el), r = el.getBoundingClientRect();
    if (r.width === 0) continue;
    let sc = el.parentElement; while (sc && !/auto|scroll/.test(getComputedStyle(sc).overflowY)) sc = sc.parentElement;
    if (r.left < -1 || r.right > innerWidth + 1 || (!sc && (r.top < -1 || r.bottom > innerHeight + 1))) out.offscreen.push(own.slice(0, 40));
    if ((cs.overflowX === "hidden" || cs.overflowX === "clip") && el.scrollWidth > el.clientWidth + 1) {
      // ellipsis allowed only where the full text is reachable as a tooltip (file names)
      const full = el.closest("[title], [data-tip]"); // data-tip = GDM's own tooltip (Tooltip.tsx)
      if (cs.textOverflow === "ellipsis" && full && (full.getAttribute("data-tip") || full.title || "").trim()) out.ellipsis++; else out.hClip.push(own.slice(0, 40) + (cs.textOverflow === "ellipsis" ? " (ellipsis without tooltip)" : ""));
    }
  }
  return out;
}

// --- vertical clipping by pixels --------------------------------------------
// For every text element under an overflow:hidden/clip box (itself or an ancestor; scrollers excluded), paint
// its text, then paint it transparent: the difference is exactly its ink. Do this with every clipping box set
// to overflow:visible, and fail if any ink row lies above or below the clip boxes' common vertical extent.
function clipSetup() {
  const isClip = e => /hidden|clip/.test(getComputedStyle(e).overflowY + getComputedStyle(e).overflowX);
  const items = [];
  let i = 0;
  for (const el of document.querySelectorAll("body *")) {
    const own = [...el.childNodes].filter(n => n.nodeType === 3 && n.textContent.trim()).map(n => n.textContent).join("");
    if (!own || el.getBoundingClientRect().width === 0 || el.closest("[inert]")) continue;   // inert = behind the modal scrim (blurred)
    let top = -Infinity, bottom = Infinity, any = false;
    for (let a = el; a && a !== document.body; a = a.parentElement) if (isClip(a)) {
      const r = a.getBoundingClientRect(); top = Math.max(top, r.top); bottom = Math.min(bottom, r.bottom); any = true; a.dataset.unclip = "";
    }
    if (!any) continue;
    const r = el.getBoundingClientRect();
    if (r.bottom < 0 || r.top > innerHeight) continue;
    el.dataset.ck = i;
    items.push({ i: i++, id: el.id || el.parentElement.id || "", text: own.slice(0, 40), top, bottom, x: r.left, w: r.width, y: r.top, h: r.height, fs: parseFloat(getComputedStyle(el).fontSize) });
  }
  const st = document.createElement("style"); st.id = "unclip";
  // every OTHER text is transparent while one element is measured: hiding a text repaints its box, and at a
  // fractional scale (125 %) that repaint can cover the edge of a neighbour's text (a row's "Open" link above),
  // which the diff would then count as this element's ink (CI-only failure 2026-11-14)
  st.textContent = "[data-unclip]{overflow:visible!important;text-overflow:clip!important}[data-hide]{color:transparent!important;-webkit-text-fill-color:transparent!important}[data-hide] svg{visibility:hidden}"
    + "[data-solo] *:not([data-meas]):not(:has([data-meas])){color:transparent!important;-webkit-text-fill-color:transparent!important}";
  document.head.append(st);
  return items;
}
async function clipRun(page, diffPage) {
  const items = await page.evaluate(clipSetup);
  const dpr = await page.evaluate(() => devicePixelRatio);
  const vp = page.viewportSize(), out = [];
  for (const it of items) {
    const moved = await page.evaluate(it => { const r = document.querySelector(`[data-ck="${it.i}"]`).getBoundingClientRect(); return Math.abs(r.top - it.y) > .5 || Math.abs(r.height - it.h) > .5; }, it);
    if (moved) { out.push({ id: it.id, text: it.text, clipped: "geometry changed when unclipped" }); continue; }
    const pad = Math.ceil(it.fs);
    const y0 = Math.max(0, Math.floor(Math.min(it.top, it.y) - pad)), y1 = Math.min(vp.height, Math.ceil(Math.max(it.bottom, it.y + it.h) + pad));
    const x0 = Math.max(0, Math.floor(it.x)), x1 = Math.min(vp.width, Math.ceil(it.x + it.w));
    if (x1 - x0 < 1 || y1 - y0 < 1) continue;
    const clip = { x: x0, y: y0, width: x1 - x0, height: y1 - y0 };
    await page.evaluate(i => { document.body.dataset.solo = ""; document.querySelector(`[data-ck="${i}"]`).dataset.meas = ""; }, it.i);
    const a = (await page.screenshot({ clip })).toString("base64");
    await page.evaluate(i => document.querySelector(`[data-ck="${i}"]`).setAttribute("data-hide", ""), it.i);
    const b = (await page.screenshot({ clip })).toString("base64");
    await page.evaluate(i => { const e = document.querySelector(`[data-ck="${i}"]`); e.removeAttribute("data-hide"); delete e.dataset.meas; delete document.body.dataset.solo; }, it.i);
    const rows = await diffPage.evaluate(async ([a, b]) => {
      const load = async s => { const i = await createImageBitmap(await (await fetch("data:image/png;base64," + s)).blob()); const c = new OffscreenCanvas(i.width, i.height); const x = c.getContext("2d"); x.drawImage(i, 0, 0); return x.getImageData(0, 0, i.width, i.height); };
      const A = await load(a), B = await load(b), rows = [];
      for (let y = 0; y < A.height; y++) for (let x = 0; x < A.width; x++) { const k = (y * A.width + x) * 4;
        if (Math.abs(A.data[k] - B.data[k]) + Math.abs(A.data[k + 1] - B.data[k + 1]) + Math.abs(A.data[k + 2] - B.data[k + 2]) > 60) { rows.push(y); break; } }
      return rows;
    }, [a, b]);
    if (!rows.length) continue;
    const inkTop = y0 + rows[0] / dpr, inkBot = y0 + (rows[rows.length - 1] + 1) / dpr;
    const over = Math.max(it.top - inkTop, inkBot - it.bottom);
    if (over > 0.75) out.push({ id: it.id, text: it.text, clipped: `ink ${inkTop.toFixed(1)}-${inkBot.toFixed(1)} outside box ${it.top.toFixed(1)}-${it.bottom.toFixed(1)}` });
  }
  await page.evaluate(() => { document.getElementById("unclip").remove(); document.querySelectorAll("[data-unclip],[data-ck]").forEach(e => { delete e.dataset.unclip; delete e.dataset.ck; }); });
  return out;
}

// --- real font per Khmer node (CDP) -----------------------------------------
async function fontRun(page) {
  const cdp = await page.context().newCDPSession(page);
  await page.evaluate(() => { let n = 0; for (const el of document.querySelectorAll("body *")) {
    const own = [...el.childNodes].filter(t => t.nodeType === 3).map(t => t.textContent).join("");
    if (own.trim() && !el.closest("[inert]") && n++ < 120) el.dataset.km = ""; } });   // every visible text node: Khmer and Latin must both come from bundled fonts
  await cdp.send("DOM.enable"); await cdp.send("CSS.enable");
  const { root } = await cdp.send("DOM.getDocument", { depth: -1 });
  const { nodeIds } = await cdp.send("DOM.querySelectorAll", { nodeId: root.nodeId, selector: "[data-km]" });
  const bad = new Set(), seen = new Set();
  for (const nodeId of nodeIds) {
    const { fonts } = await cdp.send("CSS.getPlatformFontsForNode", { nodeId });
    for (const f of fonts) { const fam = FONTS_OK.find(k => f.familyName.startsWith(k)) || f.familyName; seen.add(fam);
      if (!FONTS_OK.includes(fam)) { const { outerHTML } = await cdp.send("DOM.getOuterHTML", { nodeId }); bad.add(fam + " in " + outerHTML.replace(/<[^>]+>/g, "").slice(0, 40)); } }
  }
  await page.evaluate(() => document.querySelectorAll("[data-km]").forEach(e => delete e.dataset.km));
  await cdp.detach();
  return { nodes: nodeIds.length, fonts: [...seen], fallback: [...bad] };
}

// --- focus: real Tab, pixels at rest vs focused -----------------------------
async function focusContrast(diffPage, restB64, focB64) {
  return diffPage.evaluate(async ([a, b]) => {
    const load = async s => { const i = await createImageBitmap(await (await fetch("data:image/png;base64," + s)).blob()); const c = new OffscreenCanvas(i.width, i.height); const x = c.getContext("2d"); x.drawImage(i, 0, 0); return x.getImageData(0, 0, i.width, i.height); };
    const A = await load(a), Bm = await load(b);
    const L = (d, i) => { const f = v => { v /= 255; return v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4; }; return .2126 * f(d[i]) + .7152 * f(d[i + 1]) + .0722 * f(d[i + 2]); };
    let n3 = 0, nv = 0, max = 1;
    for (let i = 0; i < A.data.length; i += 4) { const l1 = L(A.data, i), l2 = L(Bm.data, i); const c = (Math.max(l1, l2) + .05) / (Math.min(l1, l2) + .05); if (c >= 3) n3++; if (c >= 1.08) nv++; if (c > max) max = c; }
    return { px3: n3, pxv: nv, max: +max.toFixed(2) };
  }, [restB64, focB64]);
}
const CAP = 80;
async function focusRun(page, diffPage) {
  await page.mouse.move(2, 2);
  const res = [];
  await page.evaluate(() => document.activeElement && document.activeElement.blur());
  let i = 0;
  for (; i < CAP; i++) {
    await page.keyboard.press("Tab");
    const info = await page.evaluate(() => { const e = document.activeElement; if (!e || e === document.body) return null;
      const typed = e.tagName === "INPUT" || e.tagName === "TEXTAREA"; const field = typed || e.getAttribute("role") === "row" || e.classList.contains("nav"); const t = typed ? (e.closest(".fld, .search") || e) : e; const r = t.getBoundingClientRect();
      return { tag: e.tagName.toLowerCase(), label: (e.getAttribute("aria-label") || e.value || e.textContent || "").trim().slice(0, 30), x: r.x, y: r.y, w: r.width, h: r.height, field }; });
    if (!info) break;
    const key = Math.round(info.x) + "," + Math.round(info.y) + info.label;
    if (res.find(r => r.key === key)) break;
    if (info.w < 1 || info.y + info.h > page.viewportSize().height + 1) { res.push({ key, ...info, ok: "FAIL (focused element not visible on screen)" }); continue; }
    const clip = { x: Math.max(0, info.x), y: Math.max(0, info.y), width: Math.max(1, info.w), height: Math.max(1, info.h) };
    const foc = await page.screenshot({ clip });
    await page.evaluate(() => { window.__f = document.activeElement; window.__f.blur(); });
    const rest = await page.screenshot({ clip });
    await page.evaluate(() => window.__f.focus({ preventScroll: true }));   // keyboard modality stays, so :focus-visible returns
    const c = await focusContrast(diffPage, rest.toString("base64"), foc.toString("base64"));
    const need = 4 * (info.w + info.h);            // area of a 2 CSS px thick perimeter (WCAG 2.4.13), at DPR 1
    // owner 2026-11-13 (replaces Q2's plate for fields): a field's own fill brightens a little (Mazer). A field passes when
    // its fill visibly changes over at least half its box; a field whose focus changes nothing still fails. Buttons keep 3:1.
    const ok = info.field ? c.pxv >= 0.5 * info.w * info.h : c.px3 >= need;
    res.push({ key, ...info, ...c, need: info.field ? Math.round(0.5 * info.w * info.h) : need, ok: ok ? "pass" : "FAIL" });
  }
  if (!res.length) res.push({ label: "(no tab stops)", ok: "FAIL" });
  if (i === CAP) res.push({ label: "(tab cap reached)", ok: "FAIL" });
  return res;
}

// --- self-test: planted faults must be caught, clean fixtures must pass -----
const FIXTURE = `
<div style="padding:20px;display:grid;gap:14px;width:420px;font-size:16px">
 <div id="t-clip" style="overflow:hidden;height:15px;line-height:15px">ស្ត្រី ក្ស្ត្រ ញ្ញុំ</div>
 <div id="t-anc" style="overflow:hidden;height:16px"><span style="line-height:1.7">ស្ត្រី ក្ស្ត្រ ញ្ញុំ</span></div>
 <div id="t-ok" style="overflow:hidden;line-height:1.9">ស្ត្រី ក្ស្ត្រ ញ្ញុំ</div>
 <button id="r1" style="outline:2px solid red">r1</button>
 <div id="r2" style="box-shadow:0 0 8px #3d9bff">r2</div>
 <div id="r3" style="box-shadow:inset 0 0 0 1px rgba(0,0,0,.5)">r3</div>
 <div id="r4" style="border-left:3px solid #fff">r4</div>
 <button class="fx t-btn t-noop">noop</button>
 <button class="fx t-btn">ok</button>
 <style>.t-btn{justify-self:start;padding:8px 14px;color:#fff;background:rgba(255,255,255,.08)} .t-btn.t-noop:focus-visible{background:rgba(255,255,255,.08)}</style>
</div>`;
async function selfTest(browser, base) {
  const ctx = await browser.newContext({ viewport: { width: 600, height: 520 }, reducedMotion: "reduce" });
  const page = await ctx.newPage(), diff = await ctx.newPage();
  await page.goto(`${base}?screen=onboarding&lang=km&theme=dark`);
  await page.evaluate(h => { document.getElementById("app").innerHTML = h; }, FIXTURE);   // fixed local fixture, not user input
  await page.evaluate(() => document.fonts.ready);
  const rules = await page.evaluate(rulesProbe);
  const clips = await clipRun(page, diff);
  const focus = await focusRun(page, diff);
  const expect = {
    "outline caught": rules.some(r => r.startsWith("#r1") && r.includes("outline")),
    "coloured shadow caught": rules.some(r => r.startsWith("#r2") && r.includes("coloured")),
    "inset ring caught": rules.some(r => r.startsWith("#r3") && r.includes("inset ring")),
    "border caught": rules.some(r => r.startsWith("#r4") && r.includes("border-Left")),
    "own-box clip caught": clips.some(c => c.id === "t-clip"),
    "ancestor clip caught": clips.some(c => c.id === "t-anc"),
    "clean text not flagged": !clips.some(c => c.id === "t-ok"),
    "no-op focus fails": focus.some(f => f.label === "noop" && f.ok === "FAIL"),
    "fill focus passes": focus.some(f => f.label === "ok" && f.ok === "pass"),
  };
  await ctx.close();
  return { expect, rules, clips, focus: focus.map(f => ({ label: f.label, ok: f.ok, px3: f.px3, need: f.need })) };
}

module.exports = { rulesProbe, layoutProbe, clipRun, fontRun, focusRun };
if (require.main === module) (async () => {
  const { chromium } = require("playwright-core");
  await new Promise(r => server.listen(0, "127.0.0.1", r));
  const base = `http://127.0.0.1:${server.address().port}/design/mockups/index.html`;
  const browser = await chromium.launch({ executablePath: process.argv[2] || undefined });
  fs.mkdirSync(SHOTS, { recursive: true });
  const report = { chromium: browser.version(), selfTest: await selfTest(browser, base), runs: [] };
  let fail = 0;
  for (const [k, v] of Object.entries(report.selfTest.expect)) { console.log(`${v ? "ok  " : "FAIL"} self-test: ${k}`); if (!v) fail++; }
  for (const [screen, [w, h]] of Object.entries(SCREENS)) for (const lang of LANGS) for (const theme of THEMES) for (const dpr of SCALES) {
    const ctx = await browser.newContext({ viewport: { width: w, height: h }, deviceScaleFactor: dpr, reducedMotion: "reduce" });
    const page = await ctx.newPage(), diffPage = await ctx.newPage();
    await page.goto(`${base}?screen=${screen}&lang=${lang}&theme=${theme}`);
    await page.evaluate(() => document.fonts.ready);
    const font = await fontRun(page);
    const rules = await page.evaluate(rulesProbe);
    const lay = await page.evaluate(layoutProbe);
    const vClip = await clipRun(page, diffPage);
    const run = { screen, lang, theme, dpr, font, rules, vClip, hClip: lay.hClip, offscreen: lay.offscreen, ellipsis: lay.ellipsis };
    const fontBad = font.fallback.length || (lang === "km" && !font.fonts.includes("Kantumruy Pro"));
    if (fontBad || rules.length || vClip.length || lay.hClip.length || lay.offscreen.length) run.FAIL = true;
    if (dpr === 1) {
      await page.mouse.move(2, 2);
      await page.screenshot({ path: path.join(SHOTS, `${screen}-${lang}-${theme}.png`) });
      run.focus = await focusRun(page, diffPage);
      await page.emulateMedia({ forcedColors: "active" });
      await page.reload(); await page.evaluate(() => document.fonts.ready);
      if (lang === "en" && theme === "dark") { await page.mouse.move(2, 2); await page.screenshot({ path: path.join(SHOTS, `${screen}-${lang}-highcontrast.png`) }); }
      run.focusHC = await focusRun(page, diffPage);
      run.rulesHC = await page.evaluate(rulesProbe, true);   // owner Q1 (2026-11-09): system borders allowed in high contrast only
      if (run.rulesHC.length || [...run.focus, ...run.focusHC].some(f => f.ok.startsWith("FAIL"))) run.FAIL = true;
    }
    if (run.FAIL) fail++;
    report.runs.push(run);
    const fl = run.focus ? [...run.focus.map(f => ["", f]), ...run.focusHC.map(f => ["hc:", f])] : [];
    console.log(`${run.FAIL ? "FAIL" : "ok  "} ${screen}/${lang}/${theme}@${dpr * 100}%  fonts:${font.fonts.join("+")} rules:${rules.length} vclip:${vClip.length} hclip:${lay.hClip.length} off:${lay.offscreen.length} ellipsis:${lay.ellipsis}` +
      (run.focus ? `  focus ${run.focus.filter(f => f.ok === "pass").length}/${run.focus.length} hc ${run.focusHC.filter(f => f.ok === "pass").length}/${run.focusHC.length}` +
        `${fl.some(([, f]) => f.ok.startsWith("FAIL")) ? " FAILS:" + fl.filter(([, f]) => f.ok.startsWith("FAIL")).map(([p, f]) => p + f.label + "(" + f.px3 + "/" + f.need + ")").join("; ") : ""}` : "") +
      (vClip.length ? " VCLIP:" + vClip.map(v => v.text + " " + v.clipped).join("; ") : "") + (rules.length ? " RULES:" + rules.join("; ") : "") + (lay.hClip.length ? " HCLIP:" + lay.hClip.join("; ") : "") + (font.fallback.length ? " FALLBACK:" + font.fallback.join("; ") : ""));
    await ctx.close();
  }
  fs.writeFileSync(path.join(ROOT, "docs", "p0", "design-check.json"), JSON.stringify(report, null, 1));
  await browser.close(); server.close();
  console.log(fail ? `\n${fail} failing checks` : "\nall checks pass");
  process.exit(fail ? 1 : 0);
})();
