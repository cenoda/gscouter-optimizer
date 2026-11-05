//! Browser-local pilot refinement. The grammar is shared with the native pilot;
//! execution is incremental, with one fight per call and no filesystem or process.
//! Run inside a dedicated Worker: the host must terminate it if any call exceeds
//! its wall-time watchdog, since the engine does not have an instruction budget.
#![allow(dead_code)]

#[path = "../../src/artifact.rs"]
mod artifact;
#[path = "../../src/discovery.rs"]
mod discovery;
#[path = "../../src/heldout.rs"]
mod heldout;
#[path = "../../src/program.rs"]
mod program;

use artifact::{Manifest, Scoring};
use heldout::Dictionary;
use program::{Program, Rng};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeMap;

pub const ENGINE: &str = "b13b7864163e75852d4b9f282be009c94369ab29";
const SEARCH_SEED: i64 = 1_000_000_000_000;
const PROMOTION_SEED: i64 = 3_000_000_000_000;
const AUDIT_SEED: i64 = 7_000_000_000_000;
const DECISION_FIGHTS: usize = 16;
const PROMOTION_FIGHTS: usize = 32;
const MAX_SOURCE_BYTES: usize = 64 * 1024;

fn default_seed() -> u64 {
    11
}
fn default_budget() -> usize {
    1024
}
fn default_audit() -> usize {
    32
}
fn default_scoring() -> Scoring {
    Scoring {
        duration: 450,
        warmup: 150,
        swap_delay: 12,
        enemy_level: 100,
        resistance: 0.1,
        radius: 2.0,
        position: [0.0, 2.4],
        periodic_energy: true,
    }
}

#[derive(Clone, Deserialize)]
pub struct Input {
    pub build: String,
    pub rotation: String,
    pub dictionary: Dictionary,
    #[serde(default = "default_seed")]
    pub seed: u64,
    /// Discovery fight attempts only. Final audit has an explicit separate cap.
    #[serde(default = "default_budget")]
    pub max_evaluations: usize,
    #[serde(default = "default_audit")]
    pub audit_samples: usize,
    #[serde(default = "default_scoring")]
    pub scoring: Scoring,
}

#[derive(Clone, Debug, Serialize)]
pub struct Stats {
    pub n: usize,
    pub mean: f64,
    pub se: f64,
}
fn stats(v: &[f64]) -> Stats {
    let n = v.len();
    let mean = v.iter().sum::<f64>() / n as f64;
    let se = if n > 1 {
        (v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n * (n - 1)) as f64).sqrt()
    } else {
        0.0
    };
    Stats { n, mean, se }
}
fn paired(a: &[f64], b: &[f64]) -> Stats {
    assert_eq!(a.len(), b.len());
    stats(&a.iter().zip(b).map(|(x, y)| x - y).collect::<Vec<_>>())
}

#[derive(Clone)]
struct Candidate {
    program: Program,
    config: String,
}
impl Candidate {
    fn new(program: Program, m: &Manifest) -> Result<Self, String> {
        let config = format!("{}{}{}", m.header(), m.build(), program.gcsl(m));
        if config.len() > MAX_SOURCE_BYTES {
            return Err("candidate exceeded source limit".into());
        }
        let parsed = gse::gcsl::parse(&config)?;
        let missing = gse::check::unsupported(&parsed);
        if !missing.is_empty() {
            return Err(missing.join("; "));
        }
        Ok(Self { program, config })
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Initial,
    Decision,
    Promotion,
    Audit,
}
struct Block {
    kind: Kind,
    reference: Candidate,
    proposal: Candidate,
    reference_samples: Vec<f64>,
    proposal_samples: Vec<f64>,
    samples: usize,
    seed: i64,
    operator: String,
    restart: bool,
}
impl Block {
    fn phase(&self) -> &'static str {
        match self.kind {
            Kind::Initial => "initial",
            Kind::Decision => "decision",
            Kind::Promotion => "promotion",
            Kind::Audit => "audit",
        }
    }
    fn completed(&self) -> bool {
        self.reference_samples.len() == self.samples && self.proposal_samples.len() == self.samples
    }
}

