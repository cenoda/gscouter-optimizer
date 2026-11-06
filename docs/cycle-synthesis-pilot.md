# Cycle-synthesis pilot (proposed)

**Status: research note and proposed protocol, written 2026-11-06 before any
cycle-synthesis code exists. Nothing in sections 4–9 is a measured result.**
Section 2 reports small diagnostic measurements made for this note on
development cases only (their A had already been read); they are evidence
about where the gap is, not about whether the proposed method works. The
owner asked for documentation only; no `src/` or workflow file is changed by
this note. Supporting scripts and JSON:
[`reports/cycle-synthesis-research-2026-11-06/`](../reports/cycle-synthesis-research-2026-11-06/).

Labels used below: **[measured]** (a number produced by a committed script
or report), **[estimate]** (arithmetic from measured rates), **[hypothesis]**
(what the pilot is meant to test).

## 1. Why a new pilot

The owner's question: instead of editing the library rotation L (local edits,
annealing) and then spending more time (2× CPU or a second pass), can the
optimizer write a new cycle from scratch or restructure L wholesale?

What the earlier runs established (selected start, informative cases, S/A on
identical audit fights; sources: [v2 report](../reports/heldout-recovery-v2-2026-11-05/README.md),
[v2.1 report](../reports/heldout-recovery-v2-2026-11-05/v2.1/README.md),
recomputed by [`sa_summary.py`](../reports/cycle-synthesis-research-2026-11-06/sa_summary.py)
into `sa_summary.json`) **[measured]**:

| Run, arm | All (n=32): median S/A, cases < 0.90 | A/L < 1.25 (n=17) | A/L ≥ 2 (n=10) |
|---|---|---|---|
| v2 V1@192 | 0.974, 8 | 0.978, 3 | 0.917, 5 |
| v2 V2S@384 | 0.990, 5 | 1.008, 1 | 0.926, 4 |
| v2 V2T@384 | 0.997, 5 | 1.011, 1 | 0.909, 4 |
| v2.1 V2S@384 | 0.998, 7 | 1.017, 1 | 0.884, 6 |
| v2.1 V2T@384 | 0.981, 8 | 1.000, 2 | 0.922, 5 |

- Where L is a reasonable rotation for the build (A/L < 1.25), the existing
  search already reaches A (median ≈ 1.0).
- The cases that stay below 90% of A in every arm are concentrated where L is
  badly matched to the build (A/L ≥ 2): 4–6 of those 10 cases per arm.
- More local search does not close them: the second pass beats equal-CPU
  continuation by about 1% of L start-averaged (v2 +1.21%, v2.1 +0.97%), the
  selected-start difference is null in both runs (+0.29%, −0.20%), and
  doubling CPU (V2S@192 → @384) moves the A/L ≥ 2 median by 0–4 points
  (v2 0.882 → 0.926, v2.1 0.883 → 0.884).

So the target of this pilot is narrow: **the broken-L stratum**, where the
remaining gap looks like a different cycle rather than a better-tuned L.

## 2. Failure-structure analysis (development cases)

Cases: the six v2 held-out cases (now development) below 0.90 A in v2.1
V2S@384 (`mavuika-chevreuse-ororon-bennett`, `arlecchino-bennett-citlali-xilonen`,
`cyno-furina-nahida-xilonen`, `mualani-candace-sucrose-xiangling`,
`neuvillette-furina-citlali-xilonen`, `mualani-mona-xilonen-mavuika`), four
contrast cases also with A/L ≥ 2 that v2.1 did solve (≥ 0.94 A), and the three
stage-1 "cycle restructure" cases (Clorinde, Flins–Sucrose, Mualani–Chiori),
which v2 development already brought to 0.93–1.08 A
(`reports/heldout-recovery-v2-2026-11-05/development/experiments.json`).

Method: every program was measured natively (`gso measure`, pinned engine
`1ba0af1`, the stage-1/v2 conditions: 450 s, 150 s warmup, level-100 target,
swap delay 12, periodic energy) on the **same 32 fights** — the first 32 audit
fights of the selected v2.1 V2S@384 start — so L, S and A rows are paired.
32-fight means are diagnostics, not audits. Full table:
[`tables.md`](../reports/cycle-synthesis-research-2026-11-06/tables.md);
per-member DPS and waits: `lsa.json`. All of section 2 cost about 0.6 CPU-h
locally (accounting in section 8).

### Findings [measured, development cases]

