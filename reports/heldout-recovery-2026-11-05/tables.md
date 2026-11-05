# Generated tables (stage1)

Conditions for every number: native engine 1ba0af1, 450 s fight with the first 150 s excluded, one level-100 target, 10% resistance, swap delay 12, periodic energy every 480-720 frames, each case's alt build.txt with the alternative's team.yaml params, fixed ER. L, A and S of a row are audited on the same 128 fresh fights of the selected start's checkpoint audit seeds. S is the audited score of the pre-audit selection of the start with the highest discovery estimate. r = (S−L)/(A−L). Regression: S−L below −2 paired SE. Start spread: range of the three starts' audited S (each on its own audit seeds) relative to L.

## Summary

| Checkpoint (CPU-s per start) | Cases | Informative | Median r (informative) | r quartiles | r ≥ 0.5 | Median r, frame / other | Informative regressions | S = L |
|---|---:|---:|---:|---|---:|---|---:|---:|
| 36 | 30 | 24 | 0.736 | 0.45, 0.74, 1.13 | 67% | 0.661 (n=12) / 0.843 (n=12) | 0 | 0% |
| 108 | 30 | 24 | 0.946 | 0.49, 0.95, 1.21 | 75% | 0.958 (n=12) / 0.946 (n=12) | 0 | 0% |
| 192 | 30 | 24 | 0.940 | 0.55, 0.94, 1.27 | 79% | 0.993 (n=12) / 0.940 (n=12) | 0 | 0% |
| 576 | 6 | 6 | 0.782 | 0.58, 0.78, 1.17 | 100% | 0.615 (n=3) / 0.948 (n=3) | 0 | 0% |

## Checkpoint 36 CPU-s per start

