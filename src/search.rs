use crate::{
    artifact::{ENGINE, Manifest},
    evaluate::{Evaluator, Sample, atomic, cpu, hash, paired, stats},
    program::{Program, Rng},
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

type R<T> = Result<T, String>;
const SEARCH_SEED: i64 = 1_000_000;
const PROMOTION_SEED: i64 = 1_000_000_000;
const AUDIT_SEED: i64 = 4_000_000_000_000;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Candidate {
    pub program: Program,
    pub allocation: Manifest,
    pub role: String,
    pub parent: Option<String>,
}
impl Candidate {
    fn key(&self) -> String {
        Evaluator::key(&Evaluator::config(&self.allocation, &self.program))
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct Chain {
    current: Candidate,
    steps: usize,
    accepted: usize,
    temperature: f64,
    score: f64,
}
#[derive(Serialize, Deserialize)]
struct State {
    format: u8,
    identity: String,
    rng: Rng,
    chains: Vec<Chain>,
    archive: Vec<Candidate>,
    best: Candidate,
    baseline: Candidate,
    baseline_alloc: Candidate,
    step: usize,
    promotions: usize,
    best_hint: f64,
    reference_dps: f64,
    parent_cpu: f64,
    events: Vec<Event>,
    curve: Vec<serde_json::Value>,
    pending: Option<Pending>,
    polish: Option<Polish>,
}
#[derive(Clone, Serialize, Deserialize)]
struct Polish {
    anchor: String,
    cursor: usize,
}
#[derive(Clone, Serialize, Deserialize)]
struct Pending {
    next: Candidate,
    family: usize,
    decision_seed: i64,
    u: f64,
    baseline_trial: bool,
    repairs_done: bool,
    reference: Candidate,
    operator: String,
    polishing: bool,
}
#[derive(Serialize, Deserialize)]
struct Event {
    step: usize,
    family: usize,
    decision_seed: i64,
    samples: usize,
    accepted: bool,
    delta: f64,
    threshold: f64,
    kind: String,
    cpu: f64,
    current_key: String,
    proposal_key: String,
    operator: String,
    phase: String,
}
#[derive(Clone)]
struct Options {
    manifest: String,
    baseline: String,
    out: PathBuf,
    seconds: f64,
    seed: u64,
    algorithm: String,
    local: bool,
    joint: bool,
    audit: usize,
    revised: bool,
    restricted: bool,
}

fn proposal(
    c: &Candidate,
    frozen: &Manifest,
    rng: &mut Rng,
    joint: bool,
    force_alloc: bool,
    revised: bool,
) -> (Candidate, String) {
    let mut next = c.clone();
    next.parent = Some(c.key());
    if joint && (force_alloc || rng.index(100) < 35) {
        for _ in 0..100 {
            let ci = rng.index(4);
            let pi = rng.index(5);
            let ti = rng.index(9);
            let choices = &next.allocation.members[ci].useful;
            let stat = if rng.index(2) == 0 {
                "er".into()
            } else {
                choices[rng.index(choices.len())].clone()
            };
            if let Some(m) = next.allocation.exchange(frozen, ci, pi, ti, &stat) {
                next.allocation = m;
                return (next, format!("allocation exchange {ci}:{pi}:{ti}->{stat}"));
            }
        }
    }
    if !force_alloc {
        if revised {
            let (program, label) = next.program.mutate_revised(rng);
            next.program = program;
            return (next, label);
        }
        next.program = next.program.mutate(rng);
    }
    (next, "legacy action mutation".into())
}

fn record_proposal(out: &Path, step: usize, pending: &Pending) -> R<()> {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(out.join("proposals.jsonl"))
        .map_err(|e| e.to_string())?;
    serde_json::to_writer(
        &mut f,
        &serde_json::json!({"step":step,
        "phase":if pending.polishing {"polish"} else {"discovery"},
        "operator":pending.operator,"repairs_done":pending.repairs_done,"reference_key":pending.reference.key(),
        "key":pending.next.key(),"candidate":pending.next}),
    )
    .map_err(|e| e.to_string())?;
    f.write_all(b"\n")
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())
}

fn polish_neighbors(c: &Candidate, frozen: &Manifest, joint: bool) -> Vec<(Candidate, String)> {
    let mut programs = c.program.local_neighbors().into_iter();
    let mut allocations = Vec::new();
    if joint {
        // Round-robin token positions; character ownership and physical
        // feasibility remain enforced by Manifest::exchange.
        for ti in 0..9 {
            for pi in 0..5 {
                for ci in 0..4 {
                    for stat in &c.allocation.members[ci].useful {
                        if let Some(a) = c.allocation.exchange(frozen, ci, pi, ti, stat) {
                            allocations
                                .push((a, format!("polish allocation {ci}:{pi}:{ti}->{stat}")));
                        }
                    }
                }
            }
        }
    }
    let mut allocations = allocations.into_iter();
    let mut result = Vec::new();
    let mut seen = std::collections::BTreeSet::from([c.key()]);
    loop {
        let mut produced = false;
        for allocation_turn in [false, true] {
            let item = if allocation_turn {
                allocations.next().map(|(a, label)| {
                    (
                        Candidate {
                            allocation: a,
                            ..c.clone()
                        },
                        label,
                    )
                })
            } else {
                programs.next().map(|(p, label)| {
                    (
                        Candidate {
                            program: p,
                            ..c.clone()
                        },
                        label,
                    )
                })
            };
            if let Some((mut next, label)) = item {
                produced = true;
                next.parent = Some(c.key());
                if seen.insert(next.key()) {
                    result.push((next, label));
                }
            }
        }
        if !produced || result.len() >= 256 {
            break;
        }
    }
    result.truncate(256);
    result
}
fn eval(e: &mut Evaluator, c: &Candidate, seed: i64, n: usize) -> R<Vec<Sample>> {
    e.eval(&c.allocation, &c.program, seed, n)
}
fn spent(e: &Evaluator, carried: f64, parent_start: f64, worker_start: f64) -> f64 {
    carried + cpu() - parent_start + e.worker_cpu - worker_start
}

fn initialize(o: &Options, m: &Manifest, baseline: &str, identity: &str) -> State {
    let mut initial_rng = Rng(o.seed);
    let base = Candidate {
        program: if o.restricted {
            Program::restricted_domain()[initial_rng.index(8)].clone()
        } else {
            Program::Library(baseline.into())
        },
        allocation: m.clone(),
        role: "baseline / C0-on-field pattern".into(),
        parent: None,
    };
    let programs = if o.restricted {
        vec![base.program.clone()]
    } else {
        Program::families(baseline)
    };
    let programs = if o.local && !o.restricted {
        vec![programs[0].clone()]
    } else {
        programs
    };
    let chains = programs
        .into_iter()
        .enumerate()
        .map(|(i, p)| Chain {
            current: Candidate {
                program: p,
                allocation: m.clone(),
                role: format!("structure-{i}"),
                parent: None,
            },
            steps: 0,
            accepted: 0,
            temperature: 0.015,
            score: 0.0,
        })
        .collect();
    State {
        format: 2,
        identity: identity.into(),
        rng: initial_rng,
        chains,
        archive: vec![base.clone()],
        best: base.clone(),
        baseline: base.clone(),
        baseline_alloc: base,
        step: 0,
        promotions: 0,
        best_hint: 0.0,
        reference_dps: 0.0,
        parent_cpu: 0.0,
        events: Vec::new(),
        curve: Vec::new(),
        pending: None,
        polish: None,
    }
}

fn run(o: &Options) -> R<()> {
    if !o.seconds.is_finite() || o.seconds <= 0.0 || o.audit == 0 {
        return Err("positive CPU seconds and audit count required".into());
    }
    if o.restricted && o.joint {
        return Err("restricted-domain recovery requires --fixed-er".into());
    }
    let parent_start = cpu();
    let wall = Instant::now();
    let m = Manifest::load(&o.manifest)?;
    // This executable currently supports this declared four-member pilot,
    // rather than silently applying its mechanics to an unrelated party.
    if m.members.iter().map(|m| m.key.as_str()).collect::<Vec<_>>() != crate::program::NAMES {
        return Err("this pilot supports the declared Flins party only".into());
    }
    if m.members
        .iter()
        .any(|m| m.key != "sucrose" && ![1, 2, 4, 6].contains(&m.cons))
        || m.members[3].cons != 6
    {
        return Err("ineligible new-discovery constellation".into());
    }
    let baseline = fs::read_to_string(&o.baseline).map_err(|e| e.to_string())?;
    if !baseline.trim_start().starts_with("active ") {
        return Err("pilot baseline must begin with active setup".into());
    }
    let identity = hash(&format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        serde_json::to_string(&m).unwrap(),
        baseline,
        o.seed,
        o.algorithm,
        o.local,
        o.joint,
        o.revised,
        o.restricted,
        implementation_hash()
    ));
    fs::create_dir_all(&o.out).map_err(|e| e.to_string())?;
    let checkpoint = o.out.join("checkpoint.json");
    let mut state: State = if checkpoint.exists() {
        serde_json::from_slice(&fs::read(&checkpoint).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
    } else {
        initialize(o, &m, &baseline, &identity)
    };
    if state.identity != identity || state.format != 2 {
        return Err("checkpoint experiment identity mismatch".into());
    }
    let mut e = Evaluator::new(&o.out)?;
    let carried = state.parent_cpu;
    let worker_start = e.worker_cpu;
    if state.best_hint == 0.0 {
        state.best_hint = stats(&eval(
            &mut e,
            &state.baseline,
            if o.restricted {
                6_000_000_000_000
            } else {
                SEARCH_SEED
            },
            if o.restricted { 32 } else { 16 },
        )?)
        .mean;
        state.reference_dps = state.best_hint;
    }
    let initial_dps = state.reference_dps.max(1.0);
    let decision_count = if o.restricted { 32 } else { 16 };
    let mut polish_cache: Option<(String, Vec<(Candidate, String)>)> = None;
    while spent(&e, carried, parent_start, worker_start) < o.seconds {
        let polishing = state.pending.as_ref().map(|p| p.polishing).unwrap_or(
            o.revised
                && !o.restricted
                && spent(&e, carried, parent_start, worker_start) >= o.seconds * 0.75,
        );
        let family = if let Some(p) = &state.pending {
            p.family
        } else if state.step < state.chains.len() * 4 || state.step.is_multiple_of(5) {
            state.step % state.chains.len()
        } else {
            state
                .chains
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| {
                    let priority = |c: &Chain| {
                        c.score / initial_dps
                            + 0.05
                                * ((state.step as f64 + 1.0).ln() / (c.steps as f64 + 1.0)).sqrt()
                    };
                    priority(a).total_cmp(&priority(b))
                })
                .unwrap()
                .0
        };
        // One family is executing; all other family states are paused. This
        // serial implementation uses at most one of the two permitted slots.
        let current = state.chains[family].current.clone();
        let baseline_trial = !polishing && o.joint && state.step % 5 == 4;
        let origin = if polishing {
            state.best.clone()
        } else if baseline_trial {
            state.baseline_alloc.clone()
        } else if o.algorithm == "random" {
            initialize(o, &m, &baseline, &identity).chains[family]
                .current
                .clone()
        } else {
            current.clone()
        };
        if state.pending.is_none() {
            let (mut next, mut operator) = if polishing {
                let anchor = origin.key();
                if state.polish.as_ref().is_none_or(|p| p.anchor != anchor) {
                    state.polish = Some(Polish { anchor, cursor: 0 });
                }
                if polish_cache
                    .as_ref()
                    .is_none_or(|(key, _)| key != &origin.key())
                {
                    polish_cache = Some((origin.key(), polish_neighbors(&origin, &m, o.joint)));
                }
                let neighbors = &polish_cache.as_ref().unwrap().1;
                let cursor = state.polish.as_ref().unwrap().cursor;
                let Some(next) = neighbors.get(cursor).cloned() else {
                    break;
                };
                next
            } else if state.chains[family].steps == 0 && !baseline_trial {
                (origin.clone(), "initial family evaluation".into())
            } else if o.restricted {
                let domain = Program::restricted_domain();
                let index = domain
                    .iter()
                    .position(|p| p == &origin.program)
                    .ok_or("candidate outside restricted domain")?;
                let bit = state.rng.index(3);
                (
                    Candidate {
                        program: domain[index ^ (1 << bit)].clone(),
                        parent: Some(origin.key()),
                        ..origin.clone()
                    },
                    format!("restricted bit {bit}"),
                )
            } else {
                proposal(
                    &origin,
                    &m,
                    &mut state.rng,
                    o.joint,
                    baseline_trial,
                    o.revised,
                )
            };
            if o.algorithm == "random" && !baseline_trial && state.chains[family].steps > 0 {
                for _ in 0..state.rng.index(6) {
                    (next, operator) =
                        proposal(&next, &m, &mut state.rng, o.joint, false, o.revised);
                }
            }
            state.pending = Some(Pending {
                next,
                family,
                decision_seed: if o.restricted {
                    6_000_000_000_000
                } else {
                    SEARCH_SEED + ((state.step / 4) * 64) as i64
                },
                u: state.rng.uniform(),
                baseline_trial,
                repairs_done: false,
                reference: if polishing {
                    origin.clone()
                } else if baseline_trial {
                    state.baseline_alloc.clone()
                } else {
                    current.clone()
                },
                operator,
                polishing,
            });
            record_proposal(&o.out, state.step, state.pending.as_ref().unwrap())?;
        }
        let pending = state.pending.clone().unwrap();
        let Pending {
            mut next,
            family,
            decision_seed,
            u,
            baseline_trial,
            repairs_done,
            reference,
            operator,
            polishing,
        } = pending;
        let phase = if polishing { "polish" } else { "discovery" }.to_string();
        if o.revised && operator != "initial family evaluation" && next.key() == reference.key() {
            state.events.push(Event {
                step: state.step,
                family,
                decision_seed,
                samples: 0,
                accepted: false,
                delta: 0.0,
                threshold: 0.0,
                kind: "exact no-op".into(),
                cpu: spent(&e, carried, parent_start, worker_start),
                current_key: reference.key(),
                proposal_key: next.key(),
                operator,
                phase,
            });
            state.chains[family].steps += 1;
            state.step += 1;
            state.pending = None;
            if polishing {
                state.polish.as_mut().unwrap().cursor += 1;
            }
            continue;
        }
        let mut temperature = state.chains[family].temperature;
        let mut threshold = if o.algorithm == "anneal" && !baseline_trial && !polishing {
            temperature * u.ln()
        } else {
            0.0
        };
        let mut completed = None;
        let mut failed = None;
        for n in [2, 4, 8, 16, 32]
            .into_iter()
            .filter(|n| *n <= decision_count)
        {
            if spent(&e, carried, parent_start, worker_start) >= o.seconds {
                break;
            }
            match eval(&mut e, &next, decision_seed, n) {
                Ok(s) => {
                    if n == decision_count {
                        completed = Some(s);
                    }
                }
                Err(err) => {
                    failed = Some(err);
                    break;
                }
            }
        }
        if let Some(err) = failed {
            state.events.push(Event {
                step: state.step,
                family,
                decision_seed,
                samples: 0,
                accepted: false,
                delta: 0.0,
                threshold,
                kind: format!("failed/unresolved: {err}"),
                cpu: spent(&e, carried, parent_start, worker_start),
                current_key: reference.key(),
                proposal_key: next.key(),
                operator: operator.clone(),
                phase: phase.clone(),
            });
        } else if let Some(mut y) = completed {
            if spent(&e, carried, parent_start, worker_start) >= o.seconds {
                break;
            }
            // At most two nearby allocations repair a selected structural
            // change. Every full comparison is charged to this same cap.
            // An unfinished repair remains a proposal, never an installed state.
            if o.joint
                && !polishing
                && !baseline_trial
                && !repairs_done
                && state.step % 4 == 3
                && next.program != origin.program
            {
                state.pending.as_mut().unwrap().repairs_done = true;
                let diagnostic = stats(&y);
                let ci = (0..4)
                    .max_by(|&a, &b| {
                        diagnostic.waits[a]
                            .get("insufficient_energy")
                            .unwrap_or(&0.0)
                            .total_cmp(
                                diagnostic.waits[b]
                                    .get("insufficient_energy")
                                    .unwrap_or(&0.0),
                            )
                    })
                    .unwrap();
                let add_er = if o.revised {
                    diagnostic.waits[ci]
                        .get("insufficient_energy")
                        .copied()
                        .unwrap_or(0.0)
                        > 1.0
                } else {
                    diagnostic.waits[ci].values().sum::<f64>() > 1.0
                };
                let mut repaired = next.clone();
                for _ in 0..2 {
                    let mut allocation = None;
                    'pieces: for pi in 0..5 {
                        for ti in 0..9 {
                            let choices = if add_er {
                                vec!["er".into()]
                            } else {
                                repaired.allocation.members[ci].useful.clone()
                            };
                            for s in choices {
                                if let Some(a) = repaired.allocation.exchange(&m, ci, pi, ti, &s) {
                                    allocation = Some(a);
                                    break 'pieces;
                                }
                            }
                        }
                    }
                    let Some(a) = allocation else {
                        break;
                    };
                    repaired.allocation = a;
                    if spent(&e, carried, parent_start, worker_start) >= o.seconds {
                        break;
                    }
                    if let Ok(s) = eval(&mut e, &repaired, decision_seed, 16)
                        && stats(&s).mean > stats(&y).mean
                    {
                        next = repaired.clone();
                        y = s;
                    }
                }
                state.pending.as_mut().unwrap().next = next.clone();
                record_proposal(&o.out, state.step, state.pending.as_ref().unwrap())?;
            }
            if spent(&e, carried, parent_start, worker_start) >= o.seconds {
                break;
            }
            let reference_key = reference.key();
            match eval(&mut e, &reference, decision_seed, decision_count) {
                Err(err) => state.events.push(Event {
                    step: state.step,
                    family,
                    decision_seed,
                    samples: decision_count,
                    accepted: false,
                    delta: 0.0,
                    threshold,
                    kind: format!("unresolved incumbent: {err}"),
                    cpu: spent(&e, carried, parent_start, worker_start),
                    current_key: reference_key.clone(),
                    proposal_key: next.key(),
                    operator: operator.clone(),
                    phase: phase.clone(),
                }),
                Ok(x) => {
                    let delta = (stats(&y).mean - stats(&x).mean) / initial_dps;
                    if state.chains[family].steps == 1 && !baseline_trial && !polishing {
                        // Bootstrap the paired block differences to calibrate
                        // noise scale, separately from the acceptance draw u.
                        let mut bootstrap = Rng(0x43414c4942524154u64 ^ family as u64 ^ o.seed);
                        let mut draws = Vec::new();
                        for _ in 0..64 {
                            draws.push(
                                (0..16)
                                    .map(|_| {
                                        let j = bootstrap.index(16);
                                        y[j].dps - x[j].dps
                                    })
                                    .sum::<f64>()
                                    / 16.0,
                            );
                        }
                        let center = draws.iter().sum::<f64>() / 64.0;
                        let noise =
                            (draws.iter().map(|v| (v - center).powi(2)).sum::<f64>() / 63.0).sqrt()
                                / initial_dps;
                        temperature = (2.0 * noise).max(delta.abs() * 0.25).clamp(0.001, 0.05);
                        state.chains[family].temperature = temperature;
                        threshold = if o.algorithm == "anneal" {
                            temperature * u.ln()
                        } else {
                            0.0
                        };
                    }
                    let accept = delta >= threshold;
                    if accept {
                        if baseline_trial {
                            state.baseline_alloc = next.clone();
                        } else if !polishing {
                            state.chains[family].current = next.clone();
                            state.chains[family].accepted += 1;
                            state.chains[family].score = stats(&y).mean;
                        }
                    }
                    state.events.push(Event {
                        step: state.step,
                        family,
                        decision_seed,
                        samples: decision_count,
                        accepted: accept,
                        delta,
                        threshold,
                        kind: if polishing {
                            "polish"
                        } else if baseline_trial {
                            "baseline allocation"
                        } else {
                            "chain"
                        }
                        .into(),
                        cpu: spent(&e, carried, parent_start, worker_start),
                        current_key: reference_key.clone(),
                        proposal_key: next.key(),
                        operator: operator.clone(),
                        phase: phase.clone(),
                    });
                    if ((polishing && delta > 0.0) || stats(&y).mean > state.best_hint)
                        && spent(&e, carried, parent_start, worker_start) < o.seconds
                    {
                        let seed = if o.restricted {
                            6_000_000_000_000
                        } else {
                            PROMOTION_SEED + (state.promotions * 128) as i64
                        };
                        state.promotions += 1;
                        let a = eval(&mut e, &next, seed, 32);
                        if spent(&e, carried, parent_start, worker_start) < o.seconds
                            && let (Ok(a), Ok(b)) = (a, eval(&mut e, &state.best, seed, 32))
                        {
                            let (difference, se) = paired(&a, &b);
                            state.best_hint = stats(&b).mean;
                            if difference > 0.0 {
                                state.best = next.clone();
                                state.best_hint = stats(&a).mean;
                                retain(&mut state.archive, next.clone(), true);
                            } else if difference.abs() <= 2.0 * se {
                                retain(&mut state.archive, next.clone(), false);
                            }
                        }
                    }
                    if accept && state.archive.len() < 6 {
                        retain(&mut state.archive, next, false);
                    }
                }
            }
        } else {
            break;
        }
        state.chains[family].steps += 1;
        if polishing {
            state.polish.as_mut().unwrap().cursor += 1;
        }
        // Temperature is tied to completed decisions and persisted, never
        // restarted when the outer scheduler resumes a family.
        state.chains[family].temperature *= 0.997;
        state.step += 1;
        state.pending = None;
        if state.step.is_multiple_of(8) {
            state.parent_cpu = spent(&e, carried, parent_start, worker_start);
            atomic(&checkpoint, &state)?;
        }
        if state.step.is_multiple_of(25) {
            eprintln!(
                "{}: {} decisions, {:.2}s CPU, best search estimate {:.1}",
                o.out.display(),
                state.step,
                spent(&e, carried, parent_start, worker_start),
                state.best_hint
            );
        }
    }
    let discovery_cpu = spent(&e, carried, parent_start, worker_start);
    state.curve.push(serde_json::json!({"budget_cpu_seconds":o.seconds,"actual_cpu_seconds":discovery_cpu,"decisions":state.step,"best_search_estimate":state.best_hint,"best_key":state.best.key()}));
    state.parent_cpu = spent(&e, carried, parent_start, worker_start);
    atomic(&checkpoint, &state)?;
    // Freeze the entire shortlist before looking at any final-audit seeds.
    let mut shortlist = state.archive.clone();
    retain(&mut shortlist, state.baseline_alloc.clone(), false);
    retain(&mut shortlist, state.best.clone(), true);
    let frozen_list: Vec<_> = shortlist.iter().map(|c| c.key()).collect();
    atomic(&o.out.join("shortlist.json"), &shortlist)?;
    let audit_start = cpu();
    let audit_worker_start = e.worker_cpu;
    let audit_seed =
        AUDIT_SEED + (o.seed as i64) * 1_000_000 + (o.seconds * 1000.0).round() as i64 * 1000;
    let baseline_samples = eval(&mut e, &state.baseline, audit_seed, o.audit)?;
    let mut audits = Vec::new();
    for (i, c) in shortlist.iter().enumerate() {
        c.allocation.validate(&m)?;
        fs::write(
            o.out.join(format!("candidate-{i}.gcsl")),
            Evaluator::config(&c.allocation, &c.program),
        )
        .map_err(|e| e.to_string())?;
        let result = match eval(&mut e, c, audit_seed, o.audit) {
            Ok(s) => {
                let same_alloc_base = Candidate {
                    allocation: c.allocation.clone(),
                    ..state.baseline.clone()
                };
                let b = eval(&mut e, &same_alloc_base, audit_seed, o.audit)?;
                let fixed = Candidate {
                    allocation: m.clone(),
                    ..c.clone()
                };
                let f = eval(&mut e, &fixed, audit_seed, o.audit)?;
                serde_json::json!({"index":i,"key":c.key(),"role":c.role,"status":"independently-audited shortlist member",
                    "joint":stats(&s),"baseline_same_allocation":stats(&b),"program_fixed_initial_allocation":stats(&f),
                    "joint_minus_original_baseline":paired(&s,&baseline_samples),"cycle_only_same_allocation":paired(&s,&b),
                    "artifact_scores":c.allocation.score()})
            }
            Err(err) => {
                serde_json::json!({"index":i,"key":c.key(),"status":"failed/unresolved audit","error":err})
            }
        };
        audits.push(result);
    }
    let audit_cpu = cpu() - audit_start + e.worker_cpu - audit_worker_start;
    let mut latencies = e.batch_wall.clone();
    latencies.sort_by(f64::total_cmp);
    let report = serde_json::json!({"format":1,"status":"development pilot only; expansion deferred","engine":ENGINE,
        "implementation_sha256":implementation_hash(),"manifest_sha256":hash(&serde_json::to_string(&m).unwrap()),"site_commit":m.site_commit,
        "algorithm":o.algorithm,"seed":o.seed,"joint_er":o.joint,"local_only":o.local,"revised":o.revised,"restricted_domain":o.restricted,
        "objective":{"duration":450,"warmup":150},"audit_seed":audit_seed,"discovery_cpu_seconds":discovery_cpu,"discovery_cpu_cap":o.seconds,
        "cap_overrun_seconds":(discovery_cpu-o.seconds).max(0.0),"audit_cpu_seconds":audit_cpu,"this_invocation_wall_seconds":wall.elapsed().as_secs_f64(),
        "completed_decisions":state.step,"accepted":state.chains.iter().map(|s|s.accepted).sum::<usize>(),"families":state.chains.len(),"max_executing_chains":1,
        "provisional_best_key":state.best.key(),"baseline":stats(&baseline_samples),"shortlist_frozen_keys":frozen_list,
        "audits":audits,"curve":state.curve,"worker_calls":e.calls,"completed_or_failed_fights":e.fights,"failed_fights":e.failures,
        "timeouts_this_invocation":e.timeouts,"cache_hit_samples_this_invocation":e.cache_hits,
        "worker_batch_p50_seconds":latencies.get(latencies.len()/2),"worker_batch_p95_seconds":latencies.get(latencies.len()*95/100),
        "limitations":["single declared development party; not evidence of broad transfer","final audit seeds never used by proposal or promotion logic","audit shortlist ranking still has multiple-comparison selection uncertainty","no early low-DPS pruning","cross-profile refinement and serving integration are deferred"]});
    atomic(&o.out.join("report.json"), &report)?;
    println!(
        "{}",
        serde_json::json!({"report":o.out.join("report.json"),"decisions":state.step,"search_cpu":discovery_cpu,"audit_cpu":audit_cpu,"baseline":stats(&baseline_samples).mean,"best_search":state.best_hint})
    );
    Ok(())
}
fn retain(archive: &mut Vec<Candidate>, c: Candidate, peak: bool) {
    let key = c.key();
    if archive.iter().any(|x| x.key() == key) {
        return;
    }
    if archive.len() < 6 {
        archive.push(c);
    } else if peak {
        archive[1] = c;
    } else {
        // Slot zero remains the actual baseline; retain a bounded mix of
        // source families and an allocation alternative rather than only two.
        if let Some(i) = archive
            .iter()
            .enumerate()
            .skip(2)
            .find(|(_, x)| x.role == c.role)
            .map(|(i, _)| i)
        {
            archive[i] = c;
        }
    }
}

