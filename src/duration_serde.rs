use std::fmt::Formatter;

use crate::Duration;

impl serde::Serialize for Duration {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

struct DurationVisitor;

impl<'a> serde::de::Visitor<'a> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "a string obtained by `Duration::to_string`")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        value
            .parse()
            .map_err(|_| E::custom(format!("invalid duration: `{}`", value)))
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

#[cfg(test)]
mod tests {

    use std::time::Duration as StdDuration;

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
    fn test_serde_io_any_duration() {
        arbtest(|u| {
            let expected: Duration = u.arbitrary()?;
            let string = format!("\"{}\"", expected);
            let actual = serde_json::from_str(&string).unwrap();
            assert_eq!(expected, actual);
            Ok(())
        });
    }
}