pub struct Session {
    input: Input,
    manifest: Manifest,
    rng: Rng,
    baseline: Candidate,
    current: Candidate,
    best: Candidate,
    pending: Option<Block>,
    search_evaluations: usize,
    audit_evaluations: usize,
    failed_evaluations: usize,
    failed_proposals: usize,
    decisions: usize,
    accepted: usize,
    promotions: usize,
    promotion_attempts: usize,
    stall: usize,
    best_dps: Option<f64>,
    generic: Option<(String, Vec<(Program, String)>)>,
    micro: Option<(String, Vec<(String, String)>, usize)>,
    last_event: Value,
    complete: bool,
    stop_reason: Option<&'static str>,
    audit_baseline: Option<Stats>,
    audit_selected: Option<Stats>,
    improvement: Option<Stats>,
    audit_error: Option<String>,
}

impl Session {
    pub fn start(input: Input) -> Result<Self, String> {
        if !(128..=20_000).contains(&input.max_evaluations) {
            return Err(
                "max_evaluations must be between 128 and 20000 (discovery fight attempts)".into(),
            );
        }
        if !(16..=256).contains(&input.audit_samples) {
            return Err("audit_samples must be between 16 and 256".into());
        }
        if input.seed > 2_147_483_647 {
            return Err("seed must be an integer between 0 and 2147483647".into());
        }
        if input.build.len() > MAX_SOURCE_BYTES || input.rotation.len() > MAX_SOURCE_BYTES {
            return Err("build or rotation exceeds 64 KiB".into());
        }
        if input.scoring.duration > 600 {
            return Err("browser fights are limited to 600 seconds".into());
        }
        validate_dictionary(&input.dictionary)?;
        if setup_override(&input.rotation) {
            return Err("rotation cannot override the fixed build or scoring conditions".into());
        }
        let mut manifest =
            heldout::manifest_from_build(&input.build, "browser-user-build", BTreeMap::new())?;
        manifest.scoring = Some(input.scoring.clone());
        manifest.validate(&manifest)?;
        let baseline = Candidate::new(Program::baseline(&input.rotation, &manifest)?, &manifest)?;
        let seed = SEARCH_SEED + input.seed as i64 * 1_000_000;
        let pending = Some(Block {
            kind: Kind::Initial,
            reference: baseline.clone(),
            proposal: baseline.clone(),
            reference_samples: Vec::new(),
            proposal_samples: Vec::new(),
            samples: DECISION_FIGHTS,
            seed,
            operator: "baseline".into(),
            restart: false,
        });
        Ok(Self {
            rng: Rng(input.seed),
            current: baseline.clone(),
            best: baseline.clone(),
            baseline,
            manifest,
            input,
            pending,
            search_evaluations: 0,
            audit_evaluations: 0,
            failed_evaluations: 0,
            failed_proposals: 0,
            decisions: 0,
            accepted: 0,
            promotions: 0,
            promotion_attempts: 0,
            stall: 0,
            best_dps: None,
            generic: None,
            micro: None,
            last_event: json!({"kind":"started","operator":"baseline"}),
            complete: false,
            stop_reason: None,
            audit_baseline: None,
            audit_selected: None,
            improvement: None,
            audit_error: None,
        })
    }

