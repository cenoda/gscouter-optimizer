"""Held-out recovery v2: measurement after all searches froze (confirm) and tables (report).

confirm: for every case, arm, start and checkpoint, A and L are measured on the
checkpoint's audit seeds (L must reproduce the job's audited L per fight).
For the selected start of each arm at 192 and 384, S, L and A are also
measured after the G4 strip transform. A is read only here.
"""
from concurrent.futures import ThreadPoolExecutor
import json
import math
from pathlib import Path
import statistics
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
RUNS1 = ROOT / "runs/heldout"
RUNS = ROOT / "runs/heldout-v2"
OUT = ROOT / "reports/heldout-recovery-v2-2026-11-05"
STARTS = [11, 37, 91]
ARMS = {"V1": [36, 108, 192], "V2S": [36, 108, 192, 288, 384], "V2T": [288, 384]}
STRIP_CAPS = [192, 384]
N = 128


def load(p):
    return json.loads(Path(p).read_text())


def write(p, v):
    p = Path(p)
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(v, indent=1) + "\n")


def paired(a, b):
    d = [x - y for x, y in zip(a, b, strict=True)]
    m = sum(d) / len(d)
    se = (sum((x - m) ** 2 for x in d) / (len(d) * (len(d) - 1))) ** 0.5
    return m, se


def capname(c):
    return str(int(c)) if float(c).is_integer() else str(c)


class Measurer:
    def __init__(self, binary, workdir):
        self.binary, self.workdir, self.cpu = binary, Path(workdir), 0.0

    def measure(self, manifest, rotation_text, seed, tag):
        d = self.workdir / tag
        d.mkdir(parents=True, exist_ok=True)
        rot = d / "rotation.gcsl"
        rot.write_text(rotation_text)
        p = subprocess.run([str(self.binary), "measure", "--manifest", str(manifest), "--rotation", str(rot),
                            "--seed", str(seed), "--samples", str(N), "--out", str(d / "eval")],
                           text=True, capture_output=True)
        if p.returncode:
            return {"status": "failed", "error": p.stderr.strip()[-300:]}
        r = json.loads(p.stdout)
        self.cpu += r["cpu_seconds"]
        return {"status": "complete", "per_fight": r["per_fight_dps"], "mean": r["stats"]["mean"]}

    def strip(self, text):
        with tempfile.TemporaryDirectory() as t:
            src, dst = Path(t) / "in.gcsl", Path(t) / "out.gcsl"
            src.write_text(text)
            subprocess.run([str(self.binary), "strip-assumptions", "--rotation", str(src), "--out", str(dst)],
                           check=True)
            return dst.read_text()


def program_text(sel):
    p = sel["candidate"]["program"]
    return p.get("Source") or p.get("Library") or ""


