use core::convert::Infallible;
use core::fmt;
use std::ops::Range;

use crate::env::Size;
use crate::input::{IStr, IStrError};

#[derive(Debug)]
enum ErrorKind {
    IStr(crate::input::ErrorKind),
    Boxed(anyhow::Error),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::IStr(e) => e.fmt(f),
            ErrorKind::Boxed(e) => e.fmt(f),
        }
    }
}

/// A line and column combination.
#[derive(Default, Debug, Clone, Copy)]
pub struct LineCol {
    line: usize,
    start: usize,
    end: usize,
}

impl LineCol {
    pub(crate) const EMPTY: Self = Self::new(0, 0, 0);

    pub(crate) const fn new(line: usize, start: usize, end: usize) -> Self {
        Self { line, start, end }
    }
}

impl fmt::Display for LineCol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}-{}", self.line + 1, self.start, self.end)
    }
}

/// Need to be able to unwrap an error fully in case it's threaded through
/// multiple layers of processing.
fn find_cause(error: anyhow::Error) -> (ErrorKind, Range<Size>) {
    match error.downcast::<IStrError>() {
        Ok(e) => (ErrorKind::IStr(e.kind), e.span),
        Err(e) => (ErrorKind::Boxed(e), Size::ZERO..Size::ZERO),
    }
}

/// Various forms of input errors.
#[derive(Debug)]
pub struct CliError {
    path: &'static str,
    pos: LineCol,
    kind: ErrorKind,
}

impl CliError {
    /// Constructor used in macros.
    #[doc(hidden)]
    pub fn cli<E>(path: &'static str, data: IStr, error: E) -> Self
    where
        anyhow::Error: From<E>,
    {
        let (kind, span) = find_cause(error.into());
        let pos = crate::env::pos_from(data.as_data(), span);

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
