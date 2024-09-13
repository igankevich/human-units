use core::fmt::Display;
use core::fmt::Formatter;

pub struct FormattedSize {
    pub unit: &'static str,
    pub integer: u16, // max. value 1023
    pub fraction: u8, // max. value 9
}

impl Display for FormattedSize {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(f, "{}", self.integer)?;
        if self.fraction != 0 {
            write!(f, ".{}", self.fraction)?;
        }
        write!(f, " {}", self.unit)
    }
}

pub trait FormatSize {
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

const UNITS: [&str; 7] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];

#[cfg(all(test, not(feature = "no_std")))]
mod tests {
    use arbtest::arbtest;

    use crate::FormatSize;

    #[test]
    fn test_format_bytes() {
        assert_eq!("0 B", 0_u64.format_size().to_string());
        assert_eq!("1 B", 1_u64.format_size().to_string());
        assert_eq!("512 B", 512_u64.format_size().to_string());
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
}
