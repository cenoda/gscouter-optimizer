#!/usr/bin/env python3
"""Tables for docs/cycle-synthesis-pilot.md from the JSON files here."""
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent


def load(n):
    p = HERE / n
    return {r["case"]: r for r in json.loads(p.read_text())} if p.exists() else {}


def main():
    lsa, sk, tf, tp, co, ss = (load("lsa.json"), load("skeleton.json"), load("transforms.json"),
                               load("transplant.json"), load("corpus_overlap.json"), {})
    p = HERE / "search_start.json"
    if p.exists():
        for r in json.loads(p.read_text()):
            ss.setdefault(r["case"], {})[r["start"]] = r
    out = ["# Research tables (development cases; 32 audit fights unless stated)", "",
           "Values are DPS / A on the same 32 fights (the selected v2.1 V2S@384 start's audit seed for v2 cases).", "",
           "| Case | group | A/L | L blocked (s of 300) | S V2S@384 | S V2T@384 | L guard-bursts | skeleton K | transplant T | sim(L,A) | best other main rotation sim to A |",
           "|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|"]
    for c, r in lsa.items():
        pr = r["programs"]
        A = pr["A"]["measured"]["mean"]
        L = pr["L"]["measured"]
        worst = max(((w, t, r["members"][i]) for i, ws in enumerate(L["waits"]) for w, t in ws.items()),
                    key=lambda x: x[1], default=("-", 0, "-"))
        blocked = f"{worst[1]:.0f} ({worst[0]}, slot {r['members'].index(worst[2]) + 1})" if worst[1] >= 1 else "<1"
        f = lambda k: f"{pr[k]['measured']['mean'] / A:.2f}" if k in pr and "mean" in pr[k]["measured"] else "–"
        g = tf.get(c, {}).get("programs", {}).get("L_guard_bursts", {}).get("measured", {}).get("mean")
        k = (sk.get(c, {}).get("audit") or {}).get("mean")
        t = (tp.get(c, {}).get("audit") or {}).get("mean")
        o = co.get(c, {})
        out.append(f"| {c.split('--')[0]} | {r['group']} | {A / L['mean']:.2f} | {blocked} | {f('S_V2S384')} | {f('S_V2T384')} | "
                   f"{g / A if g else float('nan'):.2f} | {k / A if k else float('nan'):.2f} | {t / A if t else float('nan'):.2f} | "
                   f"{o.get('sim_L_to_A', float('nan')):.2f} | {o.get('best_other_sim_to_A') or float('nan'):.2f} |")
    out += ["", "Blocked: the largest single per-member wait in L's fights (seconds of the 300 s scoring window, engine wait reason);"
            " slot = member position in the manifest. The member is named in lsa.json.", ""]
    out += ["## Hybrids and cycle-level edits (transforms.json)", "", "| Case | Program | DPS / A |", "|---|---|---:|"]
    for c, r in tf.items():
        A = r["reference_audit_32"]["A"]
        for name, v in r["programs"].items():
            if name.startswith("L_guard"):
                continue
            m = v["measured"].get("mean")
            out.append(f"| {c.split('--')[0]} | {name} | {m / A if m else float('nan'):.3f} |")
    if ss:
        out += ["", "## v2.1 search from different starts (search_start.json; one seed, 96 CPU-s)", "",
                "| Case | start | start / A | selected after 96 CPU-s / A | decisions |", "|---|---|---:|---:|---:|"]
        for c, rs in ss.items():
            for s, r in rs.items():
                out.append(f"| {c.split('--')[0]} | {s} | {r['start_over_A']:.2f} | {r['selected_over_A']:.2f} | {r['decisions']} |")
    (HERE / "tables.md").write_text("\n".join(out) + "\n")
    print("\n".join(out))


if __name__ == "__main__":
    main()
