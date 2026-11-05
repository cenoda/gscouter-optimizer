"""Compare a frozen candidate with the pinned engine's gcsim reference.

Build the reference without editing the other repository:
  cd ../gscouter-engine/reference && go build -o ../../gscouter-optimizer/runs/gcsim-reference .
"""
import argparse
import json
import math
from pathlib import Path
import resource
import subprocess
import time


def verify(config, out, samples=2048, seed=928123, reference="runs/gcsim-reference"):
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    rows = {}
    accounting = {}
    for name, command in [
        ("native", ["target/release/examples/trace", str(config), str(samples), str(seed)]),
        ("gcsim", [reference, "-config", str(config), "-iter", str(samples), "-warmup", "150", "-seed", str(seed), "-j", "4", "-abil", "-cache", ""]),
    ]:
        before = resource.getrusage(resource.RUSAGE_CHILDREN)
        start = time.monotonic()
        p = subprocess.run(command, text=True, capture_output=True, check=True)
        after = resource.getrusage(resource.RUSAGE_CHILDREN)
        rows[name] = json.loads(p.stdout)
        (out / f"{name}.json").write_text(json.dumps(rows[name], indent=2) + "\n")
        accounting[name] = {"cpu_seconds": after.ru_utime + after.ru_stime - before.ru_utime - before.ru_stime, "wall_seconds": time.monotonic() - start, "command": command}
    native, go = rows["native"], rows["gcsim"]
    comparison = []
    for name, a, b in [("team", native["team"], go["team"])] + [
        (name, native["chars"][name], go["chars"][name]) for name in native["chars"]
    ]:
        fraction = b["mean"] / go["team"]["mean"]
        difference = a["mean"] / b["mean"] - 1
        se = math.hypot(a["se"], b["stderr"])
        comparison.append({"name": name, "native_mean": a["mean"], "gcsim_mean": b["mean"],
                           "relative_difference": difference, "combined_se_dps": se,
                           "within_engine_accuracy_bar": abs(difference) <= .01,
                           "major_contributor": name == "team" or fraction >= .1})
    passed = all(r["within_engine_accuracy_bar"] for r in comparison if r["major_contributor"])
    report = {"config": str(config), "engine": "1ba0af119e0477d6f44ee966a78fa06d84d2c5de", "gcsim": "cenoda/gcsim@eb218847f82b", "iterations": samples,
              "seed": seed, "accuracy_passed": passed, "comparison": comparison, "accounting": accounting}
    (out / "comparison.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report), flush=True)
    if not passed:
        raise RuntimeError("pinned-engine accuracy discrepancy; inspect the recorded comparison")
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("config")
    parser.add_argument("out")
    parser.add_argument("--samples", type=int, default=2048)
    parser.add_argument("--seed", type=int, default=928123)
    parser.add_argument("--reference", default="runs/gcsim-reference")
    args = parser.parse_args()
    verify(args.config, args.out, args.samples, args.seed, args.reference)