**F1. A broken L is a blocked L.** In every A/L ≥ 2 case, L spends a large
part of the 300 s scoring window waiting on one member's resource: 44–250 s
`insufficient_energy` for a support's unconditional burst (eight cases) or
109 s `skill_cd` for the on-field member (Cyno). A at the same build waits
≤ 10 s. The engine's per-member wait reasons (`gse::api::Iteration.waits`)
expose this directly.

**F2. Unblocking is necessary, not sufficient — and the search already does
it.** One generic transform of L, "put every burst behind a ready guard"
(`transforms_probe.py`), raises the broken cases a lot (Mavuika 0.09 → 0.39 A,
Mualani–Candace 0.17 → 0.84, Neuvillette 0.15 → 0.62, Wriothesley–Kazuha
0.14 → 0.92) but lowers one easy case (Mualani–Mona 0.86 → 0.65). v2.1's S is
above this transform in every broken case, so v2 finds the unblocking; what
it misses is what A does next.

**F3. In the unsolved cases the residual gap is damage, not waiting, and S
is often bloated.** S has small waits. In four of the five broken-L low cases
the on-field member deals 74–87% of A's damage with S (V2S@384: Mavuika 347k
vs 467k, Arlecchino 544k vs 661k, Mualani 49k vs 62k, Neuvillette 380k vs
456k); in Cyno the deficit is the off-field Furina (46–49k vs 77k). Three of
those five selected S programs are 4–11× A's size (Mavuika 1,042 vs 93
tokens, Cyno 529 vs 128, Neuvillette 426 vs 108), with many dead guarded
branches from accepted zero-gain edits.

**F4. What separates solved from unsolved broken-L cases is how far A is
from L.** Token-multiset similarity of A to L (`corpus_overlap.py`) is
0.20 / 0.30 / 0.38 for the three worst cases (Mavuika, Arlecchino, Cyno) and
0.53–0.86 for the four solved contrast cases. In the worst three, A is much
closer to *another party's main-library rotation with the same on-field
member* (0.91 / 0.81 / 0.81) than to L. Where A is a repair of L, local
search finds it; where A is a different cycle, it does not.

*Coordinator review (2026-11-06):* this separates only three of the six
unsolved cases. The other two unsolved broken-L cases (Mualani–Candace,
Neuvillette) have A close to L (0.75 / 0.73) and no closer to another main
rotation (0.73 / 0.72). Distance from L explains part of the failures, not
all of them; transplant families are expected to address that part only.

**F5. A's advantage is joint and interaction-heavy.** Decomposition hybrids
(hand-built, `transforms.json`):

| Case | Program | DPS / A |
|---|---|---:|
| Mavuika | A's support segment + a generic time-boxed window (`while f() − t0 < 1080 { mavuika charge:2, dash; }`, period from the probe) | 0.994 |
| Mavuika | same, but the on-field skill re-cast when ready inside the window | 0.398 |
| Mavuika | L-derived support segment (bursts guarded) + A's own window | 0.010 |
| Cyno | A with its window period 1260 → 1200 frames (probe-derived period) | 0.941 |
| Cyno | A's support + generic priority window (skill when ready, dash on 5th normal) without A's status guard | 0.682 |
| Cyno | L-derived support + A's window | 0.060 |
| Arlecchino | A without its element-wait loops (`while !.element.t0.cryo && f() − s < 120 { wait(1); }`) | 0.687 |
| Arlecchino | A without its every-4th-cycle burst | 0.891 |
| Arlecchino | L plus an every-4th-cycle burst (alone) | 0.433 (L 0.435) |
| Neuvillette | A with its trailing `wait(715)` replaced by L's `wait(173)` | 1.000 |

Reading: (i) the support segment and the on-field window must be chosen
together — inheriting L's support statements can collapse an otherwise
A-equivalent window to 1–6% of A; (ii) cycle-level parameters matter
(period, burst every N-th cycle, cast a mode skill once per cycle, aura
waits); (iii) part of A uses a member's own status as a guard (`cyno-a1`),
which is mechanical (an engine status name, used in 32 main rotations) but
per character in content.

**F6. Those cycle-level constructs are outside v2's proposal set.** In the
main library, 54 rotations use a time-boxed `while f() − t < P` window and
726 use a cycle-index condition (`if i == 3`, `i % 2`, `is_even(i)`), but the
431 v2.1 templates contain one with `f()` and none with a loop index; the
`for` bound of L is reachable only by ±1 micro steps. v2's operators are
token-level (`src/heldout_v2.rs`, `Proposer::edit`): replace/insert statement,
template, owner reassignment, unit delete/move, action substitution,
segment repeat/delete.

