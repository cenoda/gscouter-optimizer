# Hierarchical Rotation Optimization Architecture

> **Context**: Branch `refactor/hierarchical-discovery`  
> **Previous State**: Single-loop stochastic greedy search frozen after Phase 3 (max 2,014,067 DPS vs 2,250,000 DPS target in `reports/mavuika-225-2026-11-05/`).

---

## 1. Background & Need for Structural Refactoring

The baseline-free discovery engine (`--from-scratch`) proved capable of discovering viable rotations on standard C1 compositions (e.g. Flins C1 pilot: 349,694 DPS, +3.3% over library alternative).

However, high-investment C6 compositions with tight reaction windows and micro-frame cancels (e.g., Mavuika C6R5 / Xilonen / Citlali / Nicole) exposed a fatal flaw in the single-loop optimization model:
* **The search space is bimodal and fragile**: Macro ordering (which support buffs first) and micro execution (hold cancel frames, cyclic finisher delays) were mutated simultaneously in a single stochastic greedy loop.
* **Ad-hoc patches failed**: Patching character-specific heuristics (e.g. `final=1` frame clamping, hardcoded `hold: 1` skill injections) into `src/discovery.rs` increased code entropy without achieving the 2.25M benchmark target (plateauing at 2.01M, with significant seed variance).

To resolve this without corrupting optimizer generality, we decouple the optimizer into a **Hierarchical Optimization Pipeline**.

---

## 2. Core Architecture

```
+-------------------------------------------------------------------+
|                        Stage 1: Macro Search                      |
|  - Determine character visit order & cycle length                 |
|  - High-level role assignments (Opener, Buff-setup, Field DPS)    |
|  - Coarse energy & cooldown feasibility evaluation                |
|  - Uses canonical ActionBlock primitives (no micro-frame noise)   |
+-------------------------------------------------------------------+
                                  |
                                  v
+-------------------------------------------------------------------+
|                     Stage 2: Combo Block Selection                |
|  - Select valid ActionBlocks from character catalog               |
|  - E.g., Mavuika: [HoldSkill -> Burst], [ChargeHold -> Finisher]  |
|  - Preserves reaction synergy (Cryo aura -> Pyro Melt window)     |
+-------------------------------------------------------------------+
                                  |
                                  v
+-------------------------------------------------------------------+
|                   Stage 3: Micro Parameter Tuning                 |
|  - Coordinate descent / sweep on continuous & integer parameters: |
|    * Hold durations (hold_before, hold frames)                    |
|    * Cancel timings (dash cancel delay, swap delays)              |
|    * Loop repeat counts                                           |
|  - Evaluates on deterministic mini-batches                        |
+-------------------------------------------------------------------+
                                  |
                                  v
+-------------------------------------------------------------------+
|                   Stage 4: Full Independent Audit                 |
|  - 128 / 2048 fight confirmation against engine fixtures          |
+-------------------------------------------------------------------+
```

---

## 3. Data Structures & Representation

### A. `ActionBlock` (Replacing loose action tokens)
Instead of mutating raw `ActionCommand` tokens independently, characters execute structured `ActionBlock` units:

```rust
pub enum BlockKind {
    /// Support setup: quick skill/burst and immediate swap
    QuickCast { skill: bool, burst: bool },
    /// Sustained on-field damage sequence
    OnFieldCombo {
        pattern: Vec<ActionCommand>,
        repeats: u8,
        cancel_strategy: CancelStrategy,
    },
    /// Custom character-specific mechanic block (defined in character catalog)
    Specialized {
        name: &'static str,
        commands: Vec<ActionCommand>,
    },
}

pub struct ActionBlock {
    pub actor: usize,
    pub kind: BlockKind,
    pub hold_frames: u8,
}
```

### B. Micro-Parameter Coordinate Sweep
Micro parameters are isolated into a continuous vector `P = [p_1, p_2, ..., p_k]`.  
Rather than mutating `p_i` via random Gaussian noise while changing macro actions, Stage 3 locks the macro graph and optimizes `P` along discrete coordinate directions (e.g., golden-section or bounded grid sweep) across candidate blocks.

---

## 4. Verification Milestones

1. **Isolation Verification**: Existing Flins C1 pilot results and Mavuika 225 Phase 1–3 logs remain frozen and untouched in their respective report directories.
2. **Replay Validation**: Ensure `examples/replay.rs` and `examples/timeline.rs` continue to function as deterministic truth checkers.
3. **Macro Search Sanity**: Validate that Macro Search reproduces clean visit schedules across all 4 party members without infinite loops or stuck cooldowns.
4. **Mavuika 225 Benchmark**: Achieve >= 2,250,000 DPS on `data/mavuika-c6-r5-225/manifest.json` under hierarchical discovery without hardcoded character branching in the main search loop.
