#!/usr/bin/env python3
"""Structural comparison of L, A and S on development cases (research only).

Reads A from the Gscouter checkout (read-only). Every case here is a
development case: its A was already read in an earlier report. Nothing here
feeds a search.

For each case it measures L, A and the selected v2.1 programs (V2S@384 and
V2T@384, the start with the highest discovery estimate) with `gso measure`
on the first N fights of that start's audit seed, and records per-member DPS,
per-member wait seconds by reason and static program features. It also runs
the generic mechanical probe (`mechprobe`, source in this directory) on the
case's build to obtain per-member action frames, cooldowns, burst cost,
particle energy and the statuses an action creates.

Usage:
  python3 lsa_structure.py --gso target/release/gso --probe <mechprobe> \
      --sel <unzipped v2.1 selections> --gscouter /home/user/Gscouter \
      --samples 32 --out reports/cycle-synthesis-research-2026-11-06/lsa.json
"""
import argparse
import json
import re
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
V2 = ROOT / "reports/heldout-recovery-v2-2026-11-05"

# (case, source of inputs, group)
CASES = [
    # v2 held-out cases (now development) below 90% of A in v2.1 V2S@384
    ("mavuika-chevreuse-ororon-bennett--Mw9HdnKTkBfJ", "v2", "low"),
    ("arlecchino-bennett-citlali-xilonen--DnLT9CbL979h", "v2", "low"),
    ("cyno-furina-nahida-xilonen--jBkf669CgWCr", "v2", "low"),
    ("mualani-candace-sucrose-xiangling--WDNJCKpjFNqQ", "v2", "low"),
    ("neuvillette-furina-citlali-xilonen--gLgzMjmTN8z6", "v2", "low"),
    ("mualani-mona-xilonen-mavuika--M8TgfHLLtqbp", "v2", "low"),
    # contrast: L also far below A (A/L >= 2), but v2.1 reached >= 95% of A
    ("wriothesley-furina-xilonen-bennett--PFGMCrLDpbcq", "v2", "solved"),
    ("yoimiya-furina-yunjin-xilonen--rk6QCLfwKJkD", "v2", "solved"),
    ("wriothesley-kaedeharakazuha-shenhe-xilonen--bjFtkcfmqph7", "v2", "solved"),
    ("raidenshogun-furina-xilonen-yelan--PhjwKhdjDwtb", "v2", "solved"),
    # stage-1 "cycle restructure" cases (v2 development inputs; no v2.1 S)
    ("clorinde-kujousara-nahida-xilonen--DQkpHtnCkKgR", "dev", "stage1-restructure"),
    ("flins-ineffa-furina-sucrose--j86qrckHDPmb", "dev", "stage1-restructure"),
    ("mualani-chiori-xiangling-xilonen--fKzfLbrbfhj9", "dev", "stage1-restructure"),
]

ACTIONS = ("attack", "charge", "skill", "burst", "dash", "jump", "walk", "aim",
           "high_plunge", "low_plunge", "swap")


def static_features(src, members):
    """Text-level features; counts are of written statements, not executions."""
    text = re.sub(r"#.*", "", src)
    feats = {"tokens": len(re.findall(r"[A-Za-z_][\w-]*|\d+|\S", text)),
             "time_box": bool(re.search(r"f\s*\(\s*\)\s*-", text)),
             "ready_guards": len(re.findall(r"\.(skill|burst)\s*\.(ready|cd)", text)),
             "status_guards": len(re.findall(r"\.(status|mods)\s*\.", text)),
             "element_guards": len(re.findall(r"\.element\s*\.", text)),
             "while_loops": len(re.findall(r"\bwhile\b", text)),
             "if_blocks": len(re.findall(r"\bif\b", text)),
             "waits": [int(x) for x in re.findall(r"\bwait\s*\(\s*(\d+)\s*\)", text)],
             "for_bounds": [int(x) for x in re.findall(r"<\s*(\d+)\s*;", text)]}
    alias = {m: {m} for m in members}
    for m in members:  # common short names in the library
        alias[m] |= {m[:4], m[:5]} if len(m) > 5 else set()
    per = {}
    for m in members:
        names = "|".join(sorted(map(re.escape, alias[m]), key=len, reverse=True))
        counts = {a: 0 for a in ACTIONS}
        for stmt in re.finditer(rf"(?<![\w.])(?:{names})\s+([^;{{}}]+);", text):
            for item in stmt.group(1).split(","):
                mm = re.match(r"\s*([a-z_]+)(?:\[[^\]]*\])?(?:\s*:\s*(\d+))?", item)
                if mm and mm.group(1) in counts:
                    counts[mm.group(1)] += int(mm.group(2) or 1)
        per[m] = {k: v for k, v in counts.items() if v}
    feats["written_actions"] = per
    return feats


