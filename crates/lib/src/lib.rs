pub mod cli;
pub mod input;
#[macro_use]
mod macros;
mod arena;
mod ext;
mod grid;

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
    pub use crate::input::{
        Digits, IStr, InputIterator, Nl, NonEmpty, Range, Skip, Split, Split2, Ws, B, W,
    };
    pub use anyhow::{self, bail, Context, Error, Result};
    pub type ArrayVec<T, const N: usize = 16> = arrayvec::ArrayVec<T, N>;
    pub type ArrayString<const N: usize = 16> = arrayvec::ArrayString<N>;
    pub use crate::arena::{AllocIter, Arena, ArenaAllocError, ArenaWriteSliceOutOfBounds};
    pub use crate::ensure;
    pub use crate::ext::SliceExt;
    pub use crate::grid::{Grid, GridExt, GridMut, GridSliceMut, GridSliceRef};
    pub use bittle::{set as bits, Bits, BitsMut, BitsOwned, Set};
    pub use bstr::{BStr, ByteSlice};
    pub use fixed_heap::FixedHeap;
    pub use log::*;
    pub use macros::entry;
    pub use num::bigint::{BigInt as I, BigUint as U};
    pub use ringbuffer::ConstGenericRingBuffer as ArrayRingBuffer;
    pub use ringbuffer::{RingBuffer, RingBufferExt, RingBufferRead, RingBufferWrite};
    pub use std::collections::{hash_map, hash_set};
    pub use std::collections::{HashMap, HashSet};
    pub use std::mem;
}

pub use ::num;
