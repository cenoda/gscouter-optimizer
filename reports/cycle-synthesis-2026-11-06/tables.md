# Cycle-synthesis pilot tables

## primary (A/L>=2) (n = 25)

| Arm @384 | median S/A | S/A < 0.90 | median r | regressions | selected audit failures | selected from synthesized family | own-status guard |
|---|---:|---:|---:|---:|---:|---:|---:|
| V21S | 0.883 | 15 | 0.835 | 0 | 0 | 0 | 1 |
| SYN | 0.947 | 6 | 0.902 | 0 | 0 | 17 | 5 |
| XO | 0.943 | 8 | 0.915 | 0 | 0 | 11 | 0 |

| Comparison | median d (% of A) | x better | p | reading | start-averaged median d, p |
|---|---:|---:|---:|---|---|
| SYN vs V21S | +2.49% | 17/25 | 0.0214 | SYN beats V21S | +4.54%, 0.0324 |
| XO vs V21S | +3.26% | 19/25 | 0.011 | XO beats V21S | +7.29%, 0.00157 |
| SYN vs XO | -0.86% | 9/25 | 0.545 | no detectable difference at this sample size | -0.97%, 0.143 |

Tail (S/A ≥ 0.90): SYN only 9, V21S only 0, exact McNemar p = 0.00391.

## secondary (1.25<=A/L<2) (n = 15)

| Arm @384 | median S/A | S/A < 0.90 | median r | regressions | selected audit failures | selected from synthesized family | own-status guard |
|---|---:|---:|---:|---:|---:|---:|---:|
| V21S | 0.938 | 6 | 0.728 | 0 | 1 | 0 | 0 |
| SYN | 0.957 | 3 | 0.896 | 0 | 0 | 9 | 1 |
| XO | 0.976 | 4 | 0.944 | 0 | 0 | 6 | 0 |

| Comparison | median d (% of A) | x better | p | reading | start-averaged median d, p |
|---|---:|---:|---:|---|---|
| SYN vs V21S | +2.26% | 11/15 | 0.256 | no detectable difference at this sample size | +1.42%, 0.0995 |
| XO vs V21S | +1.65% | 10/15 | 0.281 | no detectable difference at this sample size | +2.44%, 0.0535 |
| SYN vs XO | +0.07% | 8/15 | 0.91 | no detectable difference at this sample size | -1.09%, 0.65 |

Tail (S/A ≥ 0.90): SYN only 4, V21S only 1, exact McNemar p = 0.375.

## non-inferiority (A/L<1.25) (n = 12)

| Arm @384 | median S/A | S/A < 0.90 | median r | regressions | selected audit failures | selected from synthesized family | own-status guard |
|---|---:|---:|---:|---:|---:|---:|---:|
| V21S | 1.002 | 1 | 1.012 | 0 | 0 | 0 | 0 |
| SYN | 1.007 | 0 | 1.062 | 0 | 0 | 8 | 0 |
| XO | 0.998 | 1 | 0.982 | 0 | 0 | 6 | 0 |

| Comparison | median d (% of A) | x better | p | reading | start-averaged median d, p |
|---|---:|---:|---:|---|---|
| SYN vs V21S | +1.49% | 9/12 | 0.0414 | SYN beats V21S | +1.40%, 0.0342 |
| XO vs V21S | +1.76% | 8/12 | 0.0844 | no detectable difference at this sample size | +2.10%, 0.015 |
| SYN vs XO | -0.18% | 6/12 | 0.814 | no detectable difference at this sample size | -0.46%, 0.388 |

Tail (S/A ≥ 0.90): SYN only 1, V21S only 0, exact McNemar p = 1.

## control (n = 8)

