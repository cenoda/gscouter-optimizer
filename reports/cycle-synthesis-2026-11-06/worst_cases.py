"""List informative cases sorted by the best arm's S/A at 384 CPU-s."""
import json, os
d = json.load(open(os.path.join(os.path.dirname(__file__), "experiments.json")))["cases"]

def sel(c, a):
    ss = c["arms"][a]["384"]["starts"]
    s = max(ss.values(), key=lambda s: s.get("selection") or -1)
    S = s["L"] if s.get("audit_failure") else s["S"]
    return S / s["A"], s

rows = []
for c in d:
    if c["class"] != "informative":
        continue
    v = {a: sel(c, a) for a in ["V21S", "SYN", "XO"]}
    s = v["V21S"][1]
    rows.append((max(x[0] for x in v.values()), c["case"].split("--")[0], c["A_over_L"], s["L"] / s["A"], {a: x[0] for a, x in v.items()}))
for best, case, al, la, v in sorted(rows)[:10]:
    print(f"{case} A/L={al:.2f} L={la:.0%} " + " ".join(f"{a}={x:.1%}" for a, x in v.items()) + f" best={best:.1%}")
