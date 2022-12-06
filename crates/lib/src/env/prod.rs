use core::fmt;
use core::ops::Range;

use crate::cli::error::LineCol;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Size;

impl Size {
    /// Default zero value.
    pub const ZERO: Self = Self;

    #[inline]
    pub(crate) fn new(_: usize) -> Self {
        Self
    }

    #[inline]
    pub(crate) fn checked_add(self, _: Size) -> Option<Self> {
        Some(Self)
    }

    #[inline]
    pub(crate) fn saturating_add(self, _: Size) -> Self {
        Self
    }

    #[inline]
    pub(crate) fn advance(&mut self, _: usize) -> Self {
        Self
    }
}

impl fmt::Debug for Size {
    #[inline]
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

pub fn pos_from(_: &[u8], _: Range<Size>) -> LineCol {
    LineCol::EMPTY
}

#[macro_export]
macro_rules! input {
    ($path:literal) => {
        $crate::input!($path, 8192)
    };

    ($path:literal, $_:literal) => {{
        (
            $crate::input::IStr::new(
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/inputs/", $path)),
                $crate::env::Size::ZERO,
            ),
            concat!("inputs/", $path),
        )
    }};
}
