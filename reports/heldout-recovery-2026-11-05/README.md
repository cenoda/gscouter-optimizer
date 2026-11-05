# Held-out recovery pilot, stage 1 (2026-11-05)

**Measured result, stage 1 only.** On 24 informative held-out cases at 192
cumulative discovery CPU-seconds per start, the median recovery is
**r = 0.94** (quartiles 0.55 / 0.94 / 1.27) with **0 regressions**; on the 6
controls there are also 0 regressions. Applied mechanically, the pre-declared
reading is **"Promising: worth the full stage"**. Read the caveats below
before using that sentence: r is a ratio to a community rotation that is not
optimal, several cases are "L breaks on the higher-constellation build", the
variant cases are easy by construction, and nothing here is checked against
gcsim.

Protocol: [`docs/heldout-recovery-pilot.md`](../../docs/heldout-recovery-pilot.md),
SHA-256 `14bea06330201a9cc603c61e03e56a67a205428ed39bb586343a5913c50ebacd`
at commit `7f1809a` (the version every run used; see [protocol.json](protocol.json)).
A dated Deviations section was appended to that document after the runs; it
changes the file hash and therefore the source identity of later builds.

## Conditions (apply to every number below unless stated)

Native engine `cenoda/gscouter-engine@1ba0af1` through this repository's
evaluator. 450 s fight, first 150 s excluded, one level-100 target, 10%
resistance, radius 2, swap delay 12, periodic energy every 480–720 frames
(the default `HEADER`, the Flins pilot's). Build B = the alternative's
`alt/<id>/build.txt` at Gscouter `6e0e82205`, with the alternative's own
`team.yaml` params (see Deviations), levels and constellations as written.
Fixed ER, no artifact moves. Team DPS, mean over fights.

In the per-case tables, L, A and S in one row are audited on **the same 128
fresh fights**: the audit seeds of the selected start's checkpoint
(`AUDIT_SEED + seed·10⁶ + cap·10⁶`). L is the search's own baseline audit; A
was measured afterwards on exactly those seeds (`confirm`), and the same
measurement path re-run on L reproduced the search's L fight-for-fight for
every start and checkpoint (asserted). S is the audited score of the
pre-audit selection (`provisional_best_key`, frozen in `shortlist.json` and
`selected-<cap>.json` before the audit) of the start with the highest
discovery estimate (`best_hint`), per the protocol's rule. r = (S−L)/(A−L).
Regression = S−L below −2 paired SE. Different rows and different
checkpoints use different audit seeds; comparisons across checkpoints (budget
gains) therefore mix two independent 128-fight samples.

## What was run

| Step | What | CPU |
|---|---|---:|
| Export | 247 alternatives (207 database entries, 40 `citlali-c6-no-burst` variants) → fixed-build manifests; all 247 exported | – |
| Dictionary | 2,755 blocks for 129 characters from the 4,570 main `teams/*/rotation.gcsl` (never `alt/`); per-case leakage filter dropped a median 7 blocks (max 17) | – |
| Classification | L and A on B, 128 paired fights, seed 8,000,000,000,000: **174 informative, 60 control, 13 excluded** | 326 s |
| Draw | seed 20261105, stratified by `hold=`/`while` in A: 24 informative (12 + 12), 6 controls (3 + 3), 6 informative continued to 576 s. Committed and pushed as `9d5ce2e` before any held-out search | – |
| Search | 30 cases × 3 starts (seeds 11, 37, 91) × checkpoints 36/108/192; 6 cases × 3 starts continued to 576 | 24,337 s discovery + 1,126 s audits |
| Confirm | A (and L, as a check) on every checkpoint's audit seeds | 390 s |
| Development | Flins C1 and Mavuika C6R5-225 only, seed 11, to 192 s (before freezing) | 384 s |

Total measured process CPU ≈ **7.4 CPU-hours** (plan: ≈ 8); wall ≈ 2 h 10 min
for the searches on 4 cores. Details in [accounting.json](accounting.json).
No case was dropped for cost; the full drawn list ran.

Exclusions (listed in [classification.json](classification.json)): 2 Chasca C6
cases where L stops with "bullets must be 6 when c6 instant charge is
active"; 1 Mualani/Traveler case where L names `aetherpyro`, not in the build;
10 collapses below 5,000 DPS (8 with L collapsing, 2 with A).

## Search, as implemented (`gso search --dictionary`, commit `61f429f`)

Greedy, fixed ER, one chain starting from L (L is the incumbent and stays in
the audited shortlist). Each proposal is one of: replace a character statement
with a dictionary block for that character (40%), insert a block for a random
member before a statement (35%), delete a statement (13%), move a statement
past the next (12%) — or, with probability 25% per draw, one of the existing
generic action-list neighbors (deletion, normal-count change, relocation,
duplication). After 40 consecutive non-improving decisions a restart perturbs
the promoted peak with 3 random macro edits and installs it unconditionally.
In the last quarter of each cumulative cap, micro coordinate search walks the
peak's numeric literals (repeat counts, action parameters, `wait`/`delay`
arguments; ±1 and ±25%) in order; when that neighborhood is exhausted without
a promotion it returns to macro edits. Every decision is the existing common
16-fight block (accept if Δ ≥ 0), every peak change the existing 32-fight
paired promotion. No character-specific rule exists anywhere; the dictionary
is mechanical extraction (statements and single-owner `if`/`while` blocks of
≤ 40 tokens appearing in ≥ 2 library rotations, top 64 per character by
count).

