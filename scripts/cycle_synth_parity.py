"""gcsim parity of the cycle-synthesis pilot's served selections (after confirm).

Builds a temporary Go tool inside a Gscouter checkout (internal/score, gcsim
cenoda/gcsim@eb218847f82b via Gscouter's go.mod, no fillers), deletes the tool
source right after the build, and runs L, A and the selected program of every
arm at 384 CPU-s: 256 iterations, seed 777000, warmup 150 s. The Gscouter tree
is left clean (checked).

Usage: python3 scripts/cycle_synth_parity.py --gscouter /home/user/Gscouter \
         --rundir reports/cycle-synthesis-2026-11-06 --results runs/cycle-synth/results \
         --experiments reports/cycle-synthesis-2026-11-06/experiments.json --dest <json>
"""
import argparse
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
WORK = ROOT / "runs/cycle-synth/parity"
TOOL = r'''package main

import (
	"context"
	"encoding/json"
	"os"

	"github.com/cenoda/gscouter/internal/score"
)

type item struct {
	Name string `json:"name"`
	Path string `json:"path"`
}

func main() {
	var items []item
	b, _ := os.ReadFile(os.Args[1])
	if err := json.Unmarshal(b, &items); err != nil {
		panic(err)
	}
	out := map[string]map[string]any{}
	for _, it := range items {
		cfg, err := os.ReadFile(it.Path)
		if err != nil {
			out[it.Name] = map[string]any{"error": err.Error()}
			continue
		}
		r, err := score.Run(context.Background(), string(cfg), score.Options{Iterations: 256, Warmup: 150, Seed: 777000})
		if err != nil {
			out[it.Name] = map[string]any{"error": err.Error()}
			continue
		}
		out[it.Name] = map[string]any{"dps": r.DPS, "se": r.StdErr}
	}
	o, _ := json.MarshalIndent(out, "", " ")
	os.WriteFile(os.Args[2], o, 0o644)
}
'''


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--gscouter", required=True)
    ap.add_argument("--rundir", required=True)
    ap.add_argument("--results", required=True)
    ap.add_argument("--experiments", required=True)
    ap.add_argument("--dest", required=True)
    ap.add_argument("--bin", default=str(ROOT / "target/release/gso"))
    a = ap.parse_args()
    g = Path(a.gscouter)
    WORK.mkdir(parents=True, exist_ok=True)
    tool_dir = g / "cmd" / "zz-optimizer-parity"
    tool_dir.mkdir(parents=True)
    try:
        (tool_dir / "main.go").write_text(TOOL)
        subprocess.run(["go", "build", "-o", str(WORK / "parity"), "./cmd/zz-optimizer-parity"], cwd=g, check=True)
    finally:
        shutil.rmtree(tool_dir)
    status = subprocess.run(["git", "status", "--porcelain"], cwd=g, capture_output=True, text=True).stdout
    assert status.strip() == "", f"Gscouter tree not clean: {status}"
    ex = json.loads(Path(a.experiments).read_text())["cases"]
    items, native = [], {}
    rundir, results = Path(a.rundir), Path(a.results)
    for c in ex:
        case = c["case"]
        inp = rundir / "inputs" / case
        for name, rot in [("L", inp / "L.gcsl"), ("A", ROOT / "runs/heldout/withheld" / case / "A.gcsl")]:
            cfg = WORK / f"{case}.{name}.gcsl"
            subprocess.run([a.bin, "config", "--manifest", inp / "manifest.json", "--rotation", rot, "--out", cfg], check=True)
            items.append({"name": f"{case}|{name}", "path": str(cfg)})
        for arm, caps in c["arms"].items():
            e = caps.get("384", {})
            s = e.get("selected_start")
            if s is None:
                continue
            st = e["starts"][str(s)]
            native[f"{case}|{arm}"] = st.get("S")
            if st.get("audit_failure"):
                continue
            items.append({"name": f"{case}|{arm}", "path": str(results / case / arm / str(s) / "selected-384.gcsl")})
        sel = c["arms"]["V21S"]["384"]
        if sel.get("selected_start") is not None:
            st = sel["starts"][str(sel["selected_start"])]
            native[f"{case}|L"], native[f"{case}|A"] = st["L"], st["A"]
    (WORK / "items.json").write_text(json.dumps(items))
    subprocess.run([str(WORK / "parity"), str(WORK / "items.json"), str(WORK / "gcsim.json")], check=True)
    gc = json.loads((WORK / "gcsim.json").read_text())
    rows = []
    for c in ex:
        case = c["case"]
        row = {"case": case, "gcsim": {}, "native": {}}
        for k in ["L", "A", "V21S", "SYN", "XO"]:
            if f"{case}|{k}" in gc:
                row["gcsim"][k] = gc[f"{case}|{k}"]
                row["native"][k] = native.get(f"{case}|{k}")
        rows.append(row)
    Path(a.dest).write_text(json.dumps({"gcsim": "cenoda/gcsim@eb218847f82b via Gscouter internal/score, no fillers",
                                        "iterations": 256, "seed": 777000, "warmup": 150, "rows": rows}, indent=1) + "\n")
    refused = [(r["case"], k, v["error"][:120]) for r in rows for k, v in r["gcsim"].items() if "error" in v]
    print("parity rows", len(rows), "refusals", len(refused))
    for x in refused:
        print(" ", x)


if __name__ == "__main__":
    main()
