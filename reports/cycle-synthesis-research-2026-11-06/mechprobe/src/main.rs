//! Generic mechanical probe: for every party member and every basic action,
//! run that action once from a fresh simulation and record what the engine
//! state says. No character names appear in this file.
use gse::gcsl::{self, Num};
use gse::sim::{Params, Sim};
use gse::types::Action;
use std::fmt::Write as _;

fn num(n: Result<Num, String>) -> Option<f64> { n.ok().map(|x| x.f()) }

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args[1] == "--alias" {
        // resolve character words (aliases) to engine keys, one per line
        for w in &args[2..] {
            println!("{w} {}", gcsl::char_key(w).map(gcsl::char_name).unwrap_or("-"));
        }
        return;
    }
    let cfg_text = std::fs::read_to_string(&args[1]).expect("config");
    // the config without its rotation and without periodic energy
    let header: String = cfg_text.lines().filter(|l| !l.starts_with("energy every") && !l.starts_with("active ") && !l.contains("while 1")).collect::<Vec<_>>().join("\n");
    let first_char = header.lines().find_map(|l| l.split_once(" char ").map(|(n, _)| n.trim().to_string())).expect("char line");
    let parsed0 = gcsl::parse(&format!("{header}\nactive {first_char};\n")).expect("parse");
    let names: Vec<&'static str> = parsed0.chars.iter().map(|c| gcsl::char_name(c.key)).collect();
    let mut out = String::from("[");
    let acts = [("skill", Action::Skill), ("burst", Action::Burst), ("attack", Action::Attack), ("charge", Action::Charge)];
    let mut first = true;
    for (ci, name) in names.iter().enumerate() {
        for (an, a) in acts {
            let src = format!("{header}\nactive {name};\n");
            let parsed = gcsl::parse(&src).expect("parse");
            let cfg = gse::profile::config(&parsed).expect("cfg");
            let mut sim = Sim::new(&cfg, 1, 0.0);
            // put the member on field as the evaluator does, then settle
            if sim.player.active as usize != ci {
                sim.queue_action(ci, Action::Swap, &Params::default());
            }
            sim.wait(60);
            let e0: Vec<f64> = (0..names.len()).map(|i| sim.chars[i].energy).collect();
            let f0 = sim.f;
            let before: Vec<(usize, &'static str, i32)> = sim.chars.iter().enumerate().flat_map(|(i, c)| c.mods.iter().filter(|m| m.expiry() > sim.f).map(move |m| (i, m.key.name, m.expiry())).collect::<Vec<_>>()).collect();
            let ok = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| sim.queue_action(ci, a, &Params::default()))) {
                Ok(v) => v,
                Err(_) => {
                    if !first { out.push(','); }
                    first = false;
                    let _ = write!(out, "{{\"char\":\"{name}\",\"action\":\"{an}\",\"ok\":false,\"error\":\"engine panic\"}}");
                    continue;
                }
            };
            let f1 = sim.f;
            let err = sim.exec_err.take();
            let cd = |sim: &Sim, what: &str| num(sim.field(&[name.to_string(), what.to_string(), "cd".to_string()]));
            let skill_cd = cd(&sim, "skill");
            let burst_cd = cd(&sim, "burst");
            let energy_after: Vec<f64> = (0..names.len()).map(|i| sim.chars[i].energy).collect();
            // statuses/mods started by the action, with remaining frames
            let mut mods = Vec::new();
            for (i, c) in sim.chars.iter().enumerate() {
                for m in &c.mods {
                    if m.expiry() > sim.f && !before.iter().any(|b| b.0 == i && b.1 == m.key.name && b.2 == m.expiry()) {
                        mods.push(format!("{{\"on\":\"{}\",\"key\":\"{}\",\"remaining\":{}}}", names[i], m.key.name, m.expiry() - sim.f));
                    }
                }
            }
            // empty every member's energy, then let particles arrive (no
            // periodic energy in this probe)
            let used = (e0[ci] - energy_after[ci]).max(0.0);
            for c in sim.chars.iter_mut() { c.energy = 0.0; }
            let energy_after: Vec<f64> = vec![0.0; names.len()];
            sim.wait(240);
            let energy_later: Vec<f64> = (0..names.len()).map(|i| sim.chars[i].energy).collect();
            let gained: f64 = (0..names.len()).map(|i| (energy_later[i] - energy_after[i]).max(0.0)).sum();
            if !first { out.push(','); }
            first = false;
            let _ = write!(out, "{{\"char\":\"{name}\",\"action\":\"{an}\",\"ok\":{ok},\"error\":{},\"frames\":{},\"skill_cd\":{},\"burst_cd\":{},\"energy_max\":{},\"energy_used\":{},\"team_energy_gain_4s\":{:.2},\"mods\":[{}]}}",
                err.map(|e| format!("{e:?}")).unwrap_or("null".into()), f1 - f0,
                skill_cd.map(|x| x.to_string()).unwrap_or("null".into()), burst_cd.map(|x| x.to_string()).unwrap_or("null".into()),
                sim.chars[ci].energy_max, used, gained, mods.join(","));
        }
    }
    out.push(']');
    println!("{out}");
}
