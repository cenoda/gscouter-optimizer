# Held-out recovery pilot, version 2

**Status: proposed protocol, written 2026-11-05 before any v2 code exists.
Nothing here is a measured result.** It follows the stage-1 protocol
(`docs/heldout-recovery-pilot.md`, unchanged) and the post-hoc diagnosis in
`reports/heldout-recovery-2026-11-05/failure-analysis/README.md`. Results go to
`reports/heldout-recovery-v2-<date>/`.

## 1. Questions

- **Q1 (v2 vs v1).** Do the generic changes in section 3 raise recovery
  r = (S − L) / (A − L) on *new* held-out cases at the same 192 CPU-s per start?
- **Q2 (second pass).** Does a second pass of 192 CPU-s that restarts from the
  first pass's result beat simply continuing the same search for 192 more
  CPU-s (equal total CPU, 384)?
- **Q3 (budget).** What do v2's budget curves (36, 108, 192, 288, 384) look like?

"No improvement" and "the second pass does not beat equal-CPU continuation"
are acceptable outcomes.

## 2. Cases

**Development cases** (may be used for debugging and tuning, never for
measurement): the 30 stage-1 cases (they were analysed with A), Flins C1
(`data/flins-c1/standard.json`) and Mavuika C6R5-225 (`data/mavuika-c6-r5-225`).

**Held-out measurement draw.** Pool: rows of
`reports/heldout-recovery-2026-11-05/classification.json` with class
`informative` or `control`, kind `database` (the `citlali-c6-no-burst`
variants are excluded: stage 1 showed they are easy by construction), and not
one of the 30 stage-1 cases. At writing this pool has 131 informative
(69 frame, 62 other) and 44 control cases. Classification, conditions and
builds are the stage-1 ones (no re-measurement of the classification).

Draw: `random.Random(20261106)`, stratified by the stage-1 `frame` stratum
(A uses `hold=` or `while`), largest-remainder proportional allocation:
**32 informative + 8 control = 40 cases**, then a seeded shuffle of the run
order. The draw file (case list, order, per-case input hashes) is committed and
pushed before any held-out search. If cost exceeds the plan, a prefix of the
committed order is used (per-start budgets are never shortened); this is
recorded as a deviation.

## 3. Changes in v2 (all character-agnostic)

Proposed G1–G5 from the failure analysis, with dispositions:

- **G1 adopted (owner-abstracted templates, longer blocks).** The extraction
  rule of stage 1 (action statements; `if`/`while` blocks referencing exactly
  one character and only built-in calls) is kept, but the token cap rises from
  40 to **96**, and every extracted block also yields a *template*: the
  owner's name and owner field paths are replaced by a placeholder. Templates
  are counted by distinct main library rotations (count ≥ 2), the **256** most
  common are kept, and the search may instantiate a template for **any** party
  member. Template picks are weighted by sqrt(count) so trivial one-action
  templates do not crowd out longer ones. The per-character dictionary of stage
  1 is kept (same 64-per-character limit, now with the 96-token cap). A
  template that is meaningless for a member fails at parse or in the engine;
  that failure is charged like any other.
