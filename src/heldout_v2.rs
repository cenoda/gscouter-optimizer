//! Held-out recovery v2 (`docs/heldout-recovery-v2.md`).
//!
//! Character-agnostic additions to the stage-1 search:
//! - owner-abstracted templates with a longer block cap (G1);
//! - a narrower, declared leakage filter (G3);
//! - new operators (owner reassignment, action substitution, segment
//!   repetition, whole-unit edits) and compound proposals (G2b);
//! - static rejection of new assumption constructs and a strip transform (G4);
//! - unit deletion used by the second pass's minimisation (G5).
use crate::heldout::{
    BLOCK_BUILTINS, Block, Dictionary, canonical, close, field_char, normalize, parses, statements,
    tokens,
};
use crate::program::Rng;
use gse::gcsl::{Tok, Token};
use std::collections::{BTreeMap, BTreeSet};

/// Block token cap in v2 (stage 1: 40).
pub const V2_MAX_BLOCK_TOKENS: usize = 96;
/// Statement templates kept, by descending library count. Guarded-block
/// templates (`if`/`while`) are all kept (count >= 2): they are few and
/// would otherwise be cut by the very common one-statement templates.
pub const MAX_TEMPLATES: usize = 256;
/// G3: a block matching A is dropped only if fewer library rotations use it.
pub const LEAK_K: usize = 10;
/// Owner placeholder in template text.
pub const SELF: &str = "$SELF";
/// Engine-wide basic actions every member has.
const BASIC_ACTIONS: &[&str] = &["attack", "charge", "skill", "burst", "dash", "jump"];

/// Compound-proposal schedule of a pass.
#[derive(Clone, Copy, Debug)]
pub struct Moves {
    pub compound_percent: usize,
    pub compound_max: usize,
}
pub const PASS1: Moves = Moves {
    compound_percent: 30,
    compound_max: 3,
};
pub const PASS2: Moves = Moves {
    compound_percent: 50,
    compound_max: 4,
};

fn owner_name(k: u16) -> &'static str {
    gse::gcsl::char_name(k)
}

/// (owner, start, end, kind) of every extractable block: action statements
/// and `if`/`while` blocks that reference exactly one character and only
/// built-in calls (the stage-1 rule), with a token cap.
fn spans(t: &[Token], cap: usize) -> Vec<(u16, usize, usize, &'static str)> {
    let mut out = Vec::new();
    for s in statements(t) {
        if s.end - s.start < cap {
            let params = t[s.start..s.end]
                .iter()
                .any(|x| x.typ == Tok::LeftSquareParen);
            out.push((
                s.owner,
                s.start,
                s.end,
                if params {
                    "action statement with parameters"
                } else {
                    "action statement"
                },
            ));
        }
    }
    for i in 0..t.len() {
        if !matches!(t[i].typ, Tok::If | Tok::While) || (i > 0 && t[i - 1].typ == Tok::Else) {
            continue;
        }
        let Some(open) = (i + 1..t.len()).find(|&j| t[j].typ == Tok::LeftBrace) else {
            continue;
        };
        let Some(end) = close(t, open) else { continue };
        if (end + 1 < t.len() && t[end + 1].typ == Tok::Else) || end - i >= cap {
            continue;
        }
        let span = &t[i..=end];
        let mut owners = BTreeSet::new();
        let mut ok = true;
        // v2: `=` inside an action parameter list is a parameter, not an
        // assignment (stage 1 rejected every guarded block with parameters).
        let mut bracket = 0i32;
        for (k, x) in span.iter().enumerate() {
            match x.typ {
                Tok::LeftSquareParen => bracket += 1,
                Tok::RightSquareParen => bracket -= 1,
                Tok::Assign if bracket > 0 => {}
                Tok::CharacterKey => {
                    owners.insert(gse::gcsl::char_key(&x.val));
                }
                Tok::Field => {
                    if let Some(c) = field_char(&x.val) {
                        owners.insert(Some(c));
                    }
                }
                Tok::Identifier
                    if bracket > 0 && span.get(k + 1).is_some_and(|n| n.typ == Tok::Assign) => {}
                Tok::Identifier => {
                    let call = span.get(k + 1).is_some_and(|n| n.typ == Tok::LeftParen);
                    ok &= call && BLOCK_BUILTINS.contains(&x.val.as_str());
                }
                Tok::Break => ok &= t[i].typ == Tok::While,
                Tok::Let
                | Tok::Fn
                | Tok::Return
                | Tok::Continue
                | Tok::Switch
                | Tok::Case
                | Tok::Default
                | Tok::Fallthrough
                | Tok::For
                | Tok::Active
                | Tok::Options
                | Tok::Target
                | Tok::Energy
                | Tok::Hurt
                | Tok::Assign => ok = false,
                _ => {}
            }
        }
        if !ok || owners.len() != 1 || statements(span).is_empty() {
            continue;
        }
        if let Some(Some(owner)) = owners.into_iter().next() {
            out.push((
                owner,
                i,
                end,
                if t[i].typ == Tok::While {
                    "guarded while block"
                } else {
                    "guarded if block"
                },
            ));
        }
    }
    out
}

