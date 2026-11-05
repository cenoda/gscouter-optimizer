//! Build-only rotation grammar. No library sources or character-specific combos.
use crate::{artifact::Manifest, program::Rng};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const MAX_VISITS: usize = 12;
const MAX_ACTIONS: usize = 12;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Command {
    pub action: String,
    pub count: u8,
    pub ready: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<String, i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancel: Option<Cancel>,
    #[serde(default)]
    pub delay: u8,
    #[serde(default)]
    pub hold_before: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Cancel {
    pub collision: bool,
    pub travel: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Field {
    pub frames: u16,
    pub pattern: Vec<Command>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeats: Option<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Visit {
    pub actor: usize,
    pub entry: Vec<Command>,
    pub field: Option<Field>,
    pub period: u8,
    pub phase: u8,
    #[serde(default)]
    pub pickup: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Rotation {
    pub active: usize,
    pub opening: Vec<Visit>,
    pub cycle: Vec<Visit>,
    #[serde(default)]
    pub advanced: bool,
}

/// Bounded action forms checked against the actual build, without party cycles.
pub struct Catalog {
    actions: Vec<Vec<String>>,
    forms: Vec<Vec<Command>>,
    cancels: Vec<bool>,
    advanced: bool,
}

impl Catalog {
    #[cfg(all(test, not(gso_browser)))]
    pub fn new(m: &Manifest) -> Result<Self, String> {
        Self::configured(m, true)
    }

    pub fn configured(m: &Manifest, advanced: bool) -> Result<Self, String> {
        let context = format!("{}{}active {};\n", m.header(), m.build(), m.members[0].key);
        let parsed = gse::gcsl::parse(&format!("{context}wait(1);"))?;
        let unsupported = gse::check::unsupported(&parsed);
        if !unsupported.is_empty() {
            return Err(unsupported.join("; "));
        }
        let mut actions = Vec::new();
        let mut forms = Vec::new();
        let mut cancels = Vec::new();
        let cfg = gse::profile::config(&parsed).map_err(|e| e.join("; "))?;
        let sim = gse::sim::Sim::new(&cfg, 1, 0.0);
        for member in &m.members {
            let ns = sim
                .field(&[member.key.clone(), "nightsoul".into(), "points".into()])
                .is_ok();
            let mut available = Vec::new();
            let mut variants = Vec::new();
            for action in ["attack", "charge", "skill", "burst", "dash", "jump"] {
                let p = gse::gcsl::parse(&format!("{context}{} {action};", member.key))?;
                if gse::check::unsupported(&p).is_empty() {
                    available.push(action.to_string());
                    variants.push(command(action, &mut Rng(0)));
                    let params: Vec<(&str, i64)> = match action {
                        "skill" => vec![("hold", 1), ("recast", 1)],
                        "charge" => vec![
                            ("final", 1),
                            ("hold", 1),
                            ("hold", 6),
                            ("hold", 15),
                            ("hold", 30),
                            ("hold", 40),
                            ("hold", 45),
                            ("buffered", 0),
                            ("buffered", 6),
                        ],
                        _ => vec![],
                    };
                    for (key, value) in params {
                        if !advanced {
                            continue;
                        }
                        if key == "recast" && !ns {
                            continue;
                        }
                        let p = gse::gcsl::parse(&format!(
                            "{context}{} {action}[{key}={value}];",
                            member.key
                        ))?;
                        if gse::check::unsupported(&p).is_empty() {
                            let mut form = command(action, &mut Rng(0));
                            form.count = 1;
                            form.params.insert(key.into(), value);
                            if key == "recast" {
                                form.ready = false;
                            }
                            variants.push(form);
                        }
                    }
                }
            }
            if !available.iter().any(|a| a == "attack") {
                return Err(format!(
                    "{}: generated grammar requires normal attacks",
                    member.key
                ));
            }
            cancels.push(ns && variants.iter().any(|v| v.params.contains_key("final")));
            actions.push(available);
            forms.push(variants);
        }
        Ok(Self {
            actions,
            forms,
            cancels,
            advanced,
        })
    }

    fn supports(&self, actor: usize, action: &str) -> bool {
        self.actions[actor].iter().any(|a| a == action)
    }

    fn command(&self, actor: usize, rng: &mut Rng) -> Command {
        let action = &self.actions[actor][rng.index(self.actions[actor].len())];
        self.named(actor, action, rng)
    }

    fn named(&self, actor: usize, action: &str, rng: &mut Rng) -> Command {
        let forms: Vec<_> = self.forms[actor]
            .iter()
            .filter(|c| c.action == action)
            .collect();
        // Sample parameter kinds before their values, so a longer numeric grid
        // does not crowd out default forms or a distinct action mode.
        let kinds: std::collections::BTreeSet<_> = forms
            .iter()
            .map(|c| c.params.keys().next().cloned())
            .collect();
        let kind = kinds.iter().nth(rng.index(kinds.len())).unwrap();
        let forms: Vec<_> = forms
            .into_iter()
            .filter(|c| c.params.keys().next().cloned() == *kind)
            .collect();
        let mut c = forms[rng.index(forms.len())].clone();
        c.count = if matches!(action, "attack" | "charge") && !c.params.contains_key("final") {
            (1 + rng.index(4)) as u8
        } else {
            1
        };
        if action == "charge" && self.cancels[actor] && rng.index(2) == 0 {
            c.cancel = Some(Cancel {
                collision: rng.index(2) == 0,
                travel: 6,
            });
        }
        if c.params.get("final") == Some(&1) && rng.index(3) == 0 {
            c.hold_before = rng.index(46) as u8;
        }
        c
    }

    fn supports_command(&self, actor: usize, command: &Command) -> bool {
        self.forms[actor]
            .iter()
            .any(|f| f.action == command.action && f.params == command.params)
            && (command.cancel.is_none() || self.cancels[actor])
    }

    fn tune(&self, command: &mut Command, actor: usize, rng: &mut Rng) {
        if !self.advanced {
            return;
        }
        match rng.index(4) {
            0 => {
                let next = self.named(actor, &command.action, rng);
                command.params = next.params;
                command.ready = next.ready;
                command.cancel = next.cancel;
                command.hold_before = next.hold_before;
            }
            1 if command.action == "charge" && self.cancels[actor] => {
                command.cancel = if command.cancel.is_some() && rng.index(4) == 0 {
                    None
                } else {
                    Some(Cancel {
                        collision: rng.index(2) == 0,
                        travel: (6 + rng.index(19)) as u8,
                    })
                };
            }
            2 if command.params.get("final") == Some(&1) => {
                command.hold_before = (i32::from(command.hold_before)
                    + [-12, -6, -3, -1, 1, 3, 6, 12][rng.index(8)])
                .clamp(0, 45) as u8;
            }
            _ => {
                command.delay = (i32::from(command.delay) + [-15, -6, -1, 1, 6, 15][rng.index(6)])
                    .clamp(0, 60) as u8
            }
        }
    }

    fn visit(&self, actor: usize, rng: &mut Rng, field: bool) -> Visit {
        let mut entry = Vec::new();
        for action in ["skill", "burst"] {
            if self.supports(actor, action) && rng.index(4) != 0 {
                entry.push(self.named(actor, action, rng));
            }
        }
        if rng.index(2) == 0 {
            entry.reverse();
        }
        if entry.is_empty() || rng.index(3) == 0 {
            entry.push(command("attack", rng));
        }
        Visit {
            actor,
            entry,
            field: field.then(|| self.field(actor, rng)),
            period: if rng.index(4) == 0 { 2 } else { 1 },
            phase: 0,
            pickup: self.advanced && rng.index(2) == 0,
        }
    }

    fn initial_visit(&self, actor: usize, rng: &mut Rng, field: bool) -> Visit {
        // Kit coverage is a generic starting prior, not a party rotation:
        // exercise every member's skill, a short normal string and ready burst.
        // Mutations can remove, reorder, replace or duplicate all of these.
        let mut entry = Vec::new();
        if self.supports(actor, "skill") {
            let mut skill = command("skill", rng);
            if field
                && self.cancels[actor]
                && rng.index(2) == 0
                && self.forms[actor]
                    .iter()
                    .any(|f| f.action == "skill" && f.params.get("hold") == Some(&1))
            {
                skill.params.insert("hold".into(), 1);
            }
            entry.push(skill);
        }
        let mut normal = command("attack", rng);
        normal.count = 2;
        entry.push(normal);
        if self.supports(actor, "burst") {
            entry.push(command("burst", rng));
        }
        let field = field.then(|| {
            if self.cancels[actor] {
                Field {
                    frames: [480, 600, 720, 900][rng.index(4)],
                    pattern: vec![self.named(actor, "charge", rng)],
                    repeats: (rng.index(2) == 0).then(|| (1 + rng.index(8)) as u8),
                }
            } else {
                self.field(actor, rng)
            }
        });
        Visit {
            actor,
            entry,
            field,
            period: 1,
            phase: 0,
            pickup: self.advanced && rng.index(2) == 0,
        }
    }

    fn field(&self, actor: usize, rng: &mut Rng) -> Field {
        let primary = if self.cancels[actor] && rng.index(2) == 0 {
            "charge"
        } else {
            "attack"
        };
        let mut pattern = vec![self.named(actor, primary, rng)];
        if rng.index(2) == 0 {
            pattern.push(self.command(actor, rng));
        }
        Field {
            frames: [180, 300, 480, 600, 900][rng.index(5)],
            pattern,
            repeats: (self.advanced && rng.index(2) == 0).then(|| (1 + rng.index(8)) as u8),
        }
    }

    fn reassign(&self, visit: &mut Visit, actor: usize, rng: &mut Rng) {
        visit.actor = actor;
        for action in visit
            .entry
            .iter_mut()
            .chain(visit.field.iter_mut().flat_map(|f| &mut f.pattern))
        {
            if !self.supports_command(actor, action) {
                *action = self.command(actor, rng);
            }
        }
    }
}

fn command(action: &str, rng: &mut Rng) -> Command {
    Command {
        action: action.into(),
        count: if action == "attack" {
            (1 + rng.index(6)) as u8
        } else {
            1
        },
        ready: matches!(action, "skill" | "burst"),
        params: BTreeMap::new(),
        cancel: None,
        delay: 0,
        hold_before: 0,
    }
}

impl Rotation {
    pub fn generate(catalog: &Catalog, rng: &mut Rng, field_actor: usize) -> Self {
        let mut order = [0, 1, 2, 3];
        for i in (1..4).rev() {
            order.swap(i, rng.index(i + 1));
        }
        let cycle: Vec<_> = order
            .into_iter()
            .map(|actor| catalog.initial_visit(actor, rng, actor == field_actor))
            .collect();
        let opening = if rng.index(2) == 0 && catalog.supports(field_actor, "skill") {
            vec![Visit {
                actor: field_actor,
                entry: vec![command("skill", rng)],
                field: None,
                period: 1,
                phase: 0,
                pickup: false,
            }]
        } else {
            Vec::new()
        };
        Self {
            active: order[0],
            opening,
            cycle,
            advanced: catalog.advanced,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.active >= 4
            || self.cycle.is_empty()
            || self.cycle.len() > MAX_VISITS
            || self.opening.len() > 4
        {
            return Err("generated rotation size or active member is invalid".into());
        }
        for v in self.opening.iter().chain(&self.cycle) {
            if !self.advanced
                && (v.pickup
                    || v.entry
                        .iter()
                        .chain(v.field.iter().flat_map(|f| &f.pattern))
                        .any(|a| {
                            !a.params.is_empty()
                                || a.cancel.is_some()
                                || a.delay != 0
                                || a.hold_before != 0
                        })
                    || v.field.as_ref().is_some_and(|f| f.repeats.is_some()))
            {
                return Err("basic grammar cannot contain extended action forms".into());
            }
            if v.actor >= 4
                || !(1..=4).contains(&v.period)
                || v.phase >= v.period
                || v.entry.len() > MAX_ACTIONS
                || (v.entry.is_empty() && v.field.is_none())
            {
                return Err("generated visit is invalid".into());
            }
            if let Some(f) = &v.field
                && (!(30..=1200).contains(&f.frames)
                    || f.pattern.is_empty()
                    || f.pattern.len() > 12
                    || f.repeats.is_some_and(|n| !(1..=8).contains(&n)))
            {
                return Err("generated field window is invalid".into());
            }
            for a in v
                .entry
                .iter()
                .chain(v.field.iter().flat_map(|f| &f.pattern))
            {
                if !["attack", "charge", "skill", "burst", "dash", "jump"]
                    .contains(&a.action.as_str())
                    || !(1..=8).contains(&a.count)
                    || (!matches!(a.action.as_str(), "attack" | "charge") && a.count != 1)
                    || (a.ready && !["skill", "burst"].contains(&a.action.as_str()))
                    || a.delay > 60
                    || a.hold_before > 45
                    || (a.hold_before > 0 && a.params.get("final") != Some(&1))
                    || (a.params.contains_key("recast") && a.ready)
                    || a.params
                        .iter()
                        .any(|(k, v)| match (a.action.as_str(), k.as_str()) {
                            ("skill", "hold" | "recast") | ("charge", "final") => {
                                ![0, 1].contains(v)
                            }
                            ("charge", "hold") => !(1..=45).contains(v),
                            ("charge", "buffered") => !(0..=15).contains(v),
                            _ => true,
                        })
                    || a.cancel
                        .as_ref()
                        .is_some_and(|c| a.action != "charge" || !(6..=24).contains(&c.travel))
                {
                    return Err("generated action is invalid".into());
                }
            }
        }
        Ok(())
    }

    pub fn gcsl(&self, m: &Manifest) -> String {
        let mut out = format!("active {};\n", m.members[self.active].key);
        for (i, visit) in self.opening.iter().enumerate() {
            render_visit(&mut out, visit, m, &format!("open{i}"));
        }
        for i in 0..self.cycle.len() {
            out.push_str(&format!("let generation_gate{i}=0;\n"));
        }
        out.push_str("while 1 {\nlet generation_start=f();\n");
        for (i, visit) in self.cycle.iter().enumerate() {
            out.push_str(&format!("if generation_gate{i} == {} {{\n", visit.phase));
            render_visit(&mut out, visit, m, &format!("visit{i}"));
            out.push_str(&format!("}}\ngeneration_gate{i}=generation_gate{i}+1;\nif generation_gate{i} >= {} {{ generation_gate{i}=0; }}\n", visit.period));
        }
        out.push_str("if f() == generation_start { wait(1); }\n}\n");
        out
    }

    /// Coarse behavior signature for reports, not a claim of semantic identity.
    pub fn structure(&self) -> String {
        self.cycle
            .iter()
            .map(|v| {
                format!(
                    "{}{}:{}/{}",
                    v.actor,
                    if v.field.is_some() { "F" } else { "Q" },
                    v.phase,
                    v.period
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn mutate(&self, catalog: &Catalog, rng: &mut Rng) -> (Self, String) {
        let op = rng.index(17);
        let i = rng.index(self.cycle.len());
        self.edit(catalog, rng, op, i)
    }

    fn edit(&self, catalog: &Catalog, rng: &mut Rng, op: usize, i: usize) -> (Self, String) {
        let mut p = self.clone();
        let label = match op {
            0 if p.cycle.len() < MAX_VISITS => {
                let actor = rng.index(4);
                let field = rng.index(2) == 0;
                p.cycle.insert(i, catalog.visit(actor, rng, field));
                "insert visit"
            }
            1 if p.cycle.len() > 1 => {
                p.cycle.remove(i);
                "delete visit"
            }
            2 => {
                let visit = p.cycle.remove(i);
                let at = rng.index(p.cycle.len() + 1);
                p.cycle.insert(at, visit);
                "relocate visit"
            }
            3 if p.cycle.len() < MAX_VISITS => {
                p.cycle.insert(i, p.cycle[i].clone());
                "duplicate visit"
            }
            4 => {
                let actor = (p.cycle[i].actor + 1 + rng.index(3)) % 4;
                catalog.reassign(&mut p.cycle[i], actor, rng);
                "change field ownership"
            }
            5 => {
                let v = &mut p.cycle[i];
                v.period = [1, 2, 3, 4][rng.index(4)];
                v.phase = rng.index(v.period as usize) as u8;
                "change visit period"
            }
            6 if p.cycle.len() < MAX_VISITS => {
                let v = &mut p.cycle[i];
                if v.entry.len() > 1 {
                    let entry = v.entry.split_off(v.entry.len() / 2);
                    let next = Visit {
                        entry,
                        field: v.field.take(),
                        ..v.clone()
                    };
                    p.cycle.insert(i + 1, next);
                } else if let Some(f) = &mut v.field
                    && f.frames >= 60
                {
                    let mut next = v.clone();
                    next.entry.clear();
                    next.field.as_mut().unwrap().frames /= 2;
                    v.field.as_mut().unwrap().frames -= next.field.as_ref().unwrap().frames;
                    p.cycle.insert(i + 1, next);
                }
                "split visit"
            }
            7 if i + 1 < p.cycle.len() => {
                let a = &p.cycle[i];
                let b = &p.cycle[i + 1];
                if a.actor == b.actor
                    && a.period == b.period
                    && a.phase == b.phase
                    && a.field.is_none()
                    && a.entry.len() + b.entry.len() <= MAX_ACTIONS
                {
                    let next = p.cycle.remove(i + 1);
                    p.cycle[i].entry.extend(next.entry);
                    p.cycle[i].field = next.field;
                }
                "merge visits"
            }
            8 => {
                let v = &mut p.cycle[i];
                if let Some(f) = &mut v.field {
                    f.frames = (i32::from(f.frames) + [-120, -30, 30, 120][rng.index(4)])
                        .clamp(30, 1200) as u16;
                    if rng.index(5) == 0 {
                        v.field = None;
                    }
                } else {
                    v.field = Some(catalog.field(v.actor, rng));
                }
                "change field length"
            }
            9 => {
                let v = &mut p.cycle[i];
                let actor = v.actor;
                let actions = if let Some(field) = &mut v.field
                    && rng.index(2) == 0
                {
                    &mut field.pattern
                } else {
                    &mut v.entry
                };
                edit_actions(actions, catalog, actor, rng);
                "edit action pattern"
            }
            10 => {
                let visit = p.cycle.remove(i);
                p.cycle.insert(0, visit);
                let v = &mut p.cycle[0];
                v.period = 1;
                v.phase = 0;
                let at = rng.index(p.cycle.len());
                let v = &mut p.cycle[at];
                v.field = Some(catalog.field(v.actor, rng));
                "compound order and field"
            }
            11 => {
                p.active = rng.index(4);
                "change active member"
            }
            12 => {
                if p.opening.len() < 4 && (p.opening.is_empty() || rng.index(2) == 0) {
                    p.opening.push(catalog.visit(rng.index(4), rng, false));
                } else if !p.opening.is_empty() {
                    let at = rng.index(p.opening.len());
                    p.opening.remove(at);
                }
                "change opening"
            }
            13 => {
                // A genuine change between a longer repeat and multiple shorter
                // opportunities: duplicate one visit, then change both gates.
                if p.cycle.len() < MAX_VISITS {
                    let mut next = p.cycle[i].clone();
                    p.cycle[i].period = 2;
                    p.cycle[i].phase = 0;
                    next.period = 2;
                    next.phase = 1;
                    if let Some(f) = &mut next.field {
                        f.frames = (f.frames / 2).max(30);
                    }
                    p.cycle.insert(i + 1, next);
                }
                "compound repeat structure"
            }
            14 => {
                let v = &mut p.cycle[i];
                let actions = if let Some(f) = &mut v.field {
                    &mut f.pattern
                } else {
                    &mut v.entry
                };
                if !actions.is_empty() {
                    let at = rng.index(actions.len());
                    catalog.tune(&mut actions[at], v.actor, rng);
                }
                "change parameters and timing"
            }
            15 => {
                if catalog.advanced {
                    p.cycle[i].pickup = !p.cycle[i].pickup;
                }
                "change crystallize pickup"
            }
            16 => {
                if catalog.advanced {
                    let v = &mut p.cycle[i];
                    if let Some(field) = &mut v.field {
                        field.repeats = if field.repeats.is_some() && rng.index(3) == 0 {
                            None
                        } else {
                            Some((1 + rng.index(8)) as u8)
                        };
                    }
                }
                "change finite field repeat"
            }
            _ => "bounded structural no-op",
        };
        if p.validate().is_err() {
            return (self.clone(), format!("{label}: invalid no-op"));
        }
        (p, format!("generated {label}"))
    }

    pub fn neighbors(&self, m: &Manifest) -> Vec<(Self, String)> {
        let Ok(catalog) = Catalog::configured(m, self.advanced) else {
            return Vec::new();
        };
        let mut rng = Rng(0x47454e45524943);
        let mut seen = std::collections::BTreeSet::from([self.gcsl(m)]);
        let mut result = Vec::new();
        let mut reductions = Vec::new();
        for i in 0..self.opening.len() {
            let mut p = self.clone();
            p.opening.remove(i);
            reductions.push(p);
        }
        for (i, v) in self.cycle.iter().enumerate() {
            if self.cycle.len() > 1 {
                let mut p = self.clone();
                p.cycle.remove(i);
                reductions.push(p);
            }
            for j in 0..v.entry.len() {
                let mut p = self.clone();
                p.cycle[i].entry.remove(j);
                reductions.push(p.clone());
                // Removing setup time can move a burst just before cooldown.
                // Also test queuing that same visit's later burst explicitly.
                for a in p.cycle[i].entry.iter_mut().skip(j) {
                    if a.action == "burst" {
                        a.ready = false;
                    }
                }
                reductions.push(p);
            }
            if let Some(f) = &v.field {
                for j in 0..f.pattern.len() {
                    if f.pattern.len() > 1 {
                        let mut p = self.clone();
                        p.cycle[i].field.as_mut().unwrap().pattern.remove(j);
                        reductions.push(p);
                    }
                }
            }
        }
        for p in reductions {
            if p.validate().is_ok() && seen.insert(p.gcsl(m)) {
                result.push((p, "generated local simplification".into()));
                if result.len() == 256 {
                    return result;
                }
            }
        }
        // Round-robin command positions across fields first, then entries.
        // These local edits enumerate forms and timing instead of drawing a
        // single random parameter mutation for an entire visit.
        for field in [true, false] {
            for position in 0..12 {
                for (i, visit) in self.cycle.iter().enumerate() {
                    let commands = if field {
                        let Some(f) = &visit.field else {
                            continue;
                        };
                        &f.pattern
                    } else {
                        &visit.entry
                    };
                    let Some(old) = commands.get(position) else {
                        continue;
                    };
                    let mut edits = Vec::new();
                    for form in &catalog.forms[visit.actor] {
                        let mut next = form.clone();
                        next.count = if matches!(next.action.as_str(), "attack" | "charge")
                            && !next.params.contains_key("final")
                        {
                            old.count.min(4)
                        } else {
                            1
                        };
                        next.delay = old.delay;
                        edits.push(next.clone());
                        if next.action == "charge" && catalog.cancels[visit.actor] {
                            for collision in [false, true] {
                                next.cancel = Some(Cancel {
                                    collision,
                                    travel: 6,
                                });
                                edits.push(next.clone());
                            }
                        }
                    }
                    for count in 1..=4 {
                        if matches!(old.action.as_str(), "attack" | "charge") {
                            let mut next = old.clone();
                            next.count = count;
                            edits.push(next);
                        }
                    }
                    if catalog.advanced {
                        if old.params.get("final") == Some(&1) {
                            for shift in [-12, -6, -3, -1, 1, 3, 6, 12] {
                                let mut next = old.clone();
                                next.hold_before =
                                    (i32::from(old.hold_before) + shift).clamp(0, 45) as u8;
                                edits.push(next);
                            }
                        }
                        for delay in [0, 1, 3, 6, 15, 30, 45, 60] {
                            let mut next = old.clone();
                            next.delay = delay;
                            edits.push(next);
                        }
                    }
                    if matches!(old.action.as_str(), "skill" | "burst")
                        && !old.params.contains_key("recast")
                    {
                        let mut next = old.clone();
                        next.ready = !next.ready;
                        edits.push(next);
                    }
                    for next in edits {
                        let mut p = self.clone();
                        let actions = if field {
                            &mut p.cycle[i].field.as_mut().unwrap().pattern
                        } else {
                            &mut p.cycle[i].entry
                        };
                        actions[position] = next;
                        if p.validate().is_ok() && seen.insert(p.gcsl(m)) {
                            result.push((p, format!("generated local form {i}:{position}")));
                            if result.len() == 256 {
                                return result;
                            }
                        }
                    }
                }
            }
        }
        for (i, visit) in self.cycle.iter().enumerate() {
            if let Some(f) = &visit.field {
                if catalog.advanced {
                    for repeats in std::iter::once(None).chain((1..=8).map(Some)) {
                        let mut p = self.clone();
                        p.cycle[i].field.as_mut().unwrap().repeats = repeats;
                        if seen.insert(p.gcsl(m)) {
                            result.push((p, format!("generated local field repeat {i}")));
                            if result.len() == 256 {
                                return result;
                            }
                        }
                    }
                }
                for shift in [-120, -60, -30, 30, 60, 120] {
                    let mut p = self.clone();
                    p.cycle[i].field.as_mut().unwrap().frames =
                        (i32::from(f.frames) + shift).clamp(30, 1200) as u16;
                    if seen.insert(p.gcsl(m)) {
                        result.push((p, format!("generated local field length {i}")));
                        if result.len() == 256 {
                            return result;
                        }
                    }
                }
            }
        }
        // Round-robin visits and operators, fixed RNG, independent of audit data.
        for op in 0..17 {
            for i in 0..self.cycle.len() {
                let (p, label) = self.edit(&catalog, &mut rng, op, i);
                if seen.insert(p.gcsl(m)) {
                    result.push((p, label));
                    if result.len() == 256 {
                        return result;
                    }
                }
            }
        }
        result
    }
}

fn edit_actions(actions: &mut Vec<Command>, catalog: &Catalog, actor: usize, rng: &mut Rng) {
    if actions.is_empty() {
        actions.push(catalog.command(actor, rng));
        return;
    }
    let i = rng.index(actions.len());
    match rng.index(8) {
        0 if actions.len() < 12 => actions.insert(i, catalog.command(actor, rng)),
        1 if actions.len() > 1 => {
            actions.remove(i);
        }
        2 => actions[i] = catalog.command(actor, rng),
        3 if matches!(actions[i].action.as_str(), "attack" | "charge") => {
            actions[i].count = (1 + rng.index(8)) as u8
        }
        4 if matches!(actions[i].action.as_str(), "skill" | "burst")
            && !actions[i].params.contains_key("recast") =>
        {
            actions[i].ready = !actions[i].ready
        }
        6 | 7 => catalog.tune(&mut actions[i], actor, rng),
        _ => {
            let j = rng.index(actions.len());
            actions.swap(i, j);
        }
    }
}

fn render_actions(out: &mut String, actions: &[Command], actor: &str) {
    let status_actor = gse::gcsl::char_key(actor)
        .map(gse::gcsl::char_name)
        .unwrap_or(actor);
    for a in actions {
        let recast = a.params.get("recast") == Some(&1);
        if recast {
            out.push_str(&format!(
                "if .{actor}.nightsoul.points > 0 && !.{actor}.skill.ready {{\n"
            ));
        }
        if a.ready {
            out.push_str(&format!("if .{actor}.{}.ready {{\n", a.action));
        }
        if a.delay > 0 {
            out.push_str(&format!("delay({});\n", a.delay));
        }
        if a.hold_before > 0 {
            out.push_str(&format!("{actor} charge[hold={}];\n", a.hold_before));
        }
        let params = if a.params.is_empty() {
            String::new()
        } else {
            format!(
                "[{}]",
                a.params
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };
        out.push_str(&format!("{actor} {}{params}:{};\n", a.action, a.count));
        if let Some(cancel) = &a.cancel {
            let required = 11
                + if cancel.collision {
                    cancel.travel.div_ceil(6)
                } else {
                    1
                };
            // A linked charge continuation preserves the engine's dash-cancel
            // prerequisite. Resource guards apply to the cancel, not the charge.
            if a.params.get("final") != Some(&1) {
                // Keep a cyclic charge held while a dash is on cooldown;
                // waiting without extending it would queue an unwanted final.
                out.push_str(&format!("while !.{actor}.dash.ready && (.{actor}.nightsoul.points > {required} || .{actor}.status.{status_actor}-burst) {{\n{actor} charge[hold=1];\n}}\n"));
            }
            out.push_str(&format!("if .{actor}.nightsoul.points > {required} || .{actor}.status.{status_actor}-burst {{\n{actor} dash[charge_cancel=1,collision={},travel={}],charge;\n}}\n", u8::from(cancel.collision),cancel.travel));
        }
        if a.ready {
            out.push_str("}\n");
        }
        if recast {
            out.push_str("}\n");
        }
    }
}

fn render_visit(out: &mut String, v: &Visit, m: &Manifest, id: &str) {
    let actor = &m.members[v.actor].key;
    out.push_str(&format!("{actor} swap;\n"));
    render_actions(out, &v.entry, actor);
    if v.pickup {
        out.push_str("pick_up_crystallize(\"any\");\n");
    }
    if let Some(f) = &v.field {
        if let Some(repeats) = f.repeats {
            out.push_str(&format!(
                "for let {id}_steps=0; {id}_steps<{repeats}; {id}_steps={id}_steps+1 {{\n"
            ));
        } else {
            out.push_str(&format!(
            "let {id}_end=f()+{};\nlet {id}_steps=0;\nwhile f() < {id}_end && {id}_steps < 64 {{\n",
            f.frames
        ));
        }
        out.push_str(&format!("let {id}_tick=f();\n"));
        render_actions(out, &f.pattern, actor);
        if f.repeats.is_none() {
            out.push_str(&format!("{id}_steps={id}_steps+1;\n"));
        }
        out.push_str(&format!("if f() == {id}_tick {{ wait(1); }}\n}}\n"));
    }
}

#[cfg(all(test, not(gso_browser)))]
mod tests {
    use super::*;
    use crate::{evaluate::Evaluator, program::Program};

    #[test]
    fn parameterized_cancels_replay_and_survive_checkpoints_without_party_cycles() {
        let m = Manifest::load("data/mavuika-c6-r5-225/manifest.json").unwrap();
        let catalog = Catalog::new(&m).unwrap();
        assert_eq!(catalog.cancels, vec![true, false, false, false]);
        assert!(
            catalog.forms[0]
                .iter()
                .any(|a| a.params.get("recast") == Some(&1))
        );
        let mut skill = command("skill", &mut Rng(1));
        skill.params.insert("hold".into(), 1);
        let mut finisher = command("charge", &mut Rng(1));
        finisher.params.insert("final".into(), 1);
        finisher.cancel = Some(Cancel {
            collision: false,
            travel: 6,
        });
        let rotation = Rotation {
            active: 0,
            advanced: true,
            opening: vec![],
            cycle: vec![Visit {
                actor: 0,
                entry: vec![skill],
                field: Some(Field {
                    frames: 600,
                    pattern: vec![finisher],
                    repeats: Some(6),
                }),
                period: 1,
                phase: 0,
                pickup: true,
            }],
        };
        rotation.validate().unwrap();
        let restored: Rotation =
            serde_json::from_str(&serde_json::to_string(&rotation).unwrap()).unwrap();
        assert_eq!(rotation, restored);
        for collision in [false, true] {
            let mut p = restored.clone();
            p.cycle[0].field.as_mut().unwrap().pattern[0]
                .cancel
                .as_mut()
                .unwrap()
                .collision = collision;
            let source = Evaluator::config(&m, &Program::Generated(p));
            let parsed = gse::gcsl::parse(&source).unwrap();
            assert!(gse::check::unsupported(&parsed).is_empty());
            let config = gse::profile::config(&parsed).unwrap();
            let mut sim = gse::sim::Sim::new(&config, 717, 0.0);
            gse::eval::run(&mut sim, &parsed.program).unwrap();
            assert_eq!(sim.f, 7200);
            assert!(
                sim.abils[0]
                    .iter()
                    .any(|(name, hits, _)| name.contains("(Final)") && *hits > 0)
            );
            assert_eq!(
                sim.abils[0]
                    .iter()
                    .any(|(name, hits, _)| *name == "Flamestrider Sprint" && *hits > 0),
                collision
            );
        }
        let mut visit = restored.cycle[0].clone();
        catalog.reassign(&mut visit, 2, &mut Rng(1));
        assert!(
            visit
                .entry
                .iter()
                .chain(visit.field.iter().flat_map(|f| &f.pattern))
                .all(|a| catalog.supports_command(2, a))
        );
    }

    #[test]
    fn generated_rotations_execute_full_fights_for_every_field_owner() {
        let m = crate::artifact::tests::fixed_party();
        let catalog = Catalog::new(&m).unwrap();
        let mut rng = Rng(37);
        for actor in 0..4 {
            let p = Rotation::generate(&catalog, &mut rng, actor);
            p.validate().unwrap();
            assert!(
                p.cycle
                    .iter()
                    .any(|v| v.actor == actor && v.field.is_some())
            );
            let source = Evaluator::config(&m, &Program::Generated(p));
            let parsed = gse::gcsl::parse(&source).unwrap();
            assert!(gse::check::unsupported(&parsed).is_empty());
            assert!(!source.contains("flins") && !source.contains("ineffa"));
            let fights = gse::api::run_iterations(&source, 150.0, 991, 0, 2).unwrap();
            assert!(
                fights
                    .iter()
                    .all(|s| s.frames == 27000 && s.dps.is_finite())
            );
        }
    }

    #[test]
    fn basic_grammar_ablation_cannot_acquire_extended_forms_during_polish() {
        let m = Manifest::load("data/mavuika-c6-r5-225/manifest.json").unwrap();
        let catalog = Catalog::configured(&m, false).unwrap();
        let mut rng = Rng(717);
        let mut p = Rotation::generate(&catalog, &mut rng, 0);
        for _ in 0..100 {
            p = p.mutate(&catalog, &mut rng).0;
            assert!(!p.advanced);
            p.validate().unwrap();
        }
        for (neighbor, _) in p.neighbors(&m) {
            assert!(!neighbor.advanced);
            neighbor.validate().unwrap();
            let source = neighbor.gcsl(&m);
            assert!(!source.contains("charge_cancel") && !source.contains("pick_up_crystallize"));
        }
    }

    #[test]
    fn structural_edits_reach_new_orders_owners_periods_and_openings() {
        let m = crate::artifact::tests::fixed_party();
        let catalog = Catalog::new(&m).unwrap();
        let mut rng = Rng(91);
        let start = Rotation::generate(&catalog, &mut rng, 0);
        let mut p = start.clone();
        let mut changed_ops = std::collections::BTreeSet::new();
        let mut structures = std::collections::BTreeSet::new();
        for _ in 0..400 {
            let (next, label) = p.mutate(&catalog, &mut rng);
            next.validate().unwrap();
            let source = Evaluator::config(&m, &Program::Generated(next.clone()));
            let parsed = gse::gcsl::parse(&source).unwrap();
            assert!(gse::check::unsupported(&parsed).is_empty(), "{label}");
            if p != next {
                changed_ops.insert(label);
            }
            structures.insert(next.structure());
            p = next;
        }
        for op in [
            "insert visit",
            "delete visit",
            "relocate visit",
            "duplicate visit",
            "split visit",
            "merge visits",
            "change field ownership",
            "change visit period",
            "change field length",
            "edit action pattern",
            "change opening",
            "compound order and field",
            "compound repeat structure",
        ] {
            assert!(
                changed_ops.contains(&format!("generated {op}")),
                "unreached operator: {op}"
            );
        }
        assert!(structures.len() > 100);
        let neighbors = start.neighbors(&m);
        assert!(!neighbors.is_empty() && neighbors.len() <= 256);
        assert_eq!(neighbors, start.neighbors(&m));
    }

    #[test]
    fn guarded_only_patterns_and_skipped_cycles_cannot_spin_forever() {
        let m = crate::artifact::tests::fixed_party();
        let p = Rotation {
            active: 0,
            advanced: false,
            opening: vec![],
            cycle: vec![Visit {
                actor: 0,
                entry: vec![Command {
                    action: "skill".into(),
                    count: 1,
                    ready: true,
                    params: BTreeMap::new(),
                    cancel: None,
                    delay: 0,
                    hold_before: 0,
                }],
                field: Some(Field {
                    frames: 30,
                    repeats: None,
                    pattern: vec![Command {
                        action: "skill".into(),
                        count: 1,
                        ready: true,
                        params: BTreeMap::new(),
                        cancel: None,
                        delay: 0,
                        hold_before: 0,
                    }],
                }),
                period: 4,
                phase: 3,
                pickup: false,
            }],
        };
        p.validate().unwrap();
        let source = Evaluator::config(&m, &Program::Generated(p));
        let fight = gse::api::run_iterations(&source, 150.0, 1, 0, 1).unwrap();
        assert_eq!(fight[0].frames, 27000);
    }
}