def confirm(args):
    draw = load(args.draw or OUT / "draw.json")
    results = Path(args.out or RUNS / "results")
    inputs_root = Path(args.inputs_root)
    cls = {r["case"]: r for r in load(ROOT / "reports/heldout-recovery-2026-11-05/classification.json")["rows"]}
    binary = Path(args.v2_bin)
    order = [o for o in draw["order"] if (results / o["case"] / "job.json").exists()]

    def job(entry):
        case = entry["case"]
        meas = Measurer(binary, RUNS / "confirm" / case)
        manifest = inputs_root / case / "manifest.json"
        L_text = (inputs_root / case / "L.gcsl").read_text()
        A_text = (RUNS1 / "withheld" / case / "A.gcsl").read_text()
        cache = {}

        def on_seed(name, text, seed):
            key = (name, seed)
            if key not in cache:
                cache[key] = meas.measure(manifest, text, seed, f"{name}-{seed}")
            return cache[key]

        out = {"case": case, "class": entry["class"], "stratum": entry["stratum"],
               "classification": {k: cls[case].get(k) for k in ("L", "A", "A_minus_L", "paired_se", "relative")},
               "features": cls[case]["features"], "job": load(results / case / "job.json"), "arms": {},
               "L_reproduction_failures": []}
        for arm, caps in ARMS.items():
            out["arms"][arm] = {}
            for cap in caps:
                starts = {}
                for seed in STARTS:
                    d = results / case / arm / str(seed)
                    rp = d / f"report-{capname(cap)}.json"
                    if not rp.exists():
                        starts[seed] = {"status": "missing"}
                        continue
                    rep, aud = load(rp), load(d / f"audit-{capname(cap)}.json")
                    sel = load(d / f"selected-{capname(cap)}.json")
                    seed_a = aud["audit_seed"]
                    Lf = aud["per_fight"][aud["baseline_key"]]
                    Sf = aud["per_fight"].get(sel["key"])
                    A = on_seed("A", A_text, seed_a)
                    Lm = on_seed("L", L_text, seed_a)
                    if Lm["status"] != "complete" or Lm["per_fight"] != Lf:
                        out["L_reproduction_failures"].append([arm, cap, seed])
                    c = {"status": "complete", "audit_seed": seed_a, "estimate": rep["curve"][-1]["best_search_estimate"],
                         "discovery_cpu": rep["discovery_cpu_seconds"], "audit_cpu": rep["audit_cpu_seconds"],
                         "cap_overrun": rep["cap_overrun_seconds"], "decisions": rep["completed_decisions"],
                         "timeouts": rep["timeouts_this_invocation"], "failed_fights": rep["failed_fights"],
                         "L": statistics.fmean(Lf), "S": statistics.fmean(Sf) if Sf and None not in Sf else None,
                         "S_is_L": sel["key"] == aud["baseline_key"], "selected_key": sel["key"],
                         "A": A.get("mean"), "second_pass": rep.get("second_pass")}
                    if c["S"] is not None and A["status"] == "complete":
                        c["S_minus_L"], c["S_minus_L_se"] = paired(Sf, Lf)
                        c["A_minus_L"], c["A_minus_L_se"] = paired(A["per_fight"], Lf)
                        c["r"] = c["S_minus_L"] / c["A_minus_L"] if c["A_minus_L"] else None
                        c["regression"] = c["S_minus_L"] < -2 * c["S_minus_L_se"]
                        c["S_per_fight"] = Sf
                    starts[seed] = c
                done = {s: v for s, v in starts.items() if v.get("status") == "complete"}
                entry_cap = {"starts": starts}
                if len(done) == len(STARTS):
                    seed = max(done, key=lambda s: done[s]["estimate"])
                    entry_cap["selected_start"] = seed
                    ss = [v["S"] for v in done.values() if v["S"] is not None]
                    entry_cap["start_spread"] = (max(ss) - min(ss)) if ss else None
                    if cap in STRIP_CAPS and done[seed]["S"] is not None:
                        d = results / case / arm / str(seed)
                        S_text = program_text(load(d / f"selected-{capname(cap)}.json"))
                        seed_a = done[seed]["audit_seed"]
                        s_re = meas.measure(manifest, S_text, seed_a, f"S-{arm}-{cap}-{seed}")
                        Sx = meas.measure(manifest, meas.strip(S_text), seed_a, f"Sstrip-{arm}-{cap}-{seed}")
                        Lx = on_seed("Lstrip", meas.strip(L_text), seed_a)
                        Ax = on_seed("Astrip", meas.strip(A_text), seed_a)
                        st = {"S_remeasure_matches": s_re.get("per_fight") == done[seed]["S_per_fight"],
                              "status": "complete"}
                        if all(x["status"] == "complete" for x in (Sx, Lx, Ax)):
                            st["S"], st["L"], st["A"] = Sx["mean"], Lx["mean"], Ax["mean"]
                            st["S_minus_L"], st["S_minus_L_se"] = paired(Sx["per_fight"], Lx["per_fight"])
                            st["A_minus_L"], _ = paired(Ax["per_fight"], Lx["per_fight"])
                            st["r"] = st["S_minus_L"] / st["A_minus_L"] if st["A_minus_L"] else None
                            st["regression"] = st["S_minus_L"] < -2 * st["S_minus_L_se"]
                        else:
                            st = {"status": "unavailable", "errors": {k: x.get("error") for k, x in
                                  (("S", Sx), ("L", Lx), ("A", Ax)) if x["status"] != "complete"},
                                  "S_remeasure_matches": st["S_remeasure_matches"]}
                        entry_cap["strip"] = st
                else:
                    entry_cap["status"] = "incomplete starts"
                out["arms"][arm][cap] = entry_cap
        out["measure_cpu"] = meas.cpu
        sel = out["arms"]["V2S"].get(384, {}).get("selected_start")
        print(case, {a: {c: round(v["starts"][v["selected_start"]].get("r") or float("nan"), 2)
                         for c, v in caps.items() if v.get("selected_start")} for a, caps in out["arms"].items()},
              flush=True)
        return out

    with ThreadPoolExecutor(max_workers=args.workers) as pool:
        rows = list(pool.map(job, order))
    for r in rows:
        for arm in r["arms"].values():
            for c in arm.values():
                for s in c["starts"].values():
                    s.pop("S_per_fight", None)
    write(args.dest or RUNS / "experiments.json", {"cases": rows})
    print("confirmed", len(rows), "cases")


