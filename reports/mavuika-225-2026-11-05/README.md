# Mavuika C6R5 2.25M Development Pilot Report & Failure Post-Mortem

> **Status: 2.25M Target Unmet (Best: 2,014,067 DPS, -10.5% vs 2.25M target, -37.7% vs 3.23M video).**  
> Ad-hoc heuristic patching across Phases 1–3 failed to discover a competitive rotation within budget. This branch and report freeze the empirical evidence before restructuring the optimizer into a hierarchical combo-block architecture.

---

## 1. Executive Summary

This pilot tested whether baseline-free discovery (`--from-scratch --fixed-er`) could discover a competitive C6R5 Mavuika rotation reaching the **2,250,000 DPS** acceptance threshold under frozen level-100 HUD conditions (`data/mavuika-c6-r5-225/manifest.json`).

While a human-transcribed KQM reference script achieves **2,316,838 DPS** under identical engine conditions, baseline-free discovery hit a hard ceiling:
* **Initial Blind Discovery**: 726k (R1) / 973k (R5)
* **Phase 1 (Extended Action Forms)**: 1,555,130 DPS
* **Phase 2 (Round-Robin Local Enumeration)**: 1,455,749 DPS (regression / instability)
* **Phase 3 (Enforced Starting Kit Coverage)**: **2,014,067 DPS** (best single seed 37; other seeds 1.66M–1.70M)

The trial confirmed that **random token/action mutation cannot reliably discover high-damage C6 play within a 12–36 CPU-second budget**. Micro-frame cancellations, charge-hold intervals, and Melt reaction timings form an ultra-narrow optimization landscape that greedy token mutations fail to climb.

---

## 2. Quantitative Results Across Phases

All runs use identical build fixtures, 120s fight duration, 1-frame swap delay, level 110 target (10% res), and independent 128-fight audits.

### 36 CPU-Second Discovery Results

| Phase | Mode | Seed 11 (Audit) | Seed 37 (Audit) | Seed 91 (Audit) | Phase Median | Max Audit DPS |
|---|---|---:|---:|---:|---:|---:|
| **Phase 1** | Basic | 1,361,649 | 1,342,495 | 1,187,664 | 1,342,495 | 1,361,649 |
| **Phase 1** | Extended | 1,383,761 | 1,507,779 | 1,555,130 | 1,507,779 | 1,555,130 |
| **Phase 2** | Basic | 1,373,853 | 1,570,316 | 1,202,430 | 1,373,853 | 1,570,316 |
| **Phase 2** | Extended | 1,283,536 | 1,074,704 | 1,455,749 | 1,283,536 | 1,455,749 |
| **Phase 3** | Basic | 1,673,084 | 1,620,575 | 1,484,446 | 1,620,575 | 1,673,084 |
| **Phase 3** | Extended | 1,703,868 | **2,014,067** | 1,667,081 | 1,703,868 | **2,014,067** |

*Target acceptance: **2,250,000 DPS** (Shortfall: -235,933 DPS)*  
*Human Reference: **2,316,838 DPS** (Shortfall: -302,771 DPS)*  
*External Video Benchmark: **3,235,781 DPS** (Shortfall: -1,221,714 DPS)*

---

## 3. Post-Mortem: Why Ad-Hoc Patching Failed

### A. Combinatorial Explosion vs. Ultra-Narrow Reaction Windows
In C6 Mavuika play, damage is heavily concentrated in Melt-amplified cyclic charged attacks and finisher hits.
* If a dash cancel is 3 frames too early, the hit does not register.
* If Citlali's Cryo application or Nicole's DEF shred falls out of sync by 20 frames, subsequent attacks lose Melt, dropping team DPS by >40%.
* Mutating single actions (`attack`, `skill`, `hold_before`, `repeats`) at random steps treats these coupled micro-mechanics as independent variables, creating an astronomical search space where 99.9% of mutations degrade DPS.

### B. Loss of Optimizer Generality (Overfitting to Mavuika)
In an effort to bridge the gap, specific heuristics were incrementally patched into `discovery.rs`:
* Hardcoding `hold: 1` skill biases
* Adding `final=1` delay clamping arrays (`[-12, -6, -3, -1, 1, 3, 6, 12]`)
* Enforcing specific opening visits

Despite violating the generic design of the optimizer, these patches still fell short of the 2.25M target and severely damaged the modularity and maintainability of the codebase.

### C. Local Minima Trapping
In Phase 2, extended grammar actually regressed compared to basic grammar (1.45M vs 1.57M) because adding action parameter degrees of freedom increased the likelihood of getting stuck in suboptimal local valleys. In Phase 3, only a single seed (37) broke 2.0M, showing extreme sensitivity to initial conditions.

---

## 4. Isolation & Remediation Plan

This directory (`reports/mavuika-225-2026-11-05/`) and branch `pilot/flins-recovery` are frozen as an audited record of the limitations of the single-loop greedy approach.

### Architectural Rework on `refactor/hierarchical-discovery`:
1. **Hierarchical Separation**:
   * **Macro Layer**: Global rotation structure, character field periods, cooldown & energy management.
   * **Micro Layer**: Local combo execution and frame-level parameter tuning.
2. **Combo Block Primitives (`ActionBlock`)**:
   * Replace single-token action soup with cohesive character combo blocks (e.g., Skill-Hold setup blocks, Finisher-Cancel loop blocks).
3. **Dedicated Coordinate / Sweep Tuning**:
   * Once a macro rotation structure is identified, perform direct coordinate descent on micro parameters (hold durations, repeat counts) rather than random stochastic mutations.