fn text_of(t: &[Token]) -> String {
    t.iter().map(canonical).collect::<Vec<_>>().join(" ")
}

/// Template text: the owner's name and field paths become the placeholder.
fn template_of(t: &[Token], owner: u16) -> String {
    t.iter()
        .map(|x| match x.typ {
            Tok::CharacterKey if gse::gcsl::char_key(&x.val) == Some(owner) => SELF.to_string(),
            Tok::Field if field_char(&x.val) == Some(owner) => format!(".{SELF}"),
            _ => canonical(x),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn instantiate(template: &str, member: u16) -> String {
    template.replace(SELF, owner_name(member))
}

/// Per-rotation (owner, text, kind) blocks and (template, kind) templates.
#[allow(clippy::type_complexity)]
pub fn extract(
    raw: &str,
) -> (
    BTreeSet<(String, String, String)>,
    BTreeSet<(String, String)>,
) {
    let t = tokens(raw);
    let mut blocks = BTreeSet::new();
    let mut templates = BTreeSet::new();
    for (owner, a, b, kind) in spans(&t, V2_MAX_BLOCK_TOKENS) {
        blocks.insert((
            owner_name(owner).to_string(),
            text_of(&t[a..=b]),
            kind.to_string(),
        ));
        templates.insert((template_of(&t[a..=b], owner), kind.to_string()));
    }
    (blocks, templates)
}

/// The v2 dictionary: stage-1 per-character blocks with the v2 cap plus the
/// most common owner-abstracted templates.
pub fn build(source: &str, rotations: &[(String, String)]) -> Dictionary {
    let mut counts: BTreeMap<(String, String, String), usize> = BTreeMap::new();
    let mut tcounts: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut unparsed = Vec::new();
    for (name, raw) in rotations {
        if gse::gcsl::lex(raw).iter().any(|t| t.typ == Tok::Error) {
            unparsed.push(name.clone());
            continue;
        }
        let (b, t) = extract(raw);
        for x in b {
            *counts.entry(x).or_default() += 1;
        }
        for x in t {
            *tcounts.entry(x).or_default() += 1;
        }
    }
    let mut blocks: BTreeMap<String, Vec<Block>> = BTreeMap::new();
    for ((owner, text, kind), count) in counts {
        if count >= crate::heldout::MIN_BLOCK_COUNT {
            blocks
                .entry(owner)
                .or_default()
                .push(Block { text, count, kind });
        }
    }
    for list in blocks.values_mut() {
        list.sort_by(|a, b| b.count.cmp(&a.count).then(a.text.cmp(&b.text)));
        list.truncate(crate::heldout::MAX_BLOCKS_PER_CHARACTER);
    }
    let mut templates: Vec<Block> = tcounts
        .into_iter()
        .filter(|(_, c)| *c >= crate::heldout::MIN_BLOCK_COUNT)
        .map(|((text, kind), count)| Block { text, count, kind })
        .collect();
    templates.sort_by(|a, b| b.count.cmp(&a.count).then(a.text.cmp(&b.text)));
    let mut statements_kept = 0;
    templates.retain(|b| {
        b.kind.starts_with("guarded") || {
            statements_kept += 1;
            statements_kept <= MAX_TEMPLATES
        }
    });
    Dictionary {
        format: 2,
        source: source.into(),
        rotations: rotations.len(),
        unparsed,
        blocks,
        dropped_for_leakage: Vec::new(),
        templates,
        param_keys: Vec::new(),
    }
}

/// G3 filter: drop a block or template only if it occurs in A (a template:
/// instantiated for any party member) and fewer than `LEAK_K` library
/// rotations contain it.
pub fn filtered(d: &Dictionary, withheld: &str, members: &[u16]) -> Dictionary {
    let haystack = format!(" {} ", normalize(withheld));
    let found = |s: &str| haystack.contains(&format!(" {s} "));
    let mut out = d.clone();
    out.dropped_for_leakage.clear();
    for list in out.blocks.values_mut() {
        list.retain(|b| {
            let leak = b.count < LEAK_K && found(&b.text);
            if leak {
                out.dropped_for_leakage.push(b.text.clone());
            }
            !leak
        });
    }
    out.templates.retain(|b| {
        let leak = b.count < LEAK_K && members.iter().any(|&m| found(&instantiate(&b.text, m)));
        if leak {
            out.dropped_for_leakage.push(b.text.clone());
        }
        !leak
    });
    out
}

/// Reproduce a per-case filtered dictionary from its committed drop list.
pub fn drop_listed(d: &Dictionary, dropped: &[String]) -> Dictionary {
    let set: BTreeSet<&str> = dropped.iter().map(String::as_str).collect();
    let mut out = d.clone();
    for list in out.blocks.values_mut() {
        list.retain(|b| !set.contains(b.text.as_str()));
    }
    out.templates.retain(|b| !set.contains(b.text.as_str()));
    out.dropped_for_leakage = dropped.to_vec();
    out
}

/// Templates derived from programs themselves (second pass: S1 and L), not
/// yet in the dictionary. Their count is set to the dictionary's largest so
/// sqrt weighting gives them the highest weight.
pub fn derive(d: &Dictionary, raws: &[&str]) -> Vec<Block> {
    let known: BTreeSet<&str> = d.templates.iter().map(|b| b.text.as_str()).collect();
    let top = d.templates.iter().map(|b| b.count).max().unwrap_or(1);
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for raw in raws {
        for (text, kind) in extract(raw).1 {
            if !known.contains(text.as_str()) && seen.insert(text.clone()) {
                out.push(Block {
                    text,
                    count: top,
                    kind: format!("derived {kind}"),
                });
            }
        }
    }
    out
}

/// Editable units: action statements and `if`/`while` blocks without an
/// `else` chain, excluding anything containing the repetition wrapper guard.
#[derive(Clone, Debug, PartialEq)]
pub struct Unit {
    pub start: usize,
    pub end: usize,
    pub owner: Option<u16>,
    pub statement: bool,
}

pub fn units(t: &[Token]) -> Vec<Unit> {
    let mut out: Vec<Unit> = statements(t)
        .into_iter()
        .map(|s| Unit {
            start: s.start,
            end: s.end,
            owner: Some(s.owner),
            statement: true,
        })
        .collect();
    for i in 0..t.len() {
        if !matches!(t[i].typ, Tok::If | Tok::While) || (i > 0 && t[i - 1].typ == Tok::Else) {
            continue;
        }
        let Some(open) = (i + 1..t.len()).find(|&j| t[j].typ == Tok::LeftBrace) else {
            continue;
        };
        let Some(end) = close(t, open) else { continue };
        if end + 1 < t.len() && t[end + 1].typ == Tok::Else {
            continue;
        }
        let span = &t[i..=end];
        if span
            .iter()
            .any(|x| x.val.starts_with("optimizer_cycle_start") || x.typ == Tok::Fn)
        {
            continue;
        }
        let owners: BTreeSet<Option<u16>> = span
            .iter()
            .filter_map(|x| match x.typ {
                Tok::CharacterKey => Some(gse::gcsl::char_key(&x.val)),
                Tok::Field => field_char(&x.val).map(Some),
                _ => None,
            })
            .collect();
        out.push(Unit {
            start: i,
            end,
            owner: if owners.len() == 1 {
                owners.into_iter().next().flatten()
            } else {
                None
            },
            statement: false,
        });
    }
    out.sort_by_key(|u| (u.start, u.end));
    out
}

/// Counts of the declared assumption constructs: `execute_action` overrides,
/// `pick_up_crystallize` calls and `counter` parameters.
pub fn assumptions(raw: &str) -> [usize; 3] {
    let t = tokens(raw);
    let mut c = [0; 3];
    for (i, x) in t.iter().enumerate() {
        if x.val == "execute_action" || x.val == "_execute_action" {
            c[0] += 1;
        }
        if x.val == "pick_up_crystallize" {
            c[1] += 1;
        }
        if x.val == "counter" && t.get(i + 1).is_some_and(|n| n.typ == Tok::Assign) {
            c[2] += 1;
        }
    }
    c
}

fn adds_assumptions(parent: &str, child: &str) -> bool {
    let (a, b) = (assumptions(parent), assumptions(child));
    (0..3).any(|i| b[i] > a[i])
}

/// Mechanical strip of the assumption constructs (for r_strip): delete
/// `fn execute_action ... { }`, `let X = execute_action;`,
/// `pick_up_crystallize(...);` and `counter=<value>` parameters.
pub fn strip(raw: &str) -> String {
    let t = tokens(raw);
    let mut keep = vec![true; t.len()];
    let mut i = 0;
    while i < t.len() {
        if t[i].typ == Tok::Fn
            && t.get(i + 1).is_some_and(|n| n.val == "execute_action")
            && let Some(open) = (i..t.len()).find(|&j| t[j].typ == Tok::LeftBrace)
            && let Some(end) = close(&t, open)
        {
            keep[i..=end].iter_mut().for_each(|k| *k = false);
            i = end + 1;
            continue;
        }
        if t[i].typ == Tok::Let
            && i + 4 < t.len()
            && t[i + 2].typ == Tok::Assign
            && t[i + 3].val == "execute_action"
            && t[i + 4].typ == Tok::TerminateLine
        {
            keep[i..=i + 4].iter_mut().for_each(|k| *k = false);
            i += 5;
            continue;
        }
        if t[i].val == "pick_up_crystallize"
            && t.get(i + 1).is_some_and(|n| n.typ == Tok::LeftParen)
            && let Some(end) = (i..t.len()).find(|&j| t[j].typ == Tok::TerminateLine)
        {
            keep[i..=end].iter_mut().for_each(|k| *k = false);
            i = end + 1;
            continue;
        }
        if t[i].val == "counter" && t.get(i + 1).is_some_and(|n| n.typ == Tok::Assign) {
            // Value runs to the next `,` or `]` at bracket depth 0.
            let mut j = i + 2;
            let mut depth = 0i32;
            while j < t.len() {
                match t[j].typ {
                    Tok::LeftParen | Tok::LeftSquareParen => depth += 1,
                    Tok::RightParen => depth -= 1,
                    Tok::RightSquareParen if depth > 0 => depth -= 1,
                    Tok::RightSquareParen | Tok::Comma if depth == 0 => break,
                    _ => {}
                }
                j += 1;
            }
            keep[i..j].iter_mut().for_each(|k| *k = false);
            // Remove one adjoining comma.
            if j < t.len() && t[j].typ == Tok::Comma {
                keep[j] = false;
            } else if i > 0 && t[i - 1].typ == Tok::Comma {
                keep[i - 1] = false;
            }
            i = j;
            continue;
        }
        i += 1;
    }
    let mut out: Vec<String> = Vec::new();
    for (k, x) in t.iter().enumerate() {
        if keep[k] {
            out.push(x.val.clone());
        }
    }
    // An emptied parameter list `[ ]` is removed.
    let mut tidy: Vec<String> = Vec::new();
    for x in out {
        if x == "]" && tidy.last().is_some_and(|l| l == "[") {
            tidy.pop();
        } else {
            tidy.push(x);
        }
    }
    tidy.join(" ")
}

/// Split an action list (tokens between owner and `;`) at top-level commas.
pub(crate) fn actions(t: &[Token], start: usize, end: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut a = start + 1;
    for (j, x) in t.iter().enumerate().take(end).skip(start + 1) {
        match x.typ {
            Tok::LeftSquareParen | Tok::LeftParen => depth += 1,
            Tok::RightSquareParen | Tok::RightParen => depth -= 1,
            Tok::Comma if depth == 0 => {
                out.push((a, j - 1));
                a = j + 1;
            }
            _ => {}
        }
    }
    if a < end {
        out.push((a, end - 1));
    }
    out
}

fn pick_weighted<'a>(blocks: &[&'a Block], rng: &mut Rng) -> &'a Block {
    let w: Vec<f64> = blocks.iter().map(|b| (b.count as f64).sqrt()).collect();
    let total: f64 = w.iter().sum();
    let mut x = rng.uniform() * total;
    for (b, w) in blocks.iter().zip(&w) {
        if x < *w {
            return b;
        }
        x -= w;
    }
    blocks[blocks.len() - 1]
}

