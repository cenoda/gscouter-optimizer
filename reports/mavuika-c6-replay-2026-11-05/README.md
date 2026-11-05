# Mavuika high-score replay investigation

**Status: the approximately 3M DPS execution has not been reproduced.** No
engine defect has been established. The engine revision and optimizer search
implementation were held fixed. The new code is standalone replay diagnostics,
not an improved discovery algorithm or a successful blind rediscovery.

The owner's working hypothesis is that the supplied rotation is inadequate.
The investigation now has an actual high-score author's input sequence rather
than only a weaker database cycle. Its GCSL transcription remains unverified.
Keep `gse` fixed while resolving that transcription and its setup.

## Primary sources obtained

- [Lin-Eric's 3.23M video](https://www.bilibili.com/video/BV1jaK36kE7s/):
  public playback, CID `40058487700`, 130-second recording. The final display
  reads 120.00 seconds, 388,293,693 total damage and 3,235,781 DPS.
- [The 3.31M video](https://www.bilibili.com/video/BV1nhsd6dEvM/): CID
  `43139991334`, 216 seconds including an introductory montage. Its public
  description supplies a main input sequence and a different ending sequence.
- [The 3.08M total-damage-cup video](https://www.bilibili.com/video/BV1qKEp6GE9q/):
  CID `38915606250`, 139 seconds. Inspected as an additional reference lead.

The public `x/player/pagelist`, `x/player/playurl` and
`x/web-interface/archive/desc` endpoints work even though the normal page and
view-metadata endpoint returned 412. The complete videos stay in ignored local
`runs/`; they are not redistributed here. Source identities, hashes and relevant
observations are in [source-observations.json](source-observations.json) and
[source-331-notation.json](source-331-notation.json). Signed playback URLs,
visitor metadata and unrelated public comments are not retained in this report.

The 3.31M author's technical notation is:

```text
Main: RCQzs ZhzzPsz PszPszZsZhzcFdc Fecdcdc
End:  CdcF QZhzzPszPszZszhZ
```

The description also states 45.2 useful rolls for Mavuika. A complete artifact
allocation, exact buffered input timings and all setup details were not obtained.
The shorthand-to-GCSL mapping in `experiments/notation.py` is an inference:
`C/c/Z/z` are treated as cyclic charged hits, `P/F` as finishers, `s/d` as
colliding charged-dash cancels, `h` as a noncolliding cancel, `R` as a recast,
and `e` as a skill refresh. Uppercase/lowercase is interpreted as intended Melt
outcomes. **These interpretations are not author-confirmed.** Additional guards,
the ending's 108-second trigger, and the skill-refresh fallback are reconstruction
choices, not supplied frame instructions. The resulting configs fail to reproduce
the recorded score and are not installed as validated library rotations.

This sequence differs substantially from repeating `C1F` and a dash. It mixes
cyclic-only cancels, additional held cyclic hits, finishers, hidden dash hits,
and a late skill refresh. Testing only repeated finisher groups omitted these
structures.

## Video observations and corrected initial interpretation

In the 3.23M recording, Mavuika's first skill hit is followed by Nicole, Citlali,
Xilonen and Mavuika taking the field again. Mavuika already deals off-field
damage during the support setup. An early contact-sheet interpretation that
there was no initial Mavuika skill, and that the pre-burst 2,335,673 hit was
definitely a finisher, was incorrect. Those assumptions were withdrawn.

The observed opening is consistent with an initial tap skill, Nicole's skill,
Citlali's skill, Xilonen's skill and two normals, then Mavuika's recast, charged
hit and burst. The precise frame inputs remain unresolved. The video shows a
pre-burst 2,335,673 hit and a 10,058,282 burst hit; damage numbers alone do not
certify the complete action sequence.

Visible setup differs from the earlier fixed search build:

| Mavuika panel | Earlier frozen build | 3.23M video, idle |
|---|---:|---:|
| Character level | 90 | 100 |
| ATK | 1,827.32 | 2,861 |
| DEF | 929.13 | 848 |
| CRIT rate | 55.7456% | 59% |
| CRIT damage | 216.8% | 252% |
| ER | 111.02% | 100% |

The recording shows a level-110 target and a 120-second window beginning with
empty energy indicators. The earlier study used a level-100 target, 450 seconds
with the first 150 excluded, periodic energy and a 12-frame swap delay. The
video does not expose every relevant setup value. A one-frame swap delay was
tested as a diagnostic assumption; it is not a verified video setting.

The `panel` configs reconstruct only Mavuika's displayed idle stats as aggregate
inputs, with all characters at level 100 and signature R5 weapons. Support gear
remains from the earlier fixture. Mavuika's unseen EM is varied explicitly;
neither the exact artifact pieces nor their legal allocation are established.
These are sensitivity checks, not faithful whole-team equipment reproductions.

## Measurements and unsuccessful reconstructions

All rows below use the pinned native engine, 2,048 fresh fights with seed
`31000000000000`. Candidates were frozen before these audits. Mean DPS and
standard error are preserved in [audit/summary.json](audit/summary.json), along
with every complete config and per-fight result.

| Config | Duration / warmup | Mean DPS | SE |
|---|---:|---:|---:|
| Published KQM cycle, original fixed level-90 build | 450 / 150 s | 1,890,472 | 579 |
| Post-reference timing refinement, identical fixed build | 450 / 150 s | 1,885,370 | 574 |
| Published KQM cycle, partial HUD reconstruction, EM 79.28 | 120 / 0 s | 2,238,411 | 825 |
| Initial video-prefix approximation, same partial HUD inputs | 120 / 0 s | 1,784,790 | 575 |
| Refined approximation, same partial HUD inputs | 120 / 0 s | 2,135,280 | 676 |
| Refined approximation, EM 158.56 sensitivity | 120 / 0 s | 2,216,718 | 709 |
| Refined approximation, EM 266.28 sensitivity | 120 / 0 s | 2,316,558 | 750 |

The timing refinement of the published cycle failed independent confirmation;
it is worse than the original and is discarded. The video-prefix refinement
improves its own initial approximation but remains worse than the published
cycle under the same partial HUD inputs. No successful replacement rotation is
claimed. The later direct shorthand transcription and geometry sensitivities
also failed; their exploratory results and configs are retained under
`notation/`. They were not promoted to a benchmark or subjected to a claim of
successful independent confirmation.

One diagnostic seed of the opening approximation produced a first burst at
5.90 seconds for **8,987,185 damage**, versus the video's 10,058,282. With a
12-frame swap delay, that approximation instead produced a non-Melt burst at
6.633 seconds for 2,846,894. This illustrates why correct timing and reaction
placement matter; it does not establish that either assumed swap setting is
the video's setting. A near burst hit does not establish 120-second DPS parity.
See [timeline-video-swap1.json](timeline-video-swap1.json) and
[timeline-video-approx.json](timeline-video-approx.json).

## New diagnostic code and verification

- `examples/replay.rs` accepts the config's own duration and an explicit warmup,
  reports independent-fight DPS, ability hit counts/damage and panel stats, and
  rejects unsupported inputs or incomplete fights.
- `examples/timeline.rs` recovers early damage events by replaying a single seed
  with successive one-frame horizons, using only public engine APIs. It rejects
  decreasing ability prefixes and damage-mode targets.

Commands and limitations are in [the diagnostic guide](../../docs/replay.md).
For example:

```sh
cargo build --release --example replay --example timeline
target/release/examples/replay reports/mavuika-c6-replay-2026-11-05/audit/published-fixed90-450.gcsl 2048 31000000000000 150
target/release/examples/timeline reports/mavuika-c6-replay-2026-11-05/video-approx-swap1.gcsl 28000000000000 1200
```

The 20-second timeline sums to 34,940,470.49089437 damage. A single replay of the
same 20-second config and seed measures 34,940,470.490894355, a difference of
`1.49e-8` damage. `cargo test --all-targets` passes all 13 existing tests;
formatting and strict all-target Clippy checks pass. No engine, character or
damage implementation was changed.

## Experiment scope, costs and remaining work

All explorations took place after external references were revealed. They are
diagnostic experiments, not the blind pilot, a fair-budget algorithm comparison,
or protocol-compliant annealing acceptance decisions. Most exploratory rankings
used eight fights; final measurements above use independent seeds and 2,048
fights. The original blind selection was not replaced or rerun. Its previous
973K result cannot be compared with a different HUD-reconstructed build to claim
discovery improvement.

[experiment-accounting.json](experiment-accounting.json) records 7,207 candidate
evaluations across the six initial exploratory groups, including rejected and
failed proposals. Their compressed summaries and generating scripts are retained
in `experiments/`. Recorded child CPU for the five metered groups totals about
287 seconds; the first 16-case ablation recorded wall time only. Independent
audits add about 46 seconds of child CPU. Later notation/geometry diagnostics,
source research, compilation, timeline checks and test/report work were not
fully metered and are additional costs. These figures are not an end-to-end
budget or throughput claim. Archived research scripts preserve the original
ignored `runs/` paths; the complete audited configs above can be rerun directly.

The immediate missing result is a verified action/frame reconstruction of the
author's sequence under sufficiently specified equipment and initial conditions.
Only after that replay succeeds should it become the held-out quality target.
The current generated grammar also lacks the action-parameter maps needed for
explicit finishers, held/recast skills and charged/hidden dash cancels. This
investigation adds diagnostics and evidence; it does not repair that search
representation or establish that the pilot can discover the 3M execution.
