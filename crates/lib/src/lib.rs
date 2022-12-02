mod input;
pub use self::input::{FromInput, Input, InputError, LineCol};

#[doc(hidden)]
pub mod macro_support {
    pub use anyhow::Error;
}

#[macro_export]
macro_rules! from_input {
    (
        |$value:ident: $ty:ty| -> $out:ident($out_ty:ty) $block:block
    ) => {
        struct $out($out_ty);

        impl $crate::FromInput for $out {
            #[inline]
            fn peek(p: &$crate::Input) -> bool {
                <$ty as $crate::FromInput>::peek(p)
            }

            #[inline]
            fn from_input(p: &mut $crate::Input) -> Result<Self, $crate::InputError> {
                let value = <$ty as $crate::FromInput>::from_input(p)?;

                match (|$value: $ty| -> Result<$out, $crate::macro_support::Error> { $block })(
                    value,
                ) {
                    Ok(value) => Ok(value),
                    Err(e) => Err($crate::InputError::any(p.path(), p.pos(), e)),
                }
            }
        }
    };
}

/// Prepare an input processor.
#[macro_export]
macro_rules! input {
    ($path:literal) => {{
        let string = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path));
        Input::new($path, string)
    }};
}
