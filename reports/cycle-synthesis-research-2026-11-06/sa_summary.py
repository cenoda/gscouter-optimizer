#!/usr/bin/env python3
"""S/A of the selected program in the v2 and v2.1 runs, by A/L stratum, and
the size of the remaining held-out pool. Reads only committed report files.

Writes sa_summary.json next to this script.
"""
import collections
import json
import statistics as st
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
S1 = ROOT / "reports/heldout-recovery-2026-11-05"
V2 = ROOT / "reports/heldout-recovery-v2-2026-11-05"


def selected(cp):
    ss = [s for s in cp["starts"].values() if s.get("status") == "complete"]
    if not ss:
        return None, []
    return max(ss, key=lambda s: s["estimate"]), ss


def per_run(path, arms):
    rows = []
    for c in json.loads(path.read_text())["cases"]:
        if c["class"] != "informative":
            continue
        r = {"case": c["case"], "stratum": c["stratum"]}
        for arm, cap in arms:
            sel, ss = selected(c["arms"][arm][cap])
            if sel is None:
                continue
            r["A_over_L"] = sel["A"] / sel["L"]
            r[f"{arm}@{cap}"] = sel["S_effective"] / sel["A"]
            r[f"{arm}@{cap}:start_mean"] = st.mean(s["S_effective"] / s["A"] for s in ss)
        rows.append(r)
    return rows


def summarize(rows, key):
    out = {}
    for name, f in (("all", lambda r: True), ("A/L<1.25", lambda r: r["A_over_L"] < 1.25),
                    ("A/L>=1.25", lambda r: r["A_over_L"] >= 1.25), ("A/L>=2", lambda r: r["A_over_L"] >= 2)):
        v = [r[key] for r in rows if f(r) and key in r]
        if v:
            out[name] = {"n": len(v), "median_S_over_A": st.median(v), "below_0.90": sum(x < 0.90 for x in v),
                         "below_0.95": sum(x < 0.95 for x in v), "at_or_above_A": sum(x >= 1.0 for x in v)}
    return out


def pool():
    cls = json.loads((S1 / "classification.json").read_text())
    s1 = {o["case"] for o in json.loads((S1 / "draw.json").read_text())["order"]}
    v2 = {o["case"] if isinstance(o, dict) else o for o in json.loads((V2 / "draw.json").read_text())["order"]}
    dev_teams = {"flins-ineffa-columbina-sucrose", "mavuika-citlali-xilonen-nicole"}
    dev_parties = {c.split("--")[0] for c in s1 | v2}
    base = [r for r in cls["rows"] if r["kind"] == "database" and r["class"] in ("informative", "control")
            and r["case"] not in s1 and r["case"] not in v2 and r["case"].split("--")[0] not in dev_teams]
    strict = [r for r in base if r["case"].split("--")[0] not in dev_parties]
    def count(rs):
        inf = [r for r in rs if r["class"] == "informative"]
        return {"total": len(rs), "informative": len(inf), "control": len(rs) - len(inf),
                "informative_by_stratum": dict(collections.Counter(r["stratum"] for r in inf)),
                "informative_A_over_L>=1.25": sum(r["A"] / r["L"] >= 1.25 for r in inf),
                "informative_A_over_L>=2": sum(r["A"] / r["L"] >= 2 for r in inf),
                "informative_A_over_L<1.25": sum(r["A"] / r["L"] < 1.25 for r in inf)}
    return {"rule": "classification rows of kind database, class informative/control, not a stage-1 or v2 case, "
                    "not an alternative of flins-ineffa-columbina-sucrose / mavuika-citlali-xilonen-nicole; "
                    "'strict' also removes parties that appear among the 70 development cases (shared L)",
            "base": count(base), "strict": count(strict),
            "strict_cases": sorted(r["case"] for r in strict)}


def main():
    v2 = per_run(V2 / "experiments.json", [("V1", "192"), ("V2S", "192"), ("V2S", "384"), ("V2T", "384")])
    v21 = per_run(V2 / "v2.1/experiments.json", [("V2S", "192"), ("V2S", "384"), ("V2T", "384")])
    res = {"v2": {k: summarize(v2, k) for k in ("V1@192", "V2S@192", "V2S@384", "V2T@384")},
           "v2.1": {k: summarize(v21, k) for k in ("V2S@192", "V2S@384", "V2T@384")},
           "v2.1_rows": v21, "pool": pool()}
    (HERE / "sa_summary.json").write_text(json.dumps(res, indent=1))
    for run in ("v2", "v2.1"):
        for k, v in res[run].items():
            print(run, k, {s: (x["n"], round(x["median_S_over_A"], 3), x["below_0.90"], x["at_or_above_A"]) for s, x in v.items()})
    print(json.dumps({k: v for k, v in res["pool"].items() if k != "strict_cases"}, indent=1))


if __name__ == "__main__":
    main()
