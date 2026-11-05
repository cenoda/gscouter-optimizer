"""Reconstruct each saved peak from actual common 32-fight journal blocks."""
from collections import defaultdict
import json
from pathlib import Path
import sys


def check(path):
    path = Path(path)
    report = json.loads((path / "report.json").read_text())
    if report.get("restricted_domain"):
        raise ValueError("restricted recovery uses its separately fixed objective")
    peak = report["shortlist_frozen_keys"][0]
    blocks = defaultdict(list)
    for line in (path / "evaluations.jsonl").read_text().splitlines():
        record = json.loads(line)
        if 1_000_000_000 <= record["seed"] < 4_000_000_000_000:
            blocks[record["seed"]].append(record)
    changes = complete = unresolved = 0
    for seed, records in sorted(blocks.items()):
        good = [r for r in records if len(r["work"]["samples"]) == 32
                and all("Ok" in v for v in r["work"]["samples"])]
        if len(good) == 2:
            candidate, incumbent = good
            assert incumbent["key"] == peak, (path.name, seed, "wrong incumbent")
            difference = sum(a["Ok"]["dps"] - b["Ok"]["dps"] for a, b in
                             zip(candidate["work"]["samples"], incumbent["work"]["samples"], strict=True)) / 32
            complete += 1
            if difference > 0:
                peak = candidate["key"]
                changes += 1
        elif len(good) == 1 and good[0]["key"] == peak:
            pass  # The proposal has the exact incumbent key, or failed.
        else:
            unresolved += 1
    assert peak == report["provisional_best_key"], path.name
    return {"run": path.name, "verified_peak_changes": changes,
            "complete_common_32_blocks": complete,
            "unresolved_or_incomplete_promotions": unresolved}


if __name__ == "__main__":
    print(json.dumps([check(p) for p in sys.argv[1:]], indent=2))
