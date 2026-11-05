use crate::artifact::{ENGINE, HEADER, Manifest};
use crate::program::Program;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub fn hash(s: &str) -> String {
    format!("{:x}", Sha256::digest(s.as_bytes()))
}
pub fn cpu() -> f64 {
    let mut t = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // Read-only process clock; includes parsing and parent scheduling work.
    unsafe {
        libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut t);
    }
    t.tv_sec as f64 + t.tv_nsec as f64 * 1e-9
}
fn children_cpu() -> f64 {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::zeroed();
    unsafe {
        if libc::getrusage(libc::RUSAGE_CHILDREN, usage.as_mut_ptr()) != 0 {
            return 0.0;
        }
        let u = usage.assume_init();
        u.ru_utime.tv_sec as f64
            + u.ru_utime.tv_usec as f64 * 1e-6
            + u.ru_stime.tv_sec as f64
            + u.ru_stime.tv_usec as f64 * 1e-6
    }
}
pub fn atomic(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
    let mut f = fs::File::create(&tmp).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(&mut f, value).map_err(|e| e.to_string())?;
    f.write_all(b"\n")
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sample {
    pub dps: f64,
    pub first: f64,
    pub second: f64,
    pub chars: Vec<f64>,
    pub waits: Vec<Vec<(String, f64)>>,
    pub frames: i32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub n: usize,
    pub mean: f64,
    pub se: f64,
    pub first: f64,
    pub second: f64,
    pub chars: Vec<f64>,
    pub waits: Vec<BTreeMap<String, f64>>,
}
pub fn stats(samples: &[Sample]) -> Stats {
    let n = samples.len();
    let nf = n as f64;
    let mean = samples.iter().map(|s| s.dps).sum::<f64>() / nf;
    let se = if n > 1 {
        (samples.iter().map(|s| (s.dps - mean).powi(2)).sum::<f64>() / (nf * (nf - 1.0))).sqrt()
    } else {
        0.0
    };
    let mut out = Stats {
        n,
        mean,
        se,
        first: 0.0,
        second: 0.0,
        chars: vec![0.0; 4],
        waits: vec![BTreeMap::new(); 4],
    };
    for s in samples {
        out.first += s.first / nf;
        out.second += s.second / nf;
        for c in 0..4 {
            out.chars[c] += s.chars[c] / nf;
            for (reason, seconds) in &s.waits[c] {
                *out.waits[c].entry(reason.clone()).or_default() += seconds / nf;
            }
        }
    }
    out
}
pub fn paired(a: &[Sample], b: &[Sample]) -> (f64, f64) {
    assert_eq!(a.len(), b.len());
    let d: Vec<f64> = a.iter().zip(b).map(|(a, b)| a.dps - b.dps).collect();
    let n = d.len() as f64;
    let m = d.iter().sum::<f64>() / n;
    (
        m,
        if n > 1.0 {
            (d.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n * (n - 1.0))).sqrt()
        } else {
            0.0
        },
    )
}
#[derive(Serialize, Deserialize)]
struct Job {
    config: String,
    seed: i64,
    count: usize,
}
#[derive(Serialize, Deserialize)]
struct Work {
    samples: Vec<Result<Sample, String>>,
    cpu: f64,
    wall: f64,
}
#[derive(Serialize, Deserialize)]
struct Record {
    key: String,
    seed: i64,
    work: Work,
    #[serde(default)]
    unresolved: Option<String>,
}