/// "character action key" triples of every action parameter in a rotation.
pub fn param_triples(raw: &str) -> BTreeSet<String> {
    let t = tokens(raw);
    let mut out = BTreeSet::new();
    for s in statements(&t) {
        for (a, b) in actions(&t, s.start, s.end) {
            if t[a].typ != Tok::ActionKey {
                continue;
            }
            let mut depth = 0;
            for k in a + 1..=b {
                match t[k].typ {
                    Tok::LeftSquareParen => depth += 1,
                    Tok::RightSquareParen => depth -= 1,
                    Tok::Identifier
                        if depth == 1 && t.get(k + 1).is_some_and(|n| n.typ == Tok::Assign) =>
                    {
                        out.insert(format!("{} {} {}", owner_name(s.owner), t[a].val, t[k].val));
                    }
                    _ => {}
                }
            }
        }
    }
    out
}

/// v2.1: the library's parameter triples (built from the same rotations).
pub fn library_params(rotations: &[(String, String)]) -> Vec<String> {
    let mut all = BTreeSet::new();
    for (_, raw) in rotations {
        if !gse::gcsl::lex(raw).iter().any(|t| t.typ == Tok::Error) {
            all.extend(param_triples(raw));
        }
    }
    all.into_iter().collect()
}

/// The v2 macro proposer for one case.
pub struct Proposer<'a> {
    pub dict: &'a Dictionary,
    pub derived: &'a [Block],
    pub members: Vec<u16>,
    pub context: String,
}

