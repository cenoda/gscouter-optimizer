"""Execute the preregistered greedy retry; never import a revealed cycle."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
BIN = ROOT / "target/release/gso"
RUNS = ROOT / "runs/retry"


def invoke(command):
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(f"{' '.join(map(str, command))}\n{result.stderr}")
    return result


def search(out, seed, cap, flags, audit=128):
    command = [str(BIN), "search", "--manifest", "data/flins-c1/standard.json",
               "--algorithm", "greedy", "--seed", str(seed), "--seconds", str(cap),
               "--out", str(out), "--audit", str(audit), *flags]
    result = invoke(command)
    report = json.loads((out / "report.json").read_text())
    return report, result


def matrix():
    variants = [("legacy-joint", []), ("revised-joint", ["--revised"]),
                ("revised-fixed", ["--revised", "--fixed-er"])]
    jobs = [(name, flags, seed) for name, flags in variants for seed in [11, 37, 91]]

    def trial(job):
        name, flags, seed = job
        out = RUNS / "matrix" / f"{name}-{seed}"
        reports = []
        for cap in [4, 12, 36, 108]:
            report, _ = search(out, seed, cap, flags)
            (out / f"report-{cap}.json").write_text(json.dumps(report, indent=2) + "\n")
            reports.append(report)
            selected = next(a for a in report["audits"] if a["key"] == report["provisional_best_key"])
            print(name, seed, cap, round(selected["joint"]["mean"]), flush=True)
        checker = invoke(["python3", "scripts/check_run.py", str(out)])
        return {"name": name, "seed": seed, "reports": reports,
                "invariant_check": json.loads(checker.stdout)}

    with ThreadPoolExecutor(max_workers=3) as pool:
        rows = list(pool.map(trial, jobs))
    (RUNS / "matrix/summary.json").write_text(json.dumps(rows, indent=2) + "\n")


def restricted():
    out = RUNS / "restricted"
    invoke([str(BIN), "enumerate", "--manifest", "data/flins-c1/standard.json",
            "--out", str(out / "exhaustive")])
    reference = json.loads((out / "exhaustive/reference.json").read_text())
    rows = []
    for seed in [11, 37, 91, 123, 231]:
        dest = out / str(seed)
        report, _ = search(dest, seed, 4, ["--fixed-er", "--restricted-domain"], audit=32)
        state = json.loads((dest / "checkpoint.json").read_text())
        index = next(i for i, row in enumerate(reference["rows"])
                     if row["program"] == state["best"]["program"])
        checker = invoke(["python3", "scripts/check_run.py", str(dest)])
        rows.append({"seed": seed, "recovered": index == reference["reference_best_index"],
                     "best_index": index, "report": report,
                     "invariant_check": json.loads(checker.stdout)})
        print("production recovery", seed, rows[-1]["recovered"], flush=True)
    (out / "production-recovery.json").write_text(json.dumps({
        "objective": "32 full fights at fixed seed 6000000000000; actual production search",
        "no_reference_table_in_search": True, "reference": reference,
        "runs": rows}, indent=2) + "\n")


def smoke():
    original = RUNS / "smoke-original"
    restored = RUNS / "smoke-restored"
    attempts = []
    if (original / "checkpoint.json").exists():
        first = json.loads((original / "report.json").read_text())
    else:
        first, _ = search(original, 231, 1, ["--revised"], audit=16)
    attempts.append(first)
    before = json.loads((original / "checkpoint.json").read_text())
    for cap in [2, 4, 6, 8]:
        if before["pending"] and before["pending"]["polishing"]:
            break
        if cap <= before["parent_cpu"]:
            continue
        first, _ = search(original, 231, cap, ["--revised"], audit=16)
        attempts.append(first)
        before = json.loads((original / "checkpoint.json").read_text())
    # Tiny cumulative extensions deliberately stop inside proposal generation
    # or a sample ladder, rather than requiring an unlucky wall-time kill.
    for _ in range(5):
        if before["pending"] and before["pending"]["polishing"]:
            break
        first, _ = search(original, 231, before["parent_cpu"] + .05, ["--revised"], audit=16)
        attempts.append(first)
        before = json.loads((original / "checkpoint.json").read_text())
    if not before["pending"] or not before["pending"]["polishing"]:
        raise RuntimeError("no unfinished polish proposal within bounded smoke attempts")
    shutil.copytree(original, restored, dirs_exist_ok=False)
    second, _ = search(restored, 231, first["discovery_cpu_cap"] + 2, ["--revised"], audit=16)
    after = json.loads((restored / "checkpoint.json").read_text())
    assert after["events"][:len(before["events"])] == before["events"]
    assert after["rng"] != before["rng"]
    assert after["step"] > before["step"]
    pending = before["pending"]
    assert after["events"][len(before["events"])]["operator"] == pending["operator"]
    checker = invoke(["python3", "scripts/check_run.py", str(restored)])
    audit_worker_cpu = sum(r["work"]["cpu"] for r in
                           (json.loads(line) for line in (restored / "evaluations.jsonl").read_text().splitlines())
                           if r["seed"] >= 4_000_000_000_000)
    (RUNS / "smoke-recovery.json").write_text(json.dumps({
        "before_decisions": before["step"], "after_decisions": after["step"],
        "pending_phase": "polish" if pending["polishing"] else "discovery",
        "pending_operator": pending["operator"], "polish_before": before["polish"],
        "polish_after": after["polish"], "prior_events_exactly_preserved": True,
        "rng_advanced": True, "before_cpu": first["discovery_cpu_seconds"],
        "after_cpu": second["discovery_cpu_seconds"],
        "audit_cpu_seconds": audit_worker_cpu,
        "audit_cpu_accounting": "all audit worker CPU from the journal, including earlier smoke attempts; parent audit overhead excluded",
        "invariant_check": json.loads(checker.stdout)}, indent=2) + "\n")
    print("checkpoint recovery verified", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("phase", choices=["smoke", "restricted", "matrix"])
    args = parser.parse_args()
    {"smoke": smoke, "restricted": restricted, "matrix": matrix}[args.phase]()
