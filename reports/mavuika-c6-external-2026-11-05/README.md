# External-reference correction: all C6R5 Mavuika / Xilonen / Citlali / Nicole

Follow-up: public playback and description endpoints subsequently made the
original videos accessible. See the [replay investigation](../mavuika-c6-replay-2026-11-05/README.md)
for observed inputs, the 3.31M author's published sequence and unsuccessful
reconstruction results. The access limitation below describes this report's
earlier investigation, not the current availability of those sources.

The owner clarified that the intended comparison is with externally known
strong rotations, because the repository rotations may be weak. The previous
repository-only comparison did not fulfill that request. R1 is outside this
clarified comparison. The frozen blind R5 selection remains unchanged; there
is no new discovery, parameter tuning or post-reference optimization here.

## A reproducible external reference

A fresh scan of the public KQM gcsim database found a C6R5 entry not present in
the local party directory:
[npQhNPRFDqhn](https://db.kqm.gg/db/npQhNPRFDqhn). Its description specifies a
17–19 second rotation, hold skill, pre-burst charged attack, C1F and charged-dash
cancel/finisher repeats, with Citlali's burst every other rotation. The external
database reports approximately 1.886M DPS under its own scoring conditions.
This is a published reference, not proof of the strongest possible rotation.

Its active build statements exactly match the frozen R5 experiment build after
removing comments and whitespace. The optional Instructor build is commented
out and was not activated. The original config and build equality evidence are
retained in this directory. Only its published rotation was extracted, then
assembled through the production baseline parser in a zero-decision export.
No support order, action parameter or timing was tuned.

At identical builds, target and scoring conditions:

| Program | Native team DPS | SE |
|---|---:|---:|
| Published external gcsim rotation | **1,891,094.23** | 612.27 |
| Previously frozen blind R5 selection | 973,374.11 | 457.46 |

Both use all four characters at C6, signature R5 weapons, level 90/90, base
talents 10/10/10, the same supplied artifact stats, one level-100 target with
10% resistance, 450-second fights excluding the first 150 seconds, and 2,048
fights at common seed 12,000,000,000,000. The external rotation exceeds the blind
selection by **917,720.13 DPS**, paired SE 737.55; blind selection is **48.53%
below** this external reference.

Pinned gcsim independently measures the same assembled external config at
**1,889,903.43 DPS**, SE 604.05. Native differs by **+0.0630%**. This verifies
calculation of this particular external script, including its guarded charged
dash cancels; it does not verify every external high-score execution.
[comparison.json](comparison.json) and the raw native/gcsim files preserve the
measurements. Native trace diagnostics rerun fights; its standalone CPU timing
was not captured. The gcsim command/timing and the separately charged zero-decision
export are retained. These are additional reference checks, not discovery CPU.

## Public records above 3M are real comparison leads, not reproduced outputs

External records for the requested four-member party were found:

- [Lin-Eric: 3.23M DPS](https://www.bilibili.com/video/BV1jaK36kE7s/).
- [3.0817M public submission](https://www.bilibili.com/video/BV1qKEp6GE9q/).
- [3.129272M public submission](https://www.bilibili.com/video/BV15f4g6oEhV/).
- [3.314730M public submission](https://www.bilibili.com/video/BV1nhsd6dEvM/).

The last three are listed by [the public DPS database](https://www.mhydps.cn/)
with all four members C6 and declared cost 48. Relevant API rows are preserved
in [public-records.json](public-records.json), including the original `clean`,
`lvl`, `tags` and confirmation fields. These metadata do not by themselves
verify each weapon, artifact, level upgrade or execution condition. The
3.314730M entry is explicitly tagged as a macro submission. The frontend labels
`lvl` as including Stella Fortuna; those conditions were not imported into the
level-90 frozen search build.

Original Bilibili pages/player metadata returned access errors (412) through
the available readers. Publicly indexed descriptions establish the 3.23M video
and provide links, but do not supply an exact frame/action log and complete
equipment/measurement setup sufficient for faithful replay. No 3M video was
converted into a verified simulator config in this follow-up. No cross-condition
percentage comparison against those displayed high scores is claimed.

The previous 1.744M repository-cycle measurement cannot refute the owner's
expectation of approximately 3M DPS. It only describes that particular stored
cycle under its declared build and scoring conditions.

## Current pilot capability

The engine executes the published external gcsim script and agrees with pinned
gcsim. The generated search grammar cannot express its action-parameter maps,
charged-dash-cancel helper, arbitrary state/resource guards, targeted waits or
crystallize-pickup builtin. The existing blind 973K result is consequently not a
successful rediscovery of this external play. Full discovery and faithful replay
of the public 3M execution remain unestablished.

Earlier blind selections were frozen before these external references were
read. The current generator was not changed or rerun after revelation. The
original six-run study remains in
[the earlier report](../mavuika-c6-blind-2026-11-05/README.md), now marked with the
comparator-scope correction. Original source/config snapshots and file hashes
are retained here; no changes were made to the other repositories.
