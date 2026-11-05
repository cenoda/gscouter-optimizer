# Recovery and pilot protocol

The owner authorized implementation and local pilot verification on 2026-11-05
after loss of the previous machine. Both local and remote optimizer `main` were
at documentation commit `6fa5c194deb759f26e0a14f12d180d8c452383cf`; no previous
implementation was available. This protocol precedes the first new measurements.
The rotation-search plan and design review remain authoritative. Cloud deployment
and the full batch are outside this local pilot.

## Declared case and equipment

Development case: Flins C1 / Ineffa C1 / Columbina C1 / Sucrose C6.
Use level 90/90, base talents 10/10/10, level 90/90 signature weapons R1
for the five-stars, and a bounded sourced weapon comparison for Sucrose.
The owner's informal DPS expectation is not a reference result, acceptance
threshold, optimization parameter or stopping condition.

Preserve the existing C0 library program as a baseline, running on the declared
build. Import only that program, its build and the site's versioned artifact
weights; do not read alternative complete cycles during generator development.
The library's alternative is withheld until the generator and experiment
configuration are frozen. Sourced ability mechanics and small character action
blocks are allowed; disclose overlap with the baseline. The target case is a
development case, not a blind test of generalization.

Main-stat recommendation sources, checked 2026-11-05:

- [Flins KQM](https://keqingmains.com/q/flins-quickguide/): ATK/ATK/CRIT;
  Night of the Sky's Unveiling.
- [Ineffa KQM](https://keqingmains.com/q/ineffa-quickguide/): ATK/ATK/CRIT;
  retain the library's Aubade set for this party.
- [Columbina KQM](https://keqingmains.com/q/columbina-quickguide/): HP/HP/CRIT
  among its recommended choices; retain the library's Silken Moon set.
- [Sucrose KQM](https://keqingmains.com/q/sucrose-quickguide/): EM/EM/EM, VV;
  compare Evenstar R5, Sacrificial Fragments R5, TTDS R5 and A Thousand Floating
  Dreams R1 on multiple generated structures and bounded program refinement.
  These are conditional tested choices, not a proof over every weapon/cycle.

Freeze real piece allocations before search. Every five-star level-20 piece
starts with four distinct substats and has five upgrades. Quality tokens retain
their discrete tier; CV ceilings cannot exceed 40. Irrelevant tokens stay fixed.
Only existing ER/useful-damage tokens exchange, without changing the useful
count. Snapshot the site's maximum rolls and weights, with source commits and
hashes. Site weights are initialization hints from its different reference build.

## Objective and evidence

Every scored fight uses 450 seconds, 150 seconds warmup, one level-100 target,
10% resistance, radius 2 at (0, 2.4), energy every 480–720 frames, amount 1,
and swap delay 12. Initial energy follows the site's duration-mode default.
Report team and character DPS, energy/cooldown waits and both scoring halves.

At most four structure families and two active chain slots per profile.
Use 2/4/8/16 full-fight stages without heuristic low-DPS rejection. State
replacement requires a common completed block of at least 16 seeds. Leading
archive promotions compare incumbent and challenger on 32 fresh common seeds.
Final audits use a separate seed namespace after freezing the shortlist.
Preserve baseline, peak and C0-oriented slots within a six-program cap.
Cache only identical full configuration/engine/seed identities.

Before expansion, require invariant tests, repeated starts, equal measured-CPU
comparisons of annealing / greedy / random and structure / local-only variants,
fixed-ER / joint-ER diagnostics, increasing-budget curves, and exhaustive recovery
within an explicitly finite grammar. Final audit differences include paired
standard errors; a small noisy difference is unresolved. No absolute DPS is a
pass condition. Demonstrated gain must survive an independent audit; enumerated
recovery concerns that finite sample domain only. Record unsuccessful runs too.

Expansion remains deferred unless the broader suite in the plan is completed:
ordinary and energy-starved controls, changed roles/periods, mixed constellations,
C0-support/C6-dealer and C0-dealer transfers, eligible actual-gear residual loss,
held-out cycles and selective gcsim verification. A successful development case
alone cannot satisfy these requirements or establish a global optimality gap.

## Local accounting and durability

Charge setup, parsing, generated/failed fights, allocation repair, incumbent
refresh, weapon preparation, enumeration and audits. CPU budgets use process CPU
time, with elapsed time separately reported. Stop launching work at each cap;
report the bounded last-operation overrun. Persist RNG, family/chain state,
temperature, evaluation cache, provenance and all outcomes atomically so another
machine can resume. Commit source and small reproducible measured artifacts;
do not rely on an uncommitted machine-local best result.
