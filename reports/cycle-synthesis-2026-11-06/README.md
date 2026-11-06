# Cycle-synthesis pilot (2026-11-06)

**Measured result on 60 fresh held-out cases** (25 primary, A/L ≥ 2).
Protocol: [`docs/cycle-synthesis-pilot.md`](../../docs/cycle-synthesis-pilot.md)
sections 5–6 and Amendment 1 (implementation choices and development run,
written before the draw). The draw ([draw.json](draw.json), seed 20261107) was
committed as `04e8202` before any held-out search. Its protocol hash was
`aae6610c…` and its implementation commit was `f0bebe4`. The search code is
the same as in `70e99c7`.

In short (conditions below apply to every number):

- **Primary (A/L ≥ 2, n = 25): "SYN beats V21S".** The paired difference of
  the selected programs at 384 CPU-s, (S_SYN − S_V21S)/A, has median
  **+2.49% of A**. SYN is better in 17/25 cases. Wilcoxon two-sided p = 0.021.
- **Low-case count.** Primary cases below 0.90 A: V21S **15**, SYN **6**,
  XO **8**. Nine cases are at or above 0.90 only with SYN; none only with
  V21S. Exact McNemar p = 0.0039.
- **Median S/A, primary:** V21S 0.883, SYN 0.947, XO 0.943.
- **XO alone (L plus transplants, no skeleton, no typed moves) also beats
  V21S** (+3.26%, 19/25, p = 0.011). SYN vs XO: no detectable difference
  (−0.86%, 9/25, p = 0.55). **The gain is attributable to whole-cycle
  retrieval and the family competition, not shown to come from the
  skeleton/typed moves.**
- **The gain survives the selection rule.** The start-averaged difference
  points the same way (SYN +4.54%, p = 0.032; XO +7.29%, p = 0.0016).
- **No cost on easy cases.** For A/L < 1.25 (n = 12), SYN vs V21S is +1.49%
  (p = 0.041), so the reading is "no cost detected". The secondary stratum
  (1.25 ≤ A/L < 2, n = 15) shows no detectable difference (+2.26%, p = 0.26).
