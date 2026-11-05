"""Held-out recovery pilot, stage 1 (docs/heldout-recovery-pilot.md).

Phases, in order:
  export     cases from Gscouter teams/*/alt/* -> fixed-build inputs; exclusions
  dictionary block dictionary from the main library rotations; per-case filter
  measure    L and A on B under the common conditions; classification
  draw       seeded stratified stage-1 draw (commit before any search)
  run        searches for the drawn cases (3 starts, checkpoints)
  confirm    A on the audit seeds after selections are frozen; r and reading

The search reads only runs/heldout/cases/<case>/{manifest.json,L.gcsl,dictionary.json}.
Withheld rotations A are copied to runs/heldout/withheld/ and are read only by
the dictionary leakage filter, the classification measurement and `confirm`.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import random
import re
import shutil
import statistics
import subprocess
import time

import yaml

ROOT = Path(__file__).resolve().parents[1]
BIN = ROOT / "target/release/gso"
RUNS = ROOT / "runs/heldout"
OUT = ROOT / "reports/heldout-recovery-2026-11-05"
GSCOUTER_COMMIT = "6e0e82205"
PROTOCOL = ROOT / "docs/heldout-recovery-pilot.md"
CLASSIFY_SEED = 8_000_000_000_000
CLASSIFY_SAMPLES = 128
AUDIT_SAMPLES = 128
COLLAPSE_DPS = 5000.0
DRAW_SEED = 20261105
DRAW_INFORMATIVE = 24
DRAW_CONTROLS = 6
DRAW_CONTINUED = 6
STARTS = [11, 37, 91]
CAPS = [36, 108, 192]
CONTINUATION = 576
WORKERS = 4


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n")


def load(path):
    return json.loads(Path(path).read_text())


def call(command, **kw):
    return subprocess.run(list(map(str, command)), cwd=ROOT, text=True, capture_output=True, **kw)


def add_params(build, key, params):
    """Gscouter config.AddParams: a char line that already has params is left alone."""
    if not params:
        return build
    pattern = re.compile(r"(?m)^(" + re.escape(key) + r" char [^;+]*?)\s*;")
    return pattern.sub(lambda m: m.group(1) + " +params=[" + params + "];", build)


def unwrap(text):
    """Remove the repetition wrapper Program::baseline adds, for S features."""
    text = re.sub(r"while\s+1\s*\{\s*let\s+optimizer_cycle_start_*\s*=\s*f\s*\(\s*\)\s*;", "", text, count=1)
    return re.sub(r"if\s+f\s*\(\s*\)\s*==\s*optimizer_cycle_start_*\s*\{\s*wait\s*\(\s*1\s*\)\s*;\s*\}", "", text)


def features(text):
    """Rotation features reported per case; `frame` = hold or while, the draw stratum."""
    hold = "hold=" in text.replace(" ", "")
    loop = re.search(r"\bwhile\b", text) is not None
    delay = re.search(r"\b(wait|delay|sleep)\s*\(", text) is not None
    return {"hold": hold, "while": loop, "wait_or_delay": delay, "frame": hold or loop}


# ---------------------------------------------------------------- export
def export(args):
    gs = Path(args.gscouter)
    head = call(["git", "-C", gs, "rev-parse", "HEAD"]).stdout.strip()
    assert head.startswith(GSCOUTER_COMMIT), f"Gscouter must be at {GSCOUTER_COMMIT}, is {head}"
    if (RUNS / "cases.json").exists():
        raise RuntimeError("cases already exported; use a fresh runs/heldout")
    cases, excluded = [], []
    for alt in sorted(gs.glob("teams/*/alt/*")):
        team = alt.parent.parent
        case = f"{team.name}--{alt.name}"
        team_yaml = yaml.safe_load((team / "team.yaml").read_text())
        alt_yaml = yaml.safe_load((alt / "team.yaml").read_text())
        row = {"case": case, "team": team.name, "alt": alt.name,
               "kind": "variant" if "variant" in alt_yaml else "database",
               "inputs": {"B": {"path": str(alt.relative_to(gs) / "build.txt"), "sha256": sha(alt / "build.txt")},
                          "L": {"path": str(team.relative_to(gs) / "rotation.gcsl"), "sha256": sha(team / "rotation.gcsl")},
                          "A": {"path": str(alt.relative_to(gs) / "rotation.gcsl"), "sha256": sha(alt / "rotation.gcsl")}}}
        team_params = team_yaml.get("params") or {}
        alt_params = alt_yaml.get("params") or {}
        # Gscouter attaches params to each rotation. One common condition is
        # needed for L, A and the search, so B is the alternative's whole
        # entry: its build.txt with its own team.yaml params, for all three.
        row["params"] = {"applied (alternative's)": alt_params, "library team's (not applied)": team_params}
        build = (alt / "build.txt").read_text()
        for member, params in alt_params.items():
            build = add_params(build, member, params)
        dest = RUNS / "cases" / case
        dest.mkdir(parents=True)
        (dest / "build.txt").write_text(build)
        shutil.copyfile(team / "rotation.gcsl", dest / "L.gcsl")
        withheld = RUNS / "withheld" / case
        withheld.mkdir(parents=True)
        shutil.copyfile(alt / "rotation.gcsl", withheld / "A.gcsl")
        p = call([BIN, "heldout-manifest", "--build", dest / "build.txt", "--site-commit", head,
                  "--source", f"{row['inputs']['B']['path']}={row['inputs']['B']['sha256']}",
                  "--out", dest / "manifest.json"])
        if p.returncode:
            row["reason"] = "build: " + (p.stderr.strip() or p.stdout.strip())
            excluded.append(row)
            continue
        row["manifest_sha256"] = sha(dest / "manifest.json")
        row["features"] = {"L": features((team / "rotation.gcsl").read_text()),
                           "A": features((alt / "rotation.gcsl").read_text())}
        cases.append(row)
    write(RUNS / "cases.json", {"gscouter_commit": head, "alternatives": len(cases) + len(excluded),
                                "exported": cases, "excluded": excluded})
    print(f"exported {len(cases)}, excluded {len(excluded)}")


# ---------------------------------------------------------------- dictionary
def dictionary(args):
    gs = Path(args.gscouter)
    meta = load(RUNS / "cases.json")
    rotations = sorted(gs.glob("teams/*/rotation.gcsl"))
    assert not any("/alt/" in str(p) for p in rotations)
    listing = RUNS / "library-rotations.tsv"
    listing.write_text("".join(f"{p.parent.name}\t{p}\n" for p in rotations))
    p = call([BIN, "dictionary", "build", "--list", listing, "--source",
              f"Gscouter {meta['gscouter_commit']} teams/*/rotation.gcsl ({len(rotations)} main rotations, no alt/)",
              "--out", RUNS / "dictionary.json"])
    assert p.returncode == 0, p.stderr
    for row in meta["exported"]:
        case = row["case"]
        p = call([BIN, "dictionary", "filter", "--dictionary", RUNS / "dictionary.json",
                  "--withheld", RUNS / "withheld" / case / "A.gcsl",
                  "--out", RUNS / "cases" / case / "dictionary.json"])
        assert p.returncode == 0, p.stderr
        row["dictionary"] = {**json.loads(p.stdout), "sha256": sha(RUNS / "cases" / case / "dictionary.json")}
    meta["dictionary"] = {"rotations": len(rotations), "sha256": sha(RUNS / "dictionary.json")}
    write(RUNS / "cases.json", meta)
    d = load(RUNS / "dictionary.json")
    print(f"dictionary: {sum(map(len, d['blocks'].values()))} blocks for {len(d['blocks'])} characters,"
          f" {len(d['unparsed'])} unparsed rotations")


# ---------------------------------------------------------------- measure
def measure_one(manifest, rotation, seed, samples, out):
    start = time.monotonic()
    p = call([BIN, "measure", "--manifest", manifest, "--rotation", rotation, "--seed", seed,
              "--samples", samples, "--out", out])
    if p.returncode:
        return {"status": "failed", "error": p.stderr.strip()[-500:], "wall_seconds": time.monotonic() - start}
    return {"status": "complete", **json.loads(p.stdout), "wall_seconds": time.monotonic() - start}


def paired(a, b):
    d = [x - y for x, y in zip(a, b, strict=True)]
    m = sum(d) / len(d)
    se = (sum((x - m) ** 2 for x in d) / (len(d) * (len(d) - 1))) ** 0.5
    return m, se


def measure(args):
    meta = load(RUNS / "cases.json")

    def job(row):
        case = row["case"]
        dest = RUNS / "measure" / case
        result = {}
        for name, rotation in [("L", RUNS / "cases" / case / "L.gcsl"),
                               ("A", RUNS / "withheld" / case / "A.gcsl")]:
            result[name] = measure_one(RUNS / "cases" / case / "manifest.json", rotation,
                                       CLASSIFY_SEED, CLASSIFY_SAMPLES, dest / name)
        print(case, {k: v.get("stats", {}).get("mean", v["status"]) for k, v in result.items()}, flush=True)
        return case, result

    with ThreadPoolExecutor(max_workers=WORKERS) as pool:
        results = dict(pool.map(job, meta["exported"]))
    rows = []
    for row in meta["exported"]:
        r = results[row["case"]]
        entry = {"case": row["case"], "kind": row["kind"], "stratum": "frame" if row["features"]["A"]["frame"] else "other",
                 "features": row["features"],
                 "cpu_seconds": sum(v.get("cpu_seconds", 0.0) for v in r.values())}
        failed = [k for k in ("L", "A") if r[k]["status"] != "complete"]
        if failed:
            entry["class"] = "excluded"
            entry["reason"] = "; ".join(f"{k} cannot run: {r[k]['error']}" for k in failed)
        else:
            L, A = r["L"]["stats"]["mean"], r["A"]["stats"]["mean"]
            d, se = paired(r["A"]["per_fight_dps"], r["L"]["per_fight_dps"])
            entry.update({"L": L, "A": A, "L_se": r["L"]["stats"]["se"], "A_se": r["A"]["stats"]["se"],
                          "A_minus_L": d, "paired_se": se, "relative": d / L if L else None})
            if min(L, A) < COLLAPSE_DPS:
                entry["class"] = "excluded"
                entry["reason"] = f"collapse: {'L' if L < COLLAPSE_DPS else 'A'} below {COLLAPSE_DPS:.0f} DPS"
            elif d >= 0.05 * L and d >= 4 * se:
                entry["class"] = "informative"
            else:
                entry["class"] = "control"
        rows.append(entry)
    write(RUNS / "classification.json", {
        "conditions": "450 s fight, first 150 s excluded, one level-100 target, 10% resistance, swap delay 12, "
                      "periodic energy every 480-720 frames; builds as in alt/<id>/build.txt with the team's params",
        "seed": CLASSIFY_SEED, "samples": CLASSIFY_SAMPLES,
        "rule": "informative: A-L >= 5% of L and >= 4 paired SE; excluded: cannot run or L/A < 5000 DPS; control: rest",
        "rows": rows, "raw": results})
    counts = {c: sum(r["class"] == c for r in rows) for c in ("informative", "control", "excluded")}
    print(counts)


# ---------------------------------------------------------------- draw
def draw(args):
    if (OUT / "draw.json").exists():
        raise RuntimeError("the draw is frozen")
    cls = load(RUNS / "classification.json")
    rng = random.Random(DRAW_SEED)
    rows = cls["rows"]

    def stratified(pool, k):
        strata = {s: sorted(r["case"] for r in pool if r["stratum"] == s) for s in ("frame", "other")}
        total = sum(map(len, strata.values()))
        k = min(k, total)
        # Largest-remainder proportional allocation, then a seeded sample.
        quota = {s: k * len(v) / total for s, v in strata.items()}
        alloc = {s: int(q) for s, q in quota.items()}
        for s in sorted(quota, key=lambda s: (-(quota[s] - alloc[s]), s))[:k - sum(alloc.values())]:
            alloc[s] += 1
        return {s: rng.sample(strata[s], alloc[s]) for s in strata}, alloc

    informative, inf_alloc = stratified([r for r in rows if r["class"] == "informative"], DRAW_INFORMATIVE)
    controls, ctl_alloc = stratified([r for r in rows if r["class"] == "control"], DRAW_CONTROLS)
    # Continuation subset: informative cases, split evenly across strata.
    continued = []
    for s in ("frame", "other"):
        continued += informative[s][:DRAW_CONTINUED // 2]
    for s in ("frame", "other"):
        for c in informative[s][DRAW_CONTINUED // 2:]:
            if len(continued) < DRAW_CONTINUED:
                continued.append(c)
    order = [c for s in informative.values() for c in s] + [c for s in controls.values() for c in s]
    rng.shuffle(order)
    by_case = {r["case"]: r for r in rows}
    draw = {"seed": DRAW_SEED, "classification_sha256": sha(RUNS / "classification.json"),
            "cases_sha256": sha(RUNS / "cases.json"), "protocol_sha256": sha(PROTOCOL),
            "method": "seeded random.Random(20261105); stratified by whether A uses hold= or while "
                      "(largest-remainder proportional allocation); run order is a seeded shuffle; "
                      "a smaller stage takes a prefix of this order",
            "allocation": {"informative": inf_alloc, "control": ctl_alloc},
            "order": [{"case": c, "class": by_case[c]["class"], "stratum": by_case[c]["stratum"],
                       "continue_to_576": c in continued} for c in order]}
    write(OUT / "draw.json", draw)
    for name in ["cases.json", "classification.json", "dictionary.json"]:
        shutil.copyfile(RUNS / name, OUT / name)
    inputs = OUT / "inputs"
    for c in order:
        (inputs / c).mkdir(parents=True, exist_ok=True)
        for f in ["manifest.json", "L.gcsl", "build.txt"]:
            shutil.copyfile(RUNS / "cases" / c / f, inputs / c / f)
        # The per-case dictionary is the global one minus these blocks
        # (`gso dictionary filter` reproduces it); only the difference is kept.
        per_case = RUNS / "cases" / c / "dictionary.json"
        write(inputs / c / "dictionary-filter.json", {"sha256": sha(per_case),
              "dropped_for_leakage": load(per_case)["dropped_for_leakage"]})
    write(OUT / "protocol.json", {"path": "docs/heldout-recovery-pilot.md", "sha256": sha(PROTOCOL)})
    print(json.dumps(draw["allocation"]), len(order), "continued:", continued)


# ---------------------------------------------------------------- run
FORBIDDEN = ("/alt/", "withheld", "/Gscouter/")


def opened_paths(log):
    paths = set()
    for line in Path(log).read_text(errors="replace").splitlines():
        m = re.search(r'(?:openat|open|execve)\((?:AT_FDCWD, )?"([^"]*)"', line)
        if m:
            paths.add(m.group(1))
    return sorted(paths)


def search(case, seed, cap, dest, label):
    inputs = RUNS / "cases" / case
    log = dest / f"strace-{cap}.log"
    dest.mkdir(parents=True, exist_ok=True)
    command = ["strace", "-f", "--seccomp-bpf", "-qq", "-e", "trace=open,openat,execve", "-o", log,
               BIN, "search", "--manifest", inputs / "manifest.json", "--baseline", inputs / "L.gcsl",
               "--dictionary", inputs / "dictionary.json", "--algorithm", "greedy", "--fixed-er",
               "--seed", seed, "--seconds", cap, "--out", dest, "--audit", AUDIT_SAMPLES]
    p = call(command)
    paths = opened_paths(log)
    log.unlink()
    leaked = [p for p in paths if any(f in p for f in FORBIDDEN)]
    write(dest / f"opened-{cap}.json", {"paths": paths, "forbidden_hits": leaked})
    if leaked:
        raise RuntimeError(f"search opened forbidden paths: {leaked}")
    if p.returncode:
        raise RuntimeError(p.stderr.strip()[-800:])
    report = load(dest / "report.json")
    write(dest / f"report-{cap}.json", report)
    # Freeze the pre-audit selection's source for this checkpoint.
    state = load(dest / "checkpoint.json")
    write(dest / f"selected-{cap}.json", {"key": report["provisional_best_key"],
                                          "estimate": state["best_hint"], "candidate": state["best"]})
    sel = next(a for a in report["audits"] if a["key"] == report["provisional_best_key"])
    shutil.copyfile(dest / f"candidate-{sel['index']}.gcsl", dest / f"selected-{cap}.gcsl")
    return report


def trial(job):
    case, seed, caps, label = job
    dest = RUNS / label / case / str(seed)
    row = {"case": case, "seed": seed, "caps": {}}
    try:
        for cap in caps:
            if (dest / f"report-{cap}.json").exists():
                report = load(dest / f"report-{cap}.json")
            else:
                report = search(case, seed, cap, dest, label)
            sel = next(a for a in report["audits"] if a["key"] == report["provisional_best_key"])
            row["caps"][cap] = {"discovery_cpu": report["discovery_cpu_seconds"],
                                "audit_cpu": report["audit_cpu_seconds"],
                                "estimate": report["curve"][-1]["best_search_estimate"],
                                "S": sel.get("joint", {}).get("mean"), "L": report["baseline"]["mean"]}
            print(f"{label} {case} seed {seed} cap {cap}: L={report['baseline']['mean']:.0f} "
                  f"S={row['caps'][cap]['S']:.0f} cpu={report['discovery_cpu_seconds']:.1f}", flush=True)
        row["status"] = "complete"
    except Exception as exc:
        row["status"] = "failed/unresolved"
        row["error"] = str(exc)[-800:]
        write(dest / "failure.json", {"error": str(exc)})
        print(f"{label} {case} seed {seed}: failed: {str(exc)[-300:]}", flush=True)
    return row


def run(args):
    draw = load(OUT / "draw.json")
    order = draw["order"][:args.cases] if args.cases else draw["order"]
    jobs = [(c["case"], s, CAPS, args.label) for c in order for s in STARTS]
    jobs += [(c["case"], s, CAPS + [CONTINUATION], args.label) for c in order if c["continue_to_576"] for s in STARTS]
    # The 192 s legs run first for every case; continuations reuse the checkpoints afterwards.
    first, second = jobs[:len(jobs) - sum(c["continue_to_576"] for c in order) * len(STARTS)], \
        jobs[len(jobs) - sum(c["continue_to_576"] for c in order) * len(STARTS):]
    with ThreadPoolExecutor(max_workers=WORKERS) as pool:
        rows = list(pool.map(trial, first))
        if not args.skip_continuation:
            rows += list(pool.map(trial, second))
    write(RUNS / args.label / "summary.json", rows)


# ---------------------------------------------------------------- confirm
def journal(dest, key, seed, n):
    """Per-fight DPS for one evaluation key from a run's journal."""
    found = {}
    for line in (dest / "evaluations.jsonl").read_text().splitlines():
        r = json.loads(line)
        if r["key"] != key:
            continue
        for i, sample in enumerate(r["work"]["samples"]):
            if seed <= r["seed"] + i < seed + n and "Ok" in sample:
                found[r["seed"] + i] = sample["Ok"]["dps"]
    return [found[seed + i] for i in range(n)]


