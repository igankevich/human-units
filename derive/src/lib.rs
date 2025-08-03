#![allow(non_snake_case)]

use core::ops::RangeInclusive;
use core::str::FromStr;
use proc_macro::Span;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::Data;
use syn::DeriveInput;
use syn::Expr;
use syn::Fields;
use syn::Ident;
use syn::Lit;
use syn::Meta;
use syn::Path;
use syn::PathArguments;
use syn::PathSegment;
use syn::Type;

#[proc_macro_attribute]
pub fn si_unit(args: TokenStream, item: TokenStream) -> TokenStream {
    generic_unit(args, item, "si_unit", "si")
}

#[proc_macro_attribute]
pub fn iec_unit(args: TokenStream, item: TokenStream) -> TokenStream {
    generic_unit(args, item, "iec_unit", "iec")
}

fn generic_unit(
    args: TokenStream,
    item: TokenStream,
    macro_name: &str,
    system: &str,
    // TODO ignore case: either lowercase or uppercase
) -> TokenStream {
    let SYSTEM = system.to_ascii_uppercase();
    let system = Ident::new(system, Span::call_site().into());
    let item = parse_macro_input!(item as DeriveInput);
    let newtype = &item.ident;
    let uint = {
        let Data::Struct(data) = &item.data else {
            panic!("`{macro_name}` can only be applied to structs with a single unnamed field");
        };
        let Fields::Unnamed(fields) = &data.fields else {
            panic!("`{macro_name}` can only be applied to structs with a single unnamed field");
        };
        if fields.unnamed.len() != 1 {
            panic!("`{macro_name}` can only be applied to structs with a single unnamed field");
        }
        let field = fields.unnamed.first().expect("Checked the length above");
        let Type::Path(path) = &field.ty else {
            panic!("`{macro_name}`: the struct field should be a primitive unsigned integer");
        };
        let uint = path
            .path
            .get_ident()
            .expect("Failed to parse the type of the struct field");
        if !is_supported_type(uint) {
            panic!("`{macro_name}`: the struct field should be a primitive unsigned integer, supported types: {UINT_TYPES:?}");
        }
        uint
    };
    let args = parse_macro_input!(args with Punctuated::<Meta, syn::Token![,]>::parse_terminated);
    let mut symbol = None;
    let mut internal = false;
    let mut iec_min_power: Option<usize> = None;
    let mut iec_max_power: Option<usize> = None;
    let mut si_min_power: Option<usize> = None;
    let mut si_max_power: Option<usize> = None;
    for arg in args.iter() {
        match arg {
            Meta::Path(path) => match path.get_ident() {
                Some(path) if path == "internal" => internal = true,
                _ => panic!("Invalid argument {:?}", path.get_ident()),
            },
            Meta::NameValue(nv) => match nv.path.get_ident() {
                Some(name) if name == "symbol" => {
                    let value = parse_string(&nv.value).unwrap_or_else(|_| {
                        panic!("`{macro_name}({name} = \"...\")` should be a string literal")
                    });
                    symbol = Some(value);
                }
                Some(name) if name == "min_prefix" && system == "si" => {
                    si_min_power = Some(parse_si_unit(&nv.value, macro_name, name) as usize)
                }
                Some(name) if name == "max_prefix" && system == "si" => {
                    si_max_power = Some(parse_si_unit(&nv.value, macro_name, name) as usize)
                }
                Some(name) if name == "min_prefix" && system == "iec" => {
                    iec_min_power = Some(parse_iec_unit(&nv.value, macro_name, name) as usize)
                }
                Some(name) if name == "max_prefix" && system == "iec" => {
                    iec_max_power = Some(parse_iec_unit(&nv.value, macro_name, name) as usize)
                }
                _ => panic!("Invalid argument {:?}", nv.path.get_ident()),
            },
            _ => panic!("Invalid argument"),
        }
    }
    let symbol = symbol.unwrap_or_else(|| {
        panic!("`{macro_name}` should at least contain `symbol = \"...\"` attribute")
    });
    let (si_min_power, si_max_power) = match (si_min_power, si_max_power) {
        (None, None) => get_power_of_1000_range(uint.to_string().as_str()).into_inner(),
        (Some(min), None) => (
            min,
            (min + get_power_of_1000_count(uint.to_string().as_str()) - 1)
                .min(SiUnit::Quetta as usize),
        ),
        (None, Some(max)) => (
            (max + 1).saturating_sub(get_power_of_1000_count(uint.to_string().as_str())),
            max,
        ),
        (Some(min), Some(max)) => (min, max),
    };
    let (iec_min_power, iec_max_power) = match (iec_min_power, iec_max_power) {
        (None, None) => get_power_of_1024_range(uint.to_string().as_str()).into_inner(),
        (Some(min), None) => (
            min,
            (min + get_power_of_1024_count(uint.to_string().as_str()) - 1)
                .min(SiUnit::Quetta as usize),
        ),
        (None, Some(max)) => (
            (max + 1).saturating_sub(get_power_of_1024_count(uint.to_string().as_str())),
            max,
        ),
        (Some(min), Some(max)) => (min, max),
    };
    let (min_power, max_power, power_count, prefix_strs) = if system == "si" {
        (
            si_min_power,
            si_max_power,
            get_power_of_1000_count(uint.to_string().as_str()),
            SiUnit::PREFIXES,
        )
    } else {
        (
            iec_min_power,
            iec_max_power,
            get_power_of_1024_count(uint.to_string().as_str()),
            IecUnit::PREFIXES,
        )
    };
    if max_power < min_power {
        panic!("`min_prefix` should be less than or equal to `max_prefix`");
    }
    if max_power - min_power > power_count {
        panic!("`min_prefix..max_prefix` range is too big for `{uint}`");
    }
    let powers_rev = (min_power..=max_power).rev().collect::<Vec<_>>();
    let min_prefix_name = match system {
        ref s if s == "si" => format!("{:?}", SiUnit::try_from(min_power).unwrap()),
        ref s if s == "iec" => format!("{:?}", IecUnit::try_from(min_power).unwrap()),
        _ => unreachable!(),
    };
    let min_prefix_name = Ident::new(&min_prefix_name, Span::call_site().into());
    let max_prefix_name = match system {
        ref s if s == "si" => format!("{:?}", SiUnit::try_from(max_power).unwrap()),
        ref s if s == "iec" => format!("{:?}", IecUnit::try_from(max_power).unwrap()),
        _ => unreachable!(),
    };
    let max_prefix_name = Ident::new(&max_prefix_name, Span::call_site().into());
    let uint_string_len = max_uint_string_len(uint.to_string().as_str());
    let space_len = 1;
    let max_prefix_len = prefix_strs[min_power].len();
    let max_string_len = uint_string_len + space_len + max_prefix_len + symbol.len();
    let serde_visitor = Ident::new(&format!("{newtype}HumanUnitsSerdeVisitor"), newtype.span());
    let crate_name = if internal {
        let mut segments = Punctuated::new();
        segments.push_value(PathSegment {
            ident: Ident::new("crate", Span::call_site().into()),
            arguments: PathArguments::None,
        });
        Path {
            leading_colon: None,
            segments,
        }
    } else {
        let mut segments = Punctuated::new();
        segments.push_value(PathSegment {
            ident: Ident::new("human_units", Span::call_site().into()),
            arguments: PathArguments::None,
        });
        Path {
            leading_colon: Some(Default::default()),
            segments,
        }
    };
    let scale: u16 = if system == "si" { 1000 } else { 1024 };
    let write_unit = Ident::new(
        &format!("write_unit_{uint}_{scale}"),
        Span::call_site().into(),
    );
    let from = Ident::new(&format!("from_{system}"), Span::call_site().into());
    let try_with_prefix = Ident::new(
        &format!("try_with_{system}_prefix"),
        Span::call_site().into(),
    );
    let with_prefix = Ident::new(&format!("with_{system}_prefix"), Span::call_site().into());
    let format = Ident::new(&format!("format_{system}"), Span::call_site().into());
    let unit_from_str = Ident::new(&format!("{uint}_unit_from_str"), Span::call_site().into());
    let prefixes = Ident::new(&format!("{SYSTEM}_PREFIXES"), Span::call_site().into());
    let serde = cfg!(feature = "serde").then_some(quote! {
        impl #crate_name::imp::serde::Serialize for #newtype {
            fn serialize<S>(&self, s: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: #crate_name::imp::serde::Serializer,
            {
                let mut buf = #crate_name::Buffer::<{ #newtype::MAX_STRING_LEN }>::new();
                buf.#write_unit::<#min_power, #max_power>(self.0, #symbol);
                s.serialize_str(unsafe { buf.as_str() })
            }
        }

        impl<'a> #crate_name::imp::serde::Deserialize<'a> for #newtype {
            fn deserialize<D>(d: D) -> Result<Self, D::Error>
            where
                D: #crate_name::imp::serde::Deserializer<'a>,
            {
                d.deserialize_str(#serde_visitor)
            }
        }

        struct #serde_visitor;

        impl<'a> #crate_name::imp::serde::de::Visitor<'a> for #serde_visitor {
            type Value = #newtype;

            fn expecting(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                f.write_str(concat!("A string obtained by `", stringify!(#newtype), "::to_string`"))
            }

            fn visit_str<E>(self, value: &str) -> ::core::result::Result<Self::Value, E>
            where
                E: #crate_name::imp::serde::de::Error,
            {
                value
                    .parse()
                    .map_err(|_| E::custom(concat!("Invalid `", stringify!(#newtype), "`")))
            }
        }
    });
    let format_unit = match system {
        ref s if s == "si" => {
            quote! {
                #(
                    {
                        const SCALE: #uint = (1000 as #uint).pow(#powers_rev as u32 - #min_power as u32);
                        if self.0 >= SCALE {
                            let integer = self.0 / SCALE;
                            let mut fraction = self.0 % SCALE;
                            if fraction != 0 {
                                // Compute the first digit of the fractional part.
                                fraction /= (SCALE / 10);
                            }
                            debug_assert!(fraction <= 9);
                            return #crate_name::imp::FormattedUnit::new(
                                #crate_name::imp::#prefixes[#powers_rev],
                                Self::SYMBOL,
                                integer,
                                fraction as u8,
                            );
                        }
                    }
                )*
                let integer = self.0;
                let fraction = 0;
                #crate_name::imp::FormattedUnit::new(
                    #crate_name::imp::#prefixes[#min_power],
                    Self::SYMBOL,
                    integer,
                    fraction,
                )
            }
        }
        ref s if s == "iec" => {
            quote! {
                #(
                    {
                        const SCALE: #uint = (1024 as #uint).pow(#powers_rev as u32 - #min_power as u32);
                        if self.0 >= SCALE {
                            let integer = self.0 / SCALE;
                            let mut fraction = self.0 % SCALE;
                            if fraction != 0 {
                                // Compute the first digit of the fractional part,
                                // i.e. `fraction = fraction * 10 / SCALE` without an
                                // overflow.
                                fraction = match fraction.checked_mul(5) {
                                    Some(numerator) => numerator / (SCALE / 2),
                                    None => {
                                        debug_assert!(0 == SCALE % 16);
                                        (fraction / 8) * 5 / (SCALE / 16)
                                    }
                                };
                            }
                            debug_assert!(fraction <= 9);
                            return #crate_name::imp::FormattedUnit::new(
                                #crate_name::imp::#prefixes[#powers_rev],
                                Self::SYMBOL,
                                integer,
                                fraction as u8,
                            );
                        }
                    }
                )*
                let integer = self.0;
                #crate_name::imp::FormattedUnit::new(
                    #crate_name::imp::#prefixes[#min_power],
                    Self::SYMBOL,
                    integer,
                    0,
                )
            }
        }
        _ => unreachable!(),
    };
    // TODO Implement via Vec::set_len
    //let to_string_fast = cfg!(feature = "alloc").then_some(quote!{
    //    impl #newtype {
    //        pub fn to_string_fast(&self) -> String {
    //            let mut buf = #crate_name::Buffer::<{ Self::MAX_STRING_LEN }>::new();
    //            buf.#write_unit::<#min_power, #max_power>(self.0, #symbol);
    //            ::alloc::string::String::from(unsafe { buf.as_str() })
    //        }
    //    }
    //});
    quote! {
        #item

        impl #newtype {
            /// Max. length in string form.
            pub const MAX_STRING_LEN: usize = #max_string_len;

            /// Unit symbol.
            pub const SYMBOL: &'static str = #symbol;

            #[doc = concat!("Minimum ", #SYSTEM, " prefix.")]
            pub const MIN_PREFIX: #crate_name::#system::Prefix = #crate_name::#system::Prefix::#min_prefix_name;

            #[doc = concat!("Maximum ", #SYSTEM, " prefix.")]
            pub const MAX_PREFIX: #crate_name::#system::Prefix = #crate_name::#system::Prefix::#max_prefix_name;

            #[doc = concat!("Convert from ", #SYSTEM, " value without prefix. Panics if the prefix is out of range.")]
            pub const fn #from(value: #uint) -> Self {
                let prefix = #crate_name::#system::Prefix::None;
                Self::#with_prefix(value, prefix)
            }

            #[doc = concat!("Convert from ", #SYSTEM, " value with prefix. Panics if the prefix is out of range.")]
            #[inline]
            pub const fn #with_prefix(value: #uint, prefix: #crate_name::#system::Prefix) -> Self {
                let power = prefix as u32;
                assert!(#min_power as u32 <= power && power <= #max_power as u32, "Invalid prefix");
                let factor = (#scale as #uint).pow(power - #min_power as u32);
                Self(value * factor)
            }

            #[doc = concat!("Convert from ", #SYSTEM, " value with prefix. Returns an error if the prefix is out of range.")]
            #[inline]
            pub const fn #try_with_prefix(value: #uint, prefix: #crate_name::#system::Prefix) -> Result<Self, #crate_name::Error> {
                let power = prefix as u32;
                if (#max_power as u32) < power && power < (#min_power as u32) {
                    return Err(#crate_name::Error);
                }
                let factor = (#scale as #uint).pow(power - #min_power as u32);
                let v = match value.checked_mul(factor) {
                    Some(value) => value,
                    None => return Err(#crate_name::Error),
                };
                Ok(Self(v))
            }

            /// Represent the value as a number using the largest possible unit prefix.
            #[allow(clippy::modulo_one)]
            pub const fn #format(&self) -> #crate_name::imp::FormattedUnit<'static, #uint, { Self::MAX_STRING_LEN }> {
                #format_unit
            }
        }

        impl ::core::fmt::Display for #newtype {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let mut buf = #crate_name::Buffer::<{ Self::MAX_STRING_LEN }>::new();
                buf.#write_unit::<#min_power, #max_power>(self.0, #symbol);
                f.write_str(unsafe { buf.as_str() })
            }
        }

        impl ::core::str::FromStr for #newtype {
            type Err = #crate_name::Error;
            fn from_str(other: &str) -> Result<Self, Self::Err> {
                #crate_name::imp::#unit_from_str::<{ #scale as #uint }>(
                    other,
                    Self::SYMBOL,
                    &#crate_name::imp::#prefixes[#min_power..=#max_power],
                ).map(#newtype)
            }
        }

        #serde
    }
    .into()
}

