# Rotation-search design review

> Status: Review incorporated into the implementation proposal; no code · Last updated: 2026-11-04

The owner supplied a seven-point critique of the first plan on 2026-11-04. This record distinguishes accepted problems from suggested fixes that would change the objective, silently narrow coverage or assume unmeasured gains. The [revised plan](rotation-search.md) is authoritative for implementation proposals. Standard equipment, the one-week deadline, $500 total and 128-core limit remain fixed.

After this review, the owner narrowed offline discovery explicitly: four-star members are C6 only; five-star members take C1/C2/C4/C6. New search has `4^n` profiles per party with n five-star members, totaling 315,043 in the frozen census. This supersedes the earlier 625-per-party coverage rule discussed below; it is an owner decision, not an inferred numeric-constellation optimization.

The owner then identified C0 Furina/C6 Skirk as a failure of the earlier interpretation that mixed-C0 users must use only the old cycle. Discovery scope and serving eligibility are now separate: C0 combinations get no additional batch search but may reuse archived programs, compared on their actual build alongside baseline within the existing candidate/sample/CPU caps. No C0 coordinate is simulated as C1 merely because C1 was used to retrieve a program. See section 5 of the plan.

## 1. Eight chains divide the search too shallowly

**Accepted risk.** The previous 1,000 proposals split eight ways meant only 125 proposals per chain before invalid moves, calibration and reevaluation. There was no evidence that this depth reached useful new cycles. It does not mathematically prove annealing cannot work, but it is insufficient support for the original convergence expectation.

Begin with two persistent chains per active profile and schedule across parties. Charge actual CPU time rather than fixed 16-fight candidates, and continue chain/cooling state across rounds. The pilot measures chain depth, accepted transitions and independently verified gain; one chain is an option if two remain shallow. Staged sampling is not assumed to produce a particular rejection rate or depth multiplier.

## 2. ER and cycles cannot be optimized in isolation

**Accepted.** Library weights were only local sensitivities under the old build and rotation. Deferring ER checks to spare capacity could reject a good cycle before discovering a viable allocation, or retain excess ER after an energy-saving change.

Search `(program, attainable substat allocation)` jointly. Support ER/damage-roll moves and bounded coupled repair when changing cycle structure. Run the baseline under the same allocation opportunities. Count every repair and comparison in discovery; distinguish cycle-only gain from combined stat/cycle gain. Overflow and waits are diagnostic hints, not an analytic solution for ER. Do not modify the player's actual stats at serving time.

## 3. Anchors need realistic support contexts and measurable acquisition

**Accepted in part.** Seventeen mandatory anchors with the other three members at C0 overemphasized one context. Under the latest scope, four-star C6 is mandatory in new discovery; five-star anchors use all-C1/all-C6 and mixed C1/C2/C4/C6. Library programs still seed discovery, while C0/C1 contrasts use an existing C0 reference without launching a C0 search. These conventional contexts are not measured user prevalence.

Replace the undefined notion of a profile being far from its true optimum with observable signals: independently checked progress per CPU time, leading-candidate uncertainty, benefit from profile-specific candidates and mechanic/energy coverage gaps. Maintain an exploration allocation so early flat results do not starve a party.

**Do not collapse all numeric-only constellations.** Changing damage distribution can change the best field time, action mix and buff allocation without changing a cooldown or energy mechanic. Mechanic tags prioritize search and share seeds; they are not equivalence proofs. Preserve comparisons within the owner's latest `4^n` scope. The earlier instruction to preserve 625 profiles is superseded by the explicit rarity/C0 decision, not by an assertion that numeric-only constellations are equivalent. C3/C5 reuse remains as directed.

## 4. A standard-build winner need not transfer to actual equipment

**Accepted risk; clarify the original archive.** The old plan already retained up to 16 shared programs per party. Its top-two rule described the most expensive reranking, not an explicit instruction to export only two programs. The output contract nevertheless lacked enough detail to prevent that implementation mistake.

