use std::fmt::Display;
use std::fmt::Formatter;
use std::time::Duration;

pub struct FormattedDuration {
    pub unit: &'static str,
    pub integer: u64,
    pub fraction: u8, // max. value 9
}

impl Display for FormattedDuration {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.integer)?;
        if self.fraction != 0 {
            write!(f, ".{}", self.fraction)?;
        }
        write!(f, " {}", self.unit)
    }
}

pub trait FormatDuration {
    fn format_duration(self) -> FormattedDuration;
}

impl FormatDuration for Duration {
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

#[cfg(test)]
mod tests {
    use arbtest::arbtest;

    use super::*;

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
}
