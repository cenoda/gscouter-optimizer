# gscouter-optimizer

> Status: Local pilot implementation; full-batch expansion deferred.

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

## Repository responsibilities

| Repository | Responsibility |
|---|---|
| [gscouter-engine](https://github.com/cenoda/gscouter-engine) | Accurate, fast simulation of a supplied build and GCSL rotation; engine parity, native and WebAssembly releases. |
| **gscouter-optimizer** | Standard-build preparation, candidate generation, rotation search, batch scheduling, checkpoints, cost accounting and search reports. |
| [Gscouter](https://github.com/cenoda/Gscouter) | Party and artifact data, user-build scoring, result presentation and application of discovered cycles. |

The optimizer uses the engine's Rust library at a pinned commit. It consumes versioned party, build and artifact-weight snapshots from Gscouter, rather than importing its private Go packages or copying the simulator. It exports GCSL candidates and metadata identifying the engine, input data, seeds, sample counts and measurements. The site evaluates candidates using the player's actual equipment and stats.

The local pilot is implemented in Rust. No cloud deployment or batch CI is configured. See the [measured pilot report](reports/pilot-2026-11-05/README.md) for results, limitations and portable machine recovery.

## Documents

- [Measured Flins pilot](reports/pilot-2026-11-05/README.md): independent results, comparisons, failure cases and recovery.
- [Greedy retry protocol](docs/pilot-retry.md): documented before implementation; revised neighborhoods, local improvement and production-path recovery checks.

- [Rotation search](docs/rotation-search.md): agreed decisions and the proposed algorithm, workload and delivery plan.
- [Design review](docs/design-review.md): critique, accepted changes, limits and pilot requirements.
- [Working instructions](CLAUDE.md): repository scope and current implementation boundary.

The plan was moved from `gscouter-engine/docs/rotation-search.md`, originally committed as [`84a53e4`](https://github.com/cenoda/gscouter-engine/commit/84a53e45e91a1c9ce3cabaaf2cf2c7082283bc53). This repository is now its authoritative home; the engine keeps a forwarding link.

## License

GNU AGPL v3 or later; see [LICENSE](LICENSE), retained from the engine repository with the moved documentation.
