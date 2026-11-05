use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    pub fn index(&mut self, n: usize) -> usize {
        self.next() as usize % n
    }
    pub fn uniform(&mut self) -> f64 {
        ((self.next() >> 11) as f64 + 0.5) / (1u64 << 53) as f64
    }
}
pub const NAMES: [&str; 4] = ["flins", "ineffa", "columbina", "sucrose"];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Action {
    pub actor: usize,
    pub name: String,
    pub count: u8,
    pub ready: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Step {
    Block {
        actions: Vec<Action>,
        period: u8,
        phase: u8,
    },
    Field {
        actor: usize,
        frames: u16,
        normals: u8,
        cancel: bool,
        skill: bool,
        burst: bool,
    },
    Wait(u16),
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Program {
    Library(String),
    Typed {
        opening: Vec<Step>,
        cycle: Vec<Step>,
    },
}

fn action(actor: usize, name: &str, ready: bool) -> Action {
    Action {
        actor,
        name: name.into(),
        count: 1,
        ready,
    }
}
fn block(actor: usize, names: &[&str], period: u8) -> Step {
    Step::Block {
        actions: names
            .iter()
            .map(|n| action(actor, n, matches!(*n, "skill" | "burst")))
            .collect(),
        period,
        phase: 0,
    }
}
impl Program {
    pub fn families(baseline: &str) -> Vec<Self> {
        vec![
            Self::Library(baseline.into()),
            Self::Typed {
                opening: vec![],
                cycle: vec![
                    block(1, &["skill", "burst"], 1),
                    block(2, &["skill", "burst"], 1),
                    block(3, &["attack", "skill"], 1),
                    Step::Field {
                        actor: 0,
                        frames: 600,
                        normals: 3,
                        cancel: true,
                        skill: true,
                        burst: true,
                    },
                ],
            },
            Self::Typed {
                opening: vec![],
                cycle: vec![
                    block(2, &["skill", "burst"], 2),
                    block(1, &["skill"], 1),
                    block(3, &["skill", "burst"], 2),
                    Step::Field {
                        actor: 0,
                        frames: 420,
                        normals: 2,
                        cancel: true,
                        skill: true,
                        burst: true,
                    },
                    block(3, &["attack", "skill"], 1),
                ],
            },
            Self::Typed {
                opening: vec![],
                cycle: vec![
                    block(0, &["skill", "attack", "skill", "burst"], 1),
                    block(1, &["skill", "burst"], 1),
                    block(2, &["skill", "burst"], 2),
                    Step::Field {
                        actor: 3,
                        frames: 480,
                        normals: 2,
                        cancel: true,
                        skill: true,
                        burst: true,
                    },
                ],
            },
        ]
    }
    pub fn validate(&self) -> Result<(), String> {
        if let Self::Typed { opening, cycle } = self {
            if cycle.is_empty() || cycle.len() > 20 || opening.len() > 8 {
                return Err("bounded program size".into());
            }
            for s in opening.iter().chain(cycle) {
                match s {
                    Step::Block {
                        actions,
                        period,
                        phase,
                    } => {
                        if !(1..=4).contains(period)
                            || phase >= period
                            || actions.is_empty()
                            || actions.len() > 12
                        {
                            return Err("invalid block bounds".into());
                        }
                        for a in actions {
                            if a.actor >= 4
                                || !(1..=5).contains(&a.count)
                                || !["attack", "charge", "skill", "burst", "dash", "jump", "swap"]
                                    .contains(&a.name.as_str())
                            {
                                return Err("unsupported typed action".into());
                            }
                        }
                    }
                    Step::Field {
                        actor,
                        frames,
                        normals,
                        ..
                    } if *actor >= 4
                        || !(60..=900).contains(frames)
                        || !(1..=5).contains(normals) =>
                    {
                        return Err("field window bounds".into());
                    }
                    Step::Wait(n) if *n > 120 => return Err("wait bound".into()),
                    _ => {}
                }
            }
        }
        Ok(())
    }
    pub fn gcsl(&self) -> String {
        match self {
            Self::Library(raw) => {
                // The imported baseline has only active setup followed by its
                // original finite for-loop. Keep all of its branches intact.
                let (setup, body) = raw
                    .split_once(';')
                    .expect("validated baseline active setup");
                format!("{setup};\nwhile 1 {{\n{body}\n}}\n")
            }
            Self::Typed { opening, cycle } => {
                let mut out = "active ineffa;\n".to_string();
                for (i, s) in opening.iter().enumerate() {
                    out.push_str(&render(s, i, false));
                }
                for i in 0..cycle.len() {
                    out.push_str(&format!("let gate{i}=0;\n"));
                }
                out.push_str("while 1 {\nlet cycle_start=f();\n");
                for (i, s) in cycle.iter().enumerate() {
                    out.push_str(&render(s, i, true));
                }
                out.push_str("if f() == cycle_start { wait(1); }\n}\n");
                out
            }
        }
    }
    pub fn mutate(&self, rng: &mut Rng) -> Self {
        if let Self::Library(raw) = self {
            if rng.index(10) != 0 {
                if let Some(source) = mutate_source(raw, rng) {
                    return Self::Library(source);
                }
                return self.clone();
            }
        }
        let mut p = self.clone();
        if matches!(p, Self::Library(_)) {
            // A deliberately simple baseline-derived seed, not a claimed
            // translation of conditional GCSL or a hidden high-cons cycle.
            p = Self::families("").remove(1);
        }
        let Self::Typed { opening, cycle } = &mut p else {
            unreachable!()
        };
        let i = rng.index(cycle.len());
        match rng.index(9) {
            0 if cycle.len() < 20 => {
                let actor = rng.index(4);
                let name = ["attack", "charge", "skill", "burst", "dash", "jump"][rng.index(6)];
                cycle.insert(i, block(actor, &[name], 1));
            }
            1 if cycle.len() > 1 => {
                cycle.remove(i);
            }
            2 => {
                let j = rng.index(cycle.len());
                cycle.swap(i, j);
            }
            3 => {
                let s = cycle[i].clone();
                if cycle.len() < 20 {
                    cycle.insert(i, s);
                }
            }
            4 => {
                if let Step::Block { period, phase, .. } = &mut cycle[i] {
                    *period = [1, 2, 4][rng.index(3)];
                    *phase = rng.index(*period as usize) as u8;
                }
            }
            5 => match &mut cycle[i] {
                Step::Field {
                    frames,
                    normals,
                    cancel,
                    ..
                } => {
                    *frames = ((*frames as i32) + [-120, -60, -30, 30, 60, 120][rng.index(6)])
                        .clamp(60, 900) as u16;
                    *normals = (1 + rng.index(5)) as u8;
                    *cancel = rng.index(2) == 0;
                }
                Step::Block { actions, .. } => {
                    let j = rng.index(actions.len());
                    actions[j].count =
                        (1 + rng.index(if actions[j].name == "attack" { 5 } else { 1 })) as u8;
                    actions[j].ready = !actions[j].ready;
                }
                Step::Wait(n) => *n = rng.index(121) as u16,
            },
            6 => match &mut cycle[i] {
                Step::Field {
                    actor,
                    skill,
                    burst,
                    ..
                } => {
                    if rng.index(4) == 0 {
                        *actor = rng.index(4);
                    } else if rng.index(2) == 0 {
                        *skill = !*skill;
                    } else {
                        *burst = !*burst;
                    }
                }
                Step::Block { actions, .. } => {
                    let j = rng.index(actions.len());
                    actions[j].actor = rng.index(4);
                }
                _ => {}
            },
            7 => {
                if opening.len() < 8 && rng.index(2) == 0 {
                    opening.push(cycle[i].clone());
                } else if !opening.is_empty() {
                    opening.remove(rng.index(opening.len()));
                }
            }
            _ => {
                cycle[i] = if rng.index(3) == 0 {
                    Step::Wait(rng.index(121) as u16)
                } else {
                    Step::Field {
                        actor: rng.index(4),
                        frames: (180 + rng.index(541)) as u16,
                        normals: (1 + rng.index(5)) as u8,
                        cancel: true,
                        skill: true,
                        burst: true,
                    }
                };
            }
        }
        if p.validate().is_ok() {
            p
        } else {
            self.clone()
        }
    }
}

/// Mutate actual action lists while retaining the original program's guards,
/// loop variables and opening. This is not a trace flattened from one seed.
/// The engine lexer retains original token spellings, including quoted strings.
fn mutate_source(raw: &str, rng: &mut Rng) -> Option<String> {
    use gse::gcsl::Tok;
    let tokens = gse::gcsl::lex(raw);
    if tokens.len() > 1200 {
        return None;
    }
    let mut text: Vec<String> = tokens
        .iter()
        .filter(|t| t.typ != Tok::Eof)
        .map(|t| t.val.clone())
        .collect();
    let candidates: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| t.typ == Tok::ActionKey)
        .map(|(i, _)| i)
        .collect();
    if candidates.is_empty() {
        return None;
    }
    let i = candidates[rng.index(candidates.len())];
    let mut end = i + 1;
    while end < text.len() && ![",", ";"].contains(&text[end].as_str()) {
        end += 1;
    }
    if end >= text.len() {
        return None;
    }
    let action = ["attack", "charge", "skill", "burst", "dash", "jump"][rng.index(6)];
    match rng.index(6) {
        0 => {
            text.splice(
                i..end,
                [format!(
                    "{action}:{}",
                    1 + rng.index(if action == "attack" { 5 } else { 1 })
                )],
            );
        }
        1 => {
            text.splice(end..end, [",".into(), action.into()]);
        }
        2 => {
            if end < text.len() && text[end] == "," {
                text.drain(i..end + 1);
            } else if i > 0 && text[i - 1] == "," {
                text.drain(i - 1..end);
            } else {
                text.splice(i..end, ["swap".into()]);
            }
        }
        3 => {
            let copy = text[i..end].to_vec();
            let mut insert = vec![",".into()];
            insert.extend(copy);
            text.splice(end..end, insert);
        }
        4 => {
            let starts: Vec<usize> = tokens
                .iter()
                .enumerate()
                .skip(2)
                .filter(|(_, t)| t.typ == Tok::CharacterKey)
                .map(|(i, _)| i)
                .collect();
            if starts.is_empty() {
                return None;
            }
            let at = starts[rng.index(starts.len())];
            text.insert(at, format!("{} {action};", NAMES[rng.index(4)]));
        }
        _ => {
            text.splice(
                i..end,
                [format!("attack:{}, dash, skill, burst", 1 + rng.index(5))],
            );
        }
    }
    let source = text.join(" ");
    let context = NAMES
        .iter()
        .map(|n| format!("{n} char lvl=90/90 cons=1 talent=10,10,10;\n"))
        .collect::<String>();
    gse::gcsl::parse(&format!("target lvl=100 resist=0.1;\n{context}{source}")).ok()?;
    Some(source)
}
fn render(s: &Step, i: usize, cyclic: bool) -> String {
    let mut out = String::new();
    match s {
        Step::Wait(n) => out.push_str(&format!("wait({n});\n")),
        Step::Block {
            actions,
            period,
            phase,
        } => {
            if cyclic {
                out.push_str(&format!("if gate{i} == {phase} {{\n"));
            }
            for a in actions {
                let name = NAMES[a.actor];
                if a.ready && matches!(a.name.as_str(), "skill" | "burst") {
                    out.push_str(&format!("if .{name}.{}.ready {{\n", a.name));
                }
                out.push_str(&format!("{name} {}:{};\n", a.name, a.count));
                if a.ready && matches!(a.name.as_str(), "skill" | "burst") {
                    out.push_str("}\n");
                }
            }
            if cyclic {
                out.push_str(&format!(
                    "}}\ngate{i}=gate{i}+1; if gate{i} >= {period} {{ gate{i}=0; }}\n"
                ));
            }
        }
        Step::Field {
            actor,
            frames,
            normals,
            cancel,
            skill,
            burst,
        } => {
            let name = NAMES[*actor];
            out.push_str(&format!(
                "{name} swap;\nlet end{i}=f()+{frames};\nlet steps{i}=0;\n"
            ));
            // An optional entry skill and normal are sourced ability blocks.
            // The same readiness policy is available to every field actor.
            if *skill {
                out.push_str(&format!("if .{name}.skill.ready {{ {name} skill; }}\n"));
            }
            out.push_str(&format!(
                "{name} attack;\nwhile f() < end{i} && steps{i} < 64 {{\n"
            ));
            if *skill {
                out.push_str(&format!("if .{name}.skill.ready {{ {name} skill; }}\n"));
            }
            if *burst {
                let prerequisite = if *actor == 0 {
                    " && .flins.status.thunderous-symphony"
                } else {
                    ""
                };
                out.push_str(&format!(
                    "if .{name}.burst.ready{prerequisite} {{ {name} burst; }}\n"
                ));
            }
            out.push_str(&format!("{name} attack:{normals};\n"));
            if *cancel {
                out.push_str(&format!("{name} dash;\n"));
            }
            out.push_str(&format!("steps{i}=steps{i}+1;\nwait(1);\n}}\n"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_mutations_preserve_guards_and_can_introduce_new_actions() {
        let raw = std::fs::read_to_string("data/flins-c1/baseline.gcsl").unwrap();
        let mut rng = Rng(287);
        let mut changed = std::collections::BTreeSet::new();
        for _ in 0..100 {
            if let Some(s) = mutate_source(&raw, &mut rng) {
                assert!(s.contains("is_even"));
                assert!(s.contains("i = i + 1"));
                changed.insert(s);
            }
        }
        assert!(changed.len() > 50);
        let m = crate::artifact::Manifest::load("data/flins-c1/manifest.json").unwrap();
        let normalized = gse::gcsl::lex(&raw)
            .iter()
            .filter(|t| t.typ != gse::gcsl::Tok::Eof)
            .map(|t| t.val.clone())
            .collect::<Vec<_>>()
            .join(" ");
        let a = format!(
            "{}{}{}",
            crate::artifact::HEADER,
            m.build(),
            Program::Library(raw).gcsl()
        );
        let b = format!(
            "{}{}{}",
            crate::artifact::HEADER,
            m.build(),
            Program::Library(normalized).gcsl()
        );
        assert_eq!(
            gse::api::run_iterations(&a, 150.0, 311, 0, 4).unwrap(),
            gse::api::run_iterations(&b, 150.0, 311, 0, 4).unwrap()
        );
    }
    #[test]
    fn generated_and_mutated_programs_parse_with_bounded_progress() {
        let m = crate::artifact::Manifest::load("data/flins-c1/manifest.json").unwrap();
        let baseline = std::fs::read_to_string("data/flins-c1/baseline.gcsl").unwrap();
        let mut rng = Rng(17);
        for mut p in Program::families(&baseline) {
            for _ in 0..100 {
                p.validate().unwrap();
                gse::gcsl::parse(&format!(
                    "{}{}{}",
                    crate::artifact::HEADER,
                    m.build(),
                    p.gcsl()
                ))
                .unwrap();
                p = p.mutate(&mut rng);
            }
        }
    }
}
