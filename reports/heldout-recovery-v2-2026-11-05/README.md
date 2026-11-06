# Held-out recovery pilot, version 2 (2026-11-05/06)

**Measured result on 40 new held-out cases (32 informative, 8 control).**
Protocol: [`docs/heldout-recovery-v2.md`](../../docs/heldout-recovery-v2.md),
written and pushed before any v2 code (`350aa02`), amended before the draw
(Amendment 1) and, for the separately labeled run v2.1 only, after the v2
results (Amendment 2). The protocol file the v2 run used has SHA-256
`3d48ef85168b787af4fb0e11ca224744412dcfc9b1cbb55fae48dfb3632e620b`
(recorded in [draw.json](draw.json); implementation frozen at `bbb906f`, draw
committed as `b572cd8` before any held-out search).

In short (each line holds only under the conditions below):

- **Q2, the owner's question.** At equal total CPU (384 CPU-s per start), the
  two-pass arm (192 + a 192 s second pass from the pass-1 result) beat one
  continuous 384 s run by the pre-declared primary metric: start-averaged
  paired difference on identical audit fights, median **+1.21% of L**, better
  in 26/32 informative cases, Wilcoxon p = 0.0045 → reading **"V2T@384 beats
  V2S@384"**. The secondary, selected-start comparison is not significant
  (median +0.29%, p = 0.27). Median r: two-pass 0.999, single pass 0.983.
- **Q1.** v2 at 192 s versus the stage-1 search (v1) at 192 s on the same
  cases and runners: median paired d −0.37%, 16/32 better, p = 0.60 →
  **"no detectable difference at this sample size"**, although median r rose
  from 0.900 to 0.991.