Define up to six diverse candidates per profile including baseline, peak standard DPS and measured lower-ER/alternate-period or less proc-dependent options. Store programs by hash, expose actual evaluation depth and annotate dependencies. A signature-only batch cannot certify behavior with Favonius, Sacrificial or a swap-buff weapon it never tested. Actual-build evaluation and fallback remain necessary; diversity is a mitigation, not proof of transfer.

The same applies to mixed-C0 constellations: exact discovery-profile membership is not a serving gate. A nearby profile can supply a complete program, not its constellation bonuses or score. Reuse the existing up-to-six candidate slots rather than adding mixed-C0 discovery combinations or unbounded serving-time search. Failed transfer remains a reported coverage gap.

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
| Full-profile coverage under the new rarity/C0 scope | 250 |
| Reranking, final audits and export | 700 |
| Main phase allocation | **7,750** |
| Unallocated contingency | **7,302.8** |
| Effective planning allowance | **15,052.8** |

These caps incorporate the subsequent rarity/C0 decision: coverage and reranking allocations fall from 1,400/3,000 to 250/700 CPU-hours, and the difference returns to contingency. They are allocations, not measured costs or a quality guarantee. Pilot results must reserve the real cost of coverage before assigning discovery depth. Keep the one-week/$500 hard stop and disclose unresolved outcomes. A staged evaluator that promotes nearly every candidate, or ER repair that runs often, earns no assumed compute discount. The 88.97% profile-count reduction is not an 88.97% reduction in total adaptive-discovery CPU time.

## Pilot requirements before the full batch

- Compare annealing, improvement-only and random mutation by independently audited gain per CPU-hour on held-out cases.
- Measure validity rate, chain depth, racing promotions, rejected-candidate recovery and stability of best/archived programs.
- Compare fixed-ER and joint-ER discovery under identical total cost, including newly feasible and energy-saving cycles.
- Exercise mixed C1/C2/C4/C6 five-star constellations with four-stars fixed at C6; numeric-only tags must not remove profiles inside that declared scope. Keep C0 references separate from new discovery.
- Evaluate diverse candidate transfer on declared alternate builds, including proc/reset/swap-dependent weapon cases where available. Do not report untested equipment as compatible.
- Include C0 Furina/C6 Skirk among transfer cases inside the existing pilot allocation. Confirm that its C0 effects remain absent, compare against its actual baseline and distinguish candidate reuse from new C0 search.
- Measure total CPU cost and elapsed throughput on both VMs, including preparation, parsing, failed fights, retries, checkpointing and independent validation.

## Follow-up review: five remaining blind spots

A second owner-supplied review on 2026-11-04 identified the following gaps. The dispositions below are incorporated into the plan and preserve the same discovery scope and resource caps. They do not authorize implementation.

### 1. C0 on-field dealer with higher-constellation supports

**Accepted.** C0 Furina/C6 Skirk does not exercise the reverse transfer direction. An on-field dealer whose C1 changes resources or combo structure can have no good transferred cycle even when its supports' constellations make a better C0 cycle possible.

Reserve one of the existing six archive slots for a distinct C0-on-field-oriented candidate. Within existing nonzero discovery quanta, retain a baseline-derived stream preserving the C0 on-field combo and varying support sequence/period/timing. Keep that stream's diversity value separate from peak C1-source DPS, and probe selected candidates with only the on-field member at C0. No separate C0 optimization location or extra slot is added. A baseline-derived pattern and a single legal probe are still not proof of a good mixed-C0 cycle; missing transfer remains a gap. Test both directions in the existing pilot.

### 2. Short stages must not install noisy chain states

**Accepted clarification.** The earlier ladder did not explicitly prohibit early acceptance. Stages 2/4/8 can diagnose, prioritize or use a separately audited rejection heuristic, but cannot replace the chain state. Both current and proposed candidates must complete a common 16-fight decision block before acceptance, or a common larger block for an extended comparison. Keep the proposal's original random acceptance draw across the ladder.

