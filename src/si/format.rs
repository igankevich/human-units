use crate::Buffer;

/// Format the value as a number using the largest possible SI prefix.
pub trait FormatSi {
    /// Represent the value as a number using the largest possible unit prefix.
    ///
    /// The number has integer part in the range `1..=999` and fractional part in the range `0..=9`.
    fn format_si(&self) -> FormattedUnit<'static>;
}

/// Format the value as a number using the largest possible SI prefix.
pub trait FormatSiUnit {
    /// Represent the value as a number using the largest possible unit prefix.
    ///
    /// The number has integer part in the range `1..=999` and fractional part in the range `0..=9`.
    fn format_si_unit(self, symbol: &str) -> FormattedUnit<'_>;
}

/// An approximate value that consists of integral and fraction parts, prefix and symbol.
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

    /// Integral part. Max. value is 999.
    pub const fn integer(&self) -> u16 {
        self.integer
    }

    /// Integral part. Max. value is 9.
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