fn opt(args: &[String], name: &str, default: &str) -> String {
    args.windows(2)
        .find(|x| x[0] == name)
        .map(|x| x[1].clone())
        .unwrap_or_else(|| default.into())
}
fn number<T: std::str::FromStr>(args: &[String], name: &str, default: &str) -> R<T> {
    opt(args, name, default)
        .parse()
        .map_err(|_| format!("invalid {name}"))
}

pub fn cli() -> R<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str){
        Some("__worker")=>{if args.len()!=3{return Err("worker input/output required".into());}crate::evaluate::worker(&args[1],&args[2])},
        Some("search")=>{
            let algorithm=opt(&args,"--algorithm","anneal");if !["anneal","greedy","random"].contains(&algorithm.as_str()){return Err("algorithm: anneal, greedy or random".into());}
            run(&Options{manifest:opt(&args,"--manifest","data/flins-c1/manifest.json"),baseline:opt(&args,"--baseline","data/flins-c1/baseline.gcsl"),
                out:opt(&args,"--out","runs/pilot").into(),seconds:number(&args,"--seconds","30")?,seed:number(&args,"--seed","1")?,algorithm,
                local:args.iter().any(|a|a=="--local-only"),joint:!args.iter().any(|a|a=="--fixed-er"),audit:number(&args,"--audit","128")?,
                revised:args.iter().any(|a|a=="--revised"),restricted:args.iter().any(|a|a=="--restricted-domain")})
        },
        Some("evaluate")=>{
            let src=fs::read_to_string(opt(&args,"--config","")).map_err(|e|e.to_string())?;
            let mut e=Evaluator::new(Path::new(&opt(&args,"--out","runs/evaluate")))?;
            let start=cpu();let samples=e.raw(&src,number(&args,"--seed","7000000000000")?,number(&args,"--samples","128")?)?;
            println!("{}",serde_json::json!({"stats":stats(&samples),"cpu_seconds":e.worker_cpu+cpu()-start,"engine":ENGINE}));Ok(())
        },
        Some("enumerate")=>enumerate(&args),
        Some("validate")=>{
            let m=Manifest::load(&opt(&args,"--manifest","data/flins-c1/manifest.json"))?;
            let baseline=fs::read_to_string(opt(&args,"--baseline","data/flins-c1/baseline.gcsl")).map_err(|e|e.to_string())?;
            let mut e=Evaluator::new(Path::new(&opt(&args,"--out","runs/validate")))?;
            let start=cpu();let mut results=Vec::new();
            for (i,p) in Program::families(&baseline).iter().enumerate(){
                let s=e.eval(&m,p,800_000_000_000,16)?;
                results.push(serde_json::json!({"family":i,"stats":stats(&s)}));
            }
            println!("{}",serde_json::json!({"families":results,"artifact_scores":m.score(),"cpu_seconds":e.worker_cpu+cpu()-start}));Ok(())
        },
        _=>Err("usage: gso validate | search [--seconds CPU --seed N --algorithm anneal|greedy|random --local-only --fixed-er --audit N --out DIR --manifest FILE] | evaluate --config FILE | enumerate".into()),
    }
}

