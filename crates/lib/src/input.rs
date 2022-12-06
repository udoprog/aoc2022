//! Input parser.

mod error;
mod iter;

use core::mem;
use core::ops;
use std::str::from_utf8;

use arrayvec::ArrayVec;
use bstr::BStr;

pub use self::error::{Custom, ErrorKind, IStrError};
pub use self::iter::Iter;

pub(self) type Result<T> = std::result::Result<T, IStrError>;
use crate::env::Size;

pub(crate) const NL: u8 = b'\n';

/// Helper to parse input.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(prod, repr(transparent))]
pub struct IStr {
    /// The path being parsed.
    data: &'static [u8],
    /// Size of index being used.
    index: Size,
}

impl IStr {
    /// Construct a new input processor.
    #[inline]
    pub fn new(data: &'static [u8], index: Size) -> Self {
        Self { data, index }
    }

    /// Access index of input string.
    #[inline]
    pub fn index(&self) -> Size {
        self.index
    }

    /// Test if input is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the length of the current input.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Get input being processed.
    #[inline]
    pub fn as_data(&self) -> &'static [u8] {
        self.data
    }

    /// Get remaining binary string of the input.
    #[inline]
    pub fn as_bstr(&self) -> &BStr {
        BStr::new(self.as_data())
    }

    /// Cosntruct an iterator over the current input.
    #[inline]
    pub fn iter<T>(&mut self) -> Iter<'_, T> {
        Iter::new(self)
    }

    /// Split `N` times.
    #[inline]
    pub fn splitn(&mut self, byte: u8) -> impl InputIterator + '_ {
        return Iterator { input: self, byte };

        struct Iterator<'a> {
            input: &'a mut IStr,
            byte: u8,
        }

        impl<'a> InputIterator for Iterator<'a> {
            #[inline]
            fn index(&self) -> Size {
                self.input.index
            }

            #[inline]
            fn next(&mut self) -> Option<IStr> {
                self.input.split_once(self.byte)
            }
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

    /// Parse the next value as `T`, errors with `Err(IStrError)` if the next
    /// element is not a valid value of type `T`.
    #[inline]
    pub fn line<T>(&mut self) -> Result<T>
    where
        T: FromInput,
    {
        let index = self.index;

        let Some(line) = self.try_line()? else {
            return Err(IStrError::new(index..self.index, ErrorKind::ExpectedLine));
        };

        Ok(line)
    }

    /// Parse the next value as `T`, errors with `Err(IStrError)` if the next
    /// element is not a valid value of type `T`, returns `Ok(None)` if there is
    /// no more non-whitespace data to process.
    #[inline]
    pub fn try_line<T>(&mut self) -> Result<Option<T>>
    where
        T: FromInput,
    {
        let Some(mut line) = self.split_once(NL) else {
            return Ok(None);
        };

        let Some(output) = line.try_next()? else {
            return Ok(None);
        };

        Ok(Some(output))
    }

    /// Shorthand for using [Ws] to scan newlines.
    #[inline]
    pub fn ws(&mut self) -> Result<usize> {
        let Ws(n) = self.next::<Ws>()?;
        Ok(n)
    }

    /// Try to parse the next word.
    pub fn try_next_word<T>(&mut self) -> Result<Option<(Size, T)>>
    where
        T: FromInput,
    {
        let s = self.find(0, |b| !u8::is_ascii_whitespace(b));
        let n = self.find(s, u8::is_ascii_whitespace);

        if s == n {
            return Ok(None);
        }

        let Some(mut input) = self.slice(s..n) else {
            return Ok(None);
        };

        let Some(value) = T::try_from_input(&mut input)? else {
            return Ok(None);
        };

        self.advance(n);
        Ok(Some((Size::new(s), value)))
    }

    fn split_once_at<T>(&mut self, find: T) -> Option<IStr>
    where
        T: FnOnce(&[u8]) -> Option<usize>,
    {
        if self.data.is_empty() {
            return None;
        }

        let Some(at) = find(self.data) else {
            self.index.advance(self.data.len());
            let data = mem::take(&mut self.data);
            return Some(IStr::new(data, self.index));
        };

        let data = self.data.get(..at)?;
        let n = at.checked_add(1)?;
        let index = self.index.checked_add(Size::new(n))?;
        self.advance(n);
        Some(IStr::new(data, index))
    }

    /// Split once at the given byte or until the end of string, returning the new IStr associated with the split.
    #[inline]
    fn split_once(&mut self, b: u8) -> Option<IStr> {
        self.split_once_at(|data| memchr::memchr(b, data))
    }

    /// Find by predicate.
    fn find(&self, mut n: usize, p: fn(&u8) -> bool) -> usize {
        while let Some(c) = self.data.get(n) {
            if p(c) {
                break;
            }

            n += 1;
        }

        n
    }

    #[inline]
    fn advance(&mut self, n: usize) {
        self.data = self.data.get(n..).unwrap_or_default();
        self.index = self.index.saturating_add(Size::new(n));
    }

    /// Construct a sub-range.
    #[inline]
    fn slice(&self, range: ops::Range<usize>) -> Option<IStr> {
        let index = self.index.checked_add(Size::new(range.start))?;

        Some(Self {
            data: self.data.get(range)?,
            index,
        })
    }
}

