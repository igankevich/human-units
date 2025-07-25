//! IEC (International Electrotechnical Commission) units.
//!
//! All units start with no prefix, end with _quebi_ prefix, and use [`u64`](::core::u64) as the underlying type.

use crate::u128_is_multiple_of;
use crate::u16_is_multiple_of;
use crate::u32_is_multiple_of;
use crate::u64_is_multiple_of;
use crate::Buffer;
use paste::paste;

#[cfg(feature = "iec-units")]
mod units;

#[cfg(feature = "iec-units")]
pub use self::units::*;

/// Add IEC unit parsing and formatting functions.
///
/// The macro adds the following trait implementations:
/// - [`FormatIec`]
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
/// - `symbol` is the unit name without IEC prefix, e.g. `"Hart"`, `"bit"`.
///
/// # Example
///
#[cfg_attr(
    feature = "derive",
    doc = r##"```rust
use human_units::iec::iec_unit;

#[iec_unit(symbol = "nit")]
struct Nit(pub u64);
```"##
)]
#[cfg(feature = "derive")]
pub use human_units_derive::iec_unit;

/// IEC unit parsing error.
#[derive(Debug)]
pub struct Error;

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// Parse value from a string with IEC unit.
pub trait IecFromStr {
    /// Parse value that has the specified unit symbol from string.
    fn iec_unit_from_str(string: &str, symbol: &str) -> Result<Self, Error>
    where
        Self: Sized;
}

/// Format the value as a number using the largest possible IEC prefix.
pub trait FormatIec {
    /// Represent the value as a number using the largest possible unit prefix.
    ///
    /// The number has integer part in the range `0..=1023` (`0..1024*1024*1024` for `u128`) and fractional part in the range `0..=9`.
    fn format_iec(&self) -> FormattedUnit<'static>;
}

/// Format the value as a number using the largest possible IEC prefix.
pub trait FormatIecUnit {
    /// Represent the value as a number using the largest possible unit prefix.
    ///
    /// The number has integer part in the range `0..=1023` (`0..1024*1024*1024` for `u128`) and fractional part in the range `0..=9`.
    fn format_iec_unit(self, symbol: &str) -> FormattedUnit<'_>;
}

/// An approximate value that consists of integer and fraction parts, prefix and symbol.
pub struct FormattedUnit<'symbol> {
    pub(crate) prefix: &'static str,
    pub(crate) symbol: &'symbol str,
    pub(crate) integer: u16,
    pub(crate) fraction: u8,
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

    /// Integer part. Max. value is 1023.
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

macro_rules! parameterize {
    ($((
        $uint: ident
        $max_prefix: ident
        ($max_prefix_integer: expr)
        ($max_integer: expr)
        ($($ilog: expr)+)
    ))+) => {
        paste! {
            $(
                pub(crate) fn [<unitify_ $uint>](value: $uint) -> ($uint, usize) {
                    if value == 0 {
                        return (0, Prefix::None as usize);
                    }
                    $(
                        {
                            const POW: $uint = (1024 as $uint).pow($ilog);
                            if [<$uint _is_multiple_of>](value, POW) {
                                return (value >> (10 * $ilog), $ilog);
                            }
                        }
                    )+
                    (value, Prefix::$max_prefix as usize)
                }

                impl<const N: usize> Buffer<N> {
                    #[doc(hidden)]
                    pub fn [<write_iec_unit_ $uint>](&mut self, value: $uint, symbol: &str) {
                        let (value, i) = [<unitify_ $uint>](value);
                        self.[<write_ $uint>](value);
                        self.write_byte(b' ');
                        self.write_str_infallible(PREFIXES[i]);
                        self.write_str_infallible(symbol);
                    }
                }

                impl IecFromStr for $uint {
                    fn iec_unit_from_str(string: &str, symbol: &str) -> Result<Self, Error> {
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
                            .position(|prefix| *prefix == prefix_str)
                        else {
                            return Err(Error);
                        };
                        let factor = (1024 as $uint).pow(i as u32);
                        Ok(value * factor)
                    }
                }

                impl FormatIecUnit for $uint {
                    fn format_iec_unit(self, symbol: &str) -> FormattedUnit<'_> {
                        #![allow(clippy::int_plus_one)]
                        $(
                            {
                                const SCALE: $uint = (1024 as $uint).pow($ilog);
                                if self >= SCALE {
                                    let integer = self / SCALE;
                                    let mut fraction = self % SCALE;
                                    if fraction != 0 {
                                        // Compute the first digit of the fractional part,
                                        // i.e. `fraction = fraction * 10 / SCALE` without an
                                        // overflow.
                                        fraction = match fraction.checked_mul(5) {
                                            Some(numerator) => numerator / (SCALE / 2),
                                            None => {
                                                debug_assert_eq!(0, SCALE % 16);
                                                (fraction / 8) * 5 / (SCALE / 16)
                                            }
                                        };
                                    }
                                    debug_assert!(integer <= $max_integer, "integer = {integer}");
                                    debug_assert!(fraction <= 9, "fraction = {fraction}");
                                    return FormattedUnit {
                                        integer: integer as u16,
                                        fraction: fraction as u8,
                                        prefix: PREFIXES[$ilog],
                                        symbol,
                                    };
                                }
                            }
                        )+
                        let integer = self;
                        debug_assert!(integer <= $max_integer, "integer = {integer}");
                        FormattedUnit {
                            integer: integer as u16,
                            fraction: 0,
                            prefix: PREFIXES[0],
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
                    fn [<check_prefix_ $uint>]() {
                        const MAX_POW_OF_1024: $uint = (1024 as $uint).pow($uint::MAX.ilog(1024));
                        assert_eq!(None, MAX_POW_OF_1024.checked_mul(1024));
                        assert_eq!((1, Prefix::Kibi as usize), [<unitify_ $uint>](1024));
                        assert_eq!(
                            ($max_prefix_integer, Prefix::$max_prefix as usize),
                            [<unitify_ $uint>](MAX_POW_OF_1024),
                            "MAX_POW_OF_1024 = {MAX_POW_OF_1024}"
                        );
                    }

                    #[test]
                    fn [<test_format_unit_ $uint>]() {
                        arbtest(|u| {
                            let exact: $uint = u.arbitrary()?;
                            let FormattedUnit { integer, fraction,  prefix, .. } = exact.format_iec_unit("");
                            let i = PREFIXES.iter().position(|p| p == &prefix).unwrap();
                            let factor = (1024 as $uint).pow(i as u32);
                            let inexact = (integer as $uint) * factor + (fraction as $uint) * (factor / 10);
                            assert!(
                                exact >= inexact && (exact - inexact) < factor.saturating_mul($max_integer),
                                "Exact   = {exact},\ninexact = {inexact},\nexact - inexact = {}, factor = {factor},\ninteger = {integer}, fraction = {fraction}, prefix = {prefix:?}",
                                exact - inexact,
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
    (u128 Quebi (1024*1024) (1024*1024*1024 - 1) (10 9 8 7 6 5 4 3 2 1))
    (u64 Exbi (1) (1023) (6 5 4 3 2 1))
    (u32 Gibi (1) (1023) (3 2 1))
    (u16 Kibi (1) (1023) (1))
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(all(test, feature = "std"), derive(arbitrary::Arbitrary))]
#[repr(u8)]
#[allow(dead_code)]
enum Prefix {
    #[default]
    None = 0,
    Kibi = 1,
    Mebi = 2,
    Gibi = 3,
    Tebi = 4,
    Pebi = 5,
    Exbi = 6,
    Zebi = 7,
    Yobi = 8,
    Robi = 9,
    Quebi = 10,
}

const PREFIXES: [&str; 11] = [
    "", "Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi", "Yi", "Ri", "Qi",
];

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    use crate::si::PREFIXES;

    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use arbtest::arbtest;

    #[test]
    fn test_io() {
        arbtest(|u| {
            let expected: FormattedUnit = u.arbitrary()?;
            let string = expected.to_string();
            let mut words = string.splitn(2, ' ');
            let number_str = words.next().unwrap();
            let unit = words.next().unwrap().to_string();
            let mut words = number_str.splitn(2, '.');
            let integer: u16 = words.next().unwrap().parse().unwrap();
            let fraction: u8 = match words.next() {
                Some(word) => word.parse().unwrap(),
                None => 0,
            };
            assert_eq!(expected.integer, integer, "string = {string:?}");
            assert_eq!(expected.fraction, fraction);
            assert_eq!(
                format!("{}{}", expected.prefix, expected.symbol),
                unit,
                "expected = `{}`",
                expected
            );
            Ok(())
        });
    }

    impl<'a> Arbitrary<'a> for FormattedUnit<'static> {
        fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, arbitrary::Error> {
            Ok(Self {
                prefix: *u.choose(&PREFIXES[..])?,
                symbol: "",
                integer: u.int_in_range(0..=999)?,
                fraction: u.int_in_range(0..=9)?,
            })
        }
    }
}
