//! Input parser.

mod iter;
pub mod muck;

use std::convert::Infallible;
use std::error;
use std::fmt;
use std::ops;
use std::str::from_utf8;

use arrayvec::ArrayVec;
use bstr::BStr;

pub use self::iter::Iter;

pub(self) type Result<T> = std::result::Result<T, InputError>;

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
#[non_exhaustive]
pub enum ErrorKind {
    NotInteger(&'static str),
    NotFloat(&'static str),
    NotUtf8,
    BadArray,
    ExpectedChar,
    ExpectedLine,
    ExpectedTuple(usize),
    NotByteMuck,
    UnexpectedEof,
    ArrayCapacity(usize),
    Boxed(anyhow::Error),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::NotInteger(n) => write!(f, "not an integer or integer overflow `{n}`"),
            ErrorKind::NotFloat(n) => write!(f, "not a float `{n}`"),
            ErrorKind::NotUtf8 => write!(f, "not utf-8"),
            ErrorKind::BadArray => write!(f, "bad array"),
            ErrorKind::ExpectedChar => write!(f, "exptected charater"),
            ErrorKind::ExpectedLine => write!(f, "bad line"),
            ErrorKind::UnexpectedEof => write!(f, "unexpected eof"),
            ErrorKind::ExpectedTuple(n) => write!(f, "expected tuple of length `{n}`"),
            ErrorKind::NotByteMuck => write!(f, "not a valid number muck"),
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
    range: ops::Range<usize>,
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

    /// Cosntruct an iterator over the current input.
    pub fn iter<T>(&mut self) -> Iter<'_, T> {
        Iter::new(self)
    }

    /// Test if input is empty.
    pub fn is_empty(&self) -> bool {
        debug_assert!(self.range.end >= self.index);
        self.index == self.range.end
    }

    /// Get the length of the current input.
    pub fn len(&self) -> usize {
        debug_assert!(self.range.end >= self.index);
        self.range.end.saturating_sub(self.index)
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
    pub fn as_bstr(&self) -> &BStr {
        BStr::new(self.as_bytes())
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
    pub fn split_once(&self, b: u8) -> Option<(Input, Input)> {
        let data = self.data.get(self.index..self.range.end)?;
        let n = memchr::memchr(b, data)?;

        let end = self.index.checked_add(n)?;

        let a = self.slice(self.index..end);
        let index = end.checked_add(1)?.min(self.range.end);
        let b = self.slice(index..self.range.end);
        Some((a, b))
    }

    /// Split `N` times.
    pub fn splitn(&self, byte: u8) -> impl InputIterator {
        return Iterator {
            byte,
            last: self.index,
            current: Some(self.clone()),
        };

        struct Iterator {
            byte: u8,
            last: usize,
            current: Option<Input>,
        }

        impl InputIterator for Iterator {
            /// End index for the last item iterated over.
            fn index(&self) -> usize {
                self.last
            }

            #[inline]
            fn try_next(&mut self) -> Option<Input> {
                let current = self.current.take()?;

                let Some((input, next)) = current.split_once(self.byte) else {
                    self.last = current.range.end;
                    return Some(current);
                };

                self.last = next.range.end;
                self.current = Some(next);
                Some(input)
            }
        }
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
        T::from_input(self)
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
        let Nl(mut line) = self.next::<Nl<Input>>()?;
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
        let Some(Nl(mut line)) = self.try_next::<Nl<Input>>()? else {
            return Ok(None);
        };

        let Some(output) = line.try_next()? else {
            // NB: Restore index if line doesn't parse.
            self.index = line.index;
            return Ok(None);
        };

        Ok(Some(output))
    }

    /// Shorthand for using [Ws] to scan newlines.
    pub fn ws(&mut self) -> Result<usize> {
        let Ws(n) = self.next::<Ws>()?;
        Ok(n)
    }

    /// Get the next line of input.
    #[inline]
    fn until(&mut self, b: u8) -> Option<ops::Range<usize>> {
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

    /// Get the byte at the given reader offset.
    fn at(&self, n: usize) -> Option<u8> {
        if n >= self.range.end {
            return None;
        }

        self.data.get(n).copied()
    }

    /// Get the byte at the given reader offset.
    #[inline]
    fn peek(&self) -> Option<u8> {
        self.at(self.index)
    }

    #[inline]
    fn advance(&mut self, n: usize) {
        if n == 0 {
            return;
        }

        self.index = self.index.saturating_add(n).min(self.range.end);
    }

    /// Construct a sub-range.
    #[inline]
    fn slice(&self, range: ops::Range<usize>) -> Input {
        Self {
            path: self.path,
            data: self.data,
            index: range.start,
            range,
        }
    }
}

/// A value that can be parsed from input.
pub trait FromInput: Sized {
    /// Custom error kind to use.
    #[inline]
    fn error_kind() -> ErrorKind {
        ErrorKind::UnexpectedEof
    }

    /// Optionally try to confuse input ignoring leading whitespace by default.
    fn try_from_input(p: &mut Input) -> Result<Option<Self>>;

    /// Parse a value from a given input.
    fn from_input(p: &mut Input) -> Result<Self> {
        let start = p.index;

        let Some(value) = Self::try_from_input(p)? else {
            let pos = p.pos_of(start);
            return Err(InputError::new(p.path, pos, Self::error_kind()));
        };

        Ok(value)
    }
}

/// Iterator over inputs.
pub trait InputIterator {
    #[inline]
    fn error_kind() -> ErrorKind {
        ErrorKind::UnexpectedEof
    }

    /// Get tail index of iterator.
    fn index(&self) -> usize;

    /// Get next input.
    fn try_next(&mut self) -> Option<Input>;

    /// Require next input.
    fn next(&mut self, p: &mut Input) -> Result<Input> {
        let index = p.index;

        let Some(value) = Self::try_next(self) else {
            let pos = p.pos_of(index);
            return Err(InputError::new(p.path, pos, Self::error_kind()));
        };

        Ok(value)
    }
}

/// Parse something from a pair of inputs.
pub trait FromInputIter: Sized {
    /// Optionally try to confuse input ignoring leading whitespace by default.
    fn from_input_iter<I>(p: &mut Input, inputs: &mut I) -> Result<Option<Self>>
    where
        I: InputIterator;
}

macro_rules! tuple {
    ($num:literal => $first:ident $first_id:ident $(, $rest:ident $rest_id:ident)* $(,)?) => {
        impl<$first, $($rest,)*> FromInput for ($first, $($rest, )*)
        where
            $first: FromInput,
            $($rest: FromInput,)*
        {
            #[inline]
            fn error_kind() -> ErrorKind {
                ErrorKind::ExpectedTuple($num)
            }

            #[inline]
            fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
                let index = p.index;

                let Some($first_id) = p.try_next()? else {
                    p.index = index;
                    return Ok(None);
                };

                $(
                    let Some($rest_id) = p.try_next()? else {
                        p.index = index;
                        return Ok(None);
                    };
                )*

                Ok(Some(($first_id, $($rest_id,)*)))
            }
        }

        impl<$first, $($rest,)*> FromInputIter for ($first, $($rest,)*)
        where
            $first: FromInput,
            $($rest: FromInput,)*
        {
            #[inline]
            fn from_input_iter<I>(_: &mut Input, inputs: &mut I) -> Result<Option<Self>>
            where
                I: InputIterator
            {
                let Some(mut $first_id) = inputs.try_next() else {
                    return Ok(None);
                };

                $(
                    let Some(mut $rest_id) = inputs.try_next() else {
                        return Ok(None);
                    };
                )*

                let Some($first_id) = <$first>::try_from_input(&mut $first_id)? else {
                    return Ok(None);
                };

                $(
                    let Some($rest_id) = <$rest>::try_from_input(&mut $rest_id)? else {
                        return Ok(None);
                    };
                )*

                Ok(Some(($first_id, $($rest_id,)*)))
            }
        }
    }
}

#[rustfmt::skip]
macro_rules! integer {
    ($ty:ty, $error:ident) => {
        impl FromInput for $ty {
            #[inline]
            fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
                let index = p.index;

                let Some(W(string)) = <W<&str>>::try_from_input(p)? else {
                    return Ok(None);
                };

                let Ok(n) = str::parse(string) else {
                    let pos = p.pos_of(index);
                    return Err(InputError::new(p.path, pos, ErrorKind::$error(string)));
                };

                Ok(Some(n))
            }
        }
    };
}

tuple!(1 => A a);
tuple!(2 => A a, B b);
tuple!(3 => A a, B b, C c);
tuple!(4 => A a, B b, C c, D d);
tuple!(5 => A a, B b, C c, D d, E e);
tuple!(6 => A a, B b, C c, D d, E e, F f);

integer!(usize, NotInteger);
integer!(isize, NotInteger);
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
    fn error_kind() -> ErrorKind {
        ErrorKind::ExpectedChar
    }

    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
        use bstr::ByteSlice;

        let Some(c) = p.data.get(p.index..).and_then(|b| b.chars().next()) else {
            return Ok(None);
        };

        p.advance(c.len_utf8());
        Ok(Some(c))
    }
}