    /// One engine fight at most. All sampling/acceptance state stays in this
    /// instance between calls; no simulated progress or asynchronous server work.
    pub fn step(&mut self) -> Result<Value, String> {
        if self.complete {
            return Ok(self.status());
        }
        if self.pending.is_none() {
            self.schedule()?;
        }
        let mut block = self.pending.take().ok_or("missing evaluation block")?;
        let on_reference = block.reference_samples.len() == block.proposal_samples.len();
        let index = if on_reference {
            block.reference_samples.len()
        } else {
            block.proposal_samples.len()
        };
        let candidate = if on_reference {
            &block.reference
        } else {
            &block.proposal
        };
        if block.kind == Kind::Audit {
            self.audit_evaluations += 1;
        } else {
            self.search_evaluations += 1;
        }
        let evaluated = self.evaluate(candidate, block.seed + index as i64);
        match evaluated {
            Ok(dps) => {
                if on_reference {
                    block.reference_samples.push(dps);
                } else {
                    block.proposal_samples.push(dps);
                }
                // Identical configs reuse a fight, and retain the paired seed.
                // The duplicate is not counted as another attempted engine fight.
                if block.reference.config == block.proposal.config {
                    if on_reference {
                        block.proposal_samples.push(dps);
                    }
                }
                if block.completed() {
                    self.resolve(block)?;
                } else {
                    self.pending = Some(block);
                }
            }
            Err(error) => {
                self.failed_evaluations += 1;
                self.last_event =
                    json!({"kind":"evaluation_failed","operator":block.operator,"error":error});
                match block.kind {
                    Kind::Initial => return Err(format!("baseline cannot be simulated: {error}")),
                    Kind::Audit => {
                        self.audit_error = Some(error);
                        self.complete = true;
                    }
                    Kind::Decision => {
                        self.decisions += 1;
                        self.stall += 1;
                    }
                    Kind::Promotion => { /* An unresolved comparison never changes the peak. */ }
                }
            }
        }
        Ok(self.status())
    }

    fn evaluate(&self, c: &Candidate, seed: i64) -> Result<f64, String> {
        let mut samples =
            gse::api::run_iterations(&c.config, self.input.scoring.warmup as f64, seed, 0, 1)?;
        let s = samples.pop().ok_or("engine returned no fight")?;
        if s.frames != self.input.scoring.duration as i32 * 60 || !s.dps.is_finite() {
            return Err("incomplete or non-finite full-horizon fight".into());
        }
        Ok(s.dps)
    }

    fn schedule(&mut self) -> Result<(), String> {
        // Reserve enough discovery attempts for a complete common decision and
        // its possible promotion. Never accept a partially sampled decision.
        if self.stop_reason.is_some()
            || self.search_evaluations + 2 * DECISION_FIGHTS + 2 * PROMOTION_FIGHTS
                > self.input.max_evaluations
        {
            self.begin_audit("budget");
            return Ok(());
        }
        let mut selected = None;
        for _ in 0..8 {
            let (reference, program, operator, restart) = self.propose();
            match Candidate::new(program, &self.manifest) {
                Ok(proposal) => {
                    selected = Some((reference, proposal, operator, restart));
                    break;
                }
                Err(e) => {
                    self.failed_proposals += 1;
                    self.last_event =
                        json!({"kind":"proposal_failed","operator":operator,"error":e});
                }
            }
        }
        // Bounded draws prevent a bad dictionary from hanging scheduling. A
        // no-op decision still runs real paired fights, consumes the declared
        // budget and never promotes an untested candidate.
        let (reference, proposal, operator, restart) = selected.unwrap_or_else(|| {
            (
                self.current.clone(),
                self.current.clone(),
                "no supported proposal in eight draws".into(),
                false,
            )
        });
        self.pending = Some(Block {
            kind: Kind::Decision,
            reference,
            proposal,
            reference_samples: Vec::new(),
            proposal_samples: Vec::new(),
            samples: DECISION_FIGHTS,
            seed: SEARCH_SEED
                + self.input.seed as i64 * 1_000_000
                + (self.decisions as i64 + 1) * 100,
            operator,
            restart,
        });
        Ok(())
    }

