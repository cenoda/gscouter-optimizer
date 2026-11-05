"""Bounded execution checks for build-only discovery, not a quality benchmark."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import zipfile

from check_run import check as check_run
from check_peaks import check as check_peaks

ROOT = Path(__file__).resolve().parents[1]
BIN = ROOT / "target/release/gso"
ENGINE = "1ba0af119e0477d6f44ee966a78fa06d84d2c5de"
IDENTITY_FILES = ["src/artifact.rs", "src/program.rs", "src/discovery.rs", "src/evaluate.rs",
                  "src/search.rs", "data/character-rarities.json", "Cargo.lock", "Cargo.toml",
                  "docs/pilot-protocol.md", "docs/pilot-retry.md", "docs/from-scratch.md"]
CASES = {
    "electro": [
        ("sucrose", "sacrificialfragments", "viridescentvenerer"),
        ("fischl", "thestringless", "thunderingfury"),
        ("xingqiu", "sacrificialsword", "emblemofseveredfate"),
        ("beidou", "favoniusgreatsword", "emblemofseveredfate"),
    ],
    "geo": [
        ("noelle", "whiteblind", "huskofopulentdreams"),
        ("gorou", "favoniuswarbow", "noblesseoblige"),
        ("yunjin", "favoniuslance", "huskofopulentdreams"),
        ("fischl", "thestringless", "thunderingfury"),
    ],
}


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def prepare(path, party):
    path.parent.mkdir(parents=True)
    build = []
    members = []
    for key, weapon, artifact_set in party:
        build.extend([
            f'{key} char lvl=90/90 cons=6 talent=9,9,9;',
            f'{key} add weapon="{weapon}" refine=5 lvl=90/90;',
            f'{key} add set="{artifact_set}" count=4;',
            f'{key} add stats hp=4780 atk=311 atk%=0.5 def%=0.6 cr=0.5 cd=1.0 er=1.0 em=100;',
        ])
        members.append({"key": key, "cons": 6, "weapon": weapon, "refinement": 5,
                        "set": artifact_set, "useful": [], "weights": {}, "pieces": []})
    write_json(path, {"format": 2, "engine": ENGINE,
                     "site_commit": "synthetic execution fixture, not site data",
                     "sources": {}, "max_rolls": {}, "members": members,
                     "fixed_build": "\n".join(build) + "\n"})


def search(manifest, out, seed, cap):
    command = [str(BIN), "search", "--from-scratch", "--fixed-er", "--algorithm", "greedy",
               "--manifest", str(manifest), "--out", str(out), "--seed", str(seed),
               "--seconds", str(cap), "--audit", "32"]
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(f"{' '.join(command)}\n{result.stderr}")
    return json.loads((out / "report.json").read_text())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=ROOT / "runs/discovery-smoke")
    parser.add_argument("--report", type=Path, default=ROOT / "reports/discovery-smoke-2026-11-05")
    args = parser.parse_args()
    # Refuse stale identities; continuation below is deliberate and tested.
    if args.out.exists() or args.report.exists():
        raise RuntimeError("use fresh output and report directories")
    args.out.mkdir(parents=True)
    args.report.mkdir(parents=True)
    expected_identity = hashlib.sha256("\n".join((ROOT / p).read_text() for p in IDENTITY_FILES).encode()).hexdigest()
    # Include the dirty working source: a HEAD commit alone cannot reproduce it.
    with zipfile.ZipFile(args.report / "source.zip", "w", zipfile.ZIP_DEFLATED) as archive:
        paths = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "README.md", ROOT / "CLAUDE.md",
                 *ROOT.glob("src/*.rs"), *ROOT.glob("docs/*.md"), *ROOT.glob("data/**/*"),
                 ROOT / "scripts/smoke_discovery.py", ROOT / "scripts/check_run.py", ROOT / "scripts/check_peaks.py"]
        for path in paths:
            if path.is_file():
                archive.write(path, path.relative_to(ROOT))
    rows = []
    hashes = set()
    for name, party in CASES.items():
        manifest = args.out / "inputs" / name / "manifest.json"
        prepare(manifest, party)
        assert not manifest.with_name("baseline.gcsl").exists()
        for seed in [11, 37, 91]:
            out = args.out / f"{name}-{seed}"
            points = []
            for cap in [4, 12]:
                report = search(manifest, out, seed, cap)
                hashes.add(report["implementation_sha256"])
                assert report["implementation_sha256"] == expected_identity, "stale binary or changed source"
                assert report["from_scratch"] and report["reference_origin"] == "generated starting control"
                selected = next(a for a in report["audits"] if a["key"] == report["provisional_best_key"])
                stage = report["stage_selections"][-1] if report["stage_selections"] else None
                stage_audit = None
                if stage and stage["budget_cpu_cap"] == cap:
                    # Candidate identity includes the manifest, engine and source.
                    checkpoint = json.loads((out / "checkpoint.json").read_text())
                    candidate = stage["candidate"]
                    stage_audit = next((a for a, c in zip(report["audits"], json.loads((out / "shortlist.json").read_text()), strict=True)
                                        if c["program"] == candidate["program"] and c["allocation"] == candidate["allocation"]), None)
                    assert checkpoint["stage_selections"] == report["stage_selections"]
                    assert stage_audit is not None
                points.append({"cap": cap, "discovery_cpu": report["discovery_cpu_seconds"],
                               "audit_cpu": report["audit_cpu_seconds"], "selected": selected,
                               "generated_control": report["baseline"],
                               "before_polish": stage_audit, "decisions": report["completed_decisions"],
                               "failed_fights": report["failed_fights"]})
                write_json(out / f"report-{cap}.json", report)
                print(name, seed, cap, round(selected["joint"]["mean"]), flush=True)
            state = json.loads((out / "checkpoint.json").read_text())
            assert all("Generated" in c["current"]["program"] for c in state["chains"])
            for line in (out / "proposals.jsonl").read_text().splitlines():
                assert "Generated" in json.loads(line)["candidate"]["program"]
            experiment = json.loads((out / "experiment.json").read_text())
            assert experiment["baseline_read"] is False
            operators = Counter(e["operator"] for e in state["events"])
            row = {"case": name, "seed": seed, "points": points, "operators": dict(operators),
                   "restart_decisions": sum(e["kind"] == "restart" for e in state["events"]),
                   "invariant_check": check_run(out), "peak_check": check_peaks(out)}
            selected = points[-1]["selected"]
            shutil.copyfile(out / f"candidate-{selected['index']}.gcsl", args.report / f"{name}-{seed}.gcsl")
            rows.append(row)
    assert len(hashes) == 1, "implementation changed during smoke execution"
    assert sum(r["restart_decisions"] for r in rows) > 0

    # Resume a copied real pending proposal, preserving all completed events.
    source = args.out / "recovery-origin"
    manifest = args.out / "inputs/geo/manifest.json"
    attempts = [search(manifest, source, 231, 1)]
    before = json.loads((source / "checkpoint.json").read_text())
    for _ in range(8):
        if before["pending"]:
            break
        cap = max(attempts[-1]["discovery_cpu_cap"], before["parent_cpu"]) + 0.01
        attempts.append(search(manifest, source, 231, cap))
        before = json.loads((source / "checkpoint.json").read_text())
    assert before["pending"] is not None, "no pending proposal within bounded recovery attempts"
    write_json(source / "recovery-attempts.json", attempts)
    restored = args.out / "restored"
    shutil.copytree(source, restored)
    recovery_report = search(manifest, restored, 231, before["parent_cpu"] + 1)
    assert recovery_report["implementation_sha256"] == expected_identity
    after = json.loads((restored / "checkpoint.json").read_text())
    assert after["events"][:len(before["events"])] == before["events"]
    assert after["step"] > before["step"]
    assert after["rng"] != before["rng"]
    if before["pending"]:
        assert after["events"][len(before["events"])]["operator"] == before["pending"]["operator"]
    recovery = {"before_decisions": before["step"], "after_decisions": after["step"],
                "had_pending": before["pending"] is not None,
                "pending_operator": None if not before["pending"] else before["pending"]["operator"],
                "prior_events_preserved": True, "rng_advanced": True,
                "extra_attempt_audit_cpu": sum(r["audit_cpu_seconds"] for r in attempts),
                "invariant_check": check_run(restored), "peak_check": check_peaks(restored),
                "report": recovery_report}
    write_json(args.report / "summary.json", {"purpose": "execution and recovery only; synthetic builds; no quality reference",
               "implementation_sha256": next(iter(hashes)), "seeds": [11, 37, 91], "caps": [4, 12],
               "runs": rows, "recovery": recovery})
    with zipfile.ZipFile(args.report / "runs.zip", "w", zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(args.out.rglob("*")):
            if path.is_file() and path.name not in {"worker-input.json", "worker-output.json"}:
                archive.write(path, path.relative_to(args.out))
    archives = {}
    for name in ["runs.zip", "source.zip"]:
        archive_path = args.report / name
        with zipfile.ZipFile(archive_path) as archive:
            assert archive.testzip() is None
        archives[name] = {"sha256": hashlib.sha256(archive_path.read_bytes()).hexdigest(),
                          "bytes": archive_path.stat().st_size, "crc_verified": True}
    write_json(args.report / "archive.json", archives)
    print(f"Verified {len(rows)} build-only runs and copied-checkpoint recovery", flush=True)


if __name__ == "__main__":
    main()
