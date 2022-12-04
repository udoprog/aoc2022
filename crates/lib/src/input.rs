//! Input parser.

use std::convert::Infallible;
use std::error;
use std::fmt;
use std::ops::Range;
use std::str::from_utf8;

type Result<T> = std::result::Result<T, InputError>;

const NL: u8 = b'\n';

/// Various forms of input errors.
#[derive(Debug)]
pub struct InputError {
    path: &'static str,
    pos: LineCol,
    kind: ErrorKind,
}

impl InputError {
    /// Construct a new input error from anyhow.
    pub fn anyhow(path: &'static str, pos: LineCol, error: anyhow::Error) -> Self {
        Self {
            path,
            pos,
            kind: ErrorKind::Boxed(error),
        }
    }

    fn new(path: &'static str, pos: LineCol, kind: ErrorKind) -> Self {
        Self { path, pos, kind }
    }
}

impl fmt::Display for InputError {
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

impl error::Error for InputError {}

impl From<Infallible> for InputError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

#[derive(Debug)]
enum ErrorKind {
    NotInteger,
    NotFloat,
    NotChar,
    NotLine,
    NotUtf8,
    MissingSplit(u8),
    ArrayCapacity(usize),
    Boxed(anyhow::Error),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::NotInteger => write!(f, "not an integer or integer overflow"),
            ErrorKind::NotFloat => write!(f, "not a float"),
            ErrorKind::NotChar => write!(f, "not a character"),
            ErrorKind::NotLine => write!(f, "not a line"),
            ErrorKind::NotUtf8 => write!(f, "not utf-8"),
            ErrorKind::MissingSplit(b) => {
                if b.is_ascii_control() {
                    write!(f, "missing split on byte {b:?}")
                } else {
                    let b = *b as char;
                    write!(f, "missing split on byte {b:?}")
                }
            }
            ErrorKind::ArrayCapacity(cap) => write!(f, "array out of capacity ({cap})"),
            ErrorKind::Boxed(error) => error.fmt(f),
        }
    }
}

/// A line and column combination.
#[derive(Default, Debug, Clone, Copy)]
pub struct LineCol {
    line: usize,
    column: usize,
}

impl LineCol {
    const EMPTY: Self = Self { line: 0, column: 0 };
}

impl fmt::Display for LineCol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.column)
    }
}

/// Helper to parse input.
#[derive(Debug, Clone)]
pub struct Input {
    /// Path being parsed.
    path: &'static str,
    /// The path being parsed.
    data: &'static [u8],
    /// Index into the current slice.
    index: usize,
    /// Index being read.
    range: Range<usize>,
}

impl Input {
    /// Construct a new input processor.
    #[doc(hidden)]
    pub fn new(path: &'static str, data: &'static [u8]) -> Self {
        Self {
            path,
            data,
            index: 0,
            range: 0..data.len(),
        }
    }

    /// Get current index.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Reset input.
    pub fn reset(&mut self) {
        self.index = self.range.start;
    }

