# Mavuika C6R5 2.25M development target

The owner selected the exact level-100 conditions of the previous 2,316,558 DPS
measurement as the comparison fixture. `data/mavuika-c6-r5-225/manifest.json`
freezes those aggregate stats: all four C6, signature R5, level 100, base talents
10, empty initial energy, 120 seconds without warmup, level-110 10%-resistance
target, radius 2 at (0, 2.4), one-frame swap delay, no periodic energy.
These are explicitly accepted numerical benchmark inputs; the reconstructed
Mavuika artifact allocation is not independently established as a legal set of
individual pieces. No gear/stat values may be changed to reach the target.

The pinned engine remains 1ba0af119e0477d6f44ee966a78fa06d84d2c5de. Discovery reads
only the manifest and capability checks, with --from-scratch --fixed-er and no
baseline. The known published/reconstructed rotations are not seed programs.
The target is external to the optimizer: no 2.25M constant guides proposals,
acceptance, pruning or early stopping inside a run.

This is a revealed development party, not a newly blinded held-out party. It
checks discovery without a supplied cycle but does not prove generalization.
Compare extended action forms with --basic-grammar at equal cumulative CPU caps
and repeated starts (11, 37, 91). Initial caps are 12 and 36 CPU seconds; extend
to 108 and, if useful, 324 seconds. Every state replacement requires a completed
common 16-fight block. Failed work, capability probing and restarts count against
the same discovery CPU cap. No low-score pruning or shortened fights.

Freeze each cap's discovery selection before independent audit. Stage audits
use 128 fights and are excluded from discovery decisions. Independently confirm
the discovery-selected winner with 2,048 fresh fights at seed 41000000000000;
confirm the fixed reference at that same seed. Acceptance means at least
2,250,000 mean team DPS on that fixed fixture, with standard error and ratio to
the reference reported. Audit rankings do not feed subsequent discovery.

Preserve source identity, manifest hash, proposals, checkpoints, common-block
acceptance evidence, all unsuccessful starts and CPU costs. Implementation
changes require fresh run directories and a separate phase; earlier failed
phases remain in the report. No cloud resources or other repositories changed.
