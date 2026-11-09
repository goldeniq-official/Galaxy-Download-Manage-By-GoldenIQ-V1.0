"""Builds docs/GDM-Blueprint.html from the content modules next to this file.

Run: python -I docs/blueprint/build.py
The S1-S15 matrix is counted from the scenario cards, so the numbers cannot drift from the cards.
"""
import html
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from content import SECTIONS, META  # noqa: E402
from scenarios import FEATURES, SCEN_DEFS, SCENARIOS  # noqa: E402

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "GDM-Blueprint.html")
SEV_ORDER = {"critical": 0, "high": 1, "medium": 2, "low": 3}

CSS = """
:root{--bg:#06112b;--surface:#0c1a3d;--surface2:#11234f;--line:#1d3266;--fg:#eaf1ff;--muted:#9fb2d9;
--accent:#3d9bff;--accent-soft:rgba(61,155,255,.14);--lime:#b5e61d;--code-bg:#0a1634;
--crit:#9f1239;--high:#c2410c;--med:#a16207;--ok:#16a34a;
--f-body:"Kantumruy Pro","Noto Sans Khmer","Khmer OS System",system-ui,sans-serif;
--f-mono:"JetBrains Mono",ui-monospace,"Cascadia Mono",Consolas,monospace;color-scheme:dark}
@media (prefers-color-scheme:light){:root:not([data-theme="dark"]){--bg:#f6f8fc;--surface:#ffffff;--surface2:#eef3fb;--line:#dbe3f1;--fg:#0b1530;--muted:#4b5b7c;--accent:#0b6fd6;--accent-soft:rgba(11,111,214,.09);--lime:#4d7c0f;--code-bg:#eef2f9;color-scheme:light}}
:root[data-theme="light"]{--bg:#f6f8fc;--surface:#ffffff;--surface2:#eef3fb;--line:#dbe3f1;--fg:#0b1530;--muted:#4b5b7c;--accent:#0b6fd6;--accent-soft:rgba(11,111,214,.09);--lime:#4d7c0f;--code-bg:#eef2f9;color-scheme:light}
*{box-sizing:border-box}
html,body{overflow-x:clip}
body{margin:0;background:var(--bg);color:var(--fg);font-family:var(--f-body);font-size:16px;line-height:1.8}
.wrap{max-width:1240px;margin:0 auto;padding:40px max(16px,env(safe-area-inset-left,0px)) 80px;display:grid;grid-template-columns:260px minmax(0,1fr);gap:48px}
header.top{grid-column:1/-1;display:grid;grid-template-columns:auto 1fr;gap:20px;align-items:center;border-bottom:1px solid var(--line);padding-bottom:28px}
header.top img{width:88px;height:88px;border-radius:20px}
.eyebrow{font-family:var(--f-mono);font-size:12px;letter-spacing:.08em;text-transform:uppercase;color:var(--accent)}
h1{font-weight:700;font-size:clamp(26px,4vw,40px);line-height:1.4;margin:0;text-wrap:balance}
.byline{color:var(--muted);font-size:14px}
nav{position:sticky;top:24px;align-self:start;max-height:calc(100vh - 48px);overflow:auto}
nav ol{list-style:none;margin:0;padding:0;display:grid;gap:2px}
nav a{display:flex;gap:10px;padding:6px 8px;border-radius:6px;color:var(--muted);text-decoration:none;font-size:14px;line-height:1.55}
nav a:hover,nav a:focus-visible{background:var(--accent-soft);color:var(--fg);outline:none}
.n{font-family:var(--f-mono);font-size:12px;color:var(--accent);padding-top:2px;flex:none}
main{min-width:0;display:grid;gap:60px}
section{display:grid;gap:16px;min-width:0;scroll-margin-top:24px}
h2{font-size:25px;font-weight:600;line-height:1.5;margin:0;display:flex;gap:14px;align-items:baseline;text-wrap:balance}
h2 .n{font-size:14px}
h3{font-size:18px;font-weight:600;margin:14px 0 0;line-height:1.55}
h4{font-size:16px;margin:8px 0 0}
p,ul,ol{margin:0;max-width:78ch}
ul,ol{padding-left:1.4em;display:grid;gap:6px}
a{color:var(--accent)}
strong{font-weight:600}
code{font-family:var(--f-mono);font-size:.84em;background:var(--code-bg);border-radius:4px;padding:.05em .35em;overflow-wrap:anywhere}
pre{background:var(--code-bg);border-radius:8px;padding:14px;overflow-x:auto;font-family:var(--f-mono);font-size:13px;line-height:1.6;margin:0}
.tw{overflow-x:auto;border-radius:10px;background:var(--surface)}
table{border-collapse:collapse;width:100%;font-size:14.5px;line-height:1.7}
th,td{text-align:left;vertical-align:top;padding:9px 13px;border-bottom:1px solid var(--line)}
th{font-weight:600;color:var(--muted);font-size:13px;background:var(--surface2);white-space:nowrap}
tr:last-child td{border-bottom:0}
td.c{text-align:center;font-family:var(--f-mono);font-size:13px}
td.z{color:var(--muted)}
.kpis{display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:12px}
.kpi{background:var(--surface);border-radius:12px;padding:14px 16px}
.kpi b{display:block;font-size:30px;line-height:1.2}.kpi span{color:var(--muted);font-size:13px}
.note{background:var(--surface);border-radius:10px;padding:12px 16px;max-width:none}
.note.warn{background:color-mix(in srgb,var(--high) 16%,var(--surface))}
.sev{display:inline-block;font-family:var(--f-mono);font-size:11px;padding:1px 8px;border-radius:99px;background:var(--surface2);color:var(--muted);white-space:nowrap}
.sev.critical{background:var(--crit);color:#fff}.sev.high{background:var(--high);color:#fff}.sev.medium{background:var(--med);color:#fff}
.tag{display:inline-block;font-family:var(--f-mono);font-size:11px;padding:1px 7px;border-radius:99px;background:var(--accent-soft);color:var(--fg);margin-right:4px;white-space:nowrap}
.tag.keep{background:color-mix(in srgb,var(--ok) 22%,transparent)}.tag.better{background:var(--accent-soft)}.tag.new{background:color-mix(in srgb,var(--lime) 22%,transparent)}.tag.swap{background:color-mix(in srgb,var(--med) 30%,transparent)}
details.f{border-radius:10px;background:var(--surface);padding:10px 14px}
details.f+details.f{margin-top:-8px}
details.f summary{cursor:pointer;line-height:1.7;list-style-position:outside}
details.f summary .id{font-family:var(--f-mono);font-size:12.5px;color:var(--accent);margin-right:6px}
details.f dl{display:grid;grid-template-columns:110px minmax(0,1fr);gap:6px 14px;margin:10px 0 4px;font-size:14.5px}
details.f dt{color:var(--muted);font-size:13px}details.f dd{margin:0;overflow-wrap:anywhere}
.flt{display:flex;flex-wrap:wrap;gap:8px;align-items:center}
.flt button{font:inherit;font-size:13px;padding:4px 12px;border-radius:99px;border:0;background:var(--surface);color:var(--muted);cursor:pointer}
.flt button[aria-pressed="true"]{background:var(--accent);color:#fff}
figure{margin:0;overflow-x:auto;border-radius:10px;background:var(--surface);padding:16px}
figure svg{display:block;width:100%;min-width:640px;height:auto}
figcaption{color:var(--muted);font-size:13px;margin-top:8px}
.grid2{display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:14px}
.card{background:var(--surface);border-radius:12px;padding:14px 16px;display:grid;gap:8px;align-content:start}
.card pre{white-space:pre-wrap;overflow-wrap:anywhere}
button.copy{justify-self:start;font:inherit;font-size:13px;padding:4px 14px;border-radius:99px;border:0;background:var(--accent-soft);color:var(--fg);cursor:pointer}
button.copy:hover,button.copy:focus-visible{background:var(--accent);color:#fff;outline:none}
.card h4{margin:0}
.swatch{display:flex;gap:10px;align-items:center;font-family:var(--f-mono);font-size:13px}
.swatch i{width:28px;height:28px;border-radius:8px;display:inline-block}
.ln td:first-child{font-family:var(--f-mono);color:var(--accent);white-space:nowrap}
.ln td:nth-child(2) code{white-space:pre}
.theme{position:fixed;right:16px;bottom:16px;font:inherit;font-size:13px;padding:6px 14px;border-radius:99px;border:0;background:var(--surface2);color:var(--fg);cursor:pointer}
@media (max-width:900px){.wrap{grid-template-columns:minmax(0,1fr);gap:32px}nav{position:static;max-height:none;background:var(--surface);border-radius:10px;padding:8px}header.top{grid-template-columns:1fr}details.f dl{grid-template-columns:minmax(0,1fr)}}
@media (prefers-reduced-motion:no-preference){html{scroll-behavior:smooth}}
@media print{nav,.theme,.flt{display:none}.wrap{grid-template-columns:1fr}details.f{break-inside:avoid}}
"""