| # | Case | Class | Stratum | L | A | A−L | S (selected) | r | Sel. start | S per start (11 / 37 / 91) | Start spread | Regression | CPU per start | A features | S features |
|---:|---|---|---|---:|---:|---:|---:|---:|---:|---|---:|---|---|---|---|
| 0 | wriothesley-shenhe-furina-xilonen--mQBdNDhMnnH6 | informative | frame | 21,147 | 339,446 | +1505.2% | 255,552 | 0.74 | 37 | 171,261 / 255,552 / 224,524 | +398.6% | no (234,406 ± 225) | 36, 36, 36 | hold | hold |
| 1 | travelerpyro-furina-bennett-citlali--citlali-c6-no-burst | informative | frame | 95,543 | 103,620 | +8.5% | 103,569 | 0.99 | 91 | 97,901 / 102,404 / 103,569 | +5.9% | no (8,026 ± 301) | 44, 36, 44 | hold,while | hold,while |
| 2 | gaming-citlali-kaedeharakazuha-bennett--citlali-c6-no-burst | control | other | 123,342 | 130,061 | +5.4% | 134,628 | 1.68 | 91 | 136,897 / 129,455 / 134,628 | +6.0% | no (11,287 ± 1,631) | 36, 36, 36 | wait_or_delay | hold,wait_or_delay |
| 3 | shenhe-chasca-citlali-mavuika--citlali-c6-no-burst | informative | frame | 112,695 | 124,690 | +10.6% | 128,450 | 1.31 | 37 | 113,442 / 128,450 / 119,480 | +13.3% | no (15,755 ± 264) | 36, 36, 36 | hold,wait_or_delay | hold,wait_or_delay |
| 4 | cyno-fischl-nahida-xilonen--kRfdGgkcpznD | informative | frame | 174,024 | 207,408 | +19.2% | 193,568 | 0.59 | 11 | 193,568 / 184,123 / 176,621 | +9.7% | no (19,544 ± 177) | 45, 40, 36 | while | while |
| 5 | xiao-faruzan-furina-xianyun--n8fCP7cBnnWz | informative | frame | 68,177 | 75,287 | +10.4% | 76,295 | 1.14 | 37 | 71,785 / 76,295 / 73,655 | +6.6% | no (8,118 ± 305) | 36, 39, 36 | while | while |
| 6 | kamisatoayaka-furina-citlali-escoffier--RmDNLghj8jqh | informative | other | 108,954 | 432,558 | +297.0% | 346,999 | 0.74 | 91 | 274,929 / 327,569 / 346,999 | +66.1% | no (238,044 ± 1,767) | 36, 36, 36 | none | none |
| 7 | wriothesley-xiangling-furina-bennett--mwjCWWCcpwJJ | informative | other | 164,814 | 224,932 | +36.5% | 228,495 | 1.06 | 11 | 228,495 / 197,352 / 184,875 | +26.5% | no (63,681 ± 379) | 42, 36, 36 | none | while |
| 8 | neuvillette-furina-lanyan-xilonen--dB9DNqJgzHBr | informative | other | 374,113 | 435,019 | +16.3% | 455,695 | 1.34 | 11 | 455,695 / 404,581 / 385,776 | +18.7% | no (81,582 ± 440) | 36, 36, 36 | wait_or_delay | wait_or_delay |
| 9 | noelle-xilonen-nicole-durin--99dNHg8bKpcw | control | other | 618,923 | 621,489 | +0.4% | 668,894 | 19.47 | 37 | 656,883 / 668,894 / 638,070 | +5.0% | no (49,972 ± 1,256) | 36, 36, 36 | none | none |
| 10 | kaedeharakazuha-faruzan-furina-xianyun--JFrLR9QLQgdw | informative | frame | 97,009 | 152,127 | +56.8% | 97,968 | 0.02 | 11 | 97,968 / 97,513 / 96,468 | +1.5% | no (958 ± 264) | 36, 42, 40 | while,wait_or_delay | none |
| 11 | chongyun-furina-escoffier-xianyun--KPmGTPNQRDW6 | informative | frame | 46,646 | 49,913 | +7.0% | 48,203 | 0.48 | 11 | 48,203 / 46,733 / 48,086 | +3.2% | no (1,557 ± 78) | 36, 46, 44 | while | while |
| 12 | nahida-fischl-beidou-kaedeharakazuha--bcR6QJHLLBtH | control | frame | 61,887 | 57,119 | -7.7% | 82,200 | -4.26 | 91 | 74,821 / 81,009 / 82,200 | +11.9% | no (20,312 ± 88) | 45, 36, 36 | hold,while,wait_or_delay | none |
| 13 | chasca-citlali-chongyun-bennett--citlali-c6-no-burst | informative | other | 78,672 | 82,940 | +5.4% | 89,937 | 2.64 | 37 | 83,373 / 89,937 / 83,771 | +8.3% | no (11,265 ± 326) | 36, 36, 36 | wait_or_delay | while,wait_or_delay |
| 14 | bennett-fischl-lisa-xingqiu--z89bwQG6KGHg | informative | frame | 43,150 | 51,523 | +19.4% | 52,163 | 1.08 | 91 | 49,167 / 47,619 / 52,163 | +10.5% | no (9,013 ± 83) | 36, 36, 36 | hold | hold |
| 15 | mavuika-citlali-charlotte-bennett--citlali-c6-no-burst | informative | frame | 114,819 | 143,608 | +25.1% | 168,950 | 1.88 | 37 | 156,817 / 168,950 / 161,946 | +10.6% | no (54,131 ± 142) | 36, 36, 36 | hold | hold |
| 16 | mualani-chiori-xiangling-xilonen--fKzfLbrbfhj9 | informative | frame | 131,670 | 157,910 | +19.9% | 141,184 | 0.36 | 11 | 141,184 / 139,143 / 137,573 | +2.7% | no (9,514 ± 479) | 41, 36, 41 | while,wait_or_delay | while |
| 17 | mavuika-citlali-xilonen-albedo--citlali-c6-no-burst | control | frame | 135,467 | 138,835 | +2.5% | 143,571 | 2.41 | 11 | 143,571 / 141,926 / 140,999 | +1.9% | no (8,104 ± 383) | 36, 36, 36 | hold,wait_or_delay | hold,wait_or_delay |
| 18 | flins-ineffa-furina-sucrose--j86qrckHDPmb | informative | other | 131,089 | 148,816 | +13.5% | 141,913 | 0.61 | 37 | 132,300 / 141,913 / 139,241 | +7.3% | no (10,824 ± 120) | 36, 36, 36 | none | while |
| 19 | neuvillette-furina-lynette-xilonen--JhJ7FPddfHRz | informative | other | 31,804 | 420,997 | +1223.7% | 414,555 | 0.98 | 91 | 190,211 / 325,998 / 414,555 | +705.4% | no (382,751 ± 188) | 42, 36, 36 | wait_or_delay | wait_or_delay |
| 20 | chasca-citlali-mualani-bennett--citlali-c6-no-burst | control | frame | 95,584 | 99,699 | +4.3% | 101,294 | 1.39 | 91 | 100,368 / 99,421 / 101,294 | +2.0% | no (5,710 ± 332) | 36, 37, 36 | while,wait_or_delay | while,wait_or_delay |
| 21 | citlali-chasca-escoffier-mavuika--citlali-c6-no-burst | control | other | 132,151 | 137,475 | +4.0% | 140,253 | 1.52 | 37 | 139,732 / 140,253 / 139,384 | +0.7% | no (8,102 ± 284) | 36, 36, 36 | wait_or_delay | wait_or_delay |
| 22 | hutao-furina-yelan-citlali--m9CWdQGTJwCH | informative | other | 271,994 | 505,152 | +85.7% | 408,513 | 0.59 | 91 | 306,756 / 346,522 / 408,513 | +37.4% | no (136,519 ± 735) | 36, 36, 36 | none | none |
| 23 | bennett-lynette-xiangling-lyney--jNtRRhR8NM9m | informative | other | 45,198 | 63,455 | +40.4% | 52,917 | 0.42 | 91 | 51,917 / 45,216 / 52,917 | +17.0% | no (7,719 ± 75) | 36, 44, 36 | wait_or_delay | wait_or_delay |
| 24 | chasca-citlali-rosaria-bennett--citlali-c6-no-burst | informative | other | 74,308 | 78,459 | +5.6% | 82,020 | 1.86 | 91 | 80,531 / 81,173 / 82,020 | +2.0% | no (7,712 ± 280) | 36, 36, 36 | none | none |
| 25 | clorinde-kujousara-nahida-xilonen--DQkpHtnCkKgR | informative | frame | 69,206 | 75,874 | +9.6% | 72,290 | 0.46 | 91 | 71,312 / 71,249 / 72,290 | +1.5% | no (3,084 ± 37) | 36, 45, 36 | while,wait_or_delay | while,wait_or_delay |
| 26 | hutao-yelan-mona-nicole--NpWw8TkzPPDz | informative | other | 126,122 | 146,410 | +16.1% | 145,423 | 0.95 | 91 | 140,057 / 141,537 / 145,423 | +4.3% | no (19,301 ± 133) | 36, 36, 36 | wait_or_delay | none |
| 27 | hutao-yelan-columbina-ineffa--7DdP6dDqRGbC | informative | other | 111,731 | 118,776 | +6.3% | 113,526 | 0.25 | 11 | 113,526 / 111,973 / 112,260 | +1.4% | no (1,795 ± 220) | 36, 36, 36 | none | none |
| 28 | mavuika-citlali-xilonen-chevreuse--dkC8Gtdm8F9d | informative | frame | 115,780 | 1,024,355 | +784.7% | 524,875 | 0.45 | 91 | 437,853 / 410,281 / 524,875 | +99.0% | no (409,095 ± 1,593) | 36, 36, 36 | hold,wait_or_delay | hold,wait_or_delay |
| 29 | flins-ineffa-columbina-durin--kwnMBjKmcJWQ | informative | other | 145,332 | 166,785 | +14.8% | 151,074 | 0.27 | 37 | 151,220 / 151,074 / 150,847 | +0.3% | no (5,742 ± 275) | 36, 36, 36 | none | none |

