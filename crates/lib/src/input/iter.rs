use std::marker::PhantomData;

use crate::input::{FromInput, Input, InputError};

/// Iterator over an [Input].
pub struct Iter<'a, T> {
    input: &'a mut Input,
    _marker: PhantomData<T>,
}

impl<'a, T> Iter<'a, T> {
    pub(crate) fn new(input: &'a mut Input) -> Self {
        Self {
            input,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: FromInput,
{
    type Item = Result<T, InputError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.input.try_next().transpose()
    }
}
