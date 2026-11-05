"""Fresh frozen-candidate confirmation and gcsim parity after retry selection."""
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import resource
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "reports/pilot-retry-2026-11-05"
SEED = 10_000_000_000_000
SAMPLES = 2048


def paired(a, b):
    values = [x - y for x, y in zip(a, b, strict=True)]
    return {"difference_dps": statistics.mean(values),
            "paired_se_dps": statistics.stdev(values) / len(values) ** .5}


def main():
    selection = json.loads((OUT / "selection.json").read_text())
    selected = (OUT / "selected.gcsl").read_text()
    original = ROOT / "reports/pilot-2026-11-05"
    baseline = (original / "baseline.gcsl").read_text()
    reference = (original / "withheld-reference.gcsl").read_text()
    header = selected.split("active ", 1)[0]
    configs = {"selected": selected,
               "baseline-same-allocation": header + "active " + baseline.split("active ", 1)[1],
               "reference-standard": reference,
               "reference-same-allocation": header + "active " + reference.split("active ", 1)[1]}
    paths = {}
    unique = {}
    aliases = {}
    for name, config in configs.items():
        path = OUT / f"confirmation-{name}.gcsl"
        path.write_text(config)
        paths[name] = path
        identity = hashlib.sha256("\n".join(line for line in config.splitlines()
                                  if not line.startswith("#")).encode()).hexdigest()
        if identity in unique:
            aliases[name] = unique[identity]
        else:
            unique[identity] = name

    def evaluate(name):
        if name == "selected":
            command = ["python3", "scripts/verify_reference.py", str(paths[name]),
                       str(OUT / "parity-selected"), "--samples", str(SAMPLES), "--seed", str(SEED)]
            subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=True)
            return name, json.loads((OUT / "parity-selected/native.json").read_text())
        command = ["target/release/examples/trace", str(paths[name]), str(SAMPLES), str(SEED)]
        result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=True)
        return name, json.loads(result.stdout)

    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    start = time.monotonic()
    with ThreadPoolExecutor(max_workers=4) as pool:
        results = dict(pool.map(evaluate, unique.values()))
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    for name, alias in aliases.items():
        results[name] = results[alias]
    for name, result in results.items():
        (OUT / f"confirmation-{name}.json").write_text(json.dumps(result, indent=2) + "\n")
    report = {"seed": SEED, "iterations": SAMPLES, "duration": 450, "warmup": 150,
              "frozen_selection": selection, "identical_noncomment_source_reuse": aliases,
              "conditions": {name: {"config_sha256": hashlib.sha256(configs[name].encode()).hexdigest(),
                                    "team": result["team"], "chars": result["chars"],
                                    "mean_wait_seconds": result["mean_wait_seconds"],
                                    "flins_abilities": result["abilities_hits_and_dps"]["flins"]}
                             for name, result in results.items()},
              "selected_minus": {name: paired(results["selected"]["per_iteration_dps"], result["per_iteration_dps"])
                                 for name, result in results.items() if name != "selected"},
              "cpu_seconds_including_selected_parity": after.ru_utime + after.ru_stime - before.ru_utime - before.ru_stime,
              "wall_seconds": time.monotonic() - start}
    (OUT / "confirmation.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
