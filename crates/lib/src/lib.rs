pub mod cli;
pub mod input;

#[doc(hidden)]
pub mod macro_support {
    pub use anyhow::Error;
}

pub mod prelude {
    //! Helper prelude with useful imports.
    pub use crate::input::{Input, Nl, Ws};
    pub use anyhow::{anyhow, bail, Context, Result};
    pub type ArrayVec<T, const N: usize = 16> = arrayvec::ArrayVec<T, N>;
    pub use bstr::{BStr, ByteSlice};
    pub use macros::entry;
}

/// Helper macro to build an input processor.
#[macro_export]
macro_rules! from_input {
    (
        |$value:ident: $ty:ty| -> $out:ident $block:block
    ) => {
        impl $crate::input::FromInput for $out {
            #[inline]
            fn from_input(
                p: &mut $crate::input::Input,
            ) -> core::result::Result<Self, $crate::input::InputError> {
                let pos = p.index();
                let value = <$ty as $crate::input::FromInput>::from_input(p)?;

                match (|$value: $ty| -> core::result::Result<$out, $crate::macro_support::Error> {
                    $block
                })(value)
                {
                    Ok(value) => Ok(value),
                    Err(e) => Err($crate::input::InputError::any(p.path(), p.pos_of(pos), e)),
                }
            }
        }
    };
}

/// Input processing.
pub fn input(
    path: &'static str,
    read_path: &str,
    storage: &'static mut Vec<u8>,
) -> anyhow::Result<self::input::Input> {
    use anyhow::{anyhow, Context};
    use std::fs::File;
    use std::io::Read;

    return inner(path, read_path, storage).with_context(|| anyhow!("{path}"));

    fn inner(
        path: &'static str,
        read_path: &str,
        storage: &'static mut Vec<u8>,
    ) -> anyhow::Result<self::input::Input> {
        let mut file = File::open(read_path)?;
        let mut buf = Vec::with_capacity(4096);
        file.read_to_end(&mut buf)?;
        *storage = buf;
        Ok(self::input::Input::new(path, storage))
    }
}

/// Prepare an input processor.
#[macro_export]
macro_rules! input {
    ($path:literal) => {
        $crate::input!($path, 32768)
    };

    ($path:literal, $buf:literal) => {{
        static mut STORAGE: Vec<u8> = Vec::new();
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
