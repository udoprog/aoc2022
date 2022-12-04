/// Helper macro to build an input processor.
#[macro_export]
macro_rules! from_input {
    (|$($value:ident)? $(($pat:pat))?: $ty:ty| -> $($rest:tt)*) => {
        $crate::from_input!(|[$($value)? $(($pat))?]: $ty| -> $($rest)*);
    };

    (|[$($value:tt)*]: $ty:ty| -> $out:ident $block:block) => {
        impl $crate::input::FromInput for $out {
            #[inline]
            fn try_from_input(
                p: &mut $crate::input::Input,
            ) -> core::result::Result<Option<Self>, $crate::input::InputError> {
                let index = p.index();

                let Some(value) = $crate::input::FromInput::try_from_input(p)? else {
                    return Ok(None);
                };

                match (|$($value)*: $ty| -> core::result::Result<$out, $crate::macro_support::Error> {
                    $block
                })(value)
                {
                    Ok(value) => Ok(Some(value)),
                    Err(e) => Err($crate::input::InputError::anyhow(
                        p.path(),
                        p.pos_of(index),
                        e,
                    )),
                }
            }
        }
    };
}