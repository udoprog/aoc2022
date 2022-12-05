pub mod cli;
pub mod input;
#[macro_use]
mod macros;
mod ext;

#[cfg(prod)]
#[path = "env/prod.rs"]
#[macro_use]
#[doc(hidden)]
pub mod env;

#[cfg(not(prod))]
#[path = "env/no_prod.rs"]
#[macro_use]
#[doc(hidden)]
pub mod env;

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
