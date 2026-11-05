# Mavuika / Xilonen / Citlali / Nicole C6, frozen blind search

Requested by the owner on 2026-11-05. The owner's approximate 3,000,000 DPS
expectation is not an objective constant, success threshold or stopping rule.
Do not change the generator after seeing results or revealed reference cycles.

## Inputs

All four characters are C6, level 90/90, base talents 10/10/10. Freeze the
existing C6 library build from Gscouter commit
`d8d28c36d3801a4383feb240e78582fc674a00f1`,
`teams/mavuika-citlali-xilonen-nicole/alt/LhrRtbWNJk8t/build.txt`.
Its original signature weapons are R5. Also evaluate an R1 copy, changing only
weapon refinement, to distinguish the original pilot's R1 convention from the
library's C6/R5 example. All sets and supplied artifact stats remain identical.
These are fixed supplied builds, not certified physical standard envelopes.
Manifest order follows the source: Mavuika, Citlali, Xilonen, Nicole.

Pinned engine: `1ba0af119e0477d6f44ee966a78fa06d84d2c5de`.
Frozen optimizer implementation:
`79123f73118bf79daf5e723273ce316b691c02981f9cad58692311bf5fb76da4`.
No optimizer modifications are part of this experiment.

Both the original library rotation and its C6 alternative are withheld: copy
their bytes and record hashes without displaying their bodies or simulating
their DPS before all blind selections are frozen. Build declarations and
engine character implementations may be inspected for support. No known
complete cycle, rotation-derived action parameters or previous winner enters
generation. The earlier four-family generated validation at R1 used seed 11,
16 fights per family and 0.217 discovery/validation CPU seconds; this is a
support check, not tuning. The generator remains unchanged.

## Discovery and selection

For each refinement condition, use `--from-scratch --fixed-er --algorithm greedy`
at starts **11, 37, 91** and cumulative discovery caps **4, 12, 36, 108 CPU
seconds**. Use the existing revised path, four generated families, restarts and
last-quarter polish. Generation, failures, common comparisons and promotions
count against discovery CPU. Separate 128-fight audits never guide proposals.
Keep failures and incomplete runs; do not replace failed seeds.

Run refinement conditions concurrently, but starts within each condition
sequentially. All runs use the existing 450-second objective excluding the
first 150 seconds, one level-100 target, resistance 10%, radius 2, position
(0, 2.4), original periodic energy and swap conditions. No short-horizon or
low-DPS pruning, artifact reallocation or target-specific tuning is added.

After all six runs finish, select one program per refinement condition using
the **largest final discovery best_hint** at the 108-second cap, without
ranking on audited means. Freeze selected source and checkpoint candidate
identity before revealing either library cycle.

## Confirmation and boundaries

Only after selection, export both library rotations through the production
baseline parser using a tiny-cap, zero-decision reference-export run. This is
configuration preparation, not optimization. Charge its work separately.
Confirm each selected program and both references at identical builds and
fresh common seed **12,000,000,000,000**, **2,048 full fights**. Retain standard
errors, paired differences, per-character damage, ability and wait diagnostics.
Parser, unsupported-feature or runtime failures are outcomes, not permission
to alter a reference. Report baseline semantics and completion explicitly.

Check selected and C6-reference programs against pinned gcsim at each weapon
refinement, 2,048 fights using the same confirmation seed. Report discrepancy
or failure; do not silently certify accuracy. Journal and peak reconstruction
must verify completed decision/promotional blocks for every successful run.
Archive source, inputs, withheld references, selection records, reports,
checkpoints and journals with costs and hashes.

The current grammar cannot generate action parameters such as Mavuika's
`hold`, `recast`, `final` or `charge_cancel`, nor custom state predicates.
Default actions are supported, but this may exclude important strong play.
A numerical result is bounded search evidence, not a full search of that
character's available mechanics or proof of unrestricted optimality.
If engine/evaluator support fails, say which layer cannot calculate it.