## Checkpoint 108 CPU-s per start

| # | Case | Class | Stratum | L | A | A−L | S (selected) | r | Sel. start | S per start (11 / 37 / 91) | Start spread | Regression | CPU per start | A features | S features |
|---:|---|---|---|---:|---:|---:|---:|---:|---:|---|---:|---|---|---|---|
| 0 | wriothesley-shenhe-furina-xilonen--mQBdNDhMnnH6 | informative | frame | 21,183 | 339,645 | +1503.4% | 283,603 | 0.82 | 37 | 271,638 / 283,603 / 226,708 | +268.6% | no (262,420 ± 194) | 108, 108, 115 | hold | hold |
| 1 | travelerpyro-furina-bennett-citlali--citlali-c6-no-burst | informative | frame | 95,314 | 103,935 | +9.0% | 104,727 | 1.09 | 37 | 104,550 / 104,727 / 104,321 | +0.4% | no (9,413 ± 333) | 111, 108, 108 | hold,while | hold,while |
| 2 | gaming-citlali-kaedeharakazuha-bennett--citlali-c6-no-burst | control | other | 124,317 | 130,016 | +4.6% | 137,667 | 2.34 | 11 | 137,667 / 129,589 / 133,366 | +6.5% | no (13,350 ± 611) | 108, 108, 108 | wait_or_delay | wait_or_delay |
| 3 | shenhe-chasca-citlali-mavuika--citlali-c6-no-burst | informative | frame | 112,669 | 124,950 | +10.9% | 128,051 | 1.25 | 37 | 117,666 / 128,051 / 119,138 | +9.2% | no (15,382 ± 268) | 108, 108, 108 | hold,wait_or_delay | hold,wait_or_delay |
| 4 | cyno-fischl-nahida-xilonen--kRfdGgkcpznD | informative | frame | 173,932 | 206,942 | +19.0% | 216,602 | 1.29 | 91 | 193,675 / 184,508 / 216,602 | +18.5% | no (42,670 ± 180) | 112, 108, 115 | while | while |
| 5 | xiao-faruzan-furina-xianyun--n8fCP7cBnnWz | informative | frame | 68,197 | 75,616 | +10.9% | 76,347 | 1.10 | 37 | 71,747 / 76,347 / 74,230 | +6.7% | no (8,150 ± 276) | 117, 108, 117 | while | while |
| 6 | kamisatoayaka-furina-citlali-escoffier--RmDNLghj8jqh | informative | other | 107,912 | 434,366 | +302.5% | 416,289 | 0.94 | 91 | 409,893 / 331,439 / 416,289 | +78.6% | no (308,376 ± 1,709) | 108, 108, 108 | none | while |
| 7 | wriothesley-xiangling-furina-bennett--mwjCWWCcpwJJ | informative | other | 165,192 | 227,644 | +37.8% | 227,737 | 1.00 | 11 | 227,737 / 209,318 / 191,816 | +21.7% | no (62,545 ± 406) | 117, 111, 108 | none | while |
| 8 | neuvillette-furina-lanyan-xilonen--dB9DNqJgzHBr | informative | other | 374,253 | 435,057 | +16.2% | 455,225 | 1.33 | 11 | 455,225 / 421,748 / 385,267 | +18.7% | no (80,972 ± 483) | 108, 108, 108 | wait_or_delay | wait_or_delay |
| 9 | noelle-xilonen-nicole-durin--99dNHg8bKpcw | control | other | 619,768 | 619,971 | +0.0% | 669,277 | 243.62 | 37 | 657,789 / 669,277 / 637,086 | +5.2% | no (49,509 ± 1,080) | 109, 108, 108 | none | none |
| 10 | kaedeharakazuha-faruzan-furina-xianyun--JFrLR9QLQgdw | informative | frame | 96,814 | 152,216 | +57.2% | 101,902 | 0.09 | 91 | 99,125 / 97,815 / 101,902 | +4.2% | no (5,088 ± 378) | 108, 108, 108 | while,wait_or_delay | while |
| 11 | chongyun-furina-escoffier-xianyun--KPmGTPNQRDW6 | informative | frame | 46,643 | 49,911 | +7.0% | 48,351 | 0.52 | 11 | 48,351 / 46,496 / 48,054 | +4.0% | no (1,708 ± 79) | 108, 108, 108 | while | while |
| 12 | nahida-fischl-beidou-kaedeharakazuha--bcR6QJHLLBtH | control | frame | 61,854 | 57,083 | -7.7% | 84,512 | -4.75 | 91 | 79,205 / 81,710 / 84,512 | +8.6% | no (22,658 ± 68) | 108, 108, 108 | hold,while,wait_or_delay | none |
| 13 | chasca-citlali-chongyun-bennett--citlali-c6-no-burst | informative | other | 78,268 | 83,425 | +6.6% | 90,171 | 2.31 | 37 | 85,902 / 90,171 / 83,283 | +8.8% | no (11,902 ± 351) | 108, 109, 108 | wait_or_delay | while,wait_or_delay |
| 14 | bennett-fischl-lisa-xingqiu--z89bwQG6KGHg | informative | frame | 43,075 | 51,360 | +19.2% | 52,121 | 1.09 | 91 | 49,069 / 47,565 / 52,121 | +10.6% | no (9,046 ± 88) | 108, 113, 117 | hold | hold |
| 15 | mavuika-citlali-charlotte-bennett--citlali-c6-no-burst | informative | frame | 114,751 | 143,386 | +25.0% | 176,273 | 2.15 | 37 | 161,767 / 176,273 / 162,356 | +12.6% | no (61,522 ± 94) | 108, 108, 108 | hold | hold |
| 16 | mualani-chiori-xiangling-xilonen--fKzfLbrbfhj9 | informative | frame | 130,977 | 157,451 | +20.2% | 140,870 | 0.37 | 11 | 140,870 / 140,336 / 138,172 | +2.1% | no (9,893 ± 515) | 110, 108, 114 | while,wait_or_delay | while |
| 17 | mavuika-citlali-xilonen-albedo--citlali-c6-no-burst | control | frame | 135,259 | 138,482 | +2.4% | 143,893 | 2.68 | 11 | 143,893 / 142,391 / 140,192 | +2.7% | no (8,633 ± 325) | 108, 108, 108 | hold,wait_or_delay | hold,wait_or_delay |
| 18 | flins-ineffa-furina-sucrose--j86qrckHDPmb | informative | other | 131,116 | 149,972 | +14.4% | 141,529 | 0.55 | 37 | 134,744 / 141,529 / 139,750 | +5.2% | no (10,413 ± 187) | 108, 108, 108 | none | while |
| 19 | neuvillette-furina-lynette-xilonen--JhJ7FPddfHRz | informative | other | 31,656 | 421,133 | +1230.4% | 414,683 | 0.98 | 91 | 296,801 / 351,367 / 414,683 | +372.4% | no (383,028 ± 166) | 108, 108, 108 | wait_or_delay | wait_or_delay |
| 20 | chasca-citlali-mualani-bennett--citlali-c6-no-burst | control | frame | 95,332 | 99,451 | +4.3% | 101,167 | 1.42 | 91 | 100,366 / 99,804 / 101,167 | +1.4% | no (5,834 ± 300) | 108, 116, 113 | while,wait_or_delay | while,wait_or_delay |
| 21 | citlali-chasca-escoffier-mavuika--citlali-c6-no-burst | control | other | 131,886 | 137,762 | +4.5% | 140,113 | 1.40 | 37 | 139,676 / 140,113 / 139,285 | +0.6% | no (8,227 ± 306) | 108, 108, 108 | wait_or_delay | wait_or_delay |
| 22 | hutao-furina-yelan-citlali--m9CWdQGTJwCH | informative | other | 270,967 | 506,099 | +86.8% | 416,742 | 0.62 | 11 | 416,742 / 347,038 / 409,510 | +25.7% | no (145,774 ± 646) | 109, 114, 108 | none | none |
| 23 | bennett-lynette-xiangling-lyney--jNtRRhR8NM9m | informative | other | 45,041 | 63,521 | +41.0% | 53,698 | 0.47 | 37 | 51,935 / 53,698 / 52,889 | +3.9% | no (8,656 ± 88) | 117, 116, 108 | wait_or_delay | while,wait_or_delay |
| 24 | chasca-citlali-rosaria-bennett--citlali-c6-no-burst | informative | other | 74,240 | 78,960 | +6.4% | 82,311 | 1.71 | 91 | 80,765 / 82,654 / 82,311 | +2.5% | no (8,071 ± 340) | 108, 111, 108 | none | none |
| 25 | clorinde-kujousara-nahida-xilonen--DQkpHtnCkKgR | informative | frame | 69,224 | 75,849 | +9.6% | 72,374 | 0.48 | 91 | 71,373 / 71,216 / 72,374 | +1.7% | no (3,150 ± 43) | 116, 114, 109 | while,wait_or_delay | while,wait_or_delay |
| 26 | hutao-yelan-mona-nicole--NpWw8TkzPPDz | informative | other | 126,135 | 146,243 | +15.9% | 145,203 | 0.95 | 91 | 140,075 / 141,392 / 145,203 | +4.1% | no (19,068 ± 125) | 108, 108, 112 | wait_or_delay | none |
| 27 | hutao-yelan-columbina-ineffa--7DdP6dDqRGbC | informative | other | 111,489 | 119,763 | +7.4% | 113,426 | 0.23 | 11 | 113,426 / 112,441 / 112,038 | +1.2% | no (1,937 ± 195) | 108, 113, 108 | none | none |
| 28 | mavuika-citlali-xilonen-chevreuse--dkC8Gtdm8F9d | informative | frame | 119,883 | 1,024,938 | +754.9% | 612,872 | 0.54 | 91 | 543,260 / 471,838 / 612,872 | +117.6% | no (492,988 ± 2,094) | 108, 108, 108 | hold,wait_or_delay | hold,wait_or_delay |
| 29 | flins-ineffa-columbina-durin--kwnMBjKmcJWQ | informative | other | 145,394 | 166,997 | +14.9% | 151,213 | 0.27 | 91 | 151,149 / 150,855 / 151,213 | +0.2% | no (5,820 ± 241) | 108, 108, 108 | none | none |

