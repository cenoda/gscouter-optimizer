//! Held-out recovery pilot support (`docs/heldout-recovery-pilot.md`).
//!
//! Three mechanical pieces, none of them character-specific:
//! - fixed-build manifests built from a library `build.txt`;
//! - a block dictionary extracted from the main library rotations, keyed by
//!   character, with a per-case leakage filter;
//! - macro statement edits using those blocks and a deterministic micro
//!   neighborhood over numeric parameters already present in a rotation.
use crate::artifact::{ENGINE, HEADER, Manifest, Member};
use crate::program::Rng;
use gse::gcsl::Tok;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Blocks longer than this many tokens are not "small" and are not kept.
pub const MAX_BLOCK_TOKENS: usize = 40;
/// A block must appear in at least this many distinct library rotations.
pub const MIN_BLOCK_COUNT: usize = 2;
/// At most this many blocks per character, by descending library count.
pub const MAX_BLOCKS_PER_CHARACTER: usize = 64;
/// Built-in calls a guarded dictionary block may contain. Any other
/// identifier (a rotation's own variable or function) is context-dependent.
pub(crate) const BLOCK_BUILTINS: &[&str] =
    &["f", "rand", "randnorm", "wait", "sleep", "delay", "is_even"];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub text: String,
    pub count: usize,
    pub kind: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Dictionary {
    pub format: u8,
    pub source: String,
    pub rotations: usize,
    pub unparsed: Vec<String>,
    /// Canonical engine character name -> blocks.
    pub blocks: BTreeMap<String, Vec<Block>>,
    #[serde(default)]
    pub dropped_for_leakage: Vec<String>,
    /// v2 only: owner-abstracted templates (`$SELF` placeholder). Omitted
    /// when empty so format-1 files serialize exactly as in stage 1.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub templates: Vec<Block>,
    /// v2.1 only: every "character action key" parameter triple used in the
    /// library rotations. Proposals may not introduce other triples.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_keys: Vec<String>,
}

pub(crate) fn tokens(raw: &str) -> Vec<gse::gcsl::Token> {
    let mut t = gse::gcsl::lex(raw);
    t.retain(|t| t.typ != Tok::Eof);
    t
}

/// The character a field path element names, if any (`.kuki` -> kuki).
pub(crate) fn field_char(val: &str) -> Option<u16> {
    gse::gcsl::char_key(val.strip_prefix('.')?)
}

/// Token text with character names replaced by the engine's canonical name.
pub(crate) fn canonical(t: &gse::gcsl::Token) -> String {
    match t.typ {
        Tok::CharacterKey => gse::gcsl::char_key(&t.val)
            .map(|k| gse::gcsl::char_name(k).to_string())
            .unwrap_or_else(|| t.val.clone()),
        Tok::Field => field_char(&t.val)
            .map(|k| format!(".{}", gse::gcsl::char_name(k)))
            .unwrap_or_else(|| t.val.clone()),
        _ => t.val.clone(),
    }
}

/// Normalized text: canonical tokens joined by single spaces. Comments and
/// layout disappear; this is the text used for the leakage filter.
pub fn normalize(raw: &str) -> String {
    tokens(raw)
        .iter()
        .map(canonical)
        .collect::<Vec<_>>()
        .join(" ")
}

/// A character action statement: `<char> <action list> ;`, at any depth.
#[derive(Clone, Debug, PartialEq)]
pub struct Statement {
    pub start: usize,
    pub end: usize,
    pub owner: u16,
}

pub fn statements(t: &[gse::gcsl::Token]) -> Vec<Statement> {
    let mut out = Vec::new();
    for i in 0..t.len().saturating_sub(1) {
        if t[i].typ != Tok::CharacterKey || t[i + 1].typ != Tok::ActionKey {
            continue;
        }
        let Some(end) = (i + 1..t.len()).find(|&j| {
            matches!(
                t[j].typ,
                Tok::TerminateLine | Tok::LeftBrace | Tok::RightBrace
            )
        }) else {
            continue;
        };
        if t[end].typ != Tok::TerminateLine {
            continue;
        }
        if let Some(owner) = gse::gcsl::char_key(&t[i].val) {
            out.push(Statement {
                start: i,
                end,
                owner,
            });
        }
    }
    out
}

