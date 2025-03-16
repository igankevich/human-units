use proc_macro::Span;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::Data;
use syn::DeriveInput;
use syn::Expr;
use syn::ExprLit;
use syn::Fields;
use syn::Ident;
use syn::Lit;
use syn::LitInt;
use syn::Meta;
use syn::Type;

#[proc_macro_attribute]
pub fn si_unit(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args with Punctuated::<Meta, syn::Token![,]>::parse_terminated);
    let symbol = args
        .iter()
        .find_map(|meta| match meta {
            Meta::NameValue(nv) => {
                let path = nv.path.get_ident()?;
                if path != "symbol" {
                    return None;
                }
                match &nv.value {
                    Expr::Lit(literal) => match &literal.lit {
                        Lit::Str(symbol) => Some(symbol.value()),
                        _ => panic!("`si_unit(symbol = \"...\")` should be a string literal"),
                    },
                    Expr::Group(group) => match &*group.expr {
                        Expr::Lit(literal) => match &literal.lit {
                            Lit::Str(symbol) => Some(symbol.value()),
                            _ => panic!("`si_unit(symbol = \"...\")` should be a string literal"),
                        },
                        _ => panic!("`si_unit(symbol = \"...\")` should be a literal"),
                    },
                    _ => panic!("`si_unit(symbol = \"...\")` should be a literal"),
                }
            }
            _ => None,
        })
        .expect("`si_unit` should at least contain `symbol = \"...\"` attribute");
    let mut unit_prefix = args
        .iter()
        .find_map(|meta| match meta {
            Meta::NameValue(nv) => {
                let path = nv.path.get_ident()?;
                if path != "prefix" {
                    return None;
                }
                match &nv.value {
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(prefix),
                        ..
                    }) => Some(prefix.value()),
                    _ => panic!("`si_unit(prefix = \"...\")` should be a string literal"),
                }
            }
            _ => None,
        })
        .map(|prefix| {
            if prefix.is_empty() {
                "none".into()
            } else {
                prefix.to_lowercase()
            }
        })
        .unwrap_or_else(|| "nano".into());
    if !PREFIXES.contains(&unit_prefix.as_str()) {
        panic!("`si_unit`: Invalid unit prefix {unit_prefix:?}, valid prefixes {PREFIXES:?}");
    }
    if let Some(first_letter) = unit_prefix.get_mut(0..1) {
        first_letter.make_ascii_uppercase();
    }
    let unit_prefix = Ident::new(&unit_prefix, Span::call_site().into());
    let internal = args.iter().any(|meta| match meta {
        Meta::Path(path) => {
            let Some(path) = path.get_ident() else {
                return false;
            };
            path == "internal"
        }
        _ => false,
    });
    let item = parse_macro_input!(item as DeriveInput);
    let newtype = &item.ident;
    let Data::Struct(data) = &item.data else {
        panic!("`si_unit` can only be applied to structs with a single unnamed field");
    };
    let Fields::Unnamed(fields) = &data.fields else {
        panic!("`si_unit` can only be applied to structs with a single unnamed field");
    };
    if fields.unnamed.len() != 1 {
        panic!("`si_unit` can only be applied to structs with a single unnamed field");
    }
    let field = fields.unnamed.first().expect("Checked the length above");
    let Type::Path(path) = &field.ty else {
        panic!("`si_unit`: the struct field should be a primitive unsigned integer");
    };
    let uint = path
        .path
        .get_ident()
        .expect("Failed to parse the type of the struct field");
    if !is_supported_type(uint) {
        panic!("`si_unit`: the struct field should be a primitive unsigned integer, supported types: {:?}", UINT_TYPES);
    }
    let uint_string_len = max_string_len(uint.to_string().as_str());
    let min_prefix_len = if unit_prefix == "Micro" { 2 } else { 1 };
    let max_string_len = uint_string_len + 1 + min_prefix_len + symbol.len();
    let serde_module = Ident::new(&format!("{}_human_units_serde", newtype), newtype.span());
    let crate_name = if internal {
        Ident::new("crate", Span::call_site().into())
    } else {
        // TODO colons
        Ident::new("human_units", Span::call_site().into())
    };
    let uint_max_pow10 = LitInt::new(
        max_power_of_10(uint.to_string().as_str()),
        Span::call_site().into(),
    );
    let serde = cfg!(feature = "serde").then_some(quote! {
        impl serde::Serialize for #newtype {
            fn serialize<S>(&self, s: S) -> core::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut buf = #crate_name::Buffer::<{ #newtype::MAX_STRING_LEN }>::new();
                self.write(&mut buf);
                s.serialize_str(unsafe { buf.as_str() })
            }
        }

        impl<'a> serde::Deserialize<'a> for #newtype {
            fn deserialize<D>(d: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'a>,
            {
                d.deserialize_str(#serde_module::Visitor)
            }
        }

        #[allow(non_snake_case)]
        mod #serde_module {
            use super::*;

            pub struct Visitor;

            impl<'a> serde::de::Visitor<'a> for Visitor {
                type Value = #newtype;

                fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    f.write_str(concat!(
                            "A string obtained by `",
                            stringify!(#newtype),
                            "::to_string`"
                    ))
                }

                fn visit_str<E>(self, value: &str) -> core::result::Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    value
                        .parse()
                        .map_err(|_| E::custom(concat!("Invalid `", stringify!(#newtype), "`")))
                }
            }
        }
    });
    quote! {
        #item

        impl #newtype {
            /// Max. length in string form.
            pub const MAX_STRING_LEN: usize = #max_string_len;

            /// Unit symbol.
            pub const SYMBOL: &str = #symbol;

            fn write(&self, buf: &mut #crate_name::Buffer<{ Self::MAX_STRING_LEN }>) {
                let (value, prefix) = #crate_name::si::unitify(self.0);
                buf.write_u64(value, #uint_max_pow10);
                buf.write_byte(b' ');
                buf.write_str_infallible(#crate_name::si::PREFIXES[prefix]);
                buf.write_str_infallible(#symbol);
            }
        }

        impl #crate_name::si::FormatSi for #newtype {
            fn format_si(&self) -> #crate_name::si::FormattedUnit<'static, 'static> {
                let (integer, fraction, i) = #crate_name::si::format_u64(self.0);
                #crate_name::si::FormattedUnit {
                    integer,
                    fraction,
                    prefix: #crate_name::si::PREFIXES[#crate_name::si::Prefix::#unit_prefix as usize + i],
                    symbol: #symbol,
                }
            }
        }

        impl core::fmt::Display for #newtype {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                let mut buf = #crate_name::Buffer::<{ Self::MAX_STRING_LEN }>::new();
                self.write(&mut buf);
                f.write_str(unsafe { buf.as_str() })
            }
        }

        impl core::str::FromStr for #newtype {
            type Err = #crate_name::si::Error;
            fn from_str(other: &str) -> Result<Self, Self::Err> {
                #crate_name::si::from_str(other, Self::SYMBOL).map(#newtype)
            }
        }

        #serde
    }
    .into()
}