Leakage: each search ran under `strace` (open/openat/execve). Across all 288
invocations it opened only its case's `manifest.json`, `L.gcsl` and
`dictionary.json`, its own run directory, the binary, libc/ld files and
`/proc/self/maps`; no path under `alt/`, `withheld/` or the Gscouter checkout,
and no other case's files ([selections.zip](selections.zip), `opened-*.json`).
Input hashes are in each run's `experiment.json`. A was read only by the
leakage filter, the classification and `confirm`.

Invariants: `check_run.py` passes for all 90 runs (137,671 verified common
16-fight decisions, 48,579 accepted including Δ = 0); `check_peaks.py`
reconstructs all 90 final peaks from 3,216 complete 32-fight promotion blocks
(1,107 peak changes, 12 unresolved promotions that do not change the peak).
5,690 proposals failed (engine action errors such as "cannot recast E", or the
10 s worker wall limit) and are charged to the budget.

Operator outcomes over all runs (strict improvements / completed decisions):
dictionary replace 2,925 / 37,672; dictionary insert 2,295 / 35,678; generic
neighbors 3,542 / 33,482; delete statement 946 / 13,492; move statement
1,188 / 11,068; restarts 149 / 852; micro repeat counts 343 / 3,959, action
parameters 75 / 855, wait/delay 137 / 523.

## Results

### Pre-declared reading (informative cases)

| Checkpoint (CPU-s per start) | Cases | Informative | Median r (informative) | r quartiles | r ≥ 0.5 | Median r, frame / other | Informative regressions | S = L |
|---|---:|---:|---:|---|---:|---|---:|---:|
| 36 | 30 | 24 | 0.736 | 0.45, 0.74, 1.13 | 67% | 0.661 (n=12) / 0.843 (n=12) | 0 | 0% |
| 108 | 30 | 24 | 0.946 | 0.49, 0.95, 1.21 | 75% | 0.958 (n=12) / 0.946 (n=12) | 0 | 0% |
| 192 | 30 | 24 | 0.940 | 0.55, 0.94, 1.27 | 79% | 0.993 (n=12) / 0.940 (n=12) | 0 | 0% |
| 576 | 6 | 6 | 0.782 | 0.58, 0.78, 1.17 | 100% | 0.615 (n=3) / 0.948 (n=3) | 0 | 0% |

At **192 s**: median r = 0.940, informative regressions 0/24 (≤ 5%) →
**"Promising: worth the full stage"**, mechanically. Controls: 0/6
regressions; in all 6, S > L (their r values are not meaningful because
A − L is small or negative).

