# Cycle-synthesis pilot tables

## primary (A/L>=2) (n = 10)

| Arm @384 | median S/A | S/A < 0.90 | median r | regressions | selected audit failures | selected from synthesized family |
|---|---:|---:|---:|---:|---:|---:|
| V21S | 0.883 | 5 | 0.811 | 0 | 0 | 0 |
| SYN | 0.916 | 4 | 0.846 | 0 | 1 | 7 |
| XO | 0.957 | 3 | 0.951 | 0 | 0 | 8 |

| Comparison | median d (% of A) | x better | p | reading | start-averaged median d, p |
|---|---:|---:|---:|---|---|
| SYN vs V21S | +1.30% | 5/10 | 0.445 | no detectable difference at this sample size | +6.79%, 0.0593 |
| XO vs V21S | +4.82% | 6/10 | 0.0745 | no detectable difference at this sample size | +5.76%, 0.0745 |
| SYN vs XO | -2.11% | 4/10 | 0.285 | no detectable difference at this sample size | +0.87%, 0.721 |

Tail (S/A ≥ 0.90): SYN only 2, V21S only 1, exact McNemar p = 1.

## non-inferiority (A/L<1.25) (n = 9)

| Arm @384 | median S/A | S/A < 0.90 | median r | regressions | selected audit failures | selected from synthesized family |
|---|---:|---:|---:|---:|---:|---:|
| V21S | 0.951 | 1 | 0.489 | 0 | 0 | 0 |
| SYN | 0.932 | 2 | 0.478 | 0 | 0 | 2 |
| XO | 0.964 | 2 | 0.609 | 0 | 0 | 2 |

| Comparison | median d (% of A) | x better | p | reading | start-averaged median d, p |
|---|---:|---:|---:|---|---|
| SYN vs V21S | -1.85% | 2/9 | 0.0858 | no detectable difference at this sample size | -0.53%, 0.374 |
| XO vs V21S | +0.07% | 5/9 | 0.678 | no detectable difference at this sample size | +1.04%, 0.11 |
| SYN vs XO | -2.35% | 3/9 | 0.214 | no detectable difference at this sample size | -1.33%, 0.214 |

Tail (S/A ≥ 0.90): SYN only 0, V21S only 1, exact McNemar p = 1.

## control (n = 1)

| Arm @384 | median S/A | S/A < 0.90 | median r | regressions | selected audit failures | selected from synthesized family |
|---|---:|---:|---:|---:|---:|---:|
| V21S | 1.080 | 0 | -2.191 | 0 | 0 | 0 |
| SYN | 1.131 | 0 | -4.228 | 0 | 0 | 0 |
| XO | 1.100 | 0 | -2.976 | 0 | 0 | 0 |

| Comparison | median d (% of A) | x better | p | reading | start-averaged median d, p |
|---|---:|---:|---:|---|---|
| SYN vs V21S | +5.11% | 1/1 | 0.317 | no detectable difference at this sample size | +2.20%, 0.317 |
| XO vs V21S | +1.97% | 1/1 | 0.317 | no detectable difference at this sample size | +0.54%, 0.317 |
| SYN vs XO | +3.14% | 1/1 | 0.317 | no detectable difference at this sample size | +1.66%, 0.317 |

Tail (S/A ≥ 0.90): SYN only 0, V21S only 0, exact McNemar p = 1.

## Per case (selected start at 384 CPU-s; S/A)

