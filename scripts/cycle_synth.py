"""Cycle-synthesis pilot (docs/cycle-synthesis-pilot.md).

Phases:
  export   per-case inputs (manifest, L, v2 drop list, donor drop list, party)
           for development cases (--cases) or the frozen held-out draw (draw)
  draw     seeded stratified census/draw of fresh cases; exports CI inputs
  job      one case: arms V21S, SYN, XO x 3 starts x caps 96/192/384, 4 processes,
           strace leakage check (no A, alt/ or Gscouter is opened)
  materialize  CI: rebuild the per-case dictionary from the global v2.1
           dictionary and the case's drop list, check the hash
  confirm  after every job: L and A on each start's audit seeds (A read here only)
  report   tables and pre-declared readings

The donor drop list is computed offline from A (like the v2 drop lists); jobs
receive only the list of dropped team names.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import json
import math
import os
from pathlib import Path
import random
import shutil
import statistics
import subprocess
import sys
import threading
import time

sys.path.insert(0, str(Path(__file__).resolve().parent))
from heldout_v2 import call, environment, journal, load, opened_paths, sha, write  # noqa: E402
from heldout_v2_report import paired, wilcoxon  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
RUNS1 = ROOT / "runs/heldout"
RUNSV2 = ROOT / "runs/heldout-v2"
RUNS = ROOT / "runs/cycle-synth"
STAGE1 = ROOT / "reports/heldout-recovery-2026-11-05"
V2 = ROOT / "reports/heldout-recovery-v2-2026-11-05"
OUT = ROOT / "reports/cycle-synthesis-2026-11-06"
PROTOCOL = ROOT / "docs/cycle-synthesis-pilot.md"
DRAW_SEED = 20261107
STARTS = [11, 37, 91]
CAPS = [96, 192, 384]
AUDIT_SAMPLES = 128
SELECTION_SAMPLES = 32
LOWER_AT = 192
ARMS = ["V21S", "SYN", "XO"]
FORBIDDEN = ("/alt/", "withheld", "/Gscouter/", "heldout-recovery-2026-11-05")
DEV_TEAMS = ("flins-ineffa-columbina-sucrose", "mavuika-citlali-xilonen-nicole")


def BIN(args):
    return Path(getattr(args, "bin", None) or ROOT / "target/release/gso").resolve()


def party(case):
    return case.split("--")[0]


# ---------------------------------------------------------------- export
def export_case(args, case, dest_root, cls_row):
    d = dest_root / case
    d.mkdir(parents=True, exist_ok=True)
    for f in ["manifest.json", "L.gcsl", "build.txt"]:
        shutil.copyfile(RUNS1 / "cases" / case / f, d / f)
    shutil.copyfile(RUNSV2 / "cases" / case / "drops-v2.json", d / "drops-v2.json")
    tmp = RUNS / "export" / case
    tmp.mkdir(parents=True, exist_ok=True)
    p = call([BIN(args), "dictionary-v2", "drop", "--dictionary", RUNSV2 / "dictionary-v21.json",
              "--drops", d / "drops-v2.json", "--out", tmp / "dictionary.json"])
    assert p.returncode == 0, p.stderr
    # The v2.1 per-case dictionary hash (global v2.1 dictionary minus the v2 drops).
    drops = load(d / "drops-v2.json")
    write(d / "drops-v2.json", {"dropped_for_leakage": drops["dropped_for_leakage"], "sha256": sha(tmp / "dictionary.json")})
    p = call([BIN(args), "synth", "donor-filter", "--corpus", OUT / "inputs/corpus.json",
              "--withheld", RUNS1 / "withheld" / case / "A.gcsl", "--out", tmp / "donor-filter.json"])
    assert p.returncode == 0, p.stderr
    df = load(tmp / "donor-filter.json")
    write(d / "donor-drops.json", {"threshold": df["threshold"], "dropped": df["dropped"]})
    (d / "party").write_text(party(case) + "\n")
    return {"case": case, "class": cls_row["class"], "stratum": cls_row["stratum"],
            "A_over_L": cls_row["A"] / cls_row["L"], "manifest_sha256": sha(d / "manifest.json"),
            "L_sha256": sha(d / "L.gcsl"), "dictionary_sha256": load(d / "drops-v2.json")["sha256"],
            "donor_drops": len(df["dropped"]), "donor_max_similarity": df["max_similarity"]}


def export(args):
    cls = {r["case"]: r for r in load(STAGE1 / "classification.json")["rows"]}
    dest = Path(args.dest)
    rows = [export_case(args, c, dest / "inputs", cls[c]) for c in args.cases]
    shutil.copyfile(RUNSV2 / "dictionary-v21.json", dest / "inputs/dictionary-v21.json")
    write(dest / "matrix.json", args.cases)
    write(dest / "cases.json", rows)
    print("exported", len(rows))


def pool_rows():
    cls = load(STAGE1 / "classification.json")
    s1 = {o["case"] for o in load(STAGE1 / "draw.json")["order"]}
    v2 = {o["case"] if isinstance(o, dict) else o for o in load(V2 / "draw.json")["order"]}
    dev_parties = {party(c) for c in s1 | v2}
    return [r for r in cls["rows"] if r["kind"] == "database" and r["class"] in ("informative", "control")
            and r["case"] not in s1 and r["case"] not in v2 and party(r["case"]) not in DEV_TEAMS
            and party(r["case"]) not in dev_parties]


def draw(args):
    if (OUT / "draw.json").exists():
        raise RuntimeError("the draw is frozen")
    rng = random.Random(DRAW_SEED)
    rows = pool_rows()
    inf = sorted((r for r in rows if r["class"] == "informative"), key=lambda r: r["case"])
    ctl = sorted((r["case"] for r in rows if r["class"] == "control"))
    al = lambda r: r["A"] / r["L"]
    hard = [r["case"] for r in inf if al(r) >= 2]
    mid = [r["case"] for r in inf if 1.25 <= al(r) < 2]
    easy = [r["case"] for r in inf if al(r) < 1.25]
    strata = {"primary (A/L>=2)": hard, "secondary (1.25<=A/L<2)": rng.sample(mid, 15),
              "non-inferiority (A/L<1.25)": rng.sample(easy, 12), "control": rng.sample(ctl, 8)}
    order = [c for v in strata.values() for c in v]
    rng.shuffle(order)
    args.cases, args.dest = order, str(OUT)
    export(args)
    head = call(["git", "rev-parse", "HEAD"]).stdout.strip()
    write(OUT / "draw.json", {
        "seed": DRAW_SEED, "protocol": {"path": str(PROTOCOL.relative_to(ROOT)), "sha256": sha(PROTOCOL)},
        "implementation_commit": head, "classification_sha256": sha(STAGE1 / "classification.json"),
        "pool": {"total": len(rows), "informative": len(inf), "control": len(ctl), "A/L>=2": len(hard),
                 "1.25<=A/L<2": len(mid), "A/L<1.25": len(easy),
                 "rule": "classification rows, kind database, class informative/control, not a stage-1 or v2 case, "
                         "not an alternative of the development teams, party not among the 70 development cases"},
        "strata": strata, "order": order,
        "inputs": {"dictionary_v21_sha256": sha(OUT / "inputs/dictionary-v21.json"),
                   "corpus_sha256": sha(OUT / "inputs/corpus.json")}})
    print("draw", len(order), {k: len(v) for k, v in strata.items()})


# ---------------------------------------------------------------- job
def materialize(args):
    src = Path(args.inputs_root) / args.case
    dest = Path(args.dest)
    dest.mkdir(parents=True, exist_ok=True)
    for f in ["manifest.json", "L.gcsl", "donor-drops.json", "party"]:
        shutil.copyfile(src / f, dest / f)
    p = call([BIN(args), "dictionary-v2", "drop", "--dictionary", Path(args.inputs_root) / "dictionary-v21.json",
              "--drops", src / "drops-v2.json", "--out", dest / "dictionary.json"])
    assert p.returncode == 0, p.stderr
    assert sha(dest / "dictionary.json") == load(src / "drops-v2.json")["sha256"], "dictionary hash mismatch"
    shutil.copyfile(Path(args.corpus), dest / "corpus.json")
    print("materialized", args.case)


def arm_flags(arm, inputs):
    base = ["--heldout-v2", "--algorithm", "anneal", "--selection-block", SELECTION_SAMPLES]
    if arm == "V21S":
        return base
    mode = {"SYN": "syn", "XO": "xo"}[arm]
    return base + ["--cycle-synthesis", mode, "--corpus", inputs / "corpus.json", "--donor-drops",
                   inputs / "donor-drops.json", "--party", (inputs / "party").read_text().strip(),
                   "--lower-at", LOWER_AT]


def search(binary, inputs, dest, arm, seed, cap, lock, stats):
    dest.mkdir(parents=True, exist_ok=True)
    log = dest / f"strace-{cap}.log"
    cmd = ["strace", "-f", "--seccomp-bpf", "-qq", "-e", "trace=open,openat,execve", "-o", log,
           binary, "search", "--manifest", inputs / "manifest.json", "--baseline", inputs / "L.gcsl",
           "--dictionary", inputs / "dictionary.json", "--fixed-er", "--seed", seed, "--seconds", cap,
           "--out", dest, "--audit", AUDIT_SAMPLES] + arm_flags(arm, inputs)
    start = time.monotonic()
    p = call(cmd)
    wall = time.monotonic() - start
    paths = opened_paths(log)
    log.unlink()
    leaked = [x for x in paths if any(f in x for f in FORBIDDEN)]
    write(dest / f"opened-{cap}.json", {"paths": paths, "forbidden_hits": leaked})
    if leaked:
        raise RuntimeError(f"search opened forbidden paths: {leaked}")
    if p.returncode:
        raise RuntimeError(p.stderr.strip()[-800:])
    rep = load(dest / "report.json")
    write(dest / f"report-{cap}.json", rep)
    state = load(dest / "checkpoint.json")
    write(dest / f"selected-{cap}.json", {"key": rep["provisional_best_key"], "estimate": state["best_hint"],
                                          "candidate": state["best"], "selection_block": rep.get("selection_block")})
    sel = next(a for a in rep["audits"] if a["key"] == rep["provisional_best_key"])
    shutil.copyfile(dest / f"candidate-{sel['index']}.gcsl", dest / f"selected-{cap}.gcsl")
    seed_a = rep["audit_seed"]
    write(dest / f"audit-{cap}.json", {"audit_seed": seed_a, "baseline_key": rep["audits"][0]["key"],
          "per_fight": {a["key"]: journal(dest, a["key"], seed_a, AUDIT_SAMPLES) for a in rep["audits"] if "joint" in a}})
    with lock:
        stats.append({"arm": arm, "seed": seed, "cap": cap, "wall": wall, "discovery_cpu": rep["discovery_cpu_seconds"],
                      "audit_cpu": rep["audit_cpu_seconds"],
                      "selection_cpu": (rep.get("selection_block") or {}).get("cpu_seconds"),
                      "decisions": rep["completed_decisions"]})
    print(f"{arm} seed {seed} cap {cap}: L={rep['baseline']['mean']:.0f} S={sel.get('joint', {}).get('mean', float('nan')):.0f} "
          f"cpu={rep['discovery_cpu_seconds']:.1f} decisions={rep['completed_decisions']} wall={wall:.0f}", flush=True)


def job(args):
    caps = [round(c * args.scale, 2) if args.scale != 1.0 else c for c in CAPS]
    inputs = Path(args.inputs).resolve()
    out = Path(args.out).resolve()
    binary = BIN(args)
    out.mkdir(parents=True, exist_ok=True)
    started, t0 = time.time(), os.times()
    lock, stats, errors = threading.Lock(), [], []
    arms = args.arms.split(",")

    def chain(arm, seed):
        try:
            for cap in caps:
                search(binary, inputs, out / arm / str(seed), arm, seed, cap, lock, stats)
        except Exception as exc:
            with lock:
                errors.append({"arm": arm, "seed": seed, "error": str(exc)[-800:]})
            print("failed:", arm, seed, str(exc)[-300:], flush=True)

    with ThreadPoolExecutor(max_workers=args.workers) as pool:
        list(pool.map(lambda a: chain(*a), [(arm, seed) for seed in STARTS for arm in arms]))
    t1 = os.times()
    for d in out.glob("*/*"):
        for name in ["evaluations.jsonl", "proposals.jsonl", "checkpoint.json", "report.json"]:
            if (d / name).exists():
                (d / name).unlink()
        for f in d.glob("candidate-*.gcsl"):
            f.unlink()
        for f in d.glob("worker-*"):
            f.unlink() if f.is_file() else shutil.rmtree(f)
    write(out / "job.json", {"case": args.case, "arms": arms, "caps": caps, "environment": environment(),
                             "workers": args.workers, "wall_seconds": time.time() - started,
                             "children_cpu_seconds": (t1.children_user + t1.children_system) - (t0.children_user + t0.children_system),
                             "searches": stats, "errors": errors, "binary_sha256": sha(binary),
                             "inputs": {f.name: sha(f) for f in sorted(inputs.iterdir()) if f.is_file()}})
    print("job done", args.case, f"{time.time() - started:.0f}s wall", len(errors), "errors", flush=True)


# ---------------------------------------------------------------- confirm
def measure(binary, manifest, text, seed, work, n=AUDIT_SAMPLES):
    work.mkdir(parents=True, exist_ok=True)
    rot = work / "rotation.gcsl"
    rot.write_text(text)
    p = subprocess.run([str(binary), "measure", "--manifest", str(manifest), "--rotation", str(rot), "--seed", str(seed),
                        "--samples", str(n), "--out", str(work / "eval")], text=True, capture_output=True)
    if p.returncode:
        return {"status": "failed", "error": p.stderr.strip()[-300:]}
    r = json.loads(p.stdout)
    return {"status": "complete", "per_fight": r["per_fight_dps"], "mean": r["stats"]["mean"], "cpu": r["cpu_seconds"]}


def confirm(args):
    rundir = Path(args.rundir)
    results = Path(args.results)
    binary = BIN(args)
    order = load(rundir / "matrix.json")
    meta = {r["case"]: r for r in load(rundir / "cases.json")}

    def one(case):
        inputs = rundir / "inputs" / case
        L_text = (inputs / "L.gcsl").read_text()
        A_text = (RUNS1 / "withheld" / case / "A.gcsl").read_text()
        work = RUNS / "confirm" / rundir.name / case
        cache = {}
        out = {"case": case, **meta[case], "job": load(results / case / "job.json"), "arms": {},
               "L_reproduction_failures": []}
        job_caps = out["job"].get("caps", CAPS)
        for arm in ARMS:
            out["arms"][arm] = {}
            for nominal, cap in zip(CAPS, job_caps):
                cap = int(cap) if float(cap).is_integer() else cap
                starts = {}
                for seed in STARTS:
                    d = results / case / arm / str(seed)
                    if not (d / f"report-{cap}.json").exists():
                        starts[seed] = {"status": "missing"}
                        continue
                    rep, aud, sel = load(d / f"report-{cap}.json"), load(d / f"audit-{cap}.json"), load(d / f"selected-{cap}.json")
                    sa = aud["audit_seed"]
                    for nm, txt in (("L", L_text), ("A", A_text)):
                        if (nm, sa) not in cache:
                            cache[(nm, sa)] = measure(binary, inputs / "manifest.json", txt, sa, work / f"{nm}-{sa}")
                    Lf = aud["per_fight"][aud["baseline_key"]]
                    if cache[("L", sa)].get("per_fight") != Lf:
                        out["L_reproduction_failures"].append([arm, cap, seed])
                    Sf = aud["per_fight"].get(sel["key"])
                    failed = Sf is None or None in Sf
                    if failed:
                        Sf = Lf
                    A = cache[("A", sa)]
                    sb = sel.get("selection_block") or {}
                    c = {"status": "complete", "audit_seed": sa, "estimate": sel["estimate"],
                         "selection": (sb.get("stats") or {}).get("mean"), "selection_cpu": sb.get("cpu_seconds"),
                         "discovery_cpu": rep["discovery_cpu_seconds"], "audit_cpu": rep["audit_cpu_seconds"],
                         "decisions": rep["completed_decisions"], "timeouts": rep["timeouts_this_invocation"],
                         "selected_key": sel["key"], "S_is_L": sel["key"] == aud["baseline_key"],
                         "audit_failure": failed, "L": statistics.fmean(Lf), "S": statistics.fmean(Sf),
                         "S_per_fight": Sf, "A": A.get("mean"), "cycle_synthesis": rep.get("cycle_synthesis"),
                         "selected_role": sel["candidate"].get("role"),
                         "selected_source": (sel["candidate"]["program"].get("Source") or "")}
                    if A["status"] == "complete":
                        c["A_minus_L"], _ = paired(A["per_fight"], Lf)
                        c["S_minus_L"], c["S_minus_L_se"] = paired(Sf, Lf)
                        c["r"] = c["S_minus_L"] / c["A_minus_L"] if c["A_minus_L"] else None
                        c["S_over_A"] = c["S"] / c["A"]
                        c["regression"] = c["S_minus_L"] < -2 * c["S_minus_L_se"]
                    starts[seed] = c
                done = {s: v for s, v in starts.items() if v.get("status") == "complete"}
                entry = {"starts": starts}
                if len(done) == len(STARTS):
                    key = lambda s: (done[s]["selection"] if done[s]["selection"] is not None else -math.inf, done[s]["estimate"])
                    entry["selected_start"] = max(done, key=key)
                out["arms"][arm][nominal] = entry
        out["measure_cpu"] = sum(v.get("cpu", 0) for v in cache.values())
        print(case, {a: round(v[384]["starts"][v[384]["selected_start"]].get("S_over_A", float("nan")), 3)
                     for a, v in out["arms"].items() if v.get(384, {}).get("selected_start")}, flush=True)
        return out

    with ThreadPoolExecutor(max_workers=args.workers) as pool:
        rows = list(pool.map(one, [c for c in order if (results / c / "job.json").exists()]))
    write(Path(args.dest), {"cases": rows})
    print("confirmed", len(rows))


# ---------------------------------------------------------------- report
def mcnemar(b, c):
    """Exact two-sided McNemar on discordant counts b, c."""
    n = b + c
    if n == 0:
        return 1.0
    k = min(b, c)
    p = sum(math.comb(n, i) for i in range(k + 1)) / 2 ** n
    return min(1.0, 2 * p)


def reading(median_d, p, x, y):
    if median_d > 0 and p < 0.05:
        return f"{x} beats {y}"
    if median_d < 0 and p < 0.05:
        return f"{y} beats {x}"
    return "no detectable difference at this sample size"


def report(args):
    ex = load(args.experiments)["cases"]
    draw = load(args.draw) if args.draw else None
    stratum_of = {}
    if draw:
        for k, v in draw["strata"].items():
            for c in v:
                stratum_of[c] = k
    else:
        for c in ex:
            stratum_of[c["case"]] = ("primary (A/L>=2)" if c["A_over_L"] >= 2 else "secondary (1.25<=A/L<2)"
                                     if c["A_over_L"] >= 1.25 else "non-inferiority (A/L<1.25)") if c["class"] == "informative" else "control"
    def sel(c, arm, cap=384):
        e = c["arms"][arm].get(cap, c["arms"][arm].get(str(cap), {}))
        s = e.get("selected_start")
        return (e["starts"].get(str(s)) or e["starts"].get(s)) if s is not None else None
    def starts(c, arm, cap=384):
        e = c["arms"][arm].get(cap, c["arms"][arm].get(str(cap), {}))
        return [v for v in e.get("starts", {}).values() if v.get("status") == "complete"]
    out = {"strata": {}, "comparisons": {}, "per_case": []}
    lines = ["# Cycle-synthesis pilot tables", ""]
    for st in ["primary (A/L>=2)", "secondary (1.25<=A/L<2)", "non-inferiority (A/L<1.25)", "control"]:
        cs = [c for c in ex if stratum_of.get(c["case"]) == st]
        if not cs:
            continue
        lines += [f"## {st} (n = {len(cs)})", "", "| Arm @384 | median S/A | S/A < 0.90 | median r | regressions | selected audit failures | selected from synthesized family |",
                  "|---|---:|---:|---:|---:|---:|---:|"]
        out["strata"][st] = {}
        for arm in ARMS:
            v = [sel(c, arm) for c in cs]
            v = [x for x in v if x and x.get("S_over_A") is not None]
            sa = [x["S_over_A"] for x in v]
            rr = [x["r"] for x in v if x.get("r") is not None]
            synth = sum(1 for x in v if (x.get("selected_role") or "").startswith("synth"))
            rec = {"n": len(v), "median_S_over_A": statistics.median(sa) if sa else None,
                   "below_0.90": sum(s < 0.9 for s in sa), "median_r": statistics.median(rr) if rr else None,
                   "regressions": sum(bool(x.get("regression")) for x in v), "audit_failures": sum(x["audit_failure"] for x in v),
                   "selected_from_synth_family": synth}
            out["strata"][st][arm] = rec
            f = lambda z: "–" if z is None else f"{z:.3f}"
            lines.append(f"| {arm} | {f(rec['median_S_over_A'])} | {rec['below_0.90']} | {f(rec['median_r'])} | {rec['regressions']} | {rec['audit_failures']} | {synth} |")
        lines.append("")
        lines += ["| Comparison | median d (% of A) | x better | p | reading | start-averaged median d, p |", "|---|---:|---:|---:|---|---|"]
        for x, y in [("SYN", "V21S"), ("XO", "V21S"), ("SYN", "XO")]:
            d, da = [], []
            for c in cs:
                a, b = sel(c, x), sel(c, y)
                if a and b and a.get("A") and b.get("A"):
                    d.append((a["S"] - b["S"]) / a["A"])
                    sx, sy = {s["audit_seed"]: s for s in starts(c, x)}, {s["audit_seed"]: s for s in starts(c, y)}
                    common = [k for k in sx if k in sy]
                    if common:
                        da.append(statistics.fmean((sx[k]["S"] - sy[k]["S"]) / sx[k]["A"] for k in common))
            if not d:
                continue
            w, wa = wilcoxon(d), wilcoxon(da)
            md, mda = statistics.median(d), statistics.median(da)
            rec = {"n": len(d), "median_d": md, "mean_d": statistics.fmean(d), "x_better": sum(v > 0 for v in d),
                   "p": w["p"], "reading": reading(md, w["p"], x, y), "start_averaged_median_d": mda, "start_averaged_p": wa["p"],
                   "start_averaged_reading": reading(mda, wa["p"], x, y)}
            out["comparisons"].setdefault(st, {})[f"{x} vs {y}"] = rec
            lines.append(f"| {x} vs {y} | {100 * md:+.2f}% | {rec['x_better']}/{len(d)} | {w['p']:.3g} | {rec['reading']} | {100 * mda:+.2f}%, {wa['p']:.3g} |")
        # tail McNemar SYN vs V21S
        b = sum(1 for c in cs if sel(c, "SYN") and sel(c, "V21S") and sel(c, "SYN")["S_over_A"] >= 0.9 > sel(c, "V21S")["S_over_A"])
        cc = sum(1 for c in cs if sel(c, "SYN") and sel(c, "V21S") and sel(c, "V21S")["S_over_A"] >= 0.9 > sel(c, "SYN")["S_over_A"])
        out["comparisons"].setdefault(st, {})["tail SYN vs V21S"] = {"SYN_only_at_or_above_0.90": b, "V21S_only": cc, "p": mcnemar(b, cc)}
        lines += ["", f"Tail (S/A ≥ 0.90): SYN only {b}, V21S only {cc}, exact McNemar p = {mcnemar(b, cc):.3g}.", ""]
    lines += ["## Per case (selected start at 384 CPU-s; S/A)", "", "| Case | stratum | A/L | V21S | SYN | XO | SYN selected family |", "|---|---|---:|---:|---:|---:|---|"]
    for c in sorted(ex, key=lambda c: (stratum_of.get(c["case"], ""), c["case"])):
        row = {"case": c["case"], "stratum": stratum_of.get(c["case"]), "A_over_L": c["A_over_L"]}
        cells = []
        for arm in ARMS:
            s = sel(c, arm)
            row[arm] = s and {k: s.get(k) for k in ("S", "A", "L", "S_over_A", "r", "audit_failure", "selected_role", "selection")}
            cells.append(f"{s['S_over_A']:.3f}" if s and s.get("S_over_A") is not None else "–")
        s = sel(c, "SYN")
        out["per_case"].append(row)
        lines.append(f"| {c['case']} | {row['stratum']} | {c['A_over_L']:.2f} | {' | '.join(cells)} | {(s or {}).get('selected_role', '')} |")
    # budget curve, primary stratum
    lines += ["", "## Budget curve (median S/A of the selected start)", "", "| Stratum | Arm | 96 | 192 | 384 |", "|---|---|---:|---:|---:|"]
    for st in out["strata"]:
        cs = [c for c in ex if stratum_of.get(c["case"]) == st]
        for arm in ARMS:
            vals = []
            for cap in CAPS:
                v = [sel(c, arm, cap) for c in cs]
                v = [x["S_over_A"] for x in v if x and x.get("S_over_A") is not None]
                vals.append(f"{statistics.median(v):.3f}" if v else "–")
            lines.append(f"| {st} | {arm} | {' | '.join(vals)} |")
    write(Path(args.dest) / "summary.json", out)
    (Path(args.dest) / "tables.md").write_text("\n".join(lines) + "\n")
    print("\n".join(lines[:60]))


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("phase", choices=["export", "draw", "materialize", "job", "confirm", "report"])
    ap.add_argument("--cases", nargs="*", default=[])
    ap.add_argument("--case", default="")
    ap.add_argument("--inputs", default="")
    ap.add_argument("--inputs-root", default="")
    ap.add_argument("--dest", default="")
    ap.add_argument("--out", default="")
    ap.add_argument("--rundir", default="")
    ap.add_argument("--results", default="")
    ap.add_argument("--experiments", default="")
    ap.add_argument("--draw", default="")
    ap.add_argument("--arms", default=",".join(ARMS))
    ap.add_argument("--workers", type=int, default=4)
    ap.add_argument("--scale", type=float, default=1.0)
    ap.add_argument("--bin", default=str(ROOT / "target/release/gso"))
    ap.add_argument("--corpus", default=str(OUT / "inputs/corpus.json"))
    args = ap.parse_args()
    {"export": export, "draw": draw, "materialize": materialize, "job": job, "confirm": confirm, "report": report}[args.phase](args)


if __name__ == "__main__":
    main()
