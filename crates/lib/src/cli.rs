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
pub use self::error::error_context;
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
    /// Number of iterations to run bench function.
    iter: Option<usize>,
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
                "--iter" => {
                    let iter = it.next().context("missing argument to `--iter`")?;
                    let iter = iter
                        .to_str()
                        .context("missing string argument to `--iter`")?;
                    opts.iter = Some(iter.parse().context("bad argument to `--iter`")?);
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

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct Percentiles {
    pub buckets: Vec<(u32, Duration)>,
}

impl Percentiles {
    /// Construct a new empty collection.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Check if percentiles is empty.
    pub(crate) fn is_empty(&self) -> bool {
        self.buckets.is_empty()
    }

    /// Insert a sample.
    pub(crate) fn insert(&mut self, p: u32, samples: &[Duration]) {
        let perc = (p as f32) / 10000f32;

        let index = ((samples.len() as f32) * perc) as usize;
        let value = samples.get(index).or(samples.last());

        if let Some(value) = value {
            self.buckets.push((p, *value));
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct Report {
    pub count: usize,
    pub min: Option<Duration>,
    pub max: Option<Duration>,
    pub avg: Duration,
    pub percentiles: Percentiles,
}

impl Report {
    fn new(
        count: usize,
        min: Option<Duration>,
        max: Option<Duration>,
        sum: Duration,
        percentiles: Percentiles,
    ) -> Self {
        let avg = sum.checked_div(count as u32).unwrap_or_default();

        Self {
            count,
            min,
            max,
            avg,
            percentiles,
        }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Report {
            count,
            min,
            max,
            avg,
            percentiles,
        } = self;

        let min = Maybe(min);
        let max = Maybe(max);

        writeln!(f, "count: {count}, min: {min}, max: {max}, avg: {avg:?}")?;

        let mut it = percentiles.buckets.iter();
        let last = it.next_back();

        for (n, value) in it {
            let rest = Rest(*n % 100);
            write!(f, "{}{rest}th: {:?}, ", n / 100, value)?;
        }

        if let Some((n, value)) = last {
            let rest = Rest(*n % 100);
            write!(f, "{}{rest}th: {:?}", n / 100, value)?;
        }

        return Ok(());

        struct Rest(u32);

        impl fmt::Display for Rest {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                if self.0 == 0 {
                    return Ok(());
                }

                let mut n = self.0;

                while n % 10 == 0 {
                    n /= 10;
                }

                write!(f, ".{}", n)
            }
        }

        struct Maybe<'a, T>(&'a Option<T>);

        impl<T> fmt::Display for Maybe<'_, T>
        where
            T: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                if let Some(value) = self.0 {
                    value.fmt(f)
                } else {
                    write!(f, "?")
                }
            }
        }
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
        self.count += rhs.count;
        self.min = self.min.and_then(|d| Some(d + rhs.min?)).or(rhs.min);
        self.max = self.max.and_then(|d| Some(d + rhs.max?)).or(rhs.max);
        self.avg += rhs.avg;

        if self.percentiles.is_empty() {
            self.percentiles = rhs.percentiles.clone();
        } else {
            for (to, from) in self
                .percentiles
                .buckets
                .iter_mut()
                .zip(&rhs.percentiles.buckets)
            {
                to.1 += from.1;
            }
        }
    }
}
