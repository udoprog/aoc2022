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

#[macro_export]
macro_rules! bail {
    ($expr:expr) => {
        Err($crate::input::ErrorKind::from($expr))?
    };
}

#[macro_export]
macro_rules! ensure {
    ($condition:expr) => {
        if !$condition {
            Err($crate::input::ErrorKind::Condition(
                stringify!($condition),
                None,
            ))?
        }
    };

    ($condition:expr, $custom:expr) => {
        if !$condition {
            Err($crate::input::ErrorKind::Condition(
                stringify!($condition),
                Some($crate::input::Custom::from($custom)),
            ))?
        }
    };
}

pub mod prelude {
    //! Helper prelude with useful imports.
    pub use crate::input::{IStr, Nl, NonEmpty, Range, Split, Ws, W};
    pub use anyhow::{anyhow, Context, Result};
    pub type ArrayVec<T, const N: usize = 16> = arrayvec::ArrayVec<T, N>;
    pub type ArrayString<const N: usize = 16> = arrayvec::ArrayString<N>;
    pub use crate::ext::SliceExt;
    pub use crate::{bail, ensure};
    pub use bstr::{BStr, ByteSlice};
    pub use log::*;
    pub use macros::entry;
    pub use num::*;
    pub use num_bigint::{BigInt as I, BigUint as U};
}
