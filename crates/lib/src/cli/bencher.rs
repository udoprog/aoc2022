use core::fmt;
use std::{
    io::Write,
    time::{Duration, Instant},
};

use anyhow::{bail, Context, Error, Result};

use crate::cli::{Opts, Output, OutputEq, OutputKind, Report};

use super::Percentiles;

/// Default warmup period in seconds.
const DEFAULT_WARMUP: u64 = 100;

/// Default time in seconds.
const DEFAULT_TIME_LIMIT: u64 = 400;

/// At 10 microsecond runtime we need to adjust our timing method.
const THRESHOLD: u32 = 10_000_000;

pub struct Bencher {
    iter: Option<usize>,
    kind: OutputKind,
    warmup: Duration,
    time_limit: Duration,
}

impl Bencher {
    /// Construct a new bencher.
    #[inline]
    pub fn new(opts: &Opts) -> Self {
        let warmup = Duration::from_millis(opts.warmup.unwrap_or(DEFAULT_WARMUP));
        let time_limit = Duration::from_millis(opts.time_limit.unwrap_or(DEFAULT_TIME_LIMIT));

        Self {
            iter: opts.iter,
            kind: if opts.json {
                OutputKind::Json
            } else {
                OutputKind::Normal
            },
            warmup,
            time_limit,
        }
    }

    /// Bench the given fn.
    pub fn run<T, O, C, E>(&self, expected: Option<C>, iter: T) -> Result<()>
    where
        T: FnMut() -> Result<O, E>,
        O: fmt::Debug + OutputEq<C>,
        C: fmt::Debug,
        Error: From<E>,
    {
        let stdout = std::io::stdout();
        let mut o = Output::new(stdout.lock(), self.kind);

        if let Err(e) = self.inner_run(&mut o, expected, iter) {
            o.error(e)?;
        }

        Ok(())
    }

    fn inner_run<T, O, C, E>(
        &self,
        o: &mut Output<impl Write>,
        expected: Option<C>,
        mut f: T,
    ) -> Result<()>
    where
        T: FnMut() -> Result<O, E>,
        O: fmt::Debug + OutputEq<C>,
        C: fmt::Debug,
        Error: From<E>,
    {
        let before = Instant::now();
        let value = f()?;

        // run once to check against expected.
        if let Some(expect) = &expected {
            if !value.output_eq(expect) {
                bail!("{value:?} (value) != {expect:?} (expected)");
            }
        }

        let _ = black_box(value);

        let iter = match self.iter {
            Some(iter) => iter,
            None => {
                let duration = before.elapsed();

                if duration.as_secs() == 0 && duration.subsec_nanos() <= THRESHOLD {
                    (THRESHOLD / duration.subsec_nanos()) as usize
                } else {
                    1
                }
            }
        };

        if !self.warmup.is_zero() {
            let start = Instant::now();

            o.info(format_args!("warming up ({:?})...", self.warmup))?;

            loop {
                black_box(f()?);

                if start.elapsed() >= self.warmup {
                    break;
                }
            }
        }

        let mut samples = Vec::new();

        o.info(format_args!("running benches ({:?})...", self.time_limit))?;

        let start = Instant::now();

        loop {
            let before = Instant::now();

            for _ in 0..iter {
                black_box(f()?);
            }

            let now = Instant::now();
            samples.push(now.duration_since(before));

            if now.duration_since(start) >= self.time_limit {
                break;
            }
        }

        samples.sort();

        let sum = samples.iter().copied().sum::<Duration>();

        for sample in &mut samples {
            *sample = sample.checked_div(iter as u32).context("zero division")?;
        }

        let mut percentiles = Percentiles::new();
        percentiles.insert(2500, &samples);
        percentiles.insert(5000, &samples);
        percentiles.insert(9500, &samples);
        percentiles.insert(9900, &samples);

        let min = samples.first().copied();
        let max = samples.last().copied();

        let report = Report::new(samples.len() * iter, min, max, sum, percentiles);
        o.report(&report)?;
        Ok(())
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
