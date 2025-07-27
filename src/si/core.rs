use paste::paste;

use crate::si::unicode;
use crate::u128_is_multiple_of;
use crate::u16_is_multiple_of;
use crate::u32_is_multiple_of;
use crate::u64_is_multiple_of;
use crate::Buffer;

/// SI unit parsing error.
#[derive(Debug)]
pub struct Error;

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
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

/// Display SI unit value.
pub trait SiDisplay {
    /// Maximum allowed length in string form.
    const MAX_STRING_LEN: usize;

    /// Display SI unit value with the specified unit symbol.
    fn si_display(self, symbol: &str) -> Display<'_, Self>
    where
        Self: Sized;
}

/// Implements [`Display`](::core::fmt::Display) for SI unit value.
pub struct Display<'a, T> {
    number: T,
    symbol: &'a str,
}

/// Format the value as a number using the largest possible SI prefix.
pub trait FormatSi {
    /// Represent the value as a number using the largest possible unit prefix.
    ///
    /// The number has integer part in the range `0..=999` and fractional part in the range `0..=9`.
    fn format_si(&self) -> FormattedUnit<'static>;
}

/// Format the value as a number using the largest possible SI prefix.
pub trait FormatSiUnit {
    /// Represent the value as a number using the largest possible unit prefix.
    ///
    /// The number has integer part in the range `0..=999` and fractional part in the range `0..=9`.
    fn format_si_unit(self, symbol: &str) -> FormattedUnit<'_>;
}

/// An approximate value that consists of integer and fraction parts, prefix and symbol.
pub struct FormattedUnit<'symbol> {
    prefix: &'static str,
    symbol: &'symbol str,
    integer: u16,
    fraction: u8,
}

impl<'symbol> FormattedUnit<'symbol> {
    /// Unit prefix.
    pub const fn prefix(&self) -> &'static str {
        self.prefix
    }

    /// Unit symbol.
    pub const fn symbol(&self) -> &'symbol str {
        self.symbol
    }

    /// Integer part. Max. value is 999.
    pub const fn integer(&self) -> u16 {
        self.integer
    }

    /// Fraction part. Max. value is 9.
    pub const fn fraction(&self) -> u8 {
        self.fraction
    }
}

impl core::fmt::Display for FormattedUnit<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut buf = Buffer::<MAX_LEN>::new();
        buf.write_u16(self.integer);
        if self.fraction != 0 {
            buf.write_byte(b'.');
            buf.write_byte(b'0' + self.fraction);
        }
        buf.write_byte(b' ');
        buf.write_str_infallible(self.prefix);
        buf.write_str_infallible(self.symbol);
        f.write_str(unsafe { buf.as_str() })
    }
}

const MAX_LEN: usize = 64;

#[rustfmt::skip]
macro_rules! max_string_len {
    (u128) => {39};
    (u64) => {20};
    (u32) => {10};
    (u16) => {5};
}

macro_rules! parameterize {
    ($(($uint: ident
        $min_prefix: ident
        $max_prefix: ident
        ($($ilog: expr)+)))+) => {
        paste! {
            $(
                fn [<unitify_ $uint>](mut value: $uint) -> ($uint, usize) {
                    if value == 0 {
                        return (0, Prefix::None as usize);
                    }
                    for prefix in Prefix::$min_prefix as usize..Prefix::$max_prefix as usize {
                        if ![<$uint _is_multiple_of>](value, 1000) {
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

                impl SiDisplay for $uint {
                    const MAX_STRING_LEN: usize = 64;

                    fn si_display(self, symbol: &str) -> Display<'_, Self> {
                        Display { number: self, symbol }
                    }
                }

                impl core::fmt::Display for Display<'_, $uint> {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        debug_assert!(
                            self.symbol.len() <= <$uint as SiDisplay>::MAX_STRING_LEN - max_string_len!($uint),
                            "The symbol is too long: {} > {}",
                            self.symbol.len(),
                            <$uint as SiDisplay>::MAX_STRING_LEN - max_string_len!($uint),
                        );
                        let mut buffer: Buffer<{ <$uint as SiDisplay>::MAX_STRING_LEN }> = Buffer::new();
                        buffer.[<write_si_unit_ $uint>](self.number, self.symbol);
                        f.write_str(unsafe { buffer.as_str() })
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
                            .skip(Prefix::$min_prefix as usize)
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
                                        prefix: PREFIXES[Prefix::$min_prefix as usize + $ilog],
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
                            prefix: PREFIXES[Prefix::$min_prefix as usize],
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
                    fn [<test_unitify_ $uint>]() {
                        arbtest(|u| {
                            let number: $uint = u.arbitrary()?;
                            let (x, prefix) = [<unitify_ $uint>](number);
                            let p = prefix as u32 - Prefix::$min_prefix as u32;
                            assert_eq!(number, x * (1000 as $uint).pow(p));
                            Ok(())
                        });
                    }

                    #[test]
                    fn [<test_max_string_len_ $uint>]() {
                        let string = format!("{}", $uint::MAX);
                        assert_eq!(max_string_len!($uint), string.len());
                    }

                    #[test]
                    fn [<test_buffer_io_ $uint>]() {
                        arbtest(|u| {
                            let number: $uint = u.arbitrary()?;
                            let symbol: String = char::from_u32(u.int_in_range(b'a'..=b'z')? as u32).unwrap().to_string();
                            let mut buffer = Buffer::<MAX_LEN>::new();
                            buffer.[<write_si_unit_ $uint>](number, &symbol);
                            let actual = $uint::si_unit_from_str(unsafe { buffer.as_str() }, &symbol).unwrap();
                            assert_eq!(number, actual);
                            Ok(())
                        });
                    }

                    #[test]
                    fn [<test_string_io_ $uint>]() {
                        arbtest(|u| {
                            let number: $uint = u.arbitrary()?;
                            let symbol: String = char::from_u32(u.int_in_range(b'a'..=b'z')? as u32).unwrap().to_string();
                            let string = format!("{}", number.si_display(&symbol));
                            let actual = $uint::si_unit_from_str(&string, &symbol).unwrap();
                            assert_eq!(number, actual);
                            Ok(())
                        });
                    }

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
                            let i = PREFIXES.iter().position(|p| p == &prefix).unwrap() - Prefix::$min_prefix as usize;
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
    (u128 Nano Ronna (12 11 10 9 8 7 6 5 4 3 2 1))
    (u64 Nano Giga (6 5 4 3 2 1))
    (u32 Nano None (3 2 1))
    (u16 Nano Micro (1))
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