def body(text):
    text = re.sub(r"\s*([;{}(),=:\[\]<>!+*/-])\s*", r" \1 ", text)
    return [line.strip() for line in re.sub(r"\s+", " ", text).replace(";", ";\n").replace("{", "{\n").splitlines() if line.strip()]


def confirm(args):
    """Measure A on each frozen checkpoint's audit seeds; compute r. Runs only
    after every search of the label has finished and its selections are saved."""
    import difflib
    draw = load(OUT / "draw.json")
    summary = load(RUNS / args.label / "summary.json")
    cls = {r["case"]: r for r in load(RUNS / "classification.json")["rows"]}
    done = {(r["case"], r["seed"]): r for r in summary}
    order = [c for c in draw["order"] if any((c["case"], s) in done for s in STARTS)]

    def job(entry):
        case = entry["case"]
        manifest = RUNS / "cases" / case / "manifest.json"
        out = {"case": case, "class": entry["class"], "stratum": entry["stratum"],
               "continued": entry["continue_to_576"], "kind": cls[case]["kind"],
               "features": cls[case]["features"], "classification": {k: cls[case].get(k) for k in
               ("L", "A", "A_minus_L", "paired_se", "relative")}, "starts": {}, "checkpoints": {},
               "measure_cpu": 0.0}
        caps = CAPS + ([CONTINUATION] if entry["continue_to_576"] else [])
        for seed in STARTS:
            row = done.get((case, seed))
            dest = RUNS / args.label / case / str(seed)
            out["starts"][seed] = {"status": row["status"] if row else "not run", "caps": {}}
            if not row:
                continue
            if row["status"] != "complete":
                out["starts"][seed]["error"] = row.get("error")
            for cap in caps:
                path = dest / f"report-{cap}.json"
                if not path.exists():
                    continue
                rep = load(path)
                sel = next(a for a in rep["audits"] if a["key"] == rep["provisional_best_key"])
                base = rep["audits"][0]
                assert base["joint_minus_original_baseline"][0] == 0
                seed_a = rep["audit_seed"]
                Lf = journal(dest, base["key"], seed_a, AUDIT_SAMPLES)
                Sf = journal(dest, sel["key"], seed_a, AUDIT_SAMPLES) if "joint" in sel else None
                Am = measure_one(manifest, RUNS / "withheld" / case / "A.gcsl", seed_a, AUDIT_SAMPLES,
                                 RUNS / f"{args.label}-confirm" / case)
                Lm = measure_one(manifest, RUNS / "cases" / case / "L.gcsl", seed_a, AUDIT_SAMPLES,
                                 RUNS / f"{args.label}-confirm" / case)
                out["measure_cpu"] += Am.get("cpu_seconds", 0) + Lm.get("cpu_seconds", 0)
                # The measurement path must reproduce the search's own L exactly.
                assert Lm["status"] == "complete" and Lm["per_fight_dps"] == Lf, case
                selected = load(dest / f"selected-{cap}.json")
                c = {"audit_seed": seed_a, "estimate": rep["curve"][-1]["best_search_estimate"],
                     "discovery_cpu": rep["discovery_cpu_seconds"], "audit_cpu": rep["audit_cpu_seconds"],
                     "decisions": rep["completed_decisions"], "L": statistics.fmean(Lf),
                     "S": statistics.fmean(Sf) if Sf else None, "S_is_L": sel["key"] == base["key"],
                     "A": statistics.fmean(Am["per_fight_dps"]) if Am["status"] == "complete" else None,
                     "S_features": features(unwrap(selected["candidate"]["program"].get("Source")
                                            or selected["candidate"]["program"].get("Library") or ""))}
                if Sf and c["A"] is not None:
                    Af = Am["per_fight_dps"]
                    c["S_minus_L"], c["S_minus_L_se"] = paired(Sf, Lf)
                    c["A_minus_L"], c["A_minus_L_se"] = paired(Af, Lf)
                    c["r"] = c["S_minus_L"] / c["A_minus_L"] if c["A_minus_L"] else None
                    c["regression"] = c["S_minus_L"] < -2 * c["S_minus_L_se"]
                out["starts"][seed]["caps"][cap] = c
        for cap in caps:
            cands = [(s, v["caps"][cap]) for s, v in out["starts"].items() if cap in v["caps"]]
            if len(cands) < len(STARTS):
                out["checkpoints"][cap] = {"status": "incomplete starts", "starts": len(cands)}
                continue
            seed, c = max(cands, key=lambda x: x[1]["estimate"])
            ss = [v["S"] for _, v in cands if v["S"] is not None]
            out["checkpoints"][cap] = {"selected_start": seed, **{k: c.get(k) for k in
                ("L", "A", "S", "r", "S_minus_L", "S_minus_L_se", "A_minus_L", "A_minus_L_se", "regression", "S_is_L")},
                "start_spread": (max(ss) - min(ss)) if ss else None,
                "cpu_per_start": [v["discovery_cpu"] for _, v in cands]}
        top = max((c for c in caps if out["checkpoints"].get(c, {}).get("selected_start")), default=None)
        if top:
            seed = out["checkpoints"][top]["selected_start"]
            dest = RUNS / args.label / case / str(seed)
            sel = load(dest / f"selected-{top}.json")["candidate"]["program"]
            text = sel.get("Source") or sel.get("Library") or ""
            base = load(dest / "checkpoint.json")["baseline"]["program"]
            btext = base.get("Source") or base.get("Library") or ""
            out["diff_against_L"] = {"checkpoint": top, "start": seed, "diff": list(difflib.unified_diff(
                body(btext), body(text), "L", "S", lineterm="", n=1))}
        print(case, {k: (round(v["r"], 3) if v.get("r") is not None else v.get("status", v.get("r")))
                     for k, v in out["checkpoints"].items()}, flush=True)
        return out

    with ThreadPoolExecutor(max_workers=WORKERS) as pool:
        rows = list(pool.map(job, order))
    write(RUNS / args.label / "experiments.json", {"label": args.label, "cases": rows})


