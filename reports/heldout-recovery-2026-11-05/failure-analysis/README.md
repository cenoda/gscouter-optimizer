# Stage-1 failure analysis

**Status: post-hoc diagnosis, not a result.** Written 2026-11-05 on `staging`
at the owner's request, after stage 1 was read as not yet a go. It explains why
the search fell short of the withheld rotation A on the weakest cases. It
changes no stage-1 number, and nothing here was fed back into the search.

**This analysis reads A.** The 30 stage-1 cases are therefore now development
cases. Any change motivated by this document must be measured on a fresh
held-out draw that excludes them, under a protocol committed before that run.

Constraints kept: the proposals below are generic search or protocol changes,
following `docs/rotation-search.md` and `CLAUDE.md`. None adds character
names, case-specific rules or thresholds tuned to these cases. Where only a
character-specific rule would close a gap, the case is listed as out of scope.

## Method

- Cases: the database (non-variant) informative cases with r < 0.75 at 192 s.
- Per-character DPS of L, S and A: [lsa-per-character.jsonl](lsa-per-character.jsonl).
- Rotations compared by hand.
- Each hypothesis was checked by running one mechanically edited program.

Every number below comes from the same setup: gcsim `cenoda/gcsim@eb218847f82b`
through Gscouter `6e0e82205` `internal/score`, no fillers, 128 iterations,
seed 777000, 450 s fight with the first 150 s excluded, and each case's
stage-1 header (B build, level-100 target, swap delay 12, periodic energy).
The edited programs are in [configs/](configs/) and their results in
[diagnostics.jsonl](diagnostics.jsonl).

## Findings

| Case | r (192 s) | L / S / A | Where A gains | Diagnosis |
|---|---:|---|---|---|
| chongyun-furina-escoffier-xianyun | 0.57 | 46,565 / 48,451 / 49,953 | Escoffier 14k → 36k; Chongyun 20k → 3k | **Role reassignment.** Only changing the owner of L's Xianyun plunge loop from Chongyun to Escoffier gives **52,198** (S with the same edit: 53,729), above A. |
| flins-ineffa-columbina-durin | 0.26 | 145,363 / 151,399 / 166,321 | Flins 55k → 76k | **Multi-position rewrite of one action list.** Replacing only S's Flins statement with A's gives **172,563**, above A. A changes several positions in one 17-action list at once. |
| hutao-yelan-columbina-ineffa | 0.27 | 111,724 / 113,523 / 119,610 | Hu Tao 43k → 50k | **Coupled moves.** In S's Hu Tao statement, `jump`→`dash` alone gives 112,335 (−1%); two more N2C repetitions alone give 104,267 (−8%); both together give **117,666** (+3.6%). A with `jump` instead of `dash` drops to 104,667. Each single step loses, so a greedy search cannot reach the joint change. |
| kaedeharakazuha-faruzan-furina-xianyun | 0.11 | 97,108 / 103,064 / 152,108 | Kazuha 52k → 121k | **Character-mechanic logic.** A runs a plunge loop gated on Kazuha C6 state (`kazuha-c6-infusion`, `kazuha-c6-dmgup`). A generic plunge-loop template for Kazuha is *worse* (75,652 from L, 80,897 from S). Closing this needs mechanic-specific conditions: **out of scope**. |
| mualani-chiori-xiangling-xilonen | 0.39 | 130,869 / 141,124 / 157,898 | Chiori 50k → 74k | **Partly an assumption in A.** A overrides `execute_action` to pick up a crystallize shard on 50% of dashes, walks and skills. Removing only that hook gives 146,265. Against that, r ≈ (141,124 − 130,869) / (146,265 − 130,869) ≈ 0.67. The rest is a rewrite of the whole cycle (field order, Xiangling, burst inside the combo). |
| clorinde-kujousara-nahida-xilonen | 0.48 | 69,272 / 72,371 / 75,821 | Kujou Sara 4.8k → 8.4k; fewer Clorinde skill-CD waits | **Cycle restructure.** A replaces L's two-iteration body with a shorter five-iteration cycle and drops the second Sara aim and Clorinde combo. No small edit was isolated. |
| flins-ineffa-furina-sucrose | 0.60 | 131,057 / 141,741 / 149,459 | Flins 65k → 91k | **Cycle restructure.** A uses a 4-iteration cycle and splits Flins into three burst statements. S grew to 3.7× L's size with nested dead branches. |
| mavuika-citlali-xilonen-chevreuse | 0.55 | 116,033 / 617,055 / 1,026,664 | Mavuika 110k → 986k | L breaks on this build. A uses frame-timed charge/hold/final/dash chains with alternating branches: the frame-sensitive kind the protocol expected to fail. |