impl Proposer<'_> {
    /// v2.1 (when the dictionary carries `param_keys`): every parameter
    /// triple of `child` is a library triple or already in `parent`.
    pub fn params_allowed(&self, parent: &str, child: &str) -> bool {
        if self.dict.param_keys.is_empty() {
            return true;
        }
        let old = param_triples(parent);
        param_triples(child)
            .iter()
            .all(|p| old.contains(p) || self.dict.param_keys.binary_search(p).is_ok())
    }

    fn vocabulary(&self, t: &[Token], owner: u16) -> Vec<String> {
        let mut v: BTreeSet<String> = BASIC_ACTIONS.iter().map(|s| s.to_string()).collect();
        for s in statements(t).iter().filter(|s| s.owner == owner) {
            for x in &t[s.start..s.end] {
                if x.typ == Tok::ActionKey {
                    v.insert(x.val.clone());
                }
            }
        }
        for b in self.dict.for_char(owner) {
            for x in tokens(&b.text) {
                if x.typ == Tok::ActionKey {
                    v.insert(x.val);
                }
            }
        }
        v.into_iter().collect()
    }

    /// One random edit; `None` when impossible, unchanged, unparseable or
    /// when it would add an assumption construct.
    pub fn edit(&self, raw: &str, rng: &mut Rng) -> Option<(String, String)> {
        let t = tokens(raw);
        let text: Vec<String> = t.iter().map(|x| x.val.clone()).collect();
        let us = units(&t);
        let st: Vec<&Unit> = us.iter().filter(|u| u.statement).collect();
        if st.is_empty() {
            return None;
        }
        let member = self.members[rng.index(self.members.len())];
        let mut changed = text.clone();
        let label = match rng.index(100) {
            0..20 => {
                let s = st[rng.index(st.len())];
                let blocks: Vec<&Block> = self.dict.for_char(s.owner?).iter().collect();
                if blocks.is_empty() {
                    return None;
                }
                let b = pick_weighted(&blocks, rng);
                changed.splice(s.start..=s.end, [b.text.clone()]);
                format!("dictionary replace statement <- {}", b.text)
            }
            20..36 => {
                let u = &us[rng.index(us.len())];
                let blocks: Vec<&Block> = self.dict.for_char(member).iter().collect();
                if blocks.is_empty() {
                    return None;
                }
                let b = pick_weighted(&blocks, rng);
                changed.insert(u.start, b.text.clone());
                format!("dictionary insert <- {}", b.text)
            }
            36..60 => {
                // Half the draws use guarded-block templates, half statements.
                let guarded = rng.index(2) == 0;
                let pool: Vec<&Block> = self
                    .dict
                    .templates
                    .iter()
                    .chain(self.derived)
                    .filter(|b| b.kind.contains("guarded") == guarded)
                    .collect();
                if pool.is_empty() {
                    return None;
                }
                let b = pick_weighted(&pool, rng);
                let inst = instantiate(&b.text, member);
                let u = &us[rng.index(us.len())];
                if rng.index(2) == 0 {
                    changed.splice(u.start..=u.end, [inst]);
                    format!("template replace for {} <- {}", owner_name(member), b.text)
                } else {
                    changed.insert(u.start, inst);
                    format!("template insert for {} <- {}", owner_name(member), b.text)
                }
            }
            60..68 => {
                let owned: Vec<&Unit> = us.iter().filter(|u| u.owner.is_some()).collect();
                if owned.is_empty() {
                    return None;
                }
                let u = owned[rng.index(owned.len())];
                let from = u.owner?;
                if from == member {
                    return None;
                }
                for k in u.start..=u.end {
                    match t[k].typ {
                        Tok::CharacterKey if gse::gcsl::char_key(&t[k].val) == Some(from) => {
                            changed[k] = owner_name(member).into()
                        }
                        Tok::Field if field_char(&t[k].val) == Some(from) => {
                            changed[k] = format!(".{}", owner_name(member))
                        }
                        _ => {}
                    }
                }
                format!(
                    "reassign owner {} -> {}",
                    owner_name(from),
                    owner_name(member)
                )
            }
            68..76 => {
                if st.len() < 2 {
                    return None;
                }
                let u = &us[rng.index(us.len())];
                changed.drain(u.start..=u.end);
                format!("delete unit at token {}", u.start)
            }
            76..82 => {
                let u = &us[rng.index(us.len())];
                let next = us.iter().find(|v| v.start > u.end)?;
                let moved: Vec<String> = text[u.start..=u.end].to_vec();
                changed.splice(next.end + 1..next.end + 1, moved);
                changed.drain(u.start..=u.end);
                "move unit past the next".into()
            }
            82..92 => {
                let s = st[rng.index(st.len())];
                let acts = actions(&t, s.start, s.end);
                let (a, b) = acts[rng.index(acts.len())];
                if t[a].typ != Tok::ActionKey {
                    return None;
                }
                let vocab = self.vocabulary(&t, s.owner?);
                let new = &vocab[rng.index(vocab.len())];
                if new == &t[a].val {
                    return None;
                }
                // Parameters belong to the old action; a repeat count is kept.
                let mut repl = vec![new.clone()];
                let mut k = a + 1;
                if k <= b && t[k].typ == Tok::LeftSquareParen {
                    while k <= b && t[k].typ != Tok::RightSquareParen {
                        k += 1;
                    }
                    k += 1;
                }
                repl.extend(text[k.min(b + 1)..=b].iter().cloned());
                changed.splice(a..=b, repl);
                format!("substitute action {} -> {new}", t[a].val)
            }
            _ => {
                let s = st[rng.index(st.len())];
                let acts = actions(&t, s.start, s.end);
                let width = 1 + rng.index(4.min(acts.len()));
                let i = rng.index(acts.len() + 1 - width);
                let (a, b) = (acts[i].0, acts[i + width - 1].1);
                if rng.index(2) == 0 {
                    let mut seg = vec![",".to_string()];
                    seg.extend(text[a..=b].iter().cloned());
                    changed.splice(b + 1..b + 1, seg);
                    format!("repeat segment of {width} actions")
                } else {
                    if width == acts.len() {
                        return None;
                    }
                    // Remove the segment and one adjoining comma.
                    let (from, to) = if i + width < acts.len() {
                        (a, acts[i + width].0 - 1)
                    } else {
                        (acts[i - 1].1 + 1, b)
                    };
                    changed.drain(from..=to);
                    format!("delete segment of {width} actions")
                }
            }
        };
        let source = changed.join(" ");
        if source == text.join(" ")
            || adds_assumptions(raw, &source)
            || !self.params_allowed(raw, &source)
            || !parses(&self.context, &source)
        {
            return None;
        }
        Some((source, label))
    }
}

