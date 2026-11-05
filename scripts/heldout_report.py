"""Tables, reading and accounting for the held-out recovery pilot.

Written and committed while the stage-1 searches were running and before
`confirm` measured A on any audit seed. The reading below applies the table in
docs/heldout-recovery-pilot.md section 5 mechanically.
"""
import argparse
import json
from pathlib import Path
import statistics

ROOT = Path(__file__).resolve().parents[1]
RUNS = ROOT / "runs/heldout"
OUT = ROOT / "reports/heldout-recovery-2026-11-05"
CONDITIONS = ("native engine 1ba0af1, 450 s fight with the first 150 s excluded, one level-100 target, "
              "10% resistance, swap delay 12, periodic energy every 480-720 frames, each case's alt build.txt "
              "with the alternative's team.yaml params, fixed ER")


def reading(median_r, regression_share):
    """Pre-declared reading. The protocol's table has no row for median r >= 0.5
    with more than 5% regressions; that cell is reported as uncovered, and the
    next lower reading (Partial) is used. Declared before any r was measured."""
    if median_r is None:
        return "no informative case completed"
    if median_r >= 0.5:
        return "Promising: worth the full stage" if regression_share <= 0.05 else \
            "uncovered by the table (median r >= 0.5 but regressions > 5%); read as Partial"
    if median_r >= 0.2:
        return "Partial: identify which kinds succeed before scaling"
    return "Not supported at this budget and grammar"


def fmt(x, nd=0):
    if x is None:
        return "–"
    return f"{x:,.{nd}f}"


