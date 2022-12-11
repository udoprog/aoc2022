use crate::env::Size;
use crate::input::IStr;

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

    /// Get the next chunk.
    fn next(&mut self) -> Option<IStr>;
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
    fn next(&mut self) -> Option<IStr> {
        dbg!(self.n);

        if self.n == 0 {
            return None;
        }

        self.n -= 1;
        self.iter.next()
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
    fn next(&mut self) -> Option<IStr> {
        (**self).next()
    }
}
