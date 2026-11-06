# Cases furthest below A

**Status: post-hoc listing, not a result.** Written 2026-11-06 after the
held-out run was read. Computed from `experiments.json` by
`worst_cases.py`; no new measurement.

Informative cases (n = 52), selected program per arm at 384 CPU-s (per-start
32-fight selection block; audit failure counts as L), DPS divided by A on the
same 128-fight audit. Sorted by the best of the three arms. "L" is L / A.

| # | Case | A/L | L | V21S | SYN | XO | Best |
|---|---|---:|---:|---:|---:|---:|---:|
| 1 | xianyun-furina-xiangling-bennett | 2.20 | 46% | 60.6% | 63.1% | 60.7% | 63.1% |
| 2 | bennett-xiangling-furina-kaedeharakazuha | 2.07 | 48% | 62.7% | 63.5% | 68.3% | 68.3% |
| 3 | kaedeharakazuha-furina-bennett-xianyun | 16.22 | 6% | 69.9% | 62.5% | 68.0% | 69.9% |
| 4 | mualani-sucrose-travelerpyro-mona | 3.17 | 32% | 70.0% | 68.5% | 61.6% | 70.0% |
| 5 | kaedeharakazuha-furina-nahida-bennett | 2.87 | 35% | 71.4% | 75.5% | 79.1% | 79.1% |
| 6 | mualani-xilonen-sucrose-mavuika | 1.51 | 66% | 78.1% | 79.4% | 82.1% | 82.1% |
| 7 | hutao-xingqiu-yelan-fischl | 1.68 | 59% | 77.9% | 82.2% | 79.4% | 82.2% |
| 8 | mavuika-chasca-citlali-bennett | 2.41 | 41% | 65.8% | 90.4% | 76.3% | 90.4% |
| 9 | sangonomiyakokomi-nilou-collei-kirara | 1.15 | 87% | 86.7% | 91.9% | 90.0% | 91.9% |
| 10 | mualani-mona-sucrose-mavuika | 1.30 | 76% | 92.5% | 81.4% | 85.8% | 92.5% |

Observations (hypotheses, not diagnosed; no rotation has been compared yet):

- The top five stall near 60–80% A in every arm, so neither transplant nor
  skeleton families reached A's structure there.
- Four of the top five are Bennett/Furina teams with Kazuha or Xianyun, a
  pattern possibly tied to member-state plunge loops (compare the
  out-of-scope Kazuha C6 case in stage 1). Three of the ten are Mualani teams,
  which also failed in stage 1 and v2.
- Competition can lose to V21S on a case (#10: SYN 81.4% vs V21S 92.5%;
  #4: XO 61.6% vs 70.0%).

Next step, if requested: per-character DPS and single-edit diagnostics as in
`reports/heldout-recovery-2026-11-05/failure-analysis/`. These cases are now
development cases.
