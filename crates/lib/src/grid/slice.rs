mod iter;
pub use self::iter::{ColumnIter, ColumnIterMut, Columns, ColumnsMut, Rows, RowsMut};

use core::marker::PhantomData;
use core::mem;
use core::ptr;
use core::slice;

use crate::grid::{Grid, GridExt, GridMut, GridSliceMut, GridSliceRef};

#[derive(Clone, Copy)]
pub(self) struct Dims {
    rows: usize,
    columns: usize,
    stride: usize,
}

/// A column into a grid slice.
#[derive(Clone)]
pub struct Column<'a, T> {
    data: ptr::NonNull<[T]>,
    dims: &'a Dims,
    column: usize,
    _marker: PhantomData<&'a [T]>,
}

impl<'a, T> Column<'a, T> {
    fn new(data: ptr::NonNull<[T]>, dims: &'a Dims, column: usize) -> Self {
        Self {
            data,
            dims,
            column,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> GridSliceRef<'a, T> for Column<'a, T> {
    type Iter<'this> = ColumnIter<'this, T> where Self: 'this, T: 'this;

    #[inline]
    fn len(&self) -> usize {
        self.dims.rows
    }

    #[inline]
    fn into_ref(self, index: usize) -> Option<&'a T> {
        if index >= self.dims.rows {
            return None;
        }

        Some(unsafe { column_index_ref(self.data, self.dims, self.column, index) })
    }

    #[inline]
    fn get(&self, index: usize) -> Option<&T> {
        if index >= self.dims.rows {
            return None;
        }

        Some(unsafe { column_index_ref(self.data, self.dims, self.column, index) })
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        ColumnIter::new(self.data, self.dims, self.column)
    }
}

impl<'a, T> IntoIterator for Column<'a, T> {
    type Item = &'a T;
    type IntoIter = ColumnIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ColumnIter::new(self.data, self.dims, self.column)
    }
}

#[derive(Clone)]
pub struct Row<'a, T> {
    data: ptr::NonNull<[T]>,
    dims: &'a Dims,
    row: usize,
    _marker: PhantomData<&'a [T]>,
}

impl<'a, T> Row<'a, T> {
    fn new(data: ptr::NonNull<[T]>, dims: &'a Dims, row: usize) -> Self {
        Self {
            data,
            dims,
            row,
            _marker: PhantomData,
        }
    }
}

impl<T> AsRef<[T]> for Row<'_, T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        // SAFETY: the layout of a row is exactly compatible with a slice.
        unsafe { row_slice_ref(self.data, self.dims, self.row) }
    }
}

impl<'a, T> GridSliceRef<'a, T> for Row<'a, T> {
    type Iter<'this> = slice::Iter<'this, T> where Self: 'this, T: 'this;

    #[inline]
    fn len(&self) -> usize {
        self.dims.columns
    }

    #[inline]
    fn into_ref(self, index: usize) -> Option<&'a T> {
        if index >= self.dims.columns {
            return None;
        }

        // SAFETY: we know the data was initialized correctly.
        Some(unsafe { row_index_ref(self.data, self.dims, self.row, index) })
    }

    #[inline]
    fn get(&self, index: usize) -> Option<&T> {
        if index >= self.dims.columns {
            return None;
        }

        // SAFETY: we know the data was initialized correctly.
        Some(unsafe { row_index_ref(self.data, self.dims, self.row, index) })
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        // SAFETY: the layout of a row is exactly compatible with a slice.
        unsafe { row_slice_ref(self.data, self.dims, self.row).iter() }
    }
}

impl<'a, T> IntoIterator for Row<'a, T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        // SAFETY: the layout of a row is exactly compatible with a slice.
        unsafe { row_slice_ref(self.data, self.dims, self.row).iter() }
    }
}

pub struct ColumnMut<'a, T> {
    data: ptr::NonNull<[T]>,
    dims: &'a Dims,
    column: usize,
    _marker: PhantomData<&'a mut [T]>,
}

