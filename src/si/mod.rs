//! SI (Système international) units.
//!
//! All units start with _nano_ and end with _giga_ prefix and use [`u64`](::core::u64) as the underlying type.
//!
//! # Caveats
//!
#![cfg_attr(
    feature = "si-units",
    doc = r##"
 - [`Time`] uses uncommon units such as *kilseconds*.
   To use more common _minutes_, _hours_ and _days_ there is [`Duration`](crate::Duration).
 - [`Mass`] uses *megagrams* instead of *tonnes*."##
)]
//! - Some units's symbols as well as _micro_ prefix use Unicode characters.
//!   Turn off `unicode` feature to replace them with alternative ASCII-only representation.

mod core;
mod format;
mod macros;
#[cfg(feature = "si-units")]
mod units;

pub use self::core::*;
pub use self::format::*;
pub(crate) use self::macros::*;
#[cfg(feature = "si-units")]
pub use self::units::*;

/// Add SI unit parsing and formatting functions.
///
/// The macro adds the following trait implementations:
/// - [`FormatSi`]
/// - [`Display`](::core::fmt::Display)
/// - [`FromStr`](::core::str::FromStr)
#[cfg_attr(
    all(feature = "derive", feature = "serde"),
    doc = " - [`Serialize`](serde::Serialize) and [`Deserialize`](serde::Deserialize) when `serde` feature is enabled."
)]
///
/// The macro also adds the following constants:
/// - `Self::SYMBOL` — the symbol,
/// - `Self::MAX_STRING_LEN` — max. length in string form.
///
/// Macro parameters:
/// - `symbol` is the unit name without SI prefix, e.g. `"Hz"`, `"°C"`.
///
/// # Example
///
#[cfg_attr(
    feature = "derive",
    doc = r##"```rust
use human_units::si::si_unit;

#[si_unit(symbol = "Hz")]
struct CpuFrequency(pub u64);
```"##
)]
#[cfg(feature = "derive")]
pub use human_units_derive::si_unit;
