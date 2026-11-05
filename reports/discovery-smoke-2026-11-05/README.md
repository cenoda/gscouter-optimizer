# Baseline-free discovery execution checks, 2026-11-05

The new `--from-scratch` path generates and searches rotations from a supplied
four-member build without reading an existing cycle. Two non-Flins parties,
three starts each, completed bounded discovery, independent audits and journal
checks. A saved unfinished generated proposal resumed successfully.

**This is execution evidence, not a held-out discovery-quality benchmark.**
Builds are synthetic fixed-stat fixtures, not physical standard artifacts or
actual-player gear. No known strong reference was scored, so these results do
not establish competitive play, general effectiveness or an optimality gap.

## Inputs and protocol

- Electro fixture: Sucrose / Fischl / Xingqiu / Beidou, all C6.
- Geo fixture: Noelle / Gorou / Yun Jin / Fischl, all C6.
- Fixed format-2 builds, no artifact-roll search, same pinned engine as the
  Flins pilot. The script defines all weapons, sets and synthetic stats.
- Greedy, from-scratch mode, seeds 11/37/91, cumulative discovery caps 4/12 CPU
  seconds. Each cap has a separate independent 32-fight audit. Every fight is
  450 seconds with a 150-second warmup.
- Initial families are generated from the build, assigning one field
  opportunity to each member. No adjacent `baseline.gcsl` exists. Every logged
  proposal and saved chain uses the generated representation.
- Candidate selection is frozen before audits. The pre-polish peak is protected
  within the same six-candidate shortlist, alongside the final selected peak
  and generated starting control.

The frozen final implementation fingerprint is
`79123f73118bf79daf5e723273ce316b691c02981f9cad58692311bf5fb76da4`.
The script verifies that this equals the fingerprint reconstructed from the
actual working source, rather than relying on the HEAD commit.

## Observed outcomes

These are independent audited means at the 12-second cumulative cap. A
32-fight mean difference is not by itself a significance claim. The two stages
use different amounts of CPU and this is not a causal ablation of polishing.

| Fixture | Seed | Selected before polish | Selected after polish |
|---|---:|---:|---:|
| Electro | 11 | 29,085 | 29,104 |
| Electro | 37 | 29,590 | 30,875 |
| Electro | 91 | 26,179 | 26,609 |
| Geo | 11 | 12,939 | 13,834 |
| Geo | 37 | 15,716 | 20,650 |
| Geo | 91 | 14,495 | 15,503 |

The `baseline` fields in these reports mean **first generated starting program**,
not a library reference. All six final search-selected programs score above
that generated starting control in these samples. This does not demonstrate
an advantage over a good existing rotation. All budget points, means, standard
errors, paired differences, operator counts and checks are in
[summary.json](summary.json). Selected GCSL files are retained alongside it.

## Correctness and recovery

- All 13 Rust tests, strict Clippy across targets, formatting and Python syntax
  checks pass. Tests cover default-action engine support, structural reachability,
  full-fight progress, baseline-independent initialization, generated pending
  proposal/RNG and polish-state round trips, and protected shortlist membership.
- The six final runs verify **835 completed common decision blocks**, including
  **72 explicit restarts**. Restarts deliberately allow a lower-scoring new
  structure after a completed common block; they are not labelled improvements.
- Independent journal reconstruction verifies **49 peak changes** through
  **62 completed common 32-fight promotion blocks**. All final reconstructed
  peaks match the saved selections. No failed fight records occur in these six
  runs. Exact no-ops remain visible without simulation.
- A fresh recovery run saves an unfinished `generated insert visit` proposal.
  Its copied checkpoint resumes **15 to 30 decisions**, preserves every prior
  event, completes that pending operator and advances the saved RNG. Journal
  and peak checks pass. One incomplete promotion remains unresolved and does
  not replace the peak. This verifies saved-checkpoint continuation, not
  arbitrary hard-kill/truncated-journal recovery.
- A two-second legacy Flins control still executes and passes common-block and
  peak checks. Incompatible baseline/local-only/restricted flags are rejected
  in from-scratch mode.

A preliminary six-run execution check also passed. Review then identified a
shortlist edge case: overwriting a fixed archive index for the pre-polish
candidate could evict a peak stored at that index. The final implementation
reserves the generated control, peak and pre-polish candidate before filling
diversity slots. A regression assertion covers the full-archive case. The full
six-run suite was repeated with the final source; preliminary evidence and
costs are retained in [preliminary.zip](preliminary.zip).

## Artifacts and costs

[source.zip](source.zip) contains the actual working source and frozen inputs,
including pre-existing uncommitted other-party support.
[runs.zip](runs.zip) contains all six final runs, both budget reports, inputs,
initialization records, checkpoints, evaluation/proposal journals and the
separate pending-recovery runs. [legacy-control.zip](legacy-control.zip)
contains the legacy smoke evidence. Archive SHA-256 values and CRC checks are
in [archive.json](archive.json) and [accounting.json](accounting.json).

Final execution checks use **74.13 discovery CPU seconds** and **7.35 separate
audit CPU seconds**, including recovery. The preliminary suite uses **76.25**
and **6.69** respectively; the legacy control uses **2.01** and **0.34**.
Builds, unit tests, orchestration and reporting are excluded from these process
sums. Audit work remains separate from discovery. Copied prefixes are counted
once; newly performed recovery audits are included.

Reproduce with fresh destinations:

```sh
cargo build --release --locked
python3 scripts/smoke_discovery.py \
  --out runs/discovery-reproduction --report reports/discovery-reproduction
```

For portable continuation, extract `source.zip` into a clean directory and
`runs.zip` beneath its `runs/smoke` directory, build, then run:

```sh
./target/release/gso search --from-scratch --fixed-er --algorithm greedy \
  --manifest runs/smoke/inputs/geo/manifest.json --seed 231 --seconds 4 \
  --out runs/smoke/restored --audit 32
```

The next quality study must freeze held-out parties and references before
tuning, compare equal budgets and repeated starts, and reveal reference cycles
only after discovery. Default action forms are currently covered; arbitrary
action parameters and character-specific status predicates are not enumerated.
No full batch, cloud resources or serving integration is included.
