"""Context-specific candidate reuse, without optimizing any C0 profile."""
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
BIN = ROOT / "target/release/gso"


def main():
    root = ROOT / "runs/transfer-probes"
    root.mkdir(parents=True, exist_ok=True)
    rows = []
    # One downgrade at a time; unchanged members retain the source context.
    cases = [("source", [1, 1, 1, 6]), ("flins-c0", [0, 1, 1, 6]),
             ("ineffa-c0", [1, 0, 1, 6]), ("columbina-c0", [1, 1, 0, 6]),
             ("mixed-c2", [2, 1, 1, 6]), ("mixed-c4", [1, 4, 2, 6])]
    for name, constellations in cases:
        results = {}
        for program in ["baseline", "selected"]:
            config = (ROOT / f"runs/final-confirmation/{program}.gcsl").read_text()
            config = config.replace("# manifest-sha256=", "# source-manifest-sha256=")
            for member, cons in zip(["flins", "ineffa", "columbina", "sucrose"], constellations):
                config, count = re.subn(rf"(?m)^({member} char lvl=90/90 cons=)\d+", rf"\g<1>{cons}", config)
                assert count == 1
            path = root / f"{name}-{program}.gcsl"
            path.write_text(config)
            command = [str(BIN), "evaluate", "--config", str(path), "--out", str(root / f"{name}-{program}"),
                       "--samples", "128", "--seed", "8000000000000"]
            p = subprocess.run(command, cwd=ROOT, text=True, capture_output=True)
            results[program] = json.loads(p.stdout) if p.returncode == 0 else {"status": "failed or unresolved", "error": p.stderr}
        rows.append({"context": name, "constellations": constellations,
                     "scope": "comparison only; no C0 search or player-constellation substitution",
                     "status": "empirical outcome in this build/seed block; no universal compatibility rule", "results": results})
        print(name, {k: round(v["stats"]["mean"]) if "stats" in v else v["status"] for k, v in results.items()}, flush=True)
    (root / "report.json").write_text(json.dumps(rows, indent=2) + "\n")


if __name__ == "__main__":
    main()
