# SPDX-License-Identifier: RPL-1.5
"""plan.md ង.9-10 report from bench.ps1 CSVs: per URL median/worst, paired ratio r = GDM MB/s / IDM MB/s,
verdict (>=1.02 faster, 0.98-1.02 equal, <0.98 slower), geometric mean of median(r).
Usage: python -I bench_report.py raw1.csv [raw2.csv ...]   (warm-up pair 0 is excluded)"""
import csv, math, statistics, sys
from collections import defaultdict

rows = [r for f in sys.argv[1:] for r in csv.DictReader(open(f, encoding="utf-8-sig"))]
by = defaultdict(lambda: defaultdict(dict))  # (config,url) -> pair -> tool -> row
for r in rows:
    if int(r["pair"]) >= 1:
        by[(r["config"], r["url_id"])][int(r["pair"])][r["tool"]] = r

def verdict(m): return "faster" if m >= 1.02 else "equal" if m >= 0.98 else "slower"

for cfg in sorted({k[0] for k in by}):
    print(f"\n### config {cfg}\n")
    print("| URL | GDM MB/s median (worst) | IDM MB/s median (worst) | r per pair | median r | verdict | conns GDM/IDM | CPU avg GDM/IDM % | GDM priv MB peak | sha ok |")
    print("|---|---|---|---|---|---|---|---|---|---|")
    meds, wins = [], 0
    for (c, url), pairs in sorted(by.items()):
        if c != cfg: continue
        ps = [p for p in sorted(pairs) if "GDM" in pairs[p] and "IDM" in pairs[p]]
        g = [float(pairs[p]["GDM"]["MBps"]) for p in ps]
        i = [float(pairs[p]["IDM"]["MBps"]) for p in ps]
        r = [a / b for a, b in zip(g, i)]
        m = statistics.median(r); meds.append(m); wins += m >= 0.98
        f = lambda t, k: "/".join(sorted({pairs[p][t][k] for p in ps}))
        cpu = lambda t: statistics.median(float(pairs[p][t]["cpu_machine_avg"]) for p in ps)
        ok = all(pairs[p][t]["sha_ok"] == "True" for p in ps for t in ("GDM", "IDM"))
        print(f"| {url} | {statistics.median(g):.1f} ({min(g):.1f}) | {statistics.median(i):.1f} ({min(i):.1f}) | {' '.join(f'{x:.2f}' for x in r)} | {m:.3f} | {verdict(m)} "
              f"| {f('GDM','tcp_conns_peak')} / {f('IDM','tcp_conns_peak')} | {cpu('GDM'):.1f} / {cpu('IDM'):.1f} "
              f"| {max(float(pairs[p]['GDM']['priv_MB_peak']) for p in ps):.1f} | {'yes' if ok else 'NO'} |")
    gm = math.exp(sum(map(math.log, meds)) / len(meds))
    print(f"\nequal-or-faster on {wins}/{len(meds)} URLs; geometric mean of median(r) = {gm:.3f} (target >= 0.95)")
