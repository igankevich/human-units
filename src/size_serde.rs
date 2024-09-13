use core::fmt::Formatter;
use core::fmt::Write;

use crate::Buffer;
use crate::Size;

impl serde::Serialize for Size {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buf = Buffer::<{ Size::MAX_STRING_LEN }>::new();
        let _ = write!(&mut buf, "{}", self);
        s.serialize_str(unsafe { core::str::from_utf8_unchecked(buf.as_slice()) })
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

struct SizeVisitor;

impl<'a> serde::de::Visitor<'a> for SizeVisitor {
    type Value = Size;

    fn expecting(&self, formatter: &mut Formatter) -> core::fmt::Result {
        write!(formatter, "a string obtained by `Size::to_string`")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        value.parse().map_err(|_| E::custom("invalid size"))
    }
}

#[cfg(all(test, not(feature = "no_std")))]
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
    fn test_max_string_len() {
        let string = format!("{}", Size(u64::MAX));
        assert_eq!(Size::MAX_STRING_LEN, string.len());
    }

    #[test]
    fn test_serde_json() {
        arbtest(|u| {
            let expected: Size = u.arbitrary()?;
            let string = serde_json::to_string(&expected).unwrap();
            let actual = serde_json::from_str(&string).unwrap();
            assert_eq!(expected, actual);
            Ok(())
        });
    }

    #[test]
    fn test_serde_yaml() {
        arbtest(|u| {
            let expected: Size = u.arbitrary()?;
            let string = serde_yaml::to_string(&expected).unwrap();
            let actual = serde_yaml::from_str(&string).unwrap();
            assert_eq!(expected, actual);
            Ok(())
        });
    }

    #[test]
    fn test_serde_toml() {
        arbtest(|u| {
            let expected: SizeWrapper = u.arbitrary()?;
            let string = toml::to_string(&expected).unwrap();
            let actual = toml::from_str(&string).unwrap();
            assert_eq!(expected, actual);
            Ok(())
        });
    }

    #[derive(
        serde::Serialize, serde::Deserialize, arbitrary::Arbitrary, Debug, PartialEq, Eq, Clone,
    )]
    struct SizeWrapper {
        size: Size,
    }
}
