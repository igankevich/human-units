use core::fmt::Debug;
use core::fmt::Display;
use core::num::NonZeroU128;
use core::num::NonZeroU16;
use core::num::NonZeroU32;
use core::num::NonZeroU64;

use crate::si::unicode;
use crate::Buffer;

/// SI unit parsing error.
#[derive(Debug)]
pub struct Error;

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

macro_rules! define_unitify {
    ($uint: ident, $func: ident, $multiplier: ident) => {
        pub(crate) fn $func(mut value: $uint) -> ($uint, usize) {
            if value == 0 {
                return (0, Prefix::None as usize);
            }
            for prefix in MIN_PREFIX..=MAX_PREFIX {
                if value % $multiplier != 0 {
                    return (value, prefix);
                }
                value /= $multiplier;
            }
            (value, MAX_PREFIX)
        }
    };
}

define_unitify!(u128, unitify_u128, MULTIPLIER_U128);
define_unitify!(u64, unitify_u64, MULTIPLIER_U64);
define_unitify!(u32, unitify_u32, MULTIPLIER_U32);
define_unitify!(u16, unitify_u16, MULTIPLIER_U16);

macro_rules! define_write_unit {
    ($uint: ident, $func: ident, $write: ident, $unitify: ident) => {
        impl<const N: usize> Buffer<N> {
            #[doc(hidden)]
            pub fn $func(&mut self, value: $uint, max_power_of_10: $uint, symbol: &str) {
                let (value, i) = $unitify(value);
                self.$write(value, max_power_of_10);
                self.write_byte(b' ');
                self.write_str_infallible(PREFIXES[i]);
                self.write_str_infallible(symbol);
            }
        }
    };
}

define_write_unit!(u128, write_unit_u128, write_u128, unitify_u128);
define_write_unit!(u64, write_unit_u64, write_u64, unitify_u64);
define_write_unit!(u32, write_unit_u32, write_u32, unitify_u32);
define_write_unit!(u16, write_unit_u16, write_u16, unitify_u16);

/// Parse value from a string with SI unit.
pub trait SiFromStr {
    /// Parse value thas has the specified unit symbol from string.
    fn si_unit_from_str(string: &str, symbol: &str) -> Result<Self, Error>
    where
        Self: Sized;
}

macro_rules! define_from_str {
    ($name: ident, $uint: ident) => {
        fn $name(string: &str, symbol: &str) -> Result<$uint, Error> {
            let string = string.trim();
            let Some(i) = string.rfind(char::is_numeric) else {
                return Err(Error);
            };
            let value: $uint = string[..=i].parse().map_err(|_| Error)?;
            let unit = string[(i + 1)..].trim_start();
            if !unit.ends_with(symbol) {
                return Err(Error);
            }
            let prefix_str = &unit[..unit.len() - symbol.len()];
            let Some(i) = PREFIXES
                .iter()
                .skip(MIN_PREFIX)
                .position(|prefix| *prefix == prefix_str)
            else {
                return Err(Error);
            };
            let factor = (1000 as $uint).pow(i as u32);
            Ok(value * factor)
        }

        impl SiFromStr for $uint {
            fn si_unit_from_str(string: &str, symbol: &str) -> Result<Self, Error> {
                $name(string, symbol)
            }
        }
    };
}

define_from_str!(unit_u128_from_str, u128);
define_from_str!(unit_u64_from_str, u64);
define_from_str!(unit_u32_from_str, u32);
define_from_str!(unit_u16_from_str, u16);

macro_rules! define_format_si {
    ($name: ident, $uint: ident, $integer: ident, $fraction: ident) => {
        /// Represent the value as a number using the largest possible unit prefix by dividing it by 1000.
        ///
        /// The number has integer part in the range `1..=999` and fractional part in the range `0..=9`.
        ///
        /// Returns the integer part, the fractional part, and the number of times the value was divided by 1000.
        pub(crate) fn $name(value: $uint) -> ($integer, $fraction, usize) {
            let mut i: usize = 0;
            let mut scale: $uint = 1;
            let mut n = value;
            while n >= 1000 {
                scale *= 1000;
                n /= 1000;
                i += 1;
            }
            let mut b = value % scale;
            if b != 0 {
                // Compute the first digit of the fractional part.
                b /= (scale / 10);
            }
            let integer = n;
            let fraction = b;
            debug_assert!(integer <= 999, "integer = {integer}");
            debug_assert!(fraction <= 9, "fraction = {fraction}");
            (integer as $integer, fraction as $fraction, i)
        }
    };
}

define_format_si!(format_unit_u128, u128, u16, u8);
define_format_si!(format_unit_u64, u64, u16, u8);
define_format_si!(format_unit_u32, u32, u16, u8);
define_format_si!(format_unit_u16, u16, u16, u8);

const MULTIPLIER_U128: NonZeroU128 = unsafe { NonZeroU128::new_unchecked(1000) };
const MULTIPLIER_U64: NonZeroU64 = unsafe { NonZeroU64::new_unchecked(1000) };
const MULTIPLIER_U32: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(1000) };
const MULTIPLIER_U16: NonZeroU16 = unsafe { NonZeroU16::new_unchecked(1000) };

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(all(test, feature = "std"), derive(arbitrary::Arbitrary))]
#[repr(u8)]
#[allow(dead_code)]
pub(crate) enum Prefix {
    Quecto = 0,
    Ronto = 1,
    Yocto = 2,
    Zepto = 3,
    Atto = 4,
    Femto = 5,
    Pico = 6,
    Nano = 7,
    Micro = 8,
    Milli = 9,
    #[default]
    None = 10,
    Kilo = 11,
    Mega = 12,
    Giga = 13,
    Tera = 14,
    Peta = 15,
    Exa = 16,
    Zetta = 17,
    Yotta = 18,
    Ronna = 19,
    Quetta = 20,
}

pub(crate) const MIN_PREFIX: usize = Prefix::Nano as usize;
const MAX_PREFIX: usize = Prefix::Giga as usize;

pub(crate) const PREFIXES: [&str; 21] = [
    "q", "r", "y", "z", "a", "f", "p", "n", MICRO, "m", "", "k", "M", "G", "T", "P", "E", "Z", "Y",
    "R", "Q",
];

const MICRO: &str = unicode!("μ", "u");

#[cfg(test)]
mod tests {
    use super::*;

    use arbtest::arbtest;

    #[test]
    fn test_min_prefix_len() {
        assert_ne!(0, u128::MAX % 1000);
        assert_ne!(0, u64::MAX % 1000);
        assert_ne!(0, u32::MAX % 1000);
        assert_ne!(0, u16::MAX % 1000);
    }

    macro_rules! test_format {
        ($func1: ident, $func2: ident, $uint: ident) => {
            #[test]
            fn $func1() {
                arbtest(|u| {
                    let exact: $uint = u.arbitrary()?;
                    let (integer, fraction, i) = $func2(exact);
                    let factor = (1000 as $uint).pow(i as u32);
                    let inexact = (integer as $uint) * factor + (fraction as $uint) * (factor / 10);
                    assert!(
                        exact >= inexact && (exact - inexact) < factor,
                        "Exact = {exact}, inexact = {inexact}",
                    );
                    Ok(())
                });
            }
        };
    }

    test_format!(test_format_unit_u128, format_unit_u128, u128);
    test_format!(test_format_unit_u64, format_unit_u64, u64);
    test_format!(test_format_unit_u32, format_unit_u32, u32);
    test_format!(test_format_unit_u16, format_unit_u16, u16);
}
