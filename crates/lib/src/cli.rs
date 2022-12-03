//! CLI helpers.

use std::io::Write;
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use serde::Serialize;

/// Run mode.
#[derive(Default)]
pub enum Mode {
    /// Default run mode.
    #[default]
    Default,
    /// Run as benchmark.
    Bench,
}

/// Input options.
#[derive(Default)]
pub struct Opts {
    /// Run as a benchmark.
    pub mode: Mode,
    /// Run in verbose mode.
    verbose: bool,
    /// Output JSON report.
    json: bool,
}

impl Opts {
    /// Parse CLI options.
    pub fn parse() -> Result<Self> {
        let mut opts = Self::default();
        let mut it = std::env::args_os().skip(1);

        while let Some(arg) = it.next() {
            let Some(arg) = arg.to_str() else {
                bail!("non-utf8 argument");
            };

            match arg {
                "--bench" => {
                    if !matches!(opts.mode, Mode::Default) {
                        bail!("duplicate `--bench` arguments");
                    }

                    opts.mode = Mode::Bench;
                }
                "--verbose" => {
                    opts.verbose = true;
                }
                "--json" => {
                    opts.json = true;
                }
                "--" => {
                    break;
                }
                other => {
                    bail!("unsupported argument: {other}");
                }
            }
        }

        Ok(opts)
    }
}

pub struct Bencher {}

impl Bencher {
    /// Construct a new bencher.
    #[inline]
    pub fn new() -> Self {
        Self {}
    }

    /// Bench the given fn.
    #[inline]
    pub fn iter<T>(&mut self, opts: &Opts, mut iter: T) -> Result<()>
    where
        T: FnMut() -> Result<()>,
    {
        let start = Instant::now();

        if opts.verbose {
            println!("warming up (5 s)...");
        }

        loop {
            iter()?;
            let cur = Instant::now();

            if cur.duration_since(start).as_secs() >= 5 {
                break;
            }
        }

        if opts.verbose {
            println!("running benches (10 s)...");
        }

        let start = Instant::now();
        let mut samples = Vec::new();

        loop {
            let s = Instant::now();

            iter()?;

            let cur = Instant::now();

            if cur.duration_since(start).as_secs() >= 10 {
                break;
            }

            samples.push(cur.duration_since(s));
        }

        samples.sort();

        let p50 = ((samples.len() as f32) * 0.50) as usize;
        let p95 = ((samples.len() as f32) * 0.95) as usize;
        let p99 = ((samples.len() as f32) * 0.99) as usize;

        let p50 = samples.get(p50).or(samples.last());
        let p95 = samples.get(p95).or(samples.last());
        let p99 = samples.get(p99).or(samples.last());

        let avg = samples.iter().map(|s| s.as_nanos()).sum::<u128>();

        let (Some(&p50), Some(&p95), Some(&p99), Some(samples)) = (p50, p95, p99, (samples.len() != 0).then_some(samples.len())) else {
            if opts.verbose {
                println!("no samples :(");
            }

            return Ok(());
        };

        let avg = avg / (samples as u128);
        let avg = Duration::from_nanos(avg as u64);

        if opts.json {
            let report = Report::new(p50, p95, p99, samples, avg);
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            serde_json::to_writer(&mut stdout, &report)?;
            writeln!(stdout)?;
        } else {
            println!("50th: {p50:?}, 95th: {p95:?}, 99th: {p99:?}");
            println!("samples: {}", samples);
            println!("average: {avg:?}")
        };
        Ok(())
    }
}

#[derive(Serialize)]
struct Report {
    p50: Duration,
    p95: Duration,
    p99: Duration,
    samples: usize,
    avg: Duration,
}

impl Report {
    fn new(p50: Duration, p95: Duration, p99: Duration, samples: usize, avg: Duration) -> Self {
        Self {
            p50,
            p95,
            p99,
            samples,
            avg,
        }
    }
}
