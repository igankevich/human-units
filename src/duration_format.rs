use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Write;

use crate::Buffer;
use crate::Duration;

/**
Approximate duration that includes unit, integral and fractional parts as fields.

This type is useful when you need custom formatting of the output,
i.e. colors, locale-specific units etc.
*/
pub struct FormattedDuration {
    /// Duration unit.
    pub unit: &'static str,
    /// Integral part. Max. values is 213503982334601.
    pub integer: u64,
    /// Fractional part. Max. value is 9.
    pub fraction: u8,
}

impl Display for FormattedDuration {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let mut buf = Buffer::<MAX_LEN>::new();
        buf.write_u64(self.integer, MAX_POWOF10);
        if self.fraction != 0 {
            buf.write_byte(b'.');
            buf.write_byte(b'0' + self.fraction);
        }
        buf.write_byte(b' ');
        buf.write_str(self.unit)?;
        f.write_str(unsafe { buf.as_str() })
    }
}

const MAX_LEN: usize = 21;
const MAX_POWOF10: u64 = 100000000000000;

/**
This trait adds [`format_duration`](FormatDuration::format_duration) method to
standard [Duration](core::time::Duration) type.
*/
pub trait FormatDuration {
    /// Splits the original duration into integral, fractional and adds a unit.
    fn format_duration(self) -> FormattedDuration;
}

impl FormatDuration for core::time::Duration {
    fn format_duration(self) -> FormattedDuration {
        let seconds = self.as_secs();
        let nanoseconds = self.subsec_nanos();
        if seconds == 0 && nanoseconds == 0 {
            FormattedDuration {
                unit: "s",
                integer: 0,
                fraction: 0,
            }
        } else if seconds == 0 {
            const UNITS: [&str; 4] = ["ns", "μs", "ms", "s"];
            let mut i = 0;
            let mut scale = 1;
            let mut n = nanoseconds;
            while n >= 1000 {
                scale *= 1000;
                n /= 1000;
                i += 1;
            }
            let mut b = nanoseconds % scale;
            if b != 0 {
                // compute the first digit of the fractional part
                b = b * 10_u32 / scale;
            }
            FormattedDuration {
                unit: UNITS[i],
                integer: n as u64,
                fraction: b as u8,
            }
        } else {
            const UNITS: [(u64, &str); 4] = [(1, "s"), (60, "m"), (60, "h"), (24, "d")];
            let mut i = 0;
            let mut scale = UNITS[0].0;
            let mut n = seconds;
            while i + 1 != UNITS.len() && n >= UNITS[i + 1].0 {
                scale *= UNITS[i + 1].0;
                n /= UNITS[i + 1].0;
                i += 1;
            }
            let mut b = seconds % scale;
            if b != 0 {
                // compute the first digit of the fractional part
                b = b * 10_u64 / scale;
            }
            FormattedDuration {
                unit: UNITS[i].1,
                integer: n,
                fraction: b as u8,
            }
        }
    }
}

impl FormatDuration for Duration {
    fn format_duration(self) -> FormattedDuration {
        FormatDuration::format_duration(self.0)
    }
}

#[cfg(all(test, not(feature = "no_std")))]
mod tests {
    #![allow(clippy::panic)]
    use core::time::Duration;

    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use arbtest::arbtest;

    use super::*;
    use crate::FormatDuration;

    #[test]
    fn test_format_duration() {
        assert_eq!("0 s", Duration::from_secs(0).format_duration().to_string());
        assert_eq!(
            "1 ns",
            Duration::from_nanos(1).format_duration().to_string()
        );
        assert_eq!(
            "1 μs",
            Duration::from_nanos(1000).format_duration().to_string()
        );
        assert_eq!(
            "1 ms",
            Duration::from_nanos(1000 * 1000)
                .format_duration()
                .to_string()
        );
        assert_eq!(
            "1.5 ms",
            Duration::from_nanos(1000 * 1000 + 1000 * 1000 / 2)
                .format_duration()
                .to_string()
        );
        assert_eq!(
            "500 μs",
            Duration::from_nanos(1000 * 1000 / 2)
                .format_duration()
                .to_string()
        );
        assert_eq!(
            "999 ms",
            Duration::from_nanos(1000 * 1000 * 999)
                .format_duration()
                .to_string()
        );
        assert_eq!("1 s", Duration::from_secs(1).format_duration().to_string());
        assert_eq!("1 m", Duration::from_secs(60).format_duration().to_string());
        assert_eq!(
            "1 h",
            Duration::from_secs(60 * 60).format_duration().to_string()
        );
        assert_eq!(
            "1 d",
            Duration::from_secs(60 * 60 * 24)
                .format_duration()
                .to_string()
        );
        assert_eq!(
            "12 h",
            Duration::from_secs(60 * 60 * 12)
                .format_duration()
                .to_string()
        );
        assert_eq!(
            "12.5 h",
            Duration::from_secs(60 * 60 * 12 + 60 * 60 / 2)
                .format_duration()
                .to_string()
        );
        assert_eq!(
            "12.5 h",
            Duration::new(60 * 60 * 12 + 60 * 60 / 2, 1000 * 1000 * 1000 - 1)
                .format_duration()
                .to_string()
        );
        assert_eq!(
            MAX_INTEGER,
            Duration::new(u64::MAX, 999_999_999_u32)
                .format_duration()
                .integer
        );
    }

    #[test]
    fn test_format_duration_arbitrary() {
        arbtest(|u| {
            let expected: Duration = u.arbitrary()?;
            let formatted = expected.format_duration();
            let x = unit_to_factor(formatted.unit) as u128;
            let nanoseconds =
                (formatted.integer as u128) * x + (formatted.fraction as u128) * x / 10;
            let actual = Duration::new(
                (nanoseconds / 1_000_000_000_u128) as u64,
                (nanoseconds % 1_000_000_000_u128) as u32,
            );
            let x_duration = Duration::new(
                (x / 1_000_000_000_u128) as u64,
                (x % 1_000_000_000_u128) as u32,
            );
            assert!(
                expected >= actual && (expected - actual) < x_duration,
                "expected = {}\nactual   = {}\nexpected - actual = {}\nx = {}\nformatted = {}",
                expected.as_nanos(),
                actual.as_nanos(),
                (expected - actual).as_nanos(),
                x_duration.as_nanos(),
                formatted,
            );
            Ok(())
        });
    }

    #[test]
    fn test_formatted_duration_io() {
        arbtest(|u| {
            let expected: FormattedDuration = u.arbitrary()?;
            let string = expected.to_string();
            let mut words = string.splitn(2, ' ');
            let number_str = words.next().unwrap();
            let unit = words.next().unwrap().to_string();
            let mut words = number_str.splitn(2, '.');
            let integer: u64 = words.next().unwrap().parse().unwrap();
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

    impl<'a> Arbitrary<'a> for FormattedDuration {
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
            "ns" => 1_u64,
            "μs" => 1000_u64,
            "ms" => 1000_u64.pow(2),
            "s" | "" => 1000_u64.pow(3),
            "m" => 60_u64 * 1000_u64.pow(3),
            "h" => 60_u64 * 60_u64 * 1000_u64.pow(3),
            "d" => 24_u64 * 60_u64 * 60_u64 * 1000_u64.pow(3),
            _ => panic!("unknown unit `{}`", unit),
        }
    }

    const UNITS: [&str; 7] = ["ns", "μs", "ms", "s", "m", "h", "d"];
    const MAX_INTEGER: u64 = 213503982334601;
}
