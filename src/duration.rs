use core::num::NonZeroU128;
use core::num::NonZeroU16;
use core::ops::Deref;
use core::ops::DerefMut;
use core::str::FromStr;
use core::time::Duration as StdDuration;

/**
Exact duration.

The intended use is the configuration files where exact values are required,
i.e. timeouts, cache max age, time-to-live etc.
*/
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(all(test, not(feature = "no_std")), derive(arbitrary::Arbitrary))]
#[repr(transparent)]
pub struct Duration(pub StdDuration);

impl Duration {
    /// Max. length of the duration in string form.
    pub const MAX_STRING_LEN: usize = 31;
}

impl core::fmt::Display for Duration {
    #[allow(clippy::assign_op_pattern)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut duration = self.0.as_nanos();
        let unit = if duration == 0 {
            "s"
        } else {
            let mut unit = "ns";
            for u in UNITS {
                let d: NonZeroU128 = u.0.into();
                if duration % d != 0 {
                    break;
                }
                duration = duration / d;
                unit = u.1;
            }
            unit
        };
        write!(f, "{}{}", duration, unit)
    }
}

impl FromStr for Duration {
    type Err = DurationError;
    fn from_str(other: &str) -> Result<Self, Self::Err> {
        let other = other.trim();
        match other.rfind(char::is_numeric) {
            None => Err(DurationError),
            Some(i) => {
                let duration: u128 = other[..=i].parse().map_err(|_| DurationError)?;
                let unit = other[(i + 1)..].trim();
                let factor = unit_to_factor(unit)? as u128;
                let duration = duration * factor;
                Ok(Self(StdDuration::new(
                    (duration / NANOS_PER_SEC as u128) as u64,
                    (duration % NANOS_PER_SEC as u128) as u32,
                )))
            }
        }
    }
}

impl From<StdDuration> for Duration {
    fn from(other: StdDuration) -> Self {
        Self(other)
    }
}

impl From<Duration> for StdDuration {
    fn from(other: Duration) -> Self {
        other.0
    }
}

impl Deref for Duration {
    type Target = StdDuration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Duration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn unit_to_factor(unit: &str) -> Result<u64, DurationError> {
    match unit {
        "ns" => Ok(1_u64),
        "μs" => Ok(1000_u64),
        "ms" => Ok(1000_u64 * 1000_u64),
        "s" | "" => Ok(1000_u64 * 1000_u64 * 1000_u64),
        "m" => Ok(60_u64 * 1000_u64 * 1000_u64 * 1000_u64),
        "h" => Ok(60_u64 * 60_u64 * 1000_u64 * 1000_u64 * 1000_u64),
        "d" => Ok(24_u64 * 60_u64 * 60_u64 * 1000_u64 * 1000_u64 * 1000_u64),
        _ => Err(DurationError),
    }
}

/// Duration parsing error.
#[derive(Debug)]
pub struct DurationError;

const UNITS: [(NonZeroU16, &str); 6] = [
    (unsafe { NonZeroU16::new_unchecked(1000) }, "μs"),
    (unsafe { NonZeroU16::new_unchecked(1000) }, "ms"),
    (unsafe { NonZeroU16::new_unchecked(1000) }, "s"),
    (unsafe { NonZeroU16::new_unchecked(60) }, "m"),
    (unsafe { NonZeroU16::new_unchecked(60) }, "h"),
    (unsafe { NonZeroU16::new_unchecked(24) }, "d"),
];

const NANOS_PER_SEC: u32 = 1_000_000_000_u32;

#[cfg(all(test, not(feature = "no_std")))]
mod tests {

    use std::ops::AddAssign;

    use arbtest::arbtest;

    use super::*;

    #[test]
    fn test_duration_display() {
        assert_eq!("123s", Duration(StdDuration::from_secs(123)).to_string());
        assert_eq!("2m", Duration(StdDuration::from_secs(120)).to_string());
        assert_eq!(
            "1d",
            Duration(StdDuration::from_secs(24 * 60 * 60)).to_string()
        );
        assert_eq!(
            "23h",
            Duration(StdDuration::from_secs(23 * 60 * 60)).to_string()
        );
        assert_eq!("0s", Duration(StdDuration::from_secs(0)).to_string());
        assert_eq!("1μs", Duration(StdDuration::from_nanos(1000)).to_string());
    }