fn parse_string(value: &Expr) -> Result<String, InvalidString> {
    match value {
        Expr::Lit(literal) => match &literal.lit {
            Lit::Str(s) => Ok(s.value()),
            _ => Err(InvalidString),
        },
        Expr::Group(group) => match &*group.expr {
            Expr::Lit(literal) => match &literal.lit {
                Lit::Str(s) => Ok(s.value()),
                _ => Err(InvalidString),
            },
            _ => Err(InvalidString),
        },
        _ => Err(InvalidString),
    }
}

fn parse_si_unit(value: &Expr, macro_name: &str, name: &Ident) -> SiUnit {
    let value = parse_string(value)
        .unwrap_or_else(|_| panic!("`{macro_name}({name} = \"...\")` should be a string literal"));
    value.parse().unwrap_or_else(|_| {
        panic!(
            "Unsupported value of `{name}`: {value:?}. Supported values are {:?} and their long counterparts {:?}",
            SiUnit::PREFIXES,
            SiUnit::LONG_PREFIXES,
        )
    })
}

fn parse_iec_unit(value: &Expr, macro_name: &str, name: &Ident) -> IecUnit {
    let value = parse_string(value)
        .unwrap_or_else(|_| panic!("`{macro_name}({name} = \"...\")` should be a string literal"));
    value.parse().unwrap_or_else(|_| {
        panic!(
            "Unsupported value of `{name}`: {value:?}. Supported values are {:?} and their long counterparts {:?}",
            IecUnit::PREFIXES,
            IecUnit::LONG_PREFIXES,
        )
    })
}

