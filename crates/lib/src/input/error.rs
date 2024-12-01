use core::fmt;
use core::ops::Range;
use std::num::ParseIntError;

use bstr::BStr;
use num::bigint::ParseBigIntError;

use crate::env::Size;

macro_rules! bails {
    ($vis:vis enum $name:ident { $($variant:ident($ty:ty)),* $(,)? }) => {
        #[derive(Debug, Clone, Copy)]
        $vis enum $name {
            $($variant($ty),)*
        }

        $(
            impl From<$ty> for ErrorKind {
                #[inline]
                fn from(value: $ty) -> ErrorKind {
                    ErrorKind::Custom($name::$variant(value))
                }
            }

            impl From<$ty> for $name {
                #[inline]
                fn from(value: $ty) -> $name {
                    $name::$variant(value)
                }
            }
        )*
    }
}

bails! {
    pub enum Custom {
        Char(char),
        Str(&'static str),
        BStr(&'static BStr),
        Bytes(&'static [u8]),
        Usize(usize),
        Isize(isize),
        U8(u8),
        U16(u16),
        U32(u32),
        U64(u64),
        U128(u128),
        I8(i8),
        I16(i16),
        I32(i32),
        I64(i64),
        I128(i128),
    }
}

impl fmt::Display for Custom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Custom::Char(c) => write!(f, "{c:?}"),
            Custom::Str(string) => write!(f, "{string:?}"),
            Custom::BStr(string) => write!(f, "{string:?}"),
            Custom::Bytes(string) => write!(f, "{string:?}", string = BStr::new(string)),
            Custom::Usize(n) => n.fmt(f),
            Custom::Isize(n) => n.fmt(f),
            Custom::U8(n) => n.fmt(f),
            Custom::U16(n) => n.fmt(f),
            Custom::U32(n) => n.fmt(f),
            Custom::U64(n) => n.fmt(f),
            Custom::U128(n) => n.fmt(f),
            Custom::I8(n) => n.fmt(f),
            Custom::I16(n) => n.fmt(f),
            Custom::I32(n) => n.fmt(f),
            Custom::I64(n) => n.fmt(f),
            Custom::I128(n) => n.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ErrorKind {
    NotInteger(&'static str),
    NotFloat(&'static str),
    NotUtf8,
    BadArray(usize, usize),
    ExpectedChar,
    ExpectedLine,
    ExpectedTuple(usize),
    NotByteMuck,
    UnexpectedEof,
    StringCapacity(usize),
    ArrayCapacity(usize),
    RingbufCapacity(usize),
    ParseIntError(ParseIntError),
    ParseBigIntError(ParseBigIntError),
    Custom(Custom),
    Condition(&'static str, Option<Custom>),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::NotInteger(n) => write!(f, "not an integer or integer overflow `{n}`"),
            ErrorKind::NotFloat(n) => write!(f, "not a float `{n}`"),
            ErrorKind::NotUtf8 => write!(f, "not utf-8"),
            ErrorKind::BadArray(expected, actual) => {
                write!(f, "bad array; expected {expected}, but got {actual}")
            }
            ErrorKind::ExpectedChar => write!(f, "exptected charater"),
            ErrorKind::ExpectedLine => write!(f, "expected line"),
            ErrorKind::UnexpectedEof => write!(f, "unexpected eof"),
            ErrorKind::ExpectedTuple(n) => write!(f, "expected tuple of length `{n}`"),
            ErrorKind::NotByteMuck => write!(f, "not a valid number muck"),
            ErrorKind::StringCapacity(cap) => write!(f, "string out of capacity ({cap})"),
            ErrorKind::ArrayCapacity(cap) => write!(f, "array out of capacity ({cap})"),
            ErrorKind::RingbufCapacity(cap) => write!(f, "ringbuf out of capacity ({cap})"),
            ErrorKind::ParseIntError(e) => write!(f, "{e}"),
            ErrorKind::ParseBigIntError(e) => write!(f, "{e}"),
            ErrorKind::Custom(c) => write!(f, "custom: {c}"),
            ErrorKind::Condition(condition, custom) => {
                if let Some(custom) = custom {
                    write!(f, "condition `{condition}` failed: {custom}")
                } else {
                    write!(f, "condition `{condition}` failed")
                }
            }
        }
    }
}

impl std::error::Error for ErrorKind {}

impl From<ParseIntError> for ErrorKind {
    #[inline]
    fn from(error: ParseIntError) -> Self {
        Self::ParseIntError(error)
    }
}

impl From<ParseBigIntError> for ErrorKind {
    #[inline]
    fn from(error: ParseBigIntError) -> Self {
        Self::ParseBigIntError(error)
    }
}

/// Error raised through string processing.
#[derive(Debug)]
pub struct IStrError {
    pub(crate) span: Range<Size>,
    pub(crate) kind: ErrorKind,
}

impl IStrError {
    /// Construct a new input error.
    #[inline]
    pub fn new(span: Range<Size>, kind: ErrorKind) -> Self {
        Self { span, kind }
    }

    #[inline]
    pub fn kind(self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Display for IStrError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (at {:?})", self.kind, self.span)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for IStrError {}
