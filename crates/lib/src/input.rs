//! Input parser.

use std::convert::Infallible;
use std::error;
use std::fmt;
use std::str::from_utf8;

use bstr::BStr;

type Result<T> = std::result::Result<T, InputError>;

/// Various forms of input errors.
#[derive(Debug)]
pub struct InputError {
    path: &'static str,
    pos: LineCol,
    kind: ErrorKind,
}

impl InputError {
    pub fn any(path: &'static str, pos: LineCol, error: anyhow::Error) -> Self {
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

/// Helper to parse input from a file.
#[derive(Debug, Clone, Copy)]
pub struct Input {
    /// Path being parsed.
    path: &'static str,
    /// The path being parsed.
    data: &'static [u8],
    /// Index into the current slice.
    index: usize,
    /// Index being read.
    start: usize,
    end: usize,
}

impl Input {
    /// Construct a new input processor.
    #[doc(hidden)]
    pub fn new(path: &'static str, string: &'static [u8]) -> Self {
        Self {
            path,
            data: string,
            index: 0,
            start: 0,
            end: string.len(),
        }
    }

    /// Get current index.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Reset input.
    pub fn reset(&mut self) {
        self.index = self.start;
    }

    /// Get remaining binary string of the input.
    pub fn as_bytes(&self) -> &'static [u8] {
        self.data.get(self.index..self.end).unwrap_or_default()
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

    /// Get the current input position based on the given index.
    pub fn pos_of(&self, index: usize) -> LineCol {
        let Some(data) = self.data.get(..=index) else {
            return LineCol::EMPTY;
        };

        let mut line = 0;
        let mut last = 0;
        let mut it = memchr::memchr_iter(b'\n', data);

        while let Some(n) = it.next() {
            line += 1;
            last = n;
        }

        let column = data.get(last.saturating_add(1)..).unwrap_or_default().len();

        LineCol {
            line,
            column
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
    fn skip_whitespace(&mut self) -> Result<usize> {
        let start = self.index;
        self.consume_whitespace();
        let data = self.data.get(start..self.index).unwrap_or_default();
        Ok(memchr::memchr_iter(b'\n', data).count())
    }

    /// Get the next line of input.
    #[inline]
    fn next_line(&mut self) -> Option<(usize, usize)> {
        let string = self.data.get(self.index..self.end)?;

        let Some(at) = memchr::memchr(b'\n', string.as_ref()) else {
            self.index = self.end;
            return Some((self.index, self.end));
        };

        let end = self.index.saturating_add(at);
        let start = std::mem::replace(&mut self.index, end.saturating_add(1));
        Some((start, end))
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
        let n = self.index.checked_add(n)?.min(self.end);

        if n >= self.end {
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

        self.index = self.index.saturating_add(n).min(self.end);
    }

    /// Step the buffer.
    fn step(&mut self) {
        self.index = self.index.saturating_add(1).min(self.end);
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

impl FromInput for &BStr {
    #[inline]
    fn from_input(p: &mut Input) -> Result<Self> {
        let data = <&[u8]>::from_input(p)?;
        Ok(BStr::new(data))
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

        let Some((start, end)) = p.next_line() else {
            let pos = p.pos_of(pos);
            return Err(InputError::new(p.path, pos, ErrorKind::NotLine));
        };

        Ok(Self(Input {
            path: p.path,
            data: p.data,
            index: start,
            start,
            end,
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
        Ok(Self(p.skip_whitespace()?))
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
