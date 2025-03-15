macro_rules! define_si_unit {
    ($newtype: ident, $uint: ident, $symbol: expr, $doc: literal, $private: ident) => {
        #[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
        #[cfg_attr(all(test, not(feature = "no_std")), derive(arbitrary::Arbitrary))]
        #[repr(transparent)]
        #[doc = $doc]
        pub struct $newtype(pub $uint);

        impl $newtype {
            /// Max. length in string form.
            pub const MAX_STRING_LEN: usize =
                $crate::si::max_string_len!($uint) + 1 + $crate::si::MIN_PREFIX_LEN + $symbol.len();

            /// Unit symbol.
            pub const SYMBOL: &str = $symbol;

            /// Represent the value as a number using the largest possible unit prefix.
            ///
            /// The number has integer part in the range `1..=999` and fractional part in the range `0..=9`.
            pub fn format_inexact(self) -> $crate::si::FormattedUnit {
                let (integer, fraction, i) = $crate::si::format_inexact(self.0);
                $crate::si::FormattedUnit {
                    integer,
                    fraction,
                    prefix: $crate::si::PREFIXES[i],
                    symbol: $symbol,
                }
            }

            fn write(self, buf: &mut $crate::Buffer<{ Self::MAX_STRING_LEN }>) {
                let (value, prefix) = $crate::si::unitify(self.0);
                buf.write_u64(value, $crate::si::max_power_of_10!($uint));
                buf.write_byte(b' ');
                buf.write_str_infallible($crate::si::PREFIXES[prefix]);
                buf.write_str_infallible($symbol);
            }
        }

        impl core::fmt::Display for $newtype {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                let mut buf = $crate::Buffer::<{ Self::MAX_STRING_LEN }>::new();
                self.write(&mut buf);
                f.write_str(unsafe { buf.as_str() })
            }
        }

        impl core::str::FromStr for $newtype {
            type Err = $crate::si::Error;
            fn from_str(other: &str) -> Result<Self, Self::Err> {
                $crate::si::from_str(other, Self::SYMBOL).map($newtype)
            }
        }

        impl From<$uint> for $newtype {
            fn from(other: $uint) -> Self {
                Self(other)
            }
        }

        impl From<$newtype> for $uint {
            fn from(other: $newtype) -> Self {
                other.0
            }
        }

        impl core::ops::Deref for $newtype {
            type Target = $uint;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl core::ops::DerefMut for $newtype {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        #[cfg(feature = "serde")]
        impl serde::Serialize for $newtype {
            fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut buf = $crate::Buffer::<{ $newtype::MAX_STRING_LEN }>::new();
                self.write(&mut buf);
                s.serialize_str(unsafe { buf.as_str() })
            }
        }

        #[cfg(feature = "serde")]
        impl<'a> serde::Deserialize<'a> for $newtype {
            fn deserialize<D>(d: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'a>,
            {
                d.deserialize_str($private::serde_impl::Visitor)
            }
        }

        mod $private {
            #[allow(unused_imports)]
            use super::*;

            #[cfg(feature = "serde")]
            pub mod serde_impl {
                use super::*;

                pub struct Visitor;

                impl<'a> serde::de::Visitor<'a> for Visitor {
                    type Value = $newtype;

                    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        f.write_str(concat!(
                            "A string obtained by `",
                            stringify!($newtype),
                            "::to_string`"
                        ))
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        value
                            .parse()
                            .map_err(|_| E::custom(concat!("Invalid `", stringify!($newtype), "`")))
                    }
                }

                #[cfg(all(test, not(feature = "no_std")))]
                mod tests {
                    use super::*;
                    use arbtest::arbtest;

                    #[test]
                    fn test_serde_json() {
                        arbtest(|u| {
                            let expected: $newtype = u.arbitrary()?;
                            let string = serde_json::to_string(&expected).unwrap();
                            let actual = serde_json::from_str(&string).unwrap();
                            assert_eq!(expected, actual);
                            Ok(())
                        });
                    }

                    #[test]
                    fn test_serde_yaml() {
                        arbtest(|u| {
                            let expected: $newtype = u.arbitrary()?;
                            let string = serde_yaml::to_string(&expected).unwrap();
                            let actual = serde_yaml::from_str(&string).unwrap();
                            assert_eq!(expected, actual);
                            Ok(())
                        });
                    }

                    #[test]
                    fn test_serde_toml() {
                        arbtest(|u| {
                            let expected: Wrapper = u.arbitrary()?;
                            let string = toml::to_string(&expected).unwrap();
                            let actual = toml::from_str(&string).unwrap();
                            assert_eq!(expected, actual);
                            Ok(())
                        });
                    }

                    #[derive(
                        serde::Serialize,
                        serde::Deserialize,
                        arbitrary::Arbitrary,
                        Debug,
                        PartialEq,
                        Eq,
                        Clone,
                    )]
                    struct Wrapper {
                        size: $newtype,
                    }
                }
            }

            #[cfg(all(test, not(feature = "no_std")))]
            mod tests {

                use arbtest::arbtest;

                use super::*;

                #[test]
                fn test_max_string_len() {
                    let string = format!("{}", $newtype($uint::MAX));
                    assert_eq!($newtype::MAX_STRING_LEN, string.len());
                }

                #[test]
                fn display_parse_symmetry() {
                    arbtest(|u| {
                        let expected: $newtype = u.arbitrary()?;
                        let string = expected.to_string();
                        let actual: $newtype = string
                            .parse()
                            .inspect_err(|_| panic!("Expected {expected:?}, string {string:?}"))
                            .unwrap();
                        assert_eq!(expected, actual, "string = `{}`", string);
                        Ok(())
                    });
                }
            }
        }
    };
}

pub(crate) use define_si_unit;

#[rustfmt::skip]
macro_rules! max_string_len {
    (u128) => {39};
    (u64) => {20};
    (u32) => {10};
    (u16) => {5};
    (u8) => {3};
}

pub(crate) use max_string_len;

/// Max. number that is power of 10 and can fit into the supplied type.
#[rustfmt::skip]
macro_rules! max_power_of_10 {
    (u128) => {100000000000000000000000000000000000000_u128};
    (u64) => {10000000000000000000_u64};
    (u32) => {1000000000_u32};
    (u16) => {10000_u16};
    (u8) => {100_u8};
}

pub(crate) use max_power_of_10;

macro_rules! unicode {
    ($utf8: literal, $ascii: literal) => {
        if cfg!(feature = "unicode") {
            $utf8
        } else {
            $ascii
        }
    };
}

pub(crate) use unicode;

#[cfg(test)]
mod tests {

    #[test]
    fn test_max_string_len() {
        assert_eq!(max_string_len!(u128), u128::MAX.to_string().len());
        assert_eq!(max_string_len!(u64), u64::MAX.to_string().len());
        assert_eq!(max_string_len!(u32), u32::MAX.to_string().len());
        assert_eq!(max_string_len!(u16), u16::MAX.to_string().len());
        assert_eq!(max_string_len!(u8), u8::MAX.to_string().len());
    }

    #[test]
    fn test_max_power_of_10() {
        assert_eq!(None, max_power_of_10!(u128).checked_mul(10));
        assert_eq!(None, max_power_of_10!(u64).checked_mul(10));
        assert_eq!(None, max_power_of_10!(u32).checked_mul(10));
        assert_eq!(None, max_power_of_10!(u16).checked_mul(10));
        assert_eq!(None, max_power_of_10!(u8).checked_mul(10));
    }
}
