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
    NotChar,
    IntegerOverflow,
    Boxed(anyhow::Error),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::NotInteger => write!(f, "not an integer"),
            ErrorKind::NotChar => write!(f, "not a character"),
            ErrorKind::IntegerOverflow => write!(f, "integer overflow"),
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
pub struct Input {
    /// Path being parsed.
    path: &'static str,
    /// The path being parsed.
    string: &'static str,
    /// Index being read.
    index: usize,
    // Current reader location.
    pos: LineCol,
    /// Check if input is whitespace sensitive or not.
    whitespace: bool,
}

impl Input {
    /// Construct a new input processor.
    pub fn new(path: &'static str, string: &'static str) -> Self {
        Self {
            path,
            string,
            index: 0,
            pos: LineCol::default(),
            whitespace: false,
        }
    }

    /// Get the current input path.
    pub fn path(&self) -> &'static str {
        self.path
    }

    /// Get the current input position.
    pub fn pos(&self) -> LineCol {
        self.pos
    }

    /// Set if input is whitespace sensitive or not.
    ///
    /// If `false`, input parsing will automatically skip whitespace which is
    /// the default.
    pub fn set_whitespace(&mut self, whitespace: bool) {
        self.whitespace = whitespace;
    }

    /// Skip whitespace and return the number of lines skipped.
    pub fn skip_whitespace(&mut self) -> Result<usize> {
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
        if !self.whitespace {
            self.consume_whitespace();
        }

        T::from_input(self)
    }

    /// Parse the next value as T.
    #[inline]
    pub fn try_next<T>(&mut self) -> Result<Option<T>>
    where
        T: FromInput,
    {
        if !self.whitespace {
            self.consume_whitespace();
        }

        if !T::peek(self) {
            return Ok(None);
        }

        Ok(Some(T::from_input(self)?))
    }

    /// Test if we're at eof.
    pub fn is_eof(&mut self) -> bool {
        self.peek().is_none()
    }

    /// Consume whitespace.
    fn consume_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if !c.is_whitespace() {
                break;
            }

            self.step();
        }
    }

    /// Get the byte at the given reader offset.
    fn peek(&self) -> Option<char> {
        self.string
            .get(self.index..)
            .unwrap_or_default()
            .chars()
            .next()
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

        self.index += c.len_utf8();
    }
}

/// A value that can be parsed from input.
pub trait FromInput: Sized {
    fn peek(p: &Input) -> bool;

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
            fn peek(p: &Input) -> bool {
                <$first>::peek(p)
            }

            #[inline]
            fn from_input(p: &mut Input) -> Result<Self> {
                let $first_id = p.next::<$first>()?;
                $(let $rest_id = p.next::<$rest>()?;)*
                Ok(($first_id, $($rest_id,)*))
            }
        }
    }
}

macro_rules! integer {
    ($ty:ty) => {
        impl FromInput for $ty {
            fn peek(p: &Input) -> bool {
                matches!(p.peek(), Some('0'..='9'))
            }

            fn from_input(p: &mut Input) -> Result<Self> {
                const ZERO: $ty = '0' as $ty;

                let mut n = match p.peek() {
                    Some(c @ '0'..='9') => c as $ty - ZERO,
                    _ => return Err(InputError::new(p.path, p.pos, ErrorKind::NotInteger)),
                };

                p.step();

                while let Some(c) = p.peek() {
                    if !matches!(c, '0'..='9') {
                        break;
                    }

                    let digit = c as $ty - ZERO;

                    let Some(update) = n.checked_mul(10).and_then(|n| n.checked_add(digit)) else {
                        return Err(InputError::new(p.path, p.pos, ErrorKind::IntegerOverflow));
                    };

                    n = update;
                    p.step();
                }

                Ok(n)
            }
        }
    };
}

tuple!(A a);
tuple!(A a, B b);
tuple!(A a, B b, C c);
tuple!(A a, B b, C c, D d);

integer!(u32);
integer!(u64);
integer!(i32);
integer!(i64);

impl FromInput for char {
    fn peek(p: &Input) -> bool {
        p.peek().is_some()
    }

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
    fn peek(p: &Input) -> bool {
        p.peek().is_some()
    }

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