impl IStr {
    pub fn test(&mut self) {
        println!("yes?");
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
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>>;

    /// Parse a value from a given input.
    #[inline]
    fn from_input(p: &mut IStr) -> Result<Self> {
        let index = p.index;

        let Some(value) = Self::try_from_input(p)? else {
            return Err(IStrError::new(index..p.index, Self::error_kind()));
        };

        Ok(value)
    }
}

/// Iterator over inputs.
pub trait InputIterator {
    /// Current index of the input iterator.
    fn index(&self) -> Size;

    /// Get next input.
    fn next(&mut self) -> Option<IStr>;
}

/// Parse something from a pair of inputs.
pub trait FromInputIter: Sized {
    /// Optionally try to confuse input ignoring leading whitespace by default.
    fn from_input_iter<I>(inputs: I) -> Result<Option<Self>>
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
            fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
                let Some($first_id) = p.try_next()? else {
                    return Ok(None);
                };

                $(
                    let Some($rest_id) = p.try_next()? else {
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
            fn from_input_iter<I>(mut inputs: I) -> Result<Option<Self>>
            where
                I: InputIterator
            {
                let Some(mut $first_id) = inputs.next() else {
                    return Ok(None);
                };

                $(
                    let Some(mut $rest_id) = inputs.next() else {
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
            fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
                let index = p.index;

                let Some((n, string)) = p.try_next_word()? else {
                    return Ok(None);
                };

                let Ok(n) = str::parse(string) else {
                    return Err(IStrError::new(index.saturating_add(n)..p.index, ErrorKind::$error(string)));
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
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        use bstr::ByteSlice;

        let Some(c) = p.data.chars().next() else {
            return Ok(None);
        };

        p.advance(c.len_utf8());
        Ok(Some(c))
    }
}

impl FromInput for IStr {
    #[inline]
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        Ok(Some(p.clone()))
    }
}

impl FromInput for &[u8] {
    #[inline]
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        Ok(Some(mem::take(&mut p.data)))
    }
}

impl FromInput for &str {
    #[inline]
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        let index = p.index;

        let Some(data) = <&[u8]>::try_from_input(p)? else {
            return Ok(None);
        };

        let Ok(data) = from_utf8(data) else {
            return Err(IStrError::new(index..p.index, ErrorKind::NotUtf8));
        };

        Ok(Some(data))
    }
}

impl FromInput for &BStr {
    #[inline]
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
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
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        let Some(mut input) = p.split_once(NL) else {
            return Ok(None);
        };

        Ok(Some(Self(input.next()?)))
    }
}

/// Consume whitespace and return the number of lines consumed.
pub struct Ws(pub usize);

impl FromInput for Ws {
    #[inline]
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        let n = p.find(0, |b| !b.is_ascii_whitespace());

        if n == 0 {
            return Ok(Some(Self(0)));
        }

        let Some(data) = p.data.get(..n) else {
            return Ok(Some(Self(0)));
        };

        p.advance(n);
        Ok(Some(Self(memchr::memchr_iter(NL, data).count())))
    }
}

impl<T, const N: usize> FromInput for [T; N]
where
    T: FromInput,
{
    #[inline]
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        let start = p.index;

        let Some(output) = arrayvec::ArrayVec::try_from_input(p)? else {
            return Ok(None);
        };

        match output.into_inner() {
            Ok(array) => Ok(Some(array)),
            Err(array) => {
                return Err(IStrError::new(
                    start..p.index,
                    ErrorKind::BadArray(N, array.len()),
                ));
            }
        }
    }
}

impl<T, const N: usize> FromInput for arrayvec::ArrayVec<T, N>
where
    T: FromInput,
{
    #[inline]
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        let index = p.index;
        let mut output = arrayvec::ArrayVec::new();

        while let Some(element) = T::try_from_input(p)? {
            if output.remaining_capacity() == 0 {
                return Err(IStrError::new(index..p.index, ErrorKind::ArrayCapacity(N)));
            }

            output.push(element);
        }

        Ok(Some(output))
    }
}

impl<T> FromInput for Vec<T>
where
    T: FromInput,
{
    #[inline]
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
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
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        let it = p.splitn(D as u8);

        let Some(out) = T::from_input_iter(it)? else {
            return Ok(None);
        };

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
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
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
    fn from_input_iter<I>(mut it: I) -> Result<Option<Self>>
    where
        I: InputIterator,
    {
        let index = it.index();
        let mut array = ArrayVec::new();

        while array.remaining_capacity() > 0 {
            let Some(mut value) = it.next() else {
                return Ok(None);
            };

            let Some(value) = T::try_from_input(&mut value)? else {
                return Ok(None);
            };

            array.push(value);
        }

        match array.into_inner() {
            Ok(array) => Ok(Some(array)),
            Err(array) => {
                return Err(IStrError::new(
                    index..it.index(),
                    ErrorKind::BadArray(N, array.len()),
                ));
            }
        }
    }
}

#[non_exhaustive]
pub struct Skip;

impl FromInput for Skip {
    #[inline]
    fn try_from_input(_: &mut IStr) -> Result<Option<Self>> {
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
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        let Some((_, value)) = p.try_next_word()? else {
            return Ok(None);
        };

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
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        if p.is_empty() {
            return Ok(None);
        }

        Ok(T::try_from_input(p)?.map(Self))
    }
}

/// Read a single byte.
pub struct B(pub u8);

impl FromInput for B {
    #[inline]
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        let Some(&b) = p.data.get(0) else {
            return Ok(None);
        };

        p.advance(1);
        Ok(Some(Self(b)))
    }
}
