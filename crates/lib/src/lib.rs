pub mod cli;
pub mod input;
#[macro_use]
mod macros;
mod ext;

#[doc(hidden)]
pub mod macro_support {
    pub use anyhow::Error;
}

pub mod prelude {
    //! Helper prelude with useful imports.
    pub use crate::input::{IStr, Nl, NonEmpty, Range, Split, Ws, W};
    pub use anyhow::{anyhow, bail, ensure, Context, Result};
    pub type ArrayVec<T, const N: usize = 16> = arrayvec::ArrayVec<T, N>;
    pub type ArrayString<const N: usize = 16> = arrayvec::ArrayString<N>;
    pub use crate::ext::SliceExt;
    pub use bstr::{BStr, ByteSlice};
    pub use log::*;
    pub use macros::entry;
    pub use num::*;
    pub use num_bigint::{BigInt as I, BigUint as U};
}

/// Input processing.
pub fn input(
    path: &'static str,
    read_path: &str,
    storage: &'static mut Vec<u8>,
) -> Result<self::input::IStr, crate::cli::CliError> {
    use std::fs::File;
    use std::io::Read;

    return inner(read_path, storage).map_err(|error| {
        crate::cli::CliError::new(
            path,
            Default::default(),
            crate::input::ErrorKind::Boxed(error),
        )
    });

    fn inner(read_path: &str, storage: &'static mut Vec<u8>) -> anyhow::Result<self::input::IStr> {
        let mut file = File::open(read_path)?;
        let mut buf = Vec::with_capacity(4096);
        file.read_to_end(&mut buf)?;
        *storage = buf;
        Ok(self::input::IStr::new(storage, 0))
    }
}

/// Prepare an input processor.
///
/// This declares static storage for the processed input because it's much
/// easier to deal with than lifetimes and memory for it will be freed once the
/// process exists *anyway*.
#[macro_export]
macro_rules! input {
    ($path:literal) => {
        $crate::input!($path, 8192)
    };

    ($path:literal, $buf:literal) => {{
        static mut STORAGE: Vec<u8> = Vec::new();
        let path = concat!("inputs/", $path);
        let read_path = concat!(env!("CARGO_MANIFEST_DIR"), "/inputs/", $path);
        (
            $crate::input(path, read_path, unsafe { &mut STORAGE })?,
            path,
        )
    }};
}
