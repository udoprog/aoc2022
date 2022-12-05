use core::convert::Infallible;
use core::fmt;

use crate::input::{IStrError, LineCol};

/// Various forms of input errors.
#[derive(Debug)]
pub struct CliError {
    path: &'static str,
    pos: LineCol,
    kind: IStrError,
}

impl CliError {
    /// Construct a new input error from anyhow.
    pub fn anyhow(path: &'static str, pos: LineCol, error: anyhow::Error) -> Self {
        Self {
            path,
            pos,
            kind: IStrError::Boxed(error),
        }
    }

    pub fn new(path: &'static str, pos: LineCol, kind: impl Into<IStrError>) -> Self {
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