def pct(x):
    return "–" if x is None else f"{100 * x:+.1f}%"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--label", default="stage1")
    args = parser.parse_args()
    exp = json.loads((RUNS / args.label / "experiments.json").read_text())
    cases = exp["cases"]
    caps = ["36", "108", "192"]
    out = {"label": args.label, "conditions": CONDITIONS, "checkpoints": {}}
    lines = []
    for cap in caps + ["576"]:
        rows = [c for c in cases if c["checkpoints"].get(cap, {}).get("selected_start") is not None]
        inf = [c for c in rows if c["class"] == "informative"]
        rs = [c["checkpoints"][cap]["r"] for c in inf if c["checkpoints"][cap]["r"] is not None]
        reg = [c for c in rows if c["checkpoints"][cap]["regression"]]
        reg_inf = [c for c in inf if c["checkpoints"][cap]["regression"]]
        entry = {"cases": len(rows), "informative": len(inf), "median_r": statistics.median(rs) if rs else None,
                 "r_quartiles": statistics.quantiles(rs, n=4) if len(rs) >= 2 else None,
                 "informative_regressions": len(reg_inf), "all_regressions": [c["case"] for c in reg],
                 "share_r_at_least_0.5": sum(r >= 0.5 for r in rs) / len(rs) if rs else None,
                 "share_S_is_L": sum(c["checkpoints"][cap]["S_is_L"] for c in rows) / len(rows) if rows else None}
        for stratum in ("frame", "other"):
            sr = [c["checkpoints"][cap]["r"] for c in inf if c["stratum"] == stratum
                  and c["checkpoints"][cap]["r"] is not None]
            entry[f"median_r_{stratum}"] = statistics.median(sr) if sr else None
            entry[f"n_{stratum}"] = len(sr)
        if cap == "192":
            entry["reading"] = reading(entry["median_r"], len(reg_inf) / len(inf) if inf else 0)
        out["checkpoints"][cap] = entry
    # Q2: budget gains of the selected program, as audited (different audit seeds per checkpoint).
    gains = {}
    for a, b in [("36", "108"), ("108", "192"), ("192", "576")]:
        g = []
        for c in cases:
            x, y = c["checkpoints"].get(a, {}), c["checkpoints"].get(b, {})
            if x.get("S") and y.get("S"):
                g.append({"case": c["case"], "class": c["class"], "stratum": c["stratum"],
                          "gain": y["S"] / x["S"] - 1, "converged": (y["S"] / x["S"] - 1) < 0.01})
        gains[f"{a}->{b}"] = {"median_gain": statistics.median([x["gain"] for x in g]) if g else None,
                              "rows": g}
    out["budget"] = gains
    (OUT / "summary.json").write_text(json.dumps(out, indent=2) + "\n")

    # Markdown tables.
    lines.append("| Checkpoint (CPU-s per start) | Cases | Informative | Median r (informative) | r quartiles | "
                 "r ≥ 0.5 | Median r, frame / other | Informative regressions | S = L |")
    lines.append("|---|---:|---:|---:|---|---:|---|---:|---:|")
    for cap in caps + ["576"]:
        e = out["checkpoints"][cap]
        q = e["r_quartiles"]
        lines.append(f"| {cap} | {e['cases']} | {e['informative']} | {fmt(e['median_r'], 3)} | "
                     f"{'–' if not q else ', '.join(f'{v:.2f}' for v in q)} | "
                     f"{'–' if e['share_r_at_least_0.5'] is None else f'{100*e['share_r_at_least_0.5']:.0f}%'} | "
                     f"{fmt(e['median_r_frame'], 3)} (n={e['n_frame']}) / {fmt(e['median_r_other'], 3)} (n={e['n_other']}) | "
                     f"{e['informative_regressions']} | "
                     f"{'–' if e['share_S_is_L'] is None else f'{100*e['share_S_is_L']:.0f}%'} |")
    table_summary = "\n".join(lines)

    lines = ["| # | Case | Class | Stratum | L | A | A−L | S (selected) | r | Sel. start | S per start (11 / 37 / 91) | "
             "Start spread | Regression | CPU per start | A features | S features |",
             "|---:|---|---|---|---:|---:|---:|---:|---:|---:|---|---:|---|---|---|---|"]
    per_cap = {}
    for cap in caps + ["576"]:
        rows = []
        for i, c in enumerate(cases):
            k = c["checkpoints"].get(cap)
            if not k:
                continue
            if k.get("selected_start") is None:
                rows.append(f"| {i} | {c['case']} | {c['class']} | {c['stratum']} | {k.get('status')} |" + " |" * 11)
                continue
            starts = " / ".join(fmt(c["starts"][str(s)]["caps"].get(cap, {}).get("S"))
                                for s in (11, 37, 91))
            fa = c["features"]["A"]
            sf = c["starts"][str(k["selected_start"])]["caps"][cap]["S_features"]
            feat = lambda f: ",".join(n for n in ("hold", "while", "wait_or_delay") if f[n]) or "none"
            rows.append(f"| {i} | {c['case']} | {c['class']} | {c['stratum']} | {fmt(k['L'])} | {fmt(k['A'])} | "
                        f"{pct(k['A_minus_L'] / k['L'])} | {fmt(k['S'])} | {fmt(k['r'], 2)} | {k['selected_start']} | "
                        f"{starts} | {pct(k['start_spread'] / k['L'])} | "
                        f"{'yes' if k['regression'] else 'no'} ({fmt(k['S_minus_L'])} ± {fmt(k['S_minus_L_se'])}) | "
                        f"{', '.join(f'{x:.0f}' for x in k['cpu_per_start'])} | {feat(fa)} | {feat(sf)} |")
        per_cap[cap] = "\n".join(lines + rows)
    (OUT / "tables.md").write_text(
        f"# Generated tables ({args.label})\n\nConditions for every number: {CONDITIONS}. L, A and S of a "
        "row are audited on the same 128 fresh fights of the selected start's checkpoint audit seeds. "
        "S is the audited score of the pre-audit selection of the start with the highest discovery "
        "estimate. r = (S−L)/(A−L). Regression: S−L below −2 paired SE. Start spread: range of the three "
        "starts' audited S (each on its own audit seeds) relative to L.\n\n## Summary\n\n"
        + table_summary + "\n\n" + "\n\n".join(f"## Checkpoint {cap} CPU-s per start\n\n{t}" for cap, t in per_cap.items())
        + "\n")
    print(json.dumps({k: {x: v[x] for x in ("cases", "median_r", "informative_regressions")} | ({"reading": v["reading"]} if "reading" in v else {}) for k, v in out["checkpoints"].items()}, indent=1))


if __name__ == "__main__":
    main()
