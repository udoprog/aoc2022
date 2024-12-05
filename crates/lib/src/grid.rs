pub mod slice;

mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for [T] {}
}

use arrayvec::ArrayVec;

use self::sealed::Sealed;

pub trait GridExt<T>: Sealed {
    /// Return value as an immutable grid.
    type Grid<'this>: Grid<T>
    where
        Self: 'this,
        T: 'this;

    /// Return value as a mutable grid.
    type GridMut<'this>: GridMut<T>
    where
        Self: 'this,
        T: 'this;

    /// Convert type into grid with a stride of `0`.
    ///
    /// See [GridExt::as_grid_with_stride].
    #[inline]
    fn as_grid(&self, columns: usize) -> Self::Grid<'_> {
        self.as_grid_with_stride(columns, 0)
    }

    /// Convert type into a grid with the given topology.
    ///
    /// The `columns` is the width of a row while `stride` is the number of
    /// elements between each row.
    ///
    /// This allows for specifying a stride that is larger than the number of
    /// columns to offset the grid inside of the data structure.
    fn as_grid_with_stride(&self, columns: usize, stride: usize) -> Self::Grid<'_>;

    /// Convert type into grid with a stride of `0`.
    ///
    /// See [GridExt::as_grid_mut_with_stride].
    #[inline]
    fn as_grid_mut(&mut self, columns: usize) -> Self::GridMut<'_> {
        self.as_grid_mut_with_stride(columns, 0)
    }

    /// Convert type into a grid with the given topology.
    ///
    /// The `columns` is the width of a row while `stride` is the number of
    /// elements between each row.
    ///
    /// This allows for specifying a stride that is larger than the number of
    /// columns to offset the grid inside of the data structure.
    fn as_grid_mut_with_stride(&mut self, columns: usize, stride: usize) -> Self::GridMut<'_>;
}

pub trait Grid<T> {
    /// The column of the grid.
    type Row<'a>: GridSliceRef<'a, T> + AsRef<[T]> + IntoIterator<Item = &'a T>
    where
        Self: 'a,
        T: 'a;

    /// The column of the grid.
    type Column<'a>: GridSliceRef<'a, T> + IntoIterator<Item = &'a T>
    where
        Self: 'a,
        T: 'a;

    /// Construct an iterator over rows in the grid.
    type Rows<'a>: Iterator<Item = Self::Row<'a>>
    where
        Self: 'a,
        T: 'a;

    /// Construct an iterator over columns in the grid.
    type Columns<'a>: Iterator<Item = Self::Column<'a>>
    where
        Self: 'a,
        T: 'a;

    /// Iterate over rows in the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use lib::prelude::*;
    ///
    /// let mut values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    /// let grid = values.as_grid(4);
    ///
    /// let mut it = grid.rows();
    /// assert!(grid.rows().flatten().copied().eq([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]));
    /// ```
    fn rows(&self) -> Self::Rows<'_>;

    /// Iterate over columns in the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use lib::prelude::*;
    ///
    /// let mut values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    /// let grid = values.as_grid(4);
    /// assert!(grid.columns().flatten().copied().eq([1, 5, 9, 2, 6, 10, 3, 7, 11, 4, 8, 12]));
    /// ```
    fn columns(&self) -> Self::Columns<'_>;

    /// Access the specified row in the grid.
    fn row(&self, row: usize) -> Option<Self::Row<'_>>;

    /// Access the specified column in the grid.
    fn column(&self, column: usize) -> Option<Self::Column<'_>>;

    /// Get number of rows in the grid.
    fn rows_len(&self) -> usize;

    /// Get number of columns in the grid.
    fn columns_len(&self) -> usize;

    /// Get the element at the given row and column.
    #[inline]
    #[track_caller]
    fn get(&self, row: usize, column: usize) -> &T {
        match self.row(row).and_then(|row| row.into_ref(column)) {
            Some(value) => value,
            None => panic!("missing row `{row}`, column `{column}`"),
        }
    }

    /// Get the element at the given row and column.
    #[inline]
    fn try_get(&self, row: usize, column: usize) -> Option<&T> {
        self.row(row)?.into_ref(column)
    }

    /// Collect an iterator of rows and columns into an array.
    ///
    /// This collects up until the array is full, an incorrect index is
    /// encountered, or the iterator completes.
    #[inline]
    fn collect<const N: usize>(
        &self,
        it: impl IntoIterator<Item = (usize, usize)>,
    ) -> ArrayVec<T, N>
    where
        T: Copy,
    {
        let mut values = ArrayVec::new();

        for (column, row) in it {
            let Some(value) = self.try_get(row, column) else {
                break;
            };

            if values.try_push(*value).is_err() {
                break;
            }
        }

        values
    }
}

impl<G, T> Grid<T> for &G
where
    G: Grid<T>,
{
    type Row<'a> = G::Row<'a>
    where
        Self: 'a,
        T: 'a;

    type Column<'a> = G::Column<'a>
    where
        Self: 'a,
        T: 'a;

    type Rows<'a> = G::Rows<'a>
    where
        Self: 'a,
        T: 'a;

    type Columns<'a> = G::Columns<'a>
    where
        Self: 'a,
        T: 'a;

    #[inline]
    fn rows(&self) -> Self::Rows<'_> {
        (**self).rows()
    }

    #[inline]
    fn columns(&self) -> Self::Columns<'_> {
        (**self).columns()
    }

    #[inline]
    fn row(&self, row: usize) -> Option<Self::Row<'_>> {
        (**self).row(row)
    }

    #[inline]
    fn column(&self, column: usize) -> Option<Self::Column<'_>> {
        (**self).column(column)
    }

    #[inline]
    fn rows_len(&self) -> usize {
        (**self).rows_len()
    }

    #[inline]
    fn columns_len(&self) -> usize {
        (**self).columns_len()
    }
}

