use crate::{
    artifact::{ENGINE, Manifest},
    discovery::{Catalog, Rotation},
    evaluate::{Evaluator, Sample, atomic, cpu, hash, paired, stats},
    heldout::Dictionary,
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
/// Held-out mode: consecutive non-improving chain decisions before a restart.
const RESTART_PATIENCE: usize = 40;
/// Held-out mode: macro edits composed into one restart perturbation.
const RESTART_EDITS: usize = 3;

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
    #[serde(default)]
    stall: usize,
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
    #[serde(default)]
    stage_selections: Vec<StageSelection>,
    /// Held-out v2 second pass (`docs/heldout-recovery-v2.md` section 4).
    #[serde(default)]
    pass2: Option<Pass2>,
}
#[derive(Clone, Serialize, Deserialize)]
struct Pass2 {
    from_cpu: f64,
    start_step: usize,
    s1_key: String,
    alternative_key: String,
    minimise_cursor: usize,
    minimise_done: bool,
    minimise_accepted: usize,
    derived: Vec<crate::heldout::Block>,
}
#[derive(Serialize, Deserialize)]
struct StageSelection {
    budget_cpu_cap: f64,
    cpu_seconds: f64,
    candidate: Candidate,
    search_estimate: f64,
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
    from_scratch: bool,
    basic_grammar: bool,
    /// Held-out recovery mode: per-case block dictionary file.
    dictionary: Option<String>,
    /// Held-out v2 operators, annealing and assumption guard.
    v2: bool,
    /// Held-out v2: start a second pass when resumed (declared CPU point).
    second_pass_from: Option<f64>,
}