- **G2 adopted, both parts.** (a) Acceptance becomes the agreed annealing rule
  (`delta >= T·ln u`, same u for the proposal, temperature calibrated from
  bootstrap of the first common block and decayed per decision, as already
  implemented for `--algorithm anneal`). Peak protection is unchanged: the
  selected program changes only through the 32-fight paired promotion.
  (b) New operators: *action substitution* (replace one action key in a
  statement with another key from that owner's vocabulary: keys in the
  current program for that owner, in that owner's dictionary blocks, and the
  engine's basic attack/charge/skill/burst/dash/jump), *segment repeat*
  (duplicate or delete a contiguous run of 1–4 actions in one action list),
  and *compound proposals* (with probability 0.3, chain 2–3 edits from the
  whole operator set into one proposal, evaluated once).
- **G3 adopted with a declared k.** For each case, a dictionary block or
  template is dropped only if (i) its normalized text (for a template: its
  instantiation for any party member) occurs in A, **and** (ii) it occurs in
  fewer than **k = 10** distinct main library rotations. Common library text
  is treated as general knowledge, rare text that matches A as possible
  leakage. v1 keeps its stage-1 filter (any match is dropped).
- **G4 adopted (assumption constructs).** The declared list is:
  overrides of `execute_action` (`fn execute_action` / `_execute_action`),
  calls to `pick_up_crystallize`, and the `counter` action parameter. The
  search may not add any of them: a proposal whose count of any construct
  exceeds its parent's is rejected statically (free, recorded), and micro
  search never changes a `counter` value. `hold` values stay searchable
  (they are play) but are reported. r is reported both on the programs as
  written and after a mechanical strip transform applied to L, S and A alike:
  delete `execute_action` overrides and their alias, delete
  `pick_up_crystallize(...)` statements, delete `counter=` parameters. A
  stripped program that fails to run is reported as unavailable, not repaired.
- **G5 adopted, inside the second pass only** (see section 4), so that its
  effect is part of what the equal-CPU comparison measures.
- **Unchanged:** micro coordinate search (±1, ±25%) in the last quarter of
  each cumulative cap; restarts after 40 non-improving decisions (3 edits from
  the peak); 16-fight common decision blocks; 32-fight promotions; checkpoints,
  journals, accounting, the 128-fight independent audit and fixed ER.
- **Not adopted:** character-mechanic gates (e.g. a C6 state guard for one
  character) and any per-case rule (owner rule).

These parameters (0.3, 2–3, sqrt weighting, 256, 96, k = 10, operator mix) are
initial values. They may be adjusted on development cases only. The final
values are frozen in the implementation commit named in the draw file before
any held-out search; every change from the values above is listed there.

## 4. Second pass

A second pass is an explicit, charged reset of the search state at a declared
CPU point P (here P = 192), applied once when a run is resumed with
`--second-pass-from P`:

1. **Restart from the result.** The pass-1 peak S1 (the program the 192 s
   selection rule would audit) becomes the start. The peak itself is not
   reset; anything found later must still beat S1 by 32-fight promotion.
2. **Minimise (G5).** One sweep over S1's action statements and guarded
   blocks, in order: delete one; accept if the common 16-fight block does not
   decrease (Δ ≥ 0). The result is the start of family 0.
3. **Derive templates from S1 and L.** Owner-abstracted blocks are extracted
   from S1 and L with the section-3 rule (no minimum count) and added to the
   template pool, so patterns the search built or the library rotation uses
   can be reassigned to other members.
4. **Outer competition of S-derived structures.** Two families, within the
   two-active-chain limit: family 0 = minimised S1; family 1 = the retained
   archive candidate least similar to S1 (token-set Jaccard), or L if the
   archive holds no other program. The existing bounded scheduler divides the
   pass-2 budget between them; both are paused states, one executes at a time.
5. **Reheat and larger moves.** Each family's temperature is recalibrated on
   its first decision; compound probability rises to 0.5 with 2–4 edits.

Everything in pass 2 (minimisation, template extraction, both families,
promotions) is charged to the same cumulative cap. The selection rule and the
audit are identical to the single-pass arm.

## 5. Arms

All arms use seeds 11, 37 and 91 (3 starts) and the same cases.

| Arm | Code | Schedule (cumulative CPU-s per start) |
|---|---|---|
| **V1** | stage-1 search, commit `67647e9` source, v1 dictionary and filter, greedy | checkpoints 36, 108, 192 |
| **V2S** (single pass) | v2 | checkpoints 36, 108, 192, 288, 384 (one continuous run) |
| **V2T** (two pass) | v2 | shares V2S's state at 192, then `--second-pass-from 192`; checkpoints 288, 384 |

V2T's first pass *is* V2S's first 192 s (same binary, seed and state), so the
V2S/V2T comparison at 384 isolates the second pass at equal total CPU. Audit
seeds depend only on start seed and cap, so V1/V2S at 192 and V2S/V2T at 288
and 384 are audited on identical fights per start.

## 6. Metrics

Per case, arm and checkpoint: S = audited score of the pre-audit selection of
the start with the highest discovery estimate (stage-1 rule). L and A are
measured on that start's audit seeds. r = (S − L)/(A − L); regression =
S − L < −2 paired SE. Also: r_strip (section 3, G4) on the 192 and 384
checkpoints, per-start S, start spread, CPU, decisions, failures, S features
and the diff of S against L.

Paired arm comparisons use, per case, the **start-averaged paired
difference** d = mean over starts of (S_X(start) − S_Y(start)) / L, where both
S values are audited on the same 128 fights. The selected-start difference is
reported as secondary.

## 7. Pre-declared readings

**Per arm and checkpoint (informative cases; the stage-1 table):**

| Median r | Regressions (S < L by > 2 SE) | Reading |
|---|---|---|
| ≥ 0.5 | ≤ 5% of cases | Promising |
| ≥ 0.5 | > 5% | Partial |
| 0.2–0.5 | any | Partial |
| < 0.2 | any | Not supported |

Applied to V1@192, V2S@192, V2S@384 and V2T@384, and separately with r_strip.

**Comparisons (informative cases, two-sided Wilcoxon signed-rank on d):**

| Comparison | Reading |
|---|---|
| median d > 0 and p < 0.05 | X beats Y |
| median d < 0 and p < 0.05 | Y beats X |
| otherwise | no detectable difference at this sample size |

Comparisons: **V2T@384 vs V2S@384** (Q2, primary), V2S@192 vs V1@192 (Q1),
V2S@384 vs V2S@192 (budget). Controls: count of regressions per arm. No other
threshold is used for a headline claim.

## 8. Leakage

- Searches open only their case's manifest, L, dictionary and their run
  directory; every search runs under `strace` (open/openat/execve) and fails
  on any path containing `/alt/`, `withheld` or `/Gscouter/`.
- Dictionaries are built from the 4,570 main `teams/*/rotation.gcsl` only. A
  is read by the leakage filter (section 3, G3, rule declared here) and by the
  final measurement, nothing else.
- A, A_strip and L measurements on audit seeds run only after every search
  job has frozen its selections.

## 9. Execution venue and budget

Searches run on GitHub-hosted runners (`cenoda/gscouter-optimizer`, workflow
triggered by a push to a dedicated branch `ci/heldout-v2-runs`, nothing on
`main`). Matrix = cases (≤ 40 jobs). **All arms and all starts of one case run
in the same job**, 4 search processes at a time on the runner's vCPUs, so
paired arm comparisons share hardware. Each job records `nproc`, the CPU model
and the runner image. Jobs receive only the case's exported inputs (manifest,
L, the per-case dictionary drop lists plus global dictionaries, the engine
pin); the checkout is sparse (source, scripts, those inputs), with no `alt/`,
no A and no Gscouter checkout. Each job uploads an artifact; results are
downloaded and committed by hand. A is measured afterwards locally with the
same engine pin; the measurement must reproduce the job's audited L per fight,
or the mismatch is reported.