**F7. Neither generator alone is good enough as a final program.** Two mechanical generators
were enumerated without reading A (`skeleton_probe.py`, `transplant_probe.py`):

- *Generic skeleton K* — one time-boxed cycle: L's support statements with
  bursts guarded, the on-field member's ready burst, then a window
  `while f() − t0 < P { skill-if-ready else unit }`, with P from the members'
  probed cooldowns, the unit from that member's three most frequent library
  statements. Best of 12–128 programs on 16 training fights, measured on the
  32 audit fights: **0.18–0.81 A, median 0.55** (24–96 programs per case);
  above L in 9/13, below v2.1's S in all 10 cases that have S.
- *Whole-cycle transplant T* — main-library rotations of other parties that
  contain the on-field member, ranked by members shared, the top 8 retargeted
  over all slot assignments (8–12 programs per case): **0.27–0.91 A, median
  0.70**; below S in 9 of 10 and equal in the tenth (Mualani–Candace 0.82).

**F8. Search from a synthesized start helps in most of the worst cases.**
The unchanged v2.1 search run for 96 CPU-s from K or T instead of L
(`search_start_probe.py`, one seed, six cases) ended higher than from L in
5 of 6 cases (section 2a). This is a single-seed probe whose K and T starts
used extra, uncharged selection CPU; it cannot support an effect claim, only
the decision to test it.

*Coordinator review (2026-11-06):* "best of K/T" takes the better of two
96 CPU-s runs after seeing both, so it compares roughly 192 CPU-s with
selection against one 96 CPU-s run from L, on one seed. It is not an
equal-cost comparison and does not count as evidence that synthesized starts
beat L at equal CPU. Only the held-out SYN vs V21S comparison of section 6
can show that.

### 2a. Search from L, K and T (one seed, 96 CPU-s)

One seed (11), 96 CPU-s per run, the unchanged v2.1 search; DPS / A on the
same 32 fights as above (`search_start.json`) **[measured]**:

| Case | start L → selected | start K → selected | start T → selected | v2.1 S@384 (3 starts, selected) |
|---|---|---|---|---:|
| Mavuika | 0.09 → 0.58 | 0.59 → **0.97** | 0.27 → 0.75 | 0.74 |
| Arlecchino | 0.44 → 0.49 | 0.48 → 0.62 | 0.57 → **0.65** | 0.82 |
| Cyno | 0.43 → **0.78** | 0.55 → 0.60 | 0.38 → 0.59 | 0.77 |
| Mualani–Candace | 0.17 → 0.66 | 0.18 → 0.33 | 0.82 → **0.97** | 0.82 |
| Neuvillette | 0.15 → 0.78 | 0.43 → 0.63 | 0.59 → **0.85** | 0.87 |
| Wriothesley–Kazuha | 0.14 → 0.77 | 0.73 → 0.90 | 0.54 → **0.99** | 1.00 |

Reading, with the caveats above (one seed, no equal-CPU accounting of the
seed choice, development cases): the best of {K, T} start beat the L start in
5 of 6 cases, by 0.07–0.39 A; T beat L in 5 of 6, K in 3 of 6; in two cases
(Mavuika 0.97, Mualani–Candace 0.97) a 96 s run from a synthesized start
exceeded the 384 s v2.1 selection. No single start type wins everywhere
(Cyno is best from L), which argues for competing families rather than
replacing L. Whether this survives fair CPU accounting, three starts and
fresh cases is exactly what the protocol below tests.

### What generic capability would have produced A-like structure

From F1–F6 [hypothesis, to be tested]: a search whose *moves* include whole
cycle structure — on-field member, time-boxed window with a priority policy,
cycle period, per-member burst period, a mode skill cast once per cycle,
aura-gated waits — and whose *starting points* include cycles that are not L
(other parties' main rotations with the same on-field member, mechanical
skeletons), with support segment and window searched jointly, not inherited.
Character knowledge is limited to what the engine reports (cooldowns, burst
cost, action frames, statuses an action creates) and library statistics.

## 3. What the pinned engine exposes (gscouter-engine 1ba0af1)

Paths are under `engine/src/` of the pinned checkout.