# ---------------------------------------------------------------- report
def wilcoxon(d):
    """Two-sided Wilcoxon signed-rank, normal approximation with tie and zero handling (Pratt dropped)."""
    x = [v for v in d if v != 0]
    n = len(x)
    if n == 0:
        return {"n": 0, "p": 1.0}
    ranked = sorted(range(n), key=lambda i: abs(x[i]))
    ranks = [0.0] * n
    i = 0
    while i < n:
        j = i
        while j + 1 < n and abs(x[ranked[j + 1]]) == abs(x[ranked[i]]):
            j += 1
        for k in range(i, j + 1):
            ranks[ranked[k]] = (i + j) / 2 + 1
        i = j + 1
    w = sum(r for r, v in zip(ranks, x) if v > 0)
    mean = n * (n + 1) / 4
    ties = {}
    for v in x:
        ties[abs(v)] = ties.get(abs(v), 0) + 1
    var = n * (n + 1) * (2 * n + 1) / 24 - sum(t ** 3 - t for t in ties.values()) / 48
    z = (w - mean) / math.sqrt(var) if var > 0 else 0.0
    p = math.erfc(abs(z) / math.sqrt(2))
    return {"n": n, "W_plus": w, "z": z, "p": p}


def reading(median_r, regress_frac):
    if median_r is None:
        return "undefined"
    if median_r >= 0.5:
        return "Promising" if regress_frac <= 0.05 else "Partial (>5% regressions)"
    if median_r >= 0.2:
        return "Partial"
    return "Not supported"


def quart(v):
    if not v:
        return None
    q = statistics.quantiles(v, n=4) if len(v) > 1 else [v[0]] * 3
    return [round(x, 3) for x in q]


