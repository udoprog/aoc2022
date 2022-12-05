use core::ops::Range;
use std::{fs::File, io::Read};

use anyhow::{anyhow, Context};

use crate::cli::error::LineCol;
use crate::input::IStr;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Size(usize);

impl Size {
    /// Default zero value.
    pub(crate) const ZERO: Self = Self(0);

    #[inline]
    pub(crate) fn usize_range(range: Range<Size>) -> Range<usize> {
        range.start.0..range.end.0
    }

    #[inline]
    pub(crate) fn new(n: usize) -> Self {
        Self(n)
    }

    #[inline]
    pub(crate) fn checked_add(self, b: Size) -> Option<Self> {
        Some(Self(self.0.checked_add(b.0)?))
    }

    #[inline]
    pub(crate) fn advance(&mut self, n: usize) {
        self.0 = self.0.saturating_add(n);
    }

    #[inline]
    pub(crate) fn saturating_add(self, n: Size) -> Self {
        Self(self.0.saturating_add(n.0))
    }
}

/// Get the current input position based on the given index.
pub(crate) fn pos_from(data: &[u8], span: Range<Size>) -> LineCol {
    use crate::input::NL;

    let span = Size::usize_range(span);

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

    LineCol::new(line, start, end)
}

/// Input processing.
#[inline]
pub fn input(
    path: &'static str,
    read_path: &str,
    storage: &'static mut Vec<u8>,
) -> anyhow::Result<IStr> {
    return inner(read_path, storage).with_context(|| anyhow!(path));

    #[inline]
    fn inner(read_path: &str, storage: &'static mut Vec<u8>) -> anyhow::Result<IStr> {
        let mut file = File::open(read_path)?;
        let mut buf = Vec::with_capacity(4096);
        file.read_to_end(&mut buf)?;
        *storage = buf;
        Ok(IStr::new(storage, Size::ZERO))
    }
}

/// Prepare an input processor.
///
/// This declares static storage for the processed input because it's much
/// easier to deal with than lifetimes and memory for it will be freed once the
/// process exists *anyway*.
#[macro_export]
macro_rules! input {
    ($path:literal) => {
        $crate::input!($path, 8192)
    };

    ($path:literal, $buf:literal) => {{
        static mut STORAGE: Vec<u8> = Vec::new();
        let path = concat!("inputs/", $path);
        let read_path = concat!(env!("CARGO_MANIFEST_DIR"), "/inputs/", $path);

        (
            $crate::env::input(path, read_path, unsafe { &mut STORAGE })?,
            path,
        )
    }};
}