def dev(args):
    """Development cases only (already revealed): Flins C1 and Mavuika C6R5."""
    gs = Path(args.gscouter)
    cases = {
        "flins": (ROOT / "data/flins-c1/standard.json", ROOT / "data/flins-c1/baseline.gcsl",
                  gs / "teams/flins-ineffa-columbina-sucrose"),
        "mavuika": (ROOT / "data/mavuika-c6-r5-225/manifest.json",
                    gs / "teams/mavuika-citlali-xilonen-nicole/rotation.gcsl",
                    gs / "teams/mavuika-citlali-xilonen-nicole"),
    }
    base = RUNS / "dev"
    dictionary = RUNS / "dictionary.json"
    for name, (manifest, baseline, team) in cases.items():
        if args.only and name != args.only:
            continue
        d = base / name
        d.mkdir(parents=True, exist_ok=True)
        alts = sorted(team.glob("alt/*/rotation.gcsl"))
        withheld = d / "known.gcsl"
        withheld.write_text("\n".join(p.read_text() for p in alts))
        call([BIN, "dictionary", "filter", "--dictionary", dictionary, "--withheld", withheld,
              "--out", d / "dictionary.json"]).check_returncode()
        for seed in args.seeds:
            out = d / f"{args.label}-{seed}"
            for cap in args.caps:
                p = call([BIN, "search", "--manifest", manifest, "--baseline", baseline,
                          "--dictionary", d / "dictionary.json", "--algorithm", "greedy", "--fixed-er",
                          "--seed", seed, "--seconds", cap, "--out", out, "--audit", 32])
                if p.returncode:
                    print(name, seed, cap, "failed", p.stderr[-500:])
                    break
                r = load(out / "report.json")
                sel = next(a for a in r["audits"] if a["key"] == r["provisional_best_key"])
                print(name, seed, cap, "L", round(r["baseline"]["mean"]), "S", round(sel["joint"]["mean"]),
                      "decisions", r["completed_decisions"], "cpu", round(r["discovery_cpu_seconds"], 1), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=["export", "dictionary", "measure", "draw", "run", "confirm", "dev"])
    parser.add_argument("--gscouter", default="/home/user/Gscouter")
    parser.add_argument("--label", default="stage1")
    parser.add_argument("--cases", type=int, default=0)
    parser.add_argument("--skip-continuation", action="store_true")
    parser.add_argument("--only", default="")
    parser.add_argument("--seeds", type=int, nargs="*", default=[11])
    parser.add_argument("--caps", type=float, nargs="*", default=[12])
    args = parser.parse_args()
    {"export": export, "dictionary": dictionary, "measure": measure, "draw": draw,
     "run": run, "confirm": confirm, "dev": dev}[args.phase](args)


if __name__ == "__main__":
    main()
