mod input;

pub use self::input::{FromInput, Input, InputError, LineCol, Nl, Ws};

#[doc(hidden)]
pub mod macro_support {
    pub use anyhow::Error;
}

pub mod prelude {
    //! Helper prelude with useful imports.
    pub use crate::input::{Nl, Ws};
    pub use anyhow::{anyhow, bail, Context, Result};
    pub type ArrayVec<T, const N: usize = 16> = arrayvec::ArrayVec<T, N>;
    pub use bstr::{BStr, ByteSlice};
}

/// Helper macro to build an input processor.
#[macro_export]
macro_rules! from_input {
    (
        |$value:ident: $ty:ty| -> $out:ident $block:block
    ) => {
        impl $crate::FromInput for $out {
            #[inline]
            fn from_input(p: &mut $crate::Input) -> core::result::Result<Self, $crate::InputError> {
                let value = <$ty as $crate::FromInput>::from_input(p)?;

                match (|$value: $ty| -> core::result::Result<$out, $crate::macro_support::Error> {
                    $block
                })(value)
                {
                    Ok(value) => Ok(value),
                    Err(e) => Err($crate::InputError::any(p.path(), p.pos(), e)),
                }
            }
        }
    };
}

/// Input processing.
pub fn input(
    path: &'static str,
    read_path: &str,
    storage: &'static mut String,
) -> anyhow::Result<Input> {
    use anyhow::{anyhow, Context};
    use bstr::BStr;
    use std::fs::File;
    use std::io::Read;

    return inner(path, read_path, storage).with_context(|| anyhow!("{path}"));

    fn inner(
        path: &'static str,
        read_path: &str,
        storage: &'static mut String,
    ) -> anyhow::Result<Input> {
        let mut file = File::open(read_path)?;
        let mut buf = String::with_capacity(4096);
        file.read_to_string(&mut buf)?;
        *storage = buf;
        Ok(Input::new(path, BStr::new(storage)))
    }
}

/// Prepare an input processor.
#[macro_export]
macro_rules! input {
    ($path:literal) => {
        $crate::input!($path, 32768)
    };

    ($path:literal, $buf:literal) => {{
        static mut STORAGE: String = String::new();
        let path = concat!("inputs/", $path);
        let read_path = concat!(env!("CARGO_MANIFEST_DIR"), "/inputs/", $path);
        $crate::input(path, read_path, unsafe { &mut STORAGE })?
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
