# Mavuika / Xilonen / Citlali / Nicole C6: blind discovery result

**Comparator scope correction:** the owner subsequently clarified that the
requested comparator is an externally known strong rotation at all C6R5,
because the repository rotations are themselves suspected to be weak. The
repository-only comparisons below do not fulfill that external benchmark and
must not be used to dismiss the owner's approximate 3,000,000 DPS expectation.
The blind selections and their measurements remain unchanged.
See the [external-reference follow-up](../mavuika-c6-external-2026-11-05/README.md)
for a new published gcsim comparison and unreproduced public records above 3M.

**The current engine can calculate this party, but the pilot failed to discover
a competitive rotation in this test.** Its independently confirmed selections
are 40.84% below the saved C6 reference at signature R1 and 44.18% below it at
signature R5. Native/gcsim agreement is good; this is evidence of inadequate
discovery, not an observed simulator discrepancy.

The owner's approximate 3,000,000 DPS expectation was not used as a search
constant, pass threshold or stopping rule. None of the measured programs reaches
that value under the conditions below. This does not establish an upper bound
for other cycles, builds, targets or scoring horizons.

## Identical-build independent confirmation

All four members are C6, level 90/90, base talents 10/10/10. The fixed artifact
stats and sets come from Gscouter commit
`d8d28c36d3801a4383feb240e78582fc674a00f1`,
`teams/mavuika-citlali-xilonen-nicole/alt/LhrRtbWNJk8t/build.txt`.
The source uses signature R5 weapons. A separate condition changes only all
four weapon refinements to R1. These supplied builds are not certified physical
standard envelopes or the owner's actual equipment.

The manifest's order is Mavuika / Citlali / Xilonen / Nicole. Weapons are
A Thousand Blazing Suns / Starcaller's Watch / Peak Patrol Song /
Angelos Heptades. Original build declarations and manifests are in
`data/mavuika-c6-blind` and the archives.

Every confirmation uses **2,048 full fights**, fresh common seed
12,000,000,000,000, duration **450 seconds**, warmup **150 seconds**, one level-100
target, resistance 10%, radius 2, position (0, 2.4), swap delay 12 and the
pilot's existing periodic energy. All programs within each column have identical
builds and evaluation conditions.

| Program | All signatures R1, team DPS | SE | All signatures R5, team DPS | SE |
|---|---:|---:|---:|---:|
| Original library cycle, evaluated at C6 | 909,502.44 | 1,408.01 | 1,286,150.35 | 2,083.30 |
| Saved C6 library alternative | **1,227,587.40** | 377.09 | **1,743,720.50** | 565.51 |
| Frozen blind-discovery selection | 726,184.91 | 292.84 | 973,374.11 | 457.46 |

Against the C6 reference, blind selection loses **501,402.49 DPS (40.84%)**
at R1 and **770,346.40 DPS (44.18%)** at R5. Paired SEs are 461.57 and
697.19 DPS respectively. It also loses to the original library cycle applied
unchanged at C6: 20.16% at R1 and 24.32% at R5. These gaps are much larger than
the confirmation sampling errors. The references are known cycles, not certified
optimal play. [confirmation.json](confirmation.json) contains all results and
paired comparisons; raw per-fight values, per-character damage, abilities and
wait diagnostics are in `confirmation/`.

## Blinding, budgets and all starts

The [protocol](protocol.md) was written before the matrix. No optimizer changes
were made for this party. Only build declarations and engine mechanics were
inspected before discovery. Both reference rotations were copied as bytes and
hashed without displaying, parsing or measuring their action sequences. All six
blind runs finished and selections were written before either reference was
exported through the production baseline parser and measured.

The frozen optimizer fingerprint is
`79123f73118bf79daf5e723273ce316b691c02981f9cad58692311bf5fb76da4`.
This is the same implementation used for the preceding generic execution smoke
checks. Each run uses `--from-scratch --fixed-er --algorithm greedy`, starts
11/37/91, cumulative caps 4/12/36/108 CPU seconds and separate 128-fight audits.
No known cycle, prior winner, rotation-derived parameter or target DPS enters
generation. Selection is based on final **discovery estimates**, not audit means.

Audited DPS of the pre-audit selected program at every budget:

| Weapon condition | Start | 4 CPU-s | 12 CPU-s | 36 CPU-s | 108 CPU-s cap |
|---|---:|---:|---:|---:|---:|
| R1 | 11 | 289,494 | 413,331 | 531,170 | 538,587 |
| R1 | 37 | 306,153 | 321,448 | 592,761 | 724,743 |
| R1 | 91 | 235,269 | 390,517 | 544,101 | 690,379 |
| R5 | 11 | 383,691 | 477,833 | 666,587 | 929,252 |
| R5 | 37 | 405,969 | 429,620 | 684,835 | 694,630 |
| R5 | 91 | 299,605 | 537,246 | 922,486 | 972,063 |