    fn propose(&mut self) -> (Candidate, Program, String, bool) {
        let polishing = self.search_evaluations >= self.input.max_evaluations * 3 / 4;
        if polishing {
            let raw = self.best.program.source_text().unwrap().to_string();
            if self.micro.as_ref().is_none_or(|(key, _, _)| key != &raw) {
                self.micro = Some((
                    raw.clone(),
                    heldout::numeric_neighbors(&raw, &self.manifest.build()),
                    0,
                ));
            }
            let (_, list, cursor) = self.micro.as_mut().unwrap();
            if let Some((source, label)) = list.get(*cursor).cloned() {
                *cursor += 1;
                return (
                    self.best.clone(),
                    self.best.program.with_source(source),
                    label,
                    false,
                );
            }
        }
        if !polishing && self.stall >= 40 {
            let mut next = self.best.program.clone();
            let mut label = String::from("restart");
            for _ in 0..3 {
                let (p, l) = self.macro_proposal(&next);
                next = p;
                label.push_str(&format!("; {l}"));
            }
            return (self.current.clone(), next, label, true);
        }
        let current = self.current.program.clone();
        let (next, label) = self.macro_proposal(&current);
        (self.current.clone(), next, label, false)
    }

    fn macro_proposal(&mut self, p: &Program) -> (Program, String) {
        let raw = p.source_text().expect("source program");
        let members: Vec<u16> = self
            .manifest
            .members
            .iter()
            .map(|m| gse::gcsl::char_key(&m.key).unwrap())
            .collect();
        let context = self.manifest.build();
        for _ in 0..8 {
            if self.rng.index(100) < 25 {
                if self.generic.as_ref().is_none_or(|(key, _)| key != raw) {
                    self.generic = Some((raw.to_string(), p.local_neighbors(&self.manifest)));
                }
                let list = &self.generic.as_ref().unwrap().1;
                if !list.is_empty() {
                    let (program, label) = list[self.rng.index(list.len())].clone();
                    return (program, format!("generic {label}"));
                }
            } else if let Some((source, label)) = heldout::macro_edit(
                raw,
                &self.input.dictionary,
                &members,
                &context,
                &mut self.rng,
            ) {
                return (p.with_source(source), label);
            }
        }
        (p.clone(), "no valid edit drawn".into())
    }

    fn resolve(&mut self, block: Block) -> Result<(), String> {
        if !block.completed() {
            return Err("incomplete paired block cannot change optimizer state".into());
        }
        let delta = paired(&block.proposal_samples, &block.reference_samples);
        match block.kind {
            Kind::Initial => {
                self.best_dps = Some(stats(&block.reference_samples).mean);
                self.last_event = json!({"kind":"baseline_measured","operator":"baseline"});
            }
            Kind::Decision => {
                self.decisions += 1;
                let accept = delta.mean >= 0.0 || block.restart;
                self.last_event = json!({"kind":if accept {"accepted"} else {"rejected"},"operator":block.operator,"delta":delta.mean,"se":delta.se});
                self.stall = if delta.mean > 0.0 || block.restart {
                    0
                } else {
                    self.stall + 1
                };
                if accept {
                    self.accepted += 1;
                    self.current = block.proposal.clone();
                    if block.proposal.config != self.best.config {
                        self.promotion_attempts += 1;
                        self.pending = Some(Block {
                            kind: Kind::Promotion,
                            reference: self.best.clone(),
                            proposal: block.proposal,
                            reference_samples: Vec::new(),
                            proposal_samples: Vec::new(),
                            samples: PROMOTION_FIGHTS,
                            seed: PROMOTION_SEED
                                + self.input.seed as i64 * 1_000_000
                                + self.promotion_attempts as i64 * 100,
                            operator: block.operator,
                            restart: false,
                        });
                    }
                }
            }
            Kind::Promotion => {
                if delta.mean > 0.0 {
                    self.best = block.proposal;
                    self.promotions += 1;
                    self.best_dps = Some(stats(&block.proposal_samples).mean);
                    self.last_event = json!({"kind":"promoted","operator":block.operator,"delta":delta.mean,"se":delta.se});
                } else {
                    self.last_event = json!({"kind":"peak_retained","operator":block.operator,"delta":delta.mean,"se":delta.se});
                }
            }
            Kind::Audit => {
                self.audit_baseline = Some(stats(&block.reference_samples));
                self.audit_selected = Some(stats(&block.proposal_samples));
                self.improvement = Some(delta);
                self.complete = true;
                self.last_event = json!({"kind":"completed","operator":"independent paired audit"});
            }
        }
        Ok(())
    }