/// Second-pass minimisation (G5): delete the unit at `cursor` of the
/// current program; `None` when the sweep is finished. At least one action
/// statement always remains.
pub fn delete_unit(raw: &str, cursor: usize, context: &str) -> Option<Option<String>> {
    let t = tokens(raw);
    let us = units(&t);
    let u = us.get(cursor)?;
    let text: Vec<String> = t.iter().map(|x| x.val.clone()).collect();
    let mut changed = text;
    changed.drain(u.start..=u.end);
    let source = changed.join(" ");
    let remaining = statements(&tokens(&source)).len();
    Some((remaining > 0 && parses(context, &source)).then_some(source))
}

/// Token-set Jaccard similarity of two programs' normalized text.
pub fn similarity(a: &str, b: &str) -> f64 {
    let sa: BTreeSet<String> = normalize(a).split(' ').map(str::to_string).collect();
    let sb: BTreeSet<String> = normalize(b).split(' ').map(str::to_string).collect();
    let i = sa.intersection(&sb).count() as f64;
    let u = sa.union(&sb).count() as f64;
    if u == 0.0 { 1.0 } else { i / u }
}

/// Micro neighbors with `counter` values frozen (G4).
pub fn micro(raw: &str, context: &str) -> Vec<(String, String)> {
    let t = tokens(raw);
    crate::heldout::numeric_neighbors(raw, context)
        .into_iter()
        .filter(|(_, label)| {
            let i: usize = label
                .split("token ")
                .nth(1)
                .and_then(|r| r.split(':').next())
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
            !(i >= 2 && t[i - 2].val == "counter")
        })
        .collect()
}

