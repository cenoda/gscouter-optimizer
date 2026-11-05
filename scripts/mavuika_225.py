"""Fixed-condition, cycle-free Mavuika development experiment.

Discovery never receives the reference path. Stage selection uses the discovery
peak; independent audit ranking cannot change the selected program.
"""
import argparse
import hashlib
import json
from pathlib import Path
import resource
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--phase", default="phase1")
    parser.add_argument("--binary", default="target/release/gso")
    parser.add_argument("--caps", default="12,36")
    parser.add_argument("--seeds", default="11,37,91")
    parser.add_argument("--modes", default="extended,basic")
    args = parser.parse_args()
    runroot = ROOT / "runs" / f"mavuika225-{args.phase}"
    reportroot = ROOT / "reports/mavuika-225-2026-11-05" / args.phase
    frozen = read(runroot / "frozen.json")
    for name, expected in frozen["source_files"].items():
        assert hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == expected, name
    manifest = ROOT / "data/mavuika-c6-r5-225/manifest.json"
    assert hashlib.sha256(manifest.read_bytes()).hexdigest() == frozen["manifest"]
    accounting = read(reportroot / "invocations.json") if (reportroot / "invocations.json").exists() else []
    for cap in map(float, args.caps.split(",")):
        for seed in map(int, args.seeds.split(",")):
            for mode in args.modes.split(","):
                assert mode in ("basic", "extended")
                out = runroot / mode / f"seed-{seed}"
                dest = reportroot / mode / f"seed-{seed}" / f"cap-{cap:g}"
                if (dest / "result.json").exists():
                    print("saved", mode, seed, cap, flush=True)
                    continue
                command = [str(ROOT / args.binary), "search", "--from-scratch",
                           "--manifest", str(manifest), "--fixed-er", "--algorithm", "anneal",
                           "--seconds", str(cap), "--seed", str(seed), "--audit", "128", "--out", str(out)]
                if mode == "basic":
                    command.append("--basic-grammar")
                start = resource.getrusage(resource.RUSAGE_CHILDREN)
                wall = time.monotonic()
                proc = subprocess.run(command, cwd=ROOT, text=True, capture_output=True)
                end = resource.getrusage(resource.RUSAGE_CHILDREN)
                record = {"mode": mode, "seed": seed, "cap": cap, "returncode": proc.returncode,
                          "cpu": end.ru_utime + end.ru_stime - start.ru_utime - start.ru_stime,
                          "wall": time.monotonic() - wall, "command": command,
                          "stdout": proc.stdout, "stderr": proc.stderr}
                accounting.append(record)
                write(reportroot / "invocations.json", accounting)
                if proc.returncode:
                    print("failed", mode, seed, cap, proc.stderr, flush=True)
                    continue
                check = subprocess.run(["python3", "scripts/check_run.py", str(out)], cwd=ROOT,
                                       text=True, capture_output=True, check=True)
                dest.mkdir(parents=True, exist_ok=True)
                for filename in ["report.json", "checkpoint.json", "experiment.json"]:
                    shutil.copyfile(out / filename, dest / filename)
                report = read(out / "report.json")
                row = next(row for row in report["audits"] if row["key"] == report["provisional_best_key"])
                shutil.copyfile(out / f"candidate-{row['index']}.gcsl", dest / "selected.gcsl")
                result = {"mode": mode, "seed": seed, "cap": cap,
                          "discovery_cpu": report["discovery_cpu_seconds"], "audit_cpu": report["audit_cpu_seconds"],
                          "key": row["key"], "discovery_estimate": read(out / "checkpoint.json")["best_hint"],
                          "independent_audit": row.get("joint"), "checker": json.loads(check.stdout)}
                write(dest / "result.json", result)
                print(json.dumps(result), flush=True)


if __name__ == "__main__":
    main()
