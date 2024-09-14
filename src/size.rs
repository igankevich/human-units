use core::num::NonZeroU16;
use core::num::NonZeroU64;
use core::ops::Deref;
use core::ops::DerefMut;
use core::str::FromStr;

/**
Exact size in bytes.

The intended use is the configuration files where exact numbers are required,
i.e. cache size, maximum HTTP body size etc.
*/
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(all(test, not(feature = "no_std")), derive(arbitrary::Arbitrary))]
#[repr(transparent)]
pub struct Size(pub u64);

impl Size {
    /// Max. length in string form.
    pub const MAX_STRING_LEN: usize = 20;
}

impl core::fmt::Display for Size {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut size = self.0;
        let unit = if size == 0 {
            ""
        } else {
            let mut unit = "";
            for u in UNITS {
                let d: NonZeroU64 = u.0.into();
                if size % d != 0 {
                    break;
                }
                size /= d;
                unit = u.1;
            }
            unit
        };
        write!(f, "{}{}", size, unit)
    }
}

impl FromStr for Size {
    type Err = SizeError;
    fn from_str(other: &str) -> Result<Self, Self::Err> {
        let other = other.trim();
        match other.rfind(char::is_numeric) {
            None => Err(SizeError),
            Some(i) => {
                let size: u64 = other[..=i].parse().map_err(|_| SizeError)?;
                let unit = other[(i + 1)..].trim();
                let factor = match unit.len() {
                    0 => 1_u64,
                    1 => unit_to_factor(unit.as_bytes()[0])?,
                    _ => return Err(SizeError),
                };
                Ok(Self(size * factor))
            }
        }
    }
}

impl From<u64> for Size {
    fn from(other: u64) -> Self {
        Self(other)
    }
}

impl From<Size> for u64 {
    fn from(other: Size) -> Self {
        other.0
    }
}

impl Deref for Size {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Size {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Size parsing error.
#[derive(Debug)]
pub struct SizeError;

const fn unit_to_factor(unit: u8) -> Result<u64, SizeError> {
    match unit {
        b'k' | b'K' => Ok(1024_u64),
        b'm' | b'M' => Ok(1024_u64 * 1024_u64),
        b'g' | b'G' => Ok(1024_u64 * 1024_u64 * 1024_u64),
        b't' | b'T' => Ok(1024_u64 * 1024_u64 * 1024_u64 * 1024_u64),
        _ => Err(SizeError),
    }
}

const UNITS: [(NonZeroU16, &str); 4] = [
    (unsafe { NonZeroU16::new_unchecked(1024) }, "k"),
    (unsafe { NonZeroU16::new_unchecked(1024) }, "m"),
    (unsafe { NonZeroU16::new_unchecked(1024) }, "g"),
    (unsafe { NonZeroU16::new_unchecked(1024) }, "t"),
];

#[cfg(all(test, not(feature = "no_std")))]
mod tests {

    use arbtest::arbtest;
    use std::ops::AddAssign;

    use super::*;

    #[test]
    fn test_display() {
        assert_eq!("0", Size(0).to_string());
        assert_eq!("1023", Size(1023).to_string());
        assert_eq!("1k", Size(1024).to_string());
        assert_eq!("1025", Size(1025).to_string());
    }

    #[test]
    fn test_parse() {
        assert_eq!("Err(SizeError)", format!("{:?}", "2km".parse::<Size>()));
        assert_eq!("Err(SizeError)", format!("{:?}", "2s".parse::<Size>()));
        assert_eq!("Err(SizeError)", format!("{:?}", "k".parse::<Size>()));
        assert_eq!("Err(SizeError)", format!("{:?}", "".parse::<Size>()));
        assert_eq!(
            "Err(SizeError)",
            format!("{:?}", format!("{}0", u64::MAX).parse::<Size>())
        );
    }

    #[test]
    fn test_deref() {
        assert_eq!(1, *Size(1));
        let mut tmp = Size(1);
        tmp.add_assign(1);
        assert_eq!(2, *tmp);
    }

    #[test]
    fn test_from_into() {
        let d1 = Size(1);
        let d2: u64 = d1.into();
        let d3: Size = d2.into();
        assert_eq!(d1, d3);
        assert_eq!(d1.0, d2);
    }

    #[test]
    fn display_parse_symmetry() {
        arbtest(|u| {
            let expected: Size = u.arbitrary()?;
            let string = expected.to_string();
            let actual: Size = string.parse().unwrap();
            assert_eq!(expected, actual, "string = `{}`", string);
            Ok(())
        });
    }

    #[test]
    fn parse_display_symmetry() {
        arbtest(|u| {
            let (unit, max) = *u
                .choose(&[
                    ("", u64::MAX),
                    ("k", u64::MAX >> 10),
                    ("m", u64::MAX >> 20),
                    ("g", u64::MAX >> 30),
                    ("t", u64::MAX >> 40),
                ])
                .unwrap();
            let mut unit = unit.to_string();
            if u.arbitrary::<bool>()? {
                unit.make_ascii_uppercase();
            }
            let number: u64 = u.int_in_range(0_u64..=max)?;
            let prefix = *u.choose(&["", " ", "  "]).unwrap();
            let infix = *u.choose(&["", " ", "  "]).unwrap();
            let suffix = *u.choose(&["", " ", "  "]).unwrap();
            let expected = format!("{}{}{}{}{}", prefix, number, infix, unit, suffix);
            let expected_size: Size = expected.parse().unwrap();
            let actual = expected_size.to_string();
            let actual_size: Size = actual.parse().unwrap();
            assert_eq!(
                expected_size, actual_size,
                "string 1 = `{}`, string 2 = `{}`",
                expected, actual
            );
            assert!(expected == actual || actual_size.0 % number == 0);
            Ok(())
        });
    }
}
