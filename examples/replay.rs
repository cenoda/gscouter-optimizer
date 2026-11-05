//! Replay diagnostics using only the pinned engine's public API.
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err("usage: replay CONFIG ITERATIONS SEED WARMUP_SECONDS".into());
    }
    let src = std::fs::read_to_string(&args[0])?;
    let n: usize = args[1].parse()?;
    let seed: i64 = args[2].parse()?;
    let warmup: f64 = args[3].parse()?;
    if n == 0 || !warmup.is_finite() || warmup < 0.0 {
        return Err("positive iterations and nonnegative finite warmup required".into());
    }
    let missing = gse::api::unsupported(&src)?;
    if !missing.is_empty() {
        return Err(missing.join("; ").into());
    }
    let parsed = gse::gcsl::parse(&src)?;
    let cfg = gse::profile::config(&parsed).map_err(|e| e.join("; "))?;
    if cfg.duration <= warmup {
        return Err("a fixed duration longer than warmup is required".into());
    }
    let mut abilities: BTreeMap<String, BTreeMap<String, (f64, f64)>> = BTreeMap::new();
    let mut values = Vec::with_capacity(n);
    let mut initial_stats = Vec::new();
    let mut final_stats = Vec::new();
    for i in 0..n {
        let mut sim = gse::sim::Sim::new(&cfg, seed.wrapping_add(i as i64) as u64, warmup);
        if i == 0 {
            initial_stats = (0..cfg.chars.len())
                .map(|ci| sim.stats(ci).to_vec())
                .collect();
        }
        gse::eval::run(&mut sim, &parsed.program)?;
        let seconds = f64::from(sim.f - sim.window_start) / 60.0;
        if sim.f != (cfg.duration * 60.0) as i32 {
            return Err(format!("incomplete fight at frame {}", sim.f).into());
        }
        values.push(sim.window_buckets.iter().sum::<f64>() / seconds);
        for (ci, abils) in sim.abils.iter().enumerate() {
            let character = gse::gcsl::char_name(parsed.chars[ci].key).to_owned();
            for (name, hits, damage) in abils {
                let entry = abilities
                    .entry(character.clone())
                    .or_default()
                    .entry(name.to_string())
                    .or_default();
                entry.0 += *hits as f64 / n as f64;
                entry.1 += damage / (seconds * n as f64);
            }
        }
        if i == 0 {
            final_stats = (0..cfg.chars.len())
                .map(|ci| sim.stats(ci).to_vec())
                .collect();
        }
    }
    let mean = values.iter().sum::<f64>() / n as f64;
    let se = if n > 1 {
        (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n * (n - 1)) as f64).sqrt()
    } else {
        0.0
    };
    println!(
        "{}",
        serde_json::json!({"iterations":n,"seed":seed,
        "duration":cfg.duration,"warmup":warmup,"team":{"mean":mean,"se":se},
        "abilities_hits_and_dps":abilities,"per_iteration_dps":values,
        "initial_stats":initial_stats,"first_fight_final_stats":final_stats})
    );
    Ok(())
}