JS = """
(function(){
  var b=document.querySelector('.theme');
  b.addEventListener('click',function(){var r=document.documentElement;var d=r.getAttribute('data-theme');
    var now=d?(d==='dark'?'light':'dark'):(matchMedia('(prefers-color-scheme: light)').matches?'dark':'light');
    r.setAttribute('data-theme',now);try{localStorage.setItem('gdm-theme',now)}catch(e){}});
  try{var t=localStorage.getItem('gdm-theme');if(t)document.documentElement.setAttribute('data-theme',t)}catch(e){}
  var bs=document.querySelectorAll('.flt button');
  bs.forEach(function(x){x.addEventListener('click',function(){
    bs.forEach(function(y){y.setAttribute('aria-pressed','false')});x.setAttribute('aria-pressed','true');
    var v=x.dataset.f;document.querySelectorAll('details.f').forEach(function(d){
      d.hidden=!(v==='all'||d.dataset.sev===v||d.dataset.feat===v)})})});
  document.querySelectorAll('button.copy').forEach(function(x){x.addEventListener('click',function(){
    var t=document.getElementById(x.dataset.for).textContent;
    var done=function(){x.textContent='ចម្លងរួច';setTimeout(function(){x.textContent='ចម្លង'},1500)};
    if(navigator.clipboard)navigator.clipboard.writeText(t).then(done,function(){});})});
  var open=document.getElementById('openall');
  if(open)open.addEventListener('click',function(){var any=[].some.call(document.querySelectorAll('details.f'),function(d){return !d.open});
    document.querySelectorAll('details.f').forEach(function(d){if(!d.hidden)d.open=any})});
})();
"""


