#![allow(missing_docs)]

// TODO dedup wtih `tests/iec.rs`
use arbtest::arbtest;
use human_units::imp::SI_PREFIXES;
use human_units::si;
use human_units::si::si_unit;
use pastey::paste;

macro_rules! parameterize {
    ($(($module: ident
        $uint: ident
        $min_prefix_str: literal
        $max_prefix_str: literal
        $min_prefix_ident: ident
        $max_prefix_ident: ident
        ))+) => {
        paste! {
            $(
                mod $module {
                    use super::*;

                    #[si_unit(symbol = "B", min_prefix = $min_prefix_str, max_prefix = $max_prefix_str)]
                    struct Size($uint);

                    #[test]
                    fn constants_are_correct() {
                        assert_eq!("B", Size::SYMBOL);
                        assert_eq!(si::Prefix::$min_prefix_ident, Size::MIN_PREFIX);
                        assert_eq!(si::Prefix::$max_prefix_ident, Size::MAX_PREFIX);
                    }

                    #[test]
                    fn max_string_len_is_correct() {
                        let size = Size($uint::MAX);
                        let string = size.to_string();
                        assert_eq!(Size::MAX_STRING_LEN, string.len(), "string = {string:?}");
                    }

                    #[test]
                    fn try_with_si_prefix_works() {
                        assert!(Size::try_with_si_prefix(1, si::Prefix::$min_prefix_ident).is_ok());
                        assert!(Size::try_with_si_prefix(1, si::Prefix::$max_prefix_ident).is_ok());
                        assert!(Size::try_with_si_prefix($uint::MAX, si::Prefix::$min_prefix_ident).is_ok());
                        if stringify!($min_prefix_ident) == stringify!($max_prefix_ident) {
                            assert!(Size::try_with_si_prefix($uint::MAX, si::Prefix::$max_prefix_ident).is_ok());
                        } else {
                            assert!(Size::try_with_si_prefix($uint::MAX, si::Prefix::$max_prefix_ident).is_err());
                        }
                    }

                    #[test]
                    fn with_si_prefix_works() {
                        // Should not panic.
                        let _ = Size::with_si_prefix(1, si::Prefix::$min_prefix_ident);
                        let _ = Size::with_si_prefix(1, si::Prefix::$max_prefix_ident);
                        let _ = Size::with_si_prefix($uint::MAX, si::Prefix::$min_prefix_ident);
                    }

                    #[test]
                    #[should_panic = "attempt to multiply with overflow"]
                    fn with_si_prefix_panics() {
                        if stringify!($min_prefix_ident) == stringify!($max_prefix_ident) {
                            // Should not panic.
                            let _ = Size::with_si_prefix($uint::MAX, si::Prefix::$max_prefix_ident);
                            panic!("attempt to multiply with overflow");
                        }
                        let _ = Size::with_si_prefix($uint::MAX, si::Prefix::$max_prefix_ident);
                    }

                    #[test]
                    fn format_si_works() {
                        arbtest(|u| {
                            let exact = Size(u.arbitrary()?);
                            let formatted = exact.format_si();
                            let i = SI_PREFIXES.iter().position(|p| p == &formatted.prefix()).unwrap() -
                                si::Prefix::$min_prefix_ident as usize;
                            let factor = (1000 as $uint).pow(i as u32);
                            let inexact = (formatted.integer() as $uint) * factor +
                                (formatted.fraction() as $uint) * (factor / 10);
                            assert!(
                                exact.0 >= inexact && (exact.0 - inexact) < factor,
                                "Exact = {exact}, inexact = {inexact}",
                            );
                            Ok(())
                        });
                    }

                    #[test]
                    fn to_string_works() {
                        arbtest(|u| {
                            let size = Size(u.arbitrary()?);
                            let string = size.to_string();
                            let s = string.trim();
                            let i = s.chars().take_while(|ch| ch.is_numeric()).count();
                            let _number = &s[..i];
                            let suffix = s[i..].trim();
                            assert!(suffix.ends_with(Size::SYMBOL), "string = {string:?}");
                            let prefix = &suffix[..suffix.len() - Size::SYMBOL.len()];
                            assert!(
                                (Size::MIN_PREFIX as u8..=Size::MAX_PREFIX as u8)
                                    .any(|p| SI_PREFIXES[p as usize] == prefix),
                                "string = {string:?}"
                            );
                            Ok(())
                        });
                    }

                    #[test]
                    fn from_str_works() {
                        arbtest(|u| {
                            let expected: $uint = u.arbitrary()?;
                            let prefix = *u.choose(&["", " ", "  "]).unwrap();
                            let infix = *u.choose(&["", " ", "  "]).unwrap();
                            let suffix = *u.choose(&["", " ", "  "]).unwrap();
                            let si_prefix = $min_prefix_str;
                            let string = format!("{prefix}{expected}{infix}{si_prefix}B{suffix}");
                            let actual: Size = string.parse()
                                .unwrap_or_else(|e| panic!("Failed to parse {string:?}: {e}"));
                            assert_eq!(expected, actual.0);
                            Ok(())
                        });
                    }

                    #[test]
                    fn from_str_overflow_does_not_panic() {
                        let expected = $uint::MAX;
                        let string = format!("{expected} {}B", $max_prefix_str);
                        if stringify!($min_prefix_ident) == stringify!($max_prefix_ident) {
                            assert!(string.parse::<Size>().is_ok(), "string = {string:?}");
                        } else {
                            assert!(string.parse::<Size>().is_err(), "string = {string:?}");
                        }
                    }

                    #[test]
                    fn string_io_works() {
                        arbtest(|u| {
                            let expected = Size(u.arbitrary()?);
                            let string = expected.to_string();
                            let actual: Size = string.parse()
                                .unwrap_or_else(|e| panic!("Failed to parse {string:?}: {e}, initial number = {}", expected.0));
                            assert_eq!(expected.0, actual.0);
                            Ok(())
                        });
                    }

                    mod serde {
                        use super::*;

                        #[test]
                        fn serde_json_works() {
                            arbtest(|u| {
                                let expected = Size(u.arbitrary()?);
                                let string = serde_json::to_string(&expected).unwrap();
                                let actual: Size = serde_json::from_str(&string).unwrap();
                                assert_eq!(expected.0, actual.0);
                                Ok(())
                            });
                        }

                        #[test]
                        fn serde_yaml_works() {
                            arbtest(|u| {
                                let expected = Size(u.arbitrary()?);
                                let string = serde_yaml::to_string(&expected).unwrap();
                                let actual: Size = serde_yaml::from_str(&string).unwrap();
                                assert_eq!(expected.0, actual.0);
                                Ok(())
                            });
                        }

                        #[test]
                        fn serde_toml() {
                            #[derive(::serde::Serialize, ::serde::Deserialize)]
                            struct SizeWrapper {
                                size: Size,
                            }
                            arbtest(|u| {
                                let expected = SizeWrapper { size: Size(u.arbitrary()?) };
                                let string = toml::to_string(&expected).unwrap();
                                let actual: SizeWrapper = toml::from_str(&string).unwrap();
                                assert_eq!(expected.size.0, actual.size.0);
                                Ok(())
                            });
                        }
                    }
                }
            )+
        }
    };
}

include!(concat!(env!("OUT_DIR"), "/si_tests.rs"));