## Checkpoint 192 CPU-s per start

| # | Case | Class | Stratum | L | A | A−L | S (selected) | r | Sel. start | S per start (11 / 37 / 91) | Start spread | Regression | CPU per start | A features | S features |
|---:|---|---|---|---:|---:|---:|---:|---:|---:|---|---:|---|---|---|---|
| 0 | wriothesley-shenhe-furina-xilonen--mQBdNDhMnnH6 | informative | frame | 21,138 | 339,454 | +1505.9% | 295,543 | 0.86 | 37 | 273,892 / 295,543 / 234,657 | +288.0% | no (274,405 ± 199) | 192, 192, 198 | hold | hold |
| 1 | travelerpyro-furina-bennett-citlali--citlali-c6-no-burst | informative | frame | 95,522 | 103,416 | +8.3% | 104,401 | 1.12 | 37 | 104,712 / 104,401 / 105,263 | +0.9% | no (8,878 ± 317) | 192, 196, 192 | hold,while | hold,while |
| 2 | gaming-citlali-kaedeharakazuha-bennett--citlali-c6-no-burst | control | other | 124,533 | 130,096 | +4.5% | 137,242 | 2.28 | 11 | 137,242 / 129,573 / 133,280 | +6.2% | no (12,709 ± 687) | 192, 192, 192 | wait_or_delay | wait_or_delay |
| 3 | shenhe-chasca-citlali-mavuika--citlali-c6-no-burst | informative | frame | 112,807 | 124,696 | +10.5% | 128,245 | 1.30 | 37 | 119,788 / 128,245 / 119,282 | +7.9% | no (15,439 ± 290) | 192, 192, 192 | hold,wait_or_delay | hold,wait_or_delay |
| 4 | cyno-fischl-nahida-xilonen--kRfdGgkcpznD | informative | frame | 173,950 | 206,982 | +19.0% | 216,524 | 1.29 | 91 | 194,118 / 184,694 / 216,524 | +18.3% | no (42,574 ± 198) | 197, 200, 198 | while | while |
| 5 | xiao-faruzan-furina-xianyun--n8fCP7cBnnWz | informative | frame | 68,393 | 75,330 | +10.1% | 76,839 | 1.22 | 37 | 71,598 / 76,839 / 74,185 | +7.7% | no (8,446 ± 298) | 201, 197, 201 | while | while |
| 6 | kamisatoayaka-furina-citlali-escoffier--RmDNLghj8jqh | informative | other | 112,505 | 434,692 | +286.4% | 416,072 | 0.94 | 91 | 410,725 / 340,314 / 416,072 | +67.3% | no (303,567 ± 1,879) | 192, 192, 192 | none | while |
| 7 | wriothesley-xiangling-furina-bennett--mwjCWWCcpwJJ | informative | other | 164,779 | 226,454 | +37.4% | 229,031 | 1.04 | 11 | 229,031 / 209,316 / 191,881 | +22.5% | no (64,252 ± 356) | 201, 194, 192 | none | while |
| 8 | neuvillette-furina-lanyan-xilonen--dB9DNqJgzHBr | informative | other | 374,547 | 435,126 | +16.2% | 454,794 | 1.32 | 11 | 454,794 / 420,480 / 385,704 | +18.4% | no (80,247 ± 532) | 196, 192, 192 | wait_or_delay | wait_or_delay |
| 9 | noelle-xilonen-nicole-durin--99dNHg8bKpcw | control | other | 619,290 | 620,400 | +0.2% | 683,814 | 58.15 | 91 | 660,463 / 669,175 / 683,814 | +3.8% | no (64,524 ± 871) | 192, 192, 192 | none | none |
| 10 | kaedeharakazuha-faruzan-furina-xianyun--JFrLR9QLQgdw | informative | frame | 96,951 | 151,845 | +56.6% | 103,032 | 0.11 | 11 | 103,032 / 97,701 / 101,847 | +5.5% | no (6,082 ± 440) | 192, 197, 192 | while,wait_or_delay | none |
| 11 | chongyun-furina-escoffier-xianyun--KPmGTPNQRDW6 | informative | frame | 46,669 | 49,928 | +7.0% | 48,531 | 0.57 | 91 | 48,306 / 46,601 / 48,531 | +4.1% | no (1,862 ± 75) | 196, 202, 200 | while | while |
| 12 | nahida-fischl-beidou-kaedeharakazuha--bcR6QJHLLBtH | control | frame | 61,871 | 57,078 | -7.7% | 89,292 | -5.72 | 91 | 87,612 / 81,644 / 89,292 | +12.4% | no (27,421 ± 63) | 192, 192, 192 | hold,while,wait_or_delay | none |
| 13 | chasca-citlali-chongyun-bennett--citlali-c6-no-burst | informative | other | 78,611 | 83,086 | +5.7% | 90,181 | 2.59 | 37 | 86,509 / 90,181 / 84,933 | +6.7% | no (11,570 ± 297) | 192, 194, 192 | wait_or_delay | while,wait_or_delay |
| 14 | bennett-fischl-lisa-xingqiu--z89bwQG6KGHg | informative | frame | 43,079 | 51,335 | +19.2% | 52,471 | 1.14 | 11 | 52,471 / 47,672 / 52,033 | +11.1% | no (9,392 ± 102) | 192, 196, 192 | hold | hold |
| 15 | mavuika-citlali-charlotte-bennett--citlali-c6-no-burst | informative | frame | 114,847 | 143,490 | +24.9% | 176,213 | 2.14 | 37 | 166,395 / 176,213 / 162,414 | +12.0% | no (61,366 ± 88) | 192, 192, 192 | hold | hold |
| 16 | mualani-chiori-xiangling-xilonen--fKzfLbrbfhj9 | informative | frame | 131,345 | 157,775 | +20.1% | 141,761 | 0.39 | 11 | 141,761 / 140,483 / 138,414 | +2.5% | no (10,416 ± 386) | 197, 198, 193 | while,wait_or_delay | while |
| 17 | mavuika-citlali-xilonen-albedo--citlali-c6-no-burst | control | frame | 136,167 | 138,569 | +1.8% | 143,656 | 3.12 | 11 | 143,656 / 142,672 / 140,533 | +2.3% | no (7,489 ± 420) | 192, 192, 195 | hold,wait_or_delay | hold,wait_or_delay |
| 18 | flins-ineffa-furina-sucrose--j86qrckHDPmb | informative | other | 131,225 | 149,183 | +13.7% | 142,001 | 0.60 | 37 | 134,631 / 142,001 / 139,556 | +5.6% | no (10,776 ± 141) | 193, 192, 192 | none | while |
| 19 | neuvillette-furina-lynette-xilonen--JhJ7FPddfHRz | informative | other | 31,827 | 420,991 | +1222.8% | 414,799 | 0.98 | 91 | 307,869 / 353,559 / 414,799 | +336.0% | no (382,972 ± 199) | 192, 198, 192 | wait_or_delay | wait_or_delay |
| 20 | chasca-citlali-mualani-bennett--citlali-c6-no-burst | control | frame | 95,352 | 99,922 | +4.8% | 101,115 | 1.26 | 91 | 100,641 / 99,856 / 101,115 | +1.3% | no (5,763 ± 327) | 192, 194, 196 | while,wait_or_delay | while,wait_or_delay |
| 21 | citlali-chasca-escoffier-mavuika--citlali-c6-no-burst | control | other | 132,001 | 137,925 | +4.5% | 143,073 | 1.87 | 37 | 140,807 / 143,073 / 143,398 | +2.0% | no (11,072 ± 301) | 192, 192, 192 | wait_or_delay | wait_or_delay |
| 22 | hutao-furina-yelan-citlali--m9CWdQGTJwCH | informative | other | 271,366 | 505,402 | +86.2% | 443,734 | 0.74 | 91 | 439,980 / 346,756 / 443,734 | +35.7% | no (172,368 ± 653) | 192, 192, 192 | none | none |
| 23 | bennett-lynette-xiangling-lyney--jNtRRhR8NM9m | informative | other | 45,229 | 63,502 | +40.4% | 56,600 | 0.62 | 91 | – / 55,997 / 56,600 | +1.3% | no (11,371 ± 116) | 197, 200, 192 | wait_or_delay | wait_or_delay |
| 24 | chasca-citlali-rosaria-bennett--citlali-c6-no-burst | informative | other | 74,803 | 78,847 | +5.4% | 82,273 | 1.85 | 91 | 80,875 / 82,248 / 82,273 | +1.9% | no (7,470 ± 291) | 201, 192, 192 | none | none |
| 25 | clorinde-kujousara-nahida-xilonen--DQkpHtnCkKgR | informative | frame | 69,222 | 75,795 | +9.5% | 72,399 | 0.48 | 91 | 71,434 / 71,202 / 72,399 | +1.7% | no (3,176 ± 42) | 202, 192, 199 | while,wait_or_delay | while,wait_or_delay |
| 26 | hutao-yelan-mona-nicole--NpWw8TkzPPDz | informative | other | 126,117 | 146,430 | +16.1% | 145,178 | 0.94 | 91 | 140,103 / 141,646 / 145,178 | +4.0% | no (19,061 ± 119) | 196, 196, 192 | wait_or_delay | none |
| 27 | hutao-yelan-columbina-ineffa--7DdP6dDqRGbC | informative | other | 111,409 | 119,605 | +7.4% | 113,604 | 0.27 | 11 | 113,604 / 111,833 / 111,793 | +1.6% | no (2,194 ± 201) | 192, 192, 192 | none | none |
| 28 | mavuika-citlali-xilonen-chevreuse--dkC8Gtdm8F9d | informative | frame | 115,361 | 1,025,563 | +789.0% | 615,239 | 0.55 | 91 | 547,980 / 570,861 / 615,239 | +58.3% | no (499,879 ± 1,699) | 192, 192, 192 | hold,wait_or_delay | hold,wait_or_delay |
| 29 | flins-ineffa-columbina-durin--kwnMBjKmcJWQ | informative | other | 145,453 | 166,698 | +14.6% | 150,872 | 0.26 | 37 | 150,980 / 150,872 / 150,904 | +0.1% | no (5,419 ± 281) | 192, 192, 192 | none | none |