At the final cap, medians are 690,379 (R1) and 929,252 (R5). The selected runs
are R1/start 37 and R5/start 91. Their discovery estimates are 726,810 and
972,448. Fresh confirmation did not influence those choices. The complete
matrix, including all shortlist audits, failures, actual CPU and pre-polish
selections, is [matrix.json](matrix.json). Selection identity and source hashes
are in [selection.json](selection.json); frozen inputs and withheld hashes are
in [frozen.json](frozen.json).

R5/start 37 exhausts the bounded final polish neighborhood at **91.60 CPU
seconds**, below the requested 108-second cap. This is not convergence or an
optimality certificate. The other five final runs spend approximately 108 CPU
seconds each. Maximum last-operation cap overrun is 0.1964 CPU seconds. No seeds
were replaced and no extra search was performed after reference revelation.

## What the current generator cannot express

The revealed C6 reference explicitly uses Mavuika charged attacks with
`hold=6` and `final=1`, plus `pick_up_crystallize("any")`. The generated grammar
has no action-parameter maps or crystallize-pickup builtin. Therefore the exact
reference program is outside the current search space. General default normal,
charged, skill, burst, dash and jump actions are supported, but that is not
complete coverage of this party's important controls.

This identifies a concrete representational limitation. It does not isolate
how much of the DPS gap comes from those missing controls versus support order,
field allocation, timing or search depth. No hand-authored replacement, parameter
injection, post-revelation tuning or extra-budget continuation is substituted
for the measured blind results. This party is now a revealed development case
for any later generator work.

## Accuracy and evidence checks

Pinned native engine: `1ba0af119e0477d6f44ee966a78fa06d84d2c5de`.
Pinned gcsim reference: `cenoda/gcsim@eb218847f82b`, with the actual reference
binary SHA-256 retained in `frozen.json`.

| Program | Native vs gcsim team difference |
|---|---:|
| Blind R1 selection | -0.0086% |
| C6 reference, R1 | +0.0172% |
| Blind R5 selection | -0.0942% |
| C6 reference, R5 | +0.0196% |

Each comparison uses 2,048 fights in each engine. Every major contributor passes
the existing 1% engine accuracy bar; all measured contributors are within 0.33%.
Raw outputs and commands are in `parity/`; [parity.json](parity.json) contains all
comparisons. Agreement applies to these supplied programs, builds and conditions.

All six discovery runs pass independent journal and peak reconstruction:
**4,280** completed common decision blocks, **356** charged fresh restarts,
**89** peak changes and **203** completed common 32-fight promotion blocks.
Two incomplete promotions do not replace the reconstructed peak. There are
**30 failed fight records**, retained and charged, rather than installed as
states. All final reconstructed peaks match their saved selections. Every
reference-export run has zero search decisions.

## Costs, artifacts and reproduction

Discovery uses **631.71 CPU seconds**; separate matrix audits use **84.07**.
Fresh native confirmation uses **153.29** and the four native/gcsim comparisons
use **452.95**. Native ability diagnostics rerun fights and their CPU is included.
Reference-export preparation and the initial generated support check are also
retained in [accounting.json](accounting.json). Build, orchestration and report
overhead are excluded. No cloud resources, full batch or serving feature is used.

- [source.zip](source.zip): actual working source, scripts, frozen manifests and
  protocol, including pre-existing uncommitted implementation work.
- [runs.zip](runs.zip): six discovery runs, complete evaluation/proposal journals,
  all budget reports, initial programs, checkpoints, withheld references,
  zero-decision reference exports and input snapshots.
- [archives.json](archives.json): archive SHA-256 and CRC checks.
- [selected-r1.gcsl](selected-r1.gcsl), [selected-r5.gcsl](selected-r5.gcsl): frozen
  blind selections, without manual cleanup.
- `reference-r*-*.gcsl`: exact reference programs after production baseline
  normalization and assembly with the same builds.

To continue an archived blind run, extract `source.zip` into a clean directory,
extract `runs.zip` under `runs/mavuika-c6-blind`, build, then for example:

```sh
cargo build --release --locked
./target/release/gso search --from-scratch --fixed-er --algorithm greedy \
  --manifest data/mavuika-c6-blind/r5.json --seed 91 --seconds 216 \
  --out runs/mavuika-c6-blind/r5-91 --audit 128
```

This is a continuation example; the published experiment stops at the frozen
108-second caps. Independent experiment reproduction requires fresh run/report
destinations and the pinned gcsim reference binary; the fixed-path orchestrator
deliberately refuses to overwrite an already sealed experiment.
