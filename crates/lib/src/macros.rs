/// Helper macro to build an input processor.
#[macro_export]
macro_rules! from_input {
    (|$value:ident: $ty:ty| -> $($rest:tt)*) => {
        $crate::from_input!(|[$value]: $ty| -> $($rest)*);
    };

    (|$($t:ident)?($pat:pat): $ty:ty| -> $($rest:tt)*) => {
        $crate::from_input!(|[$($t)*($pat)]: $ty| -> $($rest)*);
    };

    (|($($tt:tt)*): $ty:ty| -> $($rest:tt)*) => {
        $crate::from_input!(|[($($tt)*)]: $ty| -> $($rest)*);
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

#[macro_export]
macro_rules! from_input_iter {
    (|$value:ident: $ty:ty| -> $($rest:tt)*) => {
        $crate::from_input_iter!(|[$value]: $ty| -> $($rest)*);
    };

    (|$t:ident($pat:pat): $ty:ty| -> $($rest:tt)*) => {
        $crate::from_input_iter!(|[$($t)*($pat)]: $ty| -> $($rest)*);
    };

    (|($($tt:tt)*): $ty:ty| -> $($rest:tt)*) => {
        $crate::from_input_iter!(|[($($tt)*)]: $ty| -> $($rest)*);
    };

    (|[$($value:tt)*]: $ty:ty| -> $out:ident $block:block) => {
        $crate::from_input!(|[($($value)*)]: $ty| -> $out $block);

        impl $crate::input::FromInputIter for $out {
            #[inline]
            fn from_input_iter<I>(
                p: &mut $crate::input::Input,
                inputs: &mut I,
            ) -> core::result::Result<Option<Self>, $crate::input::InputError>
            where
                I: $crate::input::InputIterator
            {
                let index = p.index();

                let Some(value) = $crate::input::FromInputIter::from_input_iter(p, inputs)? else {
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
