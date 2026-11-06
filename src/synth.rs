//! Cycle synthesis (`docs/cycle-synthesis-pilot.md` section 5).
//!
//! Character-agnostic additions to the held-out v2.1 search:
//! - a mechanical probe of every member's skill and burst (frames, cooldown,
//!   delayed cooldown of mode skills, own statuses) through the pinned
//!   engine's public simulation API;
//! - a typed cycle program (time-boxed on-field window with a priority
//!   policy, support visits with burst policies) rendered to GCSL, and typed
//!   moves on it;
//! - whole-cycle transplants from other parties' main-library rotations,
//!   retargeted by member slot;
//! - seed screening and family construction for the outer competition.
//!
//! No character name, per-case rule or case-tuned threshold appears here:
//! every rule is applied identically to every member.
use crate::artifact::Manifest;
use crate::evaluate::{Evaluator, Sample, stats};
use crate::heldout::{Dictionary, field_char, statements, tokens};
use crate::heldout_v2::{actions, assumptions};
use crate::program::{Program, Rng};
use gse::gcsl::Tok;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Screening seed namespace (disjoint from search 1e6+, promotion 1e9+,
/// selection 7e12+ and audit 4e12+ seeds).
pub const SYNTH_SEED: i64 = 3_000_000_000;
/// Skeleton grid programs screened per start.
pub const MAX_SKELETONS: usize = 48;
/// Retargeted transplant programs screened per start.
pub const MAX_TRANSPLANTS: usize = 24;
/// Screened programs completed to a 16-fight block.
pub const SCREEN_FINALISTS: usize = 4;
/// Window periods considered (frames).
const PERIOD_RANGE: (i32, i32) = (300, 1800);
/// Aura-wait timeout (frames).
const AURA_TIMEOUT: u16 = 120;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ActFacts {
    pub ok: bool,
    /// Frames until the next action could be queued.
    pub frames: i32,
    /// Frames from the start of the action until it is ready again.
    pub cycle: i32,
    /// The cooldown started only after the action (a mode skill).
    pub delayed: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct MemberFacts {
    pub key: String,
    pub skill: ActFacts,
    pub burst: ActFacts,
    /// Statuses/mods on the member itself that its skill or burst (or the
    /// following 20 s of normal attacks) created, with the longest observed
    /// remaining duration in frames.
    pub own_status: Vec<(String, i32)>,
    pub element: String,
    pub normal_hits: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Facts {
    pub members: Vec<MemberFacts>,
}

impl Facts {
    /// Candidate window periods: members' skill and burst cycle times in the
    /// declared range, merged when within 30 frames.
    pub fn periods(&self) -> Vec<u16> {
        let mut p: Vec<i32> = self
            .members
            .iter()
            .flat_map(|m| [&m.skill, &m.burst])
            .filter(|a| a.ok && a.cycle >= PERIOD_RANGE.0 && a.cycle <= PERIOD_RANGE.1)
            .map(|a| a.cycle)
            .collect();
        p.sort();
        let mut out: Vec<i32> = Vec::new();
        for x in p {
            if out.last().is_none_or(|l| x - l > 30) {
                out.push(x);
            }
        }
        out.into_iter().map(|x| x as u16).collect()
    }
}

fn probe_config(m: &Manifest) -> String {
    let header: String = m
        .header()
        .lines()
        .filter(|l| !l.starts_with("energy every"))
        .map(|l| format!("{l}\n"))
        .collect();
    format!("{header}{}active {};\n", m.build(), m.members[0].key)
}

fn own_mods(sim: &gse::sim::Sim, ci: usize) -> Vec<(String, i32)> {
    sim.chars[ci]
        .mods
        .iter()
        .filter(|x| x.expiry() > sim.f && !x.key.name.contains("icd"))
        .map(|x| (x.key.name.to_string(), x.expiry() - sim.f))
        .collect()
}

/// Runs the generic probe. Each member/action runs in a fresh simulation;
/// an engine panic or refusal marks that action as not probed.
pub fn probe(m: &Manifest) -> Result<Facts, String> {
    use gse::sim::{Params, Sim};
    use gse::types::Action;
    let src = probe_config(m);
    let parsed = gse::gcsl::parse(&src)?;
    let cfg = gse::profile::config(&parsed).map_err(|e| e.join("; "))?;
    let mut facts = Facts::default();
    for (ci, member) in m.members.iter().enumerate() {
        let name = gse::gcsl::char_key(&member.key)
            .map(gse::gcsl::char_name)
            .unwrap_or(&member.key)
            .to_string();
        let mut mf = MemberFacts {
            key: name.clone(),
            ..Default::default()
        };
        let mut statuses: Vec<(String, i32)> = Vec::new();
        for (slot, action, label) in [(0, Action::Skill, "skill"), (1, Action::Burst, "burst")] {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut sim = Sim::new(&cfg, 1, 0.0);
                if sim.player.active as usize != ci {
                    sim.queue_action(ci, Action::Swap, &Params::default());
                }
                sim.wait(60);
                let before: BTreeSet<String> =
                    own_mods(&sim, ci).into_iter().map(|x| x.0).collect();
                let f0 = sim.f;
                if !sim.queue_action(ci, action, &Params::default()) {
                    return None;
                }
                let frames = sim.f - f0;
                let read = |sim: &Sim| {
                    sim.field(&[name.clone(), label.to_string(), "cd".to_string()])
                        .map(|n| n.f() as i32)
                        .unwrap_or(0)
                };
                let mut cd = read(&sim);
                let mut delayed = false;
                let mut cycle = frames + cd;
                let mut seen: Vec<(String, i32)> = own_mods(&sim, ci)
                    .into_iter()
                    .filter(|x| !before.contains(&x.0))
                    .collect();
                if cd == 0 {
                    // A mode skill: its cooldown starts later.
                    for _ in 0..1800 {
                        if sim.wait(1) {
                            break;
                        }
                        cd = read(&sim);
                        if cd > 0 {
                            delayed = true;
                            cycle = sim.f - f0 + cd;
                            break;
                        }
                    }
                }
                // 20 s of normal attacks: statuses that appear on the member.
                let end = f0 + 1200;
                while sim.f < end {
                    if !sim.queue_action(ci, Action::Attack, &Params::default()) {
                        break;
                    }
                    for x in own_mods(&sim, ci) {
                        if !before.contains(&x.0) {
                            seen.push(x);
                        }
                    }
                }
                Some((
                    ActFacts {
                        ok: true,
                        frames,
                        cycle,
                        delayed,
                    },
                    seen,
                    format!("{:?}", sim.chars[ci].element).to_lowercase(),
                    sim.chars[ci].normal_hit_num,
                ))
            }));
            if let Ok(Some((a, seen, element, hits))) = result {
                if slot == 0 {
                    mf.skill = a;
                } else {
                    mf.burst = a;
                }
                mf.element = element;
                mf.normal_hits = hits;
                statuses.extend(seen);
            }
        }
        statuses.sort();
        let mut own: Vec<(String, i32)> = Vec::new();
        for (k, d) in statuses {
            if d < 60 {
                continue;
            }
            match own.iter_mut().find(|x| x.0 == k) {
                Some(x) => x.1 = x.1.max(d),
                None => own.push((k, d)),
            }
        }
        mf.own_status = own;
        facts.members.push(mf);
    }
    Ok(facts)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Burst {
    Always,
    Ready,
    Every(u8, u8),
    Never,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Rule {
    SkillReady,
    BurstReady,
    /// Dash when the normal counter reaches n.
    Dash(u8),
    /// Skill when ready while one of the member's own statuses is active.
    OwnStatusSkill(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Visit {
    pub member: usize,
    /// Action list without bursts (may be empty).
    pub actions: String,
    pub burst: Burst,
}

/// A typed cycle (section 5.2). Rendered with `source`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CycleSpec {
    pub d: usize,
    pub opener: bool,
    pub visits: Vec<Visit>,
    pub d_burst: Burst,
    pub rules: Vec<Rule>,
    pub unit: String,
    pub window: u16,
    pub aura: Option<String>,
    pub trailing: u16,
}

fn name(m: &Manifest, i: usize) -> String {
    gse::gcsl::char_key(&m.members[i].key)
        .map(gse::gcsl::char_name)
        .unwrap_or(&m.members[i].key)
        .to_string()
}

fn burst_text(out: &mut String, who: &str, b: &Burst, gate: usize) {
    match b {
        Burst::Always => out.push_str(&format!("{who} burst;\n")),
        Burst::Ready => out.push_str(&format!("if .{who}.burst.ready {{ {who} burst; }}\n")),
        Burst::Every(k, phase) => out.push_str(&format!(
            "if syn_g{gate} == {phase} {{ {who} burst; }}\nsyn_g{gate} = syn_g{gate} + 1;\nif syn_g{gate} >= {k} {{ syn_g{gate} = 0; }}\n"
        )),
        Burst::Never => {}
    }
}

impl CycleSpec {
    /// GCSL rotation text (before the evaluator's repetition wrapper).
    pub fn source(&self, m: &Manifest) -> String {
        let d = name(m, self.d);
        let mut out = format!("active {d};\nlet syn_t0 = 0;\nlet syn_a = 0;\n");
        for g in 0..=self.visits.len() {
            out.push_str(&format!("let syn_g{g} = 0;\n"));
        }
        out.push_str("syn_t0 = f();\n");
        if self.opener {
            out.push_str(&format!("if .{d}.skill.ready {{ {d} skill; }}\n"));
        }
        for (g, v) in self.visits.iter().enumerate() {
            let who = name(m, v.member);
            if !v.actions.trim().is_empty() {
                out.push_str(&format!("{who} {};\n", v.actions));
            }
            burst_text(&mut out, &who, &v.burst, g);
        }
        burst_text(&mut out, &d, &self.d_burst, self.visits.len());
        out.push_str(&format!("while f() - syn_t0 < {} {{\n", self.window));
        let mut first = true;
        for r in &self.rules {
            let (cond, act) = match r {
                Rule::SkillReady => (format!(".{d}.skill.ready"), "skill".to_string()),
                Rule::BurstReady => (format!(".{d}.burst.ready"), "burst".to_string()),
                Rule::Dash(n) => (
                    format!(".{d}.normal == {n} && .stam > 20"),
                    "dash".to_string(),
                ),
                Rule::OwnStatusSkill(k) => (
                    format!(".{d}.mods.{k} && .{d}.skill.ready"),
                    "skill".to_string(),
                ),
            };
            out.push_str(&format!(
                "{}if {cond} {{ {d} {act}; }} ",
                if first { "" } else { "else " }
            ));
            first = false;
        }
        let aura = self
            .aura
            .as_ref()
            .map(|e| format!("syn_a = f();\nwhile !.element.t0.{e} && f() - syn_a < {AURA_TIMEOUT} {{ wait(1); }}\n"))
            .unwrap_or_default();
        if first {
            out.push_str(&format!("{aura}{d} {};\n", self.unit));
        } else {
            out.push_str(&format!("else {{\n{aura}{d} {};\n}}\n", self.unit));
        }
        out.push_str("}\n");
        if self.trailing > 0 {
            out.push_str(&format!("wait({});\n", self.trailing));
        }
        out
    }

    pub fn program(&self, m: &Manifest) -> Result<Program, String> {
        Program::baseline(&self.source(m), m)
    }

    pub fn descriptor(&self) -> String {
        format!(
            "skeleton D{} P{} rules{}",
            self.d,
            self.window,
            self.rules.len()
        )
    }
}

/// Facts and statement vocabularies a typed move may use.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Context {
    pub facts: Facts,
    /// Per member: candidate burst-free action lists (from L and the
    /// member's most frequent library statements).
    pub statements: Vec<Vec<String>>,
    /// Per member: candidate on-field units (library statements without
    /// skill or burst, plus `attack`).
    pub units: Vec<Vec<String>>,
}

/// Splits an action statement's action list into (burst-free list, had burst).
fn strip_bursts(t: &[gse::gcsl::Token], start: usize, end: usize) -> (String, bool) {
    let mut kept = Vec::new();
    let mut had = false;
    for (a, b) in actions(t, start, end) {
        if t[a].typ == Tok::ActionKey && t[a].val == "burst" {
            had = true;
            continue;
        }
        kept.push(
            t[a..=b]
                .iter()
                .map(|x| x.val.as_str())
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    (kept.join(" , "), had)
}

/// A statement is portable when its parameters are literals (no variable).
fn portable(t: &[gse::gcsl::Token], start: usize, end: usize) -> bool {
    (start..=end).all(|k| !(t[k].typ == Tok::Identifier && k > 0 && t[k - 1].typ == Tok::Assign))
}

/// L's action statements as (member index, burst-free actions, had burst).
pub fn l_statements(l: &str, m: &Manifest) -> Vec<(usize, String, bool)> {
    let keys: Vec<Option<u16>> = m
        .members
        .iter()
        .map(|x| gse::gcsl::char_key(&x.key))
        .collect();
    let t = tokens(l);
    let mut out = Vec::new();
    for s in statements(&t) {
        let Some(i) = keys.iter().position(|k| *k == Some(s.owner)) else {
            continue;
        };
        if !portable(&t, s.start, s.end) {
            continue;
        }
        let (acts, had) = strip_bursts(&t, s.start, s.end);
        out.push((i, acts, had));
    }
    out
}

/// Builds the typed-move vocabulary.
pub fn context(facts: Facts, l: &str, m: &Manifest, dict: &Dictionary) -> Context {
    let ls = l_statements(l, m);
    let mut statements_v = Vec::new();
    let mut units_v = Vec::new();
    for i in 0..m.members.len() {
        let who = name(m, i);
        let mut st: Vec<String> = Vec::new();
        for (j, a, _) in &ls {
            if *j == i && !a.is_empty() && !st.contains(a) {
                st.push(a.clone());
            }
        }
        let mut un: Vec<String> = vec!["attack".into()];
        for b in dict.blocks.get(&who).map(Vec::as_slice).unwrap_or(&[]) {
            if b.kind != "action statement" {
                continue;
            }
            let t = tokens(&b.text);
            let Some(s) = statements(&t).into_iter().next() else {
                continue;
            };
            if !portable(&t, s.start, s.end) {
                continue;
            }
            let (acts, had) = strip_bursts(&t, s.start, s.end);
            if acts.is_empty() {
                continue;
            }
            let has_skill = acts.split(',').any(|x| x.trim().starts_with("skill"));
            if !had && !has_skill && un.len() < 4 && !un.contains(&acts) {
                un.push(acts.clone());
            }
            if has_skill && st.len() < 6 && !st.contains(&acts) {
                st.push(acts);
            }
        }
        if !st.iter().any(|x| x == "skill") {
            st.push("skill".into());
        }
        statements_v.push(st);
        units_v.push(un);
    }
    Context {
        facts,
        statements: statements_v,
        units: units_v,
    }
}

fn default_rules(ctx: &Context, d: usize) -> (bool, Vec<Rule>) {
    let f = &ctx.facts.members[d];
    if f.skill.delayed {
        (true, vec![])
    } else {
        (false, vec![Rule::SkillReady])
    }
}

/// The bounded skeleton grid (section 5.3, F1), deterministic per seed.
pub fn skeleton_grid(
    ctx: &Context,
    l: &str,
    m: &Manifest,
    on_field: &[usize],
    rng: &mut Rng,
) -> Vec<CycleSpec> {
    let ls = l_statements(l, m);
    let periods = ctx.facts.periods();
    let mut grid = Vec::new();
    for &d in on_field {
        let (opener, rules) = default_rules(ctx, d);
        for support_from_l in [true, false] {
            for every2 in [false, true] {
                let policy = if every2 {
                    Burst::Every(2, 0)
                } else {
                    Burst::Ready
                };
                let visits: Vec<Visit> = if support_from_l {
                    ls.iter()
                        .filter(|(i, _, _)| *i != d)
                        .map(|(i, a, had)| Visit {
                            member: *i,
                            actions: a.clone(),
                            burst: if *had { policy.clone() } else { Burst::Never },
                        })
                        .filter(|v| !v.actions.is_empty() || v.burst != Burst::Never)
                        .collect()
                } else {
                    let mut order: Vec<usize> = Vec::new();
                    for (i, _, _) in &ls {
                        if *i != d && !order.contains(i) {
                            order.push(*i);
                        }
                    }
                    for i in 0..m.members.len() {
                        if i != d && !order.contains(&i) {
                            order.push(i);
                        }
                    }
                    order
                        .into_iter()
                        .map(|i| Visit {
                            member: i,
                            actions: ctx.statements[i]
                                .iter()
                                .rev()
                                .find(|s| s.contains("skill"))
                                .cloned()
                                .unwrap_or_else(|| "skill".into()),
                            burst: policy.clone(),
                        })
                        .collect()
                };
                for unit in &ctx.units[d] {
                    for &p in &periods {
                        grid.push(CycleSpec {
                            d,
                            opener,
                            visits: visits.clone(),
                            d_burst: Burst::Ready,
                            rules: rules.clone(),
                            unit: unit.clone(),
                            window: p,
                            aura: None,
                            trailing: 0,
                        });
                    }
                }
            }
        }
    }
    // Deterministic subsample to the declared size.
    while grid.len() > MAX_SKELETONS {
        let i = rng.index(grid.len());
        grid.swap_remove(i);
    }
    grid
}

/// One typed move (section 5.4). `None` when the drawn move is impossible.
pub fn typed_move(spec: &CycleSpec, ctx: &Context, rng: &mut Rng) -> Option<(CycleSpec, String)> {
    let mut s = spec.clone();
    let periods = ctx.facts.periods();
    let n = s.visits.len();
    let policies = [
        Burst::Always,
        Burst::Ready,
        Burst::Every(2, 0),
        Burst::Every(2, 1),
        Burst::Every(3, 0),
        Burst::Never,
    ];
    let label = match rng.index(12) {
        0 => {
            let p = if !periods.is_empty() && rng.index(2) == 0 {
                periods[rng.index(periods.len())]
            } else if rng.index(2) == 0 {
                (s.window as f64 * 0.9) as u16
            } else {
                (s.window as f64 * 1.1) as u16
            };
            if p == s.window || !(120..=2400).contains(&p) {
                return None;
            }
            s.window = p;
            format!("typed window period -> {p}")
        }
        1 if n > 0 => {
            let i = rng.index(n);
            s.visits[i].burst = policies[rng.index(policies.len())].clone();
            format!("typed visit {i} burst -> {:?}", s.visits[i].burst)
        }
        2 => {
            s.d_burst = policies[rng.index(policies.len() - 1)].clone();
            format!("typed on-field burst -> {:?}", s.d_burst)
        }
        3 if n > 1 => {
            let i = rng.index(n);
            let j = rng.index(n);
            if i == j {
                return None;
            }
            let v = s.visits.remove(i);
            s.visits.insert(j, v);
            format!("typed move visit {i} -> {j}")
        }
        4 if n > 1 => {
            let i = rng.index(n);
            s.visits.remove(i);
            format!("typed delete visit {i}")
        }
        5 => {
            let others: Vec<usize> = (0..ctx.statements.len()).filter(|&i| i != s.d).collect();
            let member = others[rng.index(others.len())];
            let st = &ctx.statements[member];
            let actions = st[rng.index(st.len())].clone();
            let at = rng.index(n + 1);
            s.visits.insert(
                at,
                Visit {
                    member,
                    actions: actions.clone(),
                    burst: Burst::Never,
                },
            );
            format!("typed insert visit {member}: {actions}")
        }
        6 if n > 0 => {
            let i = rng.index(n);
            let st = &ctx.statements[s.visits[i].member];
            let a = st[rng.index(st.len())].clone();
            if a == s.visits[i].actions {
                return None;
            }
            s.visits[i].actions = a.clone();
            format!("typed replace visit {i}: {a}")
        }
        7 => {
            let u = &ctx.units[s.d];
            let a = u[rng.index(u.len())].clone();
            if a == s.unit {
                return None;
            }
            s.unit = a.clone();
            format!("typed unit -> {a}")
        }
        8 => {
            let f = &ctx.facts.members[s.d];
            let mut options = vec![Rule::SkillReady, Rule::BurstReady];
            if f.normal_hits > 1 {
                options.push(Rule::Dash(f.normal_hits.clamp(2, 6) as u8));
                options.push(Rule::Dash((f.normal_hits - 1).clamp(2, 6) as u8));
            }
            for (k, _) in &f.own_status {
                options.push(Rule::OwnStatusSkill(k.clone()));
            }
            let r = options[rng.index(options.len())].clone();
            if s.rules.contains(&r) {
                s.rules.retain(|x| x != &r);
                format!("typed remove rule {r:?}")
            } else {
                if s.rules.len() >= 4 {
                    return None;
                }
                let at = rng.index(s.rules.len() + 1);
                s.rules.insert(at, r.clone());
                format!("typed add rule {r:?}")
            }
        }
        9 if s.rules.len() > 1 => {
            let i = rng.index(s.rules.len());
            let r = s.rules.remove(i);
            let j = rng.index(s.rules.len() + 1);
            s.rules.insert(j, r);
            format!("typed reorder rule {i} -> {j}")
        }
        10 => {
            s.opener = !s.opener;
            format!("typed opener -> {}", s.opener)
        }
        11 => {
            let elements: Vec<String> = ctx
                .facts
                .members
                .iter()
                .enumerate()
                .filter(|(i, f)| {
                    *i != s.d
                        && ["pyro", "hydro", "electro", "cryo", "dendro"]
                            .contains(&f.element.as_str())
                })
                .map(|(_, f)| f.element.clone())
                .collect();
            let choice = if s.aura.is_some() || elements.is_empty() || rng.index(3) == 0 {
                None
            } else {
                Some(elements[rng.index(elements.len())].clone())
            };
            if choice == s.aura {
                let t = [0u16, 30, 60, 120][rng.index(4)];
                if t == s.trailing {
                    return None;
                }
                s.trailing = t;
                return Some((s, format!("typed trailing wait -> {t}")));
            }
            s.aura = choice;
            format!("typed aura wait -> {:?}", s.aura)
        }
        _ => return None,
    };
    Some((s, label))
}

/// A typed proposal: one move, or a compound of 2–3 with probability 0.3.
pub fn typed_proposal(
    spec: &CycleSpec,
    ctx: &Context,
    rng: &mut Rng,
) -> Option<(CycleSpec, String)> {
    let edits = if rng.index(100) < 30 {
        2 + rng.index(2)
    } else {
        1
    };
    let mut s = spec.clone();
    let mut labels = Vec::new();
    for _ in 0..edits {
        for _ in 0..8 {
            if let Some((n, l)) = typed_move(&s, ctx, rng) {
                s = n;
                labels.push(l);
                break;
            }
        }
    }
    if labels.is_empty() {
        return None;
    }
    let prefix = if labels.len() > 1 {
        format!("compound x{}: ", labels.len())
    } else {
        String::new()
    };
    Some((s, format!("{prefix}{}", labels.join(" | "))))
}

/// A main-library rotation (corpus entry).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Donor {
    pub team: String,
    /// Canonical engine names of the team's members.
    pub members: Vec<String>,
    pub rotation: String,
}

/// Retargets a donor rotation: every character token (statement owner or
/// field path element) naming a donor member is replaced by its mapping.
pub fn retarget(src: &str, mapping: &[(u16, u16)]) -> String {
    let map = |k: Option<u16>| {
        k.and_then(|k| mapping.iter().find(|x| x.0 == k))
            .map(|x| x.1)
    };
    tokens(src)
        .iter()
        .map(|t| match t.typ {
            Tok::CharacterKey => map(gse::gcsl::char_key(&t.val))
                .map(|k| gse::gcsl::char_name(k).to_string())
                .unwrap_or_else(|| t.val.clone()),
            Tok::Field => map(field_char(&t.val))
                .map(|k| format!(".{}", gse::gcsl::char_name(k)))
                .unwrap_or_else(|| t.val.clone()),
            _ => t.val.clone(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn permutations(items: &[u16], k: usize) -> Vec<Vec<u16>> {
    if k == 0 {
        return vec![vec![]];
    }
    let mut out = Vec::new();
    for (i, x) in items.iter().enumerate() {
        let mut rest = items.to_vec();
        rest.remove(i);
        for mut p in permutations(&rest, k - 1) {
            p.insert(0, *x);
            out.push(p);
        }
    }
    out
}

/// Transplant candidates (section 5.3, F2): donors containing an on-field
/// candidate, ranked by members shared with the party (desc) then team
/// name; every assignment of the remaining members to the donor's remaining
/// slots, up to the declared count.
pub fn transplants(
    corpus: &[Donor],
    dropped: &BTreeSet<String>,
    party: &str,
    m: &Manifest,
    on_field: &[usize],
) -> Vec<(String, usize, String)> {
    let keys: Vec<u16> = m
        .members
        .iter()
        .filter_map(|x| gse::gcsl::char_key(&x.key))
        .collect();
    let mut out = Vec::new();
    let mut ranked: Vec<(usize, usize, &Donor)> = Vec::new();
    for &d in on_field {
        let dk = keys[d];
        for donor in corpus {
            if donor.team == party || dropped.contains(&donor.team) || donor.members.len() != 4 {
                continue;
            }
            let dm: Vec<u16> = donor
                .members
                .iter()
                .filter_map(|x| gse::gcsl::char_key(x))
                .collect();
            if dm.len() != 4 || !dm.contains(&dk) {
                continue;
            }
            let shared = dm.iter().filter(|k| keys.contains(k)).count();
            ranked.push((d, shared, donor));
        }
    }
    ranked.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then(a.2.team.cmp(&b.2.team))
            .then(a.0.cmp(&b.0))
    });
    let mut seen = BTreeSet::new();
    for (d, _, donor) in ranked {
        if !seen.insert(donor.team.clone()) {
            continue;
        }
        let dm: Vec<u16> = donor
            .members
            .iter()
            .filter_map(|x| gse::gcsl::char_key(x))
            .collect();
        let free_slots: Vec<u16> = dm.iter().copied().filter(|k| !keys.contains(k)).collect();
        let free_members: Vec<u16> = keys.iter().copied().filter(|k| !dm.contains(k)).collect();
        for perm in permutations(&free_members, free_slots.len()) {
            let mut mapping: Vec<(u16, u16)> = dm
                .iter()
                .filter(|k| keys.contains(k))
                .map(|k| (*k, *k))
                .collect();
            mapping.extend(free_slots.iter().copied().zip(perm));
            out.push((donor.team.clone(), d, retarget(&donor.rotation, &mapping)));
            if out.len() >= MAX_TRANSPLANTS {
                return out;
            }
        }
    }
    out
}

/// Character-normalised token multiset of a rotation (every character name
/// becomes one placeholder), for the donor leakage filter.
fn normalised(src: &str) -> std::collections::BTreeMap<String, usize> {
    let mut out = std::collections::BTreeMap::new();
    for t in tokens(src) {
        let v = match t.typ {
            Tok::CharacterKey if gse::gcsl::char_key(&t.val).is_some() => "<C>".to_string(),
            Tok::Field if field_char(&t.val).is_some() => ".<C>".to_string(),
            _ => t.val.clone(),
        };
        *out.entry(v).or_default() += 1;
    }
    out
}

/// Multiset Jaccard similarity of the character-normalised token counts.
pub fn similarity(a: &str, b: &str) -> f64 {
    let (x, y) = (normalised(a), normalised(b));
    let keys: BTreeSet<&String> = x.keys().chain(y.keys()).collect();
    let (mut inter, mut union) = (0usize, 0usize);
    for k in keys {
        let (p, q) = (*x.get(k).unwrap_or(&0), *y.get(k).unwrap_or(&0));
        inter += p.min(q);
        union += p.max(q);
    }
    if union == 0 {
        0.0
    } else {
        inter as f64 / union as f64
    }
}

/// A screened seed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Seed {
    pub kind: String,
    pub d: usize,
    pub origin: String,
    pub program: Program,
    pub spec: Option<CycleSpec>,
    pub screen2: Option<f64>,
    pub screen16: Option<f64>,
    pub error: Option<String>,
}

/// Screens seeds: 2 fights each, the best `SCREEN_FINALISTS` completed to
/// 16 fights. `budget_left` stops screening at the cap. Every fight is
/// charged through the shared evaluator.
pub fn screen(
    e: &mut Evaluator,
    m: &Manifest,
    seeds: &mut [Seed],
    budget_left: &mut dyn FnMut(&Evaluator) -> bool,
) {
    for s in seeds.iter_mut() {
        if !budget_left(e) {
            s.error = Some("not screened: cap reached".into());
            continue;
        }
        match e.eval(m, &s.program, SYNTH_SEED, 2) {
            Ok(x) => s.screen2 = Some(stats(&x).mean),
            Err(err) => s.error = Some(err.chars().take(200).collect()),
        }
    }
    let mut order: Vec<usize> = (0..seeds.len())
        .filter(|&i| seeds[i].screen2.is_some())
        .collect();
    order.sort_by(|&a, &b| {
        seeds[b]
            .screen2
            .unwrap()
            .total_cmp(&seeds[a].screen2.unwrap())
    });
    for &i in order.iter().take(SCREEN_FINALISTS) {
        if !budget_left(e) {
            break;
        }
        match e.eval(m, &seeds[i].program, SYNTH_SEED, 16) {
            Ok(x) => seeds[i].screen16 = Some(stats(&x).mean),
            Err(err) => seeds[i].error = Some(err.chars().take(200).collect()),
        }
    }
}

/// On-field candidates: L's active member and the member with the highest
/// DPS in L's measurement (deduplicated).
pub fn on_field_candidates(l: &str, m: &Manifest, l_samples: &[Sample]) -> Vec<usize> {
    let keys: Vec<Option<u16>> = m
        .members
        .iter()
        .map(|x| gse::gcsl::char_key(&x.key))
        .collect();
    let t = tokens(l);
    let mut out = Vec::new();
    if let Some(i) = t.iter().position(|x| x.typ == Tok::Active)
        && let Some(k) = t.get(i + 1).and_then(|x| gse::gcsl::char_key(&x.val))
        && let Some(j) = keys.iter().position(|x| *x == Some(k))
    {
        out.push(j);
    }
    let st = stats(l_samples);
    let top = (0..m.members.len())
        .max_by(|&a, &b| st.chars[a].total_cmp(&st.chars[b]))
        .unwrap_or(0);
    if !out.contains(&top) {
        out.push(top);
    }
    out
}

/// Static admission of a seed: parses into the evaluator's wrapper, adds no
/// assumption construct beyond L's, and passes the v2.1 parameter rule.
pub fn admissible(
    src: &str,
    l: &str,
    m: &Manifest,
    proposer: &crate::heldout_v2::Proposer,
) -> Result<Program, String> {
    if assumptions(src)
        .iter()
        .zip(assumptions(l))
        .any(|(a, b)| *a > b)
    {
        return Err("adds an assumption construct".into());
    }
    if !proposer.params_allowed(l, src) {
        return Err("non-library parameter triple".into());
    }
    let p = Program::baseline(src, m)?;
    let parsed = gse::gcsl::parse(&Evaluator::config(m, &p))?;
    let unsupported = gse::check::unsupported(&parsed);
    if !unsupported.is_empty() {
        return Err(unsupported.join("; "));
    }
    Ok(p)
}

fn arg(args: &[String], name: &str) -> Result<String, String> {
    args.windows(2)
        .find(|w| w[0] == name)
        .map(|w| w[1].clone())
        .ok_or(format!("{name} required"))
}

/// `gso synth corpus|donor-filter|probe`.
pub fn cli(args: &[String]) -> Result<(), String> {
    let read = |p: &str| std::fs::read_to_string(p).map_err(|e| format!("{p}: {e}"));
    let write = |p: &str, v: &serde_json::Value| {
        std::fs::write(
            p,
            serde_json::to_string_pretty(v).map_err(|e| e.to_string())? + "\n",
        )
        .map_err(|e| e.to_string())
    };
    match args.get(1).map(String::as_str) {
        Some("corpus") => {
            // --list: NAME<TAB>PATH lines of main-library rotations (never alt/).
            let mut out = Vec::new();
            let mut skipped = Vec::new();
            for line in read(&arg(args, "--list")?)?.lines() {
                let (team, path) = line.split_once('\t').ok_or("list: NAME<TAB>PATH")?;
                if path.contains("/alt/") {
                    return Err(format!("corpus entry under alt/: {path}"));
                }
                let members: Vec<String> = team
                    .split('-')
                    .map(|w| gse::gcsl::char_key(w).map(|k| gse::gcsl::char_name(k).to_string()))
                    .collect::<Option<Vec<_>>>()
                    .unwrap_or_default();
                if members.len() != 4 {
                    skipped.push(team.to_string());
                    continue;
                }
                out.push(Donor { team: team.into(), members, rotation: read(path)? });
            }
            eprintln!("corpus: {} rotations, {} team names not resolved to four members", out.len(), skipped.len());
            write(&arg(args, "--out")?, &serde_json::to_value(&out).map_err(|e| e.to_string())?)
        }
        Some("donor-filter") => {
            // Offline (reads A): donors whose character-normalised token
            // similarity to A reaches the declared threshold are dropped.
            let corpus: Vec<Donor> = serde_json::from_str(&read(&arg(args, "--corpus")?)?).map_err(|e| e.to_string())?;
            let a = read(&arg(args, "--withheld")?)?;
            let threshold: f64 = arg(args, "--threshold").unwrap_or("0.9".into()).parse().map_err(|_| "threshold")?;
            let mut dropped = Vec::new();
            let mut max = 0.0f64;
            for d in &corpus {
                let s = similarity(&a, &d.rotation);
                max = max.max(s);
                if s >= threshold {
                    dropped.push(serde_json::json!({"team":d.team,"similarity":s}));
                }
            }
            let names: Vec<String> = dropped.iter().map(|x| x["team"].as_str().unwrap().to_string()).collect();
            write(&arg(args, "--out")?, &serde_json::json!({"threshold":threshold,"dropped":names,"details":dropped,"max_similarity":max}))
        }
        Some("probe") => {
            let m = Manifest::load(&arg(args, "--manifest")?)?;
            println!("{}", serde_json::to_string_pretty(&probe(&m)?).map_err(|e| e.to_string())?);
            Ok(())
        }
        _ => Err("synth corpus --list F --out F | donor-filter --corpus F --withheld F --out F [--threshold X] | probe --manifest F".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> Manifest {
        Manifest::load("data/flins-c1/manifest.json").expect("pilot manifest")
    }

    #[test]
    fn probe_reports_cooldowns_for_every_member() {
        let m = manifest();
        let f = probe(&m).expect("probe");
        assert_eq!(f.members.len(), 4);
        assert!(f.members.iter().any(|x| x.burst.ok && x.burst.cycle > 300));
        assert!(!f.periods().is_empty());
    }

    #[test]
    fn skeletons_render_parse_and_typed_moves_stay_valid() {
        let m = manifest();
        let l = std::fs::read_to_string("data/flins-c1/baseline.gcsl").unwrap_or_else(|_| {
            format!(
                "active {};\n{} skill;\n{} attack;\n",
                m.members[0].key, m.members[1].key, m.members[0].key
            )
        });
        let facts = probe(&m).expect("probe");
        let ctx = context(facts, &l, &m, &Dictionary::default());
        let grid = skeleton_grid(&ctx, &l, &m, &[0], &mut Rng(1));
        assert!(!grid.is_empty() && grid.len() <= MAX_SKELETONS);
        let mut rng = Rng(7);
        let mut spec = grid[0].clone();
        for _ in 0..200 {
            if let Some((s, _)) = typed_proposal(&spec, &ctx, &mut rng) {
                spec = s;
            }
            let p = spec.program(&m).expect("typed program parses");
            gse::gcsl::parse(&Evaluator::config(&m, &p)).expect("config parses");
        }
    }

    #[test]
    fn retarget_maps_owners_and_fields_and_similarity_ignores_names() {
        let a = gse::gcsl::char_key("bennett").unwrap();
        let b = gse::gcsl::char_key("xiangling").unwrap();
        let s = retarget(
            "bennett skill; if .bennett.burst.ready { bennett burst; }",
            &[(a, b)],
        );
        assert!(!s.contains("bennett") && s.contains("xiangling") && s.contains(".xiangling"));
        assert!((similarity("bennett skill;", "xiangling skill;") - 1.0).abs() < 1e-9);
        assert_eq!(permutations(&[1, 2, 3], 2).len(), 6);
    }
}