### Per case at 192 CPU-s per start

| # | Case | Class | Stratum | L | A | A−L | S (selected) | r | Sel. start | S per start (11 / 37 / 91) | Start spread | Regression | CPU per start | A features | S features |
|---:|---|---|---|---:|---:|---:|---:|---:|---:|---|---:|---|---|---|---|
| 0 | wriothesley-shenhe-furina-xilonen--mQBdNDhMnnH6 | informative | frame | 21,138 | 339,454 | +1505.9% | 295,543 | 0.86 | 37 | 273,892 / 295,543 / 234,657 | +288.0% | no (274,405 ± 199) | 192, 192, 198 | hold | hold |
| 1 | travelerpyro-furina-bennett-citlali--citlali-c6-no-burst | informative | frame | 95,522 | 103,416 | +8.3% | 104,401 | 1.12 | 37 | 104,712 / 104,401 / 105,263 | +0.9% | no (8,878 ± 317) | 192, 196, 192 | hold,while | hold,while |
| 2 | gaming-citlali-kaedeharakazuha-bennett--citlali-c6-no-burst | control | other | 124,533 | 130,096 | +4.5% | 137,242 | 2.28 | 11 | 137,242 / 129,573 / 133,280 | +6.2% | no (12,709 ± 687) | 192, 192, 192 | wait_or_delay | wait_or_delay |
| 3 | shenhe-chasca-citlali-mavuika--citlali-c6-no-burst | informative | frame | 112,807 | 124,696 | +10.5% | 128,245 | 1.30 | 37 | 119,788 / 128,245 / 119,282 | +7.9% | no (15,439 ± 290) | 192, 192, 192 | hold,wait_or_delay | hold,wait_or_delay |
| 4 | cyno-fischl-nahida-xilonen--kRfdGgkcpznD | informative | frame | 173,950 | 206,982 | +19.0% | 216,524 | 1.29 | 91 | 194,118 / 184,694 / 216,524 | +18.3% | no (42,574 ± 198) | 197, 200, 198 | while | while |
| 5 | xiao-faruzan-furina-xianyun--n8fCP7cBnnWz | informative | frame | 68,393 | 75,330 | +10.1% | 76,839 | 1.22 | 37 | 71,598 / 76,839 / 74,185 | +7.7% | no (8,446 ± 298) | 201, 197, 201 | while | while |
| 6 | kamisatoayaka-furina-citlali-escoffier--RmDNLghj8jqh | informative | other | 112,505 | 434,692 | +286.4% | 416,072 | 0.94 | 91 | 410,725 / 340,314 / 416,072 | +67.3% | no (303,567 ± 1,879) | 192, 192, 192 | none | while |
| 7 | wriothesley-xiangling-furina-bennett--mwjCWWCcpwJJ | informative | other | 164,779 | 226,454 | +37.4% | 229,031 | 1.04 | 11 | 229,031 / 209,316 / 191,881 | +22.5% | no (64,252 ± 356) | 201, 194, 192 | none | while |
| 8 | neuvillette-furina-lanyan-xilonen--dB9DNqJgzHBr | informative | other | 374,547 | 435,126 | +16.2% | 454,794 | 1.32 | 11 | 454,794 / 420,480 / 385,704 | +18.4% | no (80,247 ± 532) | 196, 192, 192 | wait_or_delay | wait_or_delay |
| 9 | noelle-xilonen-nicole-durin--99dNHg8bKpcw | control | other | 619,290 | 620,400 | +0.2% | 683,814 | 58.15 | 91 | 660,463 / 669,175 / 683,814 | +3.8% | no (64,524 ± 871) | 192, 192, 192 | none | none |
| 10 | kaedeharakazuha-faruzan-furina-xianyun--JFrLR9QLQgdw | informative | frame | 96,951 | 151,845 | +56.6% | 103,032 | 0.11 | 11 | 103,032 / 97,701 / 101,847 | +5.5% | no (6,082 ± 440) | 192, 197, 192 | while,wait_or_delay | none |
| 11 | chongyun-furina-escoffier-xianyun--KPmGTPNQRDW6 | informative | frame | 46,669 | 49,928 | +7.0% | 48,531 | 0.57 | 91 | 48,306 / 46,601 / 48,531 | +4.1% | no (1,862 ± 75) | 196, 202, 200 | while | while |
| 12 | nahida-fischl-beidou-kaedeharakazuha--bcR6QJHLLBtH | control | frame | 61,871 | 57,078 | -7.7% | 89,292 | -5.72 | 91 | 87,612 / 81,644 / 89,292 | +12.4% | no (27,421 ± 63) | 192, 192, 192 | hold,while,wait_or_delay | none |
| 13 | chasca-citlali-chongyun-bennett--citlali-c6-no-burst | informative | other | 78,611 | 83,086 | +5.7% | 90,181 | 2.59 | 37 | 86,509 / 90,181 / 84,933 | +6.7% | no (11,570 ± 297) | 192, 194, 192 | wait_or_delay | while,wait_or_delay |
| 14 | bennett-fischl-lisa-xingqiu--z89bwQG6KGHg | informative | frame | 43,079 | 51,335 | +19.2% | 52,471 | 1.14 | 11 | 52,471 / 47,672 / 52,033 | +11.1% | no (9,392 ± 102) | 192, 196, 192 | hold | hold |
| 15 | mavuika-citlali-charlotte-bennett--citlali-c6-no-burst | informative | frame | 114,847 | 143,490 | +24.9% | 176,213 | 2.14 | 37 | 166,395 / 176,213 / 162,414 | +12.0% | no (61,366 ± 88) | 192, 192, 192 | hold | hold |
| 16 | mualani-chiori-xiangling-xilonen--fKzfLbrbfhj9 | informative | frame | 131,345 | 157,775 | +20.1% | 141,761 | 0.39 | 11 | 141,761 / 140,483 / 138,414 | +2.5% | no (10,416 ± 386) | 197, 198, 193 | while,wait_or_delay | while |
| 17 | mavuika-citlali-xilonen-albedo--citlali-c6-no-burst | control | frame | 136,167 | 138,569 | +1.8% | 143,656 | 3.12 | 11 | 143,656 / 142,672 / 140,533 | +2.3% | no (7,489 ± 420) | 192, 192, 195 | hold,wait_or_delay | hold,wait_or_delay |
| 18 | flins-ineffa-furina-sucrose--j86qrckHDPmb | informative | other | 131,225 | 149,183 | +13.7% | 142,001 | 0.60 | 37 | 134,631 / 142,001 / 139,556 | +5.6% | no (10,776 ± 141) | 193, 192, 192 | none | while |
| 19 | neuvillette-furina-lynette-xilonen--JhJ7FPddfHRz | informative | other | 31,827 | 420,991 | +1222.8% | 414,799 | 0.98 | 91 | 307,869 / 353,559 / 414,799 | +336.0% | no (382,972 ± 199) | 192, 198, 192 | wait_or_delay | wait_or_delay |
| 20 | chasca-citlali-mualani-bennett--citlali-c6-no-burst | control | frame | 95,352 | 99,922 | +4.8% | 101,115 | 1.26 | 91 | 100,641 / 99,856 / 101,115 | +1.3% | no (5,763 ± 327) | 192, 194, 196 | while,wait_or_delay | while,wait_or_delay |
| 21 | citlali-chasca-escoffier-mavuika--citlali-c6-no-burst | control | other | 132,001 | 137,925 | +4.5% | 143,073 | 1.87 | 37 | 140,807 / 143,073 / 143,398 | +2.0% | no (11,072 ± 301) | 192, 192, 192 | wait_or_delay | wait_or_delay |
| 22 | hutao-furina-yelan-citlali--m9CWdQGTJwCH | informative | other | 271,366 | 505,402 | +86.2% | 443,734 | 0.74 | 91 | 439,980 / 346,756 / 443,734 | +35.7% | no (172,368 ± 653) | 192, 192, 192 | none | none |
| 23 | bennett-lynette-xiangling-lyney--jNtRRhR8NM9m | informative | other | 45,229 | 63,502 | +40.4% | 56,600 | 0.62 | 91 | – / 55,997 / 56,600 | +1.3% | no (11,371 ± 116) | 197, 200, 192 | wait_or_delay | wait_or_delay |
| 24 | chasca-citlali-rosaria-bennett--citlali-c6-no-burst | informative | other | 74,803 | 78,847 | +5.4% | 82,273 | 1.85 | 91 | 80,875 / 82,248 / 82,273 | +1.9% | no (7,470 ± 291) | 201, 192, 192 | none | none |
| 25 | clorinde-kujousara-nahida-xilonen--DQkpHtnCkKgR | informative | frame | 69,222 | 75,795 | +9.5% | 72,399 | 0.48 | 91 | 71,434 / 71,202 / 72,399 | +1.7% | no (3,176 ± 42) | 202, 192, 199 | while,wait_or_delay | while,wait_or_delay |
| 26 | hutao-yelan-mona-nicole--NpWw8TkzPPDz | informative | other | 126,117 | 146,430 | +16.1% | 145,178 | 0.94 | 91 | 140,103 / 141,646 / 145,178 | +4.0% | no (19,061 ± 119) | 196, 196, 192 | wait_or_delay | none |
| 27 | hutao-yelan-columbina-ineffa--7DdP6dDqRGbC | informative | other | 111,409 | 119,605 | +7.4% | 113,604 | 0.27 | 11 | 113,604 / 111,833 / 111,793 | +1.6% | no (2,194 ± 201) | 192, 192, 192 | none | none |
| 28 | mavuika-citlali-xilonen-chevreuse--dkC8Gtdm8F9d | informative | frame | 115,361 | 1,025,563 | +789.0% | 615,239 | 0.55 | 91 | 547,980 / 570,861 / 615,239 | +58.3% | no (499,879 ± 1,699) | 192, 192, 192 | hold,wait_or_delay | hold,wait_or_delay |
| 29 | flins-ineffa-columbina-durin--kwnMBjKmcJWQ | informative | other | 145,453 | 166,698 | +14.6% | 150,872 | 0.26 | 37 | 150,980 / 150,872 / 150,904 | +0.1% | no (5,419 ± 281) | 192, 192, 192 | none | none |