fn arg(args: &[String], name: &str) -> Result<String, String> {
    args.windows(2)
        .find(|x| x[0] == name)
        .map(|x| x[1].clone())
        .ok_or_else(|| format!("{name} required"))
}

fn read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
}

fn load(path: &str) -> Result<Dictionary, String> {
    serde_json::from_str(&read(path)?).map_err(|e| e.to_string())
}

fn write(path: &str, value: &impl serde::Serialize) -> Result<(), String> {
    let p = std::path::Path::new(path);
    if let Some(parent) = p.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    crate::evaluate::atomic(p, value)
}

/// `dictionary-v2 build|filter|drop` and `strip-assumptions`.
pub fn cli(args: &[String]) -> Result<(), String> {
    match (args[0].as_str(), args.get(1).map(String::as_str)) {
        ("dictionary-v2", Some("build")) => {
            let mut rotations = Vec::new();
            for line in read(&arg(args, "--list")?)?.lines() {
                let (name, path) = line.split_once('\t').ok_or("list: NAME<TAB>PATH")?;
                rotations.push((name.to_string(), read(path)?));
            }
            let mut d = build(&arg(args, "--source")?, &rotations);
            if args.iter().any(|a| a == "--param-keys") {
                d.param_keys = library_params(&rotations);
            }
            write(&arg(args, "--out")?, &d)
        }
        ("dictionary-v2", Some("filter")) => {
            let d = load(&arg(args, "--dictionary")?)?;
            let m = crate::artifact::Manifest::load(&arg(args, "--manifest")?)?;
            let members: Vec<u16> = m
                .members
                .iter()
                .filter_map(|x| gse::gcsl::char_key(&x.key))
                .collect();
            let f = filtered(&d, &read(&arg(args, "--withheld")?)?, &members);
            println!(
                "{}",
                serde_json::json!({"dropped":f.dropped_for_leakage.len(),
                    "kept_blocks":f.blocks.values().map(Vec::len).sum::<usize>(),"kept_templates":f.templates.len(),"leak_k":LEAK_K})
            );
            write(&arg(args, "--out")?, &f)
        }
        ("dictionary-v2", Some("drop")) => {
            // Applies to format-1 and format-2 dictionaries alike.
            let d = load(&arg(args, "--dictionary")?)?;
            let drops: serde_json::Value =
                serde_json::from_str(&read(&arg(args, "--drops")?)?).map_err(|e| e.to_string())?;
            let list: Vec<String> = serde_json::from_value(drops["dropped_for_leakage"].clone())
                .map_err(|e| e.to_string())?;
            write(&arg(args, "--out")?, &drop_listed(&d, &list))
        }
        ("config", _) => {
            // Full engine/gcsim config of a rotation on a manifest (parity checks).
            let m = crate::artifact::Manifest::load(&arg(args, "--manifest")?)?;
            let p = crate::program::Program::baseline(&read(&arg(args, "--rotation")?)?, &m)?;
            std::fs::write(
                arg(args, "--out")?,
                crate::evaluate::Evaluator::config(&m, &p),
            )
            .map_err(|e| e.to_string())
        }
        ("param-check", _) => {
            // Parameter triples of a rotation that the library never uses.
            let d = load(&arg(args, "--dictionary")?)?;
            let unseen: Vec<String> = param_triples(&read(&arg(args, "--rotation")?)?)
                .into_iter()
                .filter(|p| d.param_keys.binary_search(p).is_err())
                .collect();
            println!(
                "{}",
                serde_json::to_string(&unseen).map_err(|e| e.to_string())?
            );
            Ok(())
        }
        ("strip-assumptions", _) => {
            let raw = read(&arg(args, "--rotation")?)?;
            std::fs::write(arg(args, "--out")?, strip(&raw)).map_err(|e| e.to_string())
        }
        _ => {
            Err("dictionary-v2 build|filter|drop | strip-assumptions | config | param-check".into())
        }
    }
}