def report(args):
    ex = load(args.dest or RUNS / "experiments.json")["cases"]
    out_dir = Path(args.report_dir or OUT)
    inf = [c for c in ex if c["class"] == "informative"]
    ctl = [c for c in ex if c["class"] == "control"]
    lines = []
    summary = {"cases": len(ex), "informative": len(inf), "control": len(ctl), "arms": {}, "comparisons": {}}

    def sel(c, arm, cap):
        e = c["arms"][arm].get(str(cap))
        if not e or "selected_start" not in e:
            return None, None
        return e, e["starts"][str(e["selected_start"])]

    lines.append("## Pre-declared reading per arm (informative cases)\n")
    lines.append("| Arm @ cap | Cases with S | Median r | r quartiles | r ≥ 0.5 | Regressions | Reading | Median r_strip (n) | Reading (strip) | Control regressions |")
    lines.append("|---|---:|---:|---|---:|---:|---|---|---|---:|")
    for arm, caps in ARMS.items():
        for cap in caps:
            rs, regs, rstrip, sregs = [], 0, [], 0
            for c in inf:
                e, s = sel(c, arm, cap)
                if s is None or s.get("r") is None:
                    continue
                rs.append(s["r"])
                regs += bool(s["regression"])
                st = e.get("strip")
                if st and st.get("r") is not None:
                    rstrip.append(st["r"])
                    sregs += bool(st["regression"])
            creg = 0
            for c in ctl:
                e, s = sel(c, arm, cap)
                creg += bool(s and s.get("regression"))
            med = statistics.median(rs) if rs else None
            mstrip = statistics.median(rstrip) if rstrip else None
            row = {"n": len(rs), "median_r": med, "quartiles": quart(rs), "r_ge_half": sum(r >= 0.5 for r in rs),
                   "regressions": regs, "reading": reading(med, regs / len(rs) if rs else 1),
                   "strip_n": len(rstrip), "median_r_strip": mstrip,
                   "reading_strip": reading(mstrip, sregs / len(rstrip) if rstrip else 1) if cap in STRIP_CAPS else None,
                   "control_regressions": creg}
            summary["arms"][f"{arm}@{cap}"] = row
            lines.append(f"| {arm} @ {cap} | {len(rs)} | {med:.3f} | {row['quartiles']} | {row['r_ge_half']} | {regs} | {row['reading']} | "
                         + (f"{mstrip:.3f} ({len(rstrip)})" if mstrip is not None else "–") + f" | {row['reading_strip'] or '–'} | {creg}/{len(ctl)} |"
                         if med is not None else f"| {arm} @ {cap} | 0 | – | – | – | – | – | – | – | – |")

    def compare(name, x, y):
        d_avg, d_sel, wins = [], [], 0
        for c in inf:
            ex_ = c["arms"][x[0]].get(str(x[1]))
            ey = c["arms"][y[0]].get(str(y[1]))
            if not ex_ or not ey:
                continue
            L = c["classification"]["L"]
            diffs = []
            for s in STARTS:
                a, b = ex_["starts"].get(str(s)), ey["starts"].get(str(s))
                if a and b and a.get("S") is not None and b.get("S") is not None and a["audit_seed"] == b["audit_seed"]:
                    diffs.append((a["S"] - b["S"]) / a["L"])
            if len(diffs) == len(STARTS):
                d_avg.append(statistics.fmean(diffs))
            _, sa = sel(c, *x)
            _, sb = sel(c, *y)
            if sa and sb and sa.get("S") is not None and sb.get("S") is not None:
                d_sel.append((sa["S"] - sb["S"]) / L)
        w = wilcoxon(d_avg)
        med = statistics.median(d_avg) if d_avg else None
        if med is not None and w["p"] < 0.05 and med > 0:
            verdict = f"{x[0]}@{x[1]} beats {y[0]}@{y[1]}"
        elif med is not None and w["p"] < 0.05 and med < 0:
            verdict = f"{y[0]}@{y[1]} beats {x[0]}@{x[1]}"
        else:
            verdict = "no detectable difference at this sample size"
        ws = wilcoxon(d_sel)
        summary["comparisons"][name] = {"x": x, "y": y, "n": len(d_avg), "median_d_start_averaged": med,
                                        "mean_d": statistics.fmean(d_avg) if d_avg else None,
                                        "x_better_cases": sum(v > 0 for v in d_avg), "wilcoxon": w, "reading": verdict,
                                        "secondary_selected": {"n": len(d_sel), "median": statistics.median(d_sel) if d_sel else None,
                                                               "wilcoxon": ws}}
        return summary["comparisons"][name]

    lines.append("\n## Pre-declared comparisons (informative cases; d = start-averaged paired (S_x − S_y)/L on identical audit fights)\n")
    lines.append("| Comparison | n | Median d | Mean d | x better | Wilcoxon p | Reading | Secondary (selected S): median, p |")
    lines.append("|---|---:|---:|---:|---:|---:|---|---|")
    for name, x, y in [("Q2 two-pass vs equal-CPU single (primary)", ("V2T", 384), ("V2S", 384)),
                       ("Q1 v2 vs v1 at 192", ("V2S", 192), ("V1", 192)),
                       ("budget V2S 384 vs 192", ("V2S", 384), ("V2S", 192)),
                       ("(descriptive) V2T 288 vs V2S 288", ("V2T", 288), ("V2S", 288))]:
        r = compare(name, x, y)
        sec = r["secondary_selected"]
        lines.append(f"| {name} | {r['n']} | {fmt_pct(r['median_d_start_averaged'])} | {fmt_pct(r['mean_d'])} | {r['x_better_cases']}/{r['n']} | "
                     f"{r['wilcoxon']['p']:.3g} | {r['reading']} | {fmt_pct(sec['median'])}, {sec['wilcoxon']['p']:.3g} |")

    lines.append("\n## Per case (selected start's audit; L and A on that start's audit fights)\n")
    lines.append("| # | Case | Class | Stratum | L | A | S V1@192 (r) | S V2S@192 (r) | S V2S@384 (r) | S V2T@384 (r) | r_strip V2S@384 / V2T@384 | regressions |")
    lines.append("|---:|---|---|---|---:|---:|---|---|---|---|---|---|")
    for i, c in enumerate(ex):
        cells, regs, L, A = [], [], None, None
        for arm, cap in [("V1", 192), ("V2S", 192), ("V2S", 384), ("V2T", 384)]:
            _, s = sel(c, arm, cap)
            if s and s.get("S") is not None:
                L, A = L or s["L"], A or s["A"]
                cells.append(f"{s['S']:,.0f} ({s['r']:.2f})" if s.get("r") is not None else f"{s['S']:,.0f}")
                if s.get("regression"):
                    regs.append(f"{arm}@{cap}")
            else:
                cells.append("–")
        strip = []
        for arm in ("V2S", "V2T"):
            e, _ = sel(c, arm, 384)
            st = (e or {}).get("strip") or {}
            strip.append(f"{st['r']:.2f}" if st.get("r") is not None else "–")
        lines.append(f"| {i} | {c['case']} | {c['class']} | {c['stratum']} | {L or 0:,.0f} | {A or 0:,.0f} | " + " | ".join(cells)
                     + f" | {' / '.join(strip)} | {', '.join(regs) or 'none'} |")

    # Budget curve
    lines.append("\n## Budget curve (informative; median r of the selected start)\n")
    lines.append("| Arm | " + " | ".join(str(c) for c in [36, 108, 192, 288, 384]) + " |")
    lines.append("|---|" + "---:|" * 5)
    for arm in ARMS:
        row = []
        for cap in [36, 108, 192, 288, 384]:
            k = summary["arms"].get(f"{arm}@{cap}")
            row.append(f"{k['median_r']:.3f}" if k and k["median_r"] is not None else "–")
        lines.append(f"| {arm} | " + " | ".join(row) + " |")
    # Accounting
    acc = {"jobs": [], "totals": {}}
    for c in ex:
        j = c["job"]
        disc = sum(s["discovery_cpu"] for s in j["searches"])
        aud = sum(s["audit_cpu"] for s in j["searches"])
        acc["jobs"].append({"case": c["case"], "wall_seconds": j["wall_seconds"], "children_cpu_seconds": j["children_cpu_seconds"],
                            "discovery_cpu": disc, "audit_cpu": aud, "errors": len(j["errors"]),
                            "environment": j["environment"], "measure_cpu": c.get("measure_cpu")})
    acc["totals"] = {"discovery_cpu_seconds": sum(j["discovery_cpu"] for j in acc["jobs"]),
                     "audit_cpu_seconds": sum(j["audit_cpu"] for j in acc["jobs"]),
                     "job_children_cpu_seconds": sum(j["children_cpu_seconds"] for j in acc["jobs"]),
                     "sum_job_wall_seconds": sum(j["wall_seconds"] for j in acc["jobs"]),
                     "max_job_wall_seconds": max(j["wall_seconds"] for j in acc["jobs"]),
                     "local_measure_cpu_seconds": sum(j["measure_cpu"] or 0 for j in acc["jobs"]),
                     "job_errors": sum(j["errors"] for j in acc["jobs"]),
                     "cpu_models": sorted({j["environment"]["cpu_model"] for j in acc["jobs"]}),
                     "nproc": sorted({j["environment"]["nproc"] for j in acc["jobs"]}),
                     "L_reproduction_failures": sum(len(c["L_reproduction_failures"]) for c in ex)}
    summary["accounting"] = acc["totals"]
    write(out_dir / "accounting.json", acc)
    write(out_dir / "summary.json", summary)
    (out_dir / "tables.md").write_text("# Held-out recovery v2: tables\n\nGenerated by `scripts/heldout_v2_report.py`; conditions in README.md.\n\n" + "\n".join(lines) + "\n")
    print("\n".join(lines[:30]))


def fmt_pct(v):
    return "–" if v is None else f"{100 * v:+.2f}%"
