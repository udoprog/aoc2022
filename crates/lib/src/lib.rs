mod input;
pub use self::input::{FromInput, Input, InputError, LineCol, Nl, Ws};

#[doc(hidden)]
pub mod macro_support {
    pub use anyhow::Error;
}

pub mod prelude {
    //! Helper prelude with useful imports.
    pub use crate::input::{Nl, Ws};
    pub use anyhow::{anyhow, bail, Result};
    pub type ArrayVec<T, const N: usize = 16> = arrayvec::ArrayVec<T, N>;
}

#[macro_export]
macro_rules! from_input {
    (
        |$value:ident: $ty:ty| -> $out:ident($out_ty:ty) $block:block
    ) => {
        struct $out($out_ty);

        impl $crate::FromInput for $out {
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
        let path = concat!("inputs/", $path);
        let string = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/inputs/", $path));
        $crate::Input::new(path, string)
    }};
}

#[macro_export]
macro_rules! timeit {
    ($($tt:tt)*) => {{
        let start = std::time::Instant::now();
        let out = { $($tt)* };
        let d = std::time::Instant::now().duration_since(start);
        println!("time: {d:?}");
        out
    }}
}