Start 11 of `bennett-lynette-xiangling-lyney` has no audited S at 192 s: its
selection's 128-fight audit hit the 10 s worker wall limit ("failed/unresolved
audit"). The selection rule chose start 91 for that case, so the case's S is
defined; that start's own value is reported as missing, not filled in.

### Continuation to 576 CPU-s per start (declared subset)

| # | Case | Class | Stratum | L | A | A−L | S (selected) | r | Sel. start | S per start (11 / 37 / 91) | Start spread | Regression | CPU per start | A features | S features |
|---:|---|---|---|---:|---:|---:|---:|---:|---:|---|---:|---|---|---|---|
| 3 | shenhe-chasca-citlali-mavuika--citlali-c6-no-burst | informative | frame | 112,426 | 124,725 | +10.9% | 128,515 | 1.31 | 37 | 119,386 / 128,515 / 119,806 | +8.1% | no (16,088 ± 261) | 576, 576, 576 | hold,wait_or_delay | hold,wait_or_delay |
| 7 | wriothesley-xiangling-furina-bennett--mwjCWWCcpwJJ | informative | other | 164,877 | 225,564 | +36.8% | 232,746 | 1.12 | 11 | 232,746 / 215,926 / 217,050 | +10.2% | no (67,868 ± 462) | 578, 576, 576 | none | while |
| 11 | chongyun-furina-escoffier-xianyun--KPmGTPNQRDW6 | informative | frame | 46,605 | 49,914 | +7.1% | 48,641 | 0.62 | 91 | 48,275 / 47,007 / 48,641 | +3.5% | no (2,036 ± 90) | 576, 576, 579 | while | while |
| 18 | flins-ineffa-furina-sucrose--j86qrckHDPmb | informative | other | 131,352 | 149,428 | +13.8% | 141,962 | 0.59 | 37 | 135,014 / 141,962 / 139,336 | +5.3% | no (10,610 ± 165) | 576, 576, 578 | none | while |
| 25 | clorinde-kujousara-nahida-xilonen--DQkpHtnCkKgR | informative | frame | 69,197 | 75,864 | +9.6% | 73,027 | 0.57 | 91 | 71,378 / 72,248 / 73,027 | +2.4% | no (3,830 ± 42) | 577, 584, 579 | while,wait_or_delay | while,wait_or_delay |
| 26 | hutao-yelan-mona-nicole--NpWw8TkzPPDz | informative | other | 126,062 | 146,306 | +16.1% | 145,254 | 0.95 | 91 | 140,048 / 141,581 / 145,254 | +4.1% | no (19,192 ± 128) | 584, 576, 578 | wait_or_delay | none |