| Case | stratum | A/L | V21S | SYN | XO | SYN selected family |
|---|---|---:|---:|---:|---:|---|
| noelle-furina-yelan-xilonen--cp6jwz87tHkb | control | 0.98 | 1.080 | 1.131 | 1.100 | structure-0 |
| alhaitham-furina-nahida-kukishinobu--PzrDR9zzLn9H | non-inferiority (A/L<1.25) | 1.11 | 0.943 | 0.946 | 0.969 | structure-0 |
| aratakiitto-albedo-gorou-furina--g6wbFkjbNKLc | non-inferiority (A/L<1.25) | 1.19 | 0.916 | 0.895 | 0.892 | synth-transplant-aratakiitto-albedo-gorou-xinyan |
| clorinde-kujousara-nahida-xilonen--DQkpHtnCkKgR | non-inferiority (A/L<1.25) | 1.10 | 0.942 | 0.938 | 0.964 | baseline / C0-on-field pattern |
| flins-ineffa-furina-sucrose--j86qrckHDPmb | non-inferiority (A/L<1.25) | 1.14 | 1.052 | 1.085 | 1.045 | synth-transplant-flins-fischl-columbina-sucrose |
| hutao-fischl-yelan-bennett--zmJmWDr9CMM8 | non-inferiority (A/L<1.25) | 1.11 | 1.003 | 0.965 | 1.004 | structure-0 |
| kinich-yaemiko-fischl-xilonen--Fqcp66H6wRq9 | non-inferiority (A/L<1.25) | 1.19 | 0.981 | 0.924 | 1.050 | structure-0 |
| mualani-chiori-xiangling-xilonen--fKzfLbrbfhj9 | non-inferiority (A/L<1.25) | 1.20 | 0.956 | 0.923 | 0.928 | structure-0 |
| mualani-mona-xilonen-mavuika--M8TgfHLLtqbp | non-inferiority (A/L<1.25) | 1.17 | 0.898 | 0.891 | 0.883 | structure-0 |
| varka-prune-nicole-durin--dDfWKHTKH8zK | non-inferiority (A/L<1.25) | 1.11 | 0.951 | 0.932 | 0.962 | baseline / C0-on-field pattern |
| arlecchino-bennett-citlali-xilonen--DnLT9CbL979h | primary (A/L>=2) | 2.29 | 0.812 | 0.806 | 0.846 | synth-transplant-arlecchino-sucrose-xilonen-bennett |
| cyno-furina-nahida-xilonen--jBkf669CgWCr | primary (A/L>=2) | 2.32 | 0.866 | 0.924 | 0.859 | synth-skeleton-skeleton D0 P1200 rules1 |
| kaedeharakazuha-furina-chongyun-xianyun--DrCQ7p7RghND | primary (A/L>=2) | 3.54 | 0.998 | 0.253 | 0.965 | structure-0 |
| mavuika-chevreuse-ororon-bennett--Mw9HdnKTkBfJ | primary (A/L>=2) | 11.86 | 0.700 | 1.032 | 0.955 | structure-0 |
| mualani-candace-sucrose-xiangling--WDNJCKpjFNqQ | primary (A/L>=2) | 5.98 | 0.825 | 0.856 | 0.960 | structure-0 |
| neuvillette-furina-citlali-xilonen--gLgzMjmTN8z6 | primary (A/L>=2) | 6.91 | 0.787 | 0.830 | 0.937 | synth-skeleton-skeleton D0 P1200 rules1 |
| raidenshogun-furina-xilonen-yelan--PhjwKhdjDwtb | primary (A/L>=2) | 2.02 | 0.918 | 0.908 | 0.899 | synth-skeleton-skeleton D0 P1200 rules1 |
| wriothesley-furina-xilonen-bennett--PFGMCrLDpbcq | primary (A/L>=2) | 2.73 | 0.958 | 0.953 | 1.020 | synth-transplant-wriothesley-yunjin-xilonen-bennett |
| wriothesley-kaedeharakazuha-shenhe-xilonen--bjFtkcfmqph7 | primary (A/L>=2) | 7.40 | 0.977 | 0.970 | 0.971 | synth-transplant-wriothesley-lanyan-shenhe-xilonen |
| yoimiya-furina-yunjin-xilonen--rk6QCLfwKJkD | primary (A/L>=2) | 2.47 | 0.900 | 1.036 | 1.034 | synth-transplant-yoimiya-furina-xilonen-bennett |

## Budget curve (median S/A of the selected start)

| Stratum | Arm | 96 | 192 | 384 |
|---|---|---:|---:|---:|
| primary (A/L>=2) | V21S | 0.818 | 0.865 | 0.883 |
| primary (A/L>=2) | SYN | 0.894 | 0.935 | 0.916 |
| primary (A/L>=2) | XO | 0.906 | 0.903 | 0.957 |
| non-inferiority (A/L<1.25) | V21S | 0.942 | 0.943 | 0.951 |
| non-inferiority (A/L<1.25) | SYN | 0.931 | 0.932 | 0.932 |
| non-inferiority (A/L<1.25) | XO | 0.956 | 0.955 | 0.964 |
| control | V21S | 1.073 | 1.077 | 1.080 |
| control | SYN | 1.088 | 1.127 | 1.131 |
| control | XO | 1.073 | 1.089 | 1.100 |
