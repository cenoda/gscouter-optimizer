"""Run declared CPU-capped experiments. No score is an absolute pass threshold.

Freeze the generator before running. This does not read withheld alternatives.
Each independent process has its own checkpoint/cache and one executing worker.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
BIN = ROOT / "target/release/gso"


def invoke(cmd):
    result = subprocess.run(cmd, cwd=ROOT, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(result.stderr)
    return result


def prepare():
    root = ROOT / "runs/preparation"
    root.mkdir(parents=True, exist_ok=True)
    original = json.loads((ROOT / "data/flins-c1/manifest.json").read_text())
    choices = [("wanderingevenstar", 5), ("sacrificialfragments", 5),
               ("thrillingtalesofdragonslayers", 5), ("athousandfloatingdreams", 1)]

    def trial(choice):
        weapon, refinement = choice
        m = json.loads(json.dumps(original))
        m["members"][3]["weapon"] = weapon
        m["members"][3]["refinement"] = refinement
        manifest = root / f"{weapon}.json"
        manifest.write_text(json.dumps(m, indent=2) + "\n")
        out = root / weapon
        invoke([str(BIN), "search", "--manifest", str(manifest), "--out", str(out),
                "--seconds", "8", "--audit", "128", "--seed", "17"])
        report = json.loads((out / "report.json").read_text())
        best = max((a for a in report["audits"] if "joint" in a), key=lambda a: a["joint"]["mean"])
        return {"weapon": weapon, "refinement": refinement, "best_audit": best, "report": report, "manifest": m}

    with ThreadPoolExecutor(max_workers=4) as pool:
        results = list(pool.map(trial, choices))
    winner = max(results, key=lambda r: r["best_audit"]["joint"]["mean"])
    (ROOT / "data/flins-c1/standard.json").write_text(json.dumps(winner["manifest"], indent=2) + "\n")
    (root / "selection.json").write_text(json.dumps({"rule": "best independently evaluated conditional result after equal 8 CPU-second preparation searches; not a global weapon optimum", "selected": winner["weapon"], "trials": results}, indent=2) + "\n")
    print("Selected conditional standard weapon:", winner["weapon"], flush=True)


def matrix(only_local=False):
    jobs = [(alg, seed, flag) for alg in ["anneal", "greedy", "random"] for seed in [11, 37, 91] for flag in [""]]
    jobs += [("anneal", seed, flag) for seed in [11, 37, 91] for flag in ["--local-only", "--fixed-er"]]

    if only_local:
        jobs = [("anneal", seed, "--local-only") for seed in [11, 37, 91]]
    matrix_root = ROOT / ("runs/matrix-local-corrected" if only_local else "runs/matrix")

    def trial(job):
        alg, seed, flag = job
        name = f"{alg}-{seed}{flag}"
        out = matrix_root / name
        results = []
        for budget in [4, 12, 36]:
            command = [str(BIN), "search", "--manifest", "data/flins-c1/standard.json",
                       "--out", str(out), "--seconds", str(budget), "--audit", "128", "--seed", str(seed), "--algorithm", alg]
            if flag:
                command.append(flag)
            invoke(command)
            report = json.loads((out / "report.json").read_text())
            (out / f"report-{budget}.json").write_text(json.dumps(report, indent=2) + "\n")
            results.append(report)
            print(name, budget, "CPU seconds completed", flush=True)
        invoke(["python3", "scripts/check_run.py", str(out)])
        return {"name": name, "reports": results}

    with ThreadPoolExecutor(max_workers=4) as pool:
        results = list(pool.map(trial, jobs))
    out = matrix_root / "summary.json"
    out.write_text(json.dumps(results, indent=2) + "\n")
    print("Matrix:", out, flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("phase", choices=["prepare", "matrix", "local"])
    args = parser.parse_args()
    prepare() if args.phase == "prepare" else matrix(args.phase == "local")