pub trait GridMut<T>: Grid<T> {
    /// The column of the grid.
    type RowMut<'a>: GridSliceMut<'a, T> + AsMut<[T]> + IntoIterator<Item = &'a mut T>
    where
        Self: 'a,
        T: 'a;

    /// The column of the grid.
    type ColumnMut<'a>: GridSliceMut<'a, T> + IntoIterator<Item = &'a mut T>
    where
        Self: 'a,
        T: 'a;

    /// Construct a mutable iterator over rows in the grid.
    type RowsMut<'a>: Iterator<Item = Self::RowMut<'a>>
    where
        Self: 'a,
        T: 'a;

    /// Construct a mutable iterator over columns in the grid.
    type ColumnsMut<'a>: Iterator<Item = Self::ColumnMut<'a>>
    where
        Self: 'a,
        T: 'a;

    /// Mutably iterate over rows in the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use lib::prelude::*;
    ///
    /// let mut values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    /// let data: &mut [u8] = &mut values[..];
    /// let mut grid = data.as_grid_mut(4);
    ///
    /// for (n, row) in grid.rows_mut().enumerate() {
    ///     for c in row {
    ///         *c = n as u8;
    ///     }
    /// }
    ///
    /// assert_eq!(&values[..], &[0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2]);
    /// ```
    fn rows_mut(&mut self) -> Self::RowsMut<'_>;

    /// Mutably iterate over columns in the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use lib::prelude::*;
    ///
    /// let mut values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    /// let data: &mut [u8] = &mut values[..];
    /// let mut grid = data.as_grid_mut(4);
    ///
    /// for (n, row) in grid.columns_mut().enumerate() {
    ///     for c in row {
    ///         *c = n as u8;
    ///     }
    /// }
    ///
    /// assert_eq!(&values[..], &[0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3]);
    /// ```
    fn columns_mut(&mut self) -> Self::ColumnsMut<'_>;

    /// Access the specified row in the grid.
    fn row_mut(&mut self, row: usize) -> Option<Self::RowMut<'_>>;

    /// Access the specified column in the grid.
    fn column_mut(&mut self, column: usize) -> Option<Self::ColumnMut<'_>>;

    /// Get the element at the given row and column.
    #[inline]
    fn get_mut(&mut self, row: usize, column: usize) -> &mut T {
        match self.row_mut(row).and_then(|row| row.into_mut(column)) {
            Some(value) => value,
            None => panic!("missing row `{row}`, column `{column}`"),
        }
    }

    /// Get the element at the given row and column.
    #[inline]
    fn try_get_mut(&mut self, row: usize, column: usize) -> Option<&mut T> {
        self.row_mut(row)?.into_mut(column)
    }
}

/// The slice into a grid.
pub trait GridSliceRef<'a, T: 'a> {
    /// Iterator over the grid slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use lib::prelude::*;
    ///
    /// let data: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    /// let grid = data.as_grid(4);
    ///
    /// grid.row(1).unwrap().into_iter().copied().eq([5, 6, 7, 8]);
    /// grid.column(1).unwrap().into_iter().copied().eq([2, 6, 10]);
    /// ```
    type Iter<'this>: Iterator<Item = &'this T>
    where
        Self: 'this,
        T: 'this;

    /// The length of a grid slice.
    fn len(&self) -> usize;

    /// Test if the grid slice is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Coerce the slice into a reference extending the lifetime that is part of
    /// the trait.
    fn into_ref(self, index: usize) -> Option<&'a T>;

    /// Access the element at the given index.
    ///
    /// # Examples
    ///
    /// ```
    /// use lib::prelude::*;
    ///
    /// let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    /// let grid = data.as_grid(4);
    ///
    /// assert_eq!(grid.try_get(0, 1), Some(&2));
    /// assert_eq!(grid.try_get(1, 1), Some(&6));
    ///
    /// assert_eq!(grid.try_get(2, 0), Some(&9));
    /// assert_eq!(grid.try_get(2, 1), Some(&10));
    ///
    /// assert_eq!(grid.try_get(3, 0), None);
    /// ```
    fn get(&self, index: usize) -> Option<&T>;

    /// Construct an iterator over the slice.
    fn iter(&self) -> Self::Iter<'_>;
}

/// The slice into a grid.
pub trait GridSliceMut<'a, T: 'a>: GridSliceRef<'a, T> {
    /// Mutable iterator of the grid slice.
    type IterMut<'this>: Iterator<Item = &'this mut T>
    where
        Self: 'this,
        T: 'this;

    /// Coerce into the underlying reference.
    fn into_mut(self, index: usize) -> Option<&'a mut T>;

    /// Get the specified value mutably.
    fn get_mut(&mut self, index: usize) -> Option<&mut T>;

    /// Construct a mutable iterator over the slice.
    fn iter_mut(&mut self) -> Self::IterMut<'_>;
}
