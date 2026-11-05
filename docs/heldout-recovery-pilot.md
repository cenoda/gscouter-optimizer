# Held-out recovery pilot

**Status: proposed protocol. Nothing in this document is a measured result
unless it cites a report.** Written 2026-11-05 at the owner's request, before
implementation, after a review of the existing pilot branches. Implementation
goes on a separate branch on top of this one.

## 1. Why this pilot

The branches so far tested discovery on two development parties. They show:

- **Flins C1** (energy-rich, macro order dominates). Greedy retry at 108
  CPU-seconds, revised fixed ER: 343,136 / 324,579 / 349,607 across starts 11,
  37 and 91, against a known library alternative of 338,537. Two of three
  starts exceed it; the selected one is +3.30% after 2,048-fight confirmation
  (`reports/pilot-retry-2026-11-05`). The search started from the library
  baseline.
- **Mavuika C6R5** (frame-level charge/dash timing). Blank-start discovery was
  44–49% below known cycles (`reports/mavuika-c6-blind-2026-11-05`,
  `reports/mavuika-c6-external-2026-11-05`). Phase 3 reached 2,014,067 against a
  human reference of 2,316,838 under identical conditions
  (`reports/mavuika-225-2026-11-05`).
- The "hierarchical combo" report on `feat/structure-competition` (`dd32e77`)
  claims +2.45% over 1,743,720. That comparator was measured at 450 s / 150 s
  warmup, level 90, level-100 target and swap delay 12. The claimed result used
  120 s, level 100, a level-110 target and swap delay 1. Under the claimed
  result's own conditions (same manifest hash `8143b8f9…`) the human reference
  is 2,316,838, so the result is 22.9% below it and below Phase 3's best. The
  report has no experiments file, accounting or protocol. **It is not evidence
  for this pilot.**

What the branches do **not** establish:

- That CPU is or is not the bottleneck. From 36 to 108 CPU-seconds, Mavuika
  Phase 3 extended starts gained +10%, 0% and +17%; Flins gained about +5%
  per tripling. Gains have not clearly stopped. Nothing measured goes beyond
  108 s.
- That seeded starts beat blank starts. The seeded success (Flins) and the
  blank failure (Mavuika) are different parties, so start type and party
  difficulty are confounded.
- Whether either result generalizes. Two parties is not a suite.

The current generator (`src/discovery.rs`) emits `while`, `hold` and state
guards only through a few hand-written templates. In the Gscouter library
snapshot (`6e0e82205`), 2,344 of 4,570 rotations use action parameters, 1,646
use `hold=` and 1,414 use `while`. Much of known good play is outside what the
generator can propose.

## 2. Questions

- **Q1 (recovery).** Starting from a team's library rotation and searching at a
  higher-constellation build, how much of a *known* higher-constellation gain
  does the search recover, without seeing the known rotation?
- **Q2 (budget).** How much is gained from 108 to 192 CPU-seconds per start,
  and from 192 to 576?
- **Q3 (scope).** Which cases do not converge, and are they the frame-sensitive
  kind that Mavuika represents?

A failed or partial answer is a valid outcome. The pilot measures; it does not
have to pass.

## 3. Held-out set

Source: Gscouter `teams/*/alt/*` at a pinned Gscouter commit (currently
`6e0e82205`: 247 alternatives over 231 teams). Each alternative is a community
database entry with a higher-constellation build than the team's library build
(Gscouter `docs/cycle.md` §7; on 191 measured teams, its rotation beats the
library rotation on its own build by a median +12.4%, over +5% for 136).

For each case:

- **B**: the alternative's build (`alt/<id>/build.txt`), used as is. Builds are
  fixed experimental conditions; no artifact allocation moves (fixed ER).
- **L**: the team's library rotation (`teams/<team>/rotation.gcsl`), the start.
- **A**: the alternative's rotation (`alt/<id>/rotation.gcsl`), **withheld**.

Measure L and A natively on B under the common conditions below. Gscouter's
scores use a different pipeline (fillers, 50 iterations), so they are only used
to choose cases, never as the reference.

- **Informative**: A − L ≥ 5% of L and ≥ 4 paired SE. Recovery is reported on
  these.
- **Control**: the rest. The search must not end below L by more than noise.
- **Excluded**: cases the pinned engine cannot parse or run, or whose L or A
  collapses (below 5k DPS). Listed with the reason, never silently dropped.

Common conditions: 450 s fight, first 150 s excluded, one level-100 target,
10% resistance, the builds' own levels and constellations, swap delay 12,
the Flins pilot's periodic energy. These match the library's scoring horizon.

### Leakage rules

- The search never reads anything under `alt/`. A file check in the run
  records the hashes of every input the search opened.
- The block dictionary (§4) is built only from the 4,570 main library
  rotations. Blocks whose normalized text appears in any case's A are dropped
  from the dictionary for **that** case.
- A is used only by the measurement step, after the search has frozen its
  selection.

## 4. Search

