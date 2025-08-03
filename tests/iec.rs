#![allow(missing_docs)]

use arbtest::arbtest;
use human_units::iec;
use human_units::iec::iec_unit;
use human_units::imp::IEC_PREFIXES;
use paste::paste;

macro_rules! parameterize {
    ($(($uint: ident
        $max_prefix_str: literal
        $max_prefix_ident: ident
        $max_prefix_symbol: literal
        ))+) => {
        paste! {
            $(
                mod [<_ $uint>] {
                    use super::*;

                    #[iec_unit(symbol = "B", min_prefix = "", max_prefix = $max_prefix_str)]
                    struct Size($uint);

                    #[test]
                    fn constants_are_correct() {
                        assert_eq!("B", Size::SYMBOL);
                        assert_eq!(iec::Prefix::None, Size::MIN_PREFIX);
                        assert_eq!(iec::Prefix::$max_prefix_ident, Size::MAX_PREFIX);
                    }

                    #[test]
                    fn max_string_len_is_correct() {
                        let size = Size($uint::MAX);
                        let string = size.to_string();
                        assert_eq!(Size::MAX_STRING_LEN, string.len(), "string = {string:?}");
                    }

                    #[test]
                    fn try_with_iec_prefix_works() {
                        assert!(Size::try_with_iec_prefix(1, iec::Prefix::None).is_ok());
                        assert!(Size::try_with_iec_prefix(1, iec::Prefix::$max_prefix_ident).is_ok());
                        assert!(Size::try_with_iec_prefix($uint::MAX, iec::Prefix::None).is_ok());
                        assert!(Size::try_with_iec_prefix($uint::MAX, iec::Prefix::$max_prefix_ident).is_err());
                    }

                    #[test]
                    fn with_iec_prefix_works() {
                        // Should not panic.
                        let _ = Size::with_iec_prefix(1, iec::Prefix::None);
                        let _ = Size::with_iec_prefix(1, iec::Prefix::$max_prefix_ident);
                        let _ = Size::with_iec_prefix($uint::MAX, iec::Prefix::None);
                    }

                    #[test]
                    #[should_panic = "attempt to multiply with overflow"]
                    fn with_iec_prefix_panics() {
                        let _ = Size::with_iec_prefix($uint::MAX, iec::Prefix::$max_prefix_ident);
                    }

                    #[test]
                    fn format_iec_works() {
                        arbtest(|u| {
                            let exact = Size(u.arbitrary()?);
                            let formatted = exact.format_iec();
                            let i = IEC_PREFIXES.iter().position(|p| p == &formatted.prefix()).unwrap() - iec::Prefix::None as usize;
                            let factor = (1024 as $uint).pow(i as u32);
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
                            // TODO will not work if MAX_PREFIX == None
                            let number_str = (Size::MIN_PREFIX as u8..=Size::MAX_PREFIX as u8)
                                .rev()
                                .find_map(|p| {
                                    let suffix = format!("{}B", IEC_PREFIXES[p as usize]);
                                    string.ends_with(&suffix)
                                        .then_some(&string[..string.len() - suffix.len()])
                                })
                                .unwrap();
                            assert!(
                                number_str.trim_end().chars().all(char::is_numeric),
                                "number str = {number_str:?}"
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
                            let string = format!("{prefix}{expected}{infix}B{suffix}");
                            let actual: Size = string.parse()
                                .unwrap_or_else(|e| panic!("Failed to parse {string:?}: {e}"));
                            assert_eq!(expected, actual.0);
                            Ok(())
                        });
                    }

                    #[test]
                    fn from_str_overflow_does_not_panic() {
                        let expected = $uint::MAX;
                        let string = format!("{expected} {}B", $max_prefix_symbol);
                        assert!(string.parse::<Size>().is_err(), "string = {string:?}");
                    }

                    #[test]
                    fn string_io_works() {
                        arbtest(|u| {
                            let expected = Size(u.arbitrary()?);
                            let string = expected.to_string();
                            let actual: Size = string.parse()
                                .unwrap_or_else(|e| panic!("Failed to parse {string:?}: {e}"));
                            assert_eq!(expected.0, actual.0);
                            Ok(())
                        });
                    }

                    #[cfg(feature = "serde")]
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

parameterize! {
    (u128 "quebi" Quebi "Qi")
    (u64 "exbi" Exbi "Ei")
    (u32 "gibi" Gibi "Gi")
    (u16 "kibi" Kibi "Ki")
}
