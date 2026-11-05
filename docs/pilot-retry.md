# Greedy pilot retry protocol

The owner requested documentation and push **before** implementation and retry
on 2026-11-05. This protocol extends `pilot-protocol.md`; all simulator,
allocation, common-block, CPU, shortlist and no-cloud constraints remain.
The preimplementation source is `74f7bdb`. Commit and push this document before
changing the search implementation. Preserve that commit as the recovery point.

## Evidence and scope

Greedy joint-ER was the strongest repeated control at the original local budget.
Its best start already found a three-burst Flins block. A later frozen-block
diagnostic found losses in cooldown filler and the surrounding support actions.
That diagnostic does not establish which improving moves were never proposed,
rejected or interrupted. The known alternative is now revealed: improvements
on this party are development evidence, not a fresh blind discovery claim.

Implement four changes before measurement:

1. Persist proposal provenance, mutation operators and phase, with enough source
   to distinguish unproposed neighbors, completed rejections, failures and
   unfinished proposals. Keep exact no-ops visible without spending simulation
   on them in the revised mode. Do not reject merely low-DPS candidates early.
2. Add character-agnostic action-list neighborhoods: contiguous action deletion,
   whole character-statement deletion, bounded normal-count adjustments, local
   action relocation and duplication. Preserve guards, loop structure and
   character ownership. Validate generated source with the pinned engine parser.
   Do not copy the revealed cycle or its action sequence into mutation templates.
3. In revised mode reserve the last quarter of each cumulative requested CPU cap
   for deterministic local improvement around the current search-selected peak.
   Enumerate a bounded, reproducible neighborhood, including physical ER/useful
   token exchanges in joint mode. Persist the neighborhood anchor and cursor.
   Reuse the production full-fight common-block decision and promotion path;
   all generation, repairs, failed attempts and comparisons consume the cap.
   Extensions of a run may return to discovery before the next polish window.
4. Execute the real production search/evaluator/checkpoint path on the existing
   eight-program domain. Use fixed allocation, one family, legal one-bit domain
   neighbors and a fixed 32-fight sample objective. Compare actual evaluated
   outcomes with independent exhaustive enumeration. No objective table may be
   passed to the search. Report failed recovery rather than relaxing the domain
   or changing seeds after seeing outcomes.

Keep the old proposal path available as an explicit legacy control; use fresh
run directories and caches for every variant. Old checkpoints deliberately
require their original source. New-mode configuration and this protocol are
included in experiment identity. Simulator code remains in the pinned engine.

## Frozen retry experiments

Use the original C0-derived baseline, original four structure families, unchanged
standard manifest, weapons, constellations and full-fight scoring conditions.
Do not initialize discovery from any previous winner or revealed reference.

Compare three variants, all greedy, at seeds **11, 37 and 91**:

- Legacy proposal, joint ER, no final polish.
- Revised proposal and final polish, joint ER.
- Revised proposal and final polish, fixed ER.

Use cumulative discovery CPU caps **4, 12, 36 and 108 seconds**, with independent
128-fight audits at each cap. Rank candidates before auditing and report each
run, medians, ranges, actual CPU, failures and proposal/operator diagnostics.
Audits never feed discovery, promotion or the polish neighborhood. Bounded
last-operation overruns remain visible. No absolute DPS threshold is a pass
condition, and the revealed reference's DPS is not a stopping condition.

For restricted recovery, exhaust the original eight configurations on 32 fixed
full fights at seed 6,000,000,000,000. Run the production greedy path from seeds
**11, 37, 91, 123 and 231** with a **4 CPU-second** cap each, no polish or ER
moves, and the same fixed 32-fight objective. Its initial point is chosen by its
RNG; all subsequent proposals flip exactly one of the three domain bits. The
search has no knowledge of exhaustive rankings. Separate final audits do not
change whether its peak matches the finite-objective maximum. This diagnostic
does not bound an unrestricted optimum.

Before matrix execution, commit the tested implementation and scripts. Freeze
them throughout the matrix. Run parser/neighborhood and pending-state tests,
strict Clippy and formatting checks, and validate actual common-block decisions
from journals. Restore an unfinished revised run from its saved checkpoint to
verify provenance, polish cursor and RNG continuation. Diagnose implementation
bugs before freezing; preserve and charge any discarded attempts.

After the matrix, freeze the highest search-selected result at the largest cap
using discovery estimates only, then confirm it, its same-allocation baseline
and the known reference on fresh 2,048-fight seeds. Confirm the selected program
against gcsim. Preserve all outcomes, source hashes, selected programs and a
portable recovery archive. Report diagnostics and audit costs separately from
discovery caps. Broader blind party validation and full batch expansion remain
outside this retry; choose new blind cases with the owner later.
