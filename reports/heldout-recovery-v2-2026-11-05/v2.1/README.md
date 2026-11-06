# Labeled post-unblinding run v2.1 (2026-11-06)

**Post-unblinding.** Declared in Amendment 2 of
[`docs/heldout-recovery-v2.md`](../../../docs/heldout-recovery-v2.md) *after*
the v2 held-out results and gcsim parity were seen. Same 40 cases, seeds,
caps, selection rule, metrics and reading tables as v2; it does not replace
v2. Conditions as in [../README.md](../README.md).

Change: a proposal may not introduce a `character action key` parameter
triple that is neither among the 174 triples used in the 4,570 main library
rotations nor already in its parent program. Reason: gcsim refused 6 v2
selections whose parameters came from another character's template.

Run: GitHub Actions run 42140341782 (commit `aa18c58`; an earlier attempt,
42140016791, failed in the build job before any search because the sparse
checkout lacked the v2.1 matrix). Arms V2S and V2T only (6 search chains per
job; v2 had 9), one job per case, 40 jobs, 16 min 25 s end to end. V1 was not
rerun. The v2.1 binary is `gso-v2` SHA-256 `f1459e89…`.

## Results (informative, n = 32)

| Arm @ CPU-s per start | Median r | Quartiles | Regressions | Reading | Median r_strip | Reading (strip) | Selected audit failures |
|---|---:|---|---:|---|---:|---|---:|
| V2S @ 192 | 0.920 | 0.65 / 0.92 / 1.05 | 0 | Promising | 0.902 | Partial (>5% regressions) | 0 |
| V2S @ 384 | 0.992 | 0.70 / 0.99 / 1.28 | 0 | Promising | 0.992 | Promising | 0 |
| V2T @ 384 | 0.964 | 0.77 / 0.96 / 1.21 | 0 | Promising | 0.964 (n=30) | Promising | 2 |

Controls: 0 regressions. Budget curve V2S (median r): 0.786 / 0.842 / 0.920 /
0.905 / 0.992 at 36 / 108 / 192 / 288 / 384; V2T 0.959 / 0.964 at 288 / 384.

| Comparison (pre-declared form) | Median d | x better | p | Reading | Secondary (selected) |
|---|---:|---:|---:|---|---|
| **V2T@384 vs V2S@384** (paired, same runner) | +0.97% | 21/32 | 0.019 | **V2T@384 beats V2S@384** | −0.20%, p = 0.84 |
| V2S@384 vs V2S@192 (unpaired fights) | +2.99% | 27/32 | 4e-5 | V2S@384 beats V2S@192 | +1.66%, p = 0.0002 |
| Descriptive: V2T@288 vs V2S@288 | +1.51% | 21/32 | 0.017 | – | −0.03%, p = 0.41 |

Descriptive cross-run comparisons (same audit fights, different runner jobs,
so search hardware may differ): v2.1 V2S@192 vs v2-run V1@192 median −1.30%
(13/32 better, p = 0.27); v2.1 vs v2 at V2S@384 +0.14% (p = 0.48) and at
V2T@384 −0.96% (p = 0.44).

gcsim parity ([gcsim-parity.json](gcsim-parity.json), same tool and settings
as v2): **no parse refusals**; one V2T selection (`varka-prune-nicole-durin`)
crashes gcsim as all arms did in v2. Where gcsim runs, it agrees with the
native audit to a median 0.1%. 0 of the 120 v2.1 selections contain a
non-library parameter triple. On the 29 informative cases where both arms
run on gcsim, median r on gcsim: V2S@384 1.000, V2T@384 0.971.

Audit failures counted as S := L: `eula-furina-shenhe-escoffier` V2T@288 and
@384 (the 10 s audit worker limit), `aratakiitto-albedo-gorou-furina` V2T@288
and @384 ("Saichi Slash into Saichi Slash").

## Reading

- The start-averaged second-pass gain replicates in this second, separately
  labeled run (+0.97%, 21/32, p = 0.019; v2: +1.21%, 26/32, p = 0.0045).
- The selected-start comparison is null in both runs (v2 +0.29%, v2.1
  −0.20%). Median r of the selected program is *lower* for V2T than V2S in
  v2.1 (0.964 vs 0.992), higher in v2 (0.999 vs 0.983).
- Removing gcsim-invalid parameters did not change the v2 picture at 384 s in
  a detectable way, but this run was designed after seeing v2's results and
  is not a clean held-out test.

## Files

[experiments.json](experiments.json), [summary.json](summary.json),
[tables.md](tables.md), [accounting.json](accounting.json),
[ops-summary.json](ops-summary.json), [gcsim-parity.json](gcsim-parity.json),
[selections.zip](selections.zip), [audits.zip](audits.zip), [inputs/](inputs)
(the v2 inputs with the v2.1 global dictionary and recomputed per-case
dictionary hashes), [matrix.json](matrix.json), [arms](arms).