#[cfg(all(test, not(gso_browser)))]
mod tests {
    use super::*;

    const LOOP: &str = "active xianyun; while 1 { xianyun skill; while .chongyun.status.xianyun-airborne-buff { chongyun high_plunge; } bennett skill[hold=1], attack:2, burst; }";

    #[test]
    fn templates_abstract_the_owner_and_instantiate_for_any_member() {
        let (blocks, templates) = extract(LOOP);
        assert!(
            blocks
                .iter()
                .any(|b| b.0 == "chongyun" && b.2 == "guarded while block")
        );
        let t = templates
            .iter()
            .find(|t| t.1 == "guarded while block")
            .unwrap();
        assert!(t.0.contains(".$SELF") && !t.0.contains("chongyun"));
        let k = gse::gcsl::char_key("escoffier").unwrap();
        assert!(instantiate(&t.0, k).contains(".escoffier"));
        // Long blocks are kept up to the v2 cap.
        let long = format!(
            "active bennett; while .bennett.burst.ready {{ {} }}",
            "bennett attack; ".repeat(20)
        );
        assert!(
            extract(&long)
                .0
                .iter()
                .any(|b| b.2 == "guarded while block")
        );
        assert!(
            !crate::heldout::extract(&long)
                .iter()
                .any(|b| b.2 == "guarded while block")
        );
    }

    #[test]
    fn filter_keeps_common_text_and_drop_lists_reproduce_it() {
        let common = "active bennett; bennett skill, attack;";
        let rare = "active bennett; bennett skill, attack; bennett burst, dash;";
        let mut rotations: Vec<(String, String)> = (0..12)
            .map(|i| (format!("c{i}"), common.to_string()))
            .collect();
        rotations.push(("r1".into(), rare.into()));
        rotations.push(("r2".into(), rare.into()));
        let d = build("test", &rotations);
        let m = [gse::gcsl::char_key("bennett").unwrap()];
        let f = filtered(&d, rare, &m);
        assert!(
            f.dropped_for_leakage
                .iter()
                .any(|t| t.contains("burst , dash"))
        );
        assert!(
            !f.dropped_for_leakage
                .iter()
                .any(|t| t == "bennett skill , attack ;")
        );
        assert_eq!(drop_listed(&d, &f.dropped_for_leakage), f);
        // Format-1 dictionaries serialize without the template field.
        let v1 = crate::heldout::Dictionary::build("test", &rotations);
        assert!(!serde_json::to_string(&v1).unwrap().contains("templates"));
    }

