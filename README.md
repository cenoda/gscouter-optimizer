# gscouter-optimizer

> Status: Baseline-free discovery and local refinement are implemented; broad effectiveness and full-batch execution remain unvalidated.

A native Rust rotation-search pilot using `gscouter-engine` at commit
`1ba0af119e0477d6f44ee966a78fa06d84d2c5de`. The declared development party is
Flins C1 / Ineffa C1 / Columbina C1 / Sucrose C6. Source, frozen inputs,
checkpoints and measured reports make recovery independent of a previous machine.

The [search plan](docs/rotation-search.md) defines the wider workload; the
[design review](docs/design-review.md) records its constraints. The
[pilot protocol](docs/pilot-protocol.md) freezes this experiment before measurement.
Standard equipment bounds discovery; its DPS is never a player's actual score.

```sh
cargo build --release --locked
cargo test --release --locked
./target/release/gso validate
python3 scripts/pilot_matrix.py prepare
./target/release/gso search --manifest data/flins-c1/standard.json \
  --seconds 36 --seed 11 --algorithm anneal --out runs/example
python3 scripts/check_run.py runs/example
python3 scripts/pilot_matrix.py matrix
./target/release/gso enumerate --manifest data/flins-c1/standard.json
```

The documented greedy retry is available separately from the legacy control:

```sh
./target/release/gso search --manifest data/flins-c1/standard.json \
  --algorithm greedy --revised --seconds 36 --seed 11 --out runs/revised
python3 scripts/retry_pilot.py restricted
python3 scripts/retry_pilot.py matrix
```

`--revised` adds character-agnostic action-list neighborhoods, proposal
provenance and a final local-improvement window within the same CPU cap.
The legacy path remains available without that flag. `--restricted-domain
--fixed-er` runs the production evaluator and decision loop on the declared
eight-program domain; it never reads the enumeration table. New checkpoint
identities include the retry protocol, modes and build configuration. Exact
JSON floating-point round trips preserve restored candidate keys and logs.

`--seconds` caps cumulative discovery CPU, including child workers, failed
samples, parsing, ER repairs and incumbent comparisons. A worker is also bounded
by a wall-time limit; a timeout is unresolved. Checkpoints retain programs,
piece allocations, family states, temperature, random state and unfinished
proposals. Repeat a command with a larger cumulative CPU cap to continue.
Audits have separate seed namespaces for each budget and never affect proposals.
The fixed bounded shortlist is evaluated separately; audit CPU is reported.

The prototype executes one chain at a time and pauses other family states,
remaining within the two-active-chain limit. It offers up to four competing
families, condition-preserving mutations of the actual baseline GCSL and typed
new-role programs. State changes require a common 16-fight block; peak promotions
reevaluate both candidates on 32 fresh seeds. Early 2/4/8-fight stages never
install a state or reject a candidate for low DPS. Per-piece allocation moves
retain nine maximum-tier tokens, four distinct substats, fixed main stats,
useful-roll count and a CV ceiling of 38.9. Other quality tiers are explicitly
unsupported in this first manifest.

`data/flins-c1` contains versioned public-file snapshots from Gscouter, source
hashes and the frozen standard build. `scripts/export_pilot.py` reads only the
original C0 baseline and artifact data; complete higher-constellation alternatives
are withheld from generator construction. The finite enumeration command reports
an eight-program fixed-sample reference and explicitly labelled table-replay
recovery diagnostics; this is not an unrestricted optimum or a CPU benchmark.

Pending broader pilot work includes additional party controls, cross-profile
adaptation, actual-player-gear residual loss, both required mixed-C0 transfer
cases, comprehensive scheduling and deployment. No cloud resources or full
315,043-profile batch are started by these commands.

## Other-party input support

The same `search` and `validate` commands now accept four-member manifests for
other engine-supported parties. Pass `--manifest FILE` and, optionally,
`--baseline FILE`; the default baseline is `baseline.gcsl` beside the manifest.
Without `--from-scratch`, other parties use one baseline-derived source chain, retaining global setup,
functions, guards and action ownership. They do not use the Flins templates.
The existing 1,200-token source-mutation cap still applies; longer programs
can be evaluated but receive no source edits.
The original Flins physical manifest retains its legacy families and restricted
eight-program diagnostic. These diagnostics remain Flins-only.

For local refinement of a supplied build, format 2 adds `fixed_build`, a string
containing the complete character/weapon/set/stat GCSL declarations. It preserves
talents, weapon parameters and mixed sets. Keep the existing top-level fields
(`engine`, `site_commit`, `sources`, `max_rolls`, `members`); use an empty
`max_rolls` map. Each member records `key`, `cons`, canonical `weapon`,
`refinement`, `set` (descriptive), empty `useful`/`pieces` lists and an empty
`weights` map. Member order must match the build declarations. Simulation
options, targets, energy settings and active character belong outside
`fixed_build`. An optional frozen `scoring` object sets duration, warmup, swap
delay, target level/resistance/geometry and periodic energy. Without it the
objective remains 450 seconds excluding 150 seconds. Exported configs carry
the scoring-window metadata and evaluation caches distinguish the window.
Use `--algorithm greedy --revised --fixed-er` for baseline refinement, or
`--from-scratch --fixed-er` for build-only discovery with this format. It does not
construct hypothetical artifact pieces or certify a standard discovery build.
Format 1 still accepts physical envelopes with the pinned roll values and its
existing allocation constraints; signature weapons are no longer forced.

