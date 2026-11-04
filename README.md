# gscouter-optimizer

> Status: Planning only; no implementation · Last updated: 2026-11-04

Find build-dependent sustained-DPS rotations for Gscouter using the Rust simulation engine. Intermediate constellations matter: Flins changes at C1, so a C0 rotation is not sufficient for every build.

The agreed scope, standard builds, search algorithm and compute estimate are in [the rotation-search plan](docs/rotation-search.md). The first batch targets each party member independently at C0, C1, C2, C4 and C6, with a hard limit of one week, $500 total and two 64-core Azure Fals v7 VMs. Estimates remain provisional until measured on those VMs.

## Repository responsibilities

| Repository | Responsibility |
|---|---|
| [gscouter-engine](https://github.com/cenoda/gscouter-engine) | Accurate, fast simulation of a supplied build and GCSL rotation; engine parity, native and WebAssembly releases. |
| **gscouter-optimizer** | Standard-build preparation, candidate generation, rotation search, batch scheduling, checkpoints, cost accounting and search reports. |
| [Gscouter](https://github.com/cenoda/Gscouter) | Party and artifact data, user-build scoring, result presentation and application of discovered cycles. |

The planned optimizer uses the engine's Rust library at a pinned commit. It consumes versioned party, build and artifact-weight snapshots from Gscouter, rather than importing its private Go packages or copying the simulator. It exports GCSL candidates and metadata identifying the engine, input data, seeds, sample counts and measurements. The site evaluates candidates using the player's actual equipment and stats.

The optimizer's main implementation language is planned to be Rust. No engine dependency, executable, CI workflow or cloud deployment is configured yet. Creating this repository does not start implementation or allocate cloud resources.

## Documents

- [Rotation search](docs/rotation-search.md): agreed decisions and the proposed algorithm, workload and delivery plan.
- [Working instructions](CLAUDE.md): repository scope and current implementation boundary.

The plan was moved from `gscouter-engine/docs/rotation-search.md`, originally committed as [`84a53e4`](https://github.com/cenoda/gscouter-engine/commit/84a53e45e91a1c9ce3cabaaf2cf2c7082283bc53). This repository is now its authoritative home; the engine keeps a forwarding link.

## License

GNU AGPL v3 or later; see [LICENSE](LICENSE), retained from the engine repository with the moved documentation.
