"""Publish compact measured artifacts, an exported plot, and one resumable run."""
import csv
import json
from pathlib import Path
import shutil
import statistics
import sys
import zipfile

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "reports/pilot-2026-11-05"


def main():
    old = json.loads((ROOT / "runs/matrix/summary.json").read_text())
    corrected = json.loads((ROOT / "runs/matrix-local-corrected/summary.json").read_text())
    valid = [r for r in old if "--local-only" not in r["name"]] + corrected
    compact = []
    for row in valid:
        reports = []
        for r in row["reports"]:
            selected = next(a for a in r["audits"] if a["key"] == r["provisional_best_key"])
            reports.append({k: r[k] for k in ["engine", "implementation_sha256", "manifest_sha256", "seed", "algorithm", "joint_er", "local_only",
                "discovery_cpu_seconds", "discovery_cpu_cap", "audit_cpu_seconds", "cap_overrun_seconds", "completed_decisions", "accepted", "audit_seed", "failed_fights", "baseline"]} | {"selected_before_audit": selected})
        compact.append({"name": row["name"], "reports": reports})
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "experiments.json").write_text(json.dumps(compact, indent=2) + "\n")
    (OUT / "excluded-local-starts.json").write_text(json.dumps([r for r in old if "--local-only" in r["name"]], indent=2) + "\n")
    labels = ["anneal", "greedy", "random", "--local-only", "--fixed-er"]
    names = ["Annealing + joint ER", "Greedy + joint ER", "Random + joint ER", "Annealing, one family", "Annealing, fixed ER"]
    grouped = []
    for label, name in zip(labels, names):
        rows = [r for r in compact if (label in r["name"] if label.startswith("--") else r["name"].split("-")[0] == label and "--" not in r["name"])]
        points = []
        for j, budget in enumerate([4, 12, 36]):
            values = [r["reports"][j]["selected_before_audit"]["joint"]["mean"] for r in rows]
            points.append({"cpu_cap_seconds": budget, "median_actual_cpu_seconds": statistics.median(r["reports"][j]["discovery_cpu_seconds"] for r in rows),
                           "median_dps": statistics.median(values), "minimum_dps": min(values), "maximum_dps": max(values), "all_starts_dps": values})
        grouped.append({"method": name, "points": points})
    (OUT / "budget-curves.json").write_text(json.dumps(grouped, indent=2) + "\n")
    with (OUT / "budget-curves.csv").open("w") as f:
        writer = csv.writer(f)
        writer.writerow(["method", "cpu_cap_seconds", "median_actual_cpu_seconds", "median_dps", "minimum_dps", "maximum_dps"])
        for row in grouped:
            for p in row["points"]:
                writer.writerow([row["method"]] + [p[k] for k in ["cpu_cap_seconds", "median_actual_cpu_seconds", "median_dps", "minimum_dps", "maximum_dps"]])
    sys.path.insert(0, str(ROOT / "runs/plot-packages"))
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    fig, ax = plt.subplots(figsize=(8.5, 4.8))
    for row in grouped:
        x = [p["median_actual_cpu_seconds"] for p in row["points"]]
        y = [p["median_dps"] / 1000 for p in row["points"]]
        line, = ax.plot(x, y, marker="o", label=row["method"])
        ax.fill_between(x, [p["minimum_dps"] / 1000 for p in row["points"]], [p["maximum_dps"] / 1000 for p in row["points"]], color=line.get_color(), alpha=.08)
    ax.axhline(338.426, color="black", linestyle="--", linewidth=1, label="Withheld library reference (not an optimum)")
    ax.set(xlabel="Cumulative measured discovery CPU seconds", ylabel="Independent audit team DPS (thousands)", title="Flins C1 / Ineffa C1 / Columbina C1 / Sucrose C6")
    ax.set_title("Flins C1 / Ineffa C1 / Columbina C1 / Sucrose C6\n3 starts per method; median and observed range", fontsize=11)
    ax.grid(alpha=.2)
    ax.legend(loc="center left", bbox_to_anchor=(1.01, .5), fontsize=8)
    fig.tight_layout()
    fig.savefig(OUT / "budget-curves.png", dpi=170, bbox_inches="tight")
    fig.savefig(OUT / "budget-curves.svg", bbox_inches="tight")
    for source, destination in [
        ("runs/enumerated-final/reference.json", "enumerated-reference.json"),
        ("runs/transfer-probes/report.json", "transfer-probes.json"),
        ("runs/final-confirmation/selected.gcsl", "discovered.gcsl"),
        ("runs/final-confirmation/selected.json", "discovered.json"),
        ("runs/final-confirmation/baseline.gcsl", "baseline.gcsl"),
        ("runs/final-confirmation/baseline-native.json", "baseline-confirmation.json"),
        ("runs/withheld-reference/withheld.gcsl", "withheld-reference.gcsl"),
        ("runs/withheld-reference/provenance.json", "withheld-provenance.json"),
    ]:
        shutil.copyfile(ROOT / source, OUT / destination)
    for name in ["baseline", "selected", "withheld"]:
        src = ROOT / f"runs/reference-{name}"
        dst = OUT / f"reference-{name}"
        shutil.copytree(src, dst, dirs_exist_ok=True)
    selected = json.loads((ROOT / "runs/final-confirmation/selection.json").read_text())
    run = Path(selected["source_run"])
    if not run.is_absolute():
        run = ROOT / run
    with zipfile.ZipFile(OUT / "recovery.zip", "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        for name in ["checkpoint.json", "evaluations.jsonl", "shortlist.json", "report.json"]:
            z.write(run / name, name)
        z.writestr("experiment.json", json.dumps({"source_commit": "4e09efb", "engine_commit": "1ba0af119e0477d6f44ee966a78fa06d84d2c5de",
            "command": "gso search --manifest data/flins-c1/standard.json --seconds 36 --seed 11 --algorithm anneal --fixed-er --out RESTORED --audit 128",
            "resume": "use source commit 4e09efb; a different executable fingerprint intentionally refuses this checkpoint"}, indent=2))
    accounting = {"valid_matrix_discovery_cpu_seconds": sum(r["reports"][-1]["discovery_cpu_seconds"] for r in valid),
        "valid_matrix_audit_cpu_seconds": sum(p["audit_cpu_seconds"] for r in valid for p in r["reports"]),
        "excluded_initial_local_discovery_cpu_seconds": sum(r["reports"][-1]["discovery_cpu_seconds"] for r in old if "--local-only" in r["name"]),
        "excluded_initial_local_audit_cpu_seconds": sum(p["audit_cpu_seconds"] for r in old if "--local-only" in r["name"] for p in r["reports"]),
        "preparation_cpu_seconds": sum(t["report"]["discovery_cpu_seconds"] + t["report"]["audit_cpu_seconds"] for t in json.loads((OUT / "preparation.json").read_text())["trials"]),
        "enumeration_cpu_seconds": json.loads((OUT / "enumerated-reference.json").read_text())["cpu_seconds"],
        "transfer_probe_cpu_seconds": sum(v["cpu_seconds"] for r in json.loads((OUT / "transfer-probes.json").read_text()) for v in r["results"].values() if "cpu_seconds" in v),
        "notes": "Add recorded native/gcsim verification CPU, exploratory smoke runs, build/tool setup and report generation. No VM-throughput or total-batch forecast is inferred."}
    (OUT / "accounting.json").write_text(json.dumps(accounting, indent=2) + "\n")
    print(OUT)


if __name__ == "__main__":
    main()
