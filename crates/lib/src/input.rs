//! Input parser.

mod error;
pub mod input_iter;
mod iter;

use core::fmt;
use core::mem;
use core::ops;
use core::str::from_utf8;

use arrayvec::ArrayVec;
use bstr::BStr;
use ringbuffer::ConstGenericRingBuffer;

pub use self::error::{Custom, ErrorKind, IStrError};
pub use self::iter::Iter;

pub(self) type Result<T> = std::result::Result<T, IStrError>;
use self::input_iter::InputIterator;
use crate::env::Size;

pub(crate) const NL: u8 = b'\n';

/// Helper to parse input.
#[derive(Clone, Copy)]
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
    pub fn split<'a, F>(&'a mut self, finder: F) -> impl InputIterator + 'a
    where
        F: 'a + Fn(&[u8]) -> Option<(usize, usize)>,
    {
        return SplitInputIterator {
            input: self,
            finder,
        };

        /// Input iterator produced by [IStr::split].
        struct SplitInputIterator<'a, F> {
            input: &'a mut IStr,
            finder: F,
        }

        impl<'a, F> InputIterator for SplitInputIterator<'a, F>
        where
            F: Fn(&[u8]) -> Option<(usize, usize)>,
        {
            #[inline]
            fn index(&self) -> Size {
                self.input.index
            }

            #[inline]
            fn next(&mut self) -> Option<IStr> {
                self.input.split_once_at(|bytes| (self.finder)(bytes))
            }
        }
    }

    /// Try to peek for next value `T`.
    #[inline]
    pub fn peek<T>(&mut self) -> Result<Option<T>>
    where
        T: FromInput,
    {
        let data = self.data;

        let Some(value) = self.try_next()? else {
            return Ok(None);
        };

        self.data = data;
        Ok(Some(value))
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
        let data = self.data;

        let Some(mut line) = self.split_once(NL) else {
            return Ok(None);
        };

        let Some(output) = line.try_next()? else {
            self.data = data;
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
    #[inline]
    pub fn try_next_word<T>(&mut self) -> Result<Option<(Size, T)>>
    where
        T: FromInput,
    {
        self.try_next_with(u8::is_ascii_whitespace)
    }

    /// Try to parse the next word.
    #[inline]
    pub fn try_next_with<T>(&mut self, find: fn(&u8) -> bool) -> Result<Option<(Size, T)>>
    where
        T: FromInput,
    {
        let s = self.find(0, |b| !u8::is_ascii_whitespace(b));
        let n = self.find(s, find);

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

    #[inline]
    fn split_once_at<T>(&mut self, find: T) -> Option<IStr>
    where
        T: FnOnce(&[u8]) -> Option<(usize, usize)>,
    {
        if self.data.is_empty() {
            return None;
        }

        let Some((at, stride)) = find(self.data) else {
            self.index.advance(self.data.len());
            let data = mem::take(&mut self.data);
            return Some(IStr::new(data, self.index));
        };

        let data = self.data.get(..at)?;
        let n = at.checked_add(stride)?;
        let index = self.index.checked_add(Size::new(n))?;
        self.advance(n);
        Some(IStr::new(data, index))
    }

    /// Split once at the given byte or until the end of string, returning the new IStr associated with the split.
    #[inline]
    fn split_once(&mut self, b: u8) -> Option<IStr> {
        self.split_once_at(|data| Some((memchr::memchr(b, data)?, 1)))
    }

    /// Find by predicate.
    #[inline]
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

impl fmt::Debug for IStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        BStr::new(self.data).fmt(f)
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

/// Parse something from a pair of inputs.
pub trait FromInputIter: Sized {
    /// Optionally try to confuse input ignoring leading whitespace by default.
    fn from_input_iter<I>(it: &mut I) -> Result<Option<Self>>
    where
        I: InputIterator;
}

impl<T> FromInput for Option<T>
where
    T: FromInput,
{
    #[inline]
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        let Some(value) = T::try_from_input(p)? else {
            return Ok(Some(None));
        };

        Ok(Some(Some(value)))
    }
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
            fn from_input_iter<I>(it: &mut I) -> Result<Option<Self>>
            where
                I: InputIterator
            {
                let Some(mut $first_id) = it.next() else {
                    return Ok(None);
                };

                $(
                    let Some(mut $rest_id) = it.next() else {
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

tuple!(1 => T0 t0);
tuple!(2 => T0 t0, T1 t1);
tuple!(3 => T0 t0, T1 t1, T2 t2);
tuple!(4 => T0 t0, T1 t1, T2 t2, T3 t3);
tuple!(5 => T0 t0, T1 t1, T2 t2, T3 t3, T4 t4);
tuple!(6 => T0 t0, T1 t1, T2 t2, T3 t3, T4 t4, T5 t5);
tuple!(7 => T0 t0, T1 t1, T2 t2, T3 t3, T4 t4, T5 t5, T6 t6);
tuple!(8 => T0 t0, T1 t1, T2 t2, T3 t3, T4 t4, T5 t5, T6 t6, T7 t7);
tuple!(9 => T0 t0, T1 t1, T2 t2, T3 t3, T4 t4, T5 t5, T6 t6, T7 t7, T8 t8);
tuple!(10 => T0 t0, T1 t1, T2 t2, T3 t3, T4 t4, T5 t5, T6 t6, T7 t7, T8 t8, T9 t9);
tuple!(11 => T0 t0, T1 t1, T2 t2, T3 t3, T4 t4, T5 t5, T6 t6, T7 t7, T8 t8, T9 t9, T10 t10);
tuple!(12 => T0 t0, T1 t1, T2 t2, T3 t3, T4 t4, T5 t5, T6 t6, T7 t7, T8 t8, T9 t9, T10 t10, T11 t11);
tuple!(13 => T0 t0, T1 t1, T2 t2, T3 t3, T4 t4, T5 t5, T6 t6, T7 t7, T8 t8, T9 t9, T10 t10, T11 t11, T12 t12);
tuple!(14 => T0 t0, T1 t1, T2 t2, T3 t3, T4 t4, T5 t5, T6 t6, T7 t7, T8 t8, T9 t9, T10 t10, T11 t11, T12 t12, T13 t13);

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
integer!(num::bigint::BigInt, NotInteger);
integer!(num::bigint::BigUint, NotInteger);

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
        Ok(Some(*p))
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
        p.try_line()
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
        let index = p.index();
        let mut array = ArrayVec::<T, N>::new();

        while array.remaining_capacity() > 0 {
            let value = p.next::<T>()?;
            array.push(value);
        }

        match array.into_inner() {
            Ok(array) => Ok(Some(array)),
            Err(array) => Err(IStrError::new(
                index..p.index(),
                ErrorKind::BadArray(N, array.len()),
            )),
        }
    }
}

impl<T, const N: usize> FromInput for ArrayVec<T, N>
where
    T: FromInput,
{
    #[inline]
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        let index = p.index;
        let mut output = ArrayVec::new();

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
pub struct Split<const D0: char, T>(pub T);

impl<const D0: char, T> FromInput for Split<D0, T>
where
    T: FromInputIter,
{
    #[inline]
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        let mut string = [0u8; 4];
        let string = D0.encode_utf8(&mut string);
        let finder = memchr::memmem::Finder::new(string);
        let mut it = p.split(|bytes| Some((finder.find(bytes)?, string.len())));

        let Some(out) = T::from_input_iter(&mut it)? else {
            return Ok(None);
        };

        Ok(Some(Self(out)))
    }
}

/// Split on pair of characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Split2<const D0: char, const D1: char, T>(pub T);

impl<const D0: char, const D1: char, T> FromInput for Split2<D0, D1, T>
where
    T: FromInputIter,
{
    #[inline]
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        let mut string = [0u8; 8];
        let d0 = D0.encode_utf8(&mut string[0..]).len();
        let d1 = D1.encode_utf8(&mut string[d0..]).len();
        let string = &string[..d0 + d1];
        let finder = memchr::memmem::Finder::new(string);
        let mut it = p.split(|bytes| Some((finder.find(bytes)?, string.len())));

        let Some(out) = T::from_input_iter(&mut it)? else {
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
    fn from_input_iter<I>(it: &mut I) -> Result<Option<Self>>
    where
        I: InputIterator,
    {
        let index = it.index();
        let mut array = ArrayVec::<T, N>::new();

        while array.remaining_capacity() > 0 {
            let Some(mut value) = it.next() else {
                return Err(IStrError::new(
                    index..it.index(),
                    ErrorKind::BadArray(N, array.len()),
                ));
            };

            array.push(value.next()?);
        }

        match array.into_inner() {
            Ok(array) => Ok(Some(array)),
            Err(array) => Err(IStrError::new(
                index..it.index(),
                ErrorKind::BadArray(N, array.len()),
            )),
        }
    }
}

impl<const N: usize, T> FromInputIter for ArrayVec<T, N>
where
    T: FromInput,
{
    #[inline]
    fn from_input_iter<I>(it: &mut I) -> Result<Option<Self>>
    where
        I: InputIterator,
    {
        let index = it.index();
        let mut array = ArrayVec::<T, N>::new();

        while let Some(mut value) = it.next() {
            let Some(value) = T::try_from_input(&mut value)? else {
                return Ok(None);
            };

            if let Err(..) = array.try_push(value) {
                return Err(IStrError::new(
                    index..it.index(),
                    ErrorKind::ArrayCapacity(N),
                ));
            }
        }

        Ok(Some(array))
    }
}

impl<const N: usize, T> FromInputIter for ConstGenericRingBuffer<T, N>
where
    T: FromInput,
{
    #[inline]
    fn from_input_iter<I>(it: &mut I) -> Result<Option<Self>>
    where
        I: InputIterator,
    {
        use ringbuffer::{RingBuffer, RingBufferWrite};

        let index = it.index();
        let mut array = ConstGenericRingBuffer::new();

        while let Some(mut value) = it.next() {
            let Some(value) = T::try_from_input(&mut value)? else {
                return Ok(None);
            };

            if array.is_full() {
                return Err(IStrError::new(
                    index..it.index(),
                    ErrorKind::RingbufCapacity(N),
                ));
            }

            array.push(value);
        }

        Ok(Some(array))
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

/// Parse a word of input, which parses until we reach a whitespace or control character.
pub struct Digits<T = Skip>(pub T);

impl<T> FromInput for Digits<T>
where
    T: FromInput,
{
    #[inline]
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        let Some((_, value)) = p.try_next_with(|d| !d.is_ascii_digit())? else {
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
        let Some(&b) = p.data.first() else {
            return Ok(None);
        };

        p.advance(1);
        Ok(Some(Self(b)))
    }
}