    #[test]
    fn test_duration_parse() {
        assert_eq!(Duration(StdDuration::from_secs(1)), "1".parse().unwrap());
        assert_eq!(
            Duration(StdDuration::from_nanos(1000)),
            "1μs".parse().unwrap()
        );
        assert_eq!(Duration(StdDuration::from_secs(120)), "2m".parse().unwrap());
        assert_eq!(
            "Err(DurationError)",
            format!("{:?}", "2km".parse::<Duration>())
        );
        assert_eq!(
            "Err(DurationError)",
            format!("{:?}", "ms".parse::<Duration>())
        );
        assert_eq!(
            "Err(DurationError)",
            format!("{:?}", format!("{}0", u128::MAX).parse::<Duration>())
        );
    }

    #[test]
    fn test_deref() {
        assert_eq!(
            StdDuration::from_secs(1),
            *Duration(StdDuration::from_secs(1)),
        );
        let mut tmp = Duration(StdDuration::from_secs(1));
        tmp.add_assign(StdDuration::from_secs(1));
        assert_eq!(StdDuration::from_secs(2), *tmp);
    }

    #[test]
    fn test_from_into() {
        let d1 = Duration(StdDuration::from_secs(1));
        let d2: StdDuration = d1.into();
        let d3: Duration = d2.into();
        assert_eq!(d1, d3);
        assert_eq!(d1.0, d2);
    }

    #[test]
    fn display_parse_symmetry() {
        arbtest(|u| {
            let expected: Duration = u.arbitrary()?;
            let string = expected.to_string();
            let actual: Duration = string.parse().unwrap();
            assert_eq!(expected, actual, "string = `{}`", string);
            Ok(())
        });
    }

    #[test]
    fn parse_display_symmetry() {
        arbtest(|u| {
            let (unit, max) = *u
                .choose(&[
                    ("ns", MAX_NANOSECONDS),
                    ("μs", MAX_NANOSECONDS / 1000_u128),
                    ("ms", MAX_NANOSECONDS / (1000_u128 * 1000_u128)),
                    ("s", MAX_NANOSECONDS / (1000_u128 * 1000_u128 * 1000_u128)),
                    (
                        "m",
                        MAX_NANOSECONDS / (1000_u128 * 1000_u128 * 1000_u128 * 60_u128),
                    ),
                    (
                        "h",
                        MAX_NANOSECONDS / (1000_u128 * 1000_u128 * 1000_u128 * 60_u128 * 60_u128),
                    ),
                    (
                        "d",
                        MAX_NANOSECONDS
                            / (1000_u128 * 1000_u128 * 1000_u128 * 60_u128 * 60_u128 * 24_u128),
                    ),
                ])
                .unwrap();
            let number: u128 = u.int_in_range(0_u128..=max)?;
            let prefix = *u.choose(&["", " ", "  "]).unwrap();
            let infix = *u.choose(&["", " ", "  "]).unwrap();
            let suffix = *u.choose(&["", " ", "  "]).unwrap();
            let expected = format!("{}{}{}{}{}", prefix, number, infix, unit, suffix);
            let expected_duration: Duration = expected.parse().unwrap();
            let actual = expected_duration.to_string();
            let actual_duration: Duration = actual.parse().unwrap();
            assert_eq!(
                expected_duration, actual_duration,
                "string 1 = `{}`, string 2 = `{}`",
                expected, actual
            );
            assert!(
                expected == actual || actual_duration.0.as_nanos() % number == 0 || number == 0
            );
            Ok(())
        });
    }

    const MAX_NANOSECONDS: u128 =
        (u64::MAX as u128) * (NANOS_PER_SEC as u128) + (NANOS_PER_SEC as u128) - 1_u128;
}
