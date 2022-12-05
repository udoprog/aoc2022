//! CLI helpers.

mod bencher;
pub(crate) mod error;
mod output;
mod output_eq;
mod stdout_logger;

use core::fmt;
use core::ops::AddAssign;
use core::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};

pub use self::bencher::Bencher;
pub use self::error::CliError;
pub(self) use self::output::{Output, OutputKind};
pub use self::output_eq::OutputEq;

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
    time_limit: Option<u64>,
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
                "--time-limit" => {
                    let time_limit = it.next().context("missing argument to `--time-limit`")?;
                    let time_limit = time_limit
                        .to_str()
                        .context("missing string argument to `--time-limit`")?;
                    opts.time_limit = Some(
                        time_limit
                            .parse()
                            .context("bad argument to `--time-limit`")?,
                    );
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

        if !opts.json {
            log::set_max_level(log::LevelFilter::Info);
            log::set_logger(&STDOUT_LOGGER)
                .map_err(|error| anyhow!("failed to set log: {error}"))?;
        }

        Ok(opts)
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
    pub avg: Duration,
}

impl Report {
    fn new(
        p50: Duration,
        p95: Duration,
        p99: Duration,
        count: usize,
        min: Duration,
        max: Duration,
        sum: Duration,
    ) -> Self {
        let avg = if count == 0 {
            Duration::default()
        } else {
            Duration::from_nanos(
                u64::try_from((sum.as_nanos()) / (count as u128)).unwrap_or_default(),
            )
        };

        Self {
            p50,
            p95,
            p99,
            count,
            min,
            max,
            avg,
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
            avg,
        } = self;

        write!(f, "count: {count}, min: {min:?}, max: {max:?}, avg: {avg:?}, 50th: {p50:?}, 95th: {p95:?}, 99th: {p99:?}")
    }
}

struct Maybe<'a, T>(&'a Option<T>);

impl<T> fmt::Display for Maybe<'_, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(value) => value.fmt(f),
            None => "-".fmt(f),
        }
    }
}

impl AddAssign<&Report> for Report {
    fn add_assign(&mut self, rhs: &Report) {
        self.p50 += rhs.p50;
        self.p95 += rhs.p95;
        self.p99 += rhs.p99;
        self.count += rhs.count;
        self.min += rhs.min;
        self.max += rhs.max;
        self.avg += rhs.avg;
    }
}
