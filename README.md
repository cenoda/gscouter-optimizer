# gscouter-optimizer

> Status: Planning only; no implementation · Last updated: 2026-11-04

Find build-dependent sustained-DPS rotations for Gscouter using the Rust simulation engine. Intermediate constellations matter: Flins changes at C1, so a C0 rotation is not sufficient for every build.

The agreed scope, standard builds, search algorithm and compute estimate are in [the rotation-search plan](docs/rotation-search.md). In offline discovery, four-star members are fixed at C6 and five-star members independently take C1, C2, C4 or C6. A party with n five-star members has 4^n new search profiles. C0 and mixed-C0 cases keep the existing build/scoring path and receive no new batch search. The hard limit remains one week, $500 total and two 64-core Azure Fals v7 VMs; throughput estimates remain provisional until measured on those VMs.

The frozen 4,570-party census gives **315,043 new profiles**, versus 2,856,250 under the superseded five-stage-for-every-member rule. This is a profile-count reduction of about 89%, not a measured reduction in all search CPU time.

The [design review](docs/design-review.md) records seven implementation risks and their dispositions. The revised proposal couples ER allocation with cycles, uses two persistent search chains with staged evaluation, validates archive promotions, exports diverse candidates and schedules work by measured CPU cost. It supersedes the fixed 1,000-candidate/eight-chain allocation and the derived 5.33-day forecast; these changes are still unimplemented.

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
- [Design review](docs/design-review.md): critique, accepted changes, limits and pilot requirements.
- [Working instructions](CLAUDE.md): repository scope and current implementation boundary.

The plan was moved from `gscouter-engine/docs/rotation-search.md`, originally committed as [`84a53e4`](https://github.com/cenoda/gscouter-engine/commit/84a53e45e91a1c9ce3cabaaf2cf2c7082283bc53). This repository is now its authoritative home; the engine keeps a forwarding link.

## License

GNU AGPL v3 or later; see [LICENSE](LICENSE), retained from the engine repository with the moved documentation.
