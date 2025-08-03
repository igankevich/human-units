use crate::Buffer;
use crate::Error;
use paste::paste;

#[cfg(feature = "serde")]
pub use serde;

macro_rules! unicode {
    ($utf8: literal, $ascii: literal) => {
        if cfg!(feature = "unicode") {
            $utf8
        } else {
            $ascii
        }
    };
}

const MICRO: &str = unicode!("μ", "u");

#[doc(hidden)]
pub const SI_PREFIXES: [&str; 21] = [
    "q", "r", "y", "z", "a", "f", "p", "n", MICRO, "m", "", "k", "M", "G", "T", "P", "E", "Z", "Y",
    "R", "Q",
];

#[doc(hidden)]
pub const IEC_PREFIXES: [&str; 11] = [
    "", "Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi", "Yi", "Ri", "Qi",
];

#[inline]
fn pre_parse_str<'a>(string: &'a str, symbol: &str) -> Result<(&'a str, &'a str), Error> {
    let string = string.trim();
    let i = string.rfind(char::is_numeric).ok_or(Error)?;
    let value = &string[..=i];
    let unit = string[i + 1..].trim_start();
    if !unit.ends_with(symbol) {
        return Err(Error);
    }
    let prefix = &unit[..unit.len() - symbol.len()];
    Ok((value, prefix))
}

/// An approximate value that consists of integer and fraction parts, prefix and symbol.
pub struct FormattedUnit<'symbol, T, const N: usize> {
    prefix: &'static str,
    symbol: &'symbol str,
    integer: T,
    fraction: u8,
}

impl<'symbol, T, const N: usize> FormattedUnit<'symbol, T, N> {
    /// Create new instance.
    #[doc(hidden)]
    pub const fn new(prefix: &'static str, symbol: &'symbol str, integer: T, fraction: u8) -> Self {
        Self {
            prefix,
            symbol,
            integer,
            fraction,
        }
    }

    /// Unit prefix.
    pub const fn prefix(&self) -> &'static str {
        self.prefix
    }

    /// Unit symbol.
    pub const fn symbol(&self) -> &'symbol str {
        self.symbol
    }

    /// Integer part.
    pub const fn integer(&self) -> T
    where
        T: Copy,
    {
        self.integer
    }

    /// Fraction part. Max. value is 9.
    pub const fn fraction(&self) -> u8 {
        self.fraction
    }
}

macro_rules! parameterize {
    ($($uint: ident)+) => {
        paste! {
            $(
                #[inline]
                #[doc(hidden)]
                pub fn [<unitify_ $uint _1000>]<const MIN: usize, const MAX: usize>(
                    mut value: $uint,
                ) -> ($uint, usize) {
                    if value == 0 {
                        return (0, MIN);
                    }
                    for power_of_1000 in MIN..MAX {
                        if value % 1000 != 0 {
                            return (value, power_of_1000);
                        }
                        value /= 1000;
                    }
                    (value, MAX)
                }

                #[inline]
                #[doc(hidden)]
                pub fn [<$uint _unit_from_str>]<const SCALE: $uint>(
                    string: &str,
                    symbol: &str,
                    prefixes: &[&str],
                ) -> Result<$uint, Error> {
                    let (value_str, prefix_str) = pre_parse_str(string, symbol)?;
                    let power = prefixes
                        .iter()
                        .position(|prefix| *prefix == prefix_str)
                        .ok_or(Error)?;
                    let value: $uint = value_str.parse().map_err(|_| Error)?;
                    let factor = SCALE.pow(power as u32);
                    value.checked_mul(factor).ok_or(Error)
                }

                #[inline]
                #[doc(hidden)]
                pub fn [<unitify_ $uint _1024>]<const MIN: usize, const MAX: usize>(
                    value: $uint,
                ) -> ($uint, usize) {
                    if value == 0 {
                        return (0, MIN);
                    }
                    for p in (0..=MAX - MIN).rev() {
                        let scale = (1024 as $uint).pow(p as u32);
                        if value % scale == 0 {
                            return (value >> (10 * p), p + MIN);
                        }
                    }
                    (value, MIN)
                }

                impl<const N: usize> core::fmt::Display for FormattedUnit<'_, $uint, N> {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let mut buf = Buffer::<{ N }>::new();
                        buf.[<write_ $uint>](self.integer);
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
            )+
        }
    };
}

parameterize! { u128 u64 u32 u16 }