Use comparable block means, not a short challenger against an incumbent's differently selected pooled history. Retain full history for reporting and archive checks separately. This fixes sample-size ambiguity; it does not remove stochastic selection bias or make 16 fights universally sufficient. All current-state refresh and promotion costs remain charged.

### 3. Artifact investment must remain fixed

**Accepted.** Freeze piece-level roll counts, roll-quality tiers, main stats/rarity/level and a CV ceiling no greater than 40, plus the character's initial useful-roll allocation budget. The first implementation exchanges only existing ER and allowed useful-damage roll tokens one for one. Transfer the quality tier as well as the token: one low-quality ER roll cannot become a maximum-quality crit roll.

Do not turn initially irrelevant rolls into new useful rolls, increase artifact investment, or redefine the accounting pool after observing new weights. An ER-to-crit exchange above the frozen CV ceiling is rejected or uses an allowed non-crit recipient. Actual marginal utility can change with the cycle; that is measured separately from the fixed accounting convention. Piece legality remains mandatory.

### 4. Static dependency inference and one-fight probes

**Accepted problem; qualify the proposed remedy.** General static GCSL inference cannot reliably attribute a second skill to a constellation, reset weapon, cooldown effect or guard. Use explicit authoritative restrictions and dynamic probe records rather than a new generic causal analyzer.

**Do not set `requires_c1_plus: true` from one C0 fight.** A pass is one observed legal run; a failure can depend on energy randomness, timing, equipment or interactions and does not establish a universal requirement. A legal run may simply wait and lose DPS. Lowering all five-stars at once cannot isolate the responsible character or interaction. Probe one member at a time with other source constellations held fixed, record the full context/seed/outcome and keep unknown/provisional status. Prioritize actual-user-build evaluation; no categorical gate is inferred from one seed alone.

Six single-member downgrade probes for every nominal profile and five-star coordinate total at most 6,611,040 fights under the census, or 22.04 CPU-hours at the provisional 12 ms/fight rate before overhead. This is not a new allowance: all probes, failures and repeats fit inside the existing 700-hour audit/export cap and replace other work there. Deduplicate exact probe identities. Do not substitute these diagnostics for independent performance validation.

### 5. Discovery capacity and coverage reuse

**Arithmetic accepted; completion inference rejected.** The 2,244 parties with zero to two five-stars have 26,595 profiles, and the full census has at most 101,027 distinct discovery locations under `min(32, 4^n)`. This corrects the old 146,240-location ceiling.

Having room for every profile in a party does not mean adaptive scheduling actually explored each one. Even a fully explored local profile has not necessarily compared all other anchors' archived programs. Therefore do not delete coverage for those parties wholesale or assert a fixed saving. Reuse exact program/allocation/profile/engine/data/conditions/horizon/seed evaluations, run missing comparisons, and retain untouched final audit seeds. Measure cache hits and actual activated locations before reducing the forecast. These rules remove exact duplicate work without confusing search, cross-program ranking and final validation.

### Follow-up pilot additions

- Exercise both C0-support/C6-dealer and C0-dealer/higher-constellation-support transfer within the existing pilot allocation and six-candidate limit.
- Check that every accepted chain transition used at least a comparable 16-fight block and that early stops cannot install 2/4/8-fight states.
- Check zero-sum roll/quality accounting, fixed useful-roll budgets and piece CV ceilings on every allocation mutation.
- Record probe passes, execution errors and timing degradation separately; audit false conclusions rather than treating one seed as a reliability test.
- Report eligible versus actually activated discovery locations, exact cache reuse and missing cross-archive comparisons separately from held-out final audits.

The relevant algorithm references are the [original simulated annealing paper](https://doi.org/10.1126/science.220.4598.671) and the authors' [iterated-racing paper and supplementary material](https://iridia.ulb.ac.be/supp/IridiaSupp2016-003/). They motivate acceptance and adaptive evaluation; they do not validate this domain-specific combination or promise a speedup for it.

This review changes documentation only. No application code, CI, deployment or cloud batch is added or run.
