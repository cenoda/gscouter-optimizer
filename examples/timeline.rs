//! Recover a deterministic damage timeline through repeated public-API replays.
//! Each replay ends one frame later. This is intentionally a diagnostic, not a
//! fast search evaluator; it does not add hooks or change the pinned engine.
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: timeline CONFIG SEED FRAMES (at most 1800)".into());
    }
    let src = std::fs::read_to_string(&args[0])?;
    let seed: i64 = args[1].parse()?;
    let frames: i32 = args[2].parse()?;
    if !(1..=1800).contains(&frames) {
        return Err("frames must be between 1 and 1800".into());
    }
    let missing = gse::api::unsupported(&src)?;
    if !missing.is_empty() {
        return Err(missing.join("; ").into());
    }
    let parsed = gse::gcsl::parse(&src)?;
    let mut cfg = gse::profile::config(&parsed).map_err(|e| e.join("; "))?;
    if cfg.enemy_hp > 0.0 || cfg.duration * 60.0 < f64::from(frames) {
        return Err("requires a timed config covering the requested frames".into());
    }
    let mut previous = BTreeMap::<(usize, String), (u32, f64)>::new();
    let mut events = Vec::new();
    for frame in 1..=frames {
        cfg.duration = f64::from(frame) / 60.0;
        let mut sim = gse::sim::Sim::new(&cfg, seed as u64, 0.0);
        gse::eval::run(&mut sim, &parsed.program)?;
        for (ci, abils) in sim.abils.iter().enumerate() {
            for (name, hits, damage) in abils {
                let key = (ci, name.to_string());
                let old = previous.get(&key).copied().unwrap_or_default();
                if *hits < old.0 || *damage + 1e-6 < old.1 {
                    return Err("replay horizon changed an earlier damage prefix".into());
                }
                if *hits > old.0 {
                    events.push(
                        serde_json::json!({"frame":sim.f,"seconds":f64::from(sim.f)/60.0,
                        "character":gse::gcsl::char_name(parsed.chars[ci].key),
                        "ability":name,"hits":hits-old.0,"damage":damage-old.1}),
                    );
                }
                previous.insert(key, (*hits, *damage));
            }
        }
    }
    println!(
        "{}",
        serde_json::json!({"seed":seed,"frames":frames,
        "method":"one-frame horizon differences; same seed; warmup zero",
        "events":events})
    );
    Ok(())
}
