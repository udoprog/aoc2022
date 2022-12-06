use arrayvec::ArrayString;

/// Helper trait to more conveniently test things for equality.
pub trait OutputEq<O = Self>
where
    O: ?Sized,
{
    fn output_eq(&self, other: &O) -> bool;
}

impl<A, B, C, D> OutputEq<(C, D)> for (A, B)
where
    A: OutputEq<C>,
    B: OutputEq<D>,
{
    #[inline]
    fn output_eq(&self, other: &(C, D)) -> bool {
        self.0.output_eq(&other.0) && self.1.output_eq(&other.1)
    }
}

impl<A, B> OutputEq<Option<B>> for Option<A>
where
    A: OutputEq<B>,
{
    #[inline]
    fn output_eq(&self, other: &Option<B>) -> bool {
        match (self, other) {
            (Some(a), Some(b)) => a.output_eq(b),
            (None, None) => true,
            _ => false,
        }
    }
}

impl<const N: usize> OutputEq<ArrayString<N>> for &str {
    #[inline]
    fn output_eq(&self, other: &ArrayString<N>) -> bool {
        other.as_str() == *self
    }
}

impl<const N: usize> OutputEq<&str> for ArrayString<N> {
    #[inline]
    fn output_eq(&self, other: &&str) -> bool {
        *other == self
    }
}

macro_rules! partial_eq {
    ($ty:ty) => {
        impl OutputEq<$ty> for $ty {
            #[inline]
            fn output_eq(&self, other: &Self) -> bool {
                other == self
            }
        }
    };
}

partial_eq!(usize);
partial_eq!(isize);
partial_eq!(u8);
partial_eq!(u16);
partial_eq!(u32);
partial_eq!(u64);
partial_eq!(u128);
partial_eq!(i8);
partial_eq!(i16);
partial_eq!(i32);
partial_eq!(i64);
partial_eq!(i128);
partial_eq!(bool);
partial_eq!(());