- **Controls (n = 8).** 0 regressions in every arm.
- **gcsim.** Of 299 configurations (L, A and the three arms' selections),
  0 were refused at parse time. One SYN selection failed one gcsim iteration
  at runtime ("high_plunge can only be used while airborne"). Where both
  engines ran, gcsim agreed with native to a median 0.12% (max 2.2%). On
  gcsim, the primary comparison repeats: SYN vs V21S +2.2%, 17/25,
  p = 0.017; XO vs V21S +3.3%, p = 0.009.

## Conditions

- **Engine and fight:** native engine `gscouter-engine@1ba0af1`. 450 s fight,
  first 150 s excluded, level-100 target, 10% resistance, radius 2, swap
  delay 12, periodic energy every 480–720 frames.
- **Build and inputs:** build B from the stage-1 export, fixed ER. The v2.1
  dictionary with the v2 k = 10 drop list. Main-library corpus of Gscouter
  `6e0e82205` (4,570 rotations, no `alt/`). Donors with character-normalised
  similarity to A ≥ 0.90 were dropped offline.
- **Arms:** V21S (v2.1 single pass), SYN and XO. Starts 11/37/91, cumulative
  CPU caps 96/192/384 s.
- **Selection:** the selected start is the one with the highest 32-fight
  selection-block mean, on its own seed namespace, run by the search after the
  cap. That CPU (≈ 0.25 s) is reported separately.
- **Scoring:** S is the 128-fight audit of the selected program. All three arms
  of a start and cap use identical audit fights. A selected audit failure
  counts as S := L (0 primary cases).
- **A and L:** measured locally after every job froze. L reproduced the job's
  audited L fight for fight in all 1,620 start-cap combinations.

## Execution and CPU

- **Held-out run:** GitHub Actions run 42162146078 (push of `04e8202` to
  `ci/cycle-synthesis-runs`). 60 jobs, one per case, at most 40 concurrent,
  all arms and starts of a case on one runner. It took 53 min end to end
  (04:22–05:15 UTC); jobs ran 21–30 min each. Runner CPUs (per job in
  [accounting.json](accounting.json)): AMD EPYC 7763 / 9V45 / 9V74, Intel
  Xeon 8573C / 8370C / 6973P-C.
- **Leakage:** strace saw 0 forbidden paths in 1,620 search invocations.
- **CPU:**

  | Item | CPU-h |
  |---|---:|
  | Held-out jobs | 60.9 |
  | Development run (Amendment 1) | 20.4 |
  | Local confirm, smoke and parity | ≈ 0.9 |
  | **Total** | **≈ 82** (cap 120) |

## Mechanism and attribution

- **Where SYN's selected programs came from** (primary): L family 8,
  transplant family 11, skeleton family 6.
- **Own-status guards.** I counted selected programs at 384 s that contain a
  status-plus-skill-ready guard (`.X.mods.K && .X.skill.ready`) not present
  in L, from any source:
  - SYN: 5 primary and 1 secondary, all from skeleton families.
  - V21S: 1, a v2 template guarding Xiangling with `cyno-a1`, a status
    Xiangling never has.
  - XO: 0.

  The SYN guard programs, as-is (S/A, r), with the status names the probe
  read from the engine:

  | Case | Guard status names | S/A | r |
  |---|---|---:|---:|
  | `yoimiya-yelan-furina-xilonen` | `yelanc4`, `yelan-a4` | 0.890 | 0.86 |
  | `hutao-yelan-xingqiu-kaedeharakazuha` | `yelanburst`, `yelan-a4` | 0.947 | 0.93 |
  | `yoimiya-furina-yelan-bennett` | `yoimiya-c2` | 0.924 | 0.90 |
  | `bennett-xiangling-furina-kaedeharakazuha` | `furina-burst`, `khaj-nisut-buff` | 0.635 | 0.29 |
  | `dehya-emilie-durin-nicole` | `dehya-burst` | 0.975 | 0.97 |
  | `yelan-furina-escoffier-citlali` (secondary) | `yelan-a4`, `yelanc4` | 0.936 | 0.80 |

- **Closeness of served programs to A.** Character-normalised similarity of
  the selected program to A ([selected-similarity-to-A.json](selected-similarity-to-A.json)),
  primary, median / max: V21S 0.32 / 0.62, SYN 0.29 / 0.74, XO 0.31 / 0.74.
  No served program reaches 0.90.
- **Budget curve.** Median S/A for the primary stratum at 96 / 192 / 384 s:
  V21S 0.830 / 0.859 / 0.883, SYN 0.912 / 0.938 / 0.947, XO 0.909 / 0.936 /
  0.943. SYN and XO at 96 s are above V21S at 384 s.

## What this does and does not show

- **Shows (measured):** under the conditions above, on 25 fresh broken-L
  cases, competing families seeded from other parties' main-library cycles
  select better programs than the v2.1 single pass at the same CPU (median
  +2.5% to +3.3% of A). They cut the number of cases below 0.90 A from 15 to
  6–8, with no detected cost where L is already good.
- **Does not show** that the skeleton generator or the typed moves add
  anything. SYN does not differ detectably from XO in any stratum, and in the
  controls XO beats SYN (−1.4%, p = 0.012, n = 8). XO is the simpler design
  supported by this run.
- **Does not show** optimality or results on other builds. A is a community
  rotation, and 6 SYN / 8 XO primary cases remain below 0.90 A
  (e.g. `xianyun-furina-xiangling-bennett` 0.63, `bennett-xiangling-furina-kaedeharakazuha` 0.64).
- **Anomaly.** In control case `klee-chiori-xilonen-furina` the XO selection
  scores 15.6× A, with Xilonen at about 2.3M DPS. gcsim agrees: 2.47M team DPS
  against A's 158k. L already carries an `execute_action` crystallize hook,
  which the search kept. This looks like a simulation artifact the program
  exploits rather than play, and it is flagged rather than removed. It is
  outside the primary stratum, and the medians and ranks used here are not
  driven by it.
- **Primary stratum size.** The primary stratum is the whole remaining
  A/L ≥ 2 pool (25 cases), and it was chosen from development results before
  the draw.

## Deviations

1. Amendment 1 items 1–9 (implementation choices made before the draw).
   Notably, the selection-block CPU is reported next to the discovery cap
   rather than deducted from it.
2. **Wall time:** jobs ran 21–30 min, not the planned ≈ 16, because each
   start's three caps run sequentially. The CPU per job matches the plan
   (≈ 1.0 CPU-h).
3. **No labeled post-unblinding run was made.**

## Files

- **Results:** [experiments.json](experiments.json) (per case, arm, cap and
  start), [summary.json](summary.json), [tables.md](tables.md),
  [gcsim-parity.json](gcsim-parity.json), [accounting.json](accounting.json).
- **Archives:** [selections.zip](selections.zip) (selected programs,
  synth-seed records, reports, opened-path logs), [audits.zip](audits.zip)
  (per-fight audits).
- **Inputs:** [inputs/](inputs) (per-case inputs, v2.1 dictionary, corpus) and
  [development/](development) (Amendment 1 run; reads A, not evidence).
- **Reproduce:** `python3 scripts/cycle_synth.py draw` (needs the stage-1 export
  in `runs/`), push to `ci/cycle-synthesis-runs`, download
  `cycle-synthesis-results`, then run `cycle_synth.py confirm`,
  `cycle_synth.py report` and `scripts/cycle_synth_parity.py`.
