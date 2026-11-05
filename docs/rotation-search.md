# Rotation search

> Status: Agreed scope and budget; local pilot implementation authorized 2026-11-05. See [the recovery protocol](pilot-protocol.md) and implementation README for measured status. The broader batch design below remains a proposal.

The owner commissioned the Rust port to make rotation search affordable. A rotation written for a C0 build can miss the best play at another constellation: Flins already changes at C1. Faster result pages are a benefit, but the next objective is to find better sustained-DPS rotations for the player's build, across the constellation stages below.

This document records the owner's decisions of 2026-11-04 and the proposed first implementation. Its authoritative home is now `cenoda/gscouter-optimizer`; it was moved from [gscouter-engine commit 84a53e4](https://github.com/cenoda/gscouter-engine/commit/84a53e45e91a1c9ce3cabaaf2cf2c7082283bc53). The owner requested repository creation and documentation migration only: implementation has not started, and no code or cloud batch is authorized by this migration.

The optimizer owns standard-build preparation, search, batch orchestration and result artifacts. [gscouter-engine](https://github.com/cenoda/gscouter-engine) owns simulation and engine verification; [Gscouter](https://github.com/cenoda/Gscouter) owns site integration and actual-user build scoring. The planned optimizer pins the engine library to an explicit commit, consumes versioned site-data snapshots and exports candidate GCSL with provenance. It does not duplicate the simulator or import Gscouter's private Go packages.

The [design review](design-review.md) supersedes the original fixed eight-chain/1,000-candidate allocation, ER checks deferred to reserve, all-C0-only anchor context and the associated 5.33-day forecast. A subsequent owner decision narrows offline discovery: four-star members are fixed at C6 and five-star search stages are C1/C2/C4/C6. The owner then identified C0 Furina/C6 Skirk as a case that must still benefit: C0 is excluded from new batch search, not from candidate reuse. Equipment and hard resource constraints remain unchanged. The revised algorithm and phase allocations are proposals to measure, not validated results.

This move does not change engine code, releases, team approvals or ongoing porting work. Gscouter's [cycle document](https://github.com/cenoda/Gscouter/blob/main/docs/cycle.md) records the fixed-rotation problem and the gains from existing alternatives; its [`internal/steady/steady.go`](https://github.com/cenoda/Gscouter/blob/main/internal/steady/steady.go) currently chooses rotations and fills waits rather than generating new action sequences.

## 1. Fixed scope

The owner's latest rule is:

- **Four-star characters: C6 only** in offline discovery.
- **Five-star characters: independently C1, C2, C4 or C6** in new search.
- **C0: no new offline search profile.** Preserve the existing build and baseline, but allow cycles discovered elsewhere to compete on the actual C0 build. Do not force a C0 user back to only the original rotation.

The owner explicitly confirmed that only **4^n** combinations enter new search, where n is the party's five-star count. Mixed-C0 cases do not add discovery profiles either; do not interpret the rule as 5^n minus a single all-C0 profile. This constrains where the optimizer searches, not which users can reuse its programs. Existing cycles may seed searches or be reevaluated as baselines. An all-four-star party has one all-C6 profile (`4^0 = 1`).

The earlier interpretation that C0/mixed-C0 users must always stay on the old rotation is superseded. The motivating case is the owner's C0 Furina with C6 Skirk: retrieve suitable cycles discovered for the same party, including nearby C1 Furina/C6 Skirk profiles, and evaluate them at the actual C0/C6 build with its existing baseline. No extra offline C0 grid is required. The transfer and its remaining limitations are specified in section 5.

The owner's reason for excluding C3 and C5 is that they raise talent levels rather than introduce new abilities. Five-star C3 uses the cycles found for C2, and C5 uses those for C4, with no separate structural search. Actual scoring always applies the player's real constellation and talent increases; the offline C6 rule does not raise a real four-star character's constellation.

### Frozen party census

The library snapshot inspected on 2026-11-04 (`Gscouter` checkout `e7d6d9a6`) contains 4,570 parties, including 4,332 supported and 238 draft. Rarity comes from `internal/enka/data/character.dm.json`, cross-checked against generated `CharData.rarity` in the engine checkout `643af6c`; aliases are resolved through that checkout's generated `CHAR_NAMES`/`CHAR_KEYS` before counting. Traveler keys use their declared five-star rarity. Unknown keys or conflicting rarity values must be reported rather than guessed; this census found none after alias resolution.

| Five-star members n | All parties | Supported | Draft | New profiles per party | All new profiles |
|---|---:|---:|---:|---:|---:|
| 0 | 99 | 97 | 2 | 1 | 99 |
| 1 | 652 | 629 | 23 | 4 | 2,608 |
| 2 | 1,493 | 1,433 | 60 | 16 | 23,888 |
| 3 | 1,599 | 1,512 | 87 | 64 | 102,336 |
| 4 | 727 | 661 | 66 | 256 | 186,112 |
| Total | **4,570** | **4,332** | **238** | | **315,043** |

```
all new profiles = sum over parties (4 ^ five_star_count) = 315,043
supported-party new profiles = 291,525
draft-party new profiles = 23,518
previous five-stage-per-member envelope = 4,570 * 625 = 2,856,250
new / previous = 11.03% (88.97% fewer profiles)
```

These are rarity-based planning counts before invalid/unavailable-constellation and unsupported-config checks, not a claim that every generated profile is runnable. Record eligibility failures explicitly. Freeze the library and rarity/alias sources for a batch. Serving comparisons, mixed-C0 transfer audits and baseline reevaluations are not additional discovery profiles and must remain inside their existing CPU/candidate/sample budgets.

Supported parties receive the initial coverage and discovery passes before draft parties. Keep all 238 draft parties and their 23,518 new profiles in the inventory: process them afterward if the hard envelope permits, otherwise label them deferred. This changes priority, not the reported scope. Popularity alone cannot silently remove an eligible party.

Mechanic metadata may identify changes to energy, cooldowns, attack count, duration, action availability, field role and numeric damage. Use tags to prioritize anchors, seed candidates and explain dependencies. Numeric-only changes can still alter the best damage allocation; tags do not justify any further profile collapse within the owner's new 4^n scope.

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

The initial target is approximately **40 CV per piece** for crit-oriented builds, with useful other substats in the remaining attainable allocation. Freeze a realized per-piece CV ceiling in the standard-build manifest at no more than 40; if attainable discrete rolls fall below that target, use the recorded attainable ceiling. Do not raise it during search. When another substat is more valuable than crit, prioritize it even if CV falls. Do not force crit onto an EM, ER or other non-crit priority build.

Generate actual piece-level allocations: respect rarity, substat count, upgrade count, attainable values and the prohibition on repeating the main stat as a substat. Do not independently maximize CV and every other useful stat, or require an exactly 40 CV value if discrete rolls cannot produce it. Record the realized allocation, totals and artifact score.

Freeze the investment envelope before discovery: each piece's roll count, roll-quality distribution, rarity, level, main stat and CV ceiling, plus each character's number of rolls assigned to the declared useful-stat pool. A roll token has a quality tier; exchanging it uses the corresponding attainable value of the receiving stat, not an upgrade to that stat's maximum tier. For this first implementation, allocation moves are one-for-one `ER roll <-> allowed useful damage-stat roll` exchanges within that envelope. Leave initial irrelevant rolls unchanged; do not convert them into free useful rolls or redefine useful-roll accounting when weights change. Donor/recipient eligibility is recorded in the role manifest. Distinguish this fixed accounting convention from the actual marginal value measured by simulation.

If an ER-to-crit move exceeds the frozen CV ceiling, reject it or use another allowed useful recipient such as ATK%/HP%/EM; never inflate artifact investment. A reverse move can lower CV, and a later move may restore it up to the frozen ceiling. All moves still satisfy piece-level stat distinctness and main/substat exclusions. There is no extra free-roll pool, extra main-stat search or artifact upgrading hidden in `(P, A)`.

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

The proposed implementation is a native Rust command in this repository, separate from the engine and serving path. Use an outer budgeted competition between different play structures and inner multi-start simulated annealing with staged evaluation, persistent chains and a shared per-party archive. Imported programs may receive target-profile refinement within the same quotas. Compare the design against random mutations and improvement-only search under equal measured CPU budgets. It remains an unvalidated finite-budget heuristic, with no global-optimum or standard annealing convergence claim.

### Competing play structures before local tuning

Do not depend on a sequence of lucky local edits to discover a different way to play the party. Define a structure descriptor covering the intended on-field member, per-member burst period (for example every cycle versus every second cycle), short-swap versus longer-field blocks and concentrated versus distributed support windows. These are program-generation choices, not new equipment or constellation axes. Simulation establishes their realized timing and feasibility.

Generate an initially bounded set of **up to four distinct structure families** per selected profile, including the existing baseline structure and legal alternatives chosen for that party. Deduplicate identical programs/structures; fewer than four is valid when no other feasible seed is available. Do not instantiate the full Cartesian product of descriptors, four members, 32 profiles, structures and chain counts. Record which structures were attempted, why a seed failed and which were not explored. Existing scripts and typed blocks seed families, but generation must allow a new role or burst period not present in the original C0 script.

Give each selected valid family a small initial local-optimization quantum before allocating most remaining CPU to promising families. A weak initial seed alone does not show the family is bad. Keep **two active chain slots per profile in total**, scheduled across families; paused family states retain their program, allocation, temperature and accounting. Do not create two fresh chains for every family. The existing exploration allocation covers underexplored parties and structure families rather than adding another independent budget fraction.

Within a family, tune support/action order, combo repetitions, waits and the frozen ER allocation. Keep occasional structure-changing proposals and explicit restarts alive; a family label is not a permanent restriction on legal programs. Relabel a program when its intended structure changes and record observed behavior without counting it as a new constellation location. Families share verified programs through the archive, while archive diversity and the C0-on-field slot prevent peak source DPS alone from deleting every alternative. Compare retained finalists using the same target conditions and independent seeds.

### Joint candidate and bounded ER repair

A discovery state is `(program P, attainable substat allocation A)`. Weapon, refinement, set and selected main stats remain fixed by the standard-equipment rules. Moves can change P, exchange an existing ER/useful-damage roll token for one member under the frozen envelope in section 1, or change both together. Revalidate the individual pieces after every allocation move. Old weights suggest donors and recipients; fresh team DPS decides. Neither physical roll quality/count nor the frozen useful-roll budget/CV ceiling can increase.

For a structural proposal that changes energy demand, permit bounded coupled repairs before discarding the program for energy starvation: test the unchanged allocation and at most two valid nearby allocations, initially transferring one or two rolls for one diagnosed member. The reverse transfer tests whether reduced energy demand frees useful damage rolls. It is a local repair, not a full nested optimizer; repeated allocation moves can continue later. Count every repair evaluation and any incumbent reevaluation against the same CPU budget. If repair cannot finish, retain a deferred program and its diagnostic instead of declaring the program intrinsically bad.

Energy waits and overflow suggest a direction; neither proves the best ER. Bursts can miss reaction or buff windows even without a long recorded wait. Evaluate full sustained damage after repair. Baselines receive the same allocation opportunities. Reports separate cycle-only gain at a fixed allocation from combined cycle-and-allocation gain. On an actual user's build, no hypothetical substat reallocation is applied.

### Anchors and adaptive allocation

Use the library and alternatives as seed programs. New discovery anchors fix every four-star at C6 and start five-star contexts at all-C1, all-C6 and selected mixed C1/C2/C4/C6 combinations. Vary one five-star member around these contexts to isolate changes, and add interaction anchors. Existing C0 programs/results are references for cases such as Flins C0/C1, not extra discovery anchors or a reason to standardize C0's build. These contexts are not an asserted user-frequency distribution.

Use up to `min(32, 4^n)` distinct discovery locations per party as an initial planning ceiling, selected in rounds. Never duplicate an anchor merely to fill 32 slots. Tags help choose locations but unknown tags do not block coverage. Begin with two persistent chains per active profile; parallelize across parties rather than dividing each profile into eight short chains. Measure valid proposals, accepted transitions, distinct programs, chain length and CPU time. Continue chain and cooling state across scheduling quanta. Reheating or restarting is explicit and charged, not automatic at every round. The pilot may choose one chain if two cannot reach useful depth.

The census implies a ceiling of **101,027 distinct discovery locations**, not the old `4,570 * 32 = 146,240`. The **2,244 parties with n <= 2** contain **26,595 profiles** and could put every profile at a discovery location because their per-party grids have 1, 4 or 16 entries. This is a capacity fact, not a claim that adaptive scheduling has already explored all those locations or completed all candidate comparisons. Record actual activated locations and completion states.

Discovery and coverage use one evaluation cache. Reuse an existing sample only when program, allocation, party/profile, engine and input revisions, conditions, horizon/warmup and seed/stream identity all match. Do not rerun an identical discovery sample merely to label it coverage; evaluate missing cross-archive program/profile pairs and required additional samples instead. A discovered local winner does not establish that another anchor's program is worse. Final held-out audit samples remain independent and cannot be replaced by training samples. No numerical coverage saving is assumed before cache-hit rates are measured.

### Transfer followed by selected target-profile refinement

Candidate sharing is more than copying a source winner and stopping after reranking. For an eligible C1/C2/C4/C6 target profile, use the flow **retrieve -> evaluate at target -> select an adaptation quantum -> mutate at target -> independently compare -> share back**. Initialize its allocation under the same frozen investment envelope, rather than importing a source profile's ER assumptions or DPS. Count target repairs, current-state refreshes, seed preparation and archive checks as ordinary discovery work.

Refinement is selective. A target promoted from coverage to local optimization consumes one of the party's `min(32, 4^n)` distinct discovery locations; plan initial anchors and remaining promotion slots together. If every location is already used, a new location does not become free merely because its seed came from the archive. Reallocate depth among admitted locations or report that the target was only compared, not refined. Preserve actual activated-location counts and the 101,027-location aggregate ceiling. No extra optimization pass is silently appended to all 315,043 profiles.

Choose adaptation opportunities from the same observable progress, uncertainty and mechanic-coverage signals used by the scheduler. Compare import-only versus import-and-refine at equal CPU cost in the pilot. This proposal does not override the C0 rule: mixed-C0 builds receive candidate comparison and probes within their existing caps, not new target-profile annealing. Record parent program/source profile, target profile, allocated CPU, actual mutations and independently measured incremental gain so transfer benefit and adaptation benefit are distinguishable.

Every eligible party gets an initial exploration quantum and every covered profile gets baseline/candidate comparisons. Additional quanta use observable acquisition signals, not unknown regret against a ground-truth optimum:

- confirmed incremental gain per CPU-second in recent quanta;
- uncertainty between leading candidates on independent reevaluations;
- gain from profile-specific candidates over the shared baseline candidate;
- uncovered mechanic combinations, unused newly available actions and energy/timing diagnostics.

Normalize signals using pilot data, record the scheduling decision and reserve an initial 20% of adaptive quanta for exploration of eligible underexplored parties and structure families. This exploration fraction is a pilot parameter, not a theorem. Low absolute DPS, popularity or no early improvement alone cannot prove a profile is solved. Draft parties follow supported ones; deferred work remains visible. Do not spend equal 32-by-1,000 quotas on every party regardless of measured progress or cost. Structure seeding and target adaptation share this allocation and the 6,200-CPU-hour discovery cap; they are not additional multipliers.

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

Use a proposed full-horizon sample ladder of 2, 4, 8 and 16 fights; near ties may grow to 32 within budget. Cache reusable samples but charge all uncached current/proposal evaluations, ER variants and setup work. **No acceptance at 2, 4 or 8 fights:** those stages diagnose failures, order work and permit only the separately audited early-rejection heuristic. A couple of low DPS samples alone are not enough to declare a promising program inferior. Budget-limited comparisons are marked unresolved and can be deferred.

Before `x <- y`, both x and y must have a complete **common 16-fight decision block** under the same profile and conditions, or a common larger block if this comparison grows. Use those block means for the acceptance decision, not a two-fight challenger against a pooled 48-fight incumbent from different seeds. The block is shared by seed index, not by guaranteed identical random draws. An accepted y carries the completed block into the next step; retain its full history separately. Cache any matching current-state samples. Rotate blocks and refresh the current state within budget; if no valid reference block is available, defer rather than substitute a stale short mean. Equal minimum sample size limits one source of bias but does not remove adaptive-selection bias.

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
| Full-profile coverage | 250 | New-profile baselines, candidate comparisons, setup and any charged allocation checks. |
| Reranking, final audits and export | 700 | Leading and diversity candidates, independent samples, selective gcsim checks and result assembly. |
| Main phase allocation | **7,750** | A ceiling on planned work, not a fixed workload promise. |
| Unallocated contingency | **7,302.8** | Throughput variation, interruptions, uncertain results and explicit phase reallocation. |

At the new 315,043-profile count, 16-program-by-8-fight coverage costs 134.42 CPU-hours at 12 ms/fight, and two-by-128-fight reranking costs 268.84. At 20 ms they cost 224.03 and 448.06 respectively. These are arithmetic targets, excluding setup, extra baseline work and repair costs. The reduced phase caps are provisional; the pilot must reserve the measured cost of minimum coverage and reporting before assigning discovery time. Sample counts, shortlist sizes and quanta are tuned to the caps; unresolved quality tradeoffs are reported. No phase may silently consume the whole contingency.

The batch controller tracks measured CPU time including failed and partial work, remaining allocated wall time, Spot availability and projected billed dollars. Set per-program work limits from measured tails. Recompute the forecast after each round and redistribute explicitly. Do not imply that reducing samples makes the same quality free. One week and $500 remain hard stops, and unsupported or incomplete profiles remain visible in the report.

The owner's new rarity/C0 rule reduces the previous five-stage profile count by 88.97%. That reduction applies directly to full-profile passes, not automatically to joint discovery, preparation or audits. The discovery allocation is a ceiling, not a requirement to spend every freed hour; finish early when the measured objectives are met. No further reduction is assumed from numeric-only tags, popularity or fast early DPS, and C3/C5 must not be discounted twice.

### Cost controls and Spot recovery

An Azure Retail Prices API lookup during planning returned Linux `F64als v7 Spot` in `westus2` at $0.715546 per VM-hour. At an unchanged rate, two VMs for 168 h cost about $240.42 in compute. This is a dated planning observation, not a reservation or capacity guarantee; [query the API](https://prices.azure.com/api/retail/prices?$filter=armSkuName%20eq%20%27Standard_F64als_v7%27%20and%20priceType%20eq%20%27Consumption%27) again at execution.

The proposed per-VM price ceiling is $1.30/h: two VMs for the full week have a compute ceiling of $436.80, leaving $63.20 of the $500 envelope for storage and other charges. Account for the actual applicable charges before allocation. Track accumulated and projected total spend and enforce a stop with margin; a cloud budget alert alone is not the worker's stop mechanism.

[Azure Spot](https://learn.microsoft.com/en-us/azure/virtual-machines/spot-vms) can evict for capacity as well as price, and advance notice is best effort. Persist completed work and checkpoints periodically, independently of eviction notification. Include program, allocation, profile, structure-family/paused-chain state, transfer lineage, RNG state, seeds, sample counts, temperature, archive, activated locations and consumed budget. Resume idempotent work units from external storage without rerunning completed slices. Keep worker parallelism within 64 single-core workers per VM; do not multiply that by an inner 64-thread simulation pool.

Use the first pilot to forecast remaining CPU time from the measured workload mix. Adjust discovery depth and uncertain-result allocations within the hard deadline and spending cap. If even minimum full-profile work no longer fits, report incomplete profiles and their reason rather than quietly dropping them or claiming completion. Outstanding porting work is an explicit dependency, not a reason to change other agents' branches.

## 5. Candidate artifacts and real-build transfer

Export up to six distinct useful candidates per profile, including the valid existing baseline: best measured standard-build DPS, lower-ER or reduced-burst-frequency alternatives, less proc-dependent programs, and different role or period variants where discovered. These are diversity targets, not proof that six useful behaviors exist for every profile. Do not discard the archive solely because only two candidates received the largest reranking budget. Mark each candidate's actual validation depth; a screened diversity candidate is not a confirmed winner.

### C0-on-field transfer slot

The reverse of the owner's original example is a required pilot case too: **C0 on-field damage dealer plus higher-constellation five-star supports**. If the dealer's C1 changes the combo or resource system, nearby C1 programs can all transfer poorly. Reserve one of the existing six slots for a distinct candidate oriented toward that C0 on-field context, alongside the ordinary baseline and peak standard-DPS candidate; deduplicate if one program fills several roles. Do not add a seventh slot or invent a compatible winner when none is found.

Within existing nonzero-profile discovery quanta, retain a baseline-derived mutation stream that preserves a known C0 on-field action/combo structure while varying support sequence, burst period and timing. Treat its transfer value as a diversity objective so an inferior C1-profile DPS alone does not evict it. Charge this stream to the same discovery budget and evaluate its source states only at existing `4^n` locations. Probe selected retained candidates with the on-field member at C0 and the supports' source constellations unchanged, as described below. This does not launch a separately optimized C0 profile or guarantee optimal C0 play; it improves the chance of retaining a usable program for that missing context. If no valid C0 baseline or transferable improvement exists, record the gap explicitly.

Store programs once by content hash and let profiles reference them. The planned artifact record contains:

| Field group | Required information |
|---|---|
| Provenance | Program hash/GCSL, profile, data and engine commits, standard weapon/set/main stats, realized substat allocation, source/parent program/profile and target-refinement lineage/CPU. |
| Evaluation | Seed stream/block, sample count, mean and variability, simulated frames, CPU cost, baseline and fixed-allocation versus joint gain. |
| Behavior | Intended structure family and observed role, cycle period, burst frequency, field/swap pattern, energy/cooldown waits and measured ER tradeoff. |
| Dependencies | Documented action prerequisites and context-specific probe results: source/target constellation vector, changed member, equipment/allocation, engine/data revisions, seeds/count, completion/error and action/wait diagnostics. Separate observations, unknowns and authoritative restrictions. |
| Status | Source-only, target-evaluated or target-refined lineage; screened, promotion-checked, independently audited, uncertain, timed-out or failed evaluation, with reasons. |

A lower-ER label requires a measured alternative allocation or period, not merely fewer waits under a high-ER preset. A no-Favonius or no-Sacrificial compatibility claim requires evaluation on the declared alternate setup; the fixed standard-weapon batch cannot certify it from syntax alone. Identify possible weapon-specific loops, resets and swap buffs, retain robust existing alternatives where available, and mark untested transfer honestly.

### Compatibility probes are observations, not inferred requirements

Do not build a general static GCSL analyzer that guesses whether two close skills came from C1 charges, a weapon reset, cooldown reduction or a guard. Hard exclusion is limited to an explicit authoritative restriction; the source constellation vector alone is not one. Use bounded simulation probes to investigate dynamic behavior.

For selected final programs, run a full-horizon diagnostic with **one target five-star member changed to C0**, keeping the other members' source constellations and standard equipment/allocation fixed. Compare with a cached matching source-seed trace where available. Prioritize the C0 on-field case and the actual C0 Furina/C6 Skirk case. A single all-five-stars-at-C0 probe can additionally record a combined context, but cannot identify which member or interaction caused a difference.

One fight records an observed completion, execution error, waits, missed/changed actions and simulated frames for that exact context and seed. Cooldown or energy loss can stretch a legal program without raising an execution error. A pass does not establish reliability or good DPS across seeds; a fail does not by itself establish `requires_c1_plus: true`, identify a causal constellation or certify incompatibility on every other mixed build. Keep status unknown/provisional, use the result for retrieval priority, and require actual-user-build evaluation before final choice. Repeated evidence or an authoritative action restriction can support a narrower documented rule; never turn a probabilistic one-seed failure into a universal serving gate.

Deduplicate probes by their full evaluation identity and charge every run to the existing 700-CPU-hour audit/export cap. Probing six programs once for every one-member C0 downgrade over all nominal profiles would be at most **6,611,040 fights**, about **22.04 CPU-hours at the unvalidated 12 ms rate**, before setup and retries. This is a planning upper bound, not mandatory extra work: probes replace other work inside the cap, and no new CPU allowance or discovery dimension is added. Probe counts and uncertainty are explicit; final independent performance audits are not silently relabeled as one-fight compatibility checks.

The site evaluates applicable stored programs using the player's actual constellation, weapon/refinement, artifact set, stats and talents. Never apply the archive's optimized substat allocation or offline four-star C6 assumption to the player, assume a missing set effect, or advertise standard-build DPS as the player's result. C0 and mixed-C0 combinations are eligible for this bounded candidate reuse even though they are outside new batch discovery. If archived programs fail or transfer poorly, retain the valid baseline/filler path and report that better transferred play is not established.

### Mixed-C0 retrieval without a new search dimension

1. Always retain the existing baseline as a comparison candidate. Identify the party's deduplicated archive independently of an exact constellation-key match; a missing C0 key must not end retrieval.
2. For retrieval only, a five-star C0 coordinate can initially query nearby C1 discovery profiles; retain the other members' actual structural stages, including C6. C3/C5 use the preceding stages as already defined. This is a way to find programs, not a claim that C0 and C1 are mechanically equivalent. Other archive diversity candidates can occupy the remaining slots.
3. Rank retrieval candidates using explicit documented restrictions, context-specific probe evidence, matching on-field role/period and lower-ER or less proc-dependent diversity. A program discovered at C1 does not automatically require C1; neither a static guess about skill spacing nor one C0 probe establishes that requirement. Missing numeric bonuses or different energy balance are evaluated on the target build. Do not borrow source DPS as the target score. Unknown compatibility is disclosed and checked by actual simulation.
4. Reuse complete programs without blindly deleting presumed C1-only actions or splicing blocks from unrelated cycles. Exclude a program only when an authoritative hard prerequisite is absent; otherwise use probe results as priority/diagnostic evidence and evaluate candidates on the exact user build within the shortlist cap. A legal program can still have poor buff or energy behavior; only that evaluation establishes its target result.
5. Select the best independently confirmed result within the planned comparison budget. Keep the baseline when no improvement is established, and disclose that a newly searched optimum for this mixed-C0 build is not available.

For C0 Furina/C6 Skirk, this means comparing the baseline with retrieved C6-Skirk programs, including those found with C1 Furina, while all simulations still use C0 Furina and the user's real weapon, artifacts and talents. Missing C1 effects are never granted. This addresses automatic fallback to a C0-style cycle, but does not prove that a transferred cycle beats the baseline before it is measured.

### Unchanged computation limits

Offline discovery remains 315,043 nominal profiles and retains the same phase caps. Transfer audits use selected cases in the already allocated pilot/audit pools, not another Cartesian C0 sweep. At serving time, mixed-C0 candidates replace slots inside the planned shortlist of up to six programs including baseline, with unchanged aggregate fight and CPU limits. Errors and confirmations consume that same limit. Do not append a second search or enlarge the pool just because a C0 coordinate is present.

Candidate comparison has a real cost: this is not zero-cost universal optimization or a claim of identical latency to the site's old baseline-only path. It introduces no larger limit than the already planned bounded candidate-selection path, whose latency still needs measurement. This revision does not authorize serving-time annealing; any later local-search feature needs a separate explicit budget decision. An archive with no transferable candidate leaves a genuine coverage gap, recorded rather than hidden by a fake C1 simulation.

For five-star members, C3 uses C2's structural archive and C5 uses C4's, with actual talent increases in real scoring. No DPS interpolation is used. No serving flag, team approval or published library rotation changes as part of this documentation revision. Engine parity, search quality and real-build transfer are separately measured.

## 6. Deliverables and pilot decisions

1. A versioned standard-build manifest generated from existing data and reusable character/role recommendations, including missing/conflicting choices, tested weapon alternatives and valid piece allocations.
2. The bounded native search command with structure-family scheduling and selected target-profile refinement, staged evaluator, diverse archive, full-profile comparison runner and resumable worker queue with CPU, time and spend accounting. These remain unimplemented.
3. A held-out pilot including the existing Flins C0 reference versus new C1 search, the owner's C0 Furina/C6 Skirk case and the reverse C0 on-field dealer/higher-constellation support case, C6 four-star contexts, mixed C1/C2/C4/C6 five-star constellations and changed energy demand. Reuse the planned pilot allocation; do not turn C0 transfer checks into another discovery grid or add a new phase. Withhold known good alternatives from some starting pools. Compare algorithms by independently audited gain per CPU-hour, not raw candidate counts.
4. Pilot decisions on chain count/depth, structure discovery and family allocation, target-refinement value, valid-mutation rate, 16-fight acceptance-block enforcement, racing false rejections, incumbent stability, frozen-allocation invariants, ER-repair cost, anchor acquisition, compatibility-probe false conclusions, cache reuse and workload-weighted throughput. A failed racing or horizon heuristic can be disabled without changing the objective. Do not start the full claimed workload from the single-config 12 ms measurement.
5. Candidate coverage for eligible `4^n` new profiles per party within the envelope, a diverse reusable archive and a report separating supported, draft, completed, audited, uncertain, failed, unsupported and deferred profiles. Distinguish profiles searched directly from mixed-C0 cases only tested for transfer and from cases not tested. Report allocation and depth distributions, not just global averages.
6. Independent final comparisons of retained improvements and selective gcsim checks of newly reached behavior, charged to their phase. Finite-budget search finds the best evaluated programs and allocations, not proven optimal play.

### Discovery-quality pilot before expansion

Select a small declared pilot suite before tuning. Include role-changing, burst-period, energy-starved and mixed-constellation cases plus less complex controls. Keep development and held-out cases distinct and record the full preparation/tuning CPU cost. The 600-hour preparation/pilot allocation and existing discovery/audit caps pay for this work; do not budget a full 4,570-party run before learning whether candidate generation works.

| Experiment | Evidence required |
|---|---|
| Hide known strong full cycles from seeds and archives | Can the generator recover similar or better behavior without copying that complete cycle? Keep hidden cycles out of tuning and seed construction; report any remaining sourced block overlap. |
| Increasing measured CPU budgets | Independently audited best-DPS-versus-CPU curves; discovery of new structures versus only parameter tuning. A flat curve is not an upper bound on optimal DPS. |
| Repeated starts and RNG choices | Distribution of audited outcomes, failures and structure choices; no best-run-only reporting. Runs stuck at the same inferior structure can still look repeatable. |
| Equal-cost ablations | Compare local-only versus structure-family search and import-only versus target refinement, with the same total CPU including evaluations, repairs and promotions. |
| Small explicitly enumerated domain | Declare the finite valid program/allocation grammar, enumerate it and measure recovery of its reference maximum. The maximum is restricted to that domain and fixed sample objective; uncertain expected-DPS rankings remain uncertain. |
| Actual-equipment diagnostic | On selected already eligible non-C0 profiles, compare archived-candidate selection with bounded program-only refinement at the real equipment and fixed actual stats, with the same CPU accounting. Do not exchange hypothetical artifact rolls in this experiment. Measure residual transfer loss; do not silently assume it is zero. |

The enumerated reference does not bound the unrestricted GCSL optimum, and consistent restarts or diminishing improvements do not establish a global optimality gap. Publish the limitations and confidence of the reference comparisons. Use the pilot to predeclare tolerances, recovery targets and acceptable failure rates for the held-out cases before the expansion decision; avoid picking a success threshold after seeing the held-out result. If discovery is poor, revise the generator within the budget instead of spending more cores on the same failure. If adequate evidence or time is absent, report a pilot-only or deferred outcome rather than claim the full batch is complete.

Serving-time optimization on actual user gear remains a separate possible feature requiring its own explicit latency and compute decision. The pilot diagnostic is offline and bounded, uses only already eligible discovery profiles and does not introduce an extra C0 or equipment Cartesian sweep. C0 and mixed-C0 pilots retain their existing baseline/transfer/probe role; no C0 search or request-time annealing is introduced by this review.

The original document push and this repository's documentation migration are authorized directly to `main` with CI skipped by the owner. This repository remains documentation only; implementation and batch execution require a subsequent instruction.
