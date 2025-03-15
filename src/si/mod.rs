//! SI (Système international) units.
//!
//! All units start with _nano_ and end with _giga_ prefix and use [`u64`](core::u64) as the underlying type.
//!
//! # Caveats
//!
//! - [`Time`](crate::si::Time) uses uncommon units such as *kilseconds*.
//!   To use more common _minutes_, _hours_ and _days_ there is [`Duration`](crate::Duration).
//! - [`Mass`](crate::si::Mass) uses *megagrams* instead of *tonnes*.
//! - Some units's symbols as well as _micro_ prefix use Unicode characters.
//!   Turn off `unicode` feature to replace them with alternative ASCII-only representation.

mod core;
mod format;
mod macros;
mod units;

pub(crate) use self::core::*;
pub use self::format::*;
pub(crate) use self::macros::*;
pub use self::units::*;
