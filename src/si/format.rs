use crate::Buffer;

/// An approximate value that consists of integral and fraction parts, prefix and symbol.
pub struct FormattedUnit {
    pub(crate) prefix: &'static str,
    pub(crate) symbol: &'static str,
    pub(crate) integer: u16,
    pub(crate) fraction: u8,
}

impl FormattedUnit {
    /// Unit prefix.
    pub fn prefix(&self) -> &'static str {
        self.prefix
    }

    /// Unit symbol.
    pub fn symbol(&self) -> &'static str {
        self.symbol
    }

    /// Integral part. Max. value is 999.
    pub fn integer(&self) -> u16 {
        self.integer
    }

    /// Integral part. Max. value is 9.
    pub fn fraction(&self) -> u8 {
        self.fraction
    }
}

impl core::fmt::Display for FormattedUnit {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut buf = Buffer::<MAX_LEN>::new();
        buf.write_u64(self.integer as u64, MAX_POWOF10);
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

const MAX_LEN: usize = 20;
const MAX_POWOF10: u64 = 100;

#[cfg(all(test, not(feature = "no_std")))]
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
            assert_eq!(expected.integer, integer);
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

    impl<'a> Arbitrary<'a> for FormattedUnit {
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
