# Rotation search

> Status: Agreed scope and budget; implementation proposal revised after design review, not started · Last updated: 2026-11-04

The owner commissioned the Rust port to make rotation search affordable. A rotation written for a C0 build can miss the best play at another constellation: Flins already changes at C1. Faster result pages are a benefit, but the next objective is to find better sustained-DPS rotations for the player's build, across the constellation stages below.

This document records the owner's decisions of 2026-11-04 and the proposed first implementation. Its authoritative home is now `cenoda/gscouter-optimizer`; it was moved from [gscouter-engine commit 84a53e4](https://github.com/cenoda/gscouter-engine/commit/84a53e45e91a1c9ce3cabaaf2cf2c7082283bc53). The owner requested repository creation and documentation migration only: implementation has not started, and no code or cloud batch is authorized by this migration.

The optimizer owns standard-build preparation, search, batch orchestration and result artifacts. [gscouter-engine](https://github.com/cenoda/gscouter-engine) owns simulation and engine verification; [Gscouter](https://github.com/cenoda/Gscouter) owns site integration and actual-user build scoring. The planned optimizer pins the engine library to an explicit commit, consumes versioned site-data snapshots and exports candidate GCSL with provenance. It does not duplicate the simulator or import Gscouter's private Go packages.

The [design review](design-review.md) supersedes the original fixed eight-chain/1,000-candidate allocation, ER checks deferred to reserve, all-C0-only anchor context and the associated 5.33-day forecast. The owner's equipment, constellation and resource constraints below remain unchanged. The revised algorithm and phase allocations are proposals to measure, not validated results.

This move does not change engine code, releases, team approvals or ongoing porting work. Gscouter's [cycle document](https://github.com/cenoda/Gscouter/blob/main/docs/cycle.md) records the fixed-rotation problem and the gains from existing alternatives; its [`internal/steady/steady.go`](https://github.com/cenoda/Gscouter/blob/main/internal/steady/steady.go) currently chooses rotations and fills waits rather than generating new action sequences.

## 1. Fixed scope

The first batch covers every library party, with each member independently at **C0, C1, C2, C4 or C6**. It is not limited to all-C0 and all-C6 parties, nor to five parties whose members all have the same constellation.

The owner's reason for excluding C3 and C5 is that they raise talent levels rather than introduce new abilities. They get no separate structural search: C3 uses the cycles found for C2, and C5 uses those for C4. Actual scoring still applies the player's real constellation and talent increases. This exclusion is already included in every cost estimate below; it buys no further reduction beyond `5^4` profiles per party.

The library snapshot inspected on 2026-11-04 (`Gscouter` checkout `e7d6d9a6`) contains 4,570 parties, including 4,332 marked supported and 238 draft. Thus the planning envelope is:

```
profiles per party = 5^4 = 625
all library profiles = 4,570 * 625 = 2,856,250
supported-party profiles = 4,332 * 625 = 2,707,500
```

The full count is a budget envelope, not a promotion of draft teams or a claim that every config already runs. Unsupported or invalid cases are reported explicitly. Freeze the library snapshot for a batch and record its commit.

Supported parties receive the initial coverage and discovery passes before draft parties. Keep all 238 draft parties and their 148,750 profiles in the batch inventory: process them afterward if the hard envelope permits, otherwise label them deferred. This changes scheduling priority, not the reported scope. Popularity alone cannot silently remove an eligible party.

Mechanic metadata may identify changes to energy, cooldowns, attack count, duration, action availability, field role and numeric damage. Use tags to prioritize anchors, seed candidates and explain dependencies. A numeric-only constellation can still change the best field-time or damage allocation, so tags do not justify collapsing C0/C1/C2/C4/C6 profiles. Only the owner's C3/C5 reuse rule is fixed; keep the 625-profile coverage count.

### Standard equipment

Offline discovery uses one standard equipment configuration per party member, with these rules:

| Item | Rule |
|---|---|
| Weapon | Use the character's signature weapon when available; otherwise choose the weapon with the highest team-DPS ceiling for that role and party. |
| Refinement | Four-star weapons R5; five-star weapons R1. |
| Artifact set | The most common setup for that role; start from the party's library set and check discrepancies. |
| Main stats | The applicable KQM character-guide recommendation for that role. |
| Substats | A high-investment, physically attainable allocation, using the existing artifact scoring system; see below. |

The weapon rule selects **one weapon**, not a signature/non-signature branch for every member. The budget has no additional `2^4` weapon multiplier. Record the selected weapon and its source; do not silently call the library weapon the highest-ceiling alternative. Likewise, library frequency is a proxy to inspect, not measured player equipment frequency.

Use [KQM character guides](https://keqingmains.com/misc/artifacts/) for main-stat recommendations. The owner's higher-investment substat rule below replaces the ordinary KQM substat budget; do not import that budget merely because main stats come from KQM. Record the chosen guide, role and main stats. A character without an applicable guide needs an explicit documented choice before its build enters the batch.

Prepare one reusable recommendation record per character/role, with contextual exceptions, rather than manually documenting 4,570 parties from scratch. Import existing library builds, normalized main stats and artifact weights; join them to a small sourced recommendation table and flag missing or conflicting entries. A library-derived main stat without a checked recommendation is provisional, not automatically KQM-compliant.

For a character without a signature weapon, source a bounded shortlist from guide and library data, then compare its weapons using multiple seed cycles and the same substat-allocation rules. Run one bounded cycle/build refinement of the shortlisted winners before selecting the single standard weapon. Charge preparation and any refinement to their phase budgets. Record the tested alternatives and conditional result: this chooses the best weapon evaluated under the declared convention, not a proof over every weapon's unknown optimal cycle. Unresolved weapon choices stay explicit rather than silently fixing a library weapon or starting an unbounded nested optimization.

### Substats

The agreed interpretation is approximately **40 CV per piece** for crit-oriented builds, with useful other substats in the remaining attainable allocation. When another substat is more valuable than crit, prioritize it even if CV falls. Do not force crit onto an EM, ER or other non-crit priority build.

Generate actual piece-level allocations: respect rarity, substat count, upgrade count, attainable values and the prohibition on repeating the main stat as a substat. Do not independently maximize CV and every other useful stat, or require an exactly 40 CV value if discrete rolls cannot produce it. Record the realized allocation, totals and artifact score.

Reuse the roll values, scoring convention and versioned weight data defined by Gscouter's [`internal/artifact`](https://github.com/cenoda/Gscouter/tree/main/internal/artifact) and [`internal/weights`](https://github.com/cenoda/Gscouter/tree/main/internal/weights). Exchange data through a documented export contract when implementation begins; a separate application cannot directly import those Go internal packages. Existing weights seed allocation but are measured on a different build and cycle. ER and damage-stat allocation are joint search variables, with their evaluation costs in the main discovery budget, as specified in section 3. They are not a reserve-only correction after rejecting new cycles.

Freeze levels, base talent levels, character parameters and initial resources in the standard-build manifest before the full batch. Constellation talent bonuses are applied by the engine. Starting resources are conditions, not parameters the optimizer may inflate.

## 2. Objective and search representation

Optimize expected team DPS under the site's current looped conditions: 450 s duration, 150 s warmup, and the same enemy, energy and swap settings as Gscouter. Use an unchanged full fight for candidate scoring; do not substitute the first cycle's damage for sustained DPS.

Search a small rotation program, compiled to GCSL:

```
opening
repeat:
    ordered actions and blocks
    action parameters and repetition counts
    always / when-ready / every-N-cycle guards
    explicit waits and delays
```

A repeated block may contain different successive cycles, so a burst every second cycle is representable. The engine determines actual action timing, swaps, cancellation, energy waits, cooldowns and resource use. Preserve startup separately from the repeated section. An observed action list from one seed is only a seed candidate, not a translation of all the original script's conditions.

Insertion can use all supported actions of all four members, not only actions present in the C0 rotation. This allows the search to introduce a new on-field role. Prefer typed action blocks learned from valid scripts or sourced character combos, with explicit prerequisites, parameter bounds and transition types. Retain a smaller action-level mutation path so a fixed combo dictionary does not exclude new C1 or other constellation behavior. Bound program size, repeats and waits, and reject execution errors. Static validation cannot prove dynamic energy or aura feasibility. A long wait is not by itself an invalid candidate; its effect belongs in the objective.

## 3. Algorithm: shared search with racing and joint ER allocation

The proposed implementation is a native Rust command in this repository, separate from the engine and serving path. Use multi-start simulated annealing with staged evaluation, persistent chains and a shared per-party archive. Compare it against random mutations and improvement-only search under equal measured CPU budgets. It remains an unvalidated finite-budget heuristic, with no global-optimum or standard annealing convergence claim.

### Joint candidate and bounded ER repair

A discovery state is `(program P, attainable substat allocation A)`. Weapon, refinement, set and selected main stats remain fixed by the standard-equipment rules. Moves can change P, move a substat roll between ER and a useful damage stat for one member, or change both together. Revalidate the individual pieces after every allocation move. Old weights suggest donors and recipients; fresh team DPS decides.

For a structural proposal that changes energy demand, permit bounded coupled repairs before discarding the program for energy starvation: test the unchanged allocation and at most two valid nearby allocations, initially transferring one or two rolls for one diagnosed member. The reverse transfer tests whether reduced energy demand frees useful damage rolls. It is a local repair, not a full nested optimizer; repeated allocation moves can continue later. Count every repair evaluation and any incumbent reevaluation against the same CPU budget. If repair cannot finish, retain a deferred program and its diagnostic instead of declaring the program intrinsically bad.

Energy waits and overflow suggest a direction; neither proves the best ER. Bursts can miss reaction or buff windows even without a long recorded wait. Evaluate full sustained damage after repair. Baselines receive the same allocation opportunities. Reports separate cycle-only gain at a fixed allocation from combined cycle-and-allocation gain. On an actual user's build, no hypothetical substat reallocation is applied.

### Anchors and adaptive allocation

Seed discovery from the library and alternatives, all-C0 and all-C6, and a context with four-star members at C6 and five-star members at C0. These are coverage contexts, not an asserted user-frequency distribution. Include explicit C0/C1 contrast cases for Flins and other known behavior changes, plus mixed-constellation interaction anchors. Deduplicate overlapping anchors and vary one member around realistic contexts rather than spending 17 mandatory slots on an otherwise all-C0 party.

Use up to 32 discovery locations per party as an initial planning ceiling, selected in rounds. Tags help choose locations but unknown tags do not block coverage. Begin with two persistent chains per active profile; parallelize across parties rather than dividing each profile into eight short chains. Measure valid proposals, accepted transitions, distinct programs, chain length and CPU time. Continue chain and cooling state across scheduling quanta. Reheating or restarting is explicit and charged, not automatic at every round. The pilot may choose one chain if two cannot reach useful depth.

Every eligible party gets an initial exploration quantum and every covered profile gets baseline/candidate comparisons. Additional quanta use observable acquisition signals, not unknown regret against a ground-truth optimum:

- confirmed incremental gain per CPU-second in recent quanta;
- uncertainty between leading candidates on independent reevaluations;
- gain from profile-specific candidates over the shared baseline candidate;
- uncovered mechanic combinations, unused newly available actions and energy/timing diagnostics.

Normalize signals using pilot data, record the scheduling decision and reserve an initial 20% of adaptive quanta for round-robin exploration of eligible underexplored parties. This exploration fraction is a pilot parameter, not a theorem. Low absolute DPS, popularity or no early improvement alone cannot prove a profile is solved. Draft parties follow supported ones; deferred work remains visible. Do not spend equal 32-by-1,000 quotas on every party regardless of measured progress or cost.

### Typed mutations and acceptance

Prefer insert/delete/move/replace operations on valid character action blocks; also allow supported parameter, repetition, readiness, burst-period, wait and delay changes. Retain action-level exploration and occasional compound edits to avoid fixing the search to existing C0 combos. Statically reject malformed or unsupported programs and impossible piece allocations before simulation. Dynamic prerequisite failures are checked by the engine, not assumed away by a combo label.

For current joint candidate x, proposal y, a fixed positive profile reference DPS D0 and temperature T, the ordinary annealing decision uses:

```
delta = (mean_DPS(y) - mean_DPS(x)) / D0
u = one uniform random draw for this proposal
accept when delta >= T * log(u)
```

The same u is retained while adding samples for this proposal; resampling u at every stage would change the acceptance rule. Full expected DPS is unknown, so the measured decision is approximate. Use paired seed indices and each candidate's actual sample mean and variability. Different programs need not consume the same random draws. Calibrate temperature from resampled candidate differences so pure simulation noise does not determine the initial scale. Keep chain position, provisional challengers and validated archive entries separate.

### Staged evaluation and archive promotion

Use a proposed full-horizon sample ladder of 2, 4, 8 and 16 fights; near ties may grow to 32 within budget. Cache reusable samples but charge all uncached current/proposal evaluations, ER variants and setup work. The first two stages diagnose failures and order work. A couple of low DPS samples alone are not enough to declare a promising program inferior. At later stages, use uncertainty in the paired difference relative to the annealing threshold to decide whether to spend more samples. Budget-limited comparisons are marked unresolved and can be deferred.

Small-sample intervals and repeated adaptive looks are heuristics, especially for heavy-tailed energy waits; do not label them formal 95% guarantees. A pilot may introduce a gross-inferiority rejection rule only after measuring false rejections, with random rejected-candidate audits and a recorded threshold. No assumed rejection fraction is used to promise extra search depth.

A challenger attempting to replace a validated best or enter the leading archive must face the relevant incumbent on a fresh shared seed block, initially 32 additional fights each. Update their means and sample counts, check stability, and promote only after this reevaluation. An unresolved difference retains both candidates if space permits. Periodically refresh retained incumbents and rotate search seed blocks so one lucky 16-fight estimate cannot become a permanent reference. The promotion stream is separate from search and from the final audit stream; it is still adaptively used and not an untouched final test.

The full-profile pass initially targets up to 16 shared programs with up to 8 fights each, then fresh-seed reevaluation of the leading two with up to 128 fights each. Those are adjustable comparison targets, not fixed costs on every candidate or an instruction to export only two programs. Preserve diversity candidates as described in section 5. Freeze the shortlist and allocation before the final held-out audit; do not feed that audit's seeds back into discovery. Disclose sampling uncertainty and selection effects rather than guaranteeing a noise-free winner.

This design borrows the budget-allocation idea from [iterated racing](https://iridia.ulb.ac.be/supp/IridiaSupp2016-003/) and the acceptance mechanism from [simulated annealing](https://doi.org/10.1126/science.220.4598.671); it is not an implementation of irace and inherits no unverified statistical guarantees from it.

### Full fights and early termination

Completed fitness evaluations retain the 450 s duration and 150 s warmup. Before that, exact parse/unsupported failures, engine execution errors, proven no-progress loops and measured work limits can terminate an evaluation. A work-limit hit means timed-out or unresolved, not low DPS. Record simulated frames and consumed CPU even for failed or partial runs.

A 60-90 s rollout can expose diagnostics and prioritize candidates, but it is not comparable to the sustained objective and is not a safe 70%-of-baseline pruning test. Startup, buff buildup and multi-cycle energy behavior can reverse the ranking; even one apparently dead wait need not prove permanent failure. Measure short/long ranking agreement on held-out pilots before enabling any heuristic horizon pruning, and audit rejected programs if used. Skipping it must remain a viable configuration.

## 4. One week and $500

The owner's hard resource envelope is **one week, $500 total, and two 64-core Fals v7 VMs**, with Spot permitted. Do not extend the deadline or add machines to compensate for overruns.

[Fals v7](https://learn.microsoft.com/en-us/azure/virtual-machines/sizes/compute-optimized/falsv7-series) maps each vCPU to a physical core. The nominal weekly capacity is:

```
128 cores * 168 h = 21,504 core-hours
planning allowance at 70% effective utilization = 15,052.8 CPU-hours
```

The 70% allowance covers interruptions, scheduling and other overhead for planning. It is an assumption, not a measured availability promise. The VM rate is billed for allocated time, not only useful simulation time.

### Throughput evidence, pilot and phase caps

Measured locally on 2026-11-04 from the `gscouter-engine` checkout, using its existing release binary, not rebuilt for this timing:

```
engine/target/release/gse configs/team-looped.txt 500 150 1
```

Two 500-fight runs used 5.986 and 5.958 CPU-seconds. The rounded 12 ms/fight result describes that config on the session machine only. It is neither a library mean nor Fals v7 throughput. The previous fixed plan cost 11,455.47 CPU-hours at 12 ms, but 19,092.44 CPU-hours at 20 ms; its 5.33-day completion forecast is superseded.

The pilot must include ordinary teams, dense reaction/event workloads, low-ER profiles, alternate burst periods, new-role programs, invalid transitions and bounded generated loops. Keep enemy conditions identical to the target objective; do not introduce a different multi-target workload merely to inflate the benchmark. Measure setup/parse cost, CPU per completed fight, partial/failed evaluation cost, p50/p95 latency, worker utilization, racing promotion rate, ER-repair frequency and incumbent-promotion overhead. Forecast with a measured workload-weighted mean and a separate tail allowance; p95 alone is not the total-cost estimator.

Use phase caps in CPU-hours, not a promised count of complete candidate evaluations. Initial allocation within the 15,052.8-hour planning allowance is:

| Phase | Initial CPU-hour cap | Includes |
|---|---:|---|
| Pilot and standard-build preparation | 600 | Data validation, bounded weapon comparisons, representative throughput and noise measurements. |
| Adaptive shared discovery | 6,200 | All chains, repairs, archive transfers, temperature calibration, incumbent reevaluations and failed proposals. |
| Full-profile coverage | 1,400 | Baselines, candidate comparisons, setup and any charged allocation checks. |
| Reranking, final audits and export | 3,000 | Leading and diversity candidates, independent samples, selective gcsim checks and result assembly. |
| Main phase allocation | **11,200** | A ceiling on planned work, not a fixed workload promise. |
| Unallocated contingency | **3,852.8** | Throughput variation, interruptions, uncertain results and explicit phase reallocation. |

At 12 ms, the original 16-program-by-8-fight coverage target alone costs 1,218.67 CPU-hours, and two-by-128-fight reranking costs 2,437.33; neither includes all setup or repair costs. At 20 ms they cost 2,031.11 and 4,062.22 respectively. Therefore the pilot must reserve the measured cost of minimum coverage and reporting before assigning remaining discovery time. Sample counts, shortlist sizes and quanta are tuned to the caps; any unresolved quality tradeoff is reported. No phase may silently consume the whole contingency.

The batch controller tracks measured CPU time including failed and partial work, remaining allocated wall time, Spot availability and projected billed dollars. Set per-program work limits from measured tails. Recompute the forecast after each round and redistribute explicitly. Do not imply that reducing samples makes the same quality free. One week and $500 remain hard stops, and unsupported or incomplete profiles remain visible in the report.

The constellation reduction is still `625 / 2401`, about 26% of the seven-stage profile count. There is no second C3/C5 discount. No extra profile reduction is assumed from numeric-only tags, unknown popularity or fast early DPS.

### Cost controls and Spot recovery

An Azure Retail Prices API lookup during planning returned Linux `F64als v7 Spot` in `westus2` at $0.715546 per VM-hour. At an unchanged rate, two VMs for 168 h cost about $240.42 in compute. This is a dated planning observation, not a reservation or capacity guarantee; [query the API](https://prices.azure.com/api/retail/prices?$filter=armSkuName%20eq%20%27Standard_F64als_v7%27%20and%20priceType%20eq%20%27Consumption%27) again at execution.

The proposed per-VM price ceiling is $1.30/h: two VMs for the full week have a compute ceiling of $436.80, leaving $63.20 of the $500 envelope for storage and other charges. Account for the actual applicable charges before allocation. Track accumulated and projected total spend and enforce a stop with margin; a cloud budget alert alone is not the worker's stop mechanism.

[Azure Spot](https://learn.microsoft.com/en-us/azure/virtual-machines/spot-vms) can evict for capacity as well as price, and advance notice is best effort. Persist completed work and checkpoints periodically, independently of eviction notification. Include program, profile, RNG state, seeds, sample counts, temperature, archive and consumed budget. Resume idempotent work units from external storage without rerunning completed slices. Keep worker parallelism within 64 single-core workers per VM; do not multiply that by an inner 64-thread simulation pool.

Use the first pilot to forecast remaining CPU time from the measured workload mix. Adjust discovery depth and uncertain-result allocations within the hard deadline and spending cap. If even minimum full-profile work no longer fits, report incomplete profiles and their reason rather than quietly dropping them or claiming completion. Outstanding porting work is an explicit dependency, not a reason to change other agents' branches.

## 5. Candidate artifacts and real-build transfer

Export up to six distinct useful candidates per profile, including the valid existing baseline: best measured standard-build DPS, lower-ER or reduced-burst-frequency alternatives, less proc-dependent programs, and different role or period variants where discovered. These are diversity targets, not proof that six useful behaviors exist for every profile. Do not discard the archive solely because only two candidates received the largest reranking budget. Mark each candidate's actual validation depth; a screened diversity candidate is not a confirmed winner.

Store programs once by content hash and let profiles reference them. The planned artifact record contains:

| Field group | Required information |
|---|---|
| Provenance | Program hash/GCSL, profile, data and engine commits, standard weapon/set/main stats and realized substat allocation. |
| Evaluation | Seed stream/block, sample count, mean and variability, simulated frames, CPU cost, baseline and fixed-allocation versus joint gain. |
| Behavior | Role, cycle period, burst frequency, energy/cooldown waits and measured ER tradeoff. |
| Dependencies | Weapon/proc, set/buff, action prerequisites and what was actually tested versus only inferred. |
| Status | Screened, promotion-checked, independently audited, uncertain, timed-out or failed, with reasons. |

A lower-ER label requires a measured alternative allocation or period, not merely fewer waits under a high-ER preset. A no-Favonius or no-Sacrificial compatibility claim requires evaluation on the declared alternate setup; the fixed standard-weapon batch cannot certify it from syntax alone. Identify possible weapon-specific loops, resets and swap buffs, retain robust existing alternatives where available, and mark untested transfer honestly.

The site evaluates stored programs using the player's actual constellation, weapon/refinement, artifact set, stats and talents. Never apply the archive's optimized substat allocation to the player, assume a missing set effect, or advertise standard-build DPS as the player's result. If archived programs fail or transfer poorly, retain the valid baseline/filler path and report that better transferred play is not established. Bounded serving-time local search is a separate proposed feature, not a promise that the fixed batch solves every weapon's rotation.

C3 uses C2's structural archive and C5 uses C4's, with actual talent increases in real scoring. No DPS interpolation is used. No serving flag, team approval or published library rotation changes as part of this documentation revision. Engine parity, search quality and real-build transfer are separately measured.

## 6. Deliverables and pilot decisions

1. A versioned standard-build manifest generated from existing data and reusable character/role recommendations, including missing/conflicting choices, tested weapon alternatives and valid piece allocations.
2. The bounded native search command, staged evaluator, diverse archive, full-profile comparison runner and resumable worker queue with CPU, time and spend accounting. These remain unimplemented.
3. A held-out pilot including Flins C0/C1, C6 four-star support contexts, interacting constellations, changed energy demand and actual-build transfer cases. Withhold known good alternatives from some starting pools. Compare algorithms by independently audited gain per CPU-hour, not raw candidate counts.
4. Pilot decisions on chain count/depth, valid-mutation rate, racing false rejections, incumbent stability, ER-repair cost, anchor acquisition and workload-weighted throughput. A failed racing or horizon heuristic can be disabled without changing the objective. Do not start the full claimed workload from the single-config 12 ms measurement.
5. Candidate coverage for eligible 625 profiles per party within the envelope, a diverse reusable archive and a report separating supported, draft, completed, audited, uncertain, failed, unsupported and deferred profiles. Report allocation and depth distributions, not just global averages.
6. Independent final comparisons of retained improvements and selective gcsim checks of newly reached behavior, charged to their phase. Finite-budget search finds the best evaluated programs and allocations, not proven optimal play.

The original document push and this repository's documentation migration are authorized directly to `main` with CI skipped by the owner. This repository remains documentation only; implementation and batch execution require a subsequent instruction.