### Q2: budget

Median relative change of the selected S (audited, different seeds per
checkpoint): 36→108 +0.17% (19/30 cases under +1%), 108→192 +0.15% (23/30
under +1%), 192→576 +0.22% on the 6 continued cases (5/6 under +1% =
"converged" by the protocol's definition; `wriothesley-xiangling-furina-bennett`
+1.6% is not). Median r of the 6 continued cases: 0.769 at 192 s, 0.782 at
576 s. Most of what this search finds, it finds by 36–108 s; the extra budget
to 576 s changed little in this subset.

### Q3: which cases do not converge or recover

Informative cases with r < 0.5 at 192 s: `kaedeharakazuha-faruzan-furina-xianyun`
(0.11), `flins-ineffa-columbina-durin` (0.26), `hutao-yelan-columbina-ineffa`
(0.27), `mualani-chiori-xiangling-xilonen` (0.39), `clorinde-kujousara-nahida-xilonen`
(0.48): 3 of 12 frame-stratum and 2 of 12 other-stratum cases. Median r by
stratum at 192 s: frame 0.993, other 0.940. With n = 12 each, this does not
show that frame-level cases are harder; it also does not show they are not.
Of these five, four have start spreads ≤ 2.5% and 192→576 data for one
(clorinde: 0.48 → 0.57); low r here looks like a local optimum of this
neighborhood rather than lack of budget, but that is a hypothesis.

