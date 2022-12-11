use core::fmt;
use std::ops::Range;

use crate::env::Size;
use crate::input::{IStr, IStrError};

/// Used in macros to associate context with an error.
#[doc(hidden)]
pub fn error_context<E>(path: &'static str, data: IStr, error: E) -> anyhow::Error
where
    anyhow::Error: From<E>,
{
    let error = error.into();
    let span = find_range(&error);
    let pos = crate::env::pos_from(data.as_data(), span);

    let cli_error = ErrorContext { path, pos };

    error.context(cli_error)
}

/// A line and column combination.
#[derive(Default, Debug, Clone, Copy)]
pub struct LineCol {
    line: usize,
    start: usize,
}

impl LineCol {
    pub(crate) const EMPTY: Self = Self::new(0, 0);

    pub(crate) const fn new(line: usize, start: usize) -> Self {
        Self { line, start }
    }
}

impl fmt::Display for LineCol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let line = self.line + 1;
        write!(f, "{line}:{}", self.start)
    }
}

/// Need to be able to unwrap an error fully in case it's threaded through
/// multiple layers of processing.
fn find_range(error: &anyhow::Error) -> Range<Size> {
    match error.downcast_ref::<IStrError>() {
        Some(e) => e.span.clone(),
        None => Size::ZERO..Size::ZERO,
    }
}

/// Various forms of input errors.
#[derive(Debug)]
struct ErrorContext {
    path: &'static str,
    pos: LineCol,
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{path}:{pos}", path = self.path, pos = self.pos,)
    }
}