| Need | Available | Where |
|---|---|---|
| Burst cost, normal-hit count, talent-cons, animation delays | static `CharData` per character | `data.rs` `struct CharData` (l. 1246) |
| Cooldowns | **not** a table: set in each character's code (`set_cd_with_delay`) | e.g. `chars/bennett.rs` l. 165, 224 |
| Runtime state for guards | `.X.skill/.burst.cd/.ready/.charge`, `.X.energy`, `.energymax`, `.normal`, `.onfield`, `.status.K`, `.mods.K`, `.element.t0.E`, `.stam`, `.airborne` | `sim.rs` `Sim::field` (l. 6157) |
| Driving a simulation action by action | `Sim::new`, `Sim::queue_action`, `Sim::wait`, public `Sim.chars[i].{energy, cd, mods}`, `Mod.key.name`, `Mod::expiry()` | `sim.rs` l. 1661, 6077, 6121; `Char` l. 792; `Mod` l. 108 |
| Per-fight diagnostics | team/per-member DPS, both halves, wait seconds by reason (`swap_cd`, `skill_cd`, `burst_cd`, `insufficient_energy`, `insufficient_stamina`, `dash_cd`) | `api.rs` `run_iterations` (l. 41); `types.rs` `FAILURE_NAMES` (l. 149) |
| Action trace / field time per member | **not exposed** (the `ActionExec` event is internal) | `sim.rs` l. 5982 |

A generic probe was built for this note ([`mechprobe/`](../reports/cycle-synthesis-research-2026-11-06/mechprobe/),
outside `src/`, linking the pinned engine as a library; no character name in
the code). For each member and each of skill/burst/attack/charge it swaps the
member in, runs the action once from a fresh simulation, and records action
frames, the resulting skill/burst cooldown, energy used, team energy gained in
the next 4 s, and every status/mod the action created with its remaining
duration. On all 13 cases it ran in about 0.03 s per case **[measured]**.
Examples: Furina skill 1,182 + 18 frames, burst 789 + 111; Cyno skill 436 + 31,
burst 1,117 + 83 (`lsa.json`, `probe`).

Limitations found **[measured]**: skills that start a mode report cooldown 0
at cast (the cooldown starts later; Xilonen, Mualani, Flins), so the probe
must also read the cooldown after the mode ends; charge without a preceding
attack is refused for several members; a status that appears later in a
stance (e.g. `cyno-a1`) is not created by the action itself and needs a
longer probe window; field-time shares cannot be measured without an engine
change, which is out of scope here.

## 4. Approaches considered

| Approach | Feasibility with existing code | Cost per evaluation | Evidence / risk | Disposition |
|---|---|---|---|---|
| A. Constraint/schedule generator from metadata (timeline skeleton, then fill) | Probe and GCSL rendering are simple; `discovery.rs` already renders visits with periods, phases and time-boxed fields | probe ≈ 0.03 s per case; candidates cost ordinary fights | Raw skeleton 0.18–0.81 A (F7); schedules need facts the probe only partly gives (mode cooldowns, stance statuses); support statements are the fragile part (F5) | **Adopted as one seed generator and as typed operators**, not as a standalone method |
| B. Grammar / n-gram model over whole cycles mined from the library | Possible; v2 already mines statements and guarded blocks | same | Statement statistics are what v2 already has (F6); an n-gram model does not produce cycle-level control (windows, periods) by itself and invites long invalid programs | **Rejected** as a generator; instead the two missing cycle-level constructs are added as typed, corpus-counted operators |
| C. Macro search over cycle skeletons (on-field member, period, burst periods, support order/statements, window policy) with inner annealing | This is the agreed outer structure competition (`docs/rotation-search.md` section 3); needs a typed cycle representation and family seeding in `search.rs` | same; seed preparation is charged | Not yet built; F8 suggests no single start type wins everywhere, so competing families fit; risk that seed preparation eats the budget the L family needs | **Recommended (SYN arm)** |
| D. Crossover / whole-cycle transplant from other parties' main rotations sharing members | Retargeting is a text transform (owner reassignment already exists in v2) | same | A is 0.81–0.91 similar to another main rotation in the three worst cases (F4), raw transplant 0.27–0.91 A (F7), best start in 4 of 6 search probes (F8); leakage risk from near-copies | **Alternative arm (XO)**, and one of SYN's seed sources |
| E. Restart from blank (existing `--from-scratch` grammar) vs from L | Exists (`docs/from-scratch.md`), only smoke-tested on synthetic builds (`reports/discovery-smoke-2026-11-05`) | same | Discards L's working parts; F5 shows support segments are where blind generation fails; never benchmarked against A | **Rejected as an arm**; its visit/period/phase representation is reused |
| F. More of the same (longer runs, more passes) | Exists | — | ≈ 1% start-averaged, null selected (section 1) | **Control only** (V21S) |

