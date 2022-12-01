//! Input parser.

use std::convert::Infallible;
use std::error;
use std::fmt;
use std::io;
use std::path::Path;

use crate::buf::Buf;

/// Parser error.
#[derive(Debug)]
pub struct Error {
    path: Box<Path>,
    pos: LineCol,
    kind: ErrorKind,
}

impl<'a> Error {
    fn new(path: &'a Path, pos: LineCol, kind: ErrorKind) -> Self {
        Self {
            path: path.into(),
            pos,
            kind,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{path}:{pos}: {kind}",
            path = self.path.display(),
            pos = self.pos,
            kind = self.kind
        )
    }
}

impl error::Error for Error {}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

#[derive(Debug)]
enum ErrorKind {
    NotInteger,
    IntegerOverflow,
    System(io::Error),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::NotInteger => write!(f, "not an integer"),
            ErrorKind::IntegerOverflow => write!(f, "integer overflow"),
            ErrorKind::System(error) => error.fmt(f),
        }
    }
}

/// A line and column combination.
#[derive(Default, Debug, Clone, Copy)]
struct LineCol {
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
pub struct Input<'a, R> {
    /// The path being parsed.
    path: &'a Path,
    /// The reader.
    reader: Option<R>,
    // Current reader location.
    pos: LineCol,
    // Input buffer.
    buf: Buf<128>,
    /// Check if input is whitespace sensitive or not.
    whitespace: bool,
}

impl<'a, R> Input<'a, R> {
    /// Construct a new input processor.
    pub fn new(path: &'a Path, reader: R) -> Self {
        Self {
            path,
            reader: Some(reader),
            pos: LineCol::default(),
            buf: Buf::new(),
            whitespace: false,
        }
    }

    /// Set if input is whitespace sensitive or not.
    ///
    /// If `false`, input parsing will automatically skip whitespace which is
    /// the default.
    pub fn set_whitespace(&mut self, whitespace: bool) {
        self.whitespace = whitespace;
    }
}

impl<'a, R> Input<'a, R>
where
    R: io::Read,
{
    /// Skip whitespace and return the number of lines skipped.
    pub fn skip_whitespace(&mut self) -> Result<usize, Error> {
        let start = self.pos.line;
        self.consume_whitespace()?;
        Ok(self.pos.line - start)
    }

    /// Parse the next value as T.
    #[inline]
    pub fn next<T>(&mut self) -> Result<T, Error>
    where
        T: FromInput<'a, R>,
    {
        if !self.whitespace {
            self.consume_whitespace()?;
        }

        T::from_input(self)
    }

    /// Parse the next value as T.
    #[inline]
    pub fn try_next<T>(&mut self) -> Result<Option<T>, Error>
    where
        T: FromInput<'a, R>,
    {
        if !self.whitespace {
            self.consume_whitespace()?;
        }

        if !T::peek(self)? {
            return Ok(None);
        }

        Ok(Some(T::from_input(self)?))
    }

    /// Test if we're at eof.
    pub fn is_eof(&mut self) -> Result<bool, Error> {
        self.fill(1)?;
        Ok(self.buf.get(0).is_none())
    }

    /// Consume whitespace.
    fn consume_whitespace(&mut self) -> Result<(), Error> {
        while let Some(b) = self.read_at(0)? {
            if !b.is_ascii_whitespace() {
                break;
            }

            self.step();
        }

        Ok(())
    }

    /// Ensure to fill read buffer.
    fn fill(&mut self, n: usize) -> Result<(), Error> {
        if let Some(reader) = &mut self.reader {
            while self.buf.len() <= n {
                let data = self.buf.as_uninit_mut();

                let n = reader
                    .read(data)
                    .map_err(|io| Error::new(self.path, self.pos, ErrorKind::System(io)))?;

                if n == 0 {
                    self.reader = None;
                    break;
                }

                self.buf.advance(n);
            }
        }

        Ok(())
    }

    /// Get the byte at the given reader offset.
    fn read_at(&mut self, n: usize) -> Result<Option<u8>, Error> {
        self.fill(n)?;
        Ok(self.buf.get(n))
    }

    /// Step the buffer.
    fn step(&mut self) {
        let p = self.buf.pop_front();

        match p {
            Some(b'\n') => {
                self.pos.new_line();
            }
            Some(..) => {
                self.pos.new_column();
            }
            _ => {}
        }
    }
}

/// A value that can be parsed from input.
pub trait FromInput<'a, R>: Sized
where
    R: io::Read,
{
    fn peek(p: &mut Input<'a, R>) -> Result<bool, Error>;

    /// Parse a value from a given input.
    fn from_input(p: &mut Input<'a, R>) -> Result<Self, Error>;
}

macro_rules! integer {
    ($ty:ty) => {
        impl<'a, R> FromInput<'a, R> for $ty
        where
            R: io::Read,
        {
            fn peek(p: &mut Input<'a, R>) -> Result<bool, Error> {
                Ok(matches!(p.read_at(0)?, Some(b'0'..=b'9')))
            }

            fn from_input(p: &mut Input<'a, R>) -> Result<Self, Error> {
                const ZERO: $ty = b'0' as $ty;

                let mut n = match p.read_at(0)? {
                    Some(b @ b'0'..=b'9') => <$ty>::try_from(b)? - ZERO,
                    _ => return Err(Error::new(p.path, p.pos, ErrorKind::NotInteger)),
                };

                p.step();

                while let Some(b) = p.read_at(0)? {
                    if !matches!(b, b'0'..=b'9') {
                        break;
                    }

                    let digit = <$ty>::try_from(b)? - ZERO;

                    let Some(update) = n.checked_mul(10).and_then(|n| n.checked_add(digit)) else {
                                return Err(Error::new(p.path, p.pos, ErrorKind::IntegerOverflow));
                            };

                    n = update;
                    p.step();
                }

                Ok(n)
            }
        }
    };
}

integer!(u32);
integer!(u64);
integer!(i32);
integer!(i64);
