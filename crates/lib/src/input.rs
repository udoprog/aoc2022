//! Input parser.

use std::convert::Infallible;
use std::error;
use std::ffi::OsStr;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::buf::Buf;

type Result<T> = std::result::Result<T, InputError>;

/// Parser error.
#[derive(Debug)]
pub struct InputError {
    path: Box<Path>,
    pos: LineCol,
    kind: ErrorKind,
}

impl<'a> InputError {
    pub fn any(path: &'a Path, pos: LineCol, error: anyhow::Error) -> Self {
        Self {
            path: path.into(),
            pos,
            kind: ErrorKind::Boxed(error),
        }
    }

    fn new(path: &'a Path, pos: LineCol, kind: ErrorKind) -> Self {
        Self {
            path: path.into(),
            pos,
            kind,
        }
    }
}

impl fmt::Display for InputError {
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

impl error::Error for InputError {}

impl From<Infallible> for InputError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

#[derive(Debug)]
enum ErrorKind {
    NotInteger,
    NotByte,
    NotChar,
    IntegerOverflow,
    NotUtf8,
    System(std::io::Error),
    Boxed(anyhow::Error),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::NotInteger => write!(f, "not an integer"),
            ErrorKind::NotByte => write!(f, "not a byte"),
            ErrorKind::NotChar => write!(f, "not a character"),
            ErrorKind::IntegerOverflow => write!(f, "integer overflow"),
            ErrorKind::NotUtf8 => write!(f, "not utf-8"),
            ErrorKind::System(error) => error.fmt(f),
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
pub struct Input<'a> {
    /// The path being parsed.
    path: &'a Path,
    /// The reader.
    reader: Option<File>,
    // Current reader location.
    pos: LineCol,
    // Input buffer.
    buf: Buf<128>,
    /// Check if input is whitespace sensitive or not.
    whitespace: bool,
}

impl<'a> Input<'a> {
    /// Construct a new input processor.
    pub fn new<P>(path: &'a P) -> Result<Self>
    where
        P: ?Sized + AsRef<OsStr>,
    {
        let path = Path::new(path);
        let pos = LineCol::default();
        let file =
            File::open(path).map_err(|e| InputError::new(path, pos, ErrorKind::System(e)))?;

        Ok(Self {
            path,
            reader: Some(file),
            pos,
            buf: Buf::new(),
            whitespace: false,
        })
    }

    /// Get the current input path.
    pub fn path(&self) -> &Path {
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
        self.consume_whitespace()?;
        Ok(self.pos.line - start)
    }

    /// Parse the next value as T.
    #[inline]
    pub fn next<T>(&mut self) -> Result<T>
    where
        T: FromInput,
    {
        if !self.whitespace {
            self.consume_whitespace()?;
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
            self.consume_whitespace()?;
        }

        if !T::peek(self)? {
            return Ok(None);
        }

        Ok(Some(T::from_input(self)?))
    }

    /// Test if we're at eof.
    pub fn is_eof(&mut self) -> Result<bool> {
        self.fill(1)?;
        Ok(self.buf.get(0).is_none())
    }

    /// Consume whitespace.
    fn consume_whitespace(&mut self) -> Result<()> {
        while let Some(b) = self.read_at(0)? {
            if !b.is_ascii_whitespace() {
                break;
            }

            self.step();
        }

        Ok(())
    }

    /// Ensure to fill read buffer.
    fn fill(&mut self, n: usize) -> Result<()> {
        if let Some(reader) = &mut self.reader {
            while self.buf.len() <= n {
                let data = self.buf.as_uninit_mut();

                let n = reader
                    .read(data)
                    .map_err(|io| InputError::new(self.path, self.pos, ErrorKind::System(io)))?;

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
    fn read_at(&mut self, n: usize) -> Result<Option<u8>> {
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
pub trait FromInput: Sized {
    fn peek(p: &mut Input<'_>) -> Result<bool>;

    /// Parse a value from a given input.
    fn from_input(p: &mut Input<'_>) -> Result<Self>;
}

macro_rules! tuple {
    ($first:ident $first_id:ident $(, $rest:ident $rest_id:ident)* $(,)?) => {
        impl<$first, $($rest,)*> FromInput for ($first, $($rest, )*)
        where
            $first: FromInput,
            $($rest: FromInput,)*
        {
            #[inline]
            fn peek(p: &mut Input<'_>) -> Result<bool> {
                <$first>::peek(p)
            }

            #[inline]
            fn from_input(p: &mut Input<'_>) -> Result<Self> {
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
            fn peek(p: &mut Input<'_>) -> Result<bool> {
                Ok(matches!(p.read_at(0)?, Some(b'0'..=b'9')))
            }

            fn from_input(p: &mut Input<'_>) -> Result<Self> {
                const ZERO: $ty = b'0' as $ty;

                let mut n = match p.read_at(0)? {
                    Some(b @ b'0'..=b'9') => <$ty>::try_from(b)? - ZERO,
                    _ => return Err(InputError::new(p.path, p.pos, ErrorKind::NotInteger)),
                };

                p.step();

                while let Some(b) = p.read_at(0)? {
                    if !matches!(b, b'0'..=b'9') {
                        break;
                    }

                    let digit = <$ty>::try_from(b)? - ZERO;

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

impl FromInput for u8 {
    fn peek(p: &mut Input<'_>) -> Result<bool> {
        Ok(p.read_at(0)?.is_some())
    }

    fn from_input(p: &mut Input<'_>) -> Result<Self> {
        let b = match p.read_at(0)? {
            Some(b) => b,
            _ => return Err(InputError::new(p.path, p.pos, ErrorKind::NotByte)),
        };

        p.step();
        Ok(b)
    }
}

impl FromInput for char {
    fn peek(p: &mut Input<'_>) -> Result<bool> {
        Ok(p.read_at(0)?.is_some())
    }

    fn from_input(p: &mut Input<'_>) -> Result<Self> {
        let pos = p.pos;

        let b = match p.read_at(0)? {
            Some(b) => b,
            _ => return Err(InputError::new(p.path, pos, ErrorKind::NotChar)),
        };

        p.step();
        let count = b.leading_ones() as usize;

        let mut bytes = [b, 0, 0, 0];

        for (_, o) in (1..count).zip(bytes.iter_mut().skip(1)) {
            let Some(b) = p.read_at(0)? else {
                return Err(InputError::new(p.path, pos, ErrorKind::NotChar));
            };

            *o = b;
            p.step();
        }

        let string = std::str::from_utf8(&bytes)
            .map_err(|_| InputError::new(p.path, pos, ErrorKind::NotUtf8))?;

        let Some(c) = string.chars().next() else {
            return Err(InputError::new(p.path, pos, ErrorKind::NotChar));
        };

        Ok(c)
    }
}
