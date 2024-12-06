use std::marker::PhantomData;

use crate::input::{FromInput, IStr, IStrError};

/// Iterator over an [Input].
pub struct Iter<T> {
    input: IStr,
    _marker: PhantomData<T>,
}

impl<T> Iter<T> {
    pub(crate) fn new(input: IStr) -> Self {
        Self {
            input,
            _marker: PhantomData,
        }
    }
}

impl<T> Iterator for Iter<T>
where
    T: FromInput,
{
    type Item = Result<T, IStrError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Option::<T>::from_input(&mut self.input).transpose()
    }
}
