# Rotation-search design review

> Status: Review incorporated into the implementation proposal; no code · Last updated: 2026-11-04

The owner supplied a seven-point critique of the first plan on 2026-11-04. This record distinguishes accepted problems from suggested fixes that would change the objective, silently narrow coverage or assume unmeasured gains. The [revised plan](rotation-search.md) is authoritative for implementation proposals. The owner's standard equipment, C0/C1/C2/C4/C6 stages, one-week deadline, $500 total and 128-core limit remain fixed.

## 1. Eight chains divide the search too shallowly

**Accepted risk.** The previous 1,000 proposals split eight ways meant only 125 proposals per chain before invalid moves, calibration and reevaluation. There was no evidence that this depth reached useful new cycles. It does not mathematically prove annealing cannot work, but it is insufficient support for the original convergence expectation.

Begin with two persistent chains per active profile and schedule across parties. Charge actual CPU time rather than fixed 16-fight candidates, and continue chain/cooling state across rounds. The pilot measures chain depth, accepted transitions and independently verified gain; one chain is an option if two remain shallow. Staged sampling is not assumed to produce a particular rejection rate or depth multiplier.

## 2. ER and cycles cannot be optimized in isolation

**Accepted.** Library weights were only local sensitivities under the old build and rotation. Deferring ER checks to spare capacity could reject a good cycle before discovering a viable allocation, or retain excess ER after an energy-saving change.

Search `(program, attainable substat allocation)` jointly. Support ER/damage-roll moves and bounded coupled repair when changing cycle structure. Run the baseline under the same allocation opportunities. Count every repair and comparison in discovery; distinguish cycle-only gain from combined stat/cycle gain. Overflow and waits are diagnostic hints, not an analytic solution for ER. Do not modify the player's actual stats at serving time.

## 3. Anchors need realistic support contexts and measurable acquisition

**Accepted in part.** Seventeen mandatory anchors with the other three members at C0 overemphasized one context. Add library and mixed contexts, including four-star supports at C6, all-C0/all-C6 and C0/C1 contrasts. These conventional contexts are not measured user prevalence.

Replace the undefined notion of a profile being far from its true optimum with observable signals: independently checked progress per CPU time, leading-candidate uncertainty, benefit from profile-specific candidates and mechanic/energy coverage gaps. Maintain an exploration allocation so early flat results do not starve a party.

**Do not collapse all numeric-only constellations.** Changing damage distribution can change the best field time, action mix and buff allocation without changing a cooldown or energy mechanic. Mechanic tags prioritize search and share seeds; they are not equivalence proofs. Preserve all 625 profile comparisons. The owner explicitly excluded C3/C5; do not extend that exclusion merely to reduce a spreadsheet total.

## 4. A standard-build winner need not transfer to actual equipment

**Accepted risk; clarify the original archive.** The old plan already retained up to 16 shared programs per party. Its top-two rule described the most expensive reranking, not an explicit instruction to export only two programs. The output contract nevertheless lacked enough detail to prevent that implementation mistake.

Define up to six diverse candidates per profile including baseline, peak standard DPS and measured lower-ER/alternate-period or less proc-dependent options. Store programs by hash, expose actual evaluation depth and annotate dependencies. A signature-only batch cannot certify behavior with Favonius, Sacrificial or a swap-buff weapon it never tested. Actual-build evaluation and fallback remain necessary; diversity is a mitigation, not proof of transfer.

**Accepted preparation bottleneck and circularity.** Reuse library data and sourced character/role recommendation records, with explicit conflicts and missing data. For non-signature weapons, bounded comparisons use multiple seed cycles and a limited build/cycle refinement. Record the best tested conditional choice, not a claim that the best weapon over all possible rotations has been proven. Standard equipment is still one selected weapon per character, not a new exhaustive weapon dimension.

## 5. Noisy best-so-far estimates can trap the search

**Accepted.** Reusing a lucky short-seed estimate as an immutable incumbent can corrupt both search direction and the archive. The amount of noise depends on the build; it is not universally a particular percentage.