## Checkpoint 576 CPU-s per start

| # | Case | Class | Stratum | L | A | A−L | S (selected) | r | Sel. start | S per start (11 / 37 / 91) | Start spread | Regression | CPU per start | A features | S features |
|---:|---|---|---|---:|---:|---:|---:|---:|---:|---|---:|---|---|---|---|
| 3 | shenhe-chasca-citlali-mavuika--citlali-c6-no-burst | informative | frame | 112,426 | 124,725 | +10.9% | 128,515 | 1.31 | 37 | 119,386 / 128,515 / 119,806 | +8.1% | no (16,088 ± 261) | 576, 576, 576 | hold,wait_or_delay | hold,wait_or_delay |
| 7 | wriothesley-xiangling-furina-bennett--mwjCWWCcpwJJ | informative | other | 164,877 | 225,564 | +36.8% | 232,746 | 1.12 | 11 | 232,746 / 215,926 / 217,050 | +10.2% | no (67,868 ± 462) | 578, 576, 576 | none | while |
| 11 | chongyun-furina-escoffier-xianyun--KPmGTPNQRDW6 | informative | frame | 46,605 | 49,914 | +7.1% | 48,641 | 0.62 | 91 | 48,275 / 47,007 / 48,641 | +3.5% | no (2,036 ± 90) | 576, 576, 579 | while | while |
| 18 | flins-ineffa-furina-sucrose--j86qrckHDPmb | informative | other | 131,352 | 149,428 | +13.8% | 141,962 | 0.59 | 37 | 135,014 / 141,962 / 139,336 | +5.3% | no (10,610 ± 165) | 576, 576, 578 | none | while |
| 25 | clorinde-kujousara-nahida-xilonen--DQkpHtnCkKgR | informative | frame | 69,197 | 75,864 | +9.6% | 73,027 | 0.57 | 91 | 71,378 / 72,248 / 73,027 | +2.4% | no (3,830 ± 42) | 577, 584, 579 | while,wait_or_delay | while,wait_or_delay |
| 26 | hutao-yelan-mona-nicole--NpWw8TkzPPDz | informative | other | 126,062 | 146,306 | +16.1% | 145,254 | 0.95 | 91 | 140,048 / 141,581 / 145,254 | +4.1% | no (19,192 ± 128) | 584, 576, 578 | wait_or_delay | none |