New discovery still requires four-star C6 and five-star C1/C2/C4/C6. Rarity is
resolved through engine aliases using [the frozen roster](data/character-rarities.json),
derived from `cenoda/Gscouter` commit `d8d28c36d3801a4383feb240e78582fc674a00f1`,
`internal/enka/data/character.dm.json` (SHA-256
`b05aeda1c579e47c114e59abb276022dcfd8c6e591341f9cc4dc0dc94c9964a6`).
Unknown metadata and unsupported engine features are explicit errors.

This is input and execution support, not evidence of competitive rotations across
parties. The measured Flins reports
remain the only effectiveness evidence. Other-party unit and smoke checks only
verify execution, mutation and checkpoint invariants. No new comparison matrix,
profile sweep, cloud run or serving integration is included. Changed source
requires fresh output directories; historical checkpoints use their original
source revision.

## Discovering without a rotation

```sh
./target/release/gso search --from-scratch --fixed-er \
  --manifest data/flins-c1/standard.json --seconds 36 --seed 11 \
  --out runs/generated
python3 scripts/check_run.py runs/generated
python3 scripts/check_peaks.py runs/generated
```

`--from-scratch` reads only the build manifest. It never reads the adjacent
baseline, Flins templates or previous winners; `--baseline`, `--local-only` and
`--restricted-domain` are incompatible. It defaults to greedy and enables final
local improvement. The same mode works with other eligible four-member
manifests, including fixed builds with `--fixed-er`.

Four generated families initially give each member a field opportunity, with
shuffled visit orders, sampled actions, periods and lengths. Visits, field
ownership, openings and repeat structure can change throughout discovery.
Charged fresh-generation restarts keep exploration possible after a family
stalls; a completed common 16-fight block precedes every restart, and the peak
remains separately protected by common 32-fight promotion comparisons.
The archive remains bounded to six candidates, including the generated control.

The last quarter of the CPU cap refines the selected peak. Reports retain the
pre-polish candidate and include it in the same bounded independent audit,
allowing discovery and subsequent polish outcomes to be inspected separately.
`baseline` fields refer to the **first generated starting program** in this mode,
as stated by `reference_origin`; they are not library-cycle comparisons.
`experiment.json` records initial programs and the absence of baseline reads.

The grammar currently uses engine-supported default normal/charged attacks,
skills, bursts, dash and jump, plus bounded supported action parameters,
resource-guarded charge/dash continuations, timing delays and crystallize pickups.
`--basic-grammar` disables these additions for an equal-condition ablation.
Arbitrary character-specific
status predicates are not enumerated. Existing engine checks do not establish
all dynamic prerequisites. General effectiveness and unrestricted optimality
remain unvalidated. See [the implementation boundary](docs/from-scratch.md).

`python3 scripts/smoke_discovery.py` creates two synthetic build-only party
fixtures and runs three starts, two cumulative caps, journal/peak checks and
copied-checkpoint recovery. These are execution checks, not held-out quality
benchmarks or comparisons against known strong cycles. Use fresh run/report
directories, optionally with `--out` and `--report`.

## Repository responsibilities

| Repository | Responsibility |
|---|---|
| [gscouter-engine](https://github.com/cenoda/gscouter-engine) | Accurate, fast simulation of a supplied build and GCSL rotation; engine parity, native and WebAssembly releases. |
| **gscouter-optimizer** | Standard-build preparation, candidate generation, rotation search, batch scheduling, checkpoints, cost accounting and search reports. |
| [Gscouter](https://github.com/cenoda/Gscouter) | Party and artifact data, user-build scoring, result presentation and application of discovered cycles. |

The optimizer uses the engine's Rust library at a pinned commit. It consumes versioned party, build and artifact-weight snapshots from Gscouter, rather than importing its private Go packages or copying the simulator. It exports GCSL candidates and metadata identifying the engine, input data, seeds, sample counts and measurements. The site evaluates candidates using the player's actual equipment and stats.

The local pilot is implemented in Rust. No cloud deployment or batch CI is configured. See the [measured pilot report](reports/pilot-2026-11-05/README.md) for results, limitations and portable machine recovery.

## Documents

- [Reference replay diagnostics](docs/replay.md): configurable scoring windows and frame-by-frame damage through the pinned engine's public API.
- [Mavuika high-score replay investigation](reports/mavuika-c6-replay-2026-11-05/README.md): original video observations, the author's published input sequence, unsuccessful reconstructions and independent audits.
- [Measured Flins pilot](reports/pilot-2026-11-05/README.md): independent results, comparisons, failure cases and recovery.
- [Measured greedy retry](reports/pilot-retry-2026-11-05/README.md): frozen-budget comparison, independent confirmation and complete recovery journals.
- [Greedy retry protocol](docs/pilot-retry.md): documented before implementation; revised neighborhoods, local improvement and production-path recovery checks.

- [Rotation search](docs/rotation-search.md): agreed decisions and the proposed algorithm, workload and delivery plan.
- [Design review](docs/design-review.md): critique, accepted changes, limits and pilot requirements.
- [Working instructions](CLAUDE.md): repository scope and current implementation boundary.

The plan was moved from `gscouter-engine/docs/rotation-search.md`, originally committed as [`84a53e4`](https://github.com/cenoda/gscouter-engine/commit/84a53e45e91a1c9ce3cabaaf2cf2c7082283bc53). This repository is now its authoritative home; the engine keeps a forwarding link.

## License

GNU AGPL v3 or later; see [LICENSE](LICENSE), retained from the engine repository with the moved documentation.
