/// Helper macro to build an input processor.
#[macro_export]
macro_rules! from_input {
    (|$($value:ident)? $(($($pat:tt)*))?: $ty:ty| -> $($rest:tt)*) => {
        $crate::from_input!(|[$($value)? $(($($pat)*))?]: $ty| -> $($rest)*);
    };

    (|[$($value:tt)*]: $ty:ty| -> $out:ident $block:block) => {
        impl $crate::input::FromInput for $out {
            #[inline]
            fn from_input(
                p: &mut $crate::input::IStr,
            ) -> core::result::Result<Self, $crate::input::IStrError> {
                let index = p.index();

                let Some(value) = $crate::input::FromInput::from_input(p)? else {
                    return $crate::input::FromInput::from_empty(p);
                };

                match (|$($value)*: $ty| -> core::result::Result<$out, $crate::input::ErrorKind> {
                    $block
                })(value) {
                    Ok(value) => Ok(value),
                    Err(kind) => Err($crate::input::IStrError::new(index..p.index(), kind)),
                }
            }
        }
    };
}