Reuse the existing production path: `evaluate.rs` common 16-fight decision
blocks, the 32-fight promotion rule, checkpoints, journals and accounting.
These passed native/gcsim parity within 0.06% per major contributor.

- **Block dictionary.** Parse the library rotations with the pinned engine
  parser and extract per-character action statements and small guarded blocks
  (an action list with its parameters, a `while`/`if` body owned by one
  character, a hold or delay pattern). Key blocks by character; count how many
  library rotations contain each. This is mechanical extraction, not
  hand-written per-character rules.
- **Macro level.** Greedy local search with restarts over edits to L: replace,
  insert, delete or reorder one character statement, using dictionary blocks for
  that character and the existing generic action-list neighborhoods. A move is
  accepted only through the common-block decision.
- **Micro level.** With the macro structure fixed, deterministic coordinate
  search over numeric parameters already present (hold, delay, repeat counts).
- **Incumbent.** L stays the incumbent until something beats it; the selected
  program can be L itself.

This is a bounded deviation from the planned annealing design and is labeled
as such. It is the simplest search that tests whether library-derived blocks
plus seeded starts work. If it does not, that says nothing yet about annealing.

Development cases for debugging are Flins C1 and Mavuika C6R5 (already
revealed). Held-out cases are not used for tuning. Any change after held-out
results are seen is a new run with a new label, and both runs are reported.

## 5. Measurement

- 3 starts per case (seeds 11, 37, 91). Budget checkpoints at 36, 108 and 192
  cumulative CPU-seconds; a 576-second continuation on a declared subset.
- At each checkpoint, freeze the pre-audit selection and run an independent
  128-fight audit on fresh seeds. Audit results never feed back into search.
- Per case, S is the audited score of the selection with the highest discovery
  estimate among the starts (a rule fixed now, not chosen by audit).
- Recovery r = (S − L) / (A − L). r > 1 is possible (A is not optimal).
- Report for each case: L, A, S per start and per checkpoint, r, CPU spent,
  rotation features (uses `hold`, `while`, frame delays), and the diff of S
  against L.

### Pre-declared reading

On informative cases, at 192 s:

| Median r | Regressions (S < L by > 2 SE) | Reading |
|---|---|---|
| ≥ 0.5 | ≤ 5% of cases | Promising: worth the full stage |
| 0.2–0.5 | any | Partial: identify which kinds succeed before scaling |
| < 0.2 | any | Not supported at this budget and grammar |

Converged at a case: 192 → 576 gains < 1%. Start spread: range of audited
scores across starts, reported per case.

Stage 1 is small. Its reading decides whether stage 2 is worth running; it is
not evidence for the main batch.

## 6. Stages and budget

The earlier estimates in conversation (122 and 367 CPU-hours) multiplied by
four profiles per team. This set has one target build per case, so the
multiplier is one.

| Stage | Cases | Cost | Wall time, 4 cores |
|---|---|---|---|
| 1 (implementation branch) | ~24 informative + ~6 controls, 3 starts × 192 s; 6 of them continued to 576 s | ≈ 4.8 + 2.9 ≈ 8 CPU-h | ≈ 2–3 h |
| 2 (later, if stage 1 is not "not supported") | all eligible (~191), 3 × 192 s; ~35 continued to 576 s | ≈ 31 + 17 ≈ 48 CPU-h | ≈ 12 h, or ≈ 1 h on 64 cores |

Measurement of L and A and audits are extra and are recorded in accounting.
Stage 1 cases are drawn with a fixed seed from the eligible list, stratified by
whether A uses frame-level `hold`/`while` patterns, and the draw is committed
before any search runs. Stage 2 reuses stage 1's code unchanged, or is labeled
as a new version.

All of this falls inside the 600 CPU-hour pilot allocation. No VMs or spending
are authorized by this document.

## 7. Reporting rules

- Report what was measured, including failures, unsupported cases and
  regressions, in `reports/heldout-recovery-<date>/` with experiments,
  accounting, case list and protocol hash.
- Compare only measurements taken under identical conditions. State the
  conditions next to every number.
- Do not add character-specific rules, special cases for failing cases, or
  thresholds chosen after seeing results to make the reading look better. If
  the search does not work, the report says so.
- Distinguish measured results, estimates and hypotheses.

## Deviations (appended 2026-11-05, after stage 1 ran)

Recorded in full in `reports/heldout-recovery-2026-11-05/README.md`. In short:
B includes the alternative's own `team.yaml` params, applied to L, A and the
search alike (the protocol was silent; 21 cases have differing params); the
40 `citlali-c6-no-burst` variants counted in the 247 were kept and tagged;
the discovery-profile constellation grid is not applied to held-out builds;
restarts (perturbation of the peak after 40 non-improving decisions) and the
micro schedule (last quarter of each cumulative cap) were chosen at
implementation; the reading table's uncovered cell (median r ≥ 0.5 with more
than 5% regressions) was declared "read as Partial" before any r existed and
did not occur. Thresholds, the selection rule and the case list were not
changed. Stage 1 ran against this document's previous version (SHA-256
`14bea063…`).
