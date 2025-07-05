#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "no_std")]
compile_error!("Please use `cfg(not(feature = \"std\"))` instead of `cfg(feature = \"no_std\")`.");

mod buffer;
mod duration;
mod duration_format;
#[cfg(feature = "serde")]
mod duration_serde;
pub mod iec;
pub mod si;
mod size;
mod size_format;
#[cfg(feature = "serde")]
mod size_serde;

pub use self::buffer::*;
pub use self::duration::*;
pub use self::duration_format::*;
pub use self::size::*;
pub use self::size_format::*;
