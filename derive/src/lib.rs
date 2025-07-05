#![allow(non_snake_case)]

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
    generic_unit(args, item, "si_unit", "si", 1_000_000_000, 1)
}

#[proc_macro_attribute]
pub fn iec_unit(args: TokenStream, item: TokenStream) -> TokenStream {
    generic_unit(args, item, "iec_unit", "iec", 1, 10)
}

fn generic_unit(
    args: TokenStream,
    item: TokenStream,
    macro_name: &str,
    system: &str,
    min_prefix: u64,
    min_prefix_len: usize,
) -> TokenStream {
    let System: String = system
        .to_string()
        .chars()
        .enumerate()
        .map(|(i, ch)| if i == 0 { ch.to_ascii_uppercase() } else { ch })
        .collect();
    let SYSTEM = system.to_ascii_uppercase();
    let system = Ident::new(system, Span::call_site().into());
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
                        _ => panic!("`{macro_name}(symbol = \"...\")` should be a string literal"),
                    },
                    Expr::Group(group) => match &*group.expr {
                        Expr::Lit(literal) => match &literal.lit {
                            Lit::Str(symbol) => Some(symbol.value()),
                            _ => panic!(
                                "`{macro_name}(symbol = \"...\")` should be a string literal"
                            ),
                        },
                        _ => panic!("`{macro_name}(symbol = \"...\")` should be a literal"),
                    },
                    _ => panic!("`{macro_name}(symbol = \"...\")` should be a literal"),
                }
            }
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!("`{macro_name}` should at least contain `symbol = \"...\"` attribute")
        });
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
    let uint_string_len = max_string_len(uint.to_string().as_str());
    let max_string_len = uint_string_len + 1 + min_prefix_len + symbol.len();
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
    let write_unit = Ident::new(
        &format!("write_{system}_unit_{uint}"),
        Span::call_site().into(),
    );
    let from = Ident::new(&format!("from_{system}"), Span::call_site().into());
    let Format = Ident::new(&format!("Format{System}"), Span::call_site().into());
    let format = Ident::new(&format!("format_{system}"), Span::call_site().into());
    let FormatUnit = Ident::new(&format!("Format{System}Unit"), Span::call_site().into());
    let format_unit = Ident::new(&format!("format_{system}_unit"), Span::call_site().into());
    let UnitFromStr = Ident::new(&format!("{System}FromStr"), Span::call_site().into());
    let unit_from_str = Ident::new(&format!("{system}_unit_from_str"), Span::call_site().into());
    let serde = cfg!(feature = "serde").then_some(quote! {
        impl serde::Serialize for #newtype {
            fn serialize<S>(&self, s: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut buf = #crate_name::Buffer::<{ #newtype::MAX_STRING_LEN }>::new();
                buf.#write_unit(self.0, #symbol);
                s.serialize_str(unsafe { buf.as_str() })
            }
        }

        impl<'a> serde::Deserialize<'a> for #newtype {
            fn deserialize<D>(d: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'a>,
            {
                d.deserialize_str(#serde_visitor)
            }
        }

        struct #serde_visitor;

        impl<'a> serde::de::Visitor<'a> for #serde_visitor {
            type Value = #newtype;

            fn expecting(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                f.write_str(concat!("A string obtained by `", stringify!(#newtype), "::to_string`"))
            }

            fn visit_str<E>(self, value: &str) -> ::core::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value
                    .parse()
                    .map_err(|_| E::custom(concat!("Invalid `", stringify!(#newtype), "`")))
            }
        }
    });
    quote! {
        #item

        impl #newtype {
            /// Max. length in string form.
            pub const MAX_STRING_LEN: usize = #max_string_len;

            /// Unit symbol.
            pub const SYMBOL: &'static str = #symbol;

            #[doc = concat!("Convert from a value without ", #SYSTEM, " prefix.")]
            pub fn #from(value: #uint) -> Self {
                Self(value * #min_prefix)
            }
        }

        impl #crate_name::#system::#Format for #newtype {
            fn #format(&self) -> #crate_name::#system::FormattedUnit<'static> {
                #crate_name::#system::#FormatUnit::#format_unit(self.0, #symbol)
            }
        }

        impl ::core::fmt::Display for #newtype {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let mut buf = #crate_name::Buffer::<{ Self::MAX_STRING_LEN }>::new();
                buf.#write_unit(self.0, #symbol);
                f.write_str(unsafe { buf.as_str() })
            }
        }

        impl ::core::str::FromStr for #newtype {
            type Err = #crate_name::#system::Error;
            fn from_str(other: &str) -> Result<Self, Self::Err> {
                #crate_name::#system::#UnitFromStr::#unit_from_str(other, Self::SYMBOL).map(#newtype)
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
            assert_eq!(max, max_string_len(ty));
        }
    }
}