    fn begin_audit(&mut self, reason: &'static str) {
        if self.complete || self.pending.as_ref().is_some_and(|b| b.kind == Kind::Audit) {
            return;
        }
        self.stop_reason = Some(self.stop_reason.unwrap_or(reason));
        self.pending = Some(Block {
            kind: Kind::Audit,
            reference: self.baseline.clone(),
            proposal: self.best.clone(),
            reference_samples: Vec::new(),
            proposal_samples: Vec::new(),
            samples: self.input.audit_samples,
            seed: AUDIT_SEED + self.input.seed as i64 * 1_000_000,
            operator: "independent paired audit".into(),
            restart: false,
        });
        self.last_event = json!({"kind":"audit_started","operator":"independent paired audit"});
    }

    pub fn finish(&mut self) -> Value {
        self.begin_audit("requested");
        self.status()
    }

    pub fn status(&self) -> Value {
        let phase = if self.complete {
            "complete"
        } else {
            self.pending.as_ref().map_or("decision", Block::phase)
        };
        json!({"format":1,"phase":phase,"complete":self.complete,
            "evaluations":self.search_evaluations+self.audit_evaluations,"search_evaluations":self.search_evaluations,
            "audit_evaluations":self.audit_evaluations,"max_evaluations":self.input.max_evaluations,
            "audit_samples":self.input.audit_samples,"decisions":self.decisions,"accepted":self.accepted,
            "promotions":self.promotions,"failed_evaluations":self.failed_evaluations,
            "failed_proposals":self.failed_proposals,"best_dps":self.best_dps,
            "operator":self.pending.as_ref().map(|b| b.operator.as_str()),
            "pending_samples":self.pending.as_ref().map_or(0, |b| b.reference_samples.len()+b.proposal_samples.len()),
            "pending_target":self.pending.as_ref().map_or(0, |b| 2*b.samples),
            "last_event":self.last_event,"seed":self.input.seed,"engine":ENGINE,"scoring":self.input.scoring,
            "gcsl":self.best.config,"baseline_gcsl":self.baseline.config,"baseline":self.audit_baseline,
            "selected":self.audit_selected,"improvement":self.improvement,"audit_error":self.audit_error,
            "stop_reason":self.stop_reason,
            "algorithm":"pilot dictionary greedy / fixed build / one chain / browser fight-count budget",
            "decision_samples":DECISION_FIGHTS,"promotion_samples":PROMOTION_FIGHTS,
            "audit_seed":AUDIT_SEED+self.input.seed as i64*1_000_000,
            "limitations":["Experimental rotation refinement, not an optimality guarantee",
                "Browser fight-count budgets are not the native pilot CPU-time budgets",
                "Search estimates use changing seed blocks; the paired final audit is the comparison",
                "Action parameters are simulator assumptions and may require practical play validation"]})
    }
}

fn setup_override(source: &str) -> bool {
    use gse::gcsl::Tok;
    gse::gcsl::lex(source).iter().any(|t| {
        matches!(
            t.typ,
            Tok::Options | Tok::Target | Tok::Energy | Tok::Hurt | Tok::Char | Tok::Add
        )
    })
}

fn validate_dictionary(d: &Dictionary) -> Result<(), String> {
    if d.format != 1 || d.blocks.len() > 256 {
        return Err("unsupported dictionary format or size".into());
    }
    let mut bytes = 0;
    for blocks in d.blocks.values() {
        if blocks.len() > heldout::MAX_BLOCKS_PER_CHARACTER {
            return Err("too many dictionary blocks".into());
        }
        let mut total = 0usize;
        for b in blocks {
            bytes += b.text.len();
            if b.count == 0 || b.text.len() > 4096 || setup_override(&b.text) {
                return Err("invalid dictionary block".into());
            }
            total = total
                .checked_add(b.count)
                .ok_or("dictionary count overflow")?;
        }
    }
    if bytes > 1024 * 1024 {
        return Err("dictionary exceeds 1 MiB".into());
    }
    Ok(())
}

