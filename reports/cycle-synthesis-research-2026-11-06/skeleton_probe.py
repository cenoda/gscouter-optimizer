#!/usr/bin/env python3
"""Feasibility probe: does a generic, metadata-derived cycle skeleton reach
A-like DPS on the development cases where v2.1 local search stayed low?

Research only, development cases only (their A was already read). This is
not a search: it enumerates one small fixed grammar, picks the best program
on training fights and measures that one program on the same audit fights as
L, S and A in lsa.json. The grammar and every rule below were fixed before
the first run and are identical for every case; no character name appears.

Grammar (one cycle; `gso measure` wraps it in the usual `while 1`):

    for let c = 0; c < 1; c = c + 1 {        # scope for the window start
      let t0 = f();
      <support segment>                      # L's statements of the other
                                             # members, in L order, with every
                                             # burst behind a ready guard
      if .D.burst.ready { D burst; }
      [skill policy "once": if .D.skill.ready { D skill; }]
      while f() - t0 < P {
        [skill policy "ready": if .D.skill.ready { D skill; } else] { D <unit>; }
      }
    }

  D     in {L's active member, the member with the highest DPS in L's measurement}
  P     in the members' measured skill and burst cycle times (cooldown plus the
        action's own frames, mechprobe), kept if 300..1800 frames, merged when
        within 30 frames of each other
  unit  in {"attack"} plus D's three most frequent main-library action
        statements that contain no skill or burst (v2.1 dictionary after the
        case's own k-rare leakage drop list)
  skill policy in {"ready", "once"}

L's top-level function and variable definitions are kept so its support
statements still parse. Every program is measured on 16 training fights
(seed 5000000000000); the best training mean is then measured on the
32 audit fights used in lsa.json.
"""
import argparse
import json
import re
import subprocess
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
V2 = ROOT / "reports/heldout-recovery-v2-2026-11-05"
TRAIN_SEED = 5000000000000


def lex_statements(src):
    """Top-level split of a GCSL rotation into (kind, text) items.

    kind: 'def' for fn/let definitions before the first other statement,
    'stmt' otherwise. Blocks are kept whole."""
    src = re.sub(r"#[^\n]*", "", src)
    items, depth, cur = [], 0, ""
    for ch in src:
        cur += ch
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0 and not re.match(r"\s*(let|fn)\b", cur) and not re.search(r"\belse\s*$", cur):
                items.append(cur.strip())
                cur = ""
        elif ch == ";" and depth == 0:
            items.append(cur.strip())
            cur = ""
    # a trailing `} else {...}` was split; rejoin `if` chains
    out = []
    for it in items:
        if out and re.match(r"else\b", it):
            out[-1] += " " + it
        elif it:
            out.append(it)
    return out


def flatten_actions(src):
    """All `<word> <action list>;` statements anywhere in src, in order."""
    src = re.sub(r"#[^\n]*", "", src)
    return [(m.group(1), m.group(2).strip())
            for m in re.finditer(r"(?:(?<=[;{}])|^)\s*([a-z][a-z0-9_]*)\s+((?:[a-z_]+(?:\[[^\]]*\])?(?:\s*:\s*\d+)?\s*,\s*)*[a-z_]+(?:\[[^\]]*\])?(?:\s*:\s*\d+)?)\s*;", src, re.M)]


def resolve(probe, words):
    p = subprocess.run([probe, "--alias", *sorted(set(words))], capture_output=True, text=True, check=True)
    return dict(line.split() for line in p.stdout.splitlines())


def guard_bursts(owner, actions):
    """Split an action list so each burst sits behind a ready guard."""
    parts = [a.strip() for a in actions.split(",")]
    out, run = [], []
    for a in parts:
        if re.match(r"burst\b", a):
            if run:
                out.append(f"{owner} {', '.join(run)};")
                run = []
            out.append(f"if .{owner}.burst.ready {{ {owner} burst; }}")
        else:
            run.append(a)
    if run:
        out.append(f"{owner} {', '.join(run)};")
    return out


def split_fns(src):
    """(function definitions, source without them)."""
    src = re.sub(r"#[^\n]*", "", src)
    fns, rest, last = [], [], 0
    for m in re.finditer(r"\bfn\s+\w+\s*\([^)]*\)\s*\{", src):
        if m.start() < last:
            continue
        depth, j = 1, m.end()
        while depth and j < len(src):
            depth += {"{": 1, "}": -1}.get(src[j], 0)
            j += 1
        fns.append(src[m.start():j])
        rest.append(src[last:m.start()])
        last = j
    rest.append(src[last:])
    return fns, "".join(rest)


def defs_of(src):
    """Top-level fn definitions and top-level let statements, as text."""
    fns, rest = split_fns(src)
    lets, depth, i = [], 0, 0
    for m in re.finditer(r"[{}]|\blet\s+\w+\s*=\s*[^;]+;", rest):
        t = m.group(0)
        if t == "{":
            depth += 1
        elif t == "}":
            depth -= 1
        elif depth == 0 and not re.search(r"\bfor\s*$", rest[:m.start()]):
            lets.append(t)
    return fns + lets