## What this does and does not show

- **Shows (measured):** under the common conditions above, on a seeded
  stratified draw of 24 informative held-out cases, starting from L and
  using only library-derived blocks, the frozen selection recovers a median
  94% of the known A − L gap at 192 CPU-s per start, with no case audited
  significantly below L, and exceeds A in 10/24 cases.
- **r is relative to A, which is not optimal.** A is a community rotation
  written for gcsim and the source's own conditions; S > A (r > 1) means only
  that S scores higher natively under these conditions.
- **Easy and broken cases inflate the headline.** 5 of 24 informative cases
  are `citlali-c6-no-burst` variants (A = L minus Citlali's bursts; median r
  1.85 for them, 0.74 for the 19 database cases). 8 cases have A − L > 25%,
  several where L breaks on B (Wriothesley/Shenhe L 21k vs A 339k; Neuvillette/
  Lynette L 32k vs A 421k); their median r is 0.80, against 1.13 for the 16
  with A − L ≤ 25%. These breakdowns are descriptive and were not used for
  the reading.
- **No gcsim parity.** No gcsim binary was available on this machine. S
  programs contain edits such as `charlotte skill[hold=3]` (micro search
  changed `hold=1`) or repeated `jump`/`charge[hold=6]` blocks. The native
  engine accepts them; whether gcsim or the game would score them the same is
  not checked. Earlier native/gcsim parity (0.06% per major contributor)
  covered other programs.
