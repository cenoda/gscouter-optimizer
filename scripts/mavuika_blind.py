"""Frozen C6 build-only search; reveal library cycles only after selection."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import resource
import shutil
import subprocess
import time
import zipfile

from check_peaks import check as check_peaks
from check_run import check as check_run
from smoke_discovery import IDENTITY_FILES
from verify_reference import verify

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "reports/mavuika-c6-blind-2026-11-05"
RUNS = ROOT / "runs/mavuika-c6-blind"
IMPLEMENTATION = "79123f73118bf79daf5e723273ce316b691c02981f9cad58692311bf5fb76da4"
SEED = 12_000_000_000_000
SAMPLES = 2048


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def call(command):
    p = subprocess.run(list(map(str, command)), cwd=ROOT, text=True, capture_output=True)
    if p.returncode:
        raise RuntimeError(p.stderr.strip() or p.stdout.strip())
    return p


def prepare():
    if (RUNS / "sealed.json").exists():
        raise RuntimeError("use fresh experiment directories; a sealed experiment already exists")
    identity = hashlib.sha256("\n".join((ROOT / p).read_text() for p in IDENTITY_FILES).encode()).hexdigest()
    assert identity == IMPLEMENTATION
    library = ROOT.parent / "gscouter/teams/mavuika-citlali-xilonen-nicole"
    sealed = {}
    for name, source in [("original", library / "rotation.gcsl"),
                         ("c6-alternative", library / "alt/LhrRtbWNJk8t/rotation.gcsl")]:
        dest = RUNS / "withheld" / f"{name}.gcsl"
        dest.parent.mkdir(parents=True, exist_ok=True)
        # Bytes are sealed without printing or parsing their action sequence.
        shutil.copyfile(source, dest)
        sealed[name] = {"source": str(source), "sha256": sha(dest)}
    write(RUNS / "sealed.json", sealed)
    write(OUT / "frozen.json", {"implementation": identity, "script_sha256": sha(Path(__file__)),
          "protocol_sha256": sha(OUT / "protocol.md"), "withheld": sealed,
          "reference_binary_sha256": sha(ROOT / "runs/gcsim-reference"),
          "manifests": {f"r{r}": sha(ROOT / f"data/mavuika-c6-blind/r{r}.json") for r in [1, 5]}})
    with zipfile.ZipFile(OUT / "source.zip", "w", zipfile.ZIP_DEFLATED) as z:
        files = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "README.md", ROOT / "CLAUDE.md",
                 *ROOT.glob("src/*.rs"), *ROOT.glob("data/**/*"), *ROOT.glob("docs/*.md"),
                 *ROOT.glob("scripts/*.py"), ROOT / "examples/trace.rs", OUT / "protocol.md"]
        for p in files:
            if p.is_file():
                z.write(p, p.relative_to(ROOT))


def trial(refine, seed):
    dest = RUNS / f"r{refine}-{seed}"
    manifest = ROOT / f"data/mavuika-c6-blind/r{refine}.json"
    row = {"refinement": refine, "seed": seed, "reports": []}
    try:
        for cap in [4, 12, 36, 108]:
            call([ROOT / "target/release/gso", "search", "--from-scratch", "--fixed-er",
                  "--algorithm", "greedy", "--manifest", manifest, "--seconds", cap,
                  "--seed", seed, "--out", dest, "--audit", 128])
            report = json.loads((dest / "report.json").read_text())
            assert report["implementation_sha256"] == IMPLEMENTATION
            assert report["from_scratch"] and report["reference_origin"] == "generated starting control"
            write(dest / f"report-{cap}.json", report)
            selected = next(a for a in report["audits"] if a["key"] == report["provisional_best_key"])
            row["reports"].append(report)
            print(f"blind R{refine} seed {seed} cap {cap}: "
                  f"{round(selected['joint']['mean']) if 'joint' in selected else selected['status']}", flush=True)
        row["invariant_check"] = check_run(dest)
        row["peak_check"] = check_peaks(dest)
        row["status"] = "complete"
    except Exception as exc:
        row["status"] = "failed/unresolved"
        row["error"] = str(exc)
        write(dest / "failure.json", {"error": str(exc)})
        print(f"blind R{refine} seed {seed}: failed/unresolved: {exc}", flush=True)
    return row


def matrix():
    def condition(refine):
        return [trial(refine, seed) for seed in [11, 37, 91]]
    with ThreadPoolExecutor(max_workers=2) as pool:
        rows = [row for group in pool.map(condition, [1, 5]) for row in group]
    write(OUT / "matrix.json", rows)
    selection = {"rule": "highest final discovery best_hint among completed starts at 108 CPU seconds; audits unused",
                 "implementation": IMPLEMENTATION, "conditions": {}}
    for refine in [1, 5]:
        completed = [r for r in rows if r["refinement"] == refine and r["status"] == "complete"]
        states = [(r, json.loads((RUNS / f"r{refine}-{r['seed']}/checkpoint.json").read_text())) for r in completed]
        if not states:
            selection["conditions"][f"r{refine}"] = {"status": "no completed blind run"}
            continue
        row, state = max(states, key=lambda pair: pair[1]["best_hint"])
        report = row["reports"][-1]
        audited = next(a for a in report["audits"] if a["key"] == report["provisional_best_key"])
        source = RUNS / f"r{refine}-{row['seed']}" / f"candidate-{audited['index']}.gcsl"
        dest = OUT / f"selected-r{refine}.gcsl"
        shutil.copyfile(source, dest)
        write(OUT / f"selected-r{refine}.json", state["best"])
        selection["conditions"][f"r{refine}"] = {"status": "frozen", "seed": row["seed"],
            "key": report["provisional_best_key"], "discovery_estimate": state["best_hint"], "source_sha256": sha(dest)}
    # The sole gate before parsing or simulating a reference.
    write(OUT / "selection.json", selection)


def native(config, dest):
    dest.parent.mkdir(parents=True, exist_ok=True)
    command = [ROOT / "target/release/examples/trace", config, SAMPLES, SEED]
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    start = time.monotonic()
    p = subprocess.run(list(map(str, command)), cwd=ROOT, text=True, capture_output=True)
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    accounting = {"cpu_seconds": after.ru_utime + after.ru_stime - before.ru_utime - before.ru_stime,
                  "wall_seconds": time.monotonic() - start, "command": list(map(str, command))}
    if p.returncode:
        result = {"status": "failed/unresolved", "error": p.stderr.strip(), "accounting": accounting}
    else:
        result = {"status": "complete", "result": json.loads(p.stdout), "accounting": accounting}
    write(dest, result)
    return result


def confirm():
    selection = json.loads((OUT / "selection.json").read_text())
    frozen = json.loads((OUT / "frozen.json").read_text())
    assert sha(Path(__file__)) == frozen["script_sha256"]
    assert sha(OUT / "protocol.md") == frozen["protocol_sha256"]
    results = {}
    for refine in [1, 5]:
        conditions = {}
        if selection["conditions"][f"r{refine}"]["status"] == "frozen":
            config = OUT / f"selected-r{refine}.gcsl"
            assert sha(config) == selection["conditions"][f"r{refine}"]["source_sha256"]
            conditions["selected"] = native(config, OUT / f"confirmation/r{refine}-selected.json")
        for name in ["original", "c6-alternative"]:
            raw = RUNS / f"withheld/{name}.gcsl"
            assert sha(raw) == frozen["withheld"][name]["sha256"]
            dest = RUNS / f"reference-export-r{refine}-{name}"
            try:
                call([ROOT / "target/release/gso", "search", "--fixed-er", "--local-only",
                      "--algorithm", "greedy", "--manifest", ROOT / f"data/mavuika-c6-blind/r{refine}.json",
                      "--baseline", raw, "--seconds", "0.000000001", "--seed", "231", "--out", dest, "--audit", "1"])
                export = json.loads((dest / "report.json").read_text())
                assert export["completed_decisions"] == 0
                config = OUT / f"reference-r{refine}-{name}.gcsl"
                shutil.copyfile(dest / "candidate-0.gcsl", config)
                conditions[name] = native(config, OUT / f"confirmation/r{refine}-{name}.json")
            except Exception as exc:
                conditions[name] = {"status": "failed/unresolved", "error": str(exc)}
        comparisons = {}
        selected = conditions.get("selected", {})
        if selected.get("status") == "complete":
            a = selected["result"]["per_iteration_dps"]
            for name in ["original", "c6-alternative"]:
                if conditions[name]["status"] == "complete":
                    b = conditions[name]["result"]["per_iteration_dps"]
                    ds = [x - y for x, y in zip(a, b, strict=True)]
                    mean = sum(ds) / len(ds)
                    se = (sum((d - mean) ** 2 for d in ds) / (len(ds) * (len(ds) - 1))) ** 0.5
                    comparisons[name] = {"paired_difference": mean, "paired_se": se,
                                         "relative_difference": sum(a) / sum(b) - 1}
        results[f"r{refine}"] = {"conditions": conditions, "selected_minus_reference": comparisons}
        print(f"confirmed R{refine}: " + ", ".join(f"{k}={round(v['result']['team']['mean']) if v['status']=='complete' else v['status']}" for k,v in conditions.items()), flush=True)
    write(OUT / "confirmation.json", {"samples": SAMPLES, "seed": SEED, "conditions": results})
    parity = {}
    for refine in [1, 5]:
        for name in ["selected", "c6-alternative"]:
            if results[f"r{refine}"]["conditions"].get(name, {}).get("status") != "complete":
                continue
            config = OUT / (f"selected-r{refine}.gcsl" if name == "selected" else f"reference-r{refine}-{name}.gcsl")
            try:
                parity[f"r{refine}-{name}"] = verify(str(config), str(OUT / f"parity/r{refine}-{name}"), SAMPLES, SEED)
            except Exception as exc:
                parity[f"r{refine}-{name}"] = {"status": "failed/unresolved", "error": str(exc)}
    write(OUT / "parity.json", parity)


def archive():
    with zipfile.ZipFile(OUT / "runs.zip", "w", zipfile.ZIP_DEFLATED) as z:
        for p in sorted(RUNS.rglob("*")):
            if p.is_file() and p.name not in ["worker-input.json", "worker-output.json"]:
                z.write(p, p.relative_to(RUNS))
        for p in (ROOT / "data/mavuika-c6-blind").glob("*"):
            z.write(p, Path("inputs") / p.name)
    hashes = {}
    for name in ["runs.zip", "source.zip"]:
        p = OUT / name
        with zipfile.ZipFile(p) as z:
            assert z.testzip() is None
        hashes[name] = {"sha256": sha(p), "bytes": p.stat().st_size, "crc_verified": True}
    write(OUT / "archives.json", hashes)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=["run", "confirm", "archive"])
    args = parser.parse_args()
    if args.phase == "run":
        prepare()
        matrix()
        confirm()
        archive()
    elif args.phase == "confirm":
        confirm()
        archive()
    else:
        archive()


if __name__ == "__main__":
    main()
