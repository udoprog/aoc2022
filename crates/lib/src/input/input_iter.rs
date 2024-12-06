use core::marker::PhantomData;

use crate::env::Size;
use crate::input::{IStr, Result};

use super::FromInput;

/// Iterator over inputs.
pub trait InputIterator {
    /// Current index of the input iterator.
    fn index(&self) -> Size;

    /// Only take `n` element from the iterator.
    fn take(self, n: usize) -> Take<Self>
    where
        Self: Sized,
    {
        Take { iter: self, n }
    }

    /// Next value as type `T`.
    fn next<T>(&mut self) -> Result<Option<T>>
    where
        T: FromInput,
    {
        let Some(mut value) = self.next_input() else {
            return Ok(None);
        };

        let value = T::from_input(&mut value)?;
        Ok(Some(value))
    }

    #[inline]
    fn iter<T>(self) -> Iter<Self, T>
    where
        Self: Sized,
        T: FromInput,
    {
        Iter {
            iter: self,
            _phantom: PhantomData,
        }
    }

    /// Get next input.
    fn next_input(&mut self) -> Option<IStr>;
}

/// See [InputIterator::take].
pub struct Take<I> {
    iter: I,
    n: usize,
}

impl<I> InputIterator for Take<I>
where
    I: InputIterator,
{
    #[inline]
    fn index(&self) -> Size {
        self.iter.index()
    }

    #[inline]
    fn next_input(&mut self) -> Option<IStr> {
        if self.n == 0 {
            return None;
        }

        self.n -= 1;
        self.iter.next_input()
    }
}

impl<I> InputIterator for &mut I
where
    I: InputIterator,
{
    #[inline]
    fn index(&self) -> Size {
        (**self).index()
    }

    #[inline]
    fn next_input(&mut self) -> Option<IStr> {
        (**self).next_input()
    }
}

/// A typed iterator over an [`InputIterator`].
pub struct Iter<I, T> {
    iter: I,
    _phantom: PhantomData<T>,
}

impl<I, T> Iterator for Iter<I, T>
where
    I: InputIterator,
    T: FromInput,
{
    type Item = Result<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().transpose()
    }
}