- **Bloat.** Δ = 0 decisions are accepted (existing rule), so dead code
  accumulates; e.g. the Flins development run grew from 103 to 293 words. S
  programs are not minimized; diffs against L are in
  [experiments.json](experiments.json) (`diff_against_L`).
- **Worker wall limit under load.** Four searches ran in parallel under
  `strace`; slow candidates hit the existing 10 s per-batch wall limit and
  were charged as failures. One selected audit was lost this way (above).
- **Stage 1 is small** (24 + 6 cases, 3 starts). It decides whether stage 2
  is worth running; it is not evidence for the main batch, for annealing, for
  other builds or for C0 profiles.

## Deviations and clarifications

1. **Params.** The protocol does not say which `team.yaml` params apply.
   Gscouter attaches params per rotation; for 21 of 247 cases the library
   team's and the alternative's params differ. To keep one common condition
   for L, A and the search, B = alternative's build + alternative's params for
   all three. Decided before any measurement (first export run excluded those
   21; the export was redone before classification).
2. **Variants included.** The protocol's universe is `teams/*/alt/*` (247 over
   231 teams, which counts the 40 variants), while its text calls each
   alternative "a community database entry". The 40 variants were kept, as the
   stated count implies; they are tagged `kind: variant` everywhere.
3. **Constellation grid not applied.** The existing discovery-profile check
   (four-star C6, five-star C1/2/4/6) is skipped in `--dictionary` mode,
   since the protocol uses each build as is.
4. **Restarts and micro schedule** are design choices the protocol leaves
   open: perturbation restarts from the peak (patience 40, 3 edits); micro in
   the existing last-quarter polish window of each cumulative cap, falling
   back to macro edits when exhausted.
5. **Reading table gap.** The table has no row for median r ≥ 0.5 with > 5%
   regressions. Before any r existed, the report script (commit `2a7e8ba`)
   declared that cell "uncovered, read as Partial". It did not occur.
6. **Worker batches split at 32 fights** (same seeds and results) so 128-fight
   audits fit the existing 10 s wall limit.
7. **After the runs (measurement/report only, no search change):** `confirm`
   was re-run once with all evaluations cached to fix how S features were
   extracted (the repetition wrapper had been counted as `while`); values are
   identical. The `measure` command's printed CPU included earlier journal
   records when a directory was reused; fixed in `heldout.rs` after the run,
   and confirm CPU above is recomputed from the journals. This fix changes the
   implementation hash: the runs' source is `61f429f`
   (implementation SHA-256 `2e868739…`, recorded in every report).
8. **Commit message error.** The draw commit `9d5ce2e` says "186 database
   entries, 40 variants"; the correct split is 207 + 40 (the 186 came from the
   discarded first export). The committed files are correct.
9. **Development cases.** Flins was debugged on `data/flins-c1/standard.json`
   (team `flins-ineffa-columbina-sucrose`) and Mavuika on
   `data/mavuika-c6-r5-225` (120 s, other conditions). Two drawn cases are
   other Flins teams (`flins-ineffa-furina-sucrose`, `flins-ineffa-columbina-durin`);
   the development runs changed no code or constant.

## Files

- [draw.json](draw.json), [cases.json](cases.json), [classification.json](classification.json),
  [dictionary.json](dictionary.json), [protocol.json](protocol.json): frozen before the search.
- [inputs/](inputs): manifests, L and B for the 30 drawn cases; per-case dictionary = global minus
  `dictionary-filter.json` (`gso dictionary filter` reproduces it).
- [experiments.json](experiments.json): every case, start and checkpoint (L, A, S, r, SE, CPU,
  features, diff of S against L); [run-summary.json](run-summary.json); [summary.json](summary.json);
  [tables.md](tables.md) (all checkpoints); [accounting.json](accounting.json).
- [selections.zip](selections.zip): per start and checkpoint, the report, frozen shortlist,
  selected program and the strace list of opened files.
- Reproduce: `python3 scripts/heldout.py export|dictionary|measure|draw|run|confirm` then
  `python3 scripts/heldout_report.py` (with a Gscouter checkout at `6e0e82205`).
