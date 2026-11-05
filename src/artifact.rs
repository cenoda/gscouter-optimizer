use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const ENGINE: &str = "1ba0af119e0477d6f44ee966a78fa06d84d2c5de";
pub const HEADER: &str = "options duration=450 swap_delay=12;\ntarget lvl=100 resist=0.1 radius=2 pos=0,2.4;\nenergy every interval=480,720 amount=1;\n";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Scoring {
    pub duration: u32,
    pub warmup: u32,
    pub swap_delay: u16,
    pub enemy_level: u16,
    pub resistance: f64,
    pub radius: f64,
    pub position: [f64; 2],
    pub periodic_energy: bool,
}

impl Scoring {
    pub fn header(&self) -> String {
        format!(
            "# optimizer-warmup-seconds={}\noptions duration={} swap_delay={};\ntarget lvl={} resist={} radius={} pos={},{};\n{}",
            self.warmup,
            self.duration,
            self.swap_delay,
            self.enemy_level,
            self.resistance,
            self.radius,
            self.position[0],
            self.position[1],
            if self.periodic_energy {
                "energy every interval=480,720 amount=1;\n"
            } else {
                ""
            }
        )
    }

    fn validate(&self) -> Result<(), String> {
        if self.duration == 0
            || self.duration > 3600
            || self.warmup >= self.duration
            || self.enemy_level == 0
            || self.enemy_level > 200
            || !self.resistance.is_finite()
            || !(-1.0..=1.0).contains(&self.resistance)
            || !self.radius.is_finite()
            || self.radius <= 0.0
            || self.position.iter().any(|x| !x.is_finite())
        {
            return Err("invalid frozen scoring conditions".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Token {
    pub stat: String,
    pub quality: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Piece {
    pub slot: String,
    pub rarity: u8,
    pub level: u8,
    pub main: String,
    pub value: f64,
    pub cv_ceiling: f64,
    pub rolls: Vec<Token>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Member {
    pub key: String,
    pub cons: u8,
    pub weapon: String,
    pub refinement: u8,
    pub set: String,
    pub useful: Vec<String>,
    pub weights: BTreeMap<String, f64>,
    pub pieces: Vec<Piece>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    pub format: u8,
    pub engine: String,
    pub site_commit: String,
    pub sources: BTreeMap<String, String>,
    pub max_rolls: BTreeMap<String, f64>,
    pub members: Vec<Member>,
    /// Format 2 freezes a complete supplied build, including talents and params.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_build: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scoring: Option<Scoring>,
}

pub fn rarity(key: &str) -> Result<u8, String> {
    let id = gse::gcsl::char_key(key).ok_or_else(|| format!("unknown character: {key}"))?;
    let roster: BTreeMap<String, u8> =
        serde_json::from_str(include_str!("../data/character-rarities.json"))
            .map_err(|e| e.to_string())?;
    roster
        .into_iter()
        .find_map(|(name, rarity)| (gse::gcsl::char_key(&name) == Some(id)).then_some(rarity))
        .ok_or_else(|| format!("missing rarity metadata: {key}"))
}

impl Piece {
    pub fn totals(&self, max: &BTreeMap<String, f64>) -> BTreeMap<String, f64> {
        let mut out = BTreeMap::new();
        for r in &self.rolls {
            // This initial manifest deliberately uses only actual maximum-tier
            // rolls. Reject other tiers rather than invent rounded game values.
            *out.entry(r.stat.clone()).or_insert(0.0) += max[&r.stat];
        }
        out
    }
    pub fn cv(&self, max: &BTreeMap<String, f64>) -> f64 {
        let t = self.totals(max);
        100.0 * (2.0 * t.get("cr").unwrap_or(&0.0) + t.get("cd").unwrap_or(&0.0))
    }
}

impl Manifest {
    pub fn header(&self) -> String {
        self.scoring
            .as_ref()
            .map_or_else(|| HEADER.into(), Scoring::header)
    }

    pub fn objective(&self) -> (u32, u32) {
        self.scoring
            .as_ref()
            .map_or((450, 150), |s| (s.duration, s.warmup))
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let m: Self = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        m.validate(&m)?;
        Ok(m)
    }
    pub fn validate(&self, frozen: &Self) -> Result<(), String> {
        if self.scoring != frozen.scoring {
            return Err("scoring conditions cannot change during search".into());
        }
        if let Some(s) = &self.scoring {
            s.validate()?;
        }
        if self.engine != ENGINE || self.members.len() != 4 || frozen.members.len() != 4 {
            return Err("manifest requires four members and the pinned engine".into());
        }
        let mut keys = BTreeSet::new();
        for m in &self.members {
            let id = gse::gcsl::char_key(&m.key)
                .ok_or_else(|| format!("unknown character: {}", m.key))?;
            if !keys.insert(id) || m.cons > 6 {
                return Err("duplicate member or invalid constellation".into());
            }
            rarity(&m.key)?;
        }
        if self.format == 2 {
            if self != frozen {
                return Err("fixed build cannot exchange artifact rolls".into());
            }
            let build = self
                .fixed_build
                .as_ref()
                .ok_or("format 2 requires fixed_build")?;
            use gse::gcsl::Tok;
            if gse::gcsl::lex(build).iter().any(|t| {
                matches!(
                    t.typ,
                    Tok::Options | Tok::Target | Tok::Energy | Tok::Hurt | Tok::Active
                )
            }) {
                return Err(
                    "fixed_build cannot override simulation conditions or active character".into(),
                );
            }
            let parsed = gse::gcsl::parse(&format!(
                "{}{build}\nactive {};",
                self.header(),
                self.members[0].key
            ))?;
            if parsed.chars.len() != 4 || !parsed.program.0.is_empty() {
                return Err(
                    "fixed_build must contain only four character build declarations".into(),
                );
            }
            for (m, c) in self.members.iter().zip(&parsed.chars) {
                if gse::gcsl::char_key(&m.key) != Some(c.key)
                    || i32::from(m.cons) != c.cons
                    || m.weapon != gse::gcsl::weapon_name(c.weapon.key)
                    || i32::from(m.refinement) != c.weapon.refine
                    || !m.pieces.is_empty()
                    || !m.useful.is_empty()
                    || !m.weights.is_empty()
                {
                    return Err("fixed-build member metadata mismatch".into());
                }
            }
            return Ok(());
        }
        if self.fixed_build.is_some() {
            return Err("physical manifest cannot contain fixed_build".into());
        }
        let snapshot: Manifest =
            serde_json::from_str(include_str!("../data/flins-c1/manifest.json"))
                .map_err(|e| e.to_string())?;
        if self.max_rolls != snapshot.max_rolls {
            return Err("roll values differ from the versioned site snapshot".into());
        }
        if self.format != 1
            || self.engine != ENGINE
            || self.members.len() != 4
            || self.max_rolls != frozen.max_rolls
            || self.sources != frozen.sources
            || self.site_commit != frozen.site_commit
        {
            return Err("manifest identity or pinned engine mismatch".into());
        }
        for (m, f) in self.members.iter().zip(&frozen.members) {
            if m.useful.is_empty() || m.useful.iter().any(|s| !self.max_rolls.contains_key(s)) {
                return Err("physical manifest requires supported useful substats".into());
            }
            let mut fixed = m.clone();
            fixed.pieces = f.pieces.clone();
            if fixed != *f || m.pieces.len() != 5 {
                return Err("frozen member envelope changed".into());
            }
            for (i, (p, q)) in m.pieces.iter().zip(&f.pieces).enumerate() {
                let mut fixed = p.clone();
                fixed.rolls = q.rolls.clone();
                if fixed != *q
                    || p.rolls.len() != 9
                    || p.rarity != 5
                    || p.level != 20
                    || p.slot != ["flower", "plume", "sands", "goblet", "circlet"][i]
                    || !p.cv_ceiling.is_finite()
                    || !(0.0..=40.0).contains(&p.cv_ceiling)
                {
                    return Err(format!("{} {}: changed piece envelope", m.key, p.slot));
                }
                let legal: &[&str] = match i {
                    0 => &["hp"],
                    1 => &["atk"],
                    2 => &["hp%", "atk%", "def%", "em", "er"],
                    3 => &[
                        "hp%", "atk%", "def%", "em", "pyro%", "hydro%", "electro%", "cryo%",
                        "anemo%", "geo%", "dendro%", "phys%",
                    ],
                    _ => &["hp%", "atk%", "def%", "em", "cr", "cd", "heal"],
                };
                let expected = match p.main.as_str() {
                    "hp" => 4780.0,
                    "atk" => 311.0,
                    "em" => 187.0,
                    "er" => 0.518,
                    "cr" => 0.311,
                    "cd" => 0.622,
                    "heal" => 0.359,
                    "def%" | "phys%" => 0.583,
                    _ => 0.466,
                };
                if !legal.contains(&p.main.as_str()) || (p.value - expected).abs() > 1e-9 {
                    return Err("invalid main stat".into());
                }
                let distinct: BTreeSet<_> = p.rolls.iter().map(|t| &t.stat).collect();
                if distinct.len() != 4 || distinct.contains(&p.main) {
                    return Err("piece needs four distinct non-main substats".into());
                }
                for (a, b) in p.rolls.iter().zip(&q.rolls) {
                    if a.quality != 3
                        || a.quality != b.quality
                        || !self.max_rolls.contains_key(&a.stat)
                    {
                        return Err("unsupported or changed roll quality".into());
                    }
                    let useful = |s: &str| s == "er" || m.useful.iter().any(|x| x == s);
                    if useful(&a.stat) != useful(&b.stat) || (!useful(&b.stat) && a.stat != b.stat)
                    {
                        return Err("fixed useful-roll budget or irrelevant token changed".into());
                    }
                }
                if p.cv(&self.max_rolls) > p.cv_ceiling + 1e-9 {
                    return Err("frozen CV ceiling exceeded".into());
                }
            }
        }
        Ok(())
    }
    pub fn build(&self) -> String {
        if let Some(build) = &self.fixed_build {
            return format!("{build}\n");
        }
        let mut out = String::new();
        for m in &self.members {
            out.push_str(&format!("{} char lvl=90/90 cons={} talent=10,10,10;\n{} add weapon=\"{}\" refine={} lvl=90/90;\n{} add set=\"{}\" count=4;\n", m.key, m.cons, m.key, m.weapon, m.refinement, m.key, m.set));
            let mut t = BTreeMap::new();
            for p in &m.pieces {
                *t.entry(p.main.clone()).or_insert(0.0) += p.value;
                for (k, v) in p.totals(&self.max_rolls) {
                    *t.entry(k).or_insert(0.0) += v;
                }
            }
            out.push_str(&format!(
                "{} add stats {};\n",
                m.key,
                t.iter()
                    .map(|(k, v)| format!("{k}={v:.8}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            ));
        }
        out
    }
    pub fn score(&self) -> Vec<f64> {
        self.members
            .iter()
            .map(|m| {
                m.pieces
                    .iter()
                    .map(|p| {
                        p.totals(&self.max_rolls)
                            .iter()
                            .map(|(s, v)| {
                                v / self.max_rolls[s] * m.weights.get(s).unwrap_or(&0.0) * 7.77
                            })
                            .sum::<f64>()
                    })
                    .sum()
            })
            .collect()
    }
    pub fn exchange(
        &self,
        frozen: &Self,
        ci: usize,
        pi: usize,
        ti: usize,
        stat: &str,
    ) -> Option<Self> {
        if self.fixed_build.is_some() {
            return None;
        }
        let old = &self.members[ci].pieces[pi].rolls[ti].stat;
        let useful = &self.members[ci].useful;
        if old == stat
            || !(old == "er" && useful.iter().any(|s| s == stat)
                || stat == "er" && useful.contains(old))
        {
            return None;
        }
        let mut next = self.clone();
        next.members[ci].pieces[pi].rolls[ti].stat = stat.into();
        next.validate(frozen).ok()?;
        Some(next)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn fixed_party() -> Manifest {
        let build = "albedo char lvl=90/90 cons=1 talent=9,9,9;
albedo add weapon=\"cinnabarspindle\" refine=5 lvl=90/90;
albedo add set=\"huskofopulentdreams\" count=4 +params=[stacks=4];
bennett char lvl=90/90 cons=6 talent=9,9,9;
bennett add weapon=\"favsword\" refine=3 lvl=90/90;
xiangling char lvl=90/90 cons=6 talent=9,9,9;
xiangling add weapon=\"thecatch\" refine=5 lvl=90/90;
noelle char lvl=90/90 cons=6 talent=9,9,9;
noelle add weapon=\"favgs\" refine=3 lvl=90/90;";
        let parsed = gse::gcsl::parse(&format!("{HEADER}{build}\nactive albedo;")).unwrap();
        Manifest {
            format: 2,
            engine: ENGINE.into(),
            site_commit: "test fixture".into(),
            sources: BTreeMap::new(),
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
        }
    }

    #[test]
    fn fixed_build_is_frozen_and_member_metadata_matches() {
        let m = fixed_party();
        m.validate(&m).unwrap();
        assert_eq!(rarity("albedo").unwrap(), 5);
        assert_eq!(rarity("noelle").unwrap(), 4);
        assert_eq!(rarity("amy").unwrap(), rarity("fischl").unwrap());
        assert!(m.exchange(&m, 0, 0, 0, "er").is_none());
        let mut bad = m.clone();
        bad.members[0].cons = 2;
        assert!(bad.validate(&bad).is_err());
        let mut bad = m.clone();
        bad.fixed_build
            .as_mut()
            .unwrap()
            .push_str("\noptions duration=10;");
        assert!(bad.validate(&bad).is_err());
        let mut bad = m.clone();
        bad.fixed_build
            .as_mut()
            .unwrap()
            .push_str("\nalbedo attack;");
        assert!(bad.validate(&bad).is_err());
        let mut bad = m.clone();
        bad.fixed_build
            .as_mut()
            .unwrap()
            .push_str("\nalbedo add stats atk=1;");
        assert!(bad.validate(&m).is_err());
    }
    #[test]
    fn allocation_moves_preserve_physical_and_investment_constraints() {
        let f = Manifest::load("data/flins-c1/manifest.json").unwrap();
        let mut accepted = 0;
        for c in 0..4 {
            for p in 0..5 {
                for t in 0..9 {
                    for s in ["er", "cr", "cd", "atk%", "hp%", "em"] {
                        if let Some(m) = f.exchange(&f, c, p, t, s) {
                            accepted += 1;
                            m.validate(&f).unwrap();
                        }
                    }
                }
            }
        }
        assert!(accepted > 20);
        let mut bad = f.clone();
        bad.members[0].pieces[0].rolls[0].quality = 2;
        assert!(bad.validate(&f).is_err());
        let mut bad = f.clone();
        bad.max_rolls.insert("cr".into(), 0.5);
        assert!(bad.validate(&bad).is_err());
        let mut bad = f.clone();
        bad.members[0].pieces[0].rolls.push(Token {
            stat: "cr".into(),
            quality: 3,
        });
        assert!(bad.validate(&f).is_err());
        let mut bad = f.clone();
        bad.members[0].pieces[0].rolls[0].stat = "hp".into();
        assert!(bad.validate(&f).is_err());
    }
}