## 5. Recommended design (SYN)

Everything below is proposed implementation **[hypothesis]**; parameter
values are initial and may change on development cases only, then freeze
before the draw (section 6).

### 5.1 Mechanical facts (charged to discovery CPU)

A `gso probe` step, generalising `mechprobe`, run once per start:
per member and action — frames, cooldown at cast and after up to 30 s,
burst cost, team particle energy within 4 s, statuses/mods created with
durations, and statuses present on the member during a 20 s generic on-field
loop after skill and after burst. Output: a per-case *fact table*. No
character name is used; the same code runs for every member.

### 5.2 Typed cycle program

A new `Program` variant (or an extension of `discovery::Rotation`) rendering
to GCSL inside the usual `while 1` wrapper:

```
cycle (K iterations, index i):
  let t0 = f();
  [opener: D's mode skill once, if the fact table marks it as delayed-cooldown]
  visits: ordered list of (member, statement, burst policy)
      statement   from: L, the transplant donor, or the member's top library
                  statements (v2.1 dictionary after the k-rare filter)
      burst policy: always | ready-guarded | every k-th iteration at phase φ | never
  window (on-field member D):
      entry: D burst policy
      rules: ≤ 4 ordered (guard → statement), guards from
             {skill.ready, burst.ready, .D.normal == n (n ≤ CharData.normal_hit_num),
              .stam > x, D's own status/mod key from the fact table,
              .element.t0.E for an element of the party, with a timeout}
      default: D's unit statement (top library statements of D)
      exit: f() − t0 < P   (P from the fact table's cycle times ± 10%)
         or a repeat count n
  [trailing wait w]
```

Static checks before simulation: parse, engine support, the v2.1 parameter
triple rule, the v2 assumption-construct guard (no new `execute_action`,
`pick_up_crystallize` or `counter`), a size cap of 2× the larger of the start
program and 200 tokens.

### 5.3 Seeds and the outer structure competition

Per start, within the existing limit of **at most four families and two active
chain slots in total** (`docs/rotation-search.md`, section 3):

- **F0 — L family**: L with the unchanged v2.1 operators (keeps what already
  works for A/L < 1.25 and is the protected baseline).