    /// Get remaining bytes the input.
    pub fn as_bytes(&self) -> &'static [u8] {
        self.data
            .get(self.index..self.range.end)
            .unwrap_or_default()
    }

    /// Get remaining binary string of the input.
    pub fn as_bstr(&self) -> &bstr::BStr {
        bstr::BStr::new(self.as_bytes())
    }

    /// Get the current input path.
    pub fn path(&self) -> &'static str {
        self.path
    }

    /// Get the current line column position.
    #[inline]
    pub fn pos(&self) -> LineCol {
        self.pos_of(self.index)
    }

    /// Split input on the given byte.
    pub fn split(&mut self, b: u8) -> Option<(Input, Input)> {
        let data = self.data.get(self.index..self.range.end)?;
        let n = memchr::memchr(b, data)?;

        let end = self.index.checked_add(n)?;

        let a = Input {
            path: self.path,
            data: self.data,
            range: self.index..end,
            index: self.index,
        };

        let index = end.checked_add(1)?.min(self.range.end);

        let b = Input {
            path: self.path,
            data: self.data,
            range: index..self.range.end,
            index,
        };

        Some((a, b))
    }

    /// Get the current input position based on the given index.
    pub fn pos_of(&self, index: usize) -> LineCol {
        let Some(data) = self.data.get(..=index) else {
            return LineCol::EMPTY;
        };

        let it = memchr::memchr_iter(NL, data);
        let (line, last) = it
            .enumerate()
            .last()
            .map(|(line, n)| (line + 1, n))
            .unwrap_or_default();

        LineCol {
            line,
            column: data.get(last.saturating_add(1)..).unwrap_or_default().len(),
        }
    }

    /// Parse the next value as T.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn next<T>(&mut self) -> Result<T>
    where
        T: FromInput,
    {
        T::from_input_whitespace(self)
    }

    /// Try parse the next value as `T`, returns `None` if there is no more
    /// non-whitespace data to process.
    #[inline]
    pub fn try_next<T>(&mut self) -> Result<Option<T>>
    where
        T: FromInput,
    {
        T::try_from_input(self)
    }

    /// Parse the next value as `T`, errors with `Err(InputError)` if the next
    /// element is not a valid value of type `T`.
    #[inline]
    pub fn line<T>(&mut self) -> Result<T>
    where
        T: FromInput,
    {
        let Nl(mut line) = self.next()?;
        line.next()
    }

    /// Parse the next value as `T`, errors with `Err(InputError)` if the next
    /// element is not a valid value of type `T`, returns `Ok(None)` if there is
    /// no more non-whitespace data to process.
    #[inline]
    pub fn try_line<T>(&mut self) -> Result<Option<T>>
    where
        T: FromInput,
    {
        let Some(Nl(mut line)) = self.try_next()? else {
            return Ok(None);
        };

        Ok(Some(line.next()?))
    }

    /// Skip whitespace and return the number of lines skipped.
    fn skip_whitespace(&mut self) -> usize {
        let start = self.index;
        self.consume_whitespace();
        let data = self.data.get(start..self.index).unwrap_or_default();
        memchr::memchr_iter(NL, data).count()
    }

    /// Get the next line of input.
    #[inline]
    fn until(&mut self, b: u8) -> Option<Range<usize>> {
        let data = self.data.get(self.index..self.range.end)?;

        let Some(at) = memchr::memchr(b, data) else {
            let start = std::mem::replace(&mut self.index, self.range.end);
            return Some(start..self.range.end);
        };

        let end = self.index.saturating_add(at);
        let new_index = end.checked_add(1)?.min(self.range.end);
        let start = std::mem::replace(&mut self.index, new_index);
        Some(start..end)
    }

    /// Consume whitespace.
    fn peek_whitespace(&mut self) -> usize {
        let mut n = 0;

        while let Some(c) = self.peek_from(n) {
            if !c.is_ascii_whitespace() {
                break;
            }

            n = n.checked_add(1).expect("cursor overflow");
        }

        n
    }

    /// Consume whitespace.
    fn consume_whitespace(&mut self) {
        let n = self.peek_whitespace();
        self.advance(n);
    }

    /// Get the byte at the given reader offset.
    fn peek_from(&self, n: usize) -> Option<u8> {
        let n = self.index.checked_add(n)?.min(self.range.end);

        if n >= self.range.end {
            return None;
        }

        self.data.get(n).copied()
    }

    /// Get the byte at the given reader offset.
    #[inline]
    fn peek(&self) -> Option<u8> {
        self.peek_from(0)
    }

    #[inline]
    fn advance(&mut self, n: usize) {
        if n == 0 {
            return;
        }

        self.index = self.index.saturating_add(n).min(self.range.end);
    }

    /// Step the buffer.
    fn step(&mut self) {
        self.index = self.index.saturating_add(1).min(self.range.end);
    }
}

/// A value that can be parsed from input.
pub trait FromInput: Sized {
    /// Optionally try to confuse input ignoring leading whitespace by default.
    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
        let n = p.peek_whitespace();

        if p.peek_from(n).is_none() {
            return Ok(None);
        }

        p.advance(n);
        Ok(Some(Self::from_input(p)?))
    }

    /// From input before whitespace stripping.
    #[inline]
    fn from_input_whitespace(p: &mut Input) -> Result<Self> {
        p.consume_whitespace();
        Self::from_input(p)
    }

    /// Parse a value from a given input.
    fn from_input(p: &mut Input) -> Result<Self>;
}

macro_rules! tuple {
    ($first:ident $first_id:ident $(, $rest:ident $rest_id:ident)* $(,)?) => {
        impl<$first, $($rest,)*> FromInput for ($first, $($rest, )*)
        where
            $first: FromInput,
            $($rest: FromInput,)*
        {
            #[inline]
            fn from_input(p: &mut Input) -> Result<Self> {
                let $first_id = p.next::<$first>()?;
                $(let $rest_id = p.next::<$rest>()?;)*
                Ok(($first_id, $($rest_id,)*))
            }
        }
    }
}

#[rustfmt::skip]
macro_rules! integer {
    ($ty:ty, $error:ident) => {
        impl FromInput for $ty {
            fn from_input(p: &mut Input) -> Result<Self> {
                let pos = p.index;
                let string: &str = FromInput::from_input(p)?;

                let Ok(n) = str::parse(string) else {
                    let pos = p.pos_of(pos);
                    return Err(InputError::new(p.path, pos, ErrorKind::$error));
                };

                Ok(n)
            }
        }
    };
}

tuple!(A a);
tuple!(A a, B b);
tuple!(A a, B b, C c);
tuple!(A a, B b, C c, D d);