fn proposal(
    c: &Candidate,
    frozen: &Manifest,
    rng: &mut Rng,
    joint: bool,
    force_alloc: bool,
    revised: bool,
    catalog: Option<&Catalog>,
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
        if let (Program::Generated(rotation), Some(catalog)) = (&next.program, catalog) {
            if rng.index(64) == 0 {
                let actor = rng.index(4);
                next.program = Program::Generated(Rotation::generate(catalog, rng, actor));
                return (next, "fresh restart".into());
            }
            let (program, operator) = rotation.mutate(catalog, rng);
            next.program = Program::Generated(program);
            return (next, operator);
        }
        if revised {
            let (program, label) = next.program.mutate_revised(rng, frozen);
            next.program = program;
            return (next, label);
        }
        next.program = next.program.mutate(rng, frozen);
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

/// Held-out macro proposal: a dictionary statement edit (replace, insert,
/// delete, move) or one of the existing generic action-list neighbors.
/// Allocation never changes (fixed ER).
fn heldout_proposal(
    c: &Candidate,
    m: &Manifest,
    rng: &mut Rng,
    dict: &Dictionary,
    generic: &mut Option<(String, Vec<(Program, String)>)>,
) -> (Candidate, String) {
    let mut next = c.clone();
    next.parent = Some(c.key());
    let Some(raw) = c.program.source_text() else {
        return (next, "heldout: no editable source".into());
    };
    let members: Vec<u16> = m
        .members
        .iter()
        .filter_map(|x| gse::gcsl::char_key(&x.key))
        .collect();
    let context = m.build();
    for _ in 0..8 {
        if rng.index(100) < 25 {
            let key = c.key();
            if generic.as_ref().is_none_or(|(k, _)| k != &key) {
                *generic = Some((key, c.program.local_neighbors(m)));
            }
            let list = &generic.as_ref().unwrap().1;
            if !list.is_empty() {
                let (p, label) = list[rng.index(list.len())].clone();
                next.program = p;
                return (next, format!("generic {label}"));
            }
            continue;
        }
        if let Some((source, label)) =
            crate::heldout::macro_edit(raw, dict, &members, &context, rng)
        {
            next.program = c.program.with_source(source);
            return (next, label);
        }
    }
    (next, "heldout: no valid edit drawn".into())
}

fn heldout_micro(c: &Candidate, m: &Manifest, v2: bool) -> Vec<(Candidate, String)> {
    let Some(raw) = c.program.source_text() else {
        return Vec::new();
    };
    if v2 {
        crate::heldout_v2::micro(raw, &m.build())
    } else {
        crate::heldout::numeric_neighbors(raw, &m.build())
    }
    .into_iter()
    .map(|(source, label)| {
        (
            Candidate {
                program: c.program.with_source(source),
                parent: Some(c.key()),
                ..c.clone()
            },
            label,
        )
    })
    .collect()
}

/// Held-out v2 proposal: one or (compound) several edits from the v2
/// operator set or the generic action-list neighbors, evaluated once.
/// A result that adds an assumption construct is discarded (no-op).
fn heldout_proposal_v2(
    c: &Candidate,
    m: &Manifest,
    rng: &mut Rng,
    proposer: &crate::heldout_v2::Proposer,
    generic: &mut Option<(String, Vec<(Program, String)>)>,
    moves: crate::heldout_v2::Moves,
) -> (Candidate, String) {
    let mut next = c.clone();
    next.parent = Some(c.key());
    let Some(raw) = c.program.source_text() else {
        return (next, "heldout: no editable source".into());
    };
    let edits = if rng.index(100) < moves.compound_percent {
        2 + rng.index(moves.compound_max - 1)
    } else {
        1
    };
    let mut program = c.program.clone();
    let mut labels = Vec::new();
    for e in 0..edits {
        for _ in 0..8 {
            if e == 0 && rng.index(100) < 20 {
                let key = c.key();
                if generic.as_ref().is_none_or(|(k, _)| k != &key) {
                    *generic = Some((key, c.program.local_neighbors(m)));
                }
                let list = &generic.as_ref().unwrap().1;
                if !list.is_empty() {
                    let (p, label) = list[rng.index(list.len())].clone();
                    program = p;
                    labels.push(format!("generic {label}"));
                    break;
                }
                continue;
            }
            let current = program.source_text().unwrap_or(raw).to_string();
            if let Some((source, label)) = proposer.edit(&current, rng) {
                program = c.program.with_source(source);
                labels.push(label);
                break;
            }
        }
    }
    let changed = program.source_text().unwrap_or(raw);
    if labels.is_empty()
        || crate::heldout_v2::assumptions(changed)
            .iter()
            .zip(crate::heldout_v2::assumptions(raw))
            .any(|(a, b)| *a > b)
        || !proposer.params_allowed(raw, changed)
    {
        return (next, "heldout v2: no valid edit drawn".into());
    }
    next.program = program;
    let prefix = if labels.len() > 1 {
        format!("compound x{}: ", labels.len())
    } else {
        String::new()
    };
    (next, format!("{prefix}{}", labels.join(" | ")))
}

fn polish_neighbors(c: &Candidate, frozen: &Manifest, joint: bool) -> Vec<(Candidate, String)> {
    let mut programs = c.program.local_neighbors(frozen).into_iter();
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

fn initialize(
    o: &Options,
    m: &Manifest,
    baseline: &str,
    identity: &str,
    catalog: Option<&Catalog>,
) -> State {
    let mut initial_rng = Rng(o.seed);
    let generated: Vec<_> = if o.from_scratch {
        let catalog = catalog.expect("validated generated-action catalog");
        (0..4)
            .map(|actor| Program::Generated(Rotation::generate(catalog, &mut initial_rng, actor)))
            .collect()
    } else {
        Vec::new()
    };
    let base = Candidate {
        program: if o.from_scratch {
            generated[0].clone()
        } else if o.restricted {
            Program::restricted_domain()[initial_rng.index(8)].clone()
        } else {
            Program::baseline(baseline, m).expect("validated baseline")
        },
        allocation: m.clone(),
        role: if o.from_scratch {
            "generated starting control"
        } else {
            "baseline / C0-on-field pattern"
        }
        .into(),
        parent: None,
    };
    let programs = if o.from_scratch {
        generated
    } else if o.restricted {
        vec![base.program.clone()]
    } else {
        Program::families(baseline, m)
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
                role: format!(
                    "{}-{i}",
                    if o.from_scratch {
                        "generated-family"
                    } else {
                        "structure"
                    }
                ),
                parent: None,
            },
            steps: 0,
            accepted: 0,
            temperature: 0.015,
            score: 0.0,
            stall: 0,
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
        stage_selections: Vec::new(),
        pass2: None,
    }
}

fn run(o: &Options) -> R<()> {
    if !o.seconds.is_finite() || o.seconds <= 0.0 || o.audit == 0 {
        return Err("positive CPU seconds and audit count required".into());
    }
    if o.restricted && o.joint {
        return Err("restricted-domain recovery requires --fixed-er".into());
    }
    if o.from_scratch && (o.local || o.restricted) {
        return Err(
            "--from-scratch is incompatible with --local-only and --restricted-domain".into(),
        );
    }
    let parent_start = cpu();
    let wall = Instant::now();
    let m = Manifest::load(&o.manifest)?;
    if o.restricted && !Program::is_pilot(&m) {
        return Err("restricted-domain recovery is a Flins-only diagnostic".into());
    }
    if o.joint && m.fixed_build.is_some() {
        return Err("fixed-build manifests require --fixed-er".into());
    }
    // The held-out protocol uses each case's build as is; the discovery-profile
    // constellation grid does not apply to it.
    for member in m.members.iter().filter(|_| o.dictionary.is_none()) {
        let eligible = match crate::artifact::rarity(&member.key)? {
            4 => member.cons == 6,
            5 => [1, 2, 4, 6].contains(&member.cons),
            _ => false,
        };
        if !eligible {
            return Err(format!(
                "{}: ineligible new-discovery constellation",
                member.key
            ));
        }
    }
    let baseline = if o.from_scratch {
        String::new()
    } else {
        fs::read_to_string(&o.baseline).map_err(|e| e.to_string())?
    };
    let catalog = if o.from_scratch {
        Some(Catalog::configured(&m, !o.basic_grammar)?)
    } else {
        None
    };
    let dictionary_text = match &o.dictionary {
        Some(path) => fs::read_to_string(path).map_err(|e| e.to_string())?,
        None => String::new(),
    };
    let dictionary: Option<Dictionary> = if o.dictionary.is_some() {
        if o.joint
            || o.from_scratch
            || o.restricted
            || !o.local
            || (o.algorithm != "greedy" && !(o.v2 && o.algorithm == "anneal"))
        {
            return Err("--dictionary requires greedy (or --heldout-v2 with anneal), --fixed-er and one baseline chain".into());
        }
        Some(serde_json::from_str(&dictionary_text).map_err(|e| e.to_string())?)
    } else {
        None
    };
    if (o.v2 || o.second_pass_from.is_some()) && dictionary.is_none() {
        return Err("--heldout-v2 and --second-pass-from require --dictionary".into());
    }
    if o.second_pass_from.is_some() && !o.v2 {
        return Err("--second-pass-from requires --heldout-v2".into());
    }
    let initial = if let Some(catalog) = &catalog {
        Program::Generated(Rotation::generate(catalog, &mut Rng(o.seed), 0))
    } else {
        Program::baseline(&baseline, &m)?
    };
    let parsed = gse::gcsl::parse(&Evaluator::config(&m, &initial))?;
    let unsupported = gse::check::unsupported(&parsed);
    if !unsupported.is_empty() {
        return Err(unsupported.join("; "));
    }
    let identity = hash(
        &(format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}{}",
            serde_json::to_string(&m).unwrap(),
            baseline,
            o.seed,
            o.algorithm,
            o.local,
            o.joint,
            o.revised,
            o.restricted,
            o.from_scratch,
            o.basic_grammar,
            implementation_hash(),
            if o.dictionary.is_some() {
                format!("\nheldout dictionary\n{dictionary_text}")
            } else {
                String::new()
            }
        ) + if o.v2 { "\nheldout v2" } else { "" }),
    );
    fs::create_dir_all(&o.out).map_err(|e| e.to_string())?;
    let checkpoint = o.out.join("checkpoint.json");
    let mut state: State = if checkpoint.exists() {
        serde_json::from_slice(&fs::read(&checkpoint).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
    } else {
        initialize(o, &m, &baseline, &identity, catalog.as_ref())
    };
    if state.identity != identity || state.format != 2 {
        return Err("checkpoint experiment identity mismatch".into());
    }
    if !o.out.join("experiment.json").exists() {
        atomic(
            &o.out.join("experiment.json"),
            &serde_json::json!({
                "identity":identity,"from_scratch":o.from_scratch,"basic_grammar":o.basic_grammar,"objective":{"duration":m.objective().0,"warmup":m.objective().1},
                "reference_origin":if o.from_scratch { "generated starting control" } else { "supplied baseline" },
                "baseline_read":!o.from_scratch,"initial_chains":state.chains,
                "heldout_mode":o.dictionary.is_some(),
                "inputs_opened":[
                    {"role":"manifest","path":o.manifest,"sha256":hash(&fs::read_to_string(&o.manifest).map_err(|e| e.to_string())?)},
                    {"role":"baseline","path":if o.from_scratch { String::new() } else { o.baseline.clone() },"sha256":hash(&baseline)},
                    {"role":"dictionary","path":o.dictionary,"sha256":hash(&dictionary_text)},
                ],
            }),
        )?;
    }
    let members: Vec<u16> = m
        .members
        .iter()
        .filter_map(|x| gse::gcsl::char_key(&x.key))
        .collect();
    if let (Some(from), Some(dict)) = (o.second_pass_from, &dictionary)
        && state.pass2.is_none()
    {
        if state.parent_cpu + 1.0 < from {
            return Err(format!(
                "second pass from {from} CPU-s requires a first pass of that length (have {:.1})",
                state.parent_cpu
            ));
        }
        // Restart from the pass-1 result. The peak is kept; families,
        // temperatures, pending proposal and polish cursor are reset.
        let s1 = state.best.clone();
        let s1_src = s1.program.source_text().unwrap_or_default().to_string();
        let l_src = state
            .baseline
            .program
            .source_text()
            .unwrap_or_default()
            .to_string();
        let alternative = state
            .archive
            .iter()
            .filter(|c| c.key() != s1.key())
            .min_by(|a, b| {
                let sim = |c: &Candidate| {
                    crate::heldout_v2::similarity(
                        &s1_src,
                        c.program.source_text().unwrap_or_default(),
                    )
                };
                sim(a).total_cmp(&sim(b))
            })
            .cloned()
            .unwrap_or_else(|| state.baseline.clone());
        let derived = crate::heldout_v2::derive(dict, &[&s1_src, &l_src]);
        let chain = |c: &Candidate, role: &str| Chain {
            current: Candidate {
                role: role.into(),
                parent: Some(c.key()),
                ..c.clone()
            },
            steps: 0,
            accepted: 0,
            temperature: 0.015,
            score: 0.0,
            stall: 0,
        };
        let discarded = state.pending.take().map(|p| p.operator);
        state.chains = vec![
            chain(&s1, "pass2-minimised-S1"),
            chain(&alternative, "pass2-alternative"),
        ];
        state.polish = None;
        state.pass2 = Some(Pass2 {
            from_cpu: state.parent_cpu,
            start_step: state.step,
            s1_key: s1.key(),
            alternative_key: alternative.key(),
            minimise_cursor: 0,
            minimise_done: false,
            minimise_accepted: 0,
            derived: derived.clone(),
        });
        atomic(
            &o.out.join("second-pass.json"),
            &serde_json::json!({"declared_from_cpu":from,"from_cpu":state.parent_cpu,"start_step":state.step,
                "s1_key":s1.key(),"alternative_key":alternative.key(),"alternative_is_L":alternative.key()==state.baseline.key(),
                "derived_templates":derived.len(),"discarded_pending_operator":discarded,"protocol":"docs/heldout-recovery-v2.md section 4"}),
        )?;
        atomic(&checkpoint, &state)?;
    }
    let derived: Vec<crate::heldout::Block> = state
        .pass2
        .as_ref()
        .map(|p| p.derived.clone())
        .unwrap_or_default();
    let proposer = dictionary.as_ref().map(|dict| crate::heldout_v2::Proposer {
        dict,
        derived: &derived,
        members: members.clone(),
        context: m.build(),
    });
    let moves = if state.pass2.is_some() {
        crate::heldout_v2::PASS2
    } else {
        crate::heldout_v2::PASS1
    };
    let mut e = Evaluator::new(&o.out)?;
    e.scaled_limit = o.v2;
    let carried = state.parent_cpu;
    let worker_start = e.worker_cpu;
    if state.reference_dps == 0.0 {
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
        state.reference_dps = state.best_hint.max(1.0);
    }
    let initial_dps = state.reference_dps.max(1.0);
    let decision_count = if o.restricted { 32 } else { 16 };
    let mut polish_cache: Option<(String, Vec<(Candidate, String)>)> = None;
    let mut generic_cache: Option<(String, Vec<(Program, String)>)> = None;
    while spent(&e, carried, parent_start, worker_start) < o.seconds {
        let mut polishing = state.pending.as_ref().map(|p| p.polishing).unwrap_or(
            o.revised
                && !o.restricted
                && spent(&e, carried, parent_start, worker_start) >= o.seconds * 0.75,
        );
        let minimising = state.pass2.as_ref().is_some_and(|p| !p.minimise_done)
            && state
                .pending
                .as_ref()
                .is_none_or(|p| p.operator.starts_with("minimise"));
        if minimising {
            polishing = false;
        }
        if dictionary.is_some() && polishing && state.pending.is_none() {
            // Micro coordinate search around the peak; once its neighborhood
            // is exhausted without a promotion, return to macro edits.
            let anchor = state.best.key();
            if polish_cache.as_ref().is_none_or(|(key, _)| key != &anchor) {
                polish_cache = Some((anchor.clone(), heldout_micro(&state.best, &m, o.v2)));
            }
            let cursor = state
                .polish
                .as_ref()
                .filter(|p| p.anchor == anchor)
                .map_or(0, |p| p.cursor);
            if cursor >= polish_cache.as_ref().unwrap().1.len() {
                polishing = false;
            }
        }
        if o.from_scratch
            && polishing
            && spent(&e, carried, parent_start, worker_start) >= o.seconds * 0.75
            && state
                .stage_selections
                .last()
                .is_none_or(|s| s.budget_cpu_cap != o.seconds)
        {
            state.stage_selections.push(StageSelection {
                budget_cpu_cap: o.seconds,
                cpu_seconds: spent(&e, carried, parent_start, worker_start),
                candidate: state.best.clone(),
                search_estimate: state.best_hint,
            });
        }
        let family = if let Some(p) = &state.pending {
            p.family
        } else if minimising {
            0
        } else if o.v2 && state.chains.iter().any(|c| c.steps < 4) {
            // Initial quantum: every family gets four decisions first.
            (0..state.chains.len())
                .min_by_key(|&i| state.chains[i].steps)
                .unwrap()
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
        let baseline_trial = !o.from_scratch && !polishing && o.joint && state.step % 5 == 4;
        let origin = if polishing {
            state.best.clone()
        } else if baseline_trial {
            state.baseline_alloc.clone()
        } else if o.from_scratch
            && o.algorithm != "random"
            && state.step >= state.chains.len() * 4
            && state.step.is_multiple_of(13)
        {
            // Reuse only a peak discovered in this run. It remains a proposal
            // against the current family, requiring the same paired block.
            state.best.clone()
        } else if o.algorithm == "random" {
            if o.from_scratch {
                current.clone()
            } else {
                initialize(o, &m, &baseline, &identity, None).chains[family]
                    .current
                    .clone()
            }
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
                    polish_cache = Some((
                        origin.key(),
                        if dictionary.is_some() {
                            heldout_micro(&origin, &m, o.v2)
                        } else {
                            polish_neighbors(&origin, &m, o.joint)
                        },
                    ));
                }
                let neighbors = &polish_cache.as_ref().unwrap().1;
                let cursor = state.polish.as_ref().unwrap().cursor;
                let Some(next) = neighbors.get(cursor).cloned() else {
                    break;
                };
                next
            } else if o.from_scratch && o.algorithm == "random" && state.chains[family].steps > 0 {
                let actor = state.rng.index(4);
                (
                    Candidate {
                        program: Program::Generated(Rotation::generate(
                            catalog.as_ref().unwrap(),
                            &mut state.rng,
                            actor,
                        )),
                        parent: Some(origin.key()),
                        ..origin.clone()
                    },
                    "fresh restart".into(),
                )
            } else if state.chains[family].steps == 0 && !baseline_trial {
                (origin.clone(), "initial family evaluation".into())
            } else if minimising && family == 0 {
                // G5 sweep: delete one unit of the current program; the
                // cursor advances only when the deletion is rejected.
                let p2 = state.pass2.as_mut().unwrap();
                let src = origin.program.source_text().unwrap_or_default().to_string();
                match crate::heldout_v2::delete_unit(&src, p2.minimise_cursor, &m.build()) {
                    None => {
                        p2.minimise_done = true;
                        continue;
                    }
                    Some(None) => {
                        p2.minimise_cursor += 1;
                        continue;
                    }
                    Some(Some(source)) => (
                        Candidate {
                            program: origin.program.with_source(source),
                            parent: Some(origin.key()),
                            ..origin.clone()
                        },
                        format!("minimise delete unit {}", p2.minimise_cursor),
                    ),
                }
            } else if let (true, Some(proposer)) = (o.v2, &proposer) {
                if state.chains[family].stall >= RESTART_PATIENCE {
                    state.chains[family].stall = 0;
                    let mut next = state.best.clone();
                    let mut labels = Vec::new();
                    for _ in 0..RESTART_EDITS {
                        let (n, label) = heldout_proposal_v2(
                            &next,
                            &m,
                            &mut state.rng,
                            proposer,
                            &mut generic_cache,
                            crate::heldout_v2::Moves {
                                compound_percent: 0,
                                compound_max: 1,
                            },
                        );
                        next = n;
                        labels.push(label);
                    }
                    next.parent = Some(state.best.key());
                    (
                        next,
                        format!("restart perturbation: {}", labels.join(" | ")),
                    )
                } else {
                    heldout_proposal_v2(
                        &origin,
                        &m,
                        &mut state.rng,
                        proposer,
                        &mut generic_cache,
                        moves,
                    )
                }
            } else if let Some(dict) = &dictionary {
                if state.chains[family].stall >= RESTART_PATIENCE {
                    // Restart: perturb the promoted peak by several random
                    // macro edits; installed unconditionally like a restart.
                    state.chains[family].stall = 0;
                    let mut next = state.best.clone();
                    let mut labels = Vec::new();
                    for _ in 0..RESTART_EDITS {
                        let (n, label) =
                            heldout_proposal(&next, &m, &mut state.rng, dict, &mut generic_cache);
                        next = n;
                        labels.push(label);
                    }
                    next.parent = Some(state.best.key());
                    (
                        next,
                        format!("restart perturbation: {}", labels.join(" | ")),
                    )
                } else {
                    heldout_proposal(&origin, &m, &mut state.rng, dict, &mut generic_cache)
                }
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
                    catalog.as_ref(),
                )
            };
            if !polishing
                && !baseline_trial
                && origin.key() != current.key()
                && operator != "fresh restart"
                && o.from_scratch
                && o.algorithm != "random"
            {
                operator = format!("archive-seeded {operator}");
            }
            if !o.from_scratch
                && o.algorithm == "random"
                && !baseline_trial
                && state.chains[family].steps > 0
            {
                for _ in 0..state.rng.index(6) {
                    (next, operator) =
                        proposal(&next, &m, &mut state.rng, o.joint, false, o.revised, None);
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
        let restarting =
            operator == "fresh restart" || operator.starts_with("restart perturbation");
        let minimise_step = operator.starts_with("minimise");
        let mut threshold = if restarting {
            -f64::MAX
        } else if o.algorithm == "anneal" && !baseline_trial && !polishing && !minimise_step {
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
            if minimise_step {
                state.pass2.as_mut().unwrap().minimise_cursor += 1;
            }
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
                Err(err) => state.events.push({
                    if minimise_step {
                        state.pass2.as_mut().unwrap().minimise_cursor += 1;
                    }
                    Event {
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
                    }
                }),
                Ok(x) => {
                    let delta = (stats(&y).mean - stats(&x).mean) / initial_dps;
                    if state.chains[family].steps == 1
                        && !baseline_trial
                        && !polishing
                        && !restarting
                    {
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
                        threshold = if o.algorithm == "anneal" && !minimise_step {
                            temperature * u.ln()
                        } else {
                            0.0
                        };
                    }
                    let accept = delta >= threshold;
                    if minimise_step {
                        let p2 = state.pass2.as_mut().unwrap();
                        if accept {
                            p2.minimise_accepted += 1;
                        } else {
                            p2.minimise_cursor += 1;
                        }
                    }
                    if !polishing && !baseline_trial && !restarting && !minimise_step {
                        if accept && delta > 0.0 {
                            state.chains[family].stall = 0;
                        } else {
                            state.chains[family].stall += 1;
                        }
                    }
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
                        kind: if restarting {
                            "restart"
                        } else if minimise_step {
                            "minimise"
                        } else if polishing {
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
                    if accept && (o.from_scratch || state.archive.len() < 6) {
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
    let shortlist = freeze_shortlist(&state, o.from_scratch, o.seconds);
    let frozen_list: Vec<_> = shortlist.iter().map(|c| c.key()).collect();
    atomic(&o.out.join("shortlist.json"), &shortlist)?;
    e.scaled_limit = false;
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
                serde_json::json!({"index":i,"key":c.key(),"role":c.role,"structure":match &c.program { Program::Generated(p)=>Some(p.structure()), _=>None },"status":"independently-audited shortlist member",
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
    let mut report = serde_json::json!({"format":1,"status":if o.from_scratch { "baseline-free discovery; broad effectiveness unvalidated" } else { "local baseline refinement; broad effectiveness unvalidated" },"engine":ENGINE,
        "implementation_sha256":implementation_hash(),"manifest_sha256":hash(&serde_json::to_string(&m).unwrap()),"site_commit":m.site_commit,
        "algorithm":o.algorithm,"seed":o.seed,"joint_er":o.joint,"local_only":o.local,"revised":o.revised,"restricted_domain":o.restricted,
        "objective":{"duration":m.objective().0,"warmup":m.objective().1},"audit_seed":audit_seed,"discovery_cpu_seconds":discovery_cpu,"discovery_cpu_cap":o.seconds,
        "cap_overrun_seconds":(discovery_cpu-o.seconds).max(0.0),"audit_cpu_seconds":audit_cpu,"this_invocation_wall_seconds":wall.elapsed().as_secs_f64(),
        "completed_decisions":state.step,"accepted":state.chains.iter().map(|s|s.accepted).sum::<usize>(),"families":state.chains.len(),"max_executing_chains":1,
        "provisional_best_key":state.best.key(),"baseline":stats(&baseline_samples),"shortlist_frozen_keys":frozen_list,
        "audits":audits,"curve":state.curve,"worker_calls":e.calls,"completed_or_failed_fights":e.fights,"failed_fights":e.failures,
        "timeouts_this_invocation":e.timeouts,"cache_hit_samples_this_invocation":e.cache_hits,
        "worker_batch_p50_seconds":latencies.get(latencies.len()/2),"worker_batch_p95_seconds":latencies.get(latencies.len()*95/100),
        "limitations":[if o.from_scratch { "bounded parameter forms and resource-guarded charge/dash continuations; arbitrary mechanic predicates are not enumerated" } else { "non-Flins parties use baseline-derived source mutations only; broad effectiveness unvalidated" },"broad effectiveness and unrestricted optimality remain unvalidated","final audit seeds never used by proposal or promotion logic","audit shortlist ranking still has multiple-comparison selection uncertainty","no early low-DPS pruning","cross-profile refinement and serving integration are deferred"]});
    report["basic_grammar"] = serde_json::json!(o.basic_grammar);
    report["from_scratch"] = serde_json::json!(o.from_scratch);
    report["reference_origin"] = serde_json::json!(if o.from_scratch {
        "generated starting control"
    } else {
        "supplied baseline"
    });
    report["heldout_v2"] = serde_json::json!(o.v2);
    report["second_pass"] = serde_json::to_value(state.pass2.as_ref().map(|p| {
        serde_json::json!({"from_cpu":p.from_cpu,"start_step":p.start_step,"s1_key":p.s1_key,
            "alternative_key":p.alternative_key,"minimise_done":p.minimise_done,
            "minimise_accepted":p.minimise_accepted,"derived_templates":p.derived.len()})
    }))
    .map_err(|e| e.to_string())?;
    report["stage_selections"] =
        serde_json::to_value(&state.stage_selections).map_err(|e| e.to_string())?;
    atomic(&o.out.join("report.json"), &report)?;
    println!(
        "{}",
        serde_json::json!({"report":o.out.join("report.json"),"decisions":state.step,"search_cpu":discovery_cpu,"audit_cpu":audit_cpu,"baseline":stats(&baseline_samples).mean,"best_search":state.best_hint})
    );
    Ok(())
}
fn freeze_shortlist(state: &State, from_scratch: bool, cap: f64) -> Vec<Candidate> {
    if !from_scratch {
        let mut shortlist = state.archive.clone();
        retain(&mut shortlist, state.baseline_alloc.clone(), false);
        retain(&mut shortlist, state.best.clone(), true);
        return shortlist;
    }
    // Reserve protected candidates before adding diversity. The peak can occupy
    // any archive index; replacing a fixed slot could accidentally evict it.
    let stage = state
        .stage_selections
        .last()
        .filter(|s| s.budget_cpu_cap == cap);
    let mut shortlist = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for candidate in [&state.baseline, &state.best]
        .into_iter()
        .chain(stage.map(|s| &s.candidate))
        .chain(&state.archive)
    {
        if seen.insert(candidate.key()) {
            shortlist.push(candidate.clone());
        }
        if shortlist.len() == 6 {
            break;
        }
    }
    shortlist
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
fn baseline_path(args: &[String]) -> String {
    let manifest = opt(args, "--manifest", "data/flins-c1/manifest.json");
    let adjacent = Path::new(&manifest).with_file_name("baseline.gcsl");
    opt(args, "--baseline", &adjacent.to_string_lossy())
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
            let from_scratch=args.iter().any(|a|a=="--from-scratch");
            if from_scratch && args.iter().any(|a|a=="--baseline") { return Err("--from-scratch does not accept --baseline; evaluate reference cycles separately after discovery".into()); }
            let algorithm=opt(&args,"--algorithm",if from_scratch { "greedy" } else { "anneal" });if !["anneal","greedy","random"].contains(&algorithm.as_str()){return Err("algorithm: anneal, greedy or random".into());}
            let dictionary=args.iter().any(|a|a=="--dictionary").then(|| opt(&args,"--dictionary",""));
            run(&Options{manifest:opt(&args,"--manifest","data/flins-c1/manifest.json"),baseline:baseline_path(&args),
                out:opt(&args,"--out","runs/pilot").into(),seconds:number(&args,"--seconds","30")?,seed:number(&args,"--seed","1")?,algorithm,
                local:dictionary.is_some() || args.iter().any(|a|a=="--local-only"),joint:!args.iter().any(|a|a=="--fixed-er"),audit:number(&args,"--audit","128")?,
                revised:from_scratch || dictionary.is_some() || args.iter().any(|a|a=="--revised"),restricted:args.iter().any(|a|a=="--restricted-domain"),from_scratch,basic_grammar:args.iter().any(|a|a=="--basic-grammar"),dictionary,
                v2:args.iter().any(|a|a=="--heldout-v2"),second_pass_from:if args.iter().any(|a|a=="--second-pass-from"){Some(number(&args,"--second-pass-from","192")?)}else{None}})
        },
        Some("evaluate")=>{
            let src=fs::read_to_string(opt(&args,"--config","")).map_err(|e|e.to_string())?;
            let mut e=Evaluator::new(Path::new(&opt(&args,"--out","runs/evaluate")))?;
            let start=cpu();let samples=e.raw(&src,number(&args,"--seed","7000000000000")?,number(&args,"--samples","128")?)?;
            println!("{}",serde_json::json!({"stats":stats(&samples),"cpu_seconds":e.worker_cpu+cpu()-start,"engine":ENGINE}));Ok(())
        },
        Some("render")=>{
            let m=Manifest::load(&opt(&args,"--manifest",""))?;
            let p:Program=serde_json::from_slice(&fs::read(opt(&args,"--program","")).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
            p.validate()?;
            let source=Evaluator::config(&m,&p);
            let parsed=gse::gcsl::parse(&source)?;
            let missing=gse::check::unsupported(&parsed);
            if !missing.is_empty(){return Err(missing.join("; "));}
            let output=opt(&args,"--out","");
            if output.is_empty(){print!("{source}");}else{
                let path=Path::new(&output);
                if let Some(parent)=path.parent().filter(|p|!p.as_os_str().is_empty()){fs::create_dir_all(parent).map_err(|e|e.to_string())?;}
                fs::write(path,source).map_err(|e|e.to_string())?;
            }
            Ok(())
        },
        Some("enumerate")=>enumerate(&args),
        Some("heldout-manifest")|Some("dictionary")|Some("measure")=>crate::heldout::cli(&args),
        Some("dictionary-v2")|Some("strip-assumptions")|Some("config")|Some("param-check")=>crate::heldout_v2::cli(&args),
        Some("validate")=>{
            let m=Manifest::load(&opt(&args,"--manifest","data/flins-c1/manifest.json"))?;
            let programs=if args.iter().any(|a|a=="--from-scratch") {
                if args.iter().any(|a|a=="--baseline") { return Err("--from-scratch does not accept --baseline".into()); }
                let catalog=Catalog::configured(&m,!args.iter().any(|a|a=="--basic-grammar"))?;
                let mut rng=Rng(number(&args,"--seed","1")?);
                (0..4).map(|actor| Program::Generated(Rotation::generate(&catalog,&mut rng,actor))).collect()
            } else {
                let baseline=fs::read_to_string(baseline_path(&args)).map_err(|e|e.to_string())?;
                Program::baseline(&baseline, &m)?;
                Program::families(&baseline, &m)
            };
            let mut e=Evaluator::new(Path::new(&opt(&args,"--out","runs/validate")))?;
            let start=cpu();let mut results=Vec::new();
            for (i,p) in programs.iter().enumerate(){
                let s=e.eval(&m,p,800_000_000_000,16)?;
                results.push(serde_json::json!({"family":i,"stats":stats(&s)}));
            }
            println!("{}",serde_json::json!({"families":results,"artifact_scores":m.score(),"cpu_seconds":e.worker_cpu+cpu()-start}));Ok(())
        },
        _=>Err("usage: gso validate [--from-scratch --manifest FILE] | search [--from-scratch --seconds CPU --seed N --algorithm anneal|greedy|random --local-only --fixed-er --audit N --out DIR --manifest FILE] | evaluate --config FILE | enumerate".into()),
    }
}

fn enumerate(args: &[String]) -> R<()> {
    let out = PathBuf::from(opt(args, "--out", "runs/enumerated"));
    let m = Manifest::load(&opt(args, "--manifest", "data/flins-c1/manifest.json"))?;
    if !Program::is_pilot(&m) {
        return Err("enumeration is a Flins-only diagnostic".into());
    }
    let mut e = Evaluator::new(&out)?;
    let start = cpu();
    let mut rows = Vec::new();
    // Finite grammar frozen before its sample objective is measured: two
    // field windows x two normal counts x two support burst periods, fixed A.
    for frames in [360, 600] {
        for normals in [2, 4] {
            for period in [1, 2] {
                let mut p = Program::families("", &m).remove(1);
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
            include_str!("discovery.rs"),
            include_str!("evaluate.rs"),
            include_str!("search.rs"),
            include_str!("heldout.rs"),
            include_str!("heldout_v2.rs"),
            include_str!("../data/character-rarities.json"),
            include_str!("../Cargo.lock"),
            include_str!("../Cargo.toml"),
            include_str!("../docs/pilot-protocol.md"),
            include_str!("../docs/pilot-retry.md"),
            include_str!("../docs/from-scratch.md"),
            include_str!("../docs/heldout-recovery-pilot.md"),
            include_str!("../docs/heldout-recovery-v2.md"),
        ]
        .join("\n"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scratch_initialization_ignores_baseline_and_restores_generated_proposals() {
        let m = crate::artifact::tests::fixed_party();
        let catalog = Catalog::new(&m).unwrap();
        let options = Options {
            manifest: String::new(),
            baseline: "/missing/baseline.gcsl".into(),
            out: PathBuf::new(),
            seconds: 4.0,
            seed: 37,
            algorithm: "greedy".into(),
            local: false,
            joint: false,
            audit: 16,
            revised: true,
            restricted: false,
            from_scratch: true,
            basic_grammar: false,
            dictionary: None,
            v2: false,
            second_pass_from: None,
        };
        let mut state = initialize(
            &options,
            &m,
            "invalid source that must not be parsed",
            "test",
            Some(&catalog),
        );
        let other = initialize(
            &options,
            &m,
            "an unrelated supplied cycle",
            "test",
            Some(&catalog),
        );
        assert_eq!(
            serde_json::to_string(&state).unwrap(),
            serde_json::to_string(&other).unwrap()
        );
        assert_eq!(state.chains.len(), 4);
        for (i, chain) in state.chains.iter().enumerate() {
            let Program::Generated(p) = &chain.current.program else {
                panic!("baseline leaked");
            };
            assert!(p.cycle.iter().any(|v| v.actor == i && v.field.is_some()));
        }
        let (next, operator) = proposal(
            &state.best,
            &m,
            &mut state.rng,
            false,
            false,
            true,
            Some(&catalog),
        );
        state.pending = Some(Pending {
            next,
            family: 0,
            decision_seed: SEARCH_SEED,
            u: state.rng.uniform(),
            baseline_trial: false,
            repairs_done: false,
            reference: state.best.clone(),
            operator,
            polishing: false,
        });
        let mut restored: State =
            serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
        assert_eq!(
            restored.pending.as_ref().unwrap().next.key(),
            state.pending.as_ref().unwrap().next.key()
        );
        for _ in 0..50 {
            let a = proposal(
                &state.best,
                &m,
                &mut state.rng,
                false,
                false,
                true,
                Some(&catalog),
            );
            let b = proposal(
                &restored.best,
                &m,
                &mut restored.rng,
                false,
                false,
                true,
                Some(&catalog),
            );
            assert_eq!(a.0.key(), b.0.key());
            assert_eq!(a.1, b.1);
        }
        let mut rng = Rng(1);
        assert!((0..100).any(|_| {
            proposal(
                &state.best,
                &m,
                &mut rng,
                false,
                false,
                true,
                Some(&catalog),
            )
            .1 == "fresh restart"
        }));
        assert!(!polish_neighbors(&state.best, &m, false).is_empty());

        // A full archive can hold the peak at an arbitrary position. Protect
        // it and a distinct pre-polish candidate before adding diversity.
        let neighbors = polish_neighbors(&state.best, &m, false);
        state
            .archive
            .extend(neighbors.iter().take(5).map(|(c, _)| c.clone()));
        state.best = state.archive[2].clone();
        state.stage_selections.push(StageSelection {
            budget_cpu_cap: 4.0,
            cpu_seconds: 3.0,
            candidate: neighbors[6].0.clone(),
            search_estimate: 1000.0,
        });
        let shortlist = freeze_shortlist(&state, true, 4.0);
        assert_eq!(shortlist.len(), 6);
        assert_eq!(shortlist[0].key(), state.baseline.key());
        assert_eq!(shortlist[1].key(), state.best.key());
        assert_eq!(
            shortlist[2].key(),
            state.stage_selections[0].candidate.key()
        );
        assert_eq!(
            shortlist
                .iter()
                .map(Candidate::key)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            6
        );

        let anchor = state.best.key();
        let (next, operator) = polish_neighbors(&state.best, &m, false).remove(0);
        state.polish = Some(Polish {
            anchor: anchor.clone(),
            cursor: 0,
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
        let serialized = serde_json::to_string(&state).unwrap();
        let restored: State = serde_json::from_str(&serialized).unwrap();
        assert_eq!(serialized, serde_json::to_string(&restored).unwrap());
        assert_eq!(restored.polish.as_ref().unwrap().anchor, anchor);
        assert!(restored.pending.as_ref().unwrap().polishing);
        assert_eq!(
            restored.pending.as_ref().unwrap().next.key(),
            state.pending.as_ref().unwrap().next.key()
        );
    }

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
            from_scratch: false,
            basic_grammar: false,
            dictionary: None,
            v2: false,
            second_pass_from: None,
        };
        let mut state = initialize(&options, &m, &baseline, "test", None);
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
