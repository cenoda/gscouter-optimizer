# Baseline-free rotation discovery

The owner clarified that the optimizer must discover rotations for a new party,
not only edit a supplied rotation. This mode implements that discovery path.
It does not change the pinned simulator or the existing discovery eligibility
rules. The legacy baseline-derived path remains an explicit comparison.

## Experiment boundary

`search --from-scratch` reads a build manifest and never opens a baseline file.
`--baseline`, `--local-only` and `--restricted-domain` are incompatible with it.
Generated programs use no complete library cycle, party-specific sequence or
previous winner. Each of four initial families assigns a field opportunity to
a different member and shuffles visits. A generic starting prior exercises each
member's supported skill, two normals and a ready burst, with an optional setup
skill by the proposed field owner. Field patterns sample supported action forms
and durations. Mutations can remove or reorder every starting action and alter
repetition policies. Roles are hypotheses rather than fixed character labels.

Programs have separate opening and repeating visits. Each visit owns its
character, entry actions, optional timed field pattern and cycle period/phase.
Field patterns may instead execute a finite count of 1–8 repeats, with up to
12 commands. A finisher may include an explicit held-charge continuation before
release; distinct finishers need not share the same delay in a repeating field.
Action availability is checked against the pinned engine. The grammar covers
normal/charged attacks, skill, burst, dash and jump, plus bounded supported
parameter forms for hold, recast, final and buffering. Parameter kinds are
sampled before numeric values. A resource-guarded charged-dash continuation can
link a charged hit or finisher to the next charge, with colliding or hidden dash
hits. Recasts have resource/cooldown guards. Optional crystallize pickups and
action delays are searchable. These are per-character action capabilities;
no party-specific action order or published rotation is supplied.
Arbitrary character-specific status predicates remain outside this version.
Engine support checks do not prove dynamic prerequisites or good play.
The real simulator determines the outcome, including waits and failures.
`--basic-grammar` freezes an ablation to default action forms without parameters,
linked cancels, timing delays or pickups, including during final polishing.

Macro mutations insert, remove, relocate, duplicate, split or merge visits,
change field ownership and periods, and make compound edits. Micro mutations
change action order/counts, readiness policies and field lengths. A charged
fresh-generation proposal can restart a family after a completed common block,
even with lower fitness; this is logged as `restart`, not a greedy improvement.
The best discovered candidate remains separate. Family scheduling and the
bounded archive preserve opportunities for multiple structures, not every
structure ever visited. Only one chain executes at a time.

The last quarter of each requested CPU cap uses a reproducible bounded local
neighborhood around the discovery-selected peak. Initial generation, support
checks, proposals, failed fights, repairs, restarts and promotions consume the
same discovery CPU cap. Saved RNG, pending proposals, chain states and polish
cursor support continuation. No short-horizon or low-DPS pruning is added.
Fresh restarts are proposed with probability 1/64. Every thirteenth completed
proposal opportunity after initialization may instead mutate the peak found by
this run. That archive-seeded proposal is still compared with the current
family on a completed common block; no external cycle is used or installed.
Final polishing enumerates bounded action forms, linked cancels, repetition
counts and frame delays across command positions before structural edits.
It also tests action/visit removal and a paired removal/queued-burst change,
without presuming that removing an action improves damage. All candidates use
the same simulator comparison and budget.
Field loops wait one frame only when no action made progress, so a successful
cancel is not shifted by an unconditional extra frame.
Cyclic charge/dash links explicitly extend the charge while waiting for dash
cooldown; a bare wait would allow the engine to queue an unintended finisher.

Format-2 manifests may freeze explicit scoring conditions. The default remains
450 seconds excluding 150 seconds. An explicit window is emitted as
`# optimizer-warmup-seconds=...` driver metadata alongside the GCSL setup; it is
part of evaluation identity and cannot change during search. The worker checks
the declared duration, and reports/checkers use that duration. The engine itself
is unchanged.

The baseline fields in the compatible report schema refer to the first
generated starting program in this mode. Reports explicitly name that origin;
they are not comparisons against a library cycle. Keep an external good cycle
out of discovery and evaluate it only after selections are frozen. Independent
audit samples never guide discovery or polishing.

## Validation and next evidence

Check parser/engine support, full-fight execution, reachability of structural
edits, baseline-file independence and exact pending/RNG recovery. Local smoke
results establish execution, not broad search quality. Use fresh output
directories whenever implementation changes.

Before an effectiveness study, freeze the source, builds, budgets and seeds.
Use multiple genuinely held-out parties, repeated starts and equal CPU budgets;
report all failures and audit the search-selected result independently. Report
discovery and polish gains separately. Reference cycles are comparison-only
and are revealed after discovery. Full batch/cloud execution and an
unrestricted-optimality claim remain outside this implementation.