fn is_supported_type(ty: &Ident) -> bool {
    for t in UINT_TYPES {
        if ty == t {
            return true;
        }
    }
    false
}

fn max_string_len(ty: &str) -> usize {
    match ty {
        "u128" => 39,
        "u64" => 20,
        "u32" => 10,
        "u16" => 5,
        _ => panic!("`max_string_len`: Unsupported type {ty:?}"),
    }
}

fn max_power_of_10(ty: &str) -> &'static str {
    match ty {
        "u128" => "100000000000000000000000000000000000000",
        "u64" => "10000000000000000000",
        "u32" => "1000000000",
        "u16" => "10000",
        _ => panic!("`max_power_of_10`: Unsupported type {ty:?}"),
    }
}

const UINT_TYPES: [&str; 5] = ["u128", "u64", "u32", "u16", "u8"];
const PREFIXES: [&str; 21] = [
    "quecto", "ronto", "yocto", "zepto", "atto", "femto", "pico", "nano", "micro", "milli", "none",
    "kilo", "mega", "giga", "tera", "peta", "exa", "zetta", "yotta", "ronna", "quetta",
];

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
            assert_eq!(max, max_string_len(ty));
        }
    }

    #[test]
    fn test_max_power_of_10() {
        assert_eq!(
            None,
            max_power_of_10("u128")
                .parse::<u128>()
                .unwrap()
                .checked_mul(10)
        );
        assert_eq!(
            None,
            max_power_of_10("u64")
                .parse::<u64>()
                .unwrap()
                .checked_mul(10)
        );
        assert_eq!(
            None,
            max_power_of_10("u32")
                .parse::<u32>()
                .unwrap()
                .checked_mul(10)
        );
        assert_eq!(
            None,
            max_power_of_10("u16")
                .parse::<u16>()
                .unwrap()
                .checked_mul(10)
        );
    }
}
