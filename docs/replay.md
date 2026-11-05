# Reference replay diagnostics

Use a complete external GCSL config to check a known rotation before treating
it as a discovery benchmark. Keep the original build, initial state, target,
duration and scoring window explicit. These diagnostics use the pinned `gse`
library without changing its simulation or character implementations.

```sh
cargo build --release --example replay --example timeline
target/release/examples/replay reference.gcsl 2048 31000000000000 0 > replay.json
target/release/examples/timeline reference.gcsl 28000000000000 1200 > timeline.json
```

`replay CONFIG ITERATIONS SEED WARMUP_SECONDS` measures every iteration once.
It reports team mean and standard error, individual iteration DPS, mean ability
hit counts and DPS, initial panel-stat arrays, and the first fight's final panel
stats. Stat indices are the pinned engine's `gse::types::st` indices. Final panel
stats are not attack snapshots. The config must have a fixed duration longer
than the warmup; unsupported inputs and incomplete fights fail explicitly.

`timeline CONFIG SEED FRAMES` reconstructs the first 1–1800 frames by replaying
the same seed with successive one-frame horizons and differencing ability
totals. This is a diagnostic, not the search evaluator. Multiple same-ability
hits on one frame remain grouped. The config must use a fixed time limit and
cover the requested frames. Damage-mode targets are rejected. Each successive
prefix must preserve preceding ability damage and hit counts.

For the Mavuika replay study, timeline damage through 20 seconds matched a
single 20-second replay within `1.5e-8` damage. The existing `trace` diagnostic
remains available for the frozen 450-second / 150-second-warmup experiment.

A high video score becomes a validated benchmark only after the input sequence
and relevant setup are reproduced. Matching one burst hit, tuning an unknown
artifact stat to the displayed score, or measuring a different published cycle
does not establish that reproduction. Post-reference refinement must be labeled
separately from a previously frozen blind discovery run.