- **F1 — best skeleton**: enumerate a bounded skeleton grid (D ∈ {L's active
  member, L's top-DPS member}; P from the fact table; 4 units; burst policy
  ready vs every-2nd; support statements from L or the library top statement
  per member) — at most 48 programs on 2 fights, the best 4 completed to 16
  fights; the best becomes F1.
- **F2 — best transplant**: at most 24 retargeted donors (rule of F7, plus
  the leakage filter of section 6.4), screened the same way.
- **F3 — the best remaining seed whose descriptor (D, window or not, period
  class) differs from F1 and F2**, else none.

Each family receives an initial quantum of four completed decisions, then the
existing bounded scheduler allocates the remainder. All screening fights,
probe time and failed seeds count against the same cumulative CPU cap.
Families are paused states, not extra chains.

### 5.4 Inner moves

Typed families (F1–F3) use typed moves with the same annealing acceptance
(`Δ ≥ T ln u`, same u for the proposal):
change P (snap to the fact table or ±10%), change K, change one visit's burst
policy/period/phase, reorder/insert/delete/replace a visit (statement from the
member's library statements), add/remove/reorder a window rule, replace the
unit, toggle the mode-skill opener, change the trailing wait; compound typed
moves (2–3 edits) with probability 0.3.

**Lowering.** After half of a typed family's allocated decisions, or at the
start of the last quarter of the cap, the family's current program is
rendered to source once and continues with the v2.1 token operators and
micro search (one-way, recorded). This keeps v2's fine-tuning without
writing a second set of token operators.

### 5.5 Plug-in points

- `evaluate.rs`: unchanged. Typed programs render to GCSL through
  `Evaluator::config`; the cache key is the rendered config.
- `search.rs`: `initialize()` seeds F1–F3 when `--cycle-synthesis` is set;
  the existing family scheduler, the **common 16-fight decision block**
  (no acceptance at 2/4/8 fights), **32-fight promotion against the peak**,
  restarts, archive and the six-candidate shortlist are reused. The
  shortlist keeps the best program of each family as diversity candidates.
- `heldout_v2.rs`: `Proposer` is reused for F0 and for lowered families; the
  dictionary gains the two cycle-level template kinds of F6 only as typed
  moves, not as text templates.
- New: `gso probe` (fact table), the typed renderer, the donor retargeter
  (reusing owner reassignment), the donor leakage filter (offline, like the
  v2 drop lists).

## 6. Protocol

### 6.1 Questions

- **Q1 (primary).** On fresh broken-L cases, does SYN select a better program
  than the best existing arm (v2.1 single pass, V21S) at the same CPU?
- **Q2.** Does whole-cycle retrieval alone (XO) explain the gain, i.e. SYN vs
  XO and XO vs V21S?
- **Q3.** Does SYN cost anything where L is already good (A/L < 1.25) and on
  controls?

"No detectable difference" is an acceptable outcome.

### 6.2 Arms (equal CPU, same runner per case)

| Arm | Content | Caps (cumulative CPU-s per start) |
|---|---|---|
| **V21S** | v2.1 single pass, unchanged binary behaviour | 96, 192, 384 |
| **SYN** | section 5 (F0 + skeleton + transplant families, typed moves, lowering) | 96, 192, 384 |
| **XO** | F0 + transplant families only, v2.1 token operators only (no typed moves, no skeleton, no probe) | 96, 192, 384 |

Seeds 11, 37, 91 (three starts). Same conditions as v2: 450 s fight, 150 s
warmup, level-100 target, 10% resistance, radius 2, swap delay 12, periodic
energy every 480–720 frames, fixed ER, build B from the case's export.
Independent 128-fight audits per start and cap; audit seeds depend only on
start seed and cap, so the three arms are audited on identical fights.

### 6.3 Draw and how many cases remain

Pool rule (computed by `sa_summary.py`): stage-1 classification rows of kind
`database`, class informative or control, excluding the 30 stage-1 cases, the
40 v2 cases, the `citlali-c6-no-burst` variants (kind `variant`), the
development parties' alternatives (Flins C1, Mavuika C6R5) and, new here, any
case whose party is among the 70 development cases (they share L, and L was
studied with their A). **Remaining [measured]: 131 cases — 96 informative
(50 frame / 46 other) and 35 controls. Of the informative: 25 with
A/L ≥ 2, 37 with 1.25 ≤ A/L < 2, 34 with A/L < 1.25.** A/L uses the stage-1
classification measurement (the same use of A as the existing
informative/control class).

Draw, `random.Random(20261107)`, committed with input hashes before any
held-out search:

| Stratum | Cases | Role |
|---|---|---|
| A/L ≥ 2 | **all 25** (a census, not a sample) | primary |
| 1.25 ≤ A/L < 2 | 15 | secondary |
| A/L < 1.25 | 12 | non-inferiority |
| controls | 8 | regressions |

60 cases, one runner job per case, two waves of at most 40 concurrent jobs.
Enough? [estimate] With 25 primary cases a two-sided Wilcoxon signed-rank
test has roughly 80% power when about two-thirds of cases improve by a
consistent few percent of A; if the gain is confined to the 4–6 worst cases
(as the v2 tail suggests), the primary test may well be underpowered, which
is why the tail count in 6.5 is declared as a secondary reading rather than
replaced after the fact. The A/L ≥ 2 stratum was chosen from development
results (section 1); it is fixed here before any fresh case is drawn.

### 6.4 Leakage

- Searches open only their case's manifest, L, per-case dictionary, the
  main-library corpus snapshot for transplants (4,570 `teams/*/rotation.gcsl`
  at Gscouter `6e0e82205`, ≈ 2.5 MB, never `alt/`), the donor drop list, and
  their run directory. Every search runs under `strace`; any opened path
  containing `/alt/`, `withheld` or `/Gscouter/` fails the job (as in v2).
- Block statistics: v2.1 dictionary with the v2 k-rare filter (k = 10)
  unchanged.
- Whole-cycle donors: the case party's own main rotation is L (allowed); a
  donor is dropped when its slot-normalised token-multiset similarity to A is
  ≥ 0.90 (A read only by this offline filter, like G3). The number of dropped
  donors per case is reported. On development data this rule would drop the
  closest donor for the Mavuika case (0.91) and none for the other worst cases
  (0.81). Results are also reported per case with the selected program's
  donor and its similarity to A.
- A and L on audit fights are measured only after every job has frozen its
  selections, locally, reproducing the job's audited L fight for fight.

### 6.5 Metrics and the primary metric

The selection problem: in v2/v2.1, start-averaged gains did not survive the
selection rule (highest *discovery* estimate across starts), because the
discovery estimate is optimistic and noisy. This pilot therefore changes
**how the served program is chosen, identically for every arm**, and makes
the served program the primary object:

- **Selection block.** After each start's cap, its frozen peak is evaluated
  on a separate 32-fight selection block (a seed namespace disjoint from
  search, promotion and audit seeds), charged to that start's CPU cap. The
  selected program of a case and arm is the start with the highest
  selection-block mean. Audits never feed back.
- **Primary metric.** For each primary case, d = (S_SYN − S_V21S) / A, where
  S is the 128-fight audit mean of the selected program at 384 CPU-s and
  both are audited on the same fights (a selected audit failure counts as
  S := L, Amendment 1 item 6 of v2). Two-sided Wilcoxon signed-rank on the
  25 values.
- **Secondary.** (a) start-averaged d (the v2 primary), to show whether a
  selection-free gain exists; (b) the tail count: primary cases with
  S/A < 0.90, per arm, with an exact McNemar test on discordant pairs
  SYN vs V21S; (c) median S/A per stratum and per arm; (d) the stage-1 reading
  table per arm (median r, regressions); (e) budget curves at 96/192/384;
  (f) mechanism: the family the selected program came from, its structural
  features (window, cycle-index guard, size), timeouts and failed seeds.

### 6.6 Pre-declared readings

Primary (A/L ≥ 2, n = 25):

| Result | Reading |
|---|---|
| median d > 0 and p < 0.05 | SYN beats V21S on broken-L cases |
| median d < 0 and p < 0.05 | V21S beats SYN on broken-L cases |
| otherwise | no detectable difference at this sample size |

Secondary readings use the same rule for SYN vs XO and XO vs V21S (primary
stratum), and for the start-averaged d. Tail count: "fewer low cases" only if
McNemar p < 0.05. Non-inferiority (A/L < 1.25, n = 12): "SYN costs on easy
cases" if the median selected d < −1% of A; otherwise "no cost detected".
Controls: count regressions (S < L by > 2 paired SE) per arm. No other
threshold is used for a headline claim; per-arm readings also use the v2
section-7 table.

### 6.7 Budget [estimate]

| Item | CPU-h |
|---|---:|
| Held-out discovery: 60 cases × 3 arms × 3 starts × 384 CPU-s (probe, seeds, selection block included in the cap) | 57.6 |
| Audits: 60 × 3 arms × 3 starts × 3 caps × 128 fights (v2 measured ≈ 6 CPU-s per start-cap audit) | ≈ 2.7 |
| Local A/L measurement and gcsim parity of selections | ≈ 2 |
| Development: up to two CI runs on 20 development cases (SYN and XO, 3 starts, 384 s) plus local smoke | ≈ 30 |
| Contingency (reruns of failed jobs, a labeled post-unblinding run) | ≈ 30 |
| **Cap for this pilot** | **120** |

Previously recorded held-out work: stage 1 ≈ 7.4 CPU-h, v2 ≈ 28.1 (runners)
plus 7.7 development, v2.1 ≈ 20.4 — about 64 CPU-h, not counting the
earlier Flins/Mavuika pilots, which are not re-summed here. 64 + 120 is
inside the 600 CPU-h pilot allocation. Execution on GitHub-hosted runners
(≤ 40 concurrent jobs, 4 vCPU each) through a dedicated trigger branch
`ci/cycle-synthesis-runs`, nothing on `main`; one job runs all arms and
starts of its case, four search processes at a time. Wall time [estimate]:
per job 3 arms × 3 starts × 384 s = 3,456 CPU-s of discovery plus ≈ 160 CPU-s
of audits (≈ 1.0 CPU-h) on 4 vCPU ≈ 15–16 min, in line with v2's similar load
(11–17 min per job); two waves ≈ 40 min end to end. No VMs, spending or batch
execution are implied.

### 6.8 Stop rules

- **Before the draw (development).** Implementation and parameters may be
  tuned only on the 70 development cases. Freeze the commit, parameters and
  this protocol's SHA-256 in the draw file before drawing. If, on development
  cases, synthesized families yield no valid seed in more than a quarter of
  cases, or the SYN arm's seed preparation exceeds a quarter of the 384 s cap
  in more than a quarter of cases, fix the generator before freezing; if it
  cannot be fixed within the development budget (≈ 30 CPU-h), stop and report
  instead of drawing.
- **During the held-out run.** Per-start budgets are never shortened. If the
  first wave's measured cost exceeds 1.5× the estimate, the second wave runs
  only a declared prefix of the draw order and this is a recorded deviation.
  Hard stop at the 120 CPU-h cap.
- **After.** Any change after held-out results are seen is a separately
  labeled run. If the primary reading is not "SYN beats V21S", the design is
  not carried into a larger batch; the report states which secondary readings
  (XO, tail, start-averaged) point anywhere.

## 7. Risks

- **Budget dilution.** Seeds and the extra families take CPU from F0; SYN may
  lose a little on easy cases while not reaching the hard ones (that is what
  Q3 measures).
- **Invalid or slow programs.** Time-boxed windows with a failing unit or a
  wrong guard can spin until the worker limit; v2 already charges 2–10 s per
  timeout. Typed programs inherit v2's scaled worker wall limit.
- **Probe gaps.** Mode skills, stance statuses and charge prerequisites
  (section 3) mean the fact table is incomplete; wrong periods cost
  performance (Cyno 1260 → 1200: 0.94 A).
- **Character content through the back door.** Status guards use engine
  status names for the member's own statuses. The rule is generic (the same
  probe, the same guard grammar for every member), but rules that need a
  specific constellation state or another member's mechanic (the Kazuha C6
  case of stage 1) remain out of scope.