fn enumerate(args: &[String]) -> R<()> {
    let out = PathBuf::from(opt(args, "--out", "runs/enumerated"));
    let m = Manifest::load(&opt(args, "--manifest", "data/flins-c1/manifest.json"))?;
    let mut e = Evaluator::new(&out)?;
    let start = cpu();
    let mut rows = Vec::new();
    // Finite grammar frozen before its sample objective is measured: two
    // field windows x two normal counts x two support burst periods, fixed A.
    for frames in [360, 600] {
        for normals in [2, 4] {
            for period in [1, 2] {
                let mut p = Program::families("").remove(1);
                if let Program::Typed { cycle, .. } = &mut p {
                    if let crate::program::Step::Field {
                        frames: f,
                        normals: n,
                        ..
                    } = &mut cycle[3]
                    {
                        *f = frames;
                        *n = normals;
                    }
                    if let crate::program::Step::Block { period: p, .. } = &mut cycle[0] {
                        *p = period;
                    }
                }
                let s = e.eval(&m, &p, 6_000_000_000_000, 32)?;
                rows.push(serde_json::json!({"frames":frames,"normals":normals,"ineffa_period":period,"program":p,"stats":stats(&s)}));
            }
        }
    }
    let best = rows
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            a["stats"]["mean"]
                .as_f64()
                .unwrap()
                .total_cmp(&b["stats"]["mean"].as_f64().unwrap())
        })
        .unwrap()
        .0;
    let mut recovery = Vec::new();
    // A separate restricted search over the same fixed sample objective.
    // Bit flips are local mutations; every query is reported. Table replay
    // is an algorithmic recovery diagnostic, not a fresh CPU benchmark.
    for alg in ["anneal", "greedy", "random"] {
        for seed in [11, 37, 91, 123, 231] {
            let mut rng = Rng(seed);
            let mut at = rng.index(8);
            let mut found = at;
            let mut trace = Vec::new();
            for step in 0..24 {
                let next = if alg == "random" {
                    rng.index(8)
                } else {
                    at ^ (1 << rng.index(3))
                };
                let value = |i: usize| rows[i]["stats"]["mean"].as_f64().unwrap();
                if value(next) > value(found) {
                    found = next;
                }
                let threshold = if alg == "anneal" {
                    0.03 * (1.0 - step as f64 / 24.0) * rng.uniform().ln()
                } else {
                    0.0
                };
                if (value(next) - value(at)) / value(best) >= threshold {
                    at = next;
                }
                trace.push(found == best);
            }
            recovery.push(serde_json::json!({"algorithm":alg,"seed":seed,"queries":24,"recovered":found==best,"best_index":found,"recovery_curve":trace}));
        }
    }
    let report = serde_json::json!({"grammar":"2 field lengths x 2 normal counts x 2 Ineffa burst periods; fixed allocation; exactly 8 valid programs","objective":"32 fixed full fights at seed 6000000000000; finite sample objective only","rows":rows,"reference_best_index":best,"table_replay_recovery":recovery,"cpu_seconds":e.worker_cpu+cpu()-start,"unrestricted_optimality_gap":"unknown"});
    atomic(&out.join("reference.json"), &report)?;
    println!(
        "enumerated reference: {}",
        out.join("reference.json").display()
    );
    Ok(())
}

