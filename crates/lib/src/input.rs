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
    fn new_line(&mut self) {
        self.line += 1;
        self.column = 0;
    }

    fn new_column(&mut self) {
        self.column += 1;
    }
}

impl fmt::Display for LineCol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.column + 1)
    }
}

/// Helper to parse input from a file.
#[derive(Debug)]
pub struct Input {
    /// Path being parsed.
    path: &'static str,
    /// The path being parsed.
    data: &'static BStr,
    /// Index being read.
    index: usize,
    // Current reader location.
    pos: LineCol,
}

impl Input {
    /// Construct a new input processor.
    #[doc(hidden)]
    pub fn new(path: &'static str, string: &'static BStr) -> Self {
        Self {
            path,
            data: string,
            index: 0,
            pos: LineCol::default(),
        }
    }

    /// Reset input.
    pub fn reset(&mut self) {
        self.index = 0;
        self.pos = LineCol::default();
    }

    /// Remaining string of the current input.
    pub fn as_bstr(&self) -> &'static BStr {
        BStr::new(self.data.get(self.index..).unwrap_or_default())
    }

    /// Get the current input path.
    pub fn path(&self) -> &'static str {
        self.path
    }

    /// Get the current input position.
    pub fn pos(&self) -> LineCol {
        self.pos
    }

    /// Skip whitespace and return the number of lines skipped.
    fn skip_whitespace(&mut self) -> Result<usize> {
        let start = self.pos.line;
        self.consume_whitespace();
        Ok(self.pos.line - start)
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

    /// Parse a line according to the given specification.
    #[inline]
    pub fn line<T>(&mut self) -> Result<T>
    where
        T: FromInput,
    {
        let Nl(mut line) = self.next()?;
        line.next()
    }

    /// Parse the next value as T.
    #[inline]
    pub fn try_next<T>(&mut self) -> Result<Option<T>>
    where
        T: FromInput,
    {
        T::try_from_input_whitespace(self)
    }

    /// Parse a line according to the given specification.
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

    /// Get the next line of input.
    #[inline]
    pub fn next_line(&mut self) -> Option<&'static BStr> {
        let string = self.data.get(self.index..)?;

        let Some(end) = memchr::memchr(b'\n', string.as_ref()) else {
            self.index = self.data.len();
            self.pos.column += string.len();
            return Some(BStr::new(string));
        };

        self.pos.new_line();
        self.index = self.index.saturating_add(end.saturating_add(1));
        Some(BStr::new(string.get(..end)?))
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
        self.data
            .get(self.index..)
            .and_then(|s| s.get(n..))
            .unwrap_or_default()
            .iter()
            .next()
            .copied()
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

        let Some(string) = self.data.get(self.index..).and_then(|s| s.get(..n)) else {
            return;
        };

        for &c in string.iter() {
            match c {
                b'\n' => {
                    self.pos.new_line();
                }
                c if !c.is_ascii_control() => {
                    self.pos.new_column();
                }
                _ => {}
            }
        }

        self.index = self.index.checked_add(n).expect("cursor overflow");
    }

    /// Step the buffer.
    fn step(&mut self) {
        let Some(&c) = self.data.get(self.index..).unwrap_or_default().iter().next() else {
            return;
        };

        match c {
            b'\n' => {
                self.pos.new_line();
            }
            c if !c.is_ascii_control() => {
                self.pos.new_column();
            }
            _ => {}
        }

        self.index = self
            .index
            .checked_add(1)
            .expect("cursor overflow");
    }
}

/// A value that can be parsed from input.
pub trait FromInput: Sized {
    /// Optionally try to confuse input ignoring leading whitespace by default.
    #[inline]
    fn try_from_input_whitespace(p: &mut Input) -> Result<Option<Self>> {
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
                let pos = p.pos;
                let string: &str = FromInput::from_input(p)?;

                let Ok(n) = str::parse(string) else {
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

        let pos = p.pos;

        let Some(c) = p.data.get(p.index..).and_then(|b| b.chars().next()) else {
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
            return Err(InputError::new(p.path, p.pos, ErrorKind::NotUtf8));
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
    fn try_from_input_whitespace(p: &mut Input) -> Result<Option<Self>> {
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
        let pos = p.pos;

        let Some(string) = p.next_line() else {
            return Err(InputError::new(p.path, p.pos, ErrorKind::NotLine));
        };

        Ok(Self(Input {
            path: p.path,
            data: string,
            index: 0,
            pos,
        }))
    }
}

/// Consume whitespace and return the number of lines consumed.
pub struct Ws(pub usize);

impl FromInput for Ws {
    #[inline]
    fn try_from_input_whitespace(p: &mut Input) -> Result<Option<Self>> {
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
        let mut pos = p.pos;

        while let Some(element) = T::try_from_input_whitespace(p)? {
            if output.remaining_capacity() == 0 {
                return Err(InputError::new(p.path, pos, ErrorKind::ArrayCapacity(N)));
            }

            output.push(element);
            pos = p.pos;
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

        while let Some(element) = T::try_from_input_whitespace(p)? {
            output.push(element);
        }

        Ok(output)
    }
}