/// Matching close brace of the open brace at `open`.
pub(crate) fn close(t: &[gse::gcsl::Token], open: usize) -> Option<usize> {
    let mut depth = 0;
    for (j, tok) in t.iter().enumerate().skip(open) {
        match tok.typ {
            Tok::LeftBrace => depth += 1,
            Tok::RightBrace => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
    }
    None
}

/// Mechanically extract (owner, normalized text, kind) blocks from one
/// rotation: every action statement, and every small `if`/`while` block whose
/// condition and body reference only one character and only built-in calls.
pub fn extract(raw: &str) -> BTreeSet<(String, String, String)> {
    let t = tokens(raw);
    let mut out = BTreeSet::new();
    let text = |a: usize, b: usize| t[a..=b].iter().map(canonical).collect::<Vec<_>>().join(" ");
    for s in statements(&t) {
        if s.end - s.start + 1 > MAX_BLOCK_TOKENS {
            continue;
        }
        let params = t[s.start..s.end]
            .iter()
            .any(|x| x.typ == Tok::LeftSquareParen);
        out.insert((
            gse::gcsl::char_name(s.owner).to_string(),
            text(s.start, s.end),
            if params {
                "action statement with parameters"
            } else {
                "action statement"
            }
            .into(),
        ));
    }
    for i in 0..t.len() {
        if !matches!(t[i].typ, Tok::If | Tok::While) {
            continue;
        }
        // A nested `else if` is part of a larger chain; skip it.
        if i > 0 && t[i - 1].typ == Tok::Else {
            continue;
        }
        let Some(open) = (i + 1..t.len()).find(|&j| t[j].typ == Tok::LeftBrace) else {
            continue;
        };
        let Some(end) = close(&t, open) else {
            continue;
        };
        if end + 1 < t.len() && t[end + 1].typ == Tok::Else {
            continue;
        }
        if end - i + 1 > MAX_BLOCK_TOKENS {
            continue;
        }
        let span = &t[i..=end];
        let mut owners = BTreeSet::new();
        let mut ok = true;
        for (k, x) in span.iter().enumerate() {
            match x.typ {
                Tok::CharacterKey => {
                    owners.insert(gse::gcsl::char_key(&x.val));
                }
                Tok::Field => {
                    if let Some(c) = field_char(&x.val) {
                        owners.insert(Some(c));
                    }
                }
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
        let has_action = !statements(span).is_empty();
        if !ok || owners.len() != 1 || !has_action {
            continue;
        }
        let Some(Some(owner)) = owners.into_iter().next() else {
            continue;
        };
        out.insert((
            gse::gcsl::char_name(owner).to_string(),
            text(i, end),
            if t[i].typ == Tok::While {
                "guarded while block"
            } else {
                "guarded if block"
            }
            .into(),
        ));
    }
    out
}

impl Dictionary {
    /// Count, for each block, how many library rotations contain it; keep
    /// the most common blocks per character. Unparseable rotations are listed.
    pub fn build(source: &str, rotations: &[(String, String)]) -> Self {
        let mut counts: BTreeMap<(String, String, String), usize> = BTreeMap::new();
        let mut unparsed = Vec::new();
        for (name, raw) in rotations {
            let lexed = gse::gcsl::lex(raw);
            if lexed.iter().any(|t| t.typ == Tok::Error) {
                unparsed.push(name.clone());
                continue;
            }
            for block in extract(raw) {
                *counts.entry(block).or_default() += 1;
            }
        }
        let mut blocks: BTreeMap<String, Vec<Block>> = BTreeMap::new();
        for ((owner, text, kind), count) in counts {
            if count >= MIN_BLOCK_COUNT {
                blocks
                    .entry(owner)
                    .or_default()
                    .push(Block { text, count, kind });
            }
        }
        for list in blocks.values_mut() {
            list.sort_by(|a, b| b.count.cmp(&a.count).then(a.text.cmp(&b.text)));
            list.truncate(MAX_BLOCKS_PER_CHARACTER);
        }
        Self {
            format: 1,
            source: source.into(),
            rotations: rotations.len(),
            unparsed,
            blocks,
            dropped_for_leakage: Vec::new(),
            templates: Vec::new(),
            param_keys: Vec::new(),
        }
    }

    /// Drop every block whose normalized text appears in the withheld
    /// rotation's normalized text (token boundaries), for this case only.
    pub fn filtered(&self, withheld: &str) -> Self {
        let haystack = format!(" {} ", normalize(withheld));
        let mut out = self.clone();
        out.dropped_for_leakage.clear();
        for list in out.blocks.values_mut() {
            list.retain(|b| {
                let leak = haystack.contains(&format!(" {} ", b.text));
                if leak {
                    out.dropped_for_leakage.push(b.text.clone());
                }
                !leak
            });
        }
        out
    }

    pub(crate) fn for_char(&self, id: u16) -> &[Block] {
        self.blocks
            .get(gse::gcsl::char_name(id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

fn pick<'a>(blocks: &'a [Block], rng: &mut Rng) -> &'a Block {
    let total: usize = blocks.iter().map(|b| b.count).sum();
    let mut x = rng.index(total);
    for b in blocks {
        if x < b.count {
            return b;
        }
        x -= b.count;
    }
    unreachable!()
}

pub(crate) fn parses(context: &str, source: &str) -> bool {
    gse::gcsl::parse(&format!("{HEADER}{context}\n{source}")).is_ok()
}

/// One random statement edit: replace or insert a dictionary block for a
/// party member, delete a statement, or move one statement past the next.
/// Returns `None` when the drawn edit is impossible or does not parse.
pub fn macro_edit(
    raw: &str,
    dict: &Dictionary,
    members: &[u16],
    context: &str,
    rng: &mut Rng,
) -> Option<(String, String)> {
    let t = tokens(raw);
    let text: Vec<String> = t.iter().map(|x| x.val.clone()).collect();
    let st = statements(&t);
    if st.is_empty() {
        return None;
    }
    let si = rng.index(st.len());
    let s = &st[si];
    let mut changed = text.clone();
    let label = match rng.index(100) {
        0..40 => {
            let blocks = dict.for_char(s.owner);
            if blocks.is_empty() {
                return None;
            }
            let b = pick(blocks, rng);
            changed.splice(s.start..=s.end, [b.text.clone()]);
            format!("dictionary replace statement {si} <- {}", b.text)
        }
        40..75 => {
            let owner = members[rng.index(members.len())];
            let blocks = dict.for_char(owner);
            if blocks.is_empty() {
                return None;
            }
            let b = pick(blocks, rng);
            changed.insert(s.start, b.text.clone());
            format!("dictionary insert before statement {si} <- {}", b.text)
        }
        75..88 => {
            if st.len() < 2 {
                return None;
            }
            changed.drain(s.start..=s.end);
            format!("delete statement {si}")
        }
        _ => {
            let next = st.get(si + 1)?;
            let moved: Vec<String> = text[s.start..=s.end].to_vec();
            changed.splice(next.end + 1..next.end + 1, moved);
            changed.drain(s.start..=s.end);
            format!("move statement {si} after {}", si + 1)
        }
    };
    let source = changed.join(" ");
    if source == text.join(" ") || !parses(context, &source) {
        return None;
    }
    Some((source, label))
}

/// Deterministic coordinate neighborhood over numeric literals already in the
/// rotation: action repeat counts (`attack:3`), action parameters
/// (`skill[hold=1]`) and `wait`/`delay` arguments. The repetition wrapper's
/// own `wait(1)` guard is not a coordinate.
pub fn numeric_neighbors(raw: &str, context: &str) -> Vec<(String, String)> {
    let t = tokens(raw);
    let text: Vec<String> = t.iter().map(|x| x.val.clone()).collect();
    let guard: BTreeSet<usize> = (0..t.len())
        .filter(|&i| t[i].typ == Tok::Identifier && t[i].val.starts_with("optimizer_cycle_start"))
        .filter(|&i| i > 0 && t[i - 1].typ == Tok::OpEqual)
        .flat_map(|i| i..(i + 8).min(t.len()))
        .collect();
    let mut result = Vec::new();
    let mut seen = BTreeSet::from([text.join(" ")]);
    for i in 0..t.len() {
        if t[i].typ != Tok::Number || guard.contains(&i) {
            continue;
        }
        let Ok(v) = t[i].val.parse::<i64>() else {
            continue;
        };
        let kind = if i >= 2 && t[i - 1].typ == Tok::Colon && t[i - 2].typ == Tok::ActionKey {
            "repeat count"
        } else if i >= 2
            && t[i - 1].typ == Tok::Assign
            && (i.saturating_sub(8)..i).any(|j| t[j].typ == Tok::LeftSquareParen)
            && (i.saturating_sub(8)..i)
                .rev()
                .find(|&j| t[j].typ == Tok::LeftSquareParen)
                .is_some_and(|j| j > 0 && t[j - 1].typ == Tok::ActionKey)
        {
            "action parameter"
        } else if i >= 2
            && t[i - 1].typ == Tok::LeftParen
            && t[i - 2].typ == Tok::Identifier
            && ["wait", "delay", "sleep"].contains(&t[i - 2].val.as_str())
        {
            "wait/delay"
        } else {
            continue;
        };
        let minimum = if kind == "repeat count" { 1 } else { 0 };
        let mut values = vec![v + 1, v - 1];
        if v >= 4 {
            values.push((v as f64 * 1.25).round() as i64);
            values.push((v as f64 * 0.75).round() as i64);
        }
        for n in values {
            if n < minimum || n == v {
                continue;
            }
            let mut changed = text.clone();
            changed[i] = n.to_string();
            let source = changed.join(" ");
            if seen.insert(source.clone()) && parses(context, &source) {
                result.push((source, format!("micro {kind} token {i}: {v}->{n}")));
                if result.len() == 256 {
                    return result;
                }
            }
        }
    }
    result
}

/// A format-2 fixed-build manifest from a complete build declaration.
pub fn manifest_from_build(
    build: &str,
    site_commit: &str,
    sources: BTreeMap<String, String>,
) -> Result<Manifest, String> {
    let first = tokens(build)
        .into_iter()
        .find(|t| t.typ == Tok::CharacterKey)
        .ok_or("build declares no character")?;
    let parsed = gse::gcsl::parse(&format!("{HEADER}{build}\nactive {};", first.val))?;
    let m = Manifest {
        format: 2,
        engine: ENGINE.into(),
        site_commit: site_commit.into(),
        sources,
        max_rolls: BTreeMap::new(),
        fixed_build: Some(build.into()),
        scoring: None,
        members: parsed
            .chars
            .iter()
            .map(|c| Member {
                key: gse::gcsl::char_name(c.key).into(),
                cons: c.cons as u8,
                weapon: gse::gcsl::weapon_name(c.weapon.key).into(),
                refinement: c.weapon.refine as u8,
                set: String::new(),
                useful: vec![],
                weights: BTreeMap::new(),
                pieces: vec![],
            })
            .collect(),
    };
    m.validate(&m)?;
    Ok(m)
}

#[cfg(not(gso_browser))]
fn arg(args: &[String], name: &str) -> Result<String, String> {
    args.windows(2)
        .find(|x| x[0] == name)
        .map(|x| x[1].clone())
        .ok_or_else(|| format!("{name} required"))
}

#[cfg(not(gso_browser))]
fn read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
}

#[cfg(not(gso_browser))]
fn write(path: &str, value: &impl Serialize) -> Result<(), String> {
    let p = std::path::Path::new(path);
    if let Some(parent) = p.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    crate::evaluate::atomic(p, value)
}

/// `heldout-manifest`, `dictionary build|filter` and `measure`.
#[cfg(not(gso_browser))]
pub fn cli(args: &[String]) -> Result<(), String> {
    use crate::evaluate::{Evaluator, cpu, hash, stats};
    match args[0].as_str() {
        "heldout-manifest" => {
            let build = read(&arg(args, "--build")?)?;
            let mut sources = BTreeMap::new();
            for w in args.windows(2).filter(|w| w[0] == "--source") {
                let (k, v) = w[1].split_once('=').ok_or("--source PATH=SHA256")?;
                sources.insert(k.to_string(), v.to_string());
            }
            let m = manifest_from_build(&build, &arg(args, "--site-commit")?, sources)?;
            write(&arg(args, "--out")?, &m)
        }
        "dictionary" => match args.get(1).map(String::as_str) {
            Some("build") => {
                // Each line: a rotation name, a tab, its path.
                let mut rotations = Vec::new();
                for line in read(&arg(args, "--list")?)?.lines() {
                    let (name, path) = line.split_once('\t').ok_or("list: NAME<TAB>PATH")?;
                    rotations.push((name.to_string(), read(path)?));
                }
                let d = Dictionary::build(&arg(args, "--source")?, &rotations);
                write(&arg(args, "--out")?, &d)
            }
            Some("filter") => {
                let d: Dictionary = serde_json::from_str(&read(&arg(args, "--dictionary")?)?)
                    .map_err(|e| e.to_string())?;
                let f = d.filtered(&read(&arg(args, "--withheld")?)?);
                println!(
                    "{}",
                    serde_json::json!({"dropped":f.dropped_for_leakage.len(),
                        "kept":f.blocks.values().map(Vec::len).sum::<usize>()})
                );
                write(&arg(args, "--out")?, &f)
            }
            _ => Err("dictionary build|filter".into()),
        },
        "measure" => {
            let m = Manifest::load(&arg(args, "--manifest")?)?;
            let raw = read(&arg(args, "--rotation")?)?;
            let seed: i64 = arg(args, "--seed")?.parse().map_err(|_| "invalid --seed")?;
            let n: usize = arg(args, "--samples")?
                .parse()
                .map_err(|_| "invalid --samples")?;
            let start = cpu();
            let program = crate::program::Program::baseline(&raw, &m)?;
            let config = Evaluator::config(&m, &program);
            let parsed = gse::gcsl::parse(&config)?;
            let unsupported = gse::check::unsupported(&parsed);
            if !unsupported.is_empty() {
                return Err(format!("unsupported: {}", unsupported.join("; ")));
            }
            let mut e = Evaluator::new(std::path::Path::new(&arg(args, "--out")?))?;
            // A reused directory loads earlier journal CPU; report only this call's.
            let carried = e.worker_cpu;
            let samples = e.eval(&m, &program, seed, n)?;
            println!(
                "{}",
                serde_json::json!({"key":Evaluator::key(&config),"config_sha256":hash(&config),
                    "seed":seed,"samples":n,"stats":stats(&samples),
                    "per_fight_dps":samples.iter().map(|s|s.dps).collect::<Vec<_>>(),
                    "cpu_seconds":e.worker_cpu-carried+cpu()-start,"engine":ENGINE})
            );
            Ok(())
        }
        _ => Err("unknown held-out command".into()),
    }
}

#[cfg(all(test, not(gso_browser)))]
mod tests {
    use super::*;

    #[test]
    fn extraction_is_mechanical_and_filter_is_per_case() {
        let raw = "active kuki; for let i=0; i<4; i=i+1 {
            if .kuki.skill.ready { kuki skill; }
            if .bennett.burst.ready { kuki skill; }
            if i == 0 { kuki skill; }
            nahida skill[hold=1], burst;
            while !.nahida.burst.ready { nahida attack; }
        }";
        let blocks = extract(raw);
        let texts: Vec<_> = blocks.iter().map(|b| b.1.as_str()).collect();
        // Names are canonical engine keys, so aliases share one entry.
        assert!(texts.contains(&"kukishinobu skill ;"));
        assert!(texts.contains(&"if .kukishinobu .skill .ready { kukishinobu skill ; }"));
        // A condition on another character, or on a rotation variable, is
        // context-dependent and is not a block.
        assert!(!texts.iter().any(|t| t.contains(".bennett")));
        assert!(!texts.iter().any(|t| t.starts_with("if i ")));
        assert!(texts.iter().any(|t| t.starts_with("while ")));
        let rotations = vec![("a".to_string(), raw.to_string()), ("b".into(), raw.into())];
        let d = Dictionary::build("test", &rotations);
        assert!(d.blocks.values().flatten().all(|b| b.count == 2));
        let f = d.filtered("active nahida; nahida skill[hold=1], burst;");
        assert!(f.dropped_for_leakage.iter().any(|t| t.contains("hold")));
        assert!(!f.blocks.values().flatten().any(|b| b.text.contains("hold")));
        // The unfiltered dictionary is unchanged.
        assert!(d.blocks.values().flatten().any(|b| b.text.contains("hold")));
    }

    #[test]
    fn macro_and_micro_edits_parse_and_skip_the_wrapper_guard() {
        let m = crate::artifact::tests::fixed_party();
        let p = crate::program::Program::baseline(
            "active albedo; for let i=0; i<4; i=i+1 { albedo skill, attack:3; bennett burst, skill; wait(10); xiangling skill[hold=1]; noelle attack:4; }",
            &m,
        )
        .unwrap();
        let crate::program::Program::Source(raw) = &p else {
            panic!()
        };
        let micro = numeric_neighbors(raw, &m.build());
        assert!(micro.iter().any(|(_, l)| l.contains("repeat count")));
        assert!(micro.iter().any(|(_, l)| l.contains("wait/delay")));
        assert!(micro.iter().any(|(_, l)| l.contains("action parameter")));
        assert!(micro.iter().all(|(s, _)| s.contains("{ wait ( 1 ) ; }")));
        assert_eq!(micro, numeric_neighbors(raw, &m.build()));
        let rotations = vec![
            (
                "a".to_string(),
                "active bennett; bennett skill, attack; bennett burst;".to_string(),
            ),
            (
                "b".into(),
                "active bennett; bennett skill, attack; bennett burst;".into(),
            ),
        ];
        let d = Dictionary::build("test", &rotations);
        let members: Vec<u16> = m
            .members
            .iter()
            .map(|x| gse::gcsl::char_key(&x.key).unwrap())
            .collect();
        let mut rng = Rng(3);
        let mut labels = BTreeSet::new();
        for _ in 0..400 {
            if let Some((s, label)) = macro_edit(raw, &d, &members, &m.build(), &mut rng) {
                assert!(s.contains("optimizer_cycle_start"));
                gse::gcsl::parse(&crate::evaluate::Evaluator::config(
                    &m,
                    &crate::program::Program::Source(s),
                ))
                .unwrap();
                labels.insert(label.split(' ').take(2).collect::<Vec<_>>().join(" "));
            }
        }
        for kind in [
            "dictionary replace",
            "dictionary insert",
            "delete statement",
            "move statement",
        ] {
            assert!(labels.contains(kind), "{kind} missing from {labels:?}");
        }
    }

    #[test]
    fn manifest_from_build_matches_engine_metadata() {
        let build = std::fs::read_to_string("data/mavuika-c6-blind/build-r5.txt").unwrap();
        let m = manifest_from_build(&build, "test", BTreeMap::new()).unwrap();
        let reference = Manifest::load("data/mavuika-c6-blind/r5.json").unwrap();
        for (a, b) in m.members.iter().zip(&reference.members) {
            assert_eq!(
                (a.cons, &a.weapon, a.refinement),
                (b.cons, &b.weapon, b.refinement)
            );
        }
        assert!(manifest_from_build("nobody char lvl=90/90;", "x", BTreeMap::new()).is_err());
    }
}
