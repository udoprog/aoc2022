mod buf;
pub use self::buf::Buf;

mod input;
pub use self::input::{FromInput, Input, InputError, LineCol};

#[doc(hidden)]
pub mod macro_support {
    pub use anyhow::Error;
}

#[macro_export]
macro_rules! map {
    (
        |$value:ident: $ty:ty| -> $out:ident($out_ty:ty) $block:block
    ) => {
        struct $out($out_ty);

        impl $crate::FromInput for $out {
            #[inline]
            fn peek(p: &mut $crate::Input<'_>) -> Result<bool, $crate::InputError> {
                <$ty as $crate::FromInput>::peek(p)
            }

            #[inline]
            fn from_input(p: &mut $crate::Input<'_>) -> Result<Self, $crate::InputError> {
                let value = <$ty as $crate::FromInput>::from_input(p)?;

                match (|$value| -> Result<$out, $crate::macro_support::Error> { $block })(value) {
                    Ok(value) => Ok(value),
                    Err(e) => Err($crate::InputError::any(p.path(), p.pos(), e)),
                }
            }
        }
    };
}
