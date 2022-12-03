//! CLI helpers.

use std::fmt;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use serde::Serialize;

/// Default warmup period in seconds.
const DEFAULT_WARMUP: u64 = 5;

/// Default time in seconds.
const DEFAULT_TIME: u64 = 5;

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
    /// Warmup period.
    warmup: Option<u64>,
    /// Bench period.
    time: Option<u64>,
    /// Number of times to run benches.
    count: Option<usize>,
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
                "--warmup" => {
                    let warmup = it.next().context("missing argument to `--warmup`")?;
                    let warmup = warmup
                        .to_str()
                        .context("missing string argument to `--warmup`")?;
                    opts.warmup = Some(warmup.parse().context("bad argument to `--warmup`")?);
                }
                "--time" => {
                    let time = it.next().context("missing argument to `--time`")?;
                    let time = time
                        .to_str()
                        .context("missing string argument to `--time`")?;
                    opts.time = Some(time.parse().context("bad argument to `--time`")?);
                }
                "--count" => {
                    let count = it.next().context("missing argument to `--count`")?;
                    let count = count
                        .to_str()
                        .context("missing string argument to `--count`")?;
                    opts.count = Some(count.parse().context("bad argument to `--count`")?);
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
    pub fn iter<T, O>(&mut self, opts: &Opts, mut iter: T) -> Result<()>
    where
        T: FnMut() -> Result<O>,
    {
        let warmup = Duration::from_secs(opts.warmup.unwrap_or(DEFAULT_WARMUP));
        let time = Duration::from_secs(opts.time.unwrap_or(DEFAULT_TIME));

        let stdout = std::io::stdout();

        let mut o = Output {
            out: stdout.lock(),
            kind: if opts.json {
                OutputKind::Json
            } else {
                OutputKind::Normal
            },
        };

        if !warmup.is_zero() {
            let start = Instant::now();

            o.info(format_args!("warming up ({warmup:?})..."))?;

            loop {
                let _ = black_box(iter()?);
                let cur = Instant::now();

                if cur.duration_since(start) >= warmup {
                    break;
                }
            }
        }

        let mut samples = Vec::new();

        if let Some(count) = opts.count {
            let count = count.max(1);
            o.info(format_args!(
                "running benches {times} time(s)...",
                times = count
            ))?;

            for _ in 0..count {
                let s = Instant::now();
                let _ = black_box(iter()?);
                let cur = Instant::now();
                samples.push(cur.duration_since(s));
            }
        } else {
            o.info(format_args!("running benches ({time:?})..."))?;

            let start = Instant::now();

            loop {
                let s = Instant::now();

                let _ = black_box(iter()?);

                let cur = Instant::now();

                if cur.duration_since(start) >= time {
                    break;
                }

                samples.push(cur.duration_since(s));
            }
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
            o.error("no samples :(")?;
            return Ok(());
        };

        let avg = avg / (samples as u128);
        let avg = Duration::from_nanos(avg as u64);
        let report = Report::new(p50, p95, p99, samples, avg);
        o.report(&report)?;
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

/// A function that is opaque to the optimizer, used to prevent the compiler from
/// optimizing away computations in a benchmark.
///
/// This variant is stable-compatible, but it may cause some performance overhead
/// or fail to prevent code from being eliminated.
///
/// Borrowed from criterion under the MIT license.
fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}

struct Output<O> {
    out: O,
    kind: OutputKind,
}

enum OutputKind {
    Json,
    Normal,
}

impl<O> Output<O>
where
    O: Write,
{
    fn info(&mut self, m: impl fmt::Display) -> io::Result<()> {
        self.message(MessageKind::Info, m)
    }

    fn error(&mut self, m: impl fmt::Display) -> io::Result<()> {
        self.message(MessageKind::Error, m)
    }

    fn message(&mut self, kind: MessageKind, m: impl fmt::Display) -> io::Result<()> {
        match &self.kind {
            OutputKind::Json => {
                self.json(&Line {
                    kind: LineKind::Message,
                    data: Message { output: m, kind },
                })?;
            }
            OutputKind::Normal => {
                writeln!(self.out, "{kind}: {m}")?;
            }
        }

        Ok(())
    }

    fn report(&mut self, report: &Report) -> io::Result<()> {
        match &self.kind {
            OutputKind::Json => {
                self.json(&Line {
                    kind: LineKind::Report,
                    data: report,
                })?;
            }
            OutputKind::Normal => {
                let Report {
                    p50,
                    p95,
                    p99,
                    samples,
                    avg,
                } = report;

                writeln!(self.out, "50th: {p50:?}, 95th: {p95:?}, 99th: {p99:?}")?;
                writeln!(self.out, "samples: {}", samples)?;
                writeln!(self.out, "average: {avg:?}")?;
            }
        }

        Ok(())
    }

    fn json<T>(&mut self, m: &T) -> io::Result<()>
    where
        T: Serialize,
    {
        serde_json::to_writer(&mut self.out, m)?;
        writeln!(self.out)?;
        Ok(())
    }
}

#[derive(Serialize)]
struct Line<T> {
    kind: LineKind,
    data: T,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
enum LineKind {
    Message,
    Report,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
enum MessageKind {
    Info,
    Error,
}

impl fmt::Display for MessageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageKind::Info => write!(f, "info"),
            MessageKind::Error => write!(f, "error"),
        }
    }
}

struct Message<T> {
    output: T,
    kind: MessageKind,
}

impl<T> Serialize for Message<T>
where
    T: fmt::Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("output", &DisplayString(&self.output))?;
        map.end()
    }
}

struct DisplayString<T>(T);

impl<T> Serialize for DisplayString<T>
where
    T: fmt::Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.0)
    }
}
