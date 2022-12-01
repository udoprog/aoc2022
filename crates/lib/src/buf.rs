use core::mem;
use core::ptr::copy_nonoverlapping;
use core::slice::{from_raw_parts, from_raw_parts_mut};

#[cfg(test)]
mod tests;

/// Fixed-length buffer that can be stored on the heap.
#[derive(Debug)]
pub struct Buf<const N: usize> {
    /// Cursor position.
    head: usize,
    /// Length of the initialized buffer.
    len: usize,
    /// Buffer which might or might not have been initialized.
    mem: [u8; N],
}

impl<const N: usize> Buf<N> {
    /// Construct a new uninitialized buffer.
    pub const fn new() -> Self {
        Self {
            head: 0,
            len: 0,
            mem: [0; N],
        }
    }

    /// Get the current length of the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use lib::Buf;
    ///
    /// let mut b = Buf::<1024>::new();
    /// assert_eq!(b.len(), 0);
    /// assert_eq!(b.remaining(), 1024);
    /// ```
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Get current capacity of the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use lib::Buf;
    ///
    /// let mut b = Buf::<1024>::new();
    /// assert_eq!(b.len(), 0);
    /// assert_eq!(b.remaining(), 1024);
    ///
    /// b.write(&[1, 2, 4, 5]);
    /// assert_eq!(b.len(), 4);
    /// assert_eq!(b.remaining(), 1020);
    /// ```
    #[inline]
    pub const fn remaining(&self) -> usize {
        N - self.len
    }

    /// Get a value at the given index.
    ///
    /// # Examples
    ///
    /// ```
    /// use lib::Buf;
    ///
    /// let mut b = Buf::<16>::new();
    ///
    /// assert_eq!(b.write(&[1, 2, 3]), 3);
    /// assert_eq!(b.get(0), Some(1));
    /// assert_eq!(b.get(1), Some(2));
    /// assert_eq!(b.get(2), Some(3));
    /// assert_eq!(b.get(3), None);
    ///
    /// let mut buf = [0; 2];
    /// assert_eq!(b.read(&mut buf), 2);
    /// assert_eq!(b.write(&[4, 5, 6]), 3);
    ///
    /// assert_eq!(b.get(0), Some(3));
    /// assert_eq!(b.get(1), Some(4));
    /// assert_eq!(b.get(2), Some(5));
    /// assert_eq!(b.get(3), Some(6));
    /// assert_eq!(b.get(4), None);
    /// ```
    pub const fn get(&self, index: usize) -> Option<u8> {
        if index >= self.len {
            return None;
        }

        // SAFETY: implicit bounds check against len above.
        unsafe { Some(*self.mem.as_ptr().add((self.head + index) % N)) }
    }

    /// Try to write the given number of bytes to the buffer, returning the
    /// number of bytes written.
    ///
    /// # Examples
    ///
    /// ```
    /// use lib::Buf;
    ///
    /// let mut b = Buf::<16>::new();
    ///
    /// let mut data = [1, 2, 3, 4, 5];
    /// let mut buf = [0; 8];
    ///
    /// assert_eq!(b.write(&data[..3]), 3);
    /// assert_eq!(b.read(&mut buf[..3]), 3);
    /// assert_eq!(&buf[..3], &[1, 2, 3]);
    /// ```
    pub fn write(&mut self, buf: &[u8]) -> usize {
        let (a, b) = self.as_uninit_slices_mut();

        if a.is_empty() {
            return 0;
        }

        let head_len = a.len().min(buf.len());
        let tail_len = b.len().min(buf.len() - head_len);

        // SAFETY: We're checking that we are in-bound with both the source and
        // destination buffers just above.
        unsafe {
            let src = buf.as_ptr();
            copy_nonoverlapping(src, a.as_mut_ptr(), head_len);
            copy_nonoverlapping(src.add(head_len), b.as_mut_ptr(), tail_len);
        }

        let len = head_len + tail_len;
        self.len += len;
        len
    }

