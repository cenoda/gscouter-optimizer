//! Independent ability diagnostics through the pinned engine's public library.
//! No simulation or character implementation is copied into this repository.
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: trace CONFIG ITERATIONS SEED".into());
    }
    let src = std::fs::read_to_string(&args[0])?;
    let n: usize = args[1].parse()?;
    let seed: i64 = args[2].parse()?;
    if n == 0 {
        return Err("positive iterations required".into());
    }
    let parsed = gse::gcsl::parse(&src)?;
    let config = gse::profile::config(&parsed).map_err(|e| e.join("; "))?;
    let samples = gse::api::run_iterations(&src, 150.0, seed, 0, n)?;
    let mean = samples.iter().map(|s| s.dps).sum::<f64>() / n as f64;
    let se = if n > 1 {
        (samples.iter().map(|s| (s.dps - mean).powi(2)).sum::<f64>() / (n * (n - 1)) as f64).sqrt()
    } else {
        0.0
    };
    let mut abilities: BTreeMap<String, BTreeMap<String, (f64, f64)>> = BTreeMap::new();
    for i in 0..n {
        let mut sim = gse::sim::Sim::new(&config, seed.wrapping_add(i as i64) as u64, 150.0);
        gse::eval::run(&mut sim, &parsed.program)?;
        if sim.f != 27000 {
            return Err("incomplete full fight".into());
        }
        for (ci, abils) in sim.abils.iter().enumerate() {
            let character = gse::gcsl::char_name(parsed.chars[ci].key).to_owned();
            for (name, hits, damage) in abils {
                let entry = abilities
                    .entry(character.clone())
                    .or_default()
                    .entry(name.to_string())
                    .or_default();
                entry.0 += *hits as f64 / n as f64;
                entry.1 += damage / (300.0 * n as f64);
            }
        }
    }
    let mut chars = BTreeMap::new();
    for ci in 0..parsed.chars.len() {
        let values: Vec<f64> = samples.iter().map(|s| s.chars[ci]).collect();
        let average = values.iter().sum::<f64>() / n as f64;
        let se = if n > 1 {
            (values.iter().map(|v| (v - average).powi(2)).sum::<f64>() / (n * (n - 1)) as f64)
                .sqrt()
        } else {
            0.0
        };
        chars.insert(
            gse::gcsl::char_name(parsed.chars[ci].key),
            serde_json::json!({"mean":average,"se":se}),
        );
    }
    println!(
        "{}",
        serde_json::json!({"iterations":n,"seed":seed,"duration":450,"warmup":150,"team":{"mean":mean,"se":se},"chars":chars,"abilities_hits_and_dps":abilities})
    );
    Ok(())
}