| Arm @384 | median S/A | S/A < 0.90 | median r | regressions | selected audit failures | selected from synthesized family | own-status guard |
|---|---:|---:|---:|---:|---:|---:|---:|
| V21S | 1.054 | 0 | -0.418 | 0 | 0 | 0 | 0 |
| SYN | 1.061 | 0 | -0.569 | 0 | 0 | 2 | 0 |
| XO | 1.084 | 0 | -0.818 | 0 | 0 | 6 | 0 |

| Comparison | median d (% of A) | x better | p | reading | start-averaged median d, p |
|---|---:|---:|---:|---|---|
| SYN vs V21S | +0.43% | 5/8 | 0.401 | no detectable difference at this sample size | +0.51%, 0.401 |
| XO vs V21S | +1.87% | 6/8 | 0.028 | XO beats V21S | +1.50%, 0.0251 |
| SYN vs XO | -1.40% | 0/8 | 0.0117 | XO beats SYN | -1.05%, 0.0357 |

Tail (S/A ≥ 0.90): SYN only 0, V21S only 0, exact McNemar p = 1.

## Per case (selected start at 384 CPU-s; S/A)

| Case | stratum | A/L | V21S | SYN | XO | SYN selected family |
|---|---|---:|---:|---:|---:|---|
| aratakiitto-travelergeo-gorou-zhongli--zRhRMFP6CRLg | control | 1.04 | 0.999 | 1.005 | 1.012 | baseline / C0-on-field pattern |
| charlotte-furina-shenhe-kaedeharakazuha--dzRdDDwTbPhJ | control | 0.86 | 1.215 | 1.213 | 1.215 | baseline / C0-on-field pattern |
| columbina-nilou-lauma-nahida--kHdhjLRzw9Tt | control | 1.01 | 1.009 | 1.023 | 1.033 | structure-0 |
| kaedeharakazuha-yaemiko-fischl-nahida--MN6777qTjHNP | control | 0.97 | 1.056 | 1.067 | 1.085 | synth-transplant-kaedeharakazuha-yaemiko-fischl-baizhu |
| klee-chiori-xilonen-furina--bjPhn8ppbKh9 | control | 0.83 | 1.961 | 2.919 | 15.637 | structure-0 |
| raidenshogun-chevreuse-durin-nicole--BbFnWtCQ79JG | control | 0.99 | 1.051 | 1.055 | 1.082 | structure-0 |
| raidenshogun-furina-kujousara-xilonen--nzH9PNLrRpzJ | control | 0.94 | 1.164 | 1.140 | 1.171 | structure-0 |
| varesa-chevreuse-durin-nicole--CghBhWfKQqgK | control | 1.03 | 1.040 | 1.033 | 1.035 | synth-transplant-varesa-fischl-columbina-durin |
| arlecchino-rosaria-kaeya-bennett--QWRCBGLDDbNC | non-inferiority (A/L<1.25) | 1.17 | 1.001 | 1.015 | 0.993 | synth-transplant-arlecchino-rosaria-sucrose-bennett |
| arlecchino-zhongli-chiori-bennett--T9gR8kNtdT7G | non-inferiority (A/L<1.25) | 1.18 | 1.002 | 0.995 | 0.971 | structure-0 |
| dehya-bennett-raidenshogun-chevreuse--8HbwrHWqbtkF | non-inferiority (A/L<1.25) | 1.10 | 1.082 | 1.102 | 1.142 | synth-transplant-raidenshogun-chevreuse-fischl-dehya |
| flins-fischl-columbina-nicole--B8tTztqwDgnR | non-inferiority (A/L<1.25) | 1.19 | 0.990 | 0.979 | 0.996 | structure-0 |
| flins-fischl-columbina-sucrose--crFc77PfHpCQ | non-inferiority (A/L<1.25) | 1.15 | 0.980 | 1.001 | 0.971 | synth-transplant-flins-ineffa-columbina-sucrose |
| kamisatoayaka-sangonomiyakokomi-venti-shenhe--TChhGFDHdWng | non-inferiority (A/L<1.25) | 1.10 | 0.990 | 1.002 | 1.033 | synth-transplant-kamisatoayaka-sangonomiyakokomi-lynette-shenhe |
| keqing-durin-nicole-chevreuse--WWK8Tr6bhBzW | non-inferiority (A/L<1.25) | 1.13 | 0.969 | 0.972 | 0.997 | synth-transplant-keqing-emilie-nicole-durin |
| navia-furina-chiori-bennett--NNzFfBLtHh7J | non-inferiority (A/L<1.25) | 1.06 | 1.037 | 1.044 | 1.070 | synth-transplant-navia-furina-albedo-bennett |
| raidenshogun-furina-kujousara-xilonen--6hR8LtNDdrtQ | non-inferiority (A/L<1.25) | 1.06 | 1.022 | 1.013 | 0.999 | structure-0 |
| raidenshogun-xiangling-dehya-chevreuse--bwG8nQQpNC7z | non-inferiority (A/L<1.25) | 1.07 | 1.129 | 1.167 | 1.211 | structure-0 |
| sangonomiyakokomi-nilou-collei-kirara--JWpNJ8TzB9rd | non-inferiority (A/L<1.25) | 1.15 | 0.864 | 0.919 | 0.900 | synth-skeleton-skeleton D3 P1126 rules1 |
| xiao-faruzan-zhongli-albedo--RqLrHQfhFJ7n | non-inferiority (A/L<1.25) | 1.07 | 1.054 | 1.109 | 1.062 | synth-transplant-xiao-jean-zhongli-albedo |
| bennett-xiangling-furina-kaedeharakazuha--NwtzKPQ7CPwc | primary (A/L>=2) | 2.07 | 0.627 | 0.635 | 0.683 | synth-skeleton-skeleton D0 P733 rules1 |
| charlotte-xiangling-xianyun-bennett--rTCzzmJwMr8g | primary (A/L>=2) | 3.97 | 0.794 | 0.996 | 1.020 | synth-transplant-chongyun-xiangling-bennett-xianyun |
| clorinde-furina-xilonen-chiori--fQHPqwGcjzF7 | primary (A/L>=2) | 2.20 | 0.946 | 0.943 | 0.909 | synth-transplant-clorinde-furina-nahida-xilonen |
| clorinde-mavuika-ororon-chevreuse--JztbbjLkhwKn | primary (A/L>=2) | 8.42 | 0.960 | 0.978 | 0.980 | structure-0 |
| dehya-emilie-durin-nicole--mbnhDmffpLgt | primary (A/L>=2) | 4.56 | 0.907 | 0.975 | 1.038 | synth-skeleton-skeleton D0 P960 rules1 |
| eula-furina-rosaria-escoffier--z9rTPFd8GtBh | primary (A/L>=2) | 2.92 | 0.889 | 0.966 | 0.937 | synth-transplant-eula-furina-shenhe-escoffier |
| furina-xiangling-xianyun-yelan--rDk7m6g7CjGM | primary (A/L>=2) | 2.01 | 1.059 | 0.951 | 0.962 | structure-0 |
| hutao-yelan-xingqiu-kaedeharakazuha--GhjMjQPnhgrm | primary (A/L>=2) | 3.47 | 0.883 | 0.947 | 0.943 | synth-skeleton-skeleton D1 P1270 rules1 |
| kaedeharakazuha-furina-bennett-xianyun--PnrGrChJrnLc | primary (A/L>=2) | 16.22 | 0.699 | 0.625 | 0.680 | synth-transplant-freminet-furina-bennett-xianyun |
| kaedeharakazuha-furina-nahida-bennett--DndWrPt9DNRQ | primary (A/L>=2) | 2.87 | 0.714 | 0.755 | 0.791 | synth-transplant-kaedeharakazuha-bennett-kukishinobu-nahida |
| keqing-chevreuse-fischl-bennett--TFdMDHQ9g6rW | primary (A/L>=2) | 2.17 | 0.993 | 0.940 | 1.004 | structure-0 |
| keqing-furina-nahida-xilonen--hnphQtKJJNQd | primary (A/L>=2) | 2.28 | 1.015 | 0.978 | 0.948 | synth-skeleton-skeleton D1 P810 rules1 |
| lyney-xiangling-kaedeharakazuha-bennett--bRrtzdr7jpGh | primary (A/L>=2) | 2.47 | 1.720 | 2.048 | 2.100 | synth-transplant-lyney-xilonen-kaedeharakazuha-bennett |
| mavuika-chasca-citlali-bennett--PtRKrwg9m8dH | primary (A/L>=2) | 2.41 | 0.658 | 0.904 | 0.763 | structure-0 |
| mualani-sucrose-travelerpyro-mona--KhBLfkWFDPd7 | primary (A/L>=2) | 3.17 | 0.700 | 0.685 | 0.616 | structure-0 |
| navia-xilonen-bennett-furina--KLFtfPP6WLWB | primary (A/L>=2) | 2.96 | 1.019 | 1.023 | 1.026 | synth-transplant-navia-xilonen-mavuika-bennett |
| neuvillette-furina-baizhu-kaedeharakazuha--GRJb7RnzKm8j | primary (A/L>=2) | 3.25 | 0.879 | 0.931 | 0.941 | synth-transplant-neuvillette-furina-kaedeharakazuha-ineffa |
| neuvillette-furina-zhongli-kaedeharakazuha--gkCP8FNwGqtw | primary (A/L>=2) | 2.34 | 0.905 | 0.912 | 0.938 | structure-0 |
| wriothesley-furina-shenhe-escoffier--rCGPdjnjt9NL | primary (A/L>=2) | 16.88 | 0.862 | 0.967 | 0.988 | synth-transplant-wriothesley-furina-escoffier-citlali |
| xianyun-faruzan-durin-nicole--8nDQTqkDgKqK | primary (A/L>=2) | 3.97 | 1.198 | 1.193 | 1.195 | structure-0 |
| xianyun-furina-xiangling-bennett--cBp9fPLQQpFf | primary (A/L>=2) | 2.20 | 0.606 | 0.631 | 0.607 | synth-transplant-chongyun-furina-xiangling-xianyun |
| xilonen-furina-yelan-kaedeharakazuha--CWB6gTBpHBPc | primary (A/L>=2) | 4.64 | 0.857 | 0.968 | 0.885 | synth-transplant-dori-furina-yelan-kaedeharakazuha |
| yelan-furina-xilonen-chiori--KkWbqJbtNwjb | primary (A/L>=2) | 2.34 | 0.835 | 0.960 | 0.858 | structure-0 |
| yoimiya-furina-yelan-bennett--PCKCJzbrDWCT | primary (A/L>=2) | 4.16 | 0.842 | 0.924 | 0.988 | synth-skeleton-skeleton D0 P1200 rules1 |
| yoimiya-yelan-furina-xilonen--LFWM8kDNzwpq | primary (A/L>=2) | 4.83 | 0.896 | 0.890 | 0.985 | synth-skeleton-skeleton D1 P1200 rules1 |
| arlecchino-furina-kaedeharakazuha-bennett--PWzB7dFwcBMq | secondary (1.25<=A/L<2) | 1.59 | 0.632 | 0.961 | 0.997 | synth-transplant-arlecchino-emilie-kaedeharakazuha-bennett |
| dehya-bennett-furina-kaedeharakazuha--BnjBwTgnP6HT | secondary (1.25<=A/L<2) | 1.28 | 0.938 | 0.944 | 0.922 | structure-0 |
| dehya-bennett-rosaria-citlali--tmjgqKrTWCpC | secondary (1.25<=A/L<2) | 1.93 | 1.027 | 0.957 | 0.968 | structure-0 |
| hutao-columbina-fischl-mona--wWBKQkFnFdDk | secondary (1.25<=A/L<2) | 1.74 | 0.792 | 0.945 | 0.900 | synth-transplant-hutao-columbina-fischl-ineffa |
| hutao-xingqiu-yelan-fischl--gwwwczFcrFKg | secondary (1.25<=A/L<2) | 1.68 | 0.779 | 0.822 | 0.794 | structure-0 |
| kamisatoayato-columbina-lauma-ineffa--tkD7GgpJ6wCz | secondary (1.25<=A/L<2) | 1.81 | 0.791 | 0.941 | 0.978 | synth-transplant-columbina-ineffa-lauma-mona |
| kamisatoayato-furina-kaedeharakazuha-xilonen--dMk9F9HqRMjp | secondary (1.25<=A/L<2) | 1.76 | 1.011 | 1.036 | 0.976 | structure-0 |
| mualani-mona-sucrose-mavuika--cfDJ8TCdpgGR | secondary (1.25<=A/L<2) | 1.30 | 0.925 | 0.814 | 0.858 | synth-transplant-mualani-candace-sucrose-mavuika |
| mualani-xilonen-sucrose-mavuika--jRWpgmRPkqKD | secondary (1.25<=A/L<2) | 1.51 | 0.781 | 0.794 | 0.821 | structure-0 |
| neuvillette-furina-citlali-escoffier--6nwfwkqzfHWR | secondary (1.25<=A/L<2) | 1.37 | 0.970 | 1.009 | 1.010 | synth-skeleton-skeleton D0 P1200 rules1 |
| noelle-furina-gorou-xilonen--jwFpbrDQLc7g | secondary (1.25<=A/L<2) | 1.96 | 1.076 | 0.975 | 1.113 | synth-transplant-noelle-furina-gorou-chiori |
| raidenshogun-furina-fischl-nahida--DRMbHJmRQwkP | secondary (1.25<=A/L<2) | 1.27 | 1.025 | 1.027 | 1.033 | synth-transplant-raidenshogun-fischl-yelan-nahida |
| sethos-nahida-fischl-xilonen--FRtpGQLNrKhN | secondary (1.25<=A/L<2) | 1.46 | 1.181 | 1.110 | 1.041 | structure-0 |
| skirk-furina-mona-escoffier--HDjJgKcgtqzf | secondary (1.25<=A/L<2) | 1.35 | 1.022 | 1.044 | 1.034 | synth-skeleton-skeleton D0 P1200 rules1 |
| yelan-furina-escoffier-citlali--FTjTFjpN7fpm | secondary (1.25<=A/L<2) | 1.47 | 0.847 | 0.936 | 0.917 | synth-skeleton-skeleton D0 P1200 rules1 |

## Budget curve (median S/A of the selected start)

| Stratum | Arm | 96 | 192 | 384 |
|---|---|---:|---:|---:|
| primary (A/L>=2) | V21S | 0.830 | 0.859 | 0.883 |
| primary (A/L>=2) | SYN | 0.912 | 0.938 | 0.947 |
| primary (A/L>=2) | XO | 0.909 | 0.936 | 0.943 |
| secondary (1.25<=A/L<2) | V21S | 0.931 | 0.963 | 0.938 |
| secondary (1.25<=A/L<2) | SYN | 0.937 | 0.943 | 0.957 |
| secondary (1.25<=A/L<2) | XO | 0.941 | 0.967 | 0.976 |
| non-inferiority (A/L<1.25) | V21S | 0.973 | 0.983 | 1.002 |
| non-inferiority (A/L<1.25) | SYN | 0.988 | 0.998 | 1.007 |
| non-inferiority (A/L<1.25) | XO | 0.992 | 0.997 | 0.998 |
| control | V21S | 1.054 | 1.052 | 1.054 |
| control | SYN | 1.063 | 1.061 | 1.061 |
| control | XO | 1.067 | 1.074 | 1.084 |