def measure(gso, manifest, rotation_text, seed, n):
    with tempfile.TemporaryDirectory() as d:
        rot = Path(d) / "r.gcsl"
        rot.write_text(rotation_text)
        p = subprocess.run([gso, "measure", "--manifest", str(manifest), "--rotation", str(rot),
                            "--seed", str(seed), "--samples", str(n), "--out", str(Path(d) / "e")],
                           capture_output=True, text=True)
        if p.returncode:
            return {"error": p.stderr.strip()[-300:]}
        r = json.loads(p.stdout)
        st = r["stats"]
        return {"mean": st["mean"], "se": st["se"], "chars": st["chars"],
                "waits": st["waits"], "cpu_seconds": r["cpu_seconds"], "config_sha256": r["config_sha256"]}


def selected(sel_root, exp_case, arm):
    cp = exp_case["arms"][arm]["384"]
    ss = {k: s for k, s in cp["starts"].items() if s.get("status") == "complete"}
    seed = max(ss, key=lambda k: ss[k]["estimate"])
    j = json.loads((sel_root / exp_case["case"] / arm / seed / "selected-384.json").read_text())
    return seed, ss[seed], j["candidate"]["program"]["Source"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--gso", required=True)
    ap.add_argument("--probe", required=True)
    ap.add_argument("--sel", required=True, type=Path)
    ap.add_argument("--gscouter", required=True, type=Path)
    ap.add_argument("--samples", type=int, default=32)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    v21 = {c["case"]: c for c in json.loads((V2 / "v2.1/experiments.json").read_text())["cases"]}
    out = []
    for case, src, group in CASES:
        inp = (V2 / "inputs" / case) if src == "v2" else (V2 / "development/inputs" / case)
        manifest = inp / "manifest.json"
        members = [m["key"] for m in json.loads(manifest.read_text())["members"]]
        party, alt = case.split("--")
        L = (inp / "L.gcsl").read_text()
        A = (a.gscouter / "teams" / party / "alt" / alt / "rotation.gcsl").read_text()
        progs = {"L": L, "A": A}
        row = {"case": case, "group": group, "members": members}
        if src == "v2":
            e = v21[case]
            seed, s, S = selected(a.sel, e, "V2S")
            progs["S_V2S384"] = S
            row["selected"] = {"V2S": {"seed": seed, "audit_seed": s["audit_seed"],
                                       "S_audit": s["S_effective"], "A_audit": s["A"], "L_audit": s["L"]}}
            seed_t, t, T = selected(a.sel, e, "V2T")
            progs["S_V2T384"] = T
            row["selected"]["V2T"] = {"seed": seed_t, "audit_seed": t["audit_seed"], "S_audit": t["S_effective"]}
            audit_seed = s["audit_seed"]
        else:
            audit_seed = 4000000000000  # any fixed seed; development measurement only
        row["measure_seed"] = audit_seed
        row["programs"] = {}
        for k, text in progs.items():
            row["programs"][k] = {"static": static_features(text, members),
                                  "measured": measure(a.gso, manifest, text, audit_seed, a.samples)}
        # mechanical probe on the case build (any config with the build works)
        with tempfile.TemporaryDirectory() as d:
            cfg = Path(d) / "c.txt"
            build = (inp / "build.txt").read_text()
            cfg.write_text("options duration=450 swap_delay=12;\ntarget lvl=100 resist=0.1 radius=2 pos=0,2.4;\n" + build)
            p = subprocess.run([a.probe, str(cfg)], capture_output=True, text=True)
            row["probe"] = json.loads(p.stdout) if p.returncode == 0 else {"error": p.stderr[-300:]}
        out.append(row)
        print(case, {k: round(v["measured"].get("mean", -1)) for k, v in row["programs"].items()}, flush=True)
    Path(a.out).write_text(json.dumps(out, indent=1))


if __name__ == "__main__":
    main()
