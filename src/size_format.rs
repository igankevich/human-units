use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Write;

use crate::Buffer;
use crate::Size;

/**
Formatted duration that includes unit, integral and fractional parts as fields.

This type is useful when you need custom formatting of the output,
i.e. colors, locale-specific units etc.
*/
pub struct FormattedSize {
    /// Size unit.
    pub unit: &'static str,
    /// Integral part. Max. value is 1023.
    pub integer: u16,
    /// Fractional part. Max. value is 9.
    pub fraction: u8,
}

impl Display for FormattedSize {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let mut buf = Buffer::<MAX_LEN>::new();
        buf.write_u64(self.integer as u64, MAX_POWOF10);
        if self.fraction != 0 {
            buf.write_byte(b'.');
            buf.write_byte(b'0' + self.fraction);
        }
        buf.write_byte(b' ');
        buf.write_str(self.unit)?;
        f.write_str(unsafe { buf.as_str() })
    }
}

const MAX_LEN: usize = 10;
const MAX_POWOF10: u64 = 1000;

/**
This trait adds [`format_size`](FormatSize::format_size) method
to primitive [`u64`](core::u64) and [`usize`](core::u64) types.
*/
pub trait FormatSize {
    /// Splits the original size into integral, fractional and adds a unit.
    fn format_size(self) -> FormattedSize;
}

impl FormatSize for u64 {
    fn format_size(self) -> FormattedSize {
        let mut i = 0;
        let mut scale = 1;
        let mut n = self;
        while n >= 1024 {
            scale *= 1024;
            n >>= 10;
            i += 1;
        }
        let mut b = self & (scale - 1);
        if b != 0 {
            // compute the first digit of the fractional part
            b = (b * 10_u64) >> (i * 10);
        }
        FormattedSize {
            unit: UNITS[i],
            integer: n as u16,
            fraction: b as u8,
        }
    }
}

impl FormatSize for usize {
    fn format_size(self) -> FormattedSize {
        FormatSize::format_size(self as u64)
    }
}

impl FormatSize for Size {
    fn format_size(self) -> FormattedSize {
        FormatSize::format_size(self.0)
    }
}

const UNITS: [&str; 7] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];

#[cfg(all(test, not(feature = "no_std")))]
mod tests {
    #![allow(clippy::panic)]
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use arbtest::arbtest;

    use super::*;
    use crate::FormatSize;

    #[test]
    fn test_format_bytes() {
        assert_eq!("512 B", 512_u64.format_size().to_string());
        assert_eq!("0 B", 0_u64.format_size().to_string());
        assert_eq!("1 B", 1_u64.format_size().to_string());
        assert_eq!("1 KiB", 1024_u64.format_size().to_string());
        assert_eq!("512 KiB", (512_u64 * 1024).format_size().to_string());
        assert_eq!("1023 B", 1023_u64.format_size().to_string());
        assert_eq!("1023 KiB", (1023_u64 * 1024).format_size().to_string());
        assert_eq!("1 MiB", (1024_u64 * 1024).format_size().to_string());
        assert_eq!("1 GiB", (1024_u64 * 1024 * 1024).format_size().to_string());
        assert_eq!(
            "1023 MiB",
            (1024_u64 * 1024 * 1023).format_size().to_string()
        );
        assert_eq!(
            "3.5 GiB",
            (1024_u64 * 1024 * 1024 * 3 + 1024_u64 * 1024 * 1024 / 2)
                .format_size()
                .to_string()
        );
        assert_eq!("3.9 GiB", (u32::MAX as u64).format_size().to_string());
        assert_eq!("15.9 EiB", u64::MAX.format_size().to_string());
    }

    #[test]
    fn test_format_bytes_arbitrary() {
        arbtest(|u| {
            let expected: u64 = u.arbitrary()?;
            let bytes = expected.format_size();
            let x = unit_to_factor(bytes.unit);
            let actual = (bytes.integer as u64) * x + (bytes.fraction as u64) * x / 10;
            assert!(
                expected >= actual && (expected - actual) < x,
                "expected = {}, actual = {}",
                expected,
                actual
            );
            Ok(())
        });
    }

    #[test]
    fn test_shift_division() {
        arbtest(|u| {
            let number: u64 = u.arbitrary()?;
            let expected = number / 1024;
            let actual = number >> 10;
            assert_eq!(expected, actual);
            Ok(())
        });
    }

    #[test]
    fn test_shift_remainder() {
        arbtest(|u| {
            let number: u64 = u.arbitrary()?;
            let expected = number % 1024;
            let actual = number & (1024 - 1);
            assert_eq!(expected, actual);
            Ok(())
        });
    }

    #[test]
    fn test_formatted_size_io() {
        arbtest(|u| {
            let expected: FormattedSize = u.arbitrary()?;
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
            assert_eq!(expected.unit, unit, "expected = `{}`", expected);
            Ok(())
        });
    }

    impl<'a> Arbitrary<'a> for FormattedSize {
        fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, arbitrary::Error> {
            Ok(Self {
                unit: *u.choose(&UNITS[..])?,
                integer: u.int_in_range(0..=MAX_INTEGER)?,
                fraction: u.int_in_range(0..=9)?,
            })
        }
    }

    fn unit_to_factor(unit: &str) -> u64 {
        match unit {
            "B" => 1_u64,
            "KiB" => 1024_u64,
            "MiB" => 1024_u64.pow(2),
            "GiB" => 1024_u64.pow(3),
            "TiB" => 1024_u64.pow(4),
            "PiB" => 1024_u64.pow(5),
            "EiB" => 1024_u64.pow(6),
            _ => panic!("unknown unit `{}`", unit),
        }
    }

    const MAX_INTEGER: u16 = 1023;
}