- **Every arm reads "Promising"** on the stage-1 table (median r ≥ 0.5,
  0 regressions). With the assumption constructs stripped, three of four arm
  checkpoints read "Partial (>5% regressions)" because of two cases where the
  strip transform itself makes S fall below stripped L (section "r without
  assumption constructs").
- **Validity caveat found by gcsim parity:** gcsim refuses 6 of the 120 v2
  arm selections at parse time (parameters such as `escoffier skill[hold=…]`
  carried over from another character's template). This motivated the labeled
  post-unblinding run v2.1 (below), not a change to v2's numbers.
- **v2.1** (labeled, post-unblinding; parameters restricted to library
  triples): 0 gcsim refusals; the Q2 primary reading repeats ("V2T@384 beats
  V2S@384", median +0.97%, 21/32, p = 0.019), the selected-start comparison
  is again null (−0.20%).

**Does the two-pass search beat equal-CPU single-pass?** By the pre-declared
primary metric (start-averaged, paired), yes, by about 1% of L, in both runs.
For the one program per case that the selection rule actually picks, no
detectable difference in either run.

## Conditions (apply to every number unless stated)

Native engine `cenoda/gscouter-engine@1ba0af1` through this repository's
evaluator. 450 s fight, first 150 s excluded, one level-100 target, 10%
resistance, radius 2, swap delay 12, periodic energy every 480–720 frames.
Build B = the alternative's `alt/<id>/build.txt` at Gscouter `6e0e82205` with
its own `team.yaml` params, levels and constellations as written (the stage-1
export, reused unchanged). Fixed ER. Team DPS, mean over fights.

Per case, arm and checkpoint, S is the 128-fight audit of the pre-audit
selection of the start with the highest discovery estimate (stage-1 rule).
Audit seeds depend only on start seed and cap, so V1/V2S at 192 and V2S/V2T
at 288/384 are audited on identical fights per start. L and A in a row are
measured on the selected start's audit fights; A was measured locally after
all jobs had frozen their selections, with the CI-built binary
(`gso-v2` SHA-256 `3c9c66cb…`), and the local measurement reproduced the
job's audited L fight-for-fight in every one of the 1,200 start × checkpoint
combinations. r = (S − L)/(A − L). Regression = S − L < −2 paired SE.
A selected program whose audit fails gets S := L (Amendment 1, item 6).

Execution: GitHub-hosted runners (`ubuntu-24.04`, 4 vCPU each), workflow
[`.github/workflows/heldout-v2.yml`](../../.github/workflows/heldout-v2.yml),
run 42134765379 (push to `ci/heldout-v2-runs`, commit `b572cd8`). One job per
case runs all arms and starts of that case, 4 search processes at a time, so
every paired arm comparison is on one machine. CPU models differ between jobs
(AMD EPYC 7763 / 9V45 / 9V74, Intel Xeon 8370C / 8573C / 6973P-C; recorded
per job in [accounting.json](accounting.json)). Budgets are CPU-second caps,
so faster machines give more decisions per start; no cross-case comparison
of decision counts is made.

## What was built (v2, all character-agnostic)

| Item | Disposition |
|---|---|
| **G1** owner-abstracted templates, longer blocks | Adopted. 96-token cap; a template replaces the owner by a placeholder and can be instantiated for any member; 256 statement templates + all 175 guarded-block templates with count ≥ 2; guarded/statement draws 50/50, sqrt(count) weights. Found in development: stage 1 rejected every guarded block containing a parameterized action (`=` treated as assignment), so no `while …xianyun-airborne-buff` loop was ever extracted; fixed (Amendment 1). |
| **G2** annealing and compound moves | Adopted: the agreed annealing acceptance (`Δ ≥ T ln u`), new operators (owner reassignment, action substitution, segment repeat/delete, unit-level template replace/insert, unit delete/move) and compound proposals (30%, 2–3 edits). Peak protection by 32-fight promotion unchanged. |
| **G3** narrower leakage filter | Adopted with k = 10: a block is dropped only if it occurs in A and in fewer than 10 library rotations. Median dropped blocks per case: 7 (v1 rule) → 2 (v2 rule) on the 247 exported cases. |
| **G4** assumption constructs | Adopted: proposals may not add `execute_action` overrides, `pick_up_crystallize` calls or `counter` parameters; micro search never changes `counter`. r is also reported after a strip transform. |
| **G5** minimisation | Adopted inside the second pass only (so its effect is part of Q2). |
| Own additions | Scaled worker wall limit in v2 discovery (0.5 s/fight, 2–10 s) so non-progressing programs are cut at the 2-fight stage; second pass (next section). |
| Not adopted | Character-mechanic gates and per-case rules (owner rule). |

### Second pass

At a declared CPU point (192), a resumed run with `--second-pass-from 192`:
(1) restarts from the pass-1 peak S1 while keeping S1 as the protected peak;
(2) minimises S1 by one sweep of unit deletions accepted when the common
16-fight block does not decrease; (3) adds templates derived from S1 and L;
(4) runs two families — minimised S1 and the archive candidate least similar
to S1 (L in 73 of 120 starts) — under the existing scheduler after a
four-decision initial quantum each; (5) recalibrates temperature and raises
compound moves to 50% with 2–4 edits. All of it is charged to the same
cumulative cap. V2T forks V2S's own state at 192, so V2T vs V2S isolates the
second pass at equal CPU. Minimisation completed in all 120 starts and
accepted a median of 3 deletions.

## Results (v2, pre-registered)

Tables are generated mechanically by `scripts/heldout_v2_report.py`; full
tables, including per-case rows and the budget curve, are in
[tables.md](tables.md).

### Reading per arm (informative, n = 32)

| Arm @ CPU-s per start | Median r | Quartiles | r ≥ 0.5 | Regressions | Reading | Median r_strip | Reading (strip) | Selected audit failures |
|---|---:|---|---:|---:|---|---:|---|---:|
| V1 @ 192 | 0.900 | 0.56 / 0.90 / 1.03 | 27 | 0 | Promising | 0.880 (n=31) | Partial (>5% regressions) | 1 |
| V2S @ 192 | 0.991 | 0.67 / 0.99 / 1.13 | 28 | 0 | Promising | 0.934 | Partial (>5% regressions) | 0 |
| V2S @ 384 | 0.983 | 0.74 / 0.98 / 1.15 | 28 | 0 | Promising | 0.966 | Promising | 0 |
| V2T @ 384 | 0.999 | 0.72 / 1.00 / 1.26 | 28 | 0 | Promising | 0.964 (n=31) | Partial (>5% regressions) | 1 |

Controls (n = 8): 0 regressions in every arm and checkpoint.
Budget curve (median r): V1 0.804 / 0.865 / 0.900 at 36 / 108 / 192;
V2S 0.776 / 0.872 / 0.991 / 0.974 / 0.983 at 36 / 108 / 192 / 288 / 384;
V2T 1.016 / 0.999 at 288 / 384.

### Pre-declared comparisons (informative, n = 32)

d = per case, mean over the 3 starts of (S_x − S_y)/L on identical audit
fights; Wilcoxon signed-rank, two-sided.

| Comparison | Median d | Mean d | x better | p | Reading | Secondary: selected-start median d, p |
|---|---:|---:|---:|---:|---|---|
| **Q2: V2T@384 vs V2S@384** (primary) | +1.21% | +7.24% | 26/32 | 0.0045 | **V2T@384 beats V2S@384** | +0.29%, 0.27 |
| Q1: V2S@192 vs V1@192 | −0.37% | −7.64% | 16/32 | 0.60 | no detectable difference | +1.97%, 0.74 |
| Budget: V2S@384 vs V2S@192 (unpaired fights, Amendment 1 item 7) | +2.07% | +3.17% | 28/32 | 0.0004 | V2S@384 beats V2S@192 | +0.82%, 0.003 |
| Descriptive: V2T@288 vs V2S@288 | +1.02% | +5.61% | 28/32 | 0.0002 | (not pre-declared) | +0.27%, 0.16 |

Means are dominated by a few cases with large A − L (for Q2: three cases
above +45%, two below −15%); the readings use medians and ranks as declared.
The two largest negative Q2 cases include audit failures counted as S := L
(`kaedeharakazuha-furina-chongyun-xianyun`: the selected V2T program needs
"low_plunge or high_plunge while airborne" on one audit fight).

### Per case

L / S(V1@192) / S(V2S@192) / S(V2S@384) / S(V2T@384) / A with r for every
case are in [tables.md](tables.md) ("Per case"); per start, per checkpoint
values, CPU, decisions, audit seeds and errors are in
[experiments.json](experiments.json).

### r without assumption constructs (G4 strip)

`strip-assumptions` deletes `execute_action` overrides, `pick_up_crystallize`
statements and `counter` parameters from L, S and A alike; stripped programs
were measured on the same audit fights. Medians are in the reading table. The
strip readings marked Partial come from two cases:
`bennett-beidou-lisa-xingqiu` (removing `counter` lowers stripped A to 15.7k
against stripped L 25.0k, so A_strip − L_strip < 0 and S_strip falls below
L_strip) and `dehya-bennett-rosaria-kaeya` (A_strip − L_strip ≈ +1.8%,
S_strip ≈ 2–3% below L_strip). These are consequences of the strip
transform on programs built around those constructs, not regressions of the
search on the programs as written (0 regressions there).

### gcsim parity (after measurement)

A temporary Go tool inside the Gscouter checkout (`internal/score`, gcsim
`cenoda/gcsim@eb218847f82b`, no fillers, 256 iterations, seed 777000, same
450/150 s window) ran L, A and the selected S of V1@192, V2S@384 and V2T@384;
the tool directory was deleted afterwards and the Gscouter tree is clean.
Results: [gcsim-parity.json](gcsim-parity.json).

- Where gcsim runs the program, it agrees with the native audit to a median
  0.1% (max 1.5%; different fights).
- **gcsim refuses 6 v2 selections at parse time** (V2S@384: 3, V2T@384: 3),
  e.g. "character escoffier: key hold is invalid for action skill",
  "key bounce is invalid for action skill" (Shenhe, Yelan), "key travel is
  invalid" (Xiangling). One V2T selection fails on one gcsim iteration
  ("need to use attack right before charge"). All three arms' selections for
  `varka-prune-nicole-durin` crash gcsim (nil pointer), as does nothing in
  native. V1 has no parse refusals.
- 30 of the 120 v2 selections (V2S@192: 8, V2S@384: 12, V2T@384: 10) contain a
  `character action key` parameter triple that no library rotation uses
  ([unseen-params.json](unseen-params.json)); the native engine ignores
  unknown keys. On the 24 informative cases where all three arms run on gcsim,
  median r on gcsim: V1@192 0.915, V2S@384 1.032, V2T@384 1.044.

## Labeled post-unblinding run v2.1

Declared in Amendment 2 after the v2 results; details in
[v2.1/README.md](v2.1/README.md). Rule: no proposal may introduce a
`character action key` parameter triple absent from the library and the
parent program. Same cases, seeds and caps; V2S and V2T only (GitHub Actions
run 42140341782, 16 min 25 s; 69,174 CPU-s discovery + 4,383 CPU-s audits).

- gcsim parse refusals: 0 (v2: 6); selections with non-library parameter
  triples: 0 of 120 (v2: 30).
- Median r (informative): V2S@192 0.920, V2S@384 0.992, V2T@384 0.964; all
  "Promising", 0 regressions; 2 selected V2T audit failures (S := L).
- **Q2 again by the primary metric: "V2T@384 beats V2S@384"**, median d
  +0.97%, 21/32, p = 0.019; selected-start secondary −0.20%, p = 0.84.
- Cross-run (descriptive, same fights, other runners): v2.1 V2S@192 vs V1@192
  median −1.30%, p = 0.27.

## What this does and does not show

- **Shows (measured):** on 32 informative new held-out cases under the
  conditions above, a second pass that restarts from the first pass's result,
  minimises it, derives templates from it and L and runs two S-derived
  families gave a small, consistent paired gain over spending the same 192
  CPU-s continuing the first pass (median +1.2% of L, 26/32), by the declared
  primary metric.
- **Does not show** that the second pass improves the program that would
  actually be selected: the selected-start comparison (one program per case,
  as served) is +0.3% and not significant. Start averaging measures the
  method; selection adds selection noise.
- **Does not show that v2's changes beat v1 at 192 s.** Median r is higher
  (0.99 vs 0.90) but the paired test is null, and some v2 gains rely on
  parameters gcsim rejects.
- **r is relative to A**, a community rotation, not an optimum; r > 1 means
  only that S scores higher natively under these conditions (16/32 cases for
  V2T@384, 10/32 for V1@192).
- **Cost of the new operators.** v2 starts recorded 6,090 (V2S) and 4,296
  (V2T) worker timeouts in all, each charged up to 2 s (V1: 1,001 at 10 s);
  compound proposals failed 12,505 times of 63,321 attempts in V2S. These are
  charged to the same caps.
- **Validity.** Native validity is not gcsim validity (above). A program
  passing 16 + 32 + 32 search/promotion fights can still fail an audit fight
  (engine errors such as "Saichi Slash into Saichi Slash", "unsupported field"
  after an owner reassignment); 3 of the 400 selections (40 cases × 10 arm
  checkpoints) did: V1@108 and V1@192 in one case, V2T@384 in another.
- Runner hardware differs between cases; within a case, all arms share a
  machine.

## Deviations

1. Amendment 1 (before the draw): items 1–9, including the guarded-block fix,
   template retention, scaled wall limit, pass-2 initial quantum, the
   audit-failure rule, the unpaired budget comparison and the exclusion of the
   development parties' alternatives (found after a first, uncommitted draw
   contained one; that draw was discarded before any search).
2. Budget: the protocol's original local plan was ≈16 CPU-hours; with the
   owner's runner allocation it ran 40 cases, declared before the draw.
   Measured: 92,376 CPU-s discovery (25.7 CPU-h) + 7,346 CPU-s audits on
   runners (job children CPU total 101,089 s = 28.1 CPU-h); local A/L/strip
   measurement 1,235 CPU-s; development: one CI run on 12 stage-1 cases
   (27,693 CPU-s) plus local smoke tests. No per-start budget was shortened
   and no case was dropped.
3. Wall time: the v2 held-out workflow ran 19 min 39 s end to end
   (00:09:37–00:29:16 UTC), build 1 min 38 s; per-case jobs 11–17 min
   (median 11.5 min). Summed job wall 8.1 h.
4. Labeled post-unblinding run v2.1 (Amendment 2), reported separately.
   CI attempts that ran no search: the first development run (build job
   lacked a compile-time data file in its sparse checkout) and the first v2.1
   attempt (build job lacked the v2.1 matrix). A local development run was
   stopped when a rebuild replaced its binary; its results were discarded.
5. The development V1 numbers differ from stage 1's on the same cases
   (different hardware gives different decision counts at the same CPU cap).

## Files

- [draw.json](draw.json), [matrix.json](matrix.json), [inputs/](inputs): the
  frozen draw and exactly what each job received (manifest, L, build, drop
  lists, global dictionaries).
- [experiments.json](experiments.json): every case, arm, start, checkpoint
  (L, A, S, r, SE, CPU, decisions, timeouts, audit errors, strip values).
- [summary.json](summary.json), [tables.md](tables.md),
  [accounting.json](accounting.json) (per job: wall, CPU, nproc, CPU model,
  runner image), [ops-summary.json](ops-summary.json) (operator outcomes),
  [gcsim-parity.json](gcsim-parity.json), [unseen-params.json](unseen-params.json).
- [selections.zip](selections.zip): per job/arm/start/checkpoint report,
  frozen shortlist, selected program, opened-file list (strace), second-pass
  record; [audits.zip](audits.zip): per-fight audit DPS.
- [development/](development): the 12-case development run (reads A; not
  evidence).
- Reproduce: `python3 scripts/heldout_v2.py filter|draw` (needs the stage-1
  `runs/heldout` export), push to `ci/heldout-v2-runs`, download the
  `heldout-v2-results` artifact to `runs/heldout-v2/results`, then
  `python3 scripts/heldout_v2.py confirm --v2-bin <CI gso-v2>` and
  `python3 scripts/heldout_v2.py report`.