    #[test]
    fn strip_and_assumption_counts() {
        let raw = "let _execute_action = execute_action; fn execute_action(c number, a number, p map) { pick_up_crystallize(\"any\"); return _execute_action(c, a, p); } active beidou; beidou skill[counter=2]; beidou skill[hold=1, counter=1]; pick_up_crystallize(\"any\"); beidou burst;";
        assert_eq!(assumptions(raw), [4, 2, 2]);
        let s = strip(raw);
        assert_eq!(assumptions(&s), [0, 0, 0]);
        assert!(s.contains("beidou skill ;") && s.contains("skill [ hold = 1 ]"));
    }

    #[test]
    fn proposer_edits_parse_never_add_assumptions_and_minimise() {
        let m = crate::artifact::tests::fixed_party();
        let p = crate::program::Program::baseline(
            "active albedo; for let i=0; i<4; i=i+1 { albedo skill, attack:3; bennett burst, skill; if .bennett.skill.ready { bennett skill; } xiangling skill[hold=1]; noelle attack:4; }",
            &m,
        )
        .unwrap();
        let raw = p.source_text().unwrap().to_string();
        let rotations = vec![
            ("a".to_string(), LOOP.to_string()),
            ("b".into(), LOOP.into()),
            (
                "c".into(),
                "active bennett; bennett skill, attack; pick_up_crystallize(\"any\");".into(),
            ),
            (
                "d".into(),
                "active bennett; bennett skill, attack; pick_up_crystallize(\"any\");".into(),
            ),
        ];
        let d = build("test", &rotations);
        assert!(!d.templates.is_empty());
        let members: Vec<u16> = m
            .members
            .iter()
            .map(|x| gse::gcsl::char_key(&x.key).unwrap())
            .collect();
        let derived = derive(&d, &[&raw]);
        let prop = Proposer {
            dict: &d,
            derived: &derived,
            members,
            context: m.build(),
        };
        let mut rng = Rng(5);
        let mut kinds = BTreeSet::new();
        for _ in 0..2000 {
            if let Some((s, label)) = prop.edit(&raw, &mut rng) {
                assert!(s.contains("optimizer_cycle_start"));
                assert_eq!(assumptions(&s), assumptions(&raw));
                gse::gcsl::parse(&crate::evaluate::Evaluator::config(
                    &m,
                    &crate::program::Program::Source(s),
                ))
                .unwrap();
                kinds.insert(label.split(' ').take(2).collect::<Vec<_>>().join(" "));
            }
        }
        for k in [
            "dictionary replace",
            "dictionary insert",
            "template replace",
            "template insert",
            "reassign owner",
            "delete unit",
            "move unit",
            "substitute action",
            "repeat segment",
            "delete segment",
        ] {
            assert!(kinds.contains(k), "{k} missing from {kinds:?}");
        }
        // Minimisation walks every unit and keeps the wrapper.
        let n = units(&tokens(&raw)).len();
        assert!(n >= 6);
        for c in 0..n {
            if let Some(Some(s)) = delete_unit(&raw, c, &m.build()) {
                assert!(s.contains("optimizer_cycle_start"));
            }
        }
        assert!(delete_unit(&raw, n, &m.build()).is_none());
        assert!(similarity(&raw, &raw) == 1.0);
        // v2.1: parameter triples outside the library are rejected.
        let mut d21 = d.clone();
        d21.param_keys = library_params(&rotations);
        assert!(d21.param_keys.contains(&"bennett skill hold".to_string()));
        let p21 = Proposer {
            dict: &d21,
            derived: &derived,
            members: prop.members.clone(),
            context: m.build(),
        };
        assert!(p21.params_allowed(&raw, &raw));
        assert!(!p21.params_allowed(
            &raw,
            &raw.replace(
                "xiangling skill [ hold = 1 ]",
                "xiangling skill [ travel = 1 ]"
            )
        ));
        for _ in 0..500 {
            if let Some((s, _)) = p21.edit(&raw, &mut rng) {
                let base = param_triples(&raw);
                assert!(
                    param_triples(&s)
                        .iter()
                        .all(|t| base.contains(t) || d21.param_keys.contains(t))
                );
            }
        }
        let counter = "active bennett; while 1 { bennett skill[counter=1], attack:3; }";
        assert!(
            micro(counter, &m.build())
                .iter()
                .all(|(s, _)| s.contains("counter = 1"))
        );
    }
}
