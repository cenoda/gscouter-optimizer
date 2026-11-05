"""Freeze and export retry outcomes without feeding audits back to search."""
from collections import Counter, defaultdict
import csv
import json
from pathlib import Path
import shutil
import statistics
import subprocess
import sys
import zipfile

ROOT = Path(__file__).resolve().parents[1]
RUNS = ROOT / "runs/retry"
OUT = ROOT / "reports/pilot-retry-2026-11-05"


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    rows = json.loads((RUNS / "matrix/summary.json").read_text())
    compact = []
    diagnostics = []
    # Select by discovery estimates only. Do not inspect audited means here.
    def discovery_hint(row):
        state = json.loads((RUNS / "matrix" / f"{row['name']}-{row['seed']}" / "checkpoint.json").read_text())
        return state["best_hint"]
    winner = max(rows, key=discovery_hint)
    selected_run = RUNS / "matrix" / f"{winner['name']}-{winner['seed']}"
    selection = {"rule": "highest final discovery best_hint, frozen before new confirmation; audit scores unused",
                 "variant": winner["name"], "seed": winner["seed"],
                 "source_run": str(selected_run.relative_to(ROOT)),
                 "discovery_estimate": discovery_hint(winner),
                 "source_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()}
    for row in rows:
        run = RUNS / "matrix" / f"{row['name']}-{row['seed']}"
        state = json.loads((run / "checkpoint.json").read_text())
        reports = []
        for report in row["reports"]:
            a = next(a for a in report["audits"] if a["key"] == report["provisional_best_key"])
            reports.append({k: report[k] for k in ["implementation_sha256", "manifest_sha256", "seed",
                "revised", "joint_er", "discovery_cpu_seconds", "discovery_cpu_cap", "audit_cpu_seconds",
                "completed_decisions", "failed_fights", "baseline", "provisional_best_key"]} | {"selected": a})
        compact.append({"name": row["name"], "seed": row["seed"], "reports": reports,
                        "invariant_check": row["invariant_check"]})
        events = state["events"]
        groups = defaultdict(Counter)
        for e in events:
            operator = " ".join(e["operator"].split()[:2])
            group = groups[e["phase"] + ":" + operator]
            group["events"] += 1
            group["accepted"] += int(e["accepted"])
            group["failed_or_unresolved"] += int("unresolved" in e["kind"])
            group["exact_noop"] += int(e["kind"] == "exact no-op")
        diagnostics.append({"name": row["name"], "seed": row["seed"], "operators": dict(groups),
                            "pending": None if not state["pending"] else {k: state["pending"][k]
                            for k in ["operator", "polishing", "repairs_done"]},
                            "polish": state["polish"], "events": len(events),
                            "discovery_hint": state["best_hint"]})
        final = reports[-1]["selected"]
        shutil.copyfile(run / f"candidate-{final['index']}.gcsl", OUT / f"{row['name']}-{row['seed']}.gcsl")
    points = []
    for name in ["legacy-joint", "revised-joint", "revised-fixed"]:
        matching = [r for r in compact if r["name"] == name]
        for i, cap in enumerate([4, 12, 36, 108]):
            values = [r["reports"][i]["selected"]["joint"]["mean"] for r in matching]
            points.append({"variant": name, "cpu_cap": cap, "median_dps": statistics.median(values),
                           "minimum_dps": min(values), "maximum_dps": max(values), "values": values,
                           "median_actual_cpu": statistics.median(r["reports"][i]["discovery_cpu_seconds"] for r in matching)})
    for name, obj in [("experiments.json", compact), ("operators.json", diagnostics),
                      ("budget-curves.json", points), ("selection.json", selection)]:
        (OUT / name).write_text(json.dumps(obj, indent=2) + "\n")
    with (OUT / "budget-curves.csv").open("w") as f:
        writer = csv.DictWriter(f, fieldnames=["variant", "cpu_cap", "median_dps", "minimum_dps", "maximum_dps", "median_actual_cpu"])
        writer.writeheader()
        writer.writerows({k: p[k] for k in writer.fieldnames} for p in points)
    final_report = winner["reports"][-1]
    selected = next(a for a in final_report["audits"] if a["key"] == final_report["provisional_best_key"])
    selected_state = json.loads((selected_run / "checkpoint.json").read_text())
    shutil.copyfile(selected_run / f"candidate-{selected['index']}.gcsl", OUT / "selected.gcsl")
    (OUT / "selected.json").write_text(json.dumps(selected_state["best"], indent=2) + "\n")
    with zipfile.ZipFile(OUT / "recovery.zip", "w", zipfile.ZIP_DEFLATED) as z:
        for name in ["checkpoint.json", "evaluations.jsonl", "proposals.jsonl", "shortlist.json", "report.json"]:
            z.write(selected_run / name, name)
        z.writestr("experiment.json", json.dumps(selection, indent=2))
    for source, target in [(RUNS / "restricted/production-recovery.json", "production-recovery.json"),
                           (RUNS / "smoke-recovery.json", "smoke-recovery.json"),
                           (RUNS / "discarded-initialization-smoke/recovery.json", "discarded-smoke.json")]:
        shutil.copyfile(source, OUT / target)
    restricted = json.loads((OUT / "production-recovery.json").read_text())
    smoke = json.loads((OUT / "smoke-recovery.json").read_text())
    discarded = json.loads((OUT / "discarded-smoke.json").read_text())
    accounting = {"matrix_discovery_cpu_seconds": sum(r["reports"][-1]["discovery_cpu_seconds"] for r in rows),
                  "matrix_audit_cpu_seconds": sum(p["audit_cpu_seconds"] for r in rows for p in r["reports"]),
                  "restricted_discovery_cpu_seconds": sum(r["report"]["discovery_cpu_seconds"] for r in restricted["runs"]),
                  "restricted_audit_cpu_seconds": sum(r["report"]["audit_cpu_seconds"] for r in restricted["runs"]),
                  "enumeration_cpu_seconds": restricted["reference"]["cpu_seconds"],
                  "smoke_discovery_cpu_seconds": smoke["after_cpu"], "smoke_audit_cpu_seconds": smoke["audit_cpu_seconds"],
                  "discarded_smoke_discovery_cpu_seconds": discarded["after_cpu"],
                  "discarded_smoke_audit_cpu_seconds": discarded["audit_cpu_seconds"],
                  "notes": "Confirmation/parity costs will be recorded separately; builds, parent orchestration, reporting and wall overhead are not included in these process CPU sums. No throughput or cloud-cost extrapolation."}
    (OUT / "accounting.json").write_text(json.dumps(accounting, indent=2) + "\n")
    try:
        sys.path.insert(0, str(ROOT / "runs/plot-packages"))
        import matplotlib
        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
        fig, ax = plt.subplots(figsize=(7.5, 4.5))
        for name in ["legacy-joint", "revised-joint", "revised-fixed"]:
            p = [r for r in points if r["variant"] == name]
            x = [r["median_actual_cpu"] for r in p]
            line, = ax.plot(x, [r["median_dps"] / 1000 for r in p], marker="o", label=name)
            ax.fill_between(x, [r["minimum_dps"] / 1000 for r in p], [r["maximum_dps"] / 1000 for r in p], alpha=.12, color=line.get_color())
        ax.set(xlabel="Cumulative discovery CPU seconds", ylabel="Independent audit team DPS (thousands)",
               title="Greedy retry: median and observed range across three starts")
        ax.grid(alpha=.2)
        ax.legend()
        fig.tight_layout()
        for ext in ["png", "svg"]:
            fig.savefig(OUT / f"budget-curves.{ext}", dpi=160)
    except ImportError:
        pass
    print(json.dumps({"selection": selection, "points": points}, indent=2))


if __name__ == "__main__":
    main()
