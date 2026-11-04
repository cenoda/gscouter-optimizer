# gscouter-optimizer

The rotation-search application for Gscouter. Read `README.md` for repository boundaries and `docs/rotation-search.md` for the agreed scope and budget.

## Current boundary

- This repository contains documentation only. The owner's current instruction is to create the repository and move the documents, **without implementing code**. Do not add application code, build scaffolding, CI workflows or deployment scripts until the owner asks for implementation.
- The one-week, $500, 128-core plan is a workload budget, not authorization to provision VMs or start a batch now.
- Write documents, comments and commit messages in English; speak to the owner in Korean.
- The default branch is `main`. The initial documentation is authorized directly to `main` with CI skipped.
- Preserve other repositories' worktrees, branches and in-progress changes. Documentation migration does not authorize porting changes or an engine documentation rewrite.

## Design boundaries

- Keep the simulator in `cenoda/gscouter-engine`; use its library at an explicit commit when implementation begins.
- Keep site presentation and real-user build scoring in `cenoda/Gscouter`. Exchange versioned input data and candidate artifacts; do not copy engine code or depend directly on Gscouter's Go `internal` packages.
- In offline discovery, fix all four-star members at C6; vary only five-star members over C1, C2, C4 and C6. A party with n five-star members has 4^n new profiles. The owner explicitly excluded mixed-C0 profiles from new search too: C0 cases keep the existing build/scoring path. Five-star C3 uses C2's structural search results; C5 uses C4's. Real scoring retains actual constellation/talent bonuses and never raises a player's four-star constellation to C6.
- Standard builds bound offline discovery; they never replace the player's actual build for result scoring.
- Distinguish agreed scope, proposed implementation, measured results and estimates. Do not claim the planned algorithm or its throughput has already been validated.
- Apply the dispositions in `docs/design-review.md`: numeric-only constellation tags do not remove profiles, low early DPS is not a correctness-preserving pruning rule, and ER repair and incumbent reevaluation count against discovery CPU budgets. Retain diverse candidate programs, not only the final reranking pair.
- No extra cores, spending above $500 or deadline extension is implicit in the plan. Report incomplete or unsupported profiles explicitly.
