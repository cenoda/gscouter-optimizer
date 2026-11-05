"""Snapshot public files; never import Gscouter's Go internal packages.

Only the C0 baseline is read. Alternative complete cycles remain withheld.
"""
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT.parent / "gscouter"
OUT = ROOT / "data/flins-c1"
OUT.mkdir(parents=True, exist_ok=True)
TEAM = "flins-ineffa-columbina-sucrose"
sources = {}
for src, dst in [(f"teams/{TEAM}/rotation.gcsl", "baseline.gcsl"),
                 (f"teams/{TEAM}/build.txt", "library-build.txt")]:
    raw = (SITE / src).read_bytes()
    (OUT / dst).write_bytes(raw)
    sources[src] = hashlib.sha256(raw).hexdigest()
raw = (SITE / "internal/artifact/table.json").read_bytes()
table = json.loads(raw)
weights = table["teams"][TEAM]
(OUT / "weights.json").write_text(json.dumps({"format": table["format"], "gcsim": table["gcsim"], **weights}, indent=2) + "\n")
sources["internal/artifact/table.json"] = hashlib.sha256(raw).hexdigest()
raw = (SITE / "internal/artifact/artifact.go").read_bytes()
sources["internal/artifact/artifact.go"] = hashlib.sha256(raw).hexdigest()
rolls = dict((k, float(v)) for k, v in re.findall(r'"([a-z%]+)":\s+([\d.]+),', raw.decode().split("var MaxRolls = ")[1].split("}")[0]))
assert len(rolls) == 10
members = []
for ci, (key, weapon, refinement, aset, mains, useful) in enumerate([
    ("flins", "bloodsoakedruins", 1, "nightoftheskysunveiling", ["atk%", "atk%", "cd"], ["atk%", "cr", "cd", "em"]),
    ("ineffa", "fracturedhalo", 1, "aubadeofmorningstarandmoon", ["atk%", "atk%", "cd"], ["atk%", "cr", "cd", "em"]),
    ("columbina", "nocturnescurtaincall", 1, "silkenmoonsserenade", ["hp%", "hp%", "cd"], ["hp%", "cr", "cd", "em"]),
    ("sucrose", "wanderingevenstar", 5, "viridescentvenerer", ["em", "em", "em"], ["em", "cr", "cd", "atk%"]),
]):
    pieces = []
    for slot, main in zip(["flower", "plume", "sands", "goblet", "circlet"], ["hp", "atk"] + mains):
        dmg = "hp%" if ci == 2 else "atk%"
        if ci == 3:
            subs = ["em"] * 6 + ["er", "cr", "cd"] if main != "em" else ["er"] * 6 + ["cr", "cd", "atk%"]
        elif main == "cd":
            subs = ["cr"] * (3 if ci == 0 else 5) + [dmg] * (3 if ci == 0 else 2) + ["em"] * (2 if ci == 0 else 1) + ["er"]
        else:
            subs = ["cr"] * (2 if ci == 0 else 3) + ["cd"] * (3 if ci == 0 else 2) + ["em" if main == dmg else dmg] * 3 + ["er"]
        pieces.append({"slot": slot, "rarity": 5, "level": 20, "main": main,
            "value": {"hp": 4780, "atk": 311, "em": 187, "cd": .622, "atk%": .466, "hp%": .466}[main],
            "cv_ceiling": 38.9, "rolls": [{"stat": s, "quality": 3} for s in subs]})
    members.append({"key": key, "cons": 6 if ci == 3 else 1, "weapon": weapon, "refinement": refinement,
        "set": aset, "useful": useful, "weights": next(c["weights"] for c in weights["characters"] if c["key"] == key), "pieces": pieces})
manifest = {"format": 1, "engine": "1ba0af119e0477d6f44ee966a78fa06d84d2c5de",
    "site_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=SITE, text=True).strip(),
    "sources": sources, "max_rolls": rolls, "members": members}
(OUT / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
print("Exported baseline, weights and physically attainable frozen artifact envelope.")
