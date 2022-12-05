use core::fmt;
use std::{
    io::Write,
    time::{Duration, Instant},
};

use anyhow::{bail, Context, Error, Result};

use crate::cli::{Opts, Output, OutputEq, OutputKind, Report};

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