thread_local! { static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) }; }

pub fn command(request: &str) -> Result<Value, String> {
    if request.len() > 2 * 1024 * 1024 {
        return Err("request exceeds 2 MiB".into());
    }
    let request: Value = serde_json::from_str(request).map_err(|e| e.to_string())?;
    SESSION.with(|slot| {
        let mut slot = slot
            .try_borrow_mut()
            .map_err(|_| "recursive optimizer call")?;
        match request["op"].as_str() {
            Some("start") => {
                let input: Input =
                    serde_json::from_value(request["input"].clone()).map_err(|e| e.to_string())?;
                *slot = Some(Session::start(input)?);
                Ok(slot.as_ref().unwrap().status())
            }
            Some("step") => slot.as_mut().ok_or("no active session")?.step(),
            Some("finish") => Ok(slot.as_mut().ok_or("no active session")?.finish()),
            Some("status") => Ok(slot.as_ref().ok_or("no active session")?.status()),
            _ => Err("unknown operation; expected start, step, finish or status".into()),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn gso_alloc(len: u32) -> *mut u8 {
    Box::into_raw(vec![0u8; len as usize].into_boxed_slice()) as *mut u8
}

/// # Safety
/// `ptr` must come from gso_alloc(len), and be freed exactly once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gso_free(ptr: *mut u8, len: u32) {
    drop(unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len as usize)) });
}

/// # Safety
/// `ptr` must be an unfreed result from gso_call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gso_free_result(ptr: *mut u8) {
    let bytes = unsafe { std::slice::from_raw_parts(ptr.add(4), 4) };
    let len = u32::from_le_bytes(bytes.try_into().unwrap()) as usize;
    drop(unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len + 8)) });
}

/// # Safety
/// `ptr,len` must be a valid allocated UTF-8 request buffer; it stays owned by
/// the caller. A result is [u32 status, u32 UTF-8 byte length, JSON bytes].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gso_call(ptr: *const u8, len: u32) -> *mut u8 {
    let result = if len > 2 * 1024 * 1024 {
        Err("request exceeds 2 MiB".into())
    } else {
        std::str::from_utf8(unsafe { std::slice::from_raw_parts(ptr, len as usize) })
            .map_err(|e| e.to_string())
            .and_then(command)
    };
    let (status, value) = match result {
        Ok(value) => (0u32, value),
        Err(error) => (1, json!({"error":error})),
    };
    let body = serde_json::to_vec(&value).expect("finite report JSON");
    let mut out = Vec::with_capacity(8 + body.len());
    out.extend_from_slice(&status.to_le_bytes());
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&body);
    Box::into_raw(out.into_boxed_slice()) as *mut u8
}

#[cfg(test)]
mod tests {
    use super::*;
    const BUILD: &str = include_str!(
        "../../reports/heldout-recovery-2026-11-05/inputs/bennett-fischl-lisa-xingqiu--z89bwQG6KGHg/build.txt"
    );
    const ROTATION: &str = include_str!(
        "../../reports/heldout-recovery-2026-11-05/inputs/bennett-fischl-lisa-xingqiu--z89bwQG6KGHg/L.gcsl"
    );
    const DICTIONARY: &str =
        include_str!("../../reports/heldout-recovery-2026-11-05/dictionary.json");
    fn input() -> Input {
        Input {
            build: BUILD.replace("cons=6", "cons=0"),
            rotation: ROTATION.into(),
            dictionary: serde_json::from_str(DICTIONARY).unwrap(),
            seed: 11,
            max_evaluations: 128,
            audit_samples: 16,
            scoring: Scoring {
                duration: 30,
                warmup: 0,
                ..default_scoring()
            },
        }
    }
    fn run(session: &mut Session) -> Value {
        for _ in 0..1000 {
            let r = session.step().unwrap();
            assert!(session.search_evaluations <= session.input.max_evaluations);
            assert!(session.audit_evaluations <= 2 * session.input.audit_samples);
            if r["complete"] == true {
                return r;
            }
        }
        panic!("bounded search did not finish")
    }

