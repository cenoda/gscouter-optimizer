"""gcsim parity check of v2 held-out selections (after the measurement).

Runs, per case, L, A and the selected S of V1@192, V2S@384 and V2T@384 through
a temporary Go tool built inside a Gscouter checkout (internal/score, gcsim
cenoda/gcsim@eb218847f82b, no fillers; 256 iterations, seed 777000, 450 s,
first 150 s excluded). The tool is deleted after building; Gscouter is left
clean. L and A use the case manifest's header (`gso config`).
"""
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
RUNS = ROOT / "runs/heldout-v2"
OUT = ROOT / "reports/heldout-recovery-v2-2026-11-05"
ARMS = [("V1", "192"), ("V2S", "384"), ("V2T", "384")]


def main(tool, gso):
    ex = json.loads((OUT / "experiments.json").read_text())["cases"]
    work = RUNS / "parity"
    work.mkdir(parents=True, exist_ok=True)
    items, meta = [], {}
    for c in ex:
        case = c["case"]
        inputs = OUT / "inputs" / case
        for name, rot in [("L", inputs / "L.gcsl"), ("A", ROOT / "runs/heldout/withheld" / case / "A.gcsl")]:
            cfg = work / f"{case}.{name}.gcsl"
            subprocess.run([gso, "config", "--manifest", inputs / "manifest.json", "--rotation", rot, "--out", cfg], check=True)
            items.append({"name": f"{case}|{name}", "path": str(cfg)})
        for arm, cap in ARMS:
            e = c["arms"][arm].get(cap, {})
            if "selected_start" not in e:
                continue
            s = e["starts"][str(e["selected_start"])]
            if s.get("audit_failure"):
                continue
            path = RUNS / "results" / case / arm / str(e["selected_start"]) / f"selected-{cap}.gcsl"
            items.append({"name": f"{case}|{arm}@{cap}", "path": str(path)})
            meta[f"{case}|{arm}@{cap}"] = s["S"]
        meta[f"{case}|L"] = c["arms"]["V2S"]["384"]["starts"][str(c["arms"]["V2S"]["384"]["selected_start"])]["L"]
        meta[f"{case}|A"] = c["arms"]["V2S"]["384"]["starts"][str(c["arms"]["V2S"]["384"]["selected_start"])]["A"]
    (work / "items.json").write_text(json.dumps(items))
    subprocess.run([tool, work / "items.json", work / "gcsim.json"], check=True)
    g = json.loads((work / "gcsim.json").read_text())
    rows = []
    for c in ex:
        case = c["case"]
        row = {"case": case, "class": c["class"], "native": {}, "gcsim": {}}
        for k in ["L", "A"] + [f"{a}@{p}" for a, p in ARMS]:
            if f"{case}|{k}" in g:
                row["gcsim"][k] = g[f"{case}|{k}"]
                row["native"][k] = meta.get(f"{case}|{k}")
        L, A = row["gcsim"].get("L", {}).get("dps"), row["gcsim"].get("A", {}).get("dps")
        for k in [f"{a}@{p}" for a, p in ARMS]:
            S = row["gcsim"].get(k, {}).get("dps")
            if S and L and A and A != L:
                row.setdefault("r_gcsim", {})[k] = (S - L) / (A - L)
        rows.append(row)
    (OUT / "gcsim-parity.json").write_text(json.dumps(
        {"gcsim": "cenoda/gcsim@eb218847f82b via Gscouter 6e0e82205 internal/score, no fillers",
         "iterations": 256, "seed": 777000, "warmup": 150, "rows": rows}, indent=1) + "\n")
    print("parity rows", len(rows))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
