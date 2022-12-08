use core::marker::PhantomData;
use core::ops::Range;
use core::ptr;

use crate::grid::slice::{
    column_index_mut, column_index_ref, row_index_mut, row_index_ref, Column, ColumnMut, Dims, Row,
    RowMut,
};

macro_rules! rows {
    ($name:ident $item:ident [$($m:tt)*]) => {
        /// Iterator over rows in a slice.
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
                    range: 0..dims.rows,
                    dims,
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, T> Iterator for $name<'a, T> {
            type Item = $item<'a, T>;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                let row = self.range.next()?;
                Some($item::new(self.data, self.dims, row))
            }
        }
    }
}

macro_rules! columns {
    ($name:ident $item:ident [$($m:tt)*]) => {
        /// Iterator over columns in a slice.
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
                    range: 0..dims.columns,
                    dims,
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, T> Iterator for $name<'a, T> {
            type Item = $item<'a, T>;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                let column = self.range.next()?;
                Some($item::new(self.data, self.dims, column))
            }
        }
    }
}

macro_rules! row_iter {
    ($name:ident [$($m:tt)*] $fn:ident) => {
        /// Iterator over a row.
        pub struct $name<'a, T> {
            data: ptr::NonNull<[T]>,
            dims: &'a Dims,
            row: usize,
            range: Range<usize>,
            _marker: PhantomData<&'a $($m)* [T]>,
        }

        impl<'a, T> $name<'a, T> {
            #[inline]
            pub(super) fn new(data: ptr::NonNull<[T]>, dims: &'a Dims, row: usize) -> Self {
                Self {
                    data,
                    dims,
                    row,
                    range: 0..dims.columns,
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
                Some(unsafe { $fn(self.data, self.dims, self.row, index) })
            }
        }

        impl<'a, T> DoubleEndedIterator for $name<'a, T> {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                let index = self.range.next_back()?;
                // SAFETY: we know the data was initialized correctly.
                Some(unsafe { $fn(self.data, self.dims, self.row, index) })
            }
        }
    }
}

macro_rules! column_iter {
    ($name:ident [$($m:tt)*] $fn:ident) => {
        /// Iterator over columns.
        pub struct $name<'a, T> {
            data: ptr::NonNull<[T]>,
            dims: &'a Dims,
            column: usize,
            range: Range<usize>,
            _marker: PhantomData<&'a $($m)* [T]>,
        }

        impl<'a, T> $name<'a, T> {
            #[inline]
            pub(super) fn new(data: ptr::NonNull<[T]>, dims: &'a Dims, column: usize) -> Self {
                Self {
                    data,
                    dims,
                    column,
                    range: 0..dims.rows,
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, T> Iterator for $name<'a, T> {
            type Item = &'a $($m)* T;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                let index = self.range.next()?;
                Some(unsafe { $fn(self.data, self.dims, self.column, index) })
            }
        }
    };
}

rows!(Rows Row []);
rows!(RowsMut RowMut [mut]);
columns!(Columns Column []);
columns!(ColumnsMut ColumnMut [mut]);
column_iter!(ColumnIter [] column_index_ref);
column_iter!(ColumnIterMut [mut] column_index_mut);
row_iter!(RowIter [] row_index_ref);
row_iter!(RowIterMut [mut] row_index_mut);
