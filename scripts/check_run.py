"""Check evidence of comparable completed acceptance blocks in a saved run."""
import json
import math
import sys
from pathlib import Path


def check(path):
    path = Path(path)
    state = json.loads((path / "checkpoint.json").read_text())
    cache = {}
    for line in (path / "evaluations.jsonl").read_text().splitlines():
        record = json.loads(line)
        for i, sample in enumerate(record["work"]["samples"]):
            cache[record["key"], record["seed"] + i] = sample
    decisions = 0
    for event in state["events"]:
        if event["kind"] not in ("chain", "baseline allocation", "polish"):
            assert not event["accepted"]
            continue
        assert event["samples"] >= 16
        means = []
        for key in (event["proposal_key"], event["current_key"]):
            samples = [cache[key, event["decision_seed"] + i]["Ok"] for i in range(event["samples"])]
            assert all(s["frames"] == 27000 for s in samples)
            means.append(sum(s["dps"] for s in samples) / len(samples))
        delta = (means[0] - means[1]) / state["reference_dps"]
        assert math.isclose(delta, event["delta"], abs_tol=1e-12)
        assert event["accepted"] == (delta >= event["threshold"])
        decisions += 1
    report = json.loads((path / "report.json").read_text())
    if "revised" in report:
        rows = [json.loads(line) for line in (path / "proposals.jsonl").read_text().splitlines()]
        proposed = {(r["step"],r["key"]) for r in rows}
        assert all((e["step"],e["proposal_key"]) in proposed for e in state["events"])
        if state["pending"]:
            assert any(r["step"]==state["step"] and r["candidate"]==state["pending"]["next"] for r in rows)
        if report.get("restricted_domain"):
            assert all(e["samples"]>=32 for e in state["events"] if e["kind"]=="chain")
    if report["local_only"]:
        assert state["events"][0]["current_key"] == report["shortlist_frozen_keys"][0], "local-only must share the original baseline start"
    assert len(report["audits"]) <= 6
    assert len(report["shortlist_frozen_keys"]) == len(set(report["shortlist_frozen_keys"]))
    assert report["audits"][0]["joint_minus_original_baseline"][0] == 0
    for audit in report["audits"]:
        if "joint" in audit:
            assert math.isclose(sum(audit["joint"]["chars"]), audit["joint"]["mean"], rel_tol=1e-10)
    return {"path": str(path), "verified_common_block_decisions": decisions, "accepted_events": sum(e["accepted"] for e in state["events"])}


if __name__ == "__main__":
    for path in sys.argv[1:]:
        print(json.dumps(check(path)))