impl<'a, T> ColumnMut<'a, T> {
    fn new(data: ptr::NonNull<[T]>, dims: &'a Dims, column: usize) -> Self {
        Self {
            data,
            dims,
            column,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> GridSliceMut<'a, T> for ColumnMut<'a, T> {
    type IterMut<'this> = ColumnIterMut<'this, T> where Self: 'this, T: 'this;

    #[inline]
    fn into_mut(self, index: usize) -> Option<&'a mut T> {
        if index >= self.dims.rows {
            return None;
        }

        unsafe { Some(column_index_mut(self.data, self.dims, self.column, index)) }
    }

    #[inline]
    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.dims.rows {
            return None;
        }

        unsafe { Some(column_index_mut(self.data, self.dims, self.column, index)) }
    }

    #[inline]
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        ColumnIterMut::new(self.data, self.dims, self.column)
    }
}

impl<'a, T> GridSliceRef<'a, T> for ColumnMut<'a, T> {
    type Iter<'this> = ColumnIter<'this, T> where Self: 'this, T: 'this;

    #[inline]
    fn len(&self) -> usize {
        self.dims.rows
    }

    #[inline]
    fn into_ref(self, index: usize) -> Option<&'a T> {
        if index >= self.dims.rows {
            return None;
        }

        Some(unsafe { column_index_ref(self.data, self.dims, self.column, index) })
    }

    #[inline]
    fn get(&self, index: usize) -> Option<&T> {
        if index >= self.dims.rows {
            return None;
        }

        Some(unsafe { column_index_ref(self.data, self.dims, self.column, index) })
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        ColumnIter::new(self.data, self.dims, self.column)
    }
}

impl<'a, T> IntoIterator for ColumnMut<'a, T> {
    type Item = &'a mut T;
    type IntoIter = ColumnIterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ColumnIterMut::new(self.data, self.dims, self.column)
    }
}

/// Mutable access to a row.
pub struct RowMut<'a, T> {
    data: ptr::NonNull<[T]>,
    dims: &'a Dims,
    row: usize,
    _marker: PhantomData<&'a mut [T]>,
}

impl<'a, T> RowMut<'a, T> {
    fn new(data: ptr::NonNull<[T]>, dims: &'a Dims, row: usize) -> Self {
        Self {
            data,
            dims,
            row,
            _marker: PhantomData,
        }
    }
}

impl<T> AsMut<[T]> for RowMut<'_, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        // SAFETY: the layout of a row is exactly compatible with a slice.
        unsafe { row_slice_mut(self.data, self.dims, self.row) }
    }
}

impl<'a, T> GridSliceMut<'a, T> for RowMut<'a, T> {
    type IterMut<'this> = slice::IterMut<'this, T> where Self: 'this, T: 'this;

    #[inline]
    fn into_mut(self, index: usize) -> Option<&'a mut T> {
        if index >= self.dims.columns {
            return None;
        }

        // SAFETY: We're bounds checking these slices during construction.
        unsafe { Some(row_index_mut(self.data, self.dims, self.row, index)) }
    }

    #[inline]
    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.dims.columns {
            return None;
        }

        // SAFETY: We're bounds checking these slices during construction.
        unsafe { Some(row_index_mut(self.data, self.dims, self.row, index)) }
    }

    #[inline]
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        // SAFETY: the layout of a row is exactly compatible with a slice.
        unsafe { row_slice_mut(self.data, self.dims, self.row).iter_mut() }
    }
}

impl<'a, T> GridSliceRef<'a, T> for RowMut<'a, T> {
    type Iter<'this> = slice::Iter<'this, T> where Self: 'this, T: 'this;

    #[inline]
    fn len(&self) -> usize {
        self.dims.columns
    }

    #[inline]
    fn into_ref(self, index: usize) -> Option<&'a T> {
        if index >= self.dims.columns {
            return None;
        }

        // SAFETY: we know the data was initialized correctly.
        Some(unsafe { row_index_ref(self.data, self.dims, self.row, index) })
    }

    #[inline]
    fn get(&self, index: usize) -> Option<&T> {
        if index >= self.dims.columns {
            return None;
        }

        // SAFETY: we know the data was initialized correctly.
        Some(unsafe { row_index_ref(self.data, self.dims, self.row, index) })
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        // SAFETY: the layout of a row is exactly compatible with a slice.
        unsafe { row_slice_ref(self.data, self.dims, self.row).iter() }
    }
}

impl<'a, T> IntoIterator for RowMut<'a, T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        // SAFETY: the layout of a row is exactly compatible with a slice.
        unsafe { row_slice_mut(self.data, self.dims, self.row).iter_mut() }
    }
}

/// Mutable slice grid.
pub struct SliceGridMut<'a, T> {
    data: ptr::NonNull<[T]>,
    dims: Dims,
    _marker: PhantomData<&'a mut [T]>,
}

