# Browser rotation refinement

This independent Rust crate builds GSO's browser-local refinement module. It pins
`gscouter-engine` at `b13b7864163e75852d4b9f282be009c94369ab29`. The native pilot's
engine pin, CLI and historical experiments remain unchanged.

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --locked --target wasm32-unknown-unknown --manifest-path browser/Cargo.toml
cargo test --locked --manifest-path browser/Cargo.toml
```

The output is `browser/target/wasm32-unknown-unknown/release/gso_browser.wasm`.
The Gscouter lab asset build copies this as `gso.wasm` alongside a dictionary
extracted from the main library and a build manifest. Do not commit binaries.

## Execution boundary

Run this module in a dedicated Web Worker. Every `step` performs at most one
full-horizon engine fight. The host **must** watchdog each call and terminate the
Worker if it exceeds its wall-time limit; the engine does not expose an
instruction budget and a duration does not stop a zero-frame infinite loop.
Worker termination discards that instance. A timeout must be reported as a
failure, never as a completed comparison. Graceful finish requests instead enter
an independent audit, after discarding any incomplete decision/promotion.

This module reuses the pilot's `Program`, mechanical dictionary macro edits,
generic action-list edits and numeric-coordinate neighbors from shared source.
It implements one greedy chain with a fixed user build, 16 common fights for
incumbent replacement, 32 fresh common fights for protected-peak promotion and
an independent paired audit of the frozen peak and original baseline. Tied
promotions retain the peak. Forty stalled decisions trigger three macro edits
from the peak; the restart is installed only after a completed decision block.
The last quarter of the discovery budget tries numeric coordinates, then resumes
macro search when those coordinates are exhausted.

This is a browser implementation of the pilot's refinement operators and paired
comparison rules, **not** a reproduction of its measured native CPU-budget runs.
Its budget is actual discovery fight attempts (failed fights included), not CPU
seconds. A complete decision and possible promotion are reserved before starting
another proposal, so a run may finish below its cap. The final audit has a
separate maximum of `2 * audit_samples` actual fights; identical baseline/peak
configs reuse paired samples and cost fewer. No native process, filesystem,
background server simulation, or constellation upgrading is involved. C0 and
mixed-constellation user builds are preserved. This does not implement rotation
creation for parties with no supplied baseline.

Candidate action parameters are simulator assumptions. Passing simulation is not
proof that every timing, hold duration or counter assumption is practical play.
Search estimates use changing seed blocks and are not an unbiased monotone DPS
curve; use the paired final audit to compare the original and selected rotations.
No unrestricted optimality guarantee or pilot recovery ratio is claimed here.

## ABI and commands

Exports: `memory`, `gso_alloc(len)`, `gso_free(ptr,len)`,
`gso_call(ptr,len)`, `gso_free_result(ptr)`.
Strings are UTF-8. A call result is a little-endian `u32 status`, `u32 body_len`,
then JSON bytes. Status 0 is a report; status 1 is `{"error":"..."}`. The caller
owns/frees the request buffer and each result buffer exactly once. Memory may
grow during calls, so refresh the host view after calling the module.

A module instance owns one session. Supported JSON requests:

```json
{"op":"start","input":{"build":"...","rotation":"...","dictionary":{},"seed":11,"max_evaluations":1024,"audit_samples":32}}
```

The dictionary must have the native version-1 dictionary schema, not an empty
object. Then send `{"op":"step"}` until the report's `complete` is true.
`{"op":"status"}` only reads the current report. `{"op":"finish"}` requests
an early end to discovery; continue stepping to complete the final audit.

Bounds: seed 0–2147483647, discovery 128–20000 fights, audit 16–256 fights per
candidate, build/rotation up to 64 KiB each, request up to 2 MiB. Source and
build declarations cannot override the frozen simulation conditions. Default
conditions match the pilot: 450 seconds, first 150 excluded, level-100 target,
10% resistance, swap delay 12, periodic energy enabled. `scoring` can supply a
frozen object with `duration`, `warmup`, `swap_delay`, `enemy_level`, `resistance`,
`radius`, `position` and `periodic_energy`. Browser duration is capped at 600 s.

Reports include phase/progress, actual fight counts, decisions/acceptances/peak
promotions, failed attempts, the pending operator, latest event, seed, engine,
conditions, selected and original full GCSL, and final audit statistics. `best_dps`
is a discovery estimate. `baseline`/`selected` are final `{n,mean,se}` statistics;
`improvement` is their paired difference with the paired standard error.
`audit_error` means the final comparison is unresolved, even if `complete` is
true. `stop_reason` distinguishes `budget` from `requested`. Statistical error
bars here describe fight randomness, not practical-play or model error.

## Native/WASM replay

`cargo run --locked --manifest-path browser/Cargo.toml --example replay -- INPUT.json`
executes the same incremental state machine with the same pinned engine natively
and prints its final report. Use the same input in the WASM host to compare
selected GCSL, counters and per-run audit statistics. The unit tests also replay
final baseline/peak audits directly through the latest native engine API, check
seeded determinism, preserve C0 builds and ensure interrupted comparisons cannot
promote partial candidates.
