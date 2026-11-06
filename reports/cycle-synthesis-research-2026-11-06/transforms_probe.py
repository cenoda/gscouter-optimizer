#!/usr/bin/env python3
"""Development-case diagnostics: generic whole-program transforms of L and a
few hand-built hybrids of A and the generic skeleton (research only).

1. Generic transforms, applied identically to every case's L:
   - guard_bursts: every `burst` in an action list is split out behind
     `if .X.burst.ready { X burst; }` (a burst that cannot be used is skipped
     instead of blocking the cycle);
   - guard_all: the same for `skill` and `burst`.
2. Decomposition hybrids (hand-built, development cases only, A was read):
   which part of A carries its advantage, the support segment or the
   on-field window? Each hybrid is named in HYBRIDS with its construction.

All programs are measured on the same 32 audit fights as lsa.json.
"""
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
V2 = ROOT / "reports/heldout-recovery-v2-2026-11-05"
sys.path.insert(0, str(HERE))
from skeleton_probe import measure  # noqa: E402

STMT = re.compile(r"(?m)(^\s*|[;{}]\s*)([a-z][a-z0-9_]*)(\s+)((?:[a-z_]+(?:\[[^\]]*\])?(?:\s*:\s*\d+)?\s*,\s*)*"
                  r"[a-z_]+(?:\[[^\]]*\])?(?:\s*:\s*\d+)?)(\s*;)")
KEYWORDS = {"let", "fn", "if", "while", "for", "return", "active", "else", "break", "continue"}


def guard(src, actions):
    def repl(m):
        pre, owner, _, acts, _ = m.groups()
        if owner in KEYWORDS:
            return m.group(0)
        parts = [a.strip() for a in re.split(r",", acts)]
        out, run = [], []
        for a in parts:
            name = re.match(r"[a-z_]+", a).group(0)
            if name in actions and not re.search(r":\s*\d+", a):
                if run:
                    out.append(f"{owner} {', '.join(run)};")
                    run = []
                out.append(f"if .{owner}.{name}.ready {{ {owner} {a}; }}")
            else:
                run.append(a)
        if run:
            out.append(f"{owner} {', '.join(run)};")
        return pre + " ".join(out)
    return STMT.sub(repl, src)


CYNO_A_SUPPORT = """  xilonen skill, attack:2;
  furina skill, dash, attack;
  nahida skill, burst, attack, charge;
  furina burst, attack:2;
  xilonen skill, attack:2;"""
CYNO_SKEL_SUPPORT = """  furina skill;
  xilonen skill;
  if .xilonen.burst.ready { xilonen burst; }
  xilonen attack:2;
  furina attack;
  if .furina.burst.ready { furina burst; }
  nahida skill, attack;
  if .nahida.burst.ready { nahida burst; }"""
CYNO_A_WINDOW = """  cyno skill, burst;
  while f() - Startframe < {P} {{
    if .cyno.mods.cyno-a1 && .cyno.skill.ready {{
      cyno skill;
    }} else if .cyno.normal == 5 && .stam > 20 {{
      cyno dash;
    }} else {{
      cyno attack;
    }}
  }}"""
CYNO_SKEL_WINDOW = """  if .cyno.burst.ready {{ cyno burst; }}
  while f() - Startframe < {P} {{ if .cyno.skill.ready {{ cyno skill; }} else {{ cyno attack; }} }}"""
CYNO_GENERIC_WINDOW = """  if .cyno.burst.ready {{ cyno burst; }}
  while f() - Startframe < {P} {{
    if .cyno.skill.ready {{
      cyno skill;
    }} else if .cyno.normal == 5 && .stam > 20 {{
      cyno dash;
    }} else {{
      cyno attack;
    }}
  }}"""


def cyno(support, window, P):
    return ("active cyno;\ncyno skill;\nfor let i=0; i<4; i=i+1 {\n  let Startframe = f();\n" + support + "\n"
            + window.format(P=P) + "\n}\n")


MAV_A_SUPPORT = """  mavuika skill, attack;
  ororon burst, skill;
  chev skill[hold=1];
  bennett attack:2, skill, burst;
  mavuika burst;"""
MAV_SKEL_SUPPORT = """  ororon skill;
  if .ororon.burst.ready { ororon burst; }
  chevreuse skill[hold=1];
  if .chevreuse.burst.ready { chevreuse burst; }
  bennett skill;
  if .bennett.burst.ready { bennett burst; }
  if .mavuika.burst.ready { mavuika burst; }"""
MAV_A_WINDOW = "  mavuika " + ", ".join(["charge:2, dash"] * 7) + ";"
MAV_WINDOW = "  while f() - t0 < {P} {{ if .mavuika.skill.ready {{ mavuika skill; }} else {{ mavuika charge:2, dash; }} }}"
MAV_WINDOW_NOSKILL = "  while f() - t0 < {P} {{ mavuika charge:2, dash; }}"