def esc(s):
    return html.escape(s, quote=True)


def scenario_section():
    feat_name = {f[0]: f[1] for f in FEATURES}
    ids = set()
    for s in SCENARIOS:
        assert s["id"] not in ids, "duplicate id " + s["id"]
        ids.add(s["id"])
        assert s["feat"] in feat_name, s["id"]
        assert s["sev"] in SEV_ORDER, s["id"]
        for k in s["S"]:
            assert k in SCEN_DEFS, (s["id"], k)
    # matrix
    keys = list(SCEN_DEFS)
    rows = []
    totals = {k: 0 for k in keys}
    for code, name in FEATURES:
        mine = [s for s in SCENARIOS if s["feat"] == code]
        worst = min((SEV_ORDER[s["sev"]] for s in mine), default=3)
        worst_name = [k for k, v in SEV_ORDER.items() if v == worst][0]
        cells = []
        for k in keys:
            c = sum(1 for s in mine if k in s["S"])
            totals[k] += c
            cells.append('<td class="c%s">%s</td>' % (" z" if not c else "", c or "·"))
        rows.append('<tr><td><span class="id">%s</span> %s</td><td class="c">%d</td><td><span class="sev %s">%s</span></td>%s</tr>'
                    % (code, esc(name), len(mine), worst_name, worst_name, "".join(cells)))
    rows.append('<tr><td><strong>សរុប</strong></td><td class="c">%d</td><td></td>%s</tr>'
                % (len(SCENARIOS), "".join('<td class="c">%d</td>' % totals[k] for k in keys)))
    defs = "".join("<tr><td><code>%s</code></td><td>%s</td></tr>" % (k, v) for k, v in SCEN_DEFS.items())
    matrix = ('<h3>និយមន័យ S1–S15</h3><div class="tw"><table><thead><tr><th>កូដ</th><th>សេនារីយ៉ូដែលត្រូវរត់លើមុខងារនីមួយៗ</th></tr></thead><tbody>%s</tbody></table></div>'
              '<h3>តារាង (ចំនួនសេនារីយ៉ូក្នុងមួយក្រឡា)</h3><div class="tw"><table><thead><tr><th>មុខងារ</th><th>សរុប</th><th>ខ្ពស់បំផុត</th>%s</tr></thead><tbody>%s</tbody></table></div>'
              % (defs, "".join("<th>%s</th>" % k for k in keys), "".join(rows)))
    # cards
    sev_count = {k: sum(1 for s in SCENARIOS if s["sev"] == k) for k in SEV_ORDER}
    btns = ['<button data-f="all" aria-pressed="true">ទាំងអស់ (%d)</button>' % len(SCENARIOS)]
    btns += ['<button data-f="%s" aria-pressed="false">%s (%d)</button>' % (k, k, sev_count[k]) for k in SEV_ORDER if sev_count[k]]
    btns += ['<button data-f="%s" aria-pressed="false">%s</button>' % (c, c) for c, _ in FEATURES]
    btns.append('<button id="openall" aria-pressed="false">បើក/បិទទាំងអស់</button>')
    cards = []
    for code, name in FEATURES:
        mine = sorted([s for s in SCENARIOS if s["feat"] == code], key=lambda s: SEV_ORDER[s["sev"]])
        cards.append('<h3 id="f-%s">%s · %s (%d)</h3>' % (code, code, esc(name), len(mine)))
        for s in mine:
            cards.append(
                '<details class="f" data-sev="%s" data-feat="%s"><summary><span class="id">%s</span><span class="sev %s">%s</span> %s</summary><dl>'
                '<dt>លក្ខខណ្ឌ</dt><dd>%s</dd><dt>សេនារីយ៉ូ</dt><dd>%s</dd><dt>ដំណោះស្រាយ</dt><dd>%s</dd>'
                '<dt>តេស្តការពារ</dt><dd>%s</dd><dt>ស្តង់ដារ</dt><dd>%s</dd><dt>S</dt><dd>%s</dd></dl></details>'
                % (s["sev"], code, s["id"], s["sev"], s["sev"], s["title"], s["cond"], s["scen"], s["fix"], s["test"],
                   s.get("std", "—"), " ".join('<span class="tag">%s</span>' % k for k in s["S"])))
    return matrix, '<div class="flt">%s</div>%s' % ("".join(btns), "".join(cards)), sev_count


