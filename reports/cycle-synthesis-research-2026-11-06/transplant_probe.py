#!/usr/bin/env python3
"""Whole-cycle transplant from other parties' main-library rotations.

Research only, development cases. No A is used for donor choice: the
on-field member D is the member with the highest DPS in L's measurement;
donors are main-library rotations (teams/*/rotation.gcsl, other parties,
never alt/) containing D, ranked by the number of members shared with the
case party (desc), then by team name. The top DONORS donors are retargeted:
shared members keep their names; every assignment of the case's remaining
members to the donor's remaining slots is tried (at most 3! = 6). Character
words (and their aliases) are replaced as whole words; hyphenated status
names are left untouched. Each program is measured on 16 training fights;
the best training program is measured on the 32 audit fights of lsa.json.
The best audit value over all tried programs is also reported as an
(optimistic, selection-free) upper bound of this donor set.
"""
import itertools
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
V2 = ROOT / "reports/heldout-recovery-v2-2026-11-05"
TEAMS = Path("/home/user/Gscouter/teams")
sys.path.insert(0, str(HERE))
from skeleton_probe import measure, resolve, TRAIN_SEED  # noqa: E402

DONORS = 8
IDENT = re.compile(r"(?<![\w.-])(\.?)([a-z][a-z0-9_]*)(?![\w-])")


def retarget(src, alias, mapping):
    def repl(m):
        dot, w = m.groups()
        k = alias.get(w)
        if k in mapping:
            return dot + mapping[k]
        return m.group(0)
    return IDENT.sub(repl, src)


def main():
    gso, probe = sys.argv[1], sys.argv[2]
    lsa = json.loads((HERE / "lsa.json").read_text())
    rotations = {d.name: (d / "rotation.gcsl").read_text() for d in sorted(TEAMS.iterdir()) if (d / "rotation.gcsl").exists()}
    words = set()
    for txt in rotations.values():
        words |= set(re.findall(r"[a-z][a-z0-9_]*", re.sub(r"#[^\n]*", "", txt)))
    alias = {w: k for w, k in resolve(probe, list(words)).items() if k != "-"}
    out = []
    for row in lsa:
        case, members = row["case"], row["members"]
        party = case.split("--")[0]
        inp = V2 / "inputs" / case if (V2 / "inputs" / case).exists() else V2 / "development/inputs" / case
        lm = row["programs"]["L"]["measured"]["chars"]
        D = members[max(range(4), key=lambda i: lm[i])]
        donors = []
        for team, txt in rotations.items():
            if team == party:
                continue
            tk = [alias.get(m, m) for m in team.split("-")]
            if D in tk and len(tk) == 4:
                donors.append((-len(set(tk) & set(members)), team, tk))
        donors.sort()
        tried = []
        for neg, team, tk in donors[:DONORS]:
            shared = [m for m in tk if m in members]
            free_slots = [m for m in tk if m not in members]
            free_members = [m for m in members if m not in tk]
            for perm in itertools.permutations(free_members, len(free_slots)):
                mapping = {m: m for m in shared}
                mapping.update(dict(zip(free_slots, perm)))
                text = retarget(rotations[team], alias, mapping)
                tr = measure(gso, inp / "manifest.json", text, TRAIN_SEED, 16)
                tried.append({"donor": team, "shared": -neg, "mapping": mapping, "text": text, "train": tr})
        ok = [t for t in tried if "mean" in t["train"]]
        best = max(ok, key=lambda t: t["train"]["mean"]) if ok else None
        audit = measure(gso, inp / "manifest.json", best["text"], row["measure_seed"], 32) if best else {}
        # selection-free upper bound over this donor set (audits every valid program)
        top_audit = 0.0
        for t in sorted(ok, key=lambda t: -t["train"]["mean"])[:6]:
            m = measure(gso, inp / "manifest.json", t["text"], row["measure_seed"], 32)
            top_audit = max(top_audit, m.get("mean", 0.0))
        ref = {k: v["measured"]["mean"] for k, v in row["programs"].items()}
        rec = {"case": case, "D": D, "donors": [d[1] for d in donors[:DONORS]], "n_programs": len(tried),
               "n_valid": len(ok), "best": {k: best[k] for k in ("donor", "shared", "mapping", "text")} if best else None,
               "audit": audit, "top6_audit_max": top_audit, "reference_audit_32": ref}
        out.append(rec)
        A = ref["A"]
        s = ref.get("S_V2S384")
        print(f"{case[:42]:42s} D={D:11s} valid {len(ok):2d}/{len(tried):2d} T={audit.get('mean', 0) / A:.3f}A "
              f"top6max={top_audit / A:.3f}A L={ref['L'] / A:.3f}A S={s / A if s else float('nan'):.3f}A "
              f"donor={best['donor'] if best else '-'}", flush=True)
    (HERE / "transplant.json").write_text(json.dumps(out, indent=1))


if __name__ == "__main__":
    main()
