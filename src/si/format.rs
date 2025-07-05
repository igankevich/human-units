use crate::Buffer;

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
