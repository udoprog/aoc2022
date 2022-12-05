//! CLI helpers.

pub(crate) mod error;
mod output_eq;
mod stdout_logger;

use std::fmt;
use std::io::{self, Write};
use std::ops::AddAssign;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};

pub use self::error::CliError;
pub use self::output_eq::OutputEq;

/// Default warmup period in seconds.
const DEFAULT_WARMUP: u64 = 100;

/// Default time in seconds.
const DEFAULT_TIME: u64 = 400;

static STDOUT_LOGGER: stdout_logger::StdoutLogger = stdout_logger::StdoutLogger;

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

        match &opts.mode {
            Mode::Default => {
                log::set_max_level(log::LevelFilter::Info);
                log::set_logger(&STDOUT_LOGGER)
                    .map_err(|error| anyhow!("failed to set log: {error}"))?;
            }
            _ => {}
        }

        Ok(opts)
    }
}

#[derive(Default)]
pub struct Bencher {}

impl Bencher {
    /// Construct a new bencher.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Bench the given fn.
    #[inline]
    pub fn iter<T, O, E>(&mut self, opts: &Opts, expected: Option<E>, iter: T) -> Result<()>
    where
        T: FnMut() -> Result<O>,
        O: fmt::Debug + OutputEq<E>,
        E: fmt::Debug,
    {
        let stdout = std::io::stdout();

        let mut o = Output {
            out: stdout.lock(),
            kind: if opts.json {
                OutputKind::Json
            } else {
                OutputKind::Normal
            },
        };

        if let Err(e) = self.inner_iter(&mut o, opts, expected, iter) {
            o.error(e)?;
        }

        Ok(())
    }

    fn inner_iter<T, O, E>(
        &mut self,
        o: &mut Output<impl Write>,
        opts: &Opts,
        expected: Option<E>,
        mut iter: T,
    ) -> Result<()>
    where
        T: FnMut() -> Result<O>,
        O: fmt::Debug + OutputEq<E>,
        E: fmt::Debug,
    {
        let warmup = Duration::from_millis(opts.warmup.unwrap_or(DEFAULT_WARMUP));
        let time = Duration::from_millis(opts.time.unwrap_or(DEFAULT_TIME));

        if !warmup.is_zero() {
            let start = Instant::now();

            o.info(format_args!("warming up ({warmup:?})..."))?;

            loop {
                let value = iter()?;
                let cur = Instant::now();

                if let Some(expect) = &expected {
                    if !value.output_eq(expect) {
                        bail!("{value:?} (value) != {expect:?} (expected)");
                    }
                }

                let _ = black_box(value);

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
                let value = iter()?;
                let cur = Instant::now();

                if let Some(expect) = &expected {
                    if !value.output_eq(expect) {
                        bail!("{value:?} (value) != {expect:?} (expected)");
                    }
                }

                let _ = black_box(value);
                samples.push(cur.duration_since(s));
            }
        } else {
            o.info(format_args!("running benches ({time:?})..."))?;

            let start = Instant::now();

            loop {
                let s = Instant::now();

                let value = iter()?;

                let cur = Instant::now();

                if let Some(expect) = &expected {
                    if !value.output_eq(expect) {
                        bail!("{value:?} (value) != {expect:?} (expected)");
                    }
                }

                let _ = black_box(value);

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

        let last = samples.last();
        let p50 = samples.get(p50).or(last);
        let p95 = samples.get(p95).or(last);
        let p99 = samples.get(p99).or(last);

        let (Some(&p50), Some(&p95), Some(&p99), Some(count)) = (p50, p95, p99, (!samples.is_empty()).then_some(samples.len())) else {
            o.error("no samples :(")?;
            return Ok(());
        };

        let min = samples.first().copied().context("missing min")?;
        let max = samples.last().copied().context("missing max")?;
        let sum = samples.iter().copied().sum();
        let report = Report::new(p50, p95, p99, count, min, max, sum);
        o.report(&report)?;
        Ok(())
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct Report {
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub count: usize,
    pub min: Duration,
    pub max: Duration,
    pub sum: Duration,
}

impl Report {
    fn new(
        p50: Duration,
        p95: Duration,
        p99: Duration,
        samples: usize,
        min: Duration,
        max: Duration,
        sum: Duration,
    ) -> Self {
        Self {
            p50,
            p95,
            p99,
            count: samples,
            min,
            max,
            sum,
        }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Report {
            p50,
            p95,
            p99,
            count,
            min,
            max,
            sum,
        } = self;

        let avg = if *count == 0 {
            Duration::default()
        } else {
            Duration::from_nanos(
                u64::try_from((sum.as_nanos()) / (*count as u128)).unwrap_or_default(),
            )
        };

        write!(f, "count: {count}, min: {min:?}, max: {max:?}, avg: {avg:?}, 50th: {p50:?}, 95th: {p95:?}, 99th: {p99:?}")
    }
}

impl AddAssign<&Report> for Report {
    fn add_assign(&mut self, rhs: &Report) {
        self.p50 += rhs.p50;
        self.p95 += rhs.p95;
        self.p99 += rhs.p99;
        self.count += rhs.count;
        self.min = self.min.min(rhs.min);
        self.max = self.max.max(rhs.max);
        self.sum += rhs.sum;
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
                    ty: LineType::Message,
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
                    ty: LineType::Report,
                    data: report,
                })?;
            }
            OutputKind::Normal => {
                writeln!(self.out, "{report}")?;
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
    #[serde(rename = "type")]
    ty: LineType,
    data: T,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
enum LineType {
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
