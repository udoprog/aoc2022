use core::fmt;
use std::{
    io::Write,
    time::{Duration, Instant},
};

use anyhow::{bail, Error, Result};

use crate::cli::{Opts, Output, OutputEq, OutputKind, Report};

use super::Percentiles;

/// Default warmup period in seconds.
const DEFAULT_WARMUP: u64 = 100;

/// Default time in seconds.
const DEFAULT_TIME_LIMIT: u64 = 400;

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
    pub fn iter<T, O, C, E>(&mut self, opts: &Opts, expected: Option<C>, iter: T) -> Result<()>
    where
        T: FnMut() -> Result<O, E>,
        O: fmt::Debug + OutputEq<C>,
        C: fmt::Debug,
        Error: From<E>,
    {
        let stdout = std::io::stdout();

        let mut o = Output::new(
            stdout.lock(),
            if opts.json {
                OutputKind::Json
            } else {
                OutputKind::Normal
            },
        );

        if let Err(e) = self.inner_iter(&mut o, opts, expected, iter) {
            o.error(e)?;
        }

        Ok(())
    }

    fn inner_iter<T, O, C, E>(
        &mut self,
        o: &mut Output<impl Write>,
        opts: &Opts,
        expected: Option<C>,
        mut iter: T,
    ) -> Result<()>
    where
        T: FnMut() -> Result<O, E>,
        O: fmt::Debug + OutputEq<C>,
        C: fmt::Debug,
        Error: From<E>,
    {
        let warmup = Duration::from_millis(opts.warmup.unwrap_or(DEFAULT_WARMUP));
        let time_limit = Duration::from_millis(opts.time_limit.unwrap_or(DEFAULT_TIME_LIMIT));

        if !warmup.is_zero() {
            let s = Instant::now();

            o.info(format_args!("warming up ({warmup:?})..."))?;

            loop {
                let value = iter()?;
                let after = Instant::now();

                if let Some(expect) = &expected {
                    if !value.output_eq(expect) {
                        bail!("{value:?} (value) != {expect:?} (expected)");
                    }
                }

                let _ = black_box(value);

                if after.duration_since(s) >= warmup {
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
                let before = Instant::now();
                let value = iter()?;
                let after = Instant::now();

                if let Some(expect) = &expected {
                    if !value.output_eq(expect) {
                        bail!("{value:?} (value) != {expect:?} (expected)");
                    }
                }

                let _ = black_box(value);
                samples.push(after.duration_since(before));
            }
        } else {
            o.info(format_args!("running benches ({time_limit:?})..."))?;

            let start = Instant::now();

            loop {
                let before = Instant::now();
                let value = iter()?;
                let after = Instant::now();

                if let Some(expect) = &expected {
                    if !value.output_eq(expect) {
                        bail!("{value:?} (value) != {expect:?} (expected)");
                    }
                }

                let _ = black_box(value);
                samples.push(after.duration_since(before));

                if after.duration_since(start) >= time_limit {
                    break;
                }
            }
        }

        samples.sort();

        let mut percentiles = Percentiles::new();
        percentiles.insert(2500, &samples);
        percentiles.insert(5000, &samples);
        percentiles.insert(9500, &samples);
        percentiles.insert(9900, &samples);
        percentiles.insert(9999, &samples);

        let min = samples.first().copied();
        let max = samples.last().copied();
        let sum = samples.iter().copied().sum();
        let report = Report::new(samples.len(), min, max, sum, percentiles);
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