    #[test]
    fn local_search_is_deterministic_and_final_audit_replays_on_latest_native_engine() {
        let mut a = Session::start(input()).unwrap();
        let mut b = Session::start(input()).unwrap();
        let ar = run(&mut a);
        let br = run(&mut b);
        assert_eq!(ar, br);
        assert_eq!(ar["engine"], ENGINE);
        assert!(a.decisions > 0);
        assert_eq!(
            a.manifest
                .members
                .iter()
                .map(|m| m.cons)
                .collect::<Vec<_>>(),
            vec![0, 0, 0, 0]
        );
        assert_eq!(ar["stop_reason"], "budget");
        assert!(a.audit_error.is_none(), "{:?}", a.audit_error);
        let audit_seed = ar["audit_seed"].as_i64().unwrap();
        let base = gse::api::run_iterations(
            ar["baseline_gcsl"].as_str().unwrap(),
            0.0,
            audit_seed,
            0,
            16,
        )
        .unwrap();
        let selected =
            gse::api::run_iterations(ar["gcsl"].as_str().unwrap(), 0.0, audit_seed, 0, 16).unwrap();
        let base: Vec<_> = base.iter().map(|s| s.dps).collect();
        let selected: Vec<_> = selected.iter().map(|s| s.dps).collect();
        assert_eq!(ar["baseline"]["mean"], stats(&base).mean);
        assert_eq!(ar["selected"]["mean"], stats(&selected).mean);
        assert_eq!(ar["improvement"]["mean"], paired(&selected, &base).mean);
        assert_eq!(ar["improvement"]["se"], paired(&selected, &base).se);
        assert_eq!(ar["baseline"]["n"], 16);
        assert_eq!(ar["selected"]["n"], 16);
    }

    #[test]
    fn manual_finish_discards_incomplete_decision_and_audits_baseline() {
        let mut s = Session::start(input()).unwrap();
        // Initial identical-pair measurement uses exactly 16 actual fights.
        for _ in 0..16 {
            s.step().unwrap();
        }
        assert!(s.pending.is_none());
        s.step().unwrap();
        assert_eq!(s.decisions, 0);
        assert_eq!(s.accepted, 0);
        assert_eq!(s.promotions, 0);
        let spent = s.search_evaluations;
        assert_eq!(spent, 17);
        let finishing = s.finish();
        assert_eq!(finishing["phase"], "audit");
        assert!(!s.complete);
        let r = run(&mut s);
        assert_eq!(s.search_evaluations, spent);
        assert_eq!(r["stop_reason"], "requested");
        assert_eq!(r["gcsl"], r["baseline_gcsl"]);
        assert_eq!(r["improvement"]["mean"], 0.0);
        assert_eq!(r["improvement"]["se"], 0.0);
        assert_eq!(s.audit_evaluations, 16);
    }

