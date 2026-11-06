#!/usr/bin/env python3
"""Does the v2.1 search do better from a synthesized start than from L?

Research only, development cases, one seed, small CPU. The unchanged v2.1
binary (`gso search --heldout-v2 --algorithm anneal --fixed-er`, per-case
v2.1 dictionary = global v2.1 dictionary minus the case's v2 drop list) is
run for CAP CPU-seconds from three different start programs:
  L  the library rotation (as in v2.1),
  K  the best generic skeleton from skeleton.json,
  T  the best whole-cycle transplant from transplant.json.
The search treats its start as its baseline, so K and T runs never see L.
The selected program of each run is measured on the 32 audit fights of
lsa.json (same fights as L, S and A there). This is a feasibility probe, not
an equal-CPU comparison of a designed method: K and T were themselves chosen
with extra CPU (skeleton/transplant enumeration) that is not charged here.
"""
import json
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
V2 = ROOT / "reports/heldout-recovery-v2-2026-11-05"
sys.path.insert(0, str(HERE))
from skeleton_probe import measure  # noqa: E402

CASES = ["mavuika-chevreuse-ororon-bennett--Mw9HdnKTkBfJ", "arlecchino-bennett-citlali-xilonen--DnLT9CbL979h",
         "cyno-furina-nahida-xilonen--jBkf669CgWCr", "mualani-candace-sucrose-xiangling--WDNJCKpjFNqQ",
         "neuvillette-furina-citlali-xilonen--gLgzMjmTN8z6", "wriothesley-kaedeharakazuha-shenhe-xilonen--bjFtkcfmqph7"]
CAP, SEED = 96, 11


def run(gso, case, start, text, work):
    inp = V2 / "inputs" / case
    d = work / case / start
    d.mkdir(parents=True, exist_ok=True)
    (d / "start.gcsl").write_text(text)
    dic = d / "dictionary.json"
    subprocess.run([gso, "dictionary-v2", "drop", "--dictionary", V2 / "v2.1/inputs/dictionary-v2.json",
                    "--drops", inp / "drops-v2.json", "--out", dic], check=True, capture_output=True)
    p = subprocess.run([gso, "search", "--manifest", inp / "manifest.json", "--baseline", d / "start.gcsl",
                        "--dictionary", dic, "--fixed-er", "--seed", str(SEED), "--seconds", str(CAP),
                        "--out", d / "run", "--audit", "32", "--heldout-v2", "--algorithm", "anneal"],
                       capture_output=True, text=True)
    if p.returncode:
        return {"error": p.stderr[-400:]}
    state = json.loads((d / "run/checkpoint.json").read_text())
    prog = state["best"]["program"]
    src = prog.get("Source") or prog.get("Library")
    rep = json.loads((d / "run/report.json").read_text())
    return {"selected_source": src, "steps": state["step"], "report_keys": list(rep.keys())[:20]}


def main():
    gso = sys.argv[1]
    work = Path(sys.argv[2])
    lsa = {r["case"]: r for r in json.loads((HERE / "lsa.json").read_text())}
    sk = {r["case"]: r for r in json.loads((HERE / "skeleton.json").read_text())}
    tr = {r["case"]: r for r in json.loads((HERE / "transplant.json").read_text())}
    jobs = []
    for c in CASES:
        L = (V2 / "inputs" / c / "L.gcsl").read_text()
        for start, text in (("L", L), ("K", sk[c]["best"]["text"]), ("T", tr[c]["best"]["text"])):
            jobs.append((c, start, text))
    with ThreadPoolExecutor(4) as ex:
        results = list(ex.map(lambda j: (j, run(gso, j[0], j[1], j[2], work)), jobs))
    out = []
    for (c, start, text), r in results:
        row = lsa[c]
        inp = V2 / "inputs" / c
        start_m = measure(gso, inp / "manifest.json", text, row["measure_seed"], 32)
        sel_m = measure(gso, inp / "manifest.json", r["selected_source"], row["measure_seed"], 32) if "selected_source" in r else {}
        A = row["programs"]["A"]["measured"]["mean"]
        rec = {"case": c, "start": start, "cap": CAP, "seed": SEED, "decisions": r.get("steps"), "error": r.get("error"),
               "start_over_A": start_m.get("mean", 0) / A, "selected_over_A": sel_m.get("mean", 0) / A,
               "selected_source": r.get("selected_source")}
        out.append(rec)
        print(f"{c[:42]:42s} start={start} start={rec['start_over_A']:.3f}A selected@{CAP}s={rec['selected_over_A']:.3f}A decisions={rec['decisions']} {rec['error'] or ''}", flush=True)
    (HERE / "search_start.json").write_text(json.dumps(out, indent=1))


if __name__ == "__main__":
    main()
