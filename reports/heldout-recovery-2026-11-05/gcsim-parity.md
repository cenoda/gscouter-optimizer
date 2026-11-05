# gcsim parity check of stage-1 selections (reviewer, after the run)

Added after the results were published, by a separate review in the same session.
It changes no result above; it checks whether the gains hold on gcsim.

Conditions: `cenoda/gcsim@eb218847f82b` (the pin used by earlier parity reports),
run through Gscouter `6e0e82205` `internal/score` without fillers; 256 iterations,
seed 777000 (independent of all audit seeds), 450 s fight, first 150 s excluded.
For each case the 192 s selection S of the selected start was run unchanged. L and A
were run with S's header (build, target, energy, swap delay) and the same
`while 1` cycle wrapper. Native numbers are the report's 192 s audits.

- S differs from native by at most 0.7% (median 0.07%) across all 30 cases.
- Median r, 24 informative: native 0.94, gcsim 0.95.
- Median r, 19 database (non-variant) informative: native 0.74, gcsim 0.73.
- Post-hoc subset, not a pre-declared reading: database cases with A/L < 1.25
  (excluding cases where L breaks on the higher-constellation build), n=11:
  native 0.60, gcsim 0.57.
- One assumption parameter changed by the search was checked: Beidou `skill[counter=2]`
  in `nahida-fischl-beidou-kaedeharakazuha`. Setting it back to `counter=1` scores
  87,538 against 89,292 on gcsim (L 61,871 native), so it explains about 3 points of
  a +44% gain. Other selections still change hold durations (`hold=` 0–6) that the
  search is free to pick; whether each is achievable play was not checked.

Per-case values: [gcsim-parity.json](gcsim-parity.json).
