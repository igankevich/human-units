use core::fmt::Formatter;
use core::fmt::Write;

use crate::Buffer;
use crate::Duration;

impl serde::Serialize for Duration {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buf = Buffer::<{ Duration::MAX_STRING_LEN }>::new();
        let _ = write!(&mut buf, "{}", self);
        s.serialize_str(unsafe { core::str::from_utf8_unchecked(buf.as_slice()) })
    }
}

impl<'a> serde::Deserialize<'a> for Duration {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        d.deserialize_str(DurationVisitor)
    }
}

struct DurationVisitor;

impl<'a> serde::de::Visitor<'a> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut Formatter) -> core::fmt::Result {
        write!(formatter, "a string obtained by `Duration::to_string`")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        value.parse().map_err(|_| E::custom("invalid duration"))
    }
}

#[cfg(all(test, not(feature = "no_std")))]
mod tests {

    use core::time::Duration as StdDuration;

    use arbtest::arbtest;

    use super::*;

    #[test]
    fn test_serde_io() {
        assert_eq!(
            "\"1m\"",
            serde_json::to_string(&Duration(StdDuration::from_secs(60))).unwrap()
        );
        assert_eq!(
            Duration(StdDuration::from_secs(60)),
            serde_json::from_str("\"1m\"").unwrap()
        );
        assert_eq!(
            Duration(StdDuration::from_secs(60)),
            serde_yaml::from_str("1m").unwrap()
        );
        assert_eq!(
            Duration(StdDuration::from_secs(10)),
            serde_yaml::from_str("10").unwrap()
        );
    }

    #[test]
    fn test_max_string_len() {
        let string = format!("{}", Duration(StdDuration::new(u64::MAX, 999_999_999_u32)));
        assert_eq!(
            Duration::MAX_STRING_LEN,
            string.len(),
            "string = `{}`",
            string
        );
    }

    #[test]
    fn test_serde_json() {
        arbtest(|u| {
            let expected: Duration = u.arbitrary()?;
            let string = serde_json::to_string(&expected).unwrap();
            let actual = serde_json::from_str(&string).unwrap();
            assert_eq!(expected, actual);
            Ok(())
        });
    }

    #[test]
    fn test_serde_yaml() {
        arbtest(|u| {
            let expected: Duration = u.arbitrary()?;
            let string = serde_yaml::to_string(&expected).unwrap();
            let actual = serde_yaml::from_str(&string).unwrap();
            assert_eq!(expected, actual);
            Ok(())
        });
    }

    #[test]
    fn test_serde_toml() {
        arbtest(|u| {
            let expected: DurationWrapper = u.arbitrary()?;
            let string = toml::to_string(&expected).unwrap();
            let actual = toml::from_str(&string).unwrap();
            assert_eq!(expected, actual);
            Ok(())
        });
    }

    #[derive(
        serde::Serialize, serde::Deserialize, arbitrary::Arbitrary, Debug, PartialEq, Eq, Clone,
    )]
    struct DurationWrapper {
        duration: Duration,
    }
}