Separate chain state, provisional challengers and validated archive entries. A potential best/archive promotion triggers a fresh comparison of both challenger and incumbent, initially 32 fights each. Refresh incumbents, rotate search blocks and reserve an untouched final audit stream. Charge these runs to the search budget. One fresh block reduces selection bias; it does not eliminate all winner's curse or establish a formal guarantee after repeated adaptive selection.

Staged evaluation uses 2/4/8/16 full fights, growing where needed and affordable. The earliest stages diagnose and prioritize; one or two low scores alone do not certify inferiority. Empirical uncertainty checks under repeated looks and heavy-tailed energy waits are heuristics unless separately justified. Measure false rejections by completing an audit sample of rejected candidates.

## 6. Single-config throughput and unrestricted mutations are unsafe planning bases

**Accepted.** At 20 ms instead of 12 ms per fight, the old fixed 3,436,640,000-fight plan costs 19,092.44 CPU-hours, above the 15,052.8-hour effective allowance. The original 5.33-day forecast is withdrawn, not relabeled as a measurement.

Pilot normal and generated programs, reaction-heavy parties, ER starvation and failed/partial execution on the actual hardware. Measure workload-weighted CPU cost and tail behavior, then enforce CPU-hour phase caps. Prefer typed valid action blocks with prerequisites and retain some action-level exploration so the block dictionary does not define the answer in advance.

**Do not hard-reject a cycle just because its first 60-90 seconds are below 70% of a baseline.** That surrogate can rank startup, buff buildup and repeated energy use differently from the defined sustained objective. Use short rollouts for diagnostics; enable horizon pruning only after measuring false rejections, documenting the heuristic and auditing its rejects. Exact errors and proven no-progress execution can stop immediately. A work-limit timeout is unresolved, not evidence of low DPS.

## 7. Equal per-party discovery quotas waste scarce time

**Accepted with the existing coverage scope preserved.** All parties remain in the inventory. Supported parties receive priority; the 238 draft parties follow and are explicitly deferred if time runs out. Do not silently remove them from counts or claim an all-party result after skipping them.

Give eligible parties an initial exploration allocation, then assign additional CPU quanta by measured progress, uncertainty and coverage, with an exploration floor. A historically low-use or low-DPS party is not thereby solved. Known behavior changes such as Flins C1 get explicit pilot and anchor coverage. Discovery quanta include invalid candidates, repairs, cross-profile checks and promotion tests.

## Revised resource accounting

The previous fixed candidate/fight schedule is superseded by these initial CPU-hour caps:

| Phase | CPU-hours |
|---|---:|
| Pilot and standard-build preparation | 600 |
| Adaptive shared discovery, including ER and incumbent checks | 6,200 |
| Full-profile coverage | 1,400 |
| Reranking, final audits and export | 3,000 |
| Main phase allocation | **11,200** |
| Unallocated contingency | **3,852.8** |
| Effective planning allowance | **15,052.8** |

These are allocations, not measured costs or a quality guarantee. Pilot results must reserve the real cost of coverage before assigning discovery depth. Keep the one-week/$500 hard stop and disclose unresolved outcomes. A staged evaluator that promotes nearly every candidate, or ER repair that runs often, earns no assumed compute discount.

## Pilot requirements before the full batch

- Compare annealing, improvement-only and random mutation by independently audited gain per CPU-hour on held-out cases.
- Measure validity rate, chain depth, racing promotions, rejected-candidate recovery and stability of best/archived programs.
- Compare fixed-ER and joint-ER discovery under identical total cost, including newly feasible and energy-saving cycles.
- Exercise mixed constellations and C6 four-star support contexts; numeric-only tags must not remove profile coverage.
- Evaluate diverse candidate transfer on declared alternate builds, including proc/reset/swap-dependent weapon cases where available. Do not report untested equipment as compatible.
- Measure total CPU cost and elapsed throughput on both VMs, including preparation, parsing, failed fights, retries, checkpointing and independent validation.

The relevant algorithm references are the [original simulated annealing paper](https://doi.org/10.1126/science.220.4598.671) and the authors' [iterated-racing paper and supplementary material](https://iridia.ulb.ac.be/supp/IridiaSupp2016-003/). They motivate acceptance and adaptive evaluation; they do not validate this domain-specific combination or promise a speedup for it.

This review changes documentation only. No application code, CI, deployment or cloud batch is added or run.