def measure(gso, manifest, text, seed, n):
    with tempfile.TemporaryDirectory() as d:
        r = Path(d) / "r.gcsl"
        r.write_text(text)
        p = subprocess.run([gso, "measure", "--manifest", str(manifest), "--rotation", str(r), "--seed", str(seed),
                            "--samples", str(n), "--out", str(Path(d) / "e")], capture_output=True, text=True)
        if p.returncode:
            return {"error": p.stderr.strip()[-200:]}
        j = json.loads(p.stdout)
        return {"mean": j["stats"]["mean"], "se": j["stats"]["se"], "chars": j["stats"]["chars"],
                "waits": j["stats"]["waits"], "cpu": j["cpu_seconds"]}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--gso", required=True)
    ap.add_argument("--probe", required=True)
    ap.add_argument("--lsa", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    dictionary = json.loads((V2 / "v2.1/inputs/dictionary-v2.json").read_text())
    rows = []
    for case in json.loads(Path(a.lsa).read_text()):
        name = case["case"]
        v2case = (V2 / "inputs" / name).exists()
        inp = V2 / "inputs" / name if v2case else V2 / "development/inputs" / name
        manifest = inp / "manifest.json"
        drops = set(json.loads((inp / "drops-v2.json").read_text())["dropped_for_leakage"])
        members = case["members"]
        L = (inp / "L.gcsl").read_text()
        stmts = flatten_actions(split_fns(L)[1])
        alias = resolve(a.probe, [w for w, _ in stmts] + re.findall(r"active\s+(\w+)", L))
        active = alias[re.search(r"active\s+(\w+)", L).group(1)]
        lm = case["programs"]["L"]["measured"]
        top = members[max(range(4), key=lambda i: lm["chars"][i])]
        probe = case["probe"]
        periods = []
        for r in probe:
            if r["ok"] and r["action"] in ("skill", "burst"):
                cd = r["skill_cd"] if r["action"] == "skill" else r["burst_cd"]
                if cd:
                    periods.append(cd + r["frames"])
        periods = sorted(p for p in periods if 300 <= p <= 1800)
        merged = []
        for p in periods:
            if not merged or p - merged[-1] > 30:
                merged.append(p)
        defs = defs_of(L)
        programs = []
        for D in dict.fromkeys([active, top]):
            units = ["attack"]
            for b in dictionary["blocks"].get(D, []):
                t = b["text"]
                if b["kind"] != "action statement" or t in drops or re.search(r"\b(skill|burst)\b", t):
                    continue
                u = t[len(D):].strip().rstrip(";").strip()
                if u and u not in units:
                    units.append(u)
                if len(units) == 4:
                    break
            support = []
            for w, acts in stmts:
                o = alias.get(w)
                if o and o != D and o in members:
                    support += guard_bursts(o, acts)
            for P in merged:
                for unit in units:
                    for policy in ("ready", "once"):
                        inner = (f"if .{D}.skill.ready {{ {D} skill; }} else {{ {D} {unit}; }}" if policy == "ready"
                                 else f"{D} {unit};")
                        head = f"if .{D}.skill.ready {{ {D} skill; }}" if policy == "once" else ""
                        body = "\n  ".join(support)
                        text = (f"active {D};\n" + "\n".join(defs) + "\n"
                                f"for let c = 0; c < 1; c = c + 1 {{\n  let t0 = f();\n  {body}\n"
                                f"  if .{D}.burst.ready {{ {D} burst; }}\n  {head}\n"
                                f"  while f() - t0 < {P} {{ {inner} }}\n}}\n")
                        programs.append({"D": D, "P": P, "unit": unit, "policy": policy, "text": text})
        t0 = time.time()
        cpu = 0.0
        for p in programs:
            p["train"] = measure(a.gso, manifest, p["text"], TRAIN_SEED, 16)
            cpu += p["train"].get("cpu", 0.0)
        ok = [p for p in programs if "mean" in p["train"]]
        best = max(ok, key=lambda p: p["train"]["mean"]) if ok else None
        audit = measure(a.gso, manifest, best["text"], case["measure_seed"], 32) if best else None
        progs = case["programs"]
        ref = {k: v["measured"].get("mean") for k, v in progs.items()}
        row = {"case": name, "group": case["group"], "periods": merged, "n_programs": len(programs),
               "n_valid": len(ok), "train_cpu": cpu, "wall": time.time() - t0,
               "best": {k: best[k] for k in ("D", "P", "unit", "policy", "text")} if best else None,
               "best_train": best["train"]["mean"] if best else None,
               "audit": audit, "reference_audit_32": ref,
               "errors": sorted({p["train"]["error"][:120] for p in programs if "error" in p["train"]})[:5]}
        rows.append(row)
        A = ref["A"]
        print(f"{name[:44]:44s} valid {len(ok)}/{len(programs)} best K={audit['mean'] / A if audit and 'mean' in audit else float('nan'):.3f}A "
              f"L={ref['L'] / A:.3f}A S={ref.get('S_V2S384', float('nan')) / A if ref.get('S_V2S384') else float('nan'):.3f}A "
              f"best D={best['D'] if best else '-'} P={best['P'] if best else '-'} unit={best['unit'] if best else '-'} {best['policy'] if best else ''} cpu={cpu:.0f}",
              flush=True)
    Path(a.out).write_text(json.dumps(rows, indent=1))


if __name__ == "__main__":
    main()