#[derive(Debug)]
struct InvalidString;

macro_rules! define_prefix {
    ($enum: ident
     $error: ident
     $((
         $name: ident
         $value: literal
         $prefix: literal
         $long_prefix: literal
     ))+) => {
        #[derive(Debug, Copy, Clone)]
        enum $enum {
            $(
                $name = $value,
            )+
        }

        impl $enum {
            const PREFIXES: &'static [&'static str] = {
                &[$($prefix,)+]
            };

            const LONG_PREFIXES: &'static [&'static str] = {
                &[$($long_prefix,)+]
            };
        }

        impl FromStr for $enum {
            type Err = $error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($prefix => Ok(Self::$name),)+
                    $(s if $long_prefix.eq_ignore_ascii_case(s) => Ok(Self::$name),)+
                    _ => Err($error),
                }
            }
        }

        impl TryFrom<usize> for $enum {
            type Error = $error;

            fn try_from(other: usize) -> Result<Self, Self::Error> {
                match other {
                    $($value => Ok(Self::$name),)+
                    _ => Err($error),
                }
            }
        }

        #[derive(Debug)]
        struct $error;
    }
}

define_prefix! {
    SiUnit
    InvalidSiUnit
    (Quecto 0 "q" "quecto")
    (Ronto 1 "r" "ronto")
    (Yocto 2 "y" "yocto")
    (Zepto 3 "z" "zepto")
    (Atto 4 "a" "atto")
    (Femto 5 "f" "femto")
    (Pico 6 "p" "pico")
    (Nano 7 "n" "nano")
    (Micro 8 "μ" "micro")
    (Milli 9 "m" "milli")
    (None 10 "" "")
    (Kilo 11 "k" "kilo")
    (Mega 12 "M" "mega")
    (Giga 13 "G" "giga")
    (Tera 14 "T" "tera")
    (Peta 15 "P" "peta")
    (Exa 16 "E" "exa")
    (Zetta 17 "Z" "zetta")
    (Yotta 18 "Y" "yotta")
    (Ronna 19 "R" "ronna")
    (Quetta 20 "Q" "quetta")
}

