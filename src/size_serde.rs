use std::fmt::Formatter;

use crate::Size;

impl serde::Serialize for Size {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

struct SizeVisitor;

impl<'a> serde::de::Visitor<'a> for SizeVisitor {
    type Value = Size;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "a string obtained by `Size::to_string`")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        value
            .parse()
            .map_err(|_| E::custom(format!("invalid size: `{}`", value)))
    }
}

impl<'a> serde::Deserialize<'a> for Size {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        d.deserialize_str(SizeVisitor)
    }
}

#[cfg(test)]
mod tests {

    use arbtest::arbtest;

    use super::*;

    #[test]
    fn test_serde_io() {
        assert_eq!("\"1k\"", serde_json::to_string(&Size(1024)).unwrap());
        assert_eq!(Size(1024), serde_json::from_str("\"1k\"").unwrap());
        assert_eq!(Size(1024), serde_yaml::from_str("1024").unwrap());
    }

    #[test]
    fn test_serde_io_any_size() {
        arbtest(|u| {
            let expected: Size = u.arbitrary()?;
            let string = format!("\"{}\"", expected);
            let actual = serde_json::from_str(&string).unwrap();
            assert_eq!(expected, actual);
            Ok(())
        });
    }
}
