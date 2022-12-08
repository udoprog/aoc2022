use core::marker::PhantomData;
use core::ops::Range;
use core::ptr;

use crate::grid::slice::{column_index_mut, column_index_ref};
use crate::grid::slice::{Column, ColumnMut, Dims, Row, RowMut};

macro_rules! single {
    ($name:ident $item:ident $dim:ident [$($m:tt)*] $(#[$($meta:meta)*])?) => {
        /// Iterator over a slice.
        pub struct $name<'a, T> {
            data: ptr::NonNull<[T]>,
            range: Range<usize>,
            dims: &'a Dims,
            _marker: PhantomData<&'a $($m)* [T]>,
        }

        impl<'a, T> $name<'a, T> {
            #[inline]
            pub(super) fn new(data: ptr::NonNull<[T]>, dims: &'a Dims) -> Self {
                Self {
                    data: ptr::NonNull::from(data),
                    range: 0..dims.$dim,
                    dims,
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, T> Iterator for $name<'a, T> {
            type Item = $item<'a, T>;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                let index = self.range.next()?;
                Some($item::new(self.data, self.dims, index))
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.range.size_hint()
            }

            #[inline]
            fn nth(&mut self, n: usize) -> Option<Self::Item> {
                let index = self.range.nth(n)?;
                Some($item::new(self.data, self.dims, index))
            }
        }

        impl<'a, T> DoubleEndedIterator for $name<'a, T> {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                let index = self.range.next_back()?;
                Some($item::new(self.data, self.dims, index))
            }

            #[inline]
            fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
                let index = self.range.nth_back(n)?;
                Some($item::new(self.data, self.dims, index))
            }
        }

        impl<'a, T> ExactSizeIterator for $name<'a, T> {
            #[inline]
            fn len(&self) -> usize {
                self.range.len()
            }
        }
    }
}

macro_rules! matrix {
    ($name:ident $dim:ident [$($m:tt)*] $fn:ident $(#[$($meta:meta)*])?) => {
        /// Iterator over a row.
        $(#[$($meta)*])*
        pub struct $name<'a, T> {
            data: ptr::NonNull<[T]>,
            dims: &'a Dims,
            current: usize,
            range: Range<usize>,
            _marker: PhantomData<&'a $($m)* [T]>,
        }

        impl<'a, T> $name<'a, T> {
            #[inline]
            pub(super) fn new(data: ptr::NonNull<[T]>, dims: &'a Dims, current: usize) -> Self {
                Self {
                    data,
                    dims,
                    current,
                    range: 0..dims.$dim,
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, T> Iterator for $name<'a, T> {
            type Item = &'a $($m)* T;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                let index = self.range.next()?;
                // SAFETY: we know the data was initialized correctly.
                Some(unsafe { $fn(self.data, self.dims, self.current, index) })
            }

            #[inline]
            fn nth(&mut self, n: usize) -> Option<Self::Item> {
                let index = self.range.nth(n)?;
                // SAFETY: we know the data was initialized correctly.
                Some(unsafe { $fn(self.data, self.dims, self.current, index) })
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.range.size_hint()
            }
        }

        impl<'a, T> DoubleEndedIterator for $name<'a, T> {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                let index = self.range.next_back()?;
                // SAFETY: we know the data was initialized correctly.
                Some(unsafe { $fn(self.data, self.dims, self.current, index) })
            }

            #[inline]
            fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
                let index = self.range.nth_back(n)?;
                // SAFETY: we know the data was initialized correctly.
                Some(unsafe { $fn(self.data, self.dims, self.current, index) })
            }
        }

        impl<'a, T> ExactSizeIterator for $name<'a, T> {
            #[inline]
            fn len(&self) -> usize {
                self.range.len()
            }
        }
    }
}

single!(Rows Row rows [] #[derive(Clone)]);
single!(RowsMut RowMut rows [mut]);
single!(Columns Column columns [] #[derive(Clone)]);
single!(ColumnsMut ColumnMut columns [mut]);
matrix!(ColumnIter rows [] column_index_ref #[derive(Clone)]);
matrix!(ColumnIterMut rows [mut] column_index_mut);
