"""Held-out recovery v2 (docs/heldout-recovery-v2.md).

Phases:
  filter   per-case v2 leakage filter (reads A; drop lists only) for every exported case
  draw     seeded stratified draw of new held-out cases; exports CI inputs (no A)
  job      one case: all arms and starts (V1, V2S, V2T), 4 processes, strace check
  dev      `job` on a development case from runs/heldout/cases (stage-1 cases only)
  confirm  after every job finished: A, A_strip, L_strip, S_strip on audit seeds
  report   tables and readings

`job` needs only an inputs directory with manifest.json, L.gcsl,
dictionary-v1.json and dictionary-v2.json, and two binaries. It never reads
A, alt/ or a Gscouter checkout.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import platform
import random
import re
import shutil
import subprocess
import threading
import time

ROOT = Path(__file__).resolve().parents[1]
RUNS1 = ROOT / "runs/heldout"
RUNS = ROOT / "runs/heldout-v2"
STAGE1 = ROOT / "reports/heldout-recovery-2026-11-05"
OUT = ROOT / "reports/heldout-recovery-v2-2026-11-05"
PROTOCOL = ROOT / "docs/heldout-recovery-v2.md"
DRAW_SEED = 20261106
DRAW_INFORMATIVE = 32
DRAW_CONTROLS = 8
STARTS = [11, 37, 91]
V1_CAPS = [36, 108, 192]
V2_CAPS = [36, 108, 192]
CONT_CAPS = [288, 384]
SECOND_PASS_FROM = 192
AUDIT_SAMPLES = 128
FORBIDDEN = ("/alt/", "withheld", "/Gscouter/", "heldout-recovery-2026-11-05")


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=1) + "\n")


def load(path):
    return json.loads(Path(path).read_text())


def call(command, **kw):
    return subprocess.run(list(map(str, command)), cwd=ROOT, text=True, capture_output=True, **kw)


# ---------------------------------------------------------------- filter
def filter_(args):
    meta = load(RUNS1 / "cases.json")
    gdict = RUNS / "dictionary-v2.json"
    out = {}
    for row in meta["exported"]:
        case = row["case"]
        dest = RUNS / "cases" / case
        dest.mkdir(parents=True, exist_ok=True)
        p = call([BIN(args), "dictionary-v2", "filter", "--dictionary", gdict, "--manifest",
                  RUNS1 / "cases" / case / "manifest.json", "--withheld", RUNS1 / "withheld" / case / "A.gcsl",
                  "--out", dest / "dictionary-v2.json"])
        assert p.returncode == 0, p.stderr
        d = load(dest / "dictionary-v2.json")
        write(dest / "drops-v2.json", {"dropped_for_leakage": d["dropped_for_leakage"],
                                       "sha256": sha(dest / "dictionary-v2.json")})
        v1 = load(RUNS1 / "cases" / case / "dictionary.json")
        write(dest / "drops-v1.json", {"dropped_for_leakage": v1["dropped_for_leakage"],
                                       "sha256": sha(RUNS1 / "cases" / case / "dictionary.json")})
        out[case] = {"v2_dropped": len(d["dropped_for_leakage"]), "v1_dropped": len(v1["dropped_for_leakage"])}
    write(RUNS / "filter-summary.json", out)
    v1 = sorted(v["v1_dropped"] for v in out.values())
    v2 = sorted(v["v2_dropped"] for v in out.values())
    print("cases", len(out), "median dropped v1", v1[len(v1) // 2], "v2", v2[len(v2) // 2], "max v2", v2[-1])


def BIN(args):
    return Path(getattr(args, "v2_bin", None) or ROOT / "target/release/gso")


# ---------------------------------------------------------------- draw
def draw(args):
    if (OUT / "draw.json").exists():
        raise RuntimeError("the draw is frozen")
    cls = load(STAGE1 / "classification.json")
    stage1 = {o["case"] for o in load(STAGE1 / "draw.json")["order"]}
    rng = random.Random(DRAW_SEED)
    # Amendment 1, item 9: the development parties' own alternatives (Flins C1,
    # Mavuika C6R5) were used as references in earlier reports.
    dev_teams = ("flins-ineffa-columbina-sucrose", "mavuika-citlali-xilonen-nicole")
    pool = [r for r in cls["rows"] if r["case"] not in stage1 and r["kind"] == "database"
            and r["class"] in ("informative", "control") and r["case"].split("--")[0] not in dev_teams]

    def stratified(rows, k):
        strata = {s: sorted(r["case"] for r in rows if r["stratum"] == s) for s in ("frame", "other")}
        total = sum(map(len, strata.values()))
        quota = {s: k * len(v) / total for s, v in strata.items()}
        alloc = {s: int(q) for s, q in quota.items()}
        for s in sorted(quota, key=lambda s: (-(quota[s] - alloc[s]), s))[:k - sum(alloc.values())]:
            alloc[s] += 1
        return {s: rng.sample(strata[s], alloc[s]) for s in strata}, alloc

    informative, ia = stratified([r for r in pool if r["class"] == "informative"], DRAW_INFORMATIVE)
    controls, ca = stratified([r for r in pool if r["class"] == "control"], DRAW_CONTROLS)
    order = [c for s in informative.values() for c in s] + [c for s in controls.values() for c in s]
    rng.shuffle(order)
    by = {r["case"]: r for r in cls["rows"]}
    rows = export_inputs(OUT, order, by)
    inputs = OUT / "inputs"
    head = call(["git", "rev-parse", "HEAD"]).stdout.strip()
    write(OUT / "draw.json", {
        "seed": DRAW_SEED, "protocol": {"path": str(PROTOCOL.relative_to(ROOT)), "sha256": sha(PROTOCOL)},
        "implementation_commit": head, "classification_sha256": sha(STAGE1 / "classification.json"),
        "pool": {"informative": sum(r["class"] == "informative" for r in pool),
                 "control": sum(r["class"] == "control" for r in pool),
                 "rule": "classification rows, kind database, class informative/control, not a stage-1 case, "
                         "not an alternative of the development teams flins-ineffa-columbina-sucrose / mavuika-citlali-xilonen-nicole"},
        "method": "random.Random(20261106); stratified by stage-1 frame stratum (largest remainder); "
                  "run order is a seeded shuffle; a smaller run takes a prefix of this order",
        "allocation": {"informative": ia, "control": ca}, "cases": len(order),
        "dictionaries": {"v1_sha256": sha(inputs / "dictionary-v1.json"), "v2_sha256": sha(inputs / "dictionary-v2.json")},
        "order": rows})
    print(json.dumps({"informative": ia, "control": ca}), len(order))


def export_inputs(rundir, order, by):
    """Per-case search inputs only (manifest, L, build, drop lists) plus global dictionaries; never A."""
    inputs = rundir / "inputs"
    rows = []
    for c in order:
        d = inputs / c
        d.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(RUNS1 / "cases" / c / "manifest.json", d / "manifest.json")
        shutil.copyfile(RUNS1 / "cases" / c / "L.gcsl", d / "L.gcsl")
        shutil.copyfile(RUNS1 / "cases" / c / "build.txt", d / "build.txt")
        shutil.copyfile(RUNS / "cases" / c / "drops-v1.json", d / "drops-v1.json")
        shutil.copyfile(RUNS / "cases" / c / "drops-v2.json", d / "drops-v2.json")
        rows.append({"case": c, "class": by[c]["class"], "stratum": by[c]["stratum"],
                     "manifest_sha256": sha(d / "manifest.json"), "L_sha256": sha(d / "L.gcsl"),
                     "dictionary_v1_sha256": load(d / "drops-v1.json")["sha256"],
                     "dictionary_v2_sha256": load(d / "drops-v2.json")["sha256"],
                     "v2_dropped": len(load(d / "drops-v2.json")["dropped_for_leakage"]),
                     "v1_dropped": len(load(d / "drops-v1.json")["dropped_for_leakage"])})
    shutil.copyfile(STAGE1 / "dictionary.json", inputs / "dictionary-v1.json")
    shutil.copyfile(RUNS / "dictionary-v2.json", inputs / "dictionary-v2.json")
    write(rundir / "matrix.json", order)
    return rows


def devexport(args):
    """Development cases (stage-1 cases, already analysed with A) for a CI development run."""
    stage1 = {o["case"] for o in load(STAGE1 / "draw.json")["order"]}
    cases = args.cases
    assert cases and all(c in stage1 for c in cases), "development runs take stage-1 cases only"
    by = {r["case"]: r for r in load(STAGE1 / "classification.json")["rows"]}
    rundir = OUT / "development"
    write(rundir / "cases.json", export_inputs(rundir, cases, by))


# ---------------------------------------------------------------- job
def opened_paths(log):
    paths = set()
    for line in Path(log).read_text(errors="replace").splitlines():
        m = re.search(r'(?:openat|open|execve)\((?:AT_FDCWD, )?"([^"]*)"', line)
        if m:
            paths.add(m.group(1))
    return sorted(paths)


def journal(dest, key, seed, n):
    found = {}
    path = dest / "evaluations.jsonl"
    for line in path.read_text().splitlines():
        r = json.loads(line)
        if r["key"] != key:
            continue
        for i, sample in enumerate(r["work"]["samples"]):
            if seed <= r["seed"] + i < seed + n and "Ok" in sample:
                found[r["seed"] + i] = sample["Ok"]["dps"]
    return [found.get(seed + i) for i in range(n)]


def op_family(op):
    op = re.sub(r"^compound x\d+: ", "compound ", op)
    for k in ["compound", "minimise", "restart perturbation", "initial family", "micro", "generic",
              "dictionary replace", "dictionary insert", "template replace", "template insert",
              "reassign owner", "delete unit", "move unit", "substitute action", "repeat segment",
              "delete segment", "delete statement", "move statement"]:
        if op.startswith(k):
            return k
    return op.split(":")[0][:40]


def search(binary, inputs, dest, arm, seed, cap, extra, stats_lock, stats):
    dest.mkdir(parents=True, exist_ok=True)
    log = dest / f"strace-{cap}.log"
    dictionary = inputs / ("dictionary-v1.json" if arm == "V1" else "dictionary-v2.json")
    cmd = ["strace", "-f", "--seccomp-bpf", "-qq", "-e", "trace=open,openat,execve", "-o", log,
           binary, "search", "--manifest", inputs / "manifest.json", "--baseline", inputs / "L.gcsl",
           "--dictionary", dictionary, "--fixed-er", "--seed", seed, "--seconds", cap,
           "--out", dest, "--audit", AUDIT_SAMPLES] + extra
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
                                          "candidate": state["best"]})
    sel = next(a for a in rep["audits"] if a["key"] == rep["provisional_best_key"])
    shutil.copyfile(dest / f"candidate-{sel['index']}.gcsl", dest / f"selected-{cap}.gcsl")
    seed_a = rep["audit_seed"]
    write(dest / f"audit-{cap}.json", {"audit_seed": seed_a, "baseline_key": rep["audits"][0]["key"],
          "per_fight": {a["key"]: journal(dest, a["key"], seed_a, AUDIT_SAMPLES)
                        for a in rep["audits"] if "joint" in a}})
    ops = {}
    for ev in state["events"]:
        f = op_family(ev["operator"])
        o = ops.setdefault(f, {"decisions": 0, "accepted": 0, "improving": 0, "failed": 0})
        if ev["kind"].startswith("failed") or ev["kind"].startswith("unresolved"):
            o["failed"] += 1
        elif ev["samples"]:
            o["decisions"] += 1
            o["accepted"] += ev["accepted"]
            o["improving"] += ev["accepted"] and ev["delta"] > 0
    write(dest / f"ops-{cap}.json", ops)
    with stats_lock:
        stats.append({"arm": arm, "seed": seed, "cap": cap, "wall": wall,
                      "discovery_cpu": rep["discovery_cpu_seconds"], "audit_cpu": rep["audit_cpu_seconds"],
                      "decisions": rep["completed_decisions"]})
    print(f"{arm} seed {seed} cap {cap}: L={rep['baseline']['mean']:.0f} S={sel.get('joint', {}).get('mean', float('nan')):.0f} "
          f"cpu={rep['discovery_cpu_seconds']:.1f} decisions={rep['completed_decisions']} wall={wall:.0f}", flush=True)
    return rep


def environment():
    model = ""
    try:
        model = next(l.split(":", 1)[1].strip() for l in Path("/proc/cpuinfo").read_text().splitlines()
                     if l.startswith("model name"))
    except (StopIteration, OSError):
        pass
    return {"nproc": os.cpu_count(), "cpu_model": model, "platform": platform.platform(),
            "runner_image": {k: os.environ.get(k) for k in ("ImageOS", "ImageVersion", "RUNNER_NAME",
                                                            "RUNNER_ARCH", "GITHUB_RUN_ID", "GITHUB_SHA")}}


def job(args):
    global V1_CAPS, V2_CAPS, CONT_CAPS, SECOND_PASS_FROM
    if args.scale != 1.0:  # development smoke tests only; never for held-out cases
        V1_CAPS, V2_CAPS, CONT_CAPS = ([round(c * args.scale, 2) for c in caps] for caps in (V1_CAPS, V2_CAPS, CONT_CAPS))
        SECOND_PASS_FROM = V2_CAPS[-1]
    inputs = Path(args.inputs).resolve()
    out = Path(args.out).resolve()
    v1, v2 = Path(args.v1_bin).resolve(), Path(args.v2_bin).resolve()
    out.mkdir(parents=True, exist_ok=True)
    started = time.time()
    t0 = os.times()
    lock, stats, errors = threading.Lock(), [], []
    pool = ThreadPoolExecutor(max_workers=args.workers)
    futures, flock = [], threading.Lock()

    def guarded(fn, *a):
        try:
            fn(*a)
        except Exception as exc:  # recorded, the job continues with other arms
            with lock:
                errors.append({"task": [str(x) for x in a], "error": str(exc)[-800:]})
            print("failed:", a, str(exc)[-300:], flush=True)

    def run_caps(binary, arm, seed, dest, caps, extra):
        for cap in caps:
            search(binary, inputs, dest, arm, seed, cap, extra, lock, stats)

    def v2_chain(seed):
        base = out / "V2S" / str(seed)
        run_caps(v2, "V2S", seed, base, V2_CAPS, ["--heldout-v2", "--algorithm", "anneal"])
        two = out / "V2T" / str(seed)
        if two.exists():
            shutil.rmtree(two)
        shutil.copytree(base, two)
        for f in two.glob("*-*.json"):
            f.unlink()
        for f in two.glob("selected-*.gcsl"):
            f.unlink()
        with flock:
            futures.append(pool.submit(guarded, run_caps, v2, "V2S", seed, base, CONT_CAPS,
                                       ["--heldout-v2", "--algorithm", "anneal"]))
            futures.append(pool.submit(guarded, run_caps, v2, "V2T", seed, two, CONT_CAPS,
                                       ["--heldout-v2", "--algorithm", "anneal", "--second-pass-from", SECOND_PASS_FROM]))

    with flock:
        for seed in STARTS:
            futures.append(pool.submit(guarded, v2_chain, seed))
        for seed in STARTS:
            futures.append(pool.submit(guarded, run_caps, v1, "V1", seed, out / "V1" / str(seed), V1_CAPS,
                                       ["--algorithm", "greedy"]))
    done = 0
    while True:
        with flock:
            pending = futures[done:]
        if not pending:
            break
        for f in pending:
            f.result()
        done += len(pending)
    pool.shutdown()
    t1 = os.times()
    # Keep the artifact small: drop journals and full checkpoints.
    for d in out.glob("*/*"):
        for name in ["evaluations.jsonl", "proposals.jsonl", "checkpoint.json", "report.json"]:
            if (d / name).exists():
                (d / name).unlink()
        for f in d.glob("candidate-*.gcsl"):
            f.unlink()
        for f in d.glob("worker-*"):
            f.unlink() if f.is_file() else shutil.rmtree(f)
    write(out / "job.json", {"case": args.case, "environment": environment(), "workers": args.workers,
                             "wall_seconds": time.time() - started,
                             "children_cpu_seconds": (t1.children_user + t1.children_system) - (t0.children_user + t0.children_system),
                             "searches": stats, "errors": errors,
                             "binaries": {"v1_sha256": sha(v1), "v2_sha256": sha(v2)},
                             "inputs": {f.name: sha(f) for f in sorted(inputs.iterdir()) if f.is_file()}})
    print("job done", args.case, f"{time.time() - started:.0f}s wall", len(errors), "errors", flush=True)


def materialize(args):
    """CI: rebuild the per-case dictionaries from the committed drop lists and check hashes."""
    src = Path(args.inputs_root) / args.case
    dest = Path(args.dest)
    dest.mkdir(parents=True, exist_ok=True)
    for f in ["manifest.json", "L.gcsl"]:
        shutil.copyfile(src / f, dest / f)
    for v in ["v1", "v2"]:
        p = call([BIN(args), "dictionary-v2", "drop", "--dictionary", Path(args.inputs_root) / f"dictionary-{v}.json",
                  "--drops", src / f"drops-{v}.json", "--out", dest / f"dictionary-{v}.json"])
        assert p.returncode == 0, p.stderr
        expect = load(src / f"drops-{v}.json")["sha256"]
        assert sha(dest / f"dictionary-{v}.json") == expect, f"{v} dictionary hash mismatch"
    print("materialized", args.case)


def dev(args):
    """Development: a stage-1 case (already analysed with A) through `job`."""
    case = args.case
    d = RUNS / "dev-inputs" / case
    d.mkdir(parents=True, exist_ok=True)
    for f in ["manifest.json", "L.gcsl"]:
        shutil.copyfile(RUNS1 / "cases" / case / f, d / f)
    shutil.copyfile(RUNS1 / "cases" / case / "dictionary.json", d / "dictionary-v1.json")
    shutil.copyfile(RUNS / "cases" / case / "dictionary-v2.json", d / "dictionary-v2.json")
    args.inputs, args.out = d, RUNS / "dev" / args.label / case
    job(args)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=["filter", "draw", "devexport", "job", "materialize", "dev", "confirm", "report"])
    parser.add_argument("--cases", nargs="*", default=[])
    parser.add_argument("--case", default="")
    parser.add_argument("--inputs", default="")
    parser.add_argument("--inputs-root", default=str(OUT / "inputs"))
    parser.add_argument("--dest", default="")
    parser.add_argument("--out", default="")
    parser.add_argument("--label", default="dev")
    parser.add_argument("--workers", type=int, default=4)
    parser.add_argument("--scale", type=float, default=1.0)
    parser.add_argument("--draw", default="")
    parser.add_argument("--report-dir", default="")
    parser.add_argument("--v1-bin", default=str(ROOT / "target-v1/release/gso"))
    parser.add_argument("--v2-bin", default=str(ROOT / "target/release/gso"))
    args = parser.parse_args()
    if args.phase in ("confirm", "report"):
        import heldout_v2_report
        getattr(heldout_v2_report, args.phase)(args)
        return
    {"filter": filter_, "draw": draw, "devexport": devexport, "job": job, "materialize": materialize, "dev": dev}[args.phase](args)


if __name__ == "__main__":
    main()
