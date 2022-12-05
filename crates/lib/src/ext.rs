pub trait SliceExt<O> {
    /// Get two values out of a slice, assuming they are disjoint and in bounds.
    /// Else will return `None`.
    fn get_mut2(&mut self, a: usize, b: usize) -> Option<(&mut O, &mut O)>;
}

impl<T> SliceExt<T> for [T] {
    #[inline]
    fn get_mut2(&mut self, a: usize, b: usize) -> Option<(&mut T, &mut T)> {
        if a == b || a.max(b) >= self.len() {
            return None;
        }

        let ptr = self.as_mut_ptr();

        // SAFETY: we're bounds checking just above.
        unsafe {
            let a = &mut *ptr.add(a);
            let b = &mut *ptr.add(b);
            Some((a, b))
        }
    }
}
