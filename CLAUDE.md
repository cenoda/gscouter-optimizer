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
- In offline discovery, fix all four-star members at C6; vary only five-star members over C1, C2, C4 and C6. A party with n five-star members has 4^n new profiles. C0/mixed-C0 combinations get no new batch search, but must not be forced to use only the old rotation: reuse suitable archived programs, keep the existing baseline and evaluate the bounded shortlist on the actual user build. Discovery profiles are not serving eligibility rules. Five-star C3 uses C2's structural search results; C5 uses C4's. Real scoring retains actual constellation/talent bonuses and never raises a player's constellation.
- Mixed-C0 transfer does not authorize more profiles, extra servers, larger serving comparison limits or serving-time annealing. Include the baseline within the existing candidate cap and charge failed candidates and transfer audits to the existing budgets. Do not promise zero comparison cost or optimal mixed-C0 play without evidence.
- Standard builds bound offline discovery; they never replace the player's actual build for result scoring.
- Distinguish agreed scope, proposed implementation, measured results and estimates. Do not claim the planned algorithm or its throughput has already been validated.
- Apply the dispositions in `docs/design-review.md`: numeric-only constellation tags do not remove profiles, low early DPS is not a correctness-preserving pruning rule, and ER repair and incumbent reevaluation count against discovery CPU budgets. Retain diverse candidate programs, not only the final reranking pair.
- A chain-state replacement needs a completed common 16-fight decision block for both current and proposed states; short stages cannot accept. ER allocation moves preserve frozen roll counts, qualities and useful-roll budget, with a per-piece CV ceiling no greater than 40. Keep a C0-on-field transfer candidate slot within the six-candidate limit; missing validated transfer is reported, not fabricated.
- A C0 compatibility probe records a context/seed-specific outcome, not a generic `requires_c1_plus` proof. A discovery-location ceiling does not mean all locations were explored. Reuse samples only for identical evaluation keys and keep final audit seeds independent.
- Use a bounded outer competition between play structures and inner annealing; at most two chain slots are active per profile in total, not per structure. Imported-candidate refinement consumes the same distinct discovery-location and CPU quotas. Family count, seed preparation and refinement are not unaccounted multipliers.
- Require a pilot with withheld known cycles, equal-cost ablations, repeated starts, budget curves and an explicitly restricted enumerated reference. A plateau or repeatability does not bound the unrestricted optimality gap. Actual-gear pilot search is limited to already eligible non-C0 profiles; no C0 search grid or serving-time optimizer is implied.
- No extra cores, spending above $500 or deadline extension is implicit in the plan. Report incomplete or unsupported profiles explicitly.