fn implementation_hash() -> String {
    hash(
        &[
            include_str!("artifact.rs"),
            include_str!("program.rs"),
            include_str!("evaluate.rs"),
            include_str!("search.rs"),
            include_str!("../Cargo.lock"),
            include_str!("../Cargo.toml"),
            include_str!("../docs/pilot-protocol.md"),
            include_str!("../docs/pilot-retry.md"),
        ]
        .join("\n"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_polish_roundtrip_preserves_candidate_keys_and_rng() {
        let m = Manifest::load("data/flins-c1/standard.json").unwrap();
        let baseline = fs::read_to_string("data/flins-c1/baseline.gcsl").unwrap();
        let options = Options {
            manifest: String::new(),
            baseline: String::new(),
            out: PathBuf::new(),
            seconds: 4.0,
            seed: 11,
            algorithm: "greedy".into(),
            local: false,
            joint: true,
            audit: 128,
            revised: true,
            restricted: false,
        };
        let mut state = initialize(&options, &m, &baseline, "test");
        let anchor = state.best.key();
        let (next, operator) = polish_neighbors(&state.best, &m, true).remove(17);
        state.polish = Some(Polish {
            anchor: anchor.clone(),
            cursor: 17,
        });
        state.pending = Some(Pending {
            next,
            family: 0,
            decision_seed: SEARCH_SEED,
            u: state.rng.uniform(),
            baseline_trial: false,
            repairs_done: false,
            reference: state.best.clone(),
            operator,
            polishing: true,
        });
        state.best_hint = 305052.16876745573;
        let before = serde_json::to_string(&state).unwrap();
        let mut restored: State = serde_json::from_str(&before).unwrap();
        assert_eq!(before, serde_json::to_string(&restored).unwrap());
        assert_eq!(restored.polish.as_ref().unwrap().cursor, 17);
        assert_eq!(restored.polish.as_ref().unwrap().anchor, anchor);
        assert_eq!(
            restored.pending.as_ref().unwrap().next.key(),
            state.pending.as_ref().unwrap().next.key()
        );
        assert_eq!(restored.pending.as_ref().unwrap().reference.key(), anchor);
        assert_eq!(restored.rng.next(), state.rng.next());
    }
}