Search effort was checked and does not explain the failures. Failing cases had
419–1,816 completed decisions, overlapping the successful ones (168–2,536).
Program growth from accepted zero-gain moves (S/L size 1.1–3.7×) also appears
on both successful and failing cases.

## Causes in the implementation

1. **Long guarded blocks are missing from the dictionary.** 91 library
   rotations use a `while .<char>.status.xianyun-airborne-buff { ... }` loop,
   for 12 characters. None of these loops is in the dictionary, because
   `MAX_BLOCK_TOKENS = 40` excludes them.
2. **Blocks only exist under the character that wrote them.** A pattern
   written for Chongyun or Eula cannot be proposed for Escoffier, who has no
   such loop in the library. The Chongyun case is recovered by exactly this
   transfer.
3. **The search is greedy with single-step edits.** The Hu Tao case shows a
   gain that every single step loses. Coupled changes are what the agreed
   design's inner annealing (`docs/rotation-search.md`) would cross. Stage 1
   used greedy as a declared deviation.
4. **Single-token neighborhoods for long action lists.** The Flins case needs
   several positions of one statement changed together.
5. **Too-broad leakage filter.** Every case lost 2–12 blocks, mostly among
   the top five of their character, some used by up to 756 library rotations
   (for example `furina skill , dash , burst ;`). The filter drops any
   dictionary text that occurs in A, even when that text is common across the
   library. This hit successful cases too, so it is a general cost rather than
   the cause of these failures.
6. **Assumption parameters are not separated from play.** A sometimes gains
   from assumptions (the Mualani crystallize hook; `pick_up_crystallize("any")`
   in other A rotations). S once gained from `beidou skill[counter=2]` (about
   3 points of +44%, see `../gcsim-parity.md`). r mixes play and assumptions.

## Proposed generic changes (for a new protocol, not applied)

| # | Change | Addresses | Evidence | Stays generic because |
|---|---|---|---|---|
| G1 | Extract blocks with the owner abstracted (`<self>` placeholder, other references kept), raise the guarded-block cap (e.g. 40 → 96 tokens), and let the search instantiate a template for any party member | 1, 2 | Chongyun: 46.6k → 52.2k by owner change alone | Mechanical extraction from all library rotations; no character list |
| G2 | Replace greedy acceptance with the agreed bounded inner annealing, or add compound moves (one token substitution across a statement plus a repeat-count change) | 3, 4 | Hu Tao: −1% and −8% single steps, +3.6% combined | Restores the design in `docs/rotation-search.md`; operators are character-agnostic |
| G3 | Narrow the leakage filter: drop only dictionary text that is rare outside A's source entry (e.g. found in fewer than k other library rotations), with k declared beforehand | 5 | Common blocks with counts up to 756 dropped in every case | Same rule for every case |
| G4 | Mark assumption-bearing constructs (`execute_action` overrides, `pick_up_crystallize("any")`, `counter`, `hold` ranges): forbid the search from adding new ones, and report r against A with and without them | 6 | Mualani r 0.39 → ≈0.67 without the hook | A fixed construct list, applied to S and A alike |
| G5 | Before audit, delete statements whose removal does not lower the 16-fight block score | readability, later edit efficiency | S up to 3.7× L | Generic minimisation; not expected to change r |

Not proposed:

- Character-mechanic conditions such as Kazuha C6 state gates. That is the
  per-character rule the owner excluded.
- A cycle-structure rewriter for the Clorinde, Flins-Sucrose and Mualani
  cases. G2 may reach some of them; otherwise they stay listed as unsolved.

## What this does not show

- Whether G1–G4 improve the median on unseen cases. These diagnostics used A
  and found edits by inspection. A search has to find them without A, and
  larger neighborhoods cost decisions.
- That the out-of-scope cases are unsolvable. They are only unsolvable by
  this search without character-specific rules.