    #[test]
    fn protected_peak_requires_complete_paired_promotion_and_strict_gain() {
        let mut s = Session::start(input()).unwrap();
        let baseline_config = s.best.config.clone();
        let mut candidate = s.best.clone();
        candidate.config.push_str("\n# distinct candidate\n");
        let block = |delta: f64| Block {
            kind: Kind::Promotion,
            reference: s.baseline.clone(),
            proposal: candidate.clone(),
            reference_samples: vec![10.; 32],
            proposal_samples: vec![10. + delta; 32],
            samples: 32,
            seed: 0,
            operator: "test semantic comparison".into(),
            restart: false,
        };
        let mut partial = block(1.0);
        partial.proposal_samples.pop();
        assert!(s.resolve(partial).is_err());
        assert_eq!(s.best.config, baseline_config);
        assert_eq!(s.promotions, 0);
        let tied = Block {
            kind: Kind::Promotion,
            reference: s.baseline.clone(),
            proposal: candidate.clone(),
            reference_samples: vec![10.; 32],
            proposal_samples: vec![10.; 32],
            samples: 32,
            seed: 0,
            operator: "tied".into(),
            restart: false,
        };
        s.resolve(tied).unwrap();
        assert_eq!(s.best.config, baseline_config);
        assert_eq!(s.promotions, 0);
        let worse = Block {
            kind: Kind::Promotion,
            reference: s.baseline.clone(),
            proposal: candidate.clone(),
            reference_samples: vec![10.; 32],
            proposal_samples: vec![9.; 32],
            samples: 32,
            seed: 0,
            operator: "worse".into(),
            restart: false,
        };
        s.resolve(worse).unwrap();
        assert_eq!(s.best.config, baseline_config);
        assert_eq!(s.promotions, 0);
        let better = Block {
            kind: Kind::Promotion,
            reference: s.baseline.clone(),
            proposal: candidate.clone(),
            reference_samples: vec![10.; 32],
            proposal_samples: vec![11.; 32],
            samples: 32,
            seed: 0,
            operator: "better".into(),
            restart: false,
        };
        s.resolve(better).unwrap();
        assert_eq!(s.best.config, candidate.config);
        assert_eq!(s.baseline.config, baseline_config);
        assert_eq!(s.promotions, 1);
    }

    #[test]
    fn rng_index_uses_full_random_word_before_target_width_conversion() {
        let mut indices = Rng(11);
        let mut words = Rng(11);
        for modulus in [3usize, 7, 100, 4093, 17, 127]
            .into_iter()
            .cycle()
            .take(1000)
        {
            let word = words.next();
            assert_eq!(indices.index(modulus), (word % modulus as u64) as usize);
        }
    }

    #[test]
    fn malformed_input_and_unrunnable_baseline_are_explicit_errors() {
        let mut bad = input();
        bad.max_evaluations = 0;
        assert!(
            Session::start(bad)
                .err()
                .unwrap()
                .contains("max_evaluations")
        );
        let mut bad = input();
        bad.rotation = "active somebody; nobody attack;".into();
        assert!(Session::start(bad).is_err());
        let mut bad = input();
        bad.build.push_str("\noptions duration=1;\n");
        assert!(Session::start(bad).is_err());
        let mut bad = input();
        bad.rotation.push_str("\noptions duration=1;\n");
        assert!(
            Session::start(bad)
                .err()
                .unwrap()
                .contains("scoring conditions")
        );
        let mut bad = input();
        let block = bad
            .dictionary
            .blocks
            .values_mut()
            .find(|v| !v.is_empty())
            .unwrap();
        block[0].count = 0;
        assert!(Session::start(bad).err().unwrap().contains("dictionary"));
        assert!(command("{\"op\":\"bogus\"}").is_err());
    }

    #[test]
    fn abi_result_frame_matches_json_and_owns_its_buffers() {
        let request = b"{\"op\":\"unknown\"}";
        unsafe {
            let ptr = gso_alloc(request.len() as u32);
            std::ptr::copy_nonoverlapping(request.as_ptr(), ptr, request.len());
            let result = gso_call(ptr, request.len() as u32);
            gso_free(ptr, request.len() as u32);
            let header = std::slice::from_raw_parts(result, 8);
            assert_eq!(u32::from_le_bytes(header[0..4].try_into().unwrap()), 1);
            let len = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
            let body: Value =
                serde_json::from_slice(std::slice::from_raw_parts(result.add(8), len)).unwrap();
            assert!(
                body["error"]
                    .as_str()
                    .unwrap()
                    .contains("unknown operation")
            );
            gso_free_result(result);
        }
    }
}