impl<'a, T> Grid<T> for SliceGridMut<'a, T> {
    type Row<'this> = Row<'this, T> where Self: 'this;
    type Column<'this> = Column<'this, T> where Self: 'this;
    type Rows<'this> = Rows<'this, T> where Self: 'this;
    type Columns<'this> = Columns<'this, T> where Self: 'this;

    #[inline]
    fn row(&self, row: usize) -> Option<Self::Row<'_>> {
        if row >= self.dims.rows {
            return None;
        }

        Some(Row::new(self.data, &self.dims, row))
    }

    #[inline]
    fn column(&self, column: usize) -> Option<Self::Column<'_>> {
        if column >= self.dims.columns {
            return None;
        }

        Some(Column::new(self.data, &self.dims, column))
    }

    #[inline]
    fn rows(&self) -> Self::Rows<'_> {
        Rows::new(self.data, &self.dims)
    }

    #[inline]
    fn columns(&self) -> Self::Columns<'_> {
        Columns::new(self.data, &self.dims)
    }

    #[inline]
    fn rows_len(&self) -> usize {
        self.dims.rows
    }

    #[inline]
    fn columns_len(&self) -> usize {
        self.dims.columns
    }
}

impl<'a, T> GridMut<T> for SliceGridMut<'a, T> {
    type RowMut<'this> = RowMut<'this, T> where Self: 'this;
    type ColumnMut<'this> = ColumnMut<'this, T> where Self: 'this;
    type RowsMut<'this> = RowsMut<'this, T> where Self: 'this;
    type ColumnsMut<'this> = ColumnsMut<'this, T> where Self: 'this;

    #[inline]
    fn rows_mut(&mut self) -> Self::RowsMut<'_> {
        RowsMut::new(self.data, &self.dims)
    }

    #[inline]
    fn columns_mut(&mut self) -> Self::ColumnsMut<'_> {
        ColumnsMut::new(self.data, &self.dims)
    }

    #[inline]
    fn row_mut(&mut self, row: usize) -> Option<Self::RowMut<'_>> {
        if row >= self.dims.rows {
            return None;
        }

        Some(RowMut::new(self.data, &self.dims, row))
    }

    #[inline]
    fn column_mut(&mut self, column: usize) -> Option<Self::ColumnMut<'_>> {
        if column >= self.dims.columns {
            return None;
        }

        Some(ColumnMut::new(self.data, &self.dims, column))
    }
}

/// Mutable slice grid.
pub struct SliceGrid<'a, T> {
    data: ptr::NonNull<[T]>,
    dims: Dims,
    _marker: PhantomData<&'a [T]>,
}

impl<'a, T> Clone for SliceGrid<'a, T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            dims: self.dims,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Copy for SliceGrid<'a, T> {}

impl<'a, T> Grid<T> for SliceGrid<'a, T> {
    type Row<'this> = Row<'this, T> where Self: 'this;
    type Column<'this> = Column<'this, T> where Self: 'this;
    type Rows<'this> = Rows<'this, T> where Self: 'this;
    type Columns<'this> = Columns<'this, T> where Self: 'this;

    #[inline]
    fn rows(&self) -> Self::Rows<'_> {
        Rows::new(self.data, &self.dims)
    }

    #[inline]
    fn columns(&self) -> Self::Columns<'_> {
        Columns::new(self.data, &self.dims)
    }

    #[inline]
    fn row(&self, row: usize) -> Option<Self::Row<'_>> {
        if row >= self.dims.rows {
            return None;
        }

        Some(Row::new(self.data, &self.dims, row))
    }

    #[inline]
    fn column(&self, column: usize) -> Option<Self::Column<'_>> {
        if column >= self.dims.columns {
            return None;
        }

        Some(Column::new(self.data, &self.dims, column))
    }

    #[inline]
    fn rows_len(&self) -> usize {
        self.dims.rows
    }

    #[inline]
    fn columns_len(&self) -> usize {
        self.dims.columns
    }
}

impl<T> GridExt<T> for [T] {
    type Grid<'this> = SliceGrid<'this, T> where Self: 'this;
    type GridMut<'this> = SliceGridMut<'this, T> where Self: 'this;

