# Research tables (development cases; 32 audit fights unless stated)

Values are DPS / A on the same 32 fights (the selected v2.1 V2S@384 start's audit seed for v2 cases).

| Case | group | A/L | L blocked (s of 300) | S V2S@384 | S V2T@384 | L guard-bursts | skeleton K | transplant T | sim(L,A) | best other main rotation sim to A |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| mavuika-chevreuse-ororon-bennett | low | 11.76 | 250 (insufficient_energy, slot 3) | 0.74 | 0.73 | 0.39 | 0.59 | 0.27 | 0.20 | 0.91 |
| arlecchino-bennett-citlali-xilonen | low | 2.30 | 44 (insufficient_energy, slot 2) | 0.82 | 0.83 | 0.56 | 0.48 | 0.57 | 0.30 | 0.81 |
| cyno-furina-nahida-xilonen | low | 2.33 | 109 (skill_cd, slot 1) | 0.77 | 0.88 | 0.43 | 0.55 | 0.38 | 0.38 | 0.81 |
| mualani-candace-sucrose-xiangling | low | 5.90 | 207 (insufficient_energy, slot 2) | 0.82 | 0.84 | 0.84 | 0.18 | 0.82 | 0.75 | 0.73 |
| neuvillette-furina-citlali-xilonen | low | 6.84 | 151 (insufficient_energy, slot 4) | 0.87 | 0.88 | 0.62 | 0.43 | 0.59 | 0.73 | 0.72 |
| mualani-mona-xilonen-mavuika | low | 1.16 | 2 (burst_cd, slot 4) | 0.90 | 0.90 | 0.65 | 0.35 | 0.82 | 0.49 | 0.74 |
| wriothesley-furina-xilonen-bennett | solved | 2.77 | 165 (insufficient_energy, slot 3) | 1.01 | 0.98 | 0.74 | 0.63 | 0.69 | 0.66 | 0.71 |
| yoimiya-furina-yunjin-xilonen | solved | 2.50 | 167 (insufficient_energy, slot 3) | 0.98 | 0.98 | 0.63 | 0.66 | 0.91 | 0.74 | 0.72 |
| wriothesley-kaedeharakazuha-shenhe-xilonen | solved | 7.42 | 232 (insufficient_energy, slot 3) | 1.00 | 1.01 | 0.92 | 0.73 | 0.54 | 0.53 | 0.70 |
| raidenshogun-furina-xilonen-yelan | solved | 2.03 | 52 (insufficient_energy, slot 3) | 0.95 | 0.97 | 0.57 | 0.60 | 0.70 | 0.86 | 0.81 |
| clorinde-kujousara-nahida-xilonen | stage1-restructure | 1.10 | 17 (skill_cd, slot 1) | – | – | 0.91 | 0.33 | 0.86 | 0.64 | 0.76 |
| flins-ineffa-furina-sucrose | stage1-restructure | 1.14 | 3 (swap_cd, slot 3) | – | – | 0.87 | 0.81 | 0.88 | 0.69 | 0.71 |
| mualani-chiori-xiangling-xilonen | stage1-restructure | 1.19 | 16 (swap_cd, slot 1) | – | – | 0.84 | 0.38 | 0.75 | 0.45 | 0.56 |

Blocked: the largest single per-member wait in L's fights (seconds of the 300 s scoring window, engine wait reason); slot = member position in the manifest. The member is named in lsa.json.

## Hybrids and cycle-level edits (transforms.json)

| Case | Program | DPS / A |
|---|---|---:|
| mavuika-chevreuse-ororon-bennett | A_support+A_window (=A) | 1.000 |
| mavuika-chevreuse-ororon-bennett | A_support+timebox(skill-ready) P=1080 | 0.398 |
| mavuika-chevreuse-ororon-bennett | A_support+timebox(no skill) P=1080 | 0.994 |
| mavuika-chevreuse-ororon-bennett | skeleton_support+A_window | 0.010 |
| mavuika-chevreuse-ororon-bennett | skeleton_support+timebox(no skill) P=1080 | 0.026 |
| arlecchino-bennett-citlali-xilonen | L + `if i == 3 { arlecchino burst; }` (burst every 4th cycle) | 0.433 |
| arlecchino-bennett-citlali-xilonen | A without its arlecchino burst | 0.891 |
| arlecchino-bennett-citlali-xilonen | A without WaitForCryo calls | 0.687 |
| cyno-furina-nahida-xilonen | A_support+A_window P=1260 (=A) | 1.000 |
| cyno-furina-nahida-xilonen | A_support+A_window P=1200 (probe period) | 0.941 |
| cyno-furina-nahida-xilonen | A_support+generic_window(no status guard) P=1260 | 0.682 |
| cyno-furina-nahida-xilonen | A_support+skeleton_window P=1200 | 0.273 |
| cyno-furina-nahida-xilonen | skeleton_support+A_window P=1260 | 0.060 |
| cyno-furina-nahida-xilonen | skeleton_support+skeleton_window P=1200 | 0.538 |
| neuvillette-furina-citlali-xilonen | A with trailing wait(173) | 1.000 |

## v2.1 search from different starts (search_start.json; one seed, 96 CPU-s)

| Case | start | start / A | selected after 96 CPU-s / A | decisions |
|---|---|---:|---:|---:|
| mavuika-chevreuse-ororon-bennett | L | 0.09 | 0.58 | 671 |
| mavuika-chevreuse-ororon-bennett | K | 0.59 | 0.97 | 536 |
| mavuika-chevreuse-ororon-bennett | T | 0.27 | 0.75 | 557 |
| arlecchino-bennett-citlali-xilonen | L | 0.43 | 0.49 | 913 |
| arlecchino-bennett-citlali-xilonen | K | 0.48 | 0.62 | 554 |
| arlecchino-bennett-citlali-xilonen | T | 0.57 | 0.65 | 1085 |
| cyno-furina-nahida-xilonen | L | 0.43 | 0.78 | 488 |
| cyno-furina-nahida-xilonen | K | 0.55 | 0.60 | 608 |
| cyno-furina-nahida-xilonen | T | 0.38 | 0.59 | 623 |
| mualani-candace-sucrose-xiangling | L | 0.17 | 0.66 | 426 |
| mualani-candace-sucrose-xiangling | K | 0.18 | 0.33 | 1138 |
| mualani-candace-sucrose-xiangling | T | 0.82 | 0.97 | 454 |
| neuvillette-furina-citlali-xilonen | L | 0.15 | 0.78 | 646 |
| neuvillette-furina-citlali-xilonen | K | 0.43 | 0.63 | 599 |
| neuvillette-furina-citlali-xilonen | T | 0.59 | 0.85 | 694 |
| wriothesley-kaedeharakazuha-shenhe-xilonen | L | 0.13 | 0.77 | 1153 |
| wriothesley-kaedeharakazuha-shenhe-xilonen | K | 0.73 | 0.90 | 974 |
| wriothesley-kaedeharakazuha-shenhe-xilonen | T | 0.54 | 0.99 | 815 |
