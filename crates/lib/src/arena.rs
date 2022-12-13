use core::alloc::Layout;
use core::cell::Cell;
use core::fmt;
use core::marker::PhantomData;
use core::mem;
use core::ptr;
use core::slice;

#[derive(Debug)]
#[non_exhaustive]
pub struct ArenaWriteSliceOutOfBounds {
    pub index: usize,
}

impl fmt::Display for ArenaWriteSliceOutOfBounds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "arena allocation at index {} out of bounds", self.index)
    }
}

impl std::error::Error for ArenaWriteSliceOutOfBounds {}

#[derive(Debug)]
#[non_exhaustive]
pub struct ArenaAllocError {
    pub requested: usize,
}

impl fmt::Display for ArenaAllocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "arena allocation of size {} failed", self.requested)
    }
}

impl std::error::Error for ArenaAllocError {}

/// An arena allocator.
pub struct Arena<'a> {
    start: Cell<*mut u8>,
    end: Cell<*mut u8>,
    _marker: PhantomData<&'a mut [u8]>,
}

impl<'a> Arena<'a> {
    /// Construct a new empty arena allocator.
    ///
    /// Since we're taking a mutable reference to the array vector we're
    /// preventing it from moving and being de-allocated.
    pub fn new(storage: &'a mut [u8]) -> Self {
        let range = storage.as_mut_ptr_range();

        Self {
            start: Cell::new(range.start),
            end: Cell::new(range.end),
            _marker: PhantomData,
        }
    }

    /// Allocate a new object of the given type.
    ///
    /// ```
    /// use lib::prelude::*;
    ///
    /// let mut data = [0; 128];
    /// let arena = Arena::new(&mut data);
    ///
    /// let a = arena.alloc(4u32)?;
    /// let b = arena.alloc(8u32)?;
    /// std::mem::swap(a, b);
    /// assert_eq!(*a, 8);
    /// assert_eq!(*b, 4);
    /// # Ok::<_, Error>(())
    /// ```
    pub fn alloc<T>(&self, object: T) -> Result<&mut T, ArenaAllocError> {
        assert!(!mem::needs_drop::<T>());

        let mem = self.alloc_raw(Layout::for_value::<T>(&object))? as *mut T;

        unsafe {
            // Write into uninitialized memory.
            ptr::write(mem, object);
            Ok(&mut *mem)
        }
    }

    /// Allocate a slice with at most the given `len` number of elements.
    ///
    /// Note that this will waste memory if `len` number of elements isn't used!
    ///
    /// # Examples
    ///
    /// ```
    /// use lib::prelude::*;
    ///
    /// let mut data = [0; 128];
    /// let arena = Arena::new(&mut data);
    ///
    /// let mut it = arena.alloc_iter(5)?;
    /// it.write(4u32)?;
    /// it.write(8u32)?;
    /// let slice = it.finish();
    ///
    /// assert_eq!(slice, &[4, 8]);
    /// # Ok::<_, Error>(())
    /// ```
    pub fn alloc_iter<T>(&self, len: usize) -> Result<AllocIter<'_, T>, ArenaAllocError> {
        assert!(!mem::needs_drop::<T>(), "cannot allocate drop element");

        let mem = if len == 0 {
            ptr::null_mut()
        } else {
            self.alloc_raw(Layout::array::<T>(len).unwrap())? as *mut T
        };

        Ok(AllocIter {
            mem,
            index: 0,
            len,
            _marker: PhantomData,
        })
    }

    #[inline]
    fn alloc_raw_without_grow(&self, layout: Layout) -> Option<*mut u8> {
        let start = addr(self.start.get());
        let old_end = self.end.get();
        let end = addr(old_end);

        let align = layout.align();
        let bytes = layout.size();

        let new_end = end.checked_sub(bytes)? & !(align - 1);

        if start > new_end {
            return None;
        }

        let new_end = with_addr(old_end, new_end);
        self.end.set(new_end);
        Some(new_end)
    }

    #[inline]
    pub fn alloc_raw(&self, layout: Layout) -> Result<*mut u8, ArenaAllocError> {
        assert!(layout.size() != 0);

        if let Some(a) = self.alloc_raw_without_grow(layout) {
            return Ok(a);
        }

        Err(ArenaAllocError {
            requested: layout.size(),
        })
    }
}

#[inline]
fn addr(this: *mut u8) -> usize {
    this as usize
}

#[inline]
fn with_addr(this: *mut u8, a: usize) -> *mut u8 {
    let this_addr = addr(this) as isize;
    let dest_addr = a as isize;
    let offset = dest_addr.wrapping_sub(this_addr);
    this.wrapping_offset(offset)
}

/// An iterator allocator which once finished produces a slice.
pub struct AllocIter<'a, T> {
    mem: *mut T,
    index: usize,
    len: usize,
    _marker: PhantomData<&'a mut [T]>,
}

impl<'a, T> AllocIter<'a, T> {
    /// Write the next element into the slice.
    pub fn write(&mut self, object: T) -> Result<(), ArenaWriteSliceOutOfBounds> {
        // Sanity check is necessary to ensure memory safety.
        if self.index >= self.len {
            return Err(ArenaWriteSliceOutOfBounds { index: self.index });
        }

        unsafe {
            ptr::write(self.mem.add(self.index), object);
            self.index += 1;
            Ok(())
        }
    }

    /// Finalize the iterator being written and return the appropriate closure.
    pub fn finish(self) -> &'a mut [T] {
        if self.mem.is_null() {
            return &mut [];
        }

        unsafe { slice::from_raw_parts_mut(self.mem, self.index) }
    }
}
