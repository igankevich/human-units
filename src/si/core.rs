use core::fmt::Debug;
use core::fmt::Display;

use paste::paste;

use crate::si::unicode;
use crate::si::FormatSiUnit;
use crate::si::FormattedUnit;
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

/// Parse value from a string with SI unit.
pub trait SiFromStr {
    /// Parse value that has the specified unit symbol from string.
    fn si_unit_from_str(string: &str, symbol: &str) -> Result<Self, Error>
    where
        Self: Sized;
}

macro_rules! parameterize {
    ($($uint: ident, $max_prefix: ident, ($($ilog: expr,)+),)+) => {
        paste! {
            $(
                pub(crate) fn [<unitify_ $uint>](mut value: $uint) -> ($uint, usize) {
                    if value == 0 {
                        return (0, Prefix::None as usize);
                    }
                    for prefix in MIN_PREFIX..Prefix::$max_prefix as usize {
                        if !value.is_multiple_of(1000) {
                            return (value, prefix);
                        }
                        value /= 1000;
                    }
                    (value, Prefix::$max_prefix as usize)
                }

                impl<const N: usize> Buffer<N> {
                    #[doc(hidden)]
                    pub fn [<write_si_unit_ $uint>](&mut self, value: $uint, symbol: &str) {
                        let (value, i) = [<unitify_ $uint>](value);
                        self.[<write_ $uint>](value);
                        self.write_byte(b' ');
                        self.write_str_infallible(PREFIXES[i]);
                        self.write_str_infallible(symbol);
                    }
                }

                impl SiFromStr for $uint {
                    fn si_unit_from_str(string: &str, symbol: &str) -> Result<Self, Error> {
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
                }

                impl FormatSiUnit for $uint {
                    fn format_si_unit(self, symbol: &str) -> FormattedUnit<'_> {
                        $(
                            {
                                const SCALE: $uint = (1000 as $uint).pow($ilog);
                                if self >= SCALE {
                                    let integer = self / SCALE;
                                    let mut fraction = self % SCALE;
                                    if fraction != 0 {
                                        // Compute the first digit of the fractional part.
                                        fraction /= (SCALE / 10);
                                    }
                                    debug_assert!(integer <= 999, "integer = {integer}");
                                    debug_assert!(fraction <= 9, "fraction = {fraction}");
                                    return FormattedUnit {
                                        integer: integer as u16,
                                        fraction: fraction as u8,
                                        prefix: PREFIXES[MIN_PREFIX + $ilog],
                                        symbol,
                                    };
                                }
                            }
                        )+
                        let integer = self;
                        debug_assert!(integer <= 999, "integer = {integer}");
                        FormattedUnit {
                            integer: integer as u16,
                            fraction: 0,
                            prefix: PREFIXES[MIN_PREFIX],
                            symbol,
                        }
                    }
                }
            )+

            #[cfg(test)]
            mod unitify_tests {
                use super::*;

                use arbtest::arbtest;

                $(
                    #[test]
                    fn [<check_max_prefix_ $uint>]() {
                        const MAX_POW_OF_1000: $uint = (1000 as $uint).pow($uint::MAX.ilog(1000));
                        assert_eq!(None, MAX_POW_OF_1000.checked_mul(1000));
                        assert_eq!((1, Prefix::Micro as usize), [<unitify_ $uint>](1000));
                        assert_eq!((1, Prefix::$max_prefix as usize), [<unitify_ $uint>](MAX_POW_OF_1000));
                    }

                    #[test]
                    fn [<test_format_unit_ $uint>]() {
                        arbtest(|u| {
                            let exact: $uint = u.arbitrary()?;
                            let FormattedUnit { integer, fraction,  prefix, .. } = exact.format_si_unit("");
                            let i = PREFIXES.iter().position(|p| p == &prefix).unwrap() - MIN_PREFIX;
                            let factor = (1000 as $uint).pow(i as u32);
                            let inexact = (integer as $uint) * factor + (fraction as $uint) * (factor / 10);
                            assert!(
                                exact >= inexact && (exact - inexact) < factor,
                                "Exact = {exact}, inexact = {inexact}",
                            );
                            Ok(())
                        });
                    }
                )+
            }
        }
    };
}

parameterize! {
    u128, Ronna, (12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1,),
    u64, Giga, (6, 5, 4, 3, 2, 1,),
    u32, None, (3, 2, 1,),
    u16, Micro, (1,),
}

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

const MIN_PREFIX: usize = Prefix::Nano as usize;

pub(crate) const PREFIXES: [&str; 21] = [
    "q", "r", "y", "z", "a", "f", "p", "n", MICRO, "m", "", "k", "M", "G", "T", "P", "E", "Z", "Y",
    "R", "Q",
];

const MICRO: &str = unicode!("μ", "u");

#[cfg(test)]
mod tests {
    #[test]
    fn test_min_prefix_len() {
        assert_ne!(0, u128::MAX % 1000);
        assert_ne!(0, u64::MAX % 1000);
        assert_ne!(0, u32::MAX % 1000);
        assert_ne!(0, u16::MAX % 1000);
    }
}
