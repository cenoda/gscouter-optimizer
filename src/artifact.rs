use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const ENGINE: &str = "1ba0af119e0477d6f44ee966a78fa06d84d2c5de";
pub const HEADER: &str = "options duration=450 swap_delay=12;\ntarget lvl=100 resist=0.1 radius=2 pos=0,2.4;\nenergy every interval=480,720 amount=1;\n";

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
    pub fn load(path: &str) -> Result<Self, String> {
        let m: Self = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        m.validate(&m)?;
        Ok(m)
    }
    pub fn validate(&self, frozen: &Self) -> Result<(), String> {
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
            let required = match m.key.as_str() {
                "flins" => Some("bloodsoakedruins"),
                "ineffa" => Some("fracturedhalo"),
                "columbina" => Some("nocturnescurtaincall"),
                _ => None,
            };
            if let Some(weapon) = required
                && (m.weapon != weapon || m.refinement != 1)
            {
                return Err("pilot signature must be R1".into());
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
mod tests {
    use super::*;
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