define_prefix! {
    IecUnit
    InvalidIecUnit
    (None 0 "" "")
    (Kibi 1 "Ki" "kibi")
    (Mebi 2 "Mi" "mebi")
    (Gibi 3 "Gi" "gibi")
    (Tebi 4 "Ti" "tebi")
    (Pebi 5 "Pi" "pebi")
    (Exbi 6 "Ei" "exbi")
    (Zebi 7 "Zi" "zebi")
    (Yobi 8 "Yi" "yobi")
    (Robi 9 "Ri" "robi")
    (Quebi 10 "Qi" "quebi")
}

fn is_supported_type(ty: &Ident) -> bool {
    UINT_TYPES.iter().any(|t| ty == t)
}

fn max_uint_string_len(ty: &str) -> usize {
    match ty {
        "u128" => 39,
        "u64" => 20,
        "u32" => 10,
        "u16" => 5,
        _ => panic!("`max_uint_string_len`: Unsupported type {ty:?}"),
    }
}

fn get_power_of_1000_count(ty: &str) -> usize {
    let range = get_power_of_1000_range(ty);
    range.end() - range.start() + 1
}

fn get_power_of_1000_range(ty: &str) -> RangeInclusive<usize> {
    match ty {
        "u128" => SiUnit::Nano as usize..=SiUnit::Ronna as usize,
        "u64" => SiUnit::Nano as usize..=SiUnit::Giga as usize,
        "u32" => SiUnit::Nano as usize..=SiUnit::None as usize,
        "u16" => SiUnit::Nano as usize..=SiUnit::Micro as usize,
        _ => panic!("`max_power`: Unsupported type {ty:?}"),
    }
}

fn get_power_of_1024_count(ty: &str) -> usize {
    let range = get_power_of_1024_range(ty);
    range.end() - range.start() + 1
}

fn get_power_of_1024_range(ty: &str) -> RangeInclusive<usize> {
    match ty {
        "u128" => IecUnit::None as usize..=IecUnit::Quebi as usize,
        "u64" => IecUnit::None as usize..=IecUnit::Exbi as usize,
        "u32" => IecUnit::None as usize..=IecUnit::Gibi as usize,
        "u16" => IecUnit::None as usize..=IecUnit::Kibi as usize,
        _ => panic!("`max_power`: Unsupported type {ty:?}"),
    }
}

const UINT_TYPES: [&str; 5] = ["u128", "u64", "u32", "u16", "u8"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_string_len() {
        for (ty, max) in [
            ("u128", u128::MAX.to_string().len()),
            ("u64", u64::MAX.to_string().len()),
            ("u32", u32::MAX.to_string().len()),
            ("u16", u16::MAX.to_string().len()),
        ] {
            assert_eq!(max, max_uint_string_len(ty));
        }
    }
}