Cost plan per case: V1 3 × 192 + V2S 3 × 384 + V2T 3 × 192 = 2,304 CPU-s of
discovery, plus audits (≈ 10 checkpoints × 3 starts). For 40 cases ≈ 26
CPU-hours of discovery on runners (≈ 10–15 min wall per job), plus local
measurement (≈ 1 CPU-h) and local development (≤ 2 CPU-h). This exceeds the
earlier 16 CPU-hour local plan because the owner made 40 concurrent runner
jobs available; the case count is declared here, before the draw. Actual wall
time and summed CPU are reported. No VMs or spending are implied.

## 10. Reporting rules

Stage-1 rules apply: conditions next to every number; compare only identical
conditions; distinguish measured, estimated and hypothesized; report
failures, exclusions and regressions. Any change after held-out results are
seen is a new labeled run, reported alongside this one. Runner-to-runner
hardware differences mean decisions per CPU-second differ between cases; this
is reported, and no cross-case comparison of decision counts is made.

## Amendment 1 (2026-11-06, after development, before the held-out draw)

Made on development cases only (section 2). No held-out case had been drawn
or searched. Development results are in
`reports/heldout-recovery-v2-2026-11-05/development/`: 12 stage-1 cases, run
once with the same workflow on GitHub-hosted runners; they read A and are
not evidence for the questions above.

Implementation changes from section 3–4 values:

1. **Guarded-block extraction (G1).** Inside an action parameter list,
   `name=value` is a parameter, not an assignment. Stage 1 (and the first v2
   draft) rejected every `if`/`while` block containing a parameterized
   action, which is why no `while ... xianyun-airborne-buff` loop was
   extracted even with the larger cap.
2. **Template retention.** The 256 most common *statement* templates plus
   **all** guarded-block templates with count ≥ 2 (431 templates). A template
   draw picks guarded or statement templates with probability 1/2 each, then
   sqrt(count) weighting within the group. With a single 256 cut, the
   one-statement templates (counts up to 2,755) removed every loop template.
3. **Worker wall limit (v2 discovery only).** 0.5 s per fight in the batch,
   between 2 and 10 s (stage 1: 10 s for every batch). A program that makes
   no progress is now cut at the 2-fight stage instead of costing about 10
   CPU-s. Audits keep the 10 s limit. V1 keeps its own code unchanged.
4. **Second-pass initial quantum.** Each pass-2 family first receives four
   decisions before the existing scheduler allocates the rest.
5. **`gso config`** renders a rotation's full config (for the gcsim parity
   check); no search effect.

Measurement rules added (they are not chosen from results; development showed
the situations exist):

6. **Audit failure of the selected program.** A selected program that fails
   its 128-fight audit (an engine error on some fight, e.g. "high_plunge can
   only be used while airborne", or a worker timeout) gets **S := L** for r,
   regression and paired differences (the deployed fallback would be the
   baseline). Such cases are counted and listed per arm. The selection rule
   is unchanged (no re-selection after the audit).
7. **Budget comparison V2S@384 vs V2S@192** uses different audit fights per
   cap, so its d is the difference of relative gains
   (S_384/L_384 − S_192/L_192), not a paired difference. The two primary
   comparisons (V2T vs V2S at 384; V2S vs V1 at 192) are paired as declared.
8. **Draw size.** 40 cases as declared (32 informative + 8 control), one
   runner job per case; no prefix is expected to be needed.
9. **Pool exclusion of the development parties.** The alternatives of
   `flins-ineffa-columbina-sucrose` (Flins C1) and
   `mavuika-citlali-xilonen-nicole` (Mavuika C6R5) were used as known
   references in earlier reports (`reports/pilot-2026-11-05`,
   `reports/mavuika-c6-blind-2026-11-05`), so they are development material
   and leave the pool (131 → 129 informative). Found by a mechanical search of
   the repository for pool alternative ids after a first, uncommitted draw
   contained the Mavuika one; that draw was discarded before any search and
   the draw below is rerun with the same seed.
