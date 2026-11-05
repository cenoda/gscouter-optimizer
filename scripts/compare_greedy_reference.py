"""Post-pilot diagnostic; never feed the revealed reference into discovery.

Cross two frozen allocation headers, two support blocks and two Flins blocks.
All eight conditions use common fresh seeds and full 450-second fights. The
reference is a known library alternative, not a certified unrestricted optimum.
"""
import concurrent.futures
import hashlib
import json
from pathlib import Path
import re
import resource
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "reports/pilot-2026-11-05/greedy-reference"
SEED = 9_000_000_000_000
ITERATIONS = 2048


def digest(data):
    return hashlib.sha256(data.encode()).hexdigest()


def split_source(source):
    header, program = re.split(r"(?m)^active\s+", source, maxsplit=1)
    program = "active " + program
    pattern = r"\bflins\s+skill\b.*?(?=if\s*\.\s*sucrose\s*\.\s*skill)"
    matches = list(re.finditer(pattern, program, flags=re.S))
    if len(matches) != 1:
        raise ValueError("expected exactly one Flins main-field block")
    match = matches[0]
    return header, program[:match.start()], match.group(), program[match.end():]


def paired(a, b):
    diffs = [x - y for x, y in zip(a, b, strict=True)]
    return {"mean_dps": statistics.mean(diffs),
            "paired_se_dps": statistics.stdev(diffs) / len(diffs) ** .5}


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    sources = {
        "greedy": ROOT / "runs/matrix/greedy-37/candidate-1.gcsl",
        "reference": ROOT / "reports/pilot-2026-11-05/withheld-reference.gcsl",
    }
    text = {k: (v if v.exists() else OUT / f"source-{k}.gcsl").read_text()
            for k, v in sources.items()}
    blocks = {k: split_source(v) for k, v in text.items()}
    for name, source in text.items():
        (OUT / f"source-{name}.gcsl").write_text(source)
    conditions = []
    for allocation in blocks:
        for support in blocks:
            for flins in blocks:
                name = f"allocation-{allocation}_support-{support}_flins-{flins}"
                header = re.sub(r"(?m)^#.*\n", "", blocks[allocation][0])
                config = (f"# post-pilot diagnostic: {name}\n" + header
                          + blocks[support][1] + blocks[flins][2] + blocks[support][3])
                path = OUT / f"{name}.gcsl"
                path.write_text(config)
                conditions.append({"name": name, "allocation": allocation,
                                   "support": support, "flins": flins,
                                   "config_sha256": digest(config)})
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    started = time.monotonic()

    def evaluate(row):
        command = [str(ROOT / "target/release/examples/trace"),
                   str(OUT / f"{row['name']}.gcsl"), str(ITERATIONS), str(SEED)]
        result = subprocess.run(command, text=True, capture_output=True, check=True)
        data = json.loads(result.stdout)
        (OUT / f"{row['name']}.json").write_text(json.dumps(data, indent=2) + "\n")
        return row | {"command": command, "result": data}

    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        results = list(pool.map(evaluate, conditions))
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    contrasts = []
    for allocation in blocks:
        subset = {(r["support"], r["flins"]): r for r in results
                  if r["allocation"] == allocation}
        for axis in ["flins", "support"]:
            for fixed in blocks:
                a = subset[(fixed, "reference") if axis == "flins" else ("reference", fixed)]
                b = subset[(fixed, "greedy") if axis == "flins" else ("greedy", fixed)]
                contrasts.append({"allocation": allocation, "replace": axis,
                                  "other_block": fixed, "direction": "reference minus greedy",
                                  **paired(a["result"]["per_iteration_dps"],
                                           b["result"]["per_iteration_dps"])})
    report = {
        "purpose": "post-pilot frozen-block diagnostic; no search or generator tuning",
        "engine_commit": "1ba0af119e0477d6f44ee966a78fa06d84d2c5de",
        "iterations": ITERATIONS, "seed": SEED, "duration": 450, "warmup": 150,
        "greedy_selection": "highest audited outcome among three joint-ER greedy starts at 36 CPU-s; seed 37; not a median run",
        "sources": {k: {"path": str(v.relative_to(ROOT)), "sha256": digest(text[k])}
                    for k, v in sources.items()},
        "cpu_seconds": after.ru_utime + after.ru_stime - before.ru_utime - before.ru_stime,
        "wall_seconds": time.monotonic() - started,
        "conditions": [{k: v for k, v in r.items() if k != "result"} | {
            "team": r["result"]["team"], "chars": r["result"]["chars"],
            "mean_wait_seconds": r["result"]["mean_wait_seconds"],
            "flins_abilities": r["result"]["abilities_hits_and_dps"]["flins"]}
            for r in results],
        "paired_block_contrasts": contrasts,
        "limitations": ["known reference is not a proven optimum",
                        "block substitution changes subsequent timing and random-number consumption",
                        "support block includes both entry and exit actions",
                        "ability hit counts are not universal cast-count estimates",
                        "two frozen allocations; no artifact or rotation optimization"],
    }
    (OUT / "comparison.json").write_text(json.dumps(report, indent=2) + "\n")
    for row in report["conditions"]:
        print(row["name"], round(row["team"]["mean"], 2), flush=True)
    print(json.dumps(contrasts, indent=2), flush=True)


if __name__ == "__main__":
    main()
