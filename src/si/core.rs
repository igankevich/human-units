use core::fmt::Debug;
use core::fmt::Display;
use core::num::NonZeroU64;

use crate::si::unicode;

/// SI unit parsing error.
#[derive(Debug)]
pub struct Error;

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[cfg(not(feature = "no_std"))]
impl std::error::Error for Error {}

pub fn unitify(mut value: u64) -> (u64, usize) {
    if value == 0 {
        return (0, Prefix::None as usize);
    }
    for prefix in MIN_PREFIX as usize..=MAX_PREFIX as usize {
        if value % MULTIPLIER != 0 {
            return (value, prefix);
        }
        value /= MULTIPLIER;
    }
    (value, MAX_PREFIX as usize)
}

pub fn from_str(other: &str, suffix: &'static str) -> Result<u64, Error> {
    let other = other.trim();
    let Some(i) = other.rfind(char::is_numeric) else {
        return Err(Error);
    };
    let value: u64 = other[..=i].parse().map_err(|_| Error)?;
    let unit = other[(i + 1)..].trim_start();
    if !unit.ends_with(suffix) {
        return Err(Error);
    }
    let factor = prefix_to_factor(&unit[..unit.len() - suffix.len()])?;
    Ok(value * factor)
}

macro_rules! define_format_si {
    ($name: ident, $value: ident, $integer: ident, $fraction: ident) => {
        /// Represent the value as a number using the largest possible unit prefix.
        ///
        /// The number has integer part in the range `1..=999` and fractional part in the range `0..=9`.
        ///
        /// Returns the integer part, the fractional part and the index of the unit prefix in
        /// [`PREFIXES`](crate::si::PREFIXES).
        pub fn $name(value: $value) -> ($integer, $fraction, usize) {
            let mut i: usize = 0;
            let mut scale = 1;
            let mut n = value;
            while n >= 1000 {
                scale *= 1000;
                n /= 1000;
                i += 1;
            }
            let mut b = value % scale;
            if b != 0 {
                // Compute the first digit of the fractional part.
                b = (b * 10) / scale;
            }
            let integer = n;
            let fraction = b;
            debug_assert!(integer <= 999, "integer = {integer}");
            debug_assert!(fraction <= 9, "fraction = {fraction}");
            (integer as $integer, fraction as $fraction, i)
        }
    };
}

define_format_si!(format_u128, u128, u16, u8);
define_format_si!(format_u64, u64, u16, u8);
define_format_si!(format_u32, u32, u16, u8);
define_format_si!(format_u16, u16, u16, u8);
define_format_si!(format_usize, usize, u16, u8);

fn prefix_to_factor(prefix: &str) -> Result<u64, Error> {
    match PREFIXES.iter().position(|p| *p == prefix) {
        Some(i) if i >= Prefix::Nano as usize => Ok(1000_u64.pow(i as u32 - Prefix::Nano as u32)),
        _ => Err(Error),
    }
}

const MULTIPLIER: NonZeroU64 = unsafe { NonZeroU64::new_unchecked(1000) };

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
#[allow(dead_code)]
pub enum Prefix {
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

pub const MIN_PREFIX: Prefix = Prefix::Nano;
const MAX_PREFIX: Prefix = Prefix::Giga;

pub const PREFIXES: [&str; 21] = [
    "q",
    "r",
    "y",
    "z",
    "a",
    "f",
    "p",
    "n",
    unicode!("μ", "u"),
    "m",
    "",
    "k",
    "M",
    "G",
    "T",
    "P",
    "E",
    "Z",
    "Y",
    "R",
    "Q",
];

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

    #[test]
    fn test_format_inexact() {
        arbtest(|u| {
            let exact: u64 = u.arbitrary()?;
            let (integer, fraction, i) = format_u64(exact);
            let i = Prefix::Nano as usize + i;
            let x = prefix_index_to_factor(i);
            let inexact = (integer as u64) * x + (fraction as u64) * x / 10;
            assert!(
                exact >= inexact && (exact - inexact) < x,
                "exact = {}, inexact = {}",
                exact,
                inexact
            );
            Ok(())
        });
    }

    fn prefix_index_to_factor(i: usize) -> u64 {
        match PREFIXES[i] {
            "n" => 1,
            "μ" => 1_000,
            "m" => 1_000_000,
            "" => 1_000_000_000,
            "k" => 1_000_000_000_000,
            "M" => 1_000_000_000_000_000,
            "G" => 1_000_000_000_000_000_000,
            prefix => panic!("Unsupported prefix {:?}", prefix),
        }
    }
}