- **Leakage through near-copies.** Community rotations are often adapted
  from each other; the donor filter is a declared heuristic, not a proof
  that no A-like information enters.
- **Native-only validity.** New constructs (time windows, aura waits) are
  standard GCSL, but selections must still be checked in gcsim as in v2.
- **The primary stratum is small** (25) and was picked from development
  results; a null reading may be a power problem rather than absence of an
  effect.

## 8. Research accounting (this note)

All local, 4 vCPU, `gso` rebuilt from `cce1612` (SHA-256 `6261986b…`,
`target/release/gso`), engine `1ba0af1`; scripts in
`reports/cycle-synthesis-research-2026-11-06/`:

| Script | What it ran | Fights |
|---|---|---|
| `lsa_structure.py` | L, A, S(V2S@384), S(V2T@384) × 13 cases, 32 fights each; `mechprobe` per case | ≈ 1,500 |
| `skeleton_probe.py` | 560 skeletons (24–96 per case) × 16 training fights; best × 32 audit fights | ≈ 9,400 |
| `transforms_probe.py` | 2 transforms × 13 cases and 21 hybrids, 32 fights each | ≈ 1,500 |
| `transplant_probe.py` | 111 retargeted donor programs × 16 fights; top ≤ 6 per case × 32 | ≈ 4,000 |
| `corpus_overlap.py` | token similarity only | 0 |
| `search_start_probe.py` | v2.1 search, 6 cases × 3 starts × 96 CPU-s, then 32-fight measurements | 1,728 CPU-s search |

The measurement scripts are sequential and took about 80 s of wall time per
full pass (fights are 3–10 ms each on these builds); they were run twice
(first with the pre-existing `target/release/gso`, then with the rebuilt
binary; skeleton results were identical), so well under 0.1 CPU-h. The
search probe used 18 × 96 = 1,728 CPU-s of capped search plus audits.
Total ≈ 0.6 CPU-h **[measured, approximate]**. The Gscouter checkout was read only (A for these
development cases and the main-library corpus); its tree is unchanged.

## 9. What this pilot will not show

- That SYN finds optimal cycles: r and S/A are relative to A, a community
  rotation, under native conditions.
- Anything about other profiles, constellations or actual-user gear: the
  cases use the exported builds B only, and no C0 or serving-time search is
  implied (`CLAUDE.md`).
- That the gain (if any) survives on gcsim beyond the parity check of the
  selected programs.
- That cycle synthesis is better than a human-written alternative where the
  library already has one; the motivating use is cases where L is broken for
  the build.
- Cost at batch scale: the per-case seed preparation is measured here only on
  60 cases.