pub fn worker(input: &str, output: &str) -> Result<(), String> {
    let start = cpu();
    let wall = Instant::now();
    let j: Job = serde_json::from_slice(&fs::read(input).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let mut samples = Vec::new();
    for i in 0..j.count {
        samples.push(
            gse::api::run_iterations(&j.config, 150.0, j.seed, i, 1).and_then(|mut v| {
                let s = v.remove(0);
                if s.frames != 27000 || !s.dps.is_finite() {
                    return Err("incomplete full-horizon fight".into());
                }
                Ok(Sample {
                    dps: s.dps,
                    first: s.first,
                    second: s.second,
                    chars: s.chars,
                    waits: s
                        .waits
                        .into_iter()
                        .map(|w| w.into_iter().map(|(k, v)| (k.into(), v)).collect())
                        .collect(),
                    frames: s.frames,
                })
            }),
        );
    }
    atomic(
        Path::new(output),
        &Work {
            samples,
            cpu: cpu() - start,
            wall: wall.elapsed().as_secs_f64(),
        },
    )
}

pub struct Evaluator {
    pub cache: BTreeMap<String, BTreeMap<i64, Result<Sample, String>>>,
    pub worker_cpu: f64,
    pub calls: usize,
    pub fights: usize,
    pub cache_hits: usize,
    pub failures: usize,
    pub timeouts: usize,
    pub batch_wall: Vec<f64>,
    dir: PathBuf,
}
impl Evaluator {
    pub fn key(config: &str) -> String {
        hash(&format!("{ENGINE}\nwarmup=150\n{config}"))
    }
    pub fn new(dir: &Path) -> Result<Self, String> {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let mut e = Self {
            cache: BTreeMap::new(),
            worker_cpu: 0.0,
            calls: 0,
            fights: 0,
            cache_hits: 0,
            failures: 0,
            timeouts: 0,
            batch_wall: Vec::new(),
            dir: dir.into(),
        };
        if let Ok(s) = fs::read_to_string(dir.join("evaluations.jsonl")) {
            for line in s.lines() {
                // A crash can leave only the final append truncated; any earlier
                // corruption must remain visible instead of silently losing work.
                let r: Record = match serde_json::from_str(line) {
                    Ok(r) => r,
                    Err(err) => {
                        return Err(format!(
                            "evaluation journal: {err}; recover its truncated final line explicitly"
                        ));
                    }
                };
                e.worker_cpu += r.work.cpu;
                if r.unresolved.is_some() {
                    e.timeouts += 1;
                }
                e.calls += 1;
                e.fights += r.work.samples.len();
                e.batch_wall.push(r.work.wall);
                for (i, s) in r.work.samples.into_iter().enumerate() {
                    if s.is_err() {
                        e.failures += 1;
                    }
                    e.cache
                        .entry(r.key.clone())
                        .or_default()
                        .insert(r.seed + i as i64, s);
                }
            }
        }
        Ok(e)
    }
    pub fn config(m: &Manifest, p: &Program) -> String {
        format!("{HEADER}{}{}", m.build(), p.gcsl())
    }
    pub fn eval(
        &mut self,
        m: &Manifest,
        p: &Program,
        seed: i64,
        n: usize,
    ) -> Result<Vec<Sample>, String> {
        p.validate()?;
        self.raw(&Self::config(m, p), seed, n)
    }
    pub fn raw(&mut self, config: &str, seed: i64, n: usize) -> Result<Vec<Sample>, String> {
        if n == 0 {
            return Err("positive sample count required".into());
        }
        let key = Self::key(config);
        let mut i = 0;
        while i < n {
            if self
                .cache
                .get(&key)
                .is_some_and(|v| v.contains_key(&(seed + i as i64)))
            {
                self.cache_hits += 1;
                i += 1;
                continue;
            }
            let first = i;
            i += 1;
            while i < n
                && !self
                    .cache
                    .get(&key)
                    .is_some_and(|v| v.contains_key(&(seed + i as i64)))
            {
                i += 1;
            }
            let input = self.dir.join("worker-input.json");
            let output = self.dir.join("worker-output.json");
            atomic(
                &input,
                &Job {
                    config: config.into(),
                    seed: seed + first as i64,
                    count: i - first,
                },
            )?;
            if output.exists() {
                fs::remove_file(&output).map_err(|e| e.to_string())?;
            }
            let child_start = children_cpu();
            let mut child = Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
                .arg("__worker")
                .arg(&input)
                .arg(&output)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| e.to_string())?;
            let wall = Instant::now();
            let limit = Duration::from_secs(10);
            let status = loop {
                if let Some(s) = child.try_wait().map_err(|e| e.to_string())? {
                    break s;
                }
                if wall.elapsed() > limit {
                    child.kill().map_err(|e| e.to_string())?;
                    child.wait().map_err(|e| e.to_string())?;
                    let used = children_cpu() - child_start;
                    self.worker_cpu += used;
                    self.timeouts += 1;
                    let r = Record {
                        key: key.clone(),
                        seed: seed + first as i64,
                        work: Work {
                            samples: vec![],
                            cpu: used,
                            wall: wall.elapsed().as_secs_f64(),
                        },
                        unresolved: Some("worker work-limit timeout".into()),
                    };
                    append(&self.dir.join("evaluations.jsonl"), &r)?;
                    return Err("unresolved: worker work-limit timeout".into());
                }
                std::thread::sleep(Duration::from_millis(2));
            };
            if !status.success() {
                return Err(format!("worker failed: {status}"));
            }
            let mut w: Work =
                serde_json::from_slice(&fs::read(&output).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            w.cpu = w.cpu.max(children_cpu() - child_start);
            self.worker_cpu += w.cpu;
            self.calls += 1;
            self.fights += w.samples.len();
            self.batch_wall.push(w.wall);
            let r = Record {
                key: key.clone(),
                seed: seed + first as i64,
                work: w,
                unresolved: None,
            };
            append(&self.dir.join("evaluations.jsonl"), &r)?;
            for (k, s) in r.work.samples.into_iter().enumerate() {
                if s.is_err() {
                    self.failures += 1;
                }
                self.cache
                    .entry(key.clone())
                    .or_default()
                    .insert(r.seed + k as i64, s);
            }
        }
        (0..n)
            .map(|i| self.cache[&key][&(seed + i as i64)].clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sample_reduction_uses_full_horizon_and_disjoint_seed_slices() {
        let m = Manifest::load("data/flins-c1/manifest.json").unwrap();
        let p = Program::families("").remove(1);
        let config = Evaluator::config(&m, &p);
        let a = gse::api::run_iterations(&config, 150.0, 991, 0, 3).unwrap();
        let b = gse::api::run_iterations(&config, 150.0, 991, 1, 2).unwrap();
        assert_eq!(a[1..], b);
        for s in a {
            assert_eq!(s.frames, 27000);
            assert!((s.chars.iter().sum::<f64>() - s.dps).abs() < 1e-6);
        }
    }
}

fn append(path: &Path, r: &Record) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    serde_json::to_writer(&mut f, r).map_err(|e| e.to_string())?;
    f.write_all(b"\n")
        .and_then(|_| f.sync_data())
        .map_err(|e| e.to_string())
}
