#!/usr/bin/env python3
"""How close is each development case's A to some *other* main-library rotation?

Research only (development cases). For each case: main library rotations
(teams/*/rotation.gcsl, never alt/) of other parties that contain the case's
on-field member (the member with the highest DPS in A's measurement), how many
share 2 or 3 members, and the best token-multiset similarity to A and to L
(character names normalised to party-slot letters so a shared structure with
different supports still counts). Also measures, natively on the case build,
the best of those rotations after re-targeting: only rotations whose members
are all in the case party are runnable as-is; those are measured.
"""
import collections
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
V2 = ROOT / "reports/heldout-recovery-v2-2026-11-05"
TEAMS = Path("/home/user/Gscouter/teams")
sys.path.insert(0, str(HERE))
from skeleton_probe import measure, resolve  # noqa: E402

WORD = re.compile(r"[A-Za-z_][\w-]*|\d+|[^\s\w]")


def tokens(src, slot):
    src = re.sub(r"#[^\n]*", "", src)
    out = []
    for t in WORD.findall(src):
        out.append(slot.get(t, t))
    return collections.Counter(out)


def sim(a, b):
    inter = sum((a & b).values())
    union = sum((a | b).values())
    return inter / union if union else 0.0


def main():
    gso, probe = sys.argv[1], sys.argv[2]
    lsa = json.loads((HERE / "lsa.json").read_text())
    rotations = {}
    for d in sorted(TEAMS.iterdir()):
        r = d / "rotation.gcsl"
        if r.exists():
            rotations[d.name] = r.read_text()
    out = []
    for row in lsa:
        case, members = row["case"], row["members"]
        party, alt = case.split("--")
        A = (TEAMS / party / "alt" / alt / "rotation.gcsl").read_text()
        inp = V2 / "inputs" / case if (V2 / "inputs" / case).exists() else V2 / "development/inputs" / case
        L = (inp / "L.gcsl").read_text()
        am = row["programs"]["A"]["measured"]["chars"]
        dps = members[max(range(4), key=lambda i: am[i])]
        words = set()
        for txt in [A, L] + list(rotations.values()):
            words |= set(re.findall(r"(?m)^\s*([a-z]+)\s+(?:skill|burst|attack|charge|dash|jump|swap|aim|walk)", txt))
        alias = resolve(probe, list(words))
        def slots(team_members):
            order = [dps] + [m for m in team_members if m != dps]
            letters = {m: f"<{chr(65 + i)}>" for i, m in enumerate(order)}
            return {w: letters[k] for w, k in alias.items() if k in letters}
        sA = tokens(A, slots(members))
        sL = tokens(L, slots(members))
        cands = []
        for team, txt in rotations.items():
            if team == party:
                continue
            tm = team.split("-")
            tm_keys = {alias.get(m, m) for m in tm}
            if dps not in tm_keys:
                continue
            shared = len(tm_keys & set(members))
            st = tokens(txt, slots(list(tm_keys)))
            cands.append({"team": team, "shared": shared, "sim_A": sim(st, sA), "sim_L": sim(st, sL)})
        cands.sort(key=lambda c: -c["sim_A"])
        runnable = [c for c in cands if c["shared"] == 4]
        best_run = None
        for c in runnable[:3]:
            m = measure(gso, inp / "manifest.json", rotations[c["team"]], row["measure_seed"], 32)
            c["measured_mean"] = m.get("mean")
            c["measured_error"] = m.get("error")
        refA = row["programs"]["A"]["measured"]["mean"]
        rec = {"case": case, "on_field": dps, "rotations_with_member": len(cands),
               "sharing_3plus": sum(c["shared"] >= 3 for c in cands),
               "sim_L_to_A": sim(sL, sA), "best_other_sim_to_A": cands[0]["sim_A"] if cands else None,
               "top": cands[:3], "same_party_other_rotations": runnable[:3]}
        out.append(rec)
        print(f"{case[:42]:42s} {dps:12s} n={len(cands):3d} 3+={rec['sharing_3plus']:3d} simLA={rec['sim_L_to_A']:.2f} "
              f"bestOtherA={rec['best_other_sim_to_A'] or 0:.2f} ({cands[0]['team'][:30] if cands else '-'}) "
              + " ".join(f"{(c.get('measured_mean') or 0) / refA:.2f}A" for c in runnable[:3]), flush=True)
    (HERE / "corpus_overlap.json").write_text(json.dumps(out, indent=1))


if __name__ == "__main__":
    main()