def main():
    matrix, cards, sev_count = scenario_section()
    secs = []
    for sid, num, title, body in SECTIONS:
        body = body.replace("{{MATRIX}}", matrix).replace("{{CARDS}}", cards)
        body = body.replace("{{NSCEN}}", str(len(SCENARIOS)))
        for k, v in sev_count.items():
            body = body.replace("{{N_%s}}" % k.upper(), str(v))
        secs.append((sid, num, title, body))
    nav = "".join('<li><a href="#%s"><span class="n">%s</span>%s</a></li>' % (sid, num, esc(t)) for sid, num, t, _ in secs)
    main_html = "".join('<section id="%s"><h2><span class="n">%s</span>%s</h2>%s</section>' % (sid, num, esc(t), b) for sid, num, t, b in secs)
    page = """<!DOCTYPE html>
<html lang="km"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover">
<title>GDM Blueprint 2026–27</title>
<link rel="icon" href="%s">
<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Kantumruy+Pro:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap">
<style>%s</style></head><body>
<div class="wrap">
<header class="top"><img src="%s" alt="Galaxy Download Manager"><div><div class="eyebrow">%s</div><h1>%s</h1><div class="byline">%s</div></div></header>
<nav aria-label="មាតិកា"><ol>%s</ol></nav>
<main>%s</main>
</div>
<button class="theme" type="button">ពន្លឺ / ងងឹត</button>
<script>%s</script>
</body></html>""" % (META["logo"], CSS, META["logo"], esc(META["eyebrow"]), esc(META["title"]), esc(META["byline"]), nav, main_html, JS)
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write(page)
    print("wrote", os.path.normpath(OUT), len(page), "bytes;", len(SCENARIOS), "scenarios", sev_count)


if __name__ == "__main__":
    main()