integer!(u8, NotInteger);
integer!(u16, NotInteger);
integer!(u32, NotInteger);
integer!(u64, NotInteger);
integer!(u128, NotInteger);
integer!(i8, NotInteger);
integer!(i16, NotInteger);
integer!(i32, NotInteger);
integer!(i64, NotInteger);
integer!(i128, NotInteger);
integer!(f32, NotFloat);
integer!(f64, NotFloat);
integer!(num_bigint::BigInt, NotInteger);
integer!(num_bigint::BigUint, NotInteger);

impl FromInput for char {
    #[inline]
    fn from_input(p: &mut Input) -> Result<Self> {
        use bstr::ByteSlice;

        let pos = p.index;

        let Some(c) = p.data.get(p.index..).and_then(|b| b.chars().next()) else {
            let pos = p.pos_of(pos);
            return Err(InputError::new(p.path, pos, ErrorKind::NotChar));
        };

        p.advance(c.len_utf8());
        Ok(c)
    }
}

impl FromInput for Input {
    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
        if p.peek().is_none() {
            return Ok(None);
        }

        Ok(Some(Self::from_input(p)?))
    }

    #[inline]
    fn from_input_whitespace(p: &mut Input) -> Result<Self> {
        Self::from_input(p)
    }

    #[inline]
    fn from_input(p: &mut Input) -> Result<Self> {
        Ok(p.clone())
    }
}

impl FromInput for &[u8] {
    #[inline]
    fn from_input(p: &mut Input) -> Result<Self> {
        let start = p.index;

        while let Some(c) = p.peek() {
            if c.is_ascii_whitespace() {
                break;
            }

            p.step();
        }

        let data = p.data.get(start..p.index).unwrap_or_default();
        Ok(data)
    }
}

impl FromInput for &str {
    #[inline]
    fn from_input(p: &mut Input) -> Result<Self> {
        let data = <&[u8]>::from_input(p)?;

        let Ok(data) = from_utf8(data) else {
            return Err(InputError::new(p.path, p.pos(), ErrorKind::NotUtf8));
        };

        Ok(data)
    }
}

impl FromInput for &bstr::BStr {
    #[inline]
    fn from_input(p: &mut Input) -> Result<Self> {
        let data = <&[u8]>::from_input(p)?;
        Ok(bstr::BStr::new(data))
    }
}

/// Parse until end of line.
pub struct Nl(Input);

impl FromInput for Nl {
    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
        if p.peek().is_none() {
            return Ok(None);
        }

        Ok(Some(Self::from_input(p)?))
    }

    #[inline]
    fn from_input_whitespace(p: &mut Input) -> Result<Self> {
        Self::from_input(p)
    }

    #[inline]
    fn from_input(p: &mut Input) -> Result<Self> {
        let pos = p.index;

        let Some(range) = p.until(NL) else {
            let pos = p.pos_of(pos);
            return Err(InputError::new(p.path, pos, ErrorKind::NotLine));
        };

        Ok(Self(Input {
            path: p.path,
            data: p.data,
            index: range.start,
            range,
        }))
    }
}

/// Consume whitespace and return the number of lines consumed.
pub struct Ws(pub usize);

impl FromInput for Ws {
    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
        Ok(Some(Self::from_input(p)?))
    }

    #[inline]
    fn from_input_whitespace(p: &mut Input) -> Result<Self> {
        Self::from_input(p)
    }

    #[inline]
    fn from_input(p: &mut Input) -> Result<Self> {
        Ok(Self(p.skip_whitespace()))
    }
}

impl<T, const N: usize> FromInput for arrayvec::ArrayVec<T, N>
where
    T: FromInput,
{
    #[inline]
    fn from_input(p: &mut Input) -> Result<Self> {
        let mut output = arrayvec::ArrayVec::new();
        let mut pos = p.index;

        while let Some(element) = T::try_from_input(p)? {
            if output.remaining_capacity() == 0 {
                let pos = p.pos_of(pos);
                return Err(InputError::new(p.path, pos, ErrorKind::ArrayCapacity(N)));
            }

            output.push(element);
            pos = p.index;
        }

        Ok(output)
    }
}

impl<T> FromInput for Vec<T>
where
    T: FromInput,
{
    #[inline]
    fn from_input(p: &mut Input) -> Result<Self> {
        let mut output = Vec::new();

        while let Some(element) = T::try_from_input(p)? {
            output.push(element);
        }

        Ok(output)
    }
}

/// Split once on byte `D`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Split<const D: u8, A, B>(pub A, pub B);

impl<const D: u8, A, B> FromInput for Split<D, A, B>
where
    A: FromInput,
    B: FromInput,
{
    #[inline]
    fn from_input(p: &mut Input) -> Result<Self> {
        let pos = p.index;

        let Some((mut a_in, mut b_in)) = p.split(D) else {
            let pos = p.pos_of(pos);
            return Err(InputError::new(p.path, pos, ErrorKind::MissingSplit(D)));
        };

        let a = A::from_input(&mut a_in)?;
        let b = B::from_input(&mut b_in)?;
        p.index = b_in.index.min(p.range.end);
        Ok(Self(a, b))
    }
}