def mav(support, window, P):
    return ("active mavuika;\nfor let i=0; i<4; i=i+1 {\n  let t0 = f();\n" + support + "\n"
            + window.format(P=P) + "\n}\n")


HYBRIDS = {
    "cyno-furina-nahida-xilonen--jBkf669CgWCr": {
        "A_support+A_window P=1260 (=A)": cyno(CYNO_A_SUPPORT, CYNO_A_WINDOW, 1260),
        "A_support+A_window P=1200 (probe period)": cyno(CYNO_A_SUPPORT, CYNO_A_WINDOW, 1200),
        "A_support+generic_window(no status guard) P=1260": cyno(CYNO_A_SUPPORT, CYNO_GENERIC_WINDOW, 1260),
        "A_support+skeleton_window P=1200": cyno(CYNO_A_SUPPORT, CYNO_SKEL_WINDOW, 1200),
        "skeleton_support+A_window P=1260": cyno(CYNO_SKEL_SUPPORT, CYNO_A_WINDOW, 1260),
        "skeleton_support+skeleton_window P=1200": cyno(CYNO_SKEL_SUPPORT, CYNO_SKEL_WINDOW, 1200),
    },
    "mavuika-chevreuse-ororon-bennett--Mw9HdnKTkBfJ": {
        "A_support+A_window (=A)": mav(MAV_A_SUPPORT, MAV_A_WINDOW, 0),
        "A_support+timebox(skill-ready) P=1080": mav(MAV_A_SUPPORT, MAV_WINDOW, 1080),
        "A_support+timebox(no skill) P=1080": mav(MAV_A_SUPPORT, MAV_WINDOW_NOSKILL, 1080),
        "skeleton_support+A_window": mav(MAV_SKEL_SUPPORT, MAV_A_WINDOW, 0),
        "skeleton_support+timebox(no skill) P=1080": mav(MAV_SKEL_SUPPORT, MAV_WINDOW_NOSKILL, 1080),
    },
}


ARLE = "arlecchino-bennett-citlali-xilonen--DnLT9CbL979h"
NEUV = "neuvillette-furina-citlali-xilonen--gLgzMjmTN8z6"


def second_round(case, L, A):
    """Cycle-level parameters only: burst every Nth cycle, trailing period."""
    out = {}
    if case == ARLE:
        body_end = L.rstrip().rfind("}")
        out["L + `if i == 3 { arlecchino burst; }` (burst every 4th cycle)"] = (
            L[:body_end] + "  if i == 3 { arlecchino burst; }\n" + L[body_end:])
        out["A without its arlecchino burst"] = A.replace("    arle burst;\n", "")
        out["A without WaitForCryo calls"] = re.sub(r"\n\s*WaitForCryo\(\);", "", A)
    if case == NEUV:
        g = guard(L, {"burst"})
        for w in (173, 400, 715, 1000):
            out[f"L_guard_bursts with trailing wait({w})"] = g.replace("wait(173);", f"wait({w});")
        out["A with trailing wait(173)"] = A.replace("wait(715);", "wait(173);")
    return out


def main():
    gso = sys.argv[1]
    lsa = {r["case"]: r for r in json.loads((HERE / "lsa.json").read_text())}
    out = []
    for case, row in lsa.items():
        inp = V2 / "inputs" / case if (V2 / "inputs" / case).exists() else V2 / "development/inputs" / case
        manifest = inp / "manifest.json"
        L = (inp / "L.gcsl").read_text()
        progs = {"L_guard_bursts": guard(L, {"burst"}), "L_guard_all": guard(L, {"skill", "burst"})}
        progs.update(HYBRIDS.get(case, {}))
        party, alt = case.split("--")
        A = (Path("/home/user/Gscouter/teams") / party / "alt" / alt / "rotation.gcsl").read_text()
        progs.update(second_round(case, L, A))
        ref = {k: v["measured"]["mean"] for k, v in row["programs"].items()}
        res = {}
        for k, text in progs.items():
            m = measure(gso, manifest, text, row["measure_seed"], 32)
            res[k] = {"text": text, "measured": m}
            v = m.get("mean")
            print(f"{case[:40]:40s} {k:52s} {v / ref['A'] if v else float('nan'):.3f}A "
                  f"(L {ref['L'] / ref['A']:.3f}A) {m.get('error', '')[:80]}", flush=True)
        out.append({"case": case, "reference_audit_32": ref, "programs": res})
    (HERE / "transforms.json").write_text(json.dumps(out, indent=1))


if __name__ == "__main__":
    main()
