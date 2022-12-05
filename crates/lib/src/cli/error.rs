use core::convert::Infallible;
use core::fmt;
use std::ops::Range;

use crate::input::{ErrorKind, IStrError, NL};

/// A line and column combination.
#[derive(Default, Debug, Clone, Copy)]
pub struct LineCol {
    line: usize,
    start: usize,
    end: usize,
}

impl LineCol {
    const EMPTY: Self = Self {
        line: 0,
        start: 0,
        end: 0,
    };
}

impl fmt::Display for LineCol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}-{}", self.line + 1, self.start, self.end)
    }
}

/// Need to be able to unwrap an error fully in case it's threaded through
/// multiple layers of processing.
fn find_cause(error: anyhow::Error) -> (ErrorKind, Range<usize>) {
    match error.downcast::<IStrError>() {
        Ok(e) => (e.kind, e.span),
        Err(e) => (ErrorKind::Boxed(e), 0..0),
    }
}

/// Get the current input position based on the given index.
pub fn pos_from(data: &[u8], span: Range<usize>) -> LineCol {
    let Some(d) = data.get(..=span.start) else {
        return LineCol::EMPTY;
    };

    let it = memchr::memchr_iter(NL, d);

    let (line, last) = it
        .enumerate()
        .last()
        .map(|(line, n)| (line + 1, n))
        .unwrap_or_default();

    let start = d.get(last.saturating_add(1)..).unwrap_or_default().len();

    let end = if let Some(end) = data.get(span) {
        let len = memchr::memchr(NL, end).unwrap_or(end.len());
        start.saturating_add(len)
    } else {
        start
    };

    LineCol { line, start, end }
}

/// Various forms of input errors.
#[derive(Debug)]
pub struct CliError {
    path: &'static str,
    pos: LineCol,
    kind: ErrorKind,
}

impl CliError {
    /// Construct a new input error from anyhow.
    pub fn anyhow(path: &'static str, pos: LineCol, error: anyhow::Error) -> Self {
        Self {
            path,
            pos,
            kind: ErrorKind::Boxed(error),
        }
    }

    pub fn cli(path: &'static str, data: &'static [u8], error: anyhow::Error) -> Self {
        let (kind, span) = find_cause(error);
        let pos = pos_from(data, span);

        Self {
            path,
            pos,
            kind: kind.into(),
        }
    }

    pub fn new(path: &'static str, pos: LineCol, kind: impl Into<ErrorKind>) -> Self {
        Self {
            path,
            pos,
            kind: kind.into(),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{path}:{pos}: {kind}",
            path = self.path,
            pos = self.pos,
            kind = self.kind
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CliError {}

impl From<Infallible> for CliError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}