impl FromInput for Input {
    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
        Ok(Some(p.clone()))
    }
}

impl FromInput for &[u8] {
    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
        let data = p.data.get(p.index..p.range.end).unwrap_or_default();
        p.index = p.range.end;
        Ok(Some(data))
    }
}

impl FromInput for &str {
    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
        let Some(data) = <&[u8]>::try_from_input(p)? else {
            return Ok(None);
        };

        let Ok(data) = from_utf8(data) else {
            return Err(InputError::new(p.path, p.pos(), ErrorKind::NotUtf8));
        };

        Ok(Some(data))
    }
}

impl FromInput for &BStr {
    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
        let Some(data) = <&[u8]>::try_from_input(p)? else {
            return Ok(None);
        };

        Ok(Some(BStr::new(data)))
    }
}

/// Parse until end of line.
pub struct Nl<T>(pub T);

impl<T> FromInput for Nl<T>
where
    T: FromInput,
{
    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
        if p.peek().is_none() {
            return Ok(None);
        }

        let pos = p.index;

        let Some(range) = p.until(NL) else {
            let pos = p.pos_of(pos);
            return Err(InputError::new(p.path, pos, ErrorKind::ExpectedLine));
        };

        let mut input = p.slice(range);
        Ok(Some(Self(input.next()?)))
    }
}