    /// Treat the slice as a grid.
    #[inline]
    fn as_grid_with_stride(&self, columns: usize, stride: usize) -> SliceGrid<T> {
        let stride = columns.saturating_add(stride);
        assert!(columns != 0, "columns must be non-zero");
        assert!(
            columns <= stride,
            "columns {columns} must be less or equal to stride {stride}"
        );

        let rem = self.len() % stride;
        let len = self.len().saturating_sub(rem);
        let rows = len / stride;

        SliceGrid {
            data: ptr::NonNull::from(&self[..len]),
            dims: Dims {
                rows,
                columns,
                stride,
            },
            _marker: PhantomData,
        }
    }

    /// Treat the slice as a grid.
    #[inline]
    fn as_grid_mut_with_stride(&mut self, columns: usize, stride: usize) -> SliceGridMut<T> {
        let stride = columns.saturating_add(stride);
        assert!(columns != 0, "stride must be non-zero");
        assert!(
            columns <= stride,
            "columns {columns} must be less or equal to stride {stride}"
        );

        let rem = self.len() % stride;
        let len = self.len().saturating_sub(rem);
        let rows = len / stride;

        SliceGridMut {
            data: ptr::NonNull::from(&mut self[..len]),
            dims: Dims {
                rows,
                columns,
                stride,
            },
            _marker: PhantomData,
        }
    }
}

// Utility functions below.
//
// A note on ZST: The base address of the underlying slice can always be treated
// as the address to a reference of the ZST.
//
// Mutability also isn't a concern, because there is nothing to mutate for ZSTs.
//
// Don't like it? Bring it up with management:
//
// ```
// fn main() {
//     let array = [(); 1024];
//     let mut it = array.iter();
//     let first = it.next().unwrap();
//     let second = it.next().unwrap();
//     let base = array[..].as_ptr();
//     assert!(std::ptr::eq(&array[0], base));
//     assert!(std::ptr::eq(&array[0], first));
//     assert!(std::ptr::eq(&array[0], second));
//     assert!(std::ptr::eq(&array[0], &array[512]));
// }
// ```

#[inline]
unsafe fn row_index_ref<'a, T>(
    data: ptr::NonNull<[T]>,
    dims: &Dims,
    row: usize,
    index: usize,
) -> &'a T {
    if mem::size_of::<T>() == 0 {
        &*(data.as_ptr() as *const T)
    } else {
        &*(data.as_ptr() as *const T).add((row * dims.stride) + index)
    }
}

#[inline]
unsafe fn row_index_mut<'a, T>(
    data: ptr::NonNull<[T]>,
    dims: &Dims,
    row: usize,
    index: usize,
) -> &'a mut T {
    if mem::size_of::<T>() == 0 {
        &mut *(data.as_ptr() as *mut T)
    } else {
        &mut *(data.as_ptr() as *mut T).add((row * dims.stride) + index)
    }
}

#[inline]
unsafe fn row_slice_ref<'a, T>(data: ptr::NonNull<[T]>, dims: &Dims, row: usize) -> &'a [T] {
    let ptr = if mem::size_of::<T>() == 0 {
        data.as_ptr() as *const T
    } else {
        (data.as_ptr() as *const T).add(row * dims.stride)
    };

    slice::from_raw_parts(ptr, dims.columns)
}

#[inline]
unsafe fn row_slice_mut<'a, T>(data: ptr::NonNull<[T]>, dims: &Dims, row: usize) -> &'a mut [T] {
    let ptr = if mem::size_of::<T>() == 0 {
        data.as_ptr() as *mut T
    } else {
        (data.as_ptr() as *mut T).add(row * dims.stride)
    };

    slice::from_raw_parts_mut(ptr, dims.columns)
}

#[inline]
unsafe fn column_index_ref<'a, T>(
    data: ptr::NonNull<[T]>,
    dims: &Dims,
    column: usize,
    index: usize,
) -> &'a T {
    if mem::size_of::<T>() == 0 {
        &*(data.as_ptr() as *const T)
    } else {
        &*(data.as_ptr() as *const T).add((index * dims.stride) + column)
    }
}

#[inline]
unsafe fn column_index_mut<'a, T>(
    data: ptr::NonNull<[T]>,
    dims: &Dims,
    column: usize,
    index: usize,
) -> &'a mut T {
    if mem::size_of::<T>() == 0 {
        &mut *(data.as_ptr() as *mut T)
    } else {
        &mut *(data.as_ptr() as *mut T).add((index * dims.stride) + column)
    }
}