    /// Read into the given buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use lib::Buf;
    ///
    /// let mut b = Buf::<16>::new();
    ///
    /// let mut data = [1, 2, 3, 4, 5];
    /// let mut buf = [0; 8];
    ///
    /// assert_eq!(b.write(&data[..3]), 3);
    /// assert_eq!(b.read(&mut buf[..3]), 3);
    /// assert_eq!(&buf[..3], &[1, 2, 3]);
    /// ```
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let (a, b) = self.as_slices();

        if a.is_empty() {
            return 0;
        }

        let head_len = a.len().min(buf.len());
        let tail_len = b.len().min(buf.len() - head_len);

        // SAFETY: We're checking that we are in-bound with both the source and
        // destination buffers just above.
        unsafe {
            let dst = buf.as_mut_ptr();
            copy_nonoverlapping(a.as_ptr(), dst, head_len);
            copy_nonoverlapping(b.as_ptr(), dst.add(head_len), tail_len);
        }

        let len = head_len + tail_len;
        self.head = (self.head + len) % N;
        self.len -= len;

        // NB: Try and keep data written at the bottom of the buffer is possible.
        if self.len == 0 {
            self.head = 0;
        }

        len
    }

    /// Pop the front element of the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use lib::Buf;
    ///
    /// let mut b = Buf::<16>::new();
    ///
    /// let mut data = [1, 2, 3, 4, 5];
    ///
    /// assert_eq!(b.write(&data[..3]), 3);
    /// assert_eq!(b.pop_front(), Some(1));
    /// assert_eq!(b.pop_front(), Some(2));
    /// assert_eq!(b.pop_front(), Some(3));
    /// assert_eq!(b.pop_front(), None);
    /// ```
    pub fn pop_front(&mut self) -> Option<u8> {
        if self.len == 0 {
            return None;
        }

        let update = (self.head + 1) % N;

        self.len -= 1;
        let head = mem::replace(&mut self.head, update);
        Some(unsafe { *self.mem.as_ptr().add(head) })
    }

    /// Slices constituting the initialized parts of the buffers.
    #[inline]
    fn as_slices(&self) -> (&[u8], &[u8]) {
        // SAFETY: We know this slice is initialized.
        unsafe {
            let end = self.head + self.len;
            let ptr = self.mem.as_ptr();

            if end <= N {
                let a = from_raw_parts(ptr.add(self.head).cast(), self.len);
                (a, &[])
            } else {
                let a = from_raw_parts(ptr.add(self.head).cast(), N - self.head);
                let b = from_raw_parts(ptr.cast(), end % N);
                (a, b)
            }
        }
    }

    /// Slices constituting the uninitialized parts of the buffers.
    #[inline]
    fn as_uninit_slices_mut(&mut self) -> (&mut [u8], &mut [u8]) {
        // SAFETY: We're returning uninited so there is no safety concerns.
        unsafe {
            let end = self.head + self.len;
            let ptr = self.mem.as_mut_ptr();

            if end >= N {
                let a = from_raw_parts_mut(ptr.add(end % N), self.remaining());
                (a, &mut [])
            } else {
                let a = from_raw_parts_mut(ptr.add(end), N - end);
                let b = from_raw_parts_mut(ptr, self.head);
                (a, b)
            }
        }
    }

    /// Get uninit slice.
    pub fn as_uninit_mut(&mut self) -> &mut [u8] {
        // SAFETY: We're returning uninited so there is no safety concerns.
        unsafe {
            let end = self.head + self.len;
            let ptr = self.mem.as_mut_ptr();

            if end >= N {
                from_raw_parts_mut(ptr.add(end % N), self.remaining())
            } else {
                from_raw_parts_mut(ptr.add(end), N - end)
            }
        }
    }

    /// Advance writer marking data as initialized.
    pub fn advance(&mut self, n: usize) {
        self.len += n;
    }
}
