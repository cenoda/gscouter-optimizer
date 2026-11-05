# Greedy joint-ER versus the known three-burst reference

This is a post-pilot diagnostic requested after the reference was revealed.
It does not modify the generator, resume search or seed any search with the
reference. The reference is a known stronger library cycle, not a certified
unrestricted optimum. The comparison uses `greedy-37`, the highest independently
audited outcome among the three joint-ER greedy starts at 36 CPU-seconds; it is
not a median-run claim.

## Same three-burst structure, different filler and support actions

The main Flins blocks below use E for `skill`, Q for `burst`, N for `attack`
and D for `dash`. `N4` means four normal-attack commands, not four damage hits.
Both blocks contain four E commands (entry plus three subsequent skills) and
three Q commands. The interpretation of a skill or burst depends on live
character status; this notation does not treat every E or Q as the same action.

```text
Known reference:
E N E Q | N4 D N2 E N Q | N4 D N2 E N Q | N2

Greedy joint-ER, seed 37:
E N E N Q | N4 D N D E Q | D N2 E N Q
```

The reference includes 17 normal-attack commands in this main block; greedy
includes 10. Greedy has one additional normal before the first burst, two
dashes before the second subsequent skill, a dash after the second burst, and
no final N2. More normal attacks are not a universal improvement rule; their
placement and the resulting cooldown and buff timing matter.

Outside the Flins block, greedy retains the baseline's extra Columbina normal
in the even, noninitial branch and inserts two dashes after it. Its ending
Sucrose fallback also has an extra charged attack before the original normal
and charged attack. The reference has neither the Columbina normal/dashes nor
the extra Sucrose charged attack. Conditions and the six-pass alternating loop
otherwise retain the same structure; differing print strings have no effect.

## Frozen block substitutions

All four conditions below use the same original standard artifact allocation,
the declared C1/C1/C1/C6 party and equipment, 450-second full fights, 150-second
warmup, 2,048 iterations and common fresh seeds starting at 9,000,000,000,000.
The support block includes both the entry and exit actions around Flins.

| Support block | Flins block | Team DPS | SE | Gain over all-greedy |
|---|---|---:|---:|---:|
| Greedy | Greedy | 326,277.33 | 49.05 | — |
| Greedy | Reference | 330,822.65 | 47.78 | +4,545.32 |
| Reference | Greedy | 335,331.62 | 44.54 | +9,054.29 |
| Reference | Reference | 338,593.90 | 43.81 | +12,316.56 |

Paired standard errors of the two single-block gains are 66.51 and 63.20 DPS,
respectively. Replacing the Flins block after replacing support adds 3,262.28
DPS; replacing support after replacing Flins adds 7,771.24 DPS. The gains are
not additive, so this does not assign an independent percentage of the loss
to each block. Changing a block changes later timing, buffs, reactions and
random-number consumption. These are whole-block intervention effects, not
proof that each individual inserted action is harmful.

The same four conditions were also run at the frozen greedy artifact
allocation. DPS values are 326,324.35 / 330,869.37 / 335,377.56 / 338,640.53,
in table order. At these two allocations, artifact differences contribute only
about 46–47 DPS to each corresponding comparison; the cycle-block result is
essentially unchanged. This is not an optimality claim about artifact allocation.

## What the execution diagnostics show

All counts and wait durations are means over the scored 300-second window,
not per rotation. Ability hits are not universal cast-count estimates.

| Standard allocation condition | Flins skill-cooldown wait (s) | Spearstorm hits | Symphony hits |
|---|---:|---:|---:|
| All greedy | 28.80 | 48 | 96 |
| Reference Flins only | 2.13 | 48 | 96 |
| Reference support only | 30.60 | 51 | 100 |
| All reference | 2.27 | 50 | 100 |

At greedy support, replacing the Flins block retains the same scored
Spearstorm/Symphony hit counts while sharply reducing skill-cooldown waits
and increasing normal-attack damage. The three-burst structure is already
present; filling the intervals differs. Moving to reference support changes
the cadence as well as damage conditions. Boundary clipping of the scoring
window also affects counts; the table is not a per-cycle cast log.

Per-character team DPS at the standard allocation:

| Support / Flins | Flins | Ineffa | Columbina | Sucrose |
|---|---:|---:|---:|---:|
| Greedy / Greedy | 142,316 | 91,904 | 85,839 | 6,218 |
| Greedy / Reference | 146,742 | 92,681 | 84,382 | 7,018 |
| Reference / Greedy | 150,899 | 94,208 | 84,721 | 5,504 |
| Reference / Reference | 151,992 | 95,665 | 84,354 | 6,583 |

The measured result supports treating greedy joint-ER as the strongest repeated
control at the tested pilot budget. It does not establish superiority at larger
budgets or on other parties. In this particular winner, discovery found the
three-burst main-field pattern but retained costly surrounding actions and
less productive cooldown filler. A new blind pilot is needed to assess any
generator changes inspired by this revealed comparison.

## Reproduction and accounting

```sh
cargo build --release --locked --example trace
python3 scripts/compare_greedy_reference.py
```

The script uses the archived source copy if the original ignored run directory
is absent. Both original sources, all eight full configurations, raw outputs
including per-iteration DPS, mean waits and ability diagnostics are retained
here. [comparison.json](comparison.json) records hashes, exact commands, common
seeds and paired block contrasts. The engine remains pinned at
`1ba0af119e0477d6f44ee966a78fa06d84d2c5de`. No new gcsim parity check was
performed for these hybrids; the original baseline, selected annealing program
and known reference retain their earlier separate parity checks.

The eight evaluations consumed 122.5064 child-process CPU-seconds and 31.0762
wall-seconds at at most four concurrent diagnostic processes. This includes
both the public API score pass and the example's separate ability-diagnostic
pass. Build, parent-script and documentation overhead are separate and are
not included in that measured child-process figure. These diagnostic costs
are additional to the original pilot accounting.
