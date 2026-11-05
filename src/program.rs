use crate::artifact::Manifest;
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
const PILOT_NAMES: [&str; 4] = ["flins", "ineffa", "columbina", "sucrose"];

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
    Generated(crate::discovery::Rotation),
    /// Complete rotation source with its repetition policy already applied.
    Source(String),
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
    pub fn is_pilot(m: &Manifest) -> bool {
        m.format == 1 && m.members.iter().map(|c| c.key.as_str()).eq(PILOT_NAMES)
    }

    pub fn baseline(raw: &str, m: &Manifest) -> Result<Self, String> {
        gse::gcsl::parse(&format!("{}{}{}", m.header(), m.build(), raw))?;
        if Self::is_pilot(m) && raw.trim_start().starts_with("active ") {
            return Ok(Self::Library(raw.into()));
        }
        use gse::gcsl::Tok;
        let mut tokens = gse::gcsl::lex(raw);
        tokens.retain(|t| t.typ != Tok::Eof);
        let mut active = None;
        let mut depth = 0;
        let mut i = 0;
        while i < tokens.len() {
            match tokens[i].typ {
                Tok::LeftBrace => depth += 1,
                Tok::RightBrace => depth -= 1,
                Tok::Active if depth == 0 => {
                    if active.is_some()
                        || i + 2 >= tokens.len()
                        || tokens[i + 1].typ != Tok::CharacterKey
                        || tokens[i + 2].typ != Tok::TerminateLine
                    {
                        return Err(
                            "rotation needs exactly one top-level active declaration".into()
                        );
                    }
                    active = Some(tokens[i + 1].val.clone());
                    tokens.drain(i..i + 3);
                    continue;
                }
                _ => {}
            }
            i += 1;
        }
        let active = active.ok_or("rotation missing active declaration")?;
        // Keep global functions and initialization outside repetition. Start at
        // the first top-level control block, character action or other runtime
        // statement; function and let bodies can contain nested braces.
        let mut start = 0;
        while start < tokens.len() && matches!(tokens[start].typ, Tok::Fn | Tok::Let) {
            let function = tokens[start].typ == Tok::Fn;
            let mut nesting = 0;
            let mut end = start + 1;
            while end < tokens.len() {
                match tokens[end].typ {
                    Tok::LeftBrace => nesting += 1,
                    Tok::RightBrace => {
                        nesting -= 1;
                        if function && nesting == 0 {
                            end += 1;
                            break;
                        }
                    }
                    Tok::TerminateLine if nesting == 0 => {
                        end += 1;
                        break;
                    }
                    _ => {}
                }
                end += 1;
            }
            start = end;
        }
        if start == tokens.len() {
            return Err("rotation contains no repeated body".into());
        }
        let join = |ts: &[gse::gcsl::Token]| {
            ts.iter()
                .map(|t| t.val.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        };
        let mut guard = "optimizer_cycle_start".to_string();
        while tokens.iter().any(|t| t.val == guard) {
            guard.push('_');
        }
        let source = format!(
            "active {active};\n{}\nwhile 1 {{\nlet {guard}=f();\n{}\nif f() == {guard} {{ wait(1); }}\n}}\n",
            join(&tokens[..start]),
            join(&tokens[start..])
        );
        let p = Self::Source(source);
        gse::gcsl::parse(&format!("{}{}{}", m.header(), m.build(), p.gcsl(m)))?;
        Ok(p)
    }

    /// Editable rotation text of a source-backed program.
    pub fn source_text(&self) -> Option<&str> {
        match self {
            Self::Library(raw) | Self::Source(raw) => Some(raw),
            _ => None,
        }
    }

    pub(crate) fn with_source(&self, source: String) -> Self {
        match self {
            Self::Source(_) => Self::Source(source),
            _ => Self::Library(source),
        }
    }

    pub fn families(baseline: &str, m: &Manifest) -> Vec<Self> {
        if Self::is_pilot(m) {
            return Self::pilot_families(baseline);
        }
        vec![Self::baseline(baseline, m).expect("validated baseline")]
    }

    pub fn restricted_domain() -> Vec<Self> {
        let mut programs = Vec::new();
        for frames in [360, 600] {
            for normals in [2, 4] {
                for period in [1, 2] {
                    let mut p = Self::pilot_families("").remove(1);
                    if let Self::Typed { cycle, .. } = &mut p {
                        if let Step::Field {
                            frames: f,
                            normals: n,
                            ..
                        } = &mut cycle[3]
                        {
                            *f = frames;
                            *n = normals;
                        }
                        if let Step::Block { period: p, .. } = &mut cycle[0] {
                            *p = period;
                        }
                    }
                    programs.push(p);
                }
            }
        }
        programs
    }

    pub fn mutate_revised(&self, rng: &mut Rng, m: &Manifest) -> (Self, String) {
        if rng.index(100) < 60 {
            let neighbors = self.local_neighbors(m);
            if !neighbors.is_empty() {
                return neighbors[rng.index(neighbors.len())].clone();
            }
        }
        (self.mutate(rng, m), "legacy action mutation".into())
    }

    /// A bounded, deterministic neighborhood; no character-specific combos.
    pub fn local_neighbors(&self, m: &Manifest) -> Vec<(Self, String)> {
        if let Self::Generated(p) = self {
            return p
                .neighbors(m)
                .into_iter()
                .map(|(p, label)| (Self::Generated(p), label))
                .collect();
        }
        let (Self::Library(raw) | Self::Source(raw)) = self else {
            return Vec::new();
        };
        source_neighbors(raw, &m.build())
            .into_iter()
            .map(|(s, label)| (self.with_source(s), label))
            .collect()
    }

    fn pilot_families(baseline: &str) -> Vec<Self> {
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
        if let Self::Generated(p) = self {
            return p.validate();
        }
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
    pub fn gcsl(&self, m: &Manifest) -> String {
        match self {
            Self::Generated(p) => p.gcsl(m),
            Self::Source(raw) => raw.clone(),
            Self::Library(raw) => {
                // The imported baseline has only active setup followed by its
                // original finite for-loop. Keep all of its branches intact.
                let (setup, body) = raw
                    .split_once(';')
                    .expect("validated baseline active setup");
                format!("{setup};\nwhile 1 {{\n{body}\n}}\n")
            }
            Self::Typed { opening, cycle } => {
                let names: Vec<&str> = m.members.iter().map(|c| c.key.as_str()).collect();
                let active = if Self::is_pilot(m) {
                    names[1]
                } else {
                    names[0]
                };
                let mut out = format!("active {active};\n");
                for (i, s) in opening.iter().enumerate() {
                    out.push_str(&render(s, i, false, &names));
                }
                for i in 0..cycle.len() {
                    out.push_str(&format!("let gate{i}=0;\n"));
                }
                out.push_str("while 1 {\nlet cycle_start=f();\n");
                for (i, s) in cycle.iter().enumerate() {
                    out.push_str(&render(s, i, true, &names));
                }
                out.push_str("if f() == cycle_start { wait(1); }\n}\n");
                out
            }
        }
    }
    pub fn mutate(&self, rng: &mut Rng, m: &Manifest) -> Self {
        if let Self::Generated(p) = self {
            return crate::discovery::Catalog::configured(m, p.advanced)
                .map(|c| Self::Generated(p.mutate(&c, rng).0))
                .unwrap_or_else(|_| self.clone());
        }
        if let Self::Library(raw) | Self::Source(raw) = self
            && (matches!(self, Self::Source(_)) || rng.index(10) != 0)
        {
            if let Some(source) = mutate_source(raw, rng, m) {
                return self.with_source(source);
            }
            return self.clone();
        }
        let mut p = self.clone();
        if matches!(p, Self::Library(_) | Self::Source(_)) {
            // A deliberately simple baseline-derived seed, not a claimed
            // translation of conditional GCSL or a hidden high-cons cycle.
            p = Self::families("", m).remove(1);
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

/// Edit complete action lists while leaving all surrounding control tokens
/// untouched. Round-robin action positions keep long statements from crowding
/// out every other character under the neighborhood cap.
fn source_neighbors(raw: &str, context: &str) -> Vec<(String, String)> {
    use gse::gcsl::Tok;
    let lexed = gse::gcsl::lex(raw);
    if lexed.len() > 1200 {
        return Vec::new();
    }
    let text: Vec<String> = lexed
        .iter()
        .filter(|t| t.typ != Tok::Eof)
        .map(|t| t.val.clone())
        .collect();
    let mut statements = Vec::new();
    for i in 0..text.len().saturating_sub(1) {
        if lexed[i].typ != Tok::CharacterKey || lexed[i + 1].typ != Tok::ActionKey {
            continue;
        }
        let Some(end) = (i + 1..text.len()).find(|&j| [";", "{", "}"].contains(&text[j].as_str()))
        else {
            continue;
        };
        if text[end] != ";" {
            continue;
        }
        let actions: Vec<Vec<String>> = text[i + 1..end]
            .split(|t| t == ",")
            .map(|a| a.to_vec())
            .collect();
        if actions.iter().all(|a| !a.is_empty()) {
            statements.push((i, end, actions));
        }
    }
    let mut result = Vec::new();
    let mut seen = std::collections::BTreeSet::from([text.join(" ")]);
    let max_actions = statements
        .iter()
        .map(|(_, _, a)| a.len())
        .max()
        .unwrap_or(0);
    for position in 0..max_actions {
        for (si, (start, end, actions)) in statements.iter().enumerate() {
            if position >= actions.len() {
                continue;
            }
            let mut edits: Vec<(Vec<Vec<String>>, String)> = Vec::new();
            if position == 0 {
                edits.push((Vec::new(), format!("delete statement {si}")));
            }
            for width in 1..=3 {
                if position + width <= actions.len() {
                    let mut a = actions.clone();
                    a.drain(position..position + width);
                    edits.push((a, format!("delete actions {si}:{position}+{width}")));
                }
            }
            if actions[position][0] == "attack" {
                let count = actions[position]
                    .get(2)
                    .and_then(|v| v.parse::<i32>().ok())
                    .unwrap_or(1);
                for delta in [-2, -1, 1, 2] {
                    let n = count + delta;
                    if (1..=8).contains(&n) {
                        let mut a = actions.clone();
                        a[position] = vec!["attack".into(), ":".into(), n.to_string()];
                        edits.push((a, format!("normal count {si}:{position}:{count}->{n}")));
                    }
                }
            }
            for delta in [-1isize, 1] {
                let j = position as isize + delta;
                if j >= 0 && (j as usize) < actions.len() {
                    let mut a = actions.clone();
                    a.swap(position, j as usize);
                    edits.push((a, format!("relocate action {si}:{position}->{j}")));
                }
            }
            if actions.len() < 24 {
                let mut a = actions.clone();
                a.insert(position, a[position].clone());
                edits.push((a, format!("duplicate action {si}:{position}")));
            }
            for (actions, label) in edits {
                let replacement = if actions.is_empty() {
                    String::new()
                } else {
                    format!(
                        "{} {} ;",
                        text[*start],
                        actions
                            .iter()
                            .map(|a| a.join(" "))
                            .collect::<Vec<_>>()
                            .join(" , ")
                    )
                };
                let mut changed = text.clone();
                changed.splice(*start..=*end, [replacement]);
                let source = changed.join(" ");
                if !seen.insert(source.clone()) {
                    continue;
                }
                if gse::gcsl::parse(&format!("target lvl=100 resist=0.1;\n{context}\n{source}"))
                    .is_ok()
                {
                    result.push((source, label));
                    if result.len() == 256 {
                        return result;
                    }
                }
            }
        }
    }
    result
}

/// Mutate actual action lists while retaining the original program's guards,
/// loop variables and opening. This is not a trace flattened from one seed.
/// The engine lexer retains original token spellings, including quoted strings.
fn mutate_source(raw: &str, rng: &mut Rng, m: &Manifest) -> Option<String> {
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
            text.insert(at, format!("{} {action};", m.members[rng.index(4)].key));
        }
        _ => {
            text.splice(
                i..end,
                [format!("attack:{}, dash, skill, burst", 1 + rng.index(5))],
            );
        }
    }
    let source = text.join(" ");
    let context = m.build();
    gse::gcsl::parse(&format!("target lvl=100 resist=0.1;\n{context}\n{source}")).ok()?;
    Some(source)
}
fn render(s: &Step, i: usize, cyclic: bool, names: &[&str]) -> String {
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
                let name = names[a.actor];
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
            let name = names[*actor];
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
                let prerequisite = if name == "flins" {
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
    fn other_party_keeps_initialization_and_mutates_its_own_source() {
        let m = crate::artifact::tests::fixed_party();
        let raw = "fn ready() { return .noelle.burst.ready; }
let opening = 1;
active albedo;
for let i=0; i<4; i=i+1 {
  albedo skill, attack;
  bennett burst, skill;
  xiangling burst, skill;
  if ready() { noelle burst; }
  noelle attack:4;
}";
        let p = Program::baseline(raw, &m).unwrap();
        let source = p.gcsl(&m);
        assert!(source.find("fn ready").unwrap() < source.find("while 1").unwrap());
        assert!(source.find("let opening").unwrap() < source.find("while 1").unwrap());
        assert!(!source.contains("flins") && !source.contains("ineffa"));
        assert_eq!(Program::families(raw, &m), vec![p.clone()]);
        let neighbors = p.local_neighbors(&m);
        assert!(!neighbors.is_empty());
        for (neighbor, _) in &neighbors {
            assert!(matches!(neighbor, Program::Source(_)));
            gse::gcsl::parse(&crate::evaluate::Evaluator::config(&m, neighbor)).unwrap();
        }
        let mut rng = Rng(17);
        for _ in 0..100 {
            let candidate = p.mutate_revised(&mut rng, &m).0;
            assert!(matches!(candidate, Program::Source(_)));
            gse::gcsl::parse(&crate::evaluate::Evaluator::config(&m, &candidate)).unwrap();
        }
        let samples = gse::api::run_iterations(
            &crate::evaluate::Evaluator::config(&m, &p),
            150.0,
            991,
            0,
            2,
        )
        .unwrap();
        assert!(
            samples
                .iter()
                .all(|s| s.frames == 27000 && s.dps.is_finite())
        );
        let restored: Program = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(p, restored);
        assert!(Program::baseline("albedo attack;", &m).is_err());
    }
    #[test]
    fn block_neighborhood_preserves_guards_and_edits_every_actor() {
        let raw = "active ineffa; for let i=0; i<2; i=i+1 { if .sucrose.skill.ready { columbina attack:2, dash; sucrose attack:3; ineffa skill, burst; flins attack:4, skill, burst; } }";
        let p = Program::Library(raw.into());
        let m = crate::artifact::Manifest::load("data/flins-c1/manifest.json").unwrap();
        let neighbors = p.local_neighbors(&m);
        assert!(!neighbors.is_empty() && neighbors.len() <= 256);
        assert_eq!(neighbors, p.local_neighbors(&m));
        for (p, _) in &neighbors {
            let Program::Library(s) = p else {
                panic!("source must retain its representation");
            };
            assert!(s.contains("if .sucrose .skill .ready"));
            assert!(s.contains("i = i + 1"));
        }
        for index in 0..4 {
            assert!(
                neighbors
                    .iter()
                    .any(|(_, label)| label == &format!("delete statement {index}"))
            );
        }
        assert!(
            neighbors
                .iter()
                .any(|(_, l)| l.starts_with("normal count 3:"))
        );
        assert!(
            neighbors
                .iter()
                .any(|(_, l)| l.starts_with("normal count 0:"))
        );
        assert!(
            neighbors
                .iter()
                .any(|(_, l)| l.starts_with("delete actions") && l.ends_with("+2"))
        );
        assert!(
            neighbors
                .iter()
                .any(|(_, l)| l.starts_with("relocate action"))
        );
    }

    #[test]
    fn restricted_neighbors_are_exactly_the_declared_three_bit_domain() {
        let domain = Program::restricted_domain();
        assert_eq!(domain.len(), 8);
        for i in 0..8 {
            for bit in 0..3 {
                assert_ne!(domain[i], domain[i ^ (1 << bit)]);
            }
        }
        let m = Manifest::load("data/flins-c1/manifest.json").unwrap();
        let unique: std::collections::BTreeSet<_> = domain.iter().map(|p| p.gcsl(&m)).collect();
        assert_eq!(unique.len(), 8);
    }
    #[test]
    fn source_mutations_preserve_guards_and_can_introduce_new_actions() {
        let raw = std::fs::read_to_string("data/flins-c1/baseline.gcsl").unwrap();
        let m = crate::artifact::Manifest::load("data/flins-c1/manifest.json").unwrap();
        let mut rng = Rng(287);
        let mut changed = std::collections::BTreeSet::new();
        for _ in 0..100 {
            if let Some(s) = mutate_source(&raw, &mut rng, &m) {
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
            Program::Library(raw).gcsl(&m)
        );
        let b = format!(
            "{}{}{}",
            crate::artifact::HEADER,
            m.build(),
            Program::Library(normalized).gcsl(&m)
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
        for mut p in Program::families(&baseline, &m) {
            for _ in 0..100 {
                p.validate().unwrap();
                gse::gcsl::parse(&format!(
                    "{}{}{}",
                    crate::artifact::HEADER,
                    m.build(),
                    p.gcsl(&m)
                ))
                .unwrap();
                p = p.mutate(&mut rng, &m);
            }
        }
    }
}