/// Consume whitespace and return the number of lines consumed.
pub struct Ws(pub usize);

impl FromInput for Ws {
    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
        let mut n = p.index;

        while let Some(c) = p.at(n) {
            if !c.is_ascii_whitespace() || !c.is_ascii_control() {
                break;
            }

            n = n.saturating_add(1);
        }

        let Some(data) = p.data.get(p.index..n) else {
            return Ok(Some(Self(0)));
        };

        p.index = n;
        Ok(Some(Self(memchr::memchr_iter(NL, data).count())))
    }
}

impl<T, const N: usize> FromInput for arrayvec::ArrayVec<T, N>
where
    T: FromInput,
{
    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
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

        Ok(Some(output))
    }
}

impl<T> FromInput for Vec<T>
where
    T: FromInput,
{
    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
        let mut output = Vec::new();

        while let Some(element) = T::try_from_input(p)? {
            output.push(element);
        }

        Ok(Some(output))
    }
}

/// Split once on byte `D`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Split<const D: char, T>(pub T);

impl<const D: char, T> FromInput for Split<D, T>
where
    T: FromInputIter,
{
    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
        let mut it = p.splitn(D as u8);

        let Some(out) = T::from_input_iter(p, &mut it)? else {
            return Ok(None);
        };

        p.index = it.index();
        Ok(Some(Self(out)))
    }
}

/// Split and return a range.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Range<const D: char, T>(pub ops::Range<T>);

impl<const D: char, T> FromInput for Range<D, T>
where
    T: FromInput,
{
    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
        let Some(Split([a, b])) = Split::<D, [T; 2]>::try_from_input(p)? else {
            return Ok(None);
        };

        Ok(Some(Self(a..b)))
    }
}

impl<const N: usize, T> FromInputIter for [T; N]
where
    T: FromInput,
{
    #[inline]
    fn from_input_iter<I>(p: &mut Input, inputs: &mut I) -> Result<Option<Self>>
    where
        I: InputIterator,
    {
        let mut vec = ArrayVec::new();

        while vec.remaining_capacity() > 0 {
            let Some(mut value) = inputs.try_next() else {
                return Ok(None);
            };

            let Some(value) = T::try_from_input(&mut value)? else {
                return Ok(None);
            };

            vec.push(value);
        }

        let Ok(value) = vec.into_inner() else {
            return Err(InputError::new(p.path, p.pos(), ErrorKind::BadArray));
        };

        Ok(Some(value))
    }
}

#[non_exhaustive]
pub struct Skip;

impl FromInput for Skip {
    #[inline]
    fn try_from_input(_: &mut Input) -> Result<Option<Self>> {
        Ok(Some(Self))
    }
}

/// Parse a word of input, which parses until we reach a whitespace or control character.
pub struct W<T = Skip>(pub T);

impl<T> FromInput for W<T>
where
    T: FromInput,
{
    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
        let mut end = p.index;

        while let Some(c) = p.at(end) {
            if !(c.is_ascii_whitespace() || c.is_ascii_control()) {
                break;
            }

            end = end.saturating_add(1);
        }

        let start = end;

        while let Some(c) = p.at(end) {
            if c.is_ascii_whitespace() || c.is_ascii_control() {
                break;
            }

            end = end.saturating_add(1);
        }

        if start == end {
            return Ok(None);
        }

        let mut input = p.slice(start..end);

        let Some(value) = T::try_from_input(&mut input)? else {
            return Ok(None);
        };

        p.index = end;
        Ok(Some(Self(value)))
    }
}

/// Filter out empty values.
pub struct NonEmpty<T>(pub T);

impl<T> FromInput for NonEmpty<T>
where
    T: FromInput,
{
    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
        if p.is_empty() {
            return Ok(None);
        }

        Ok(T::try_from_input(p)?.map(Self))
    }
}
