//! Input parser.

use std::convert::Infallible;
use std::error;
use std::fmt;

type Result<T> = std::result::Result<T, InputError>;

/// Parser error.
#[derive(Debug)]
pub struct InputError {
    path: &'static str,
    pos: LineCol,
    kind: ErrorKind,
}

impl<'a> InputError {
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
    Boxed(anyhow::Error),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::NotInteger => write!(f, "not an integer or integer overflow"),
            ErrorKind::NotFloat => write!(f, "not a float"),
            ErrorKind::NotChar => write!(f, "not a character"),
            ErrorKind::NotLine => write!(f, "not a line"),
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
        write!(f, "{}:{}", self.line + 1, self.column)
    }
}

/// Helper to parse input from a file.
#[derive(Debug)]
pub struct Input {
    /// Path being parsed.
    path: &'static str,
    /// The path being parsed.
    string: &'static str,
    /// Index being read.
    index: usize,
    // Current reader location.
    pos: LineCol,
}

impl Input {
    /// Construct a new input processor.
    #[doc(hidden)]
    pub fn new(path: &'static str, string: &'static str) -> Self {
        Self {
            path,
            string,
            index: 0,
            pos: LineCol::default(),
        }
    }

    /// Remaining string of the current input.
    pub fn as_str(&self) -> &'static str {
        self.string.get(self.index..).unwrap_or_default()
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
    pub fn next<T>(&mut self) -> Result<T>
    where
        T: FromInput,
    {
        T::from_input_whitespace(self)
    }

    /// Parse the next value as T.
    #[inline]
    pub fn try_next<T>(&mut self) -> Result<Option<T>>
    where
        T: FromInput,
    {
        T::try_from_input_whitespace(self)
    }

    /// Get the next line of input.
    #[inline]
    pub fn next_line(&mut self) -> Option<&'static str> {
        let string = self.string.get(self.index..)?;

        let Some(end) = memchr::memchr(b'\n', string.as_bytes()) else {
            self.index = self.string.len();
            self.pos.column += string.chars().count();
            return Some(string);
        };

        self.pos.new_line();
        self.index = self.index.saturating_add(end.saturating_add(1));
        string.get(..end)
    }

    /// Test if we're at eof.
    pub fn is_eof(&mut self) -> bool {
        self.peek().is_none()
    }

    /// Consume whitespace.
    fn peek_whitespace(&mut self) -> usize {
        let mut n = 0;

        while let Some(c) = self.peek_from(n) {
            if !c.is_whitespace() {
                break;
            }

            n = n.checked_add(c.len_utf8()).expect("cursor overflow");
        }

        n
    }

    /// Consume whitespace.
    fn consume_whitespace(&mut self) {
        let n = self.peek_whitespace();
        self.advance(n);
    }

    /// Get the byte at the given reader offset.
    fn peek_from(&self, n: usize) -> Option<char> {
        self.string
            .get(self.index..)
            .and_then(|s| s.get(n..))
            .unwrap_or_default()
            .chars()
            .next()
    }

    /// Get the byte at the given reader offset.
    #[inline]
    fn peek(&self) -> Option<char> {
        self.peek_from(0)
    }

    #[inline]
    fn advance(&mut self, n: usize) {
        if n == 0 {
            return;
        }

        let Some(string) = self.string.get(self.index..).and_then(|s| s.get(..n)) else {
            return;
        };

        for c in string.chars() {
            match c {
                '\n' => {
                    self.pos.new_line();
                }
                c if !c.is_control() => {
                    self.pos.new_column();
                }
                _ => {}
            }
        }

        self.index = self.index.checked_add(n).expect("cursor overflow");
    }

    /// Step the buffer.
    fn step(&mut self) {
        let Some(c) = self.string.get(self.index..).unwrap_or_default().chars().next() else {
            return;
        };

        match c {
            '\n' => {
                self.pos.new_line();
            }
            c if !c.is_control() => {
                self.pos.new_column();
            }
            _ => {}
        }

        self.index = self
            .index
            .checked_add(c.len_utf8())
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
                let start = p.index;
                let pos = p.pos;

                while let Some(c) = p.peek() {
                    if !matches!(c, '-' | '.' | '0'..='9') {
                        break;
                    }

                    p.step();
                }

                let Some(n) = p.string.get(start..p.index).and_then(|s| str::parse(s).ok()) else {
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
        let pos = p.pos;

        let Some(c) = p.peek() else {
            return Err(InputError::new(p.path, pos, ErrorKind::NotChar));
        };

        p.step();
        Ok(c)
    }
}

impl FromInput for &str {
    #[inline]
    fn from_input(p: &mut Input) -> Result<Self> {
        let start = p.index;

        while let Some(c) = p.peek() {
            if c.is_whitespace() {
                break;
            }

            p.step();
        }

        Ok(p.string.get(start..p.index).unwrap_or_default())
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
            string,
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
