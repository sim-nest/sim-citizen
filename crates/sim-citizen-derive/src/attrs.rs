//! Parsing of the `#[citizen(...)]` attribute arguments for the Citizen derive.

use syn::{
    Expr, Item, LitStr, Path, meta::ParseNestedMeta, parse::Parser,
    parse::discouraged::Speculative, spanned::Spanned,
};

pub(crate) struct CitizenAttrs {
    pub(crate) symbol: LitStr,
    pub(crate) version: u32,
    pub(crate) example: Option<Path>,
    pub(crate) fixtures: Option<Path>,
}

impl CitizenAttrs {
    pub(crate) fn parse(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut symbol = None;
        let mut version = None;
        let mut example = None;
        let mut fixtures = None;
        for attr in attrs.iter().filter(|attr| attr.path().is_ident("citizen")) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("symbol") {
                    let value = meta.value()?.parse::<LitStr>()?;
                    if symbol.replace(value).is_some() {
                        return Err(meta.error("duplicate citizen symbol"));
                    }
                    Ok(())
                } else if meta.path.is_ident("version") {
                    let value = meta.value()?.parse::<Expr>()?;
                    if version.replace(expr_u32(&value)?).is_some() {
                        return Err(meta.error("duplicate citizen version"));
                    }
                    Ok(())
                } else if meta.path.is_ident("example") {
                    let value = parse_attr_path(&meta, "example")?;
                    if example.replace(value).is_some() {
                        return Err(meta.error("duplicate citizen example"));
                    }
                    Ok(())
                } else if meta.path.is_ident("fixtures") {
                    let value = parse_attr_path(&meta, "fixtures")?;
                    if fixtures.replace(value).is_some() {
                        return Err(meta.error("duplicate citizen fixtures"));
                    }
                    Ok(())
                } else {
                    Err(meta.error("unsupported citizen attribute"))
                }
            })?;
        }
        Ok(Self {
            symbol: symbol.ok_or_else(|| {
                syn::Error::new(proc_macro2::Span::call_site(), "missing citizen symbol")
            })?,
            version: version.ok_or_else(|| {
                syn::Error::new(proc_macro2::Span::call_site(), "missing citizen version")
            })?,
            example,
            fixtures,
        })
    }
}

pub(crate) struct FieldAttrs {
    pub(crate) with: Option<Path>,
}

impl FieldAttrs {
    pub(crate) fn parse(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut with = None;
        for attr in attrs.iter().filter(|attr| attr.path().is_ident("citizen")) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("list") {
                    Err(meta.error(
                        "unsupported citizen field mode list; use Vec<T> field types directly",
                    ))
                } else if meta.path.is_ident("with") {
                    let value = parse_attr_path(&meta, "with")?;
                    if with.replace(value).is_some() {
                        return Err(meta.error("duplicate citizen field with"));
                    }
                    Ok(())
                } else if meta.path.is_ident("citizen") {
                    Err(meta.error(
                        "unsupported citizen field mode citizen; use explicit field codecs instead",
                    ))
                } else {
                    Err(meta.error("unsupported citizen field attribute"))
                }
            })?;
        }
        Ok(Self { with })
    }
}

pub(crate) struct NonCitizenAttrs {
    pub(crate) reason: LitStr,
    pub(crate) kind: LitStr,
    pub(crate) descriptor: LitStr,
}

impl NonCitizenAttrs {
    pub(crate) fn parse(tokens: proc_macro2::TokenStream) -> syn::Result<Self> {
        let mut reason = None;
        let mut kind = None;
        let mut descriptor = None;
        let parser = syn::meta::parser(|meta| {
            if meta.path.is_ident("reason") {
                let value = parse_non_empty_string("reason", meta.value()?.parse::<LitStr>()?)?;
                if reason.replace(value).is_some() {
                    return Err(meta.error("duplicate non_citizen reason"));
                }
                Ok(())
            } else if meta.path.is_ident("kind") {
                let value = parse_non_empty_string("kind", meta.value()?.parse::<LitStr>()?)?;
                if kind.replace(value).is_some() {
                    return Err(meta.error("duplicate non_citizen kind"));
                }
                Ok(())
            } else if meta.path.is_ident("descriptor") {
                let value = parse_non_empty_string("descriptor", meta.value()?.parse::<LitStr>()?)?;
                if descriptor.replace(value).is_some() {
                    return Err(meta.error("duplicate non_citizen descriptor"));
                }
                Ok(())
            } else {
                Err(meta.error("unsupported non_citizen attribute"))
            }
        });
        parser.parse2(tokens)?;
        Ok(Self {
            reason: reason.ok_or_else(|| {
                syn::Error::new(proc_macro2::Span::call_site(), "missing non_citizen reason")
            })?,
            kind: kind.ok_or_else(|| {
                syn::Error::new(proc_macro2::Span::call_site(), "missing non_citizen kind")
            })?,
            descriptor: descriptor.ok_or_else(|| {
                syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "missing non_citizen descriptor",
                )
            })?,
        })
    }
}

pub(crate) fn item_type_name(item: &Item) -> syn::Result<LitStr> {
    match item {
        Item::Struct(item) => Ok(LitStr::new(&item.ident.to_string(), item.ident.span())),
        Item::Enum(item) => Ok(LitStr::new(&item.ident.to_string(), item.ident.span())),
        Item::Union(item) => Ok(LitStr::new(&item.ident.to_string(), item.ident.span())),
        Item::Type(item) => Ok(LitStr::new(&item.ident.to_string(), item.ident.span())),
        other => Err(syn::Error::new_spanned(
            other,
            "#[non_citizen] supports only structs, enums, unions, and type aliases",
        )),
    }
}

fn expr_u32(expr: &Expr) -> syn::Result<u32> {
    let Expr::Lit(expr_lit) = expr else {
        return Err(syn::Error::new(expr.span(), "expected integer literal"));
    };
    let syn::Lit::Int(lit) = &expr_lit.lit else {
        return Err(syn::Error::new(expr.span(), "expected integer literal"));
    };
    lit.base10_parse::<u32>()
}

fn parse_attr_path(meta: &ParseNestedMeta<'_>, field: &str) -> syn::Result<Path> {
    let value = meta.value()?;
    let fork = value.fork();
    if let Ok(path) = fork.parse::<Path>() {
        value.advance_to(&fork);
        return Ok(path);
    }
    let literal = value
        .parse::<LitStr>()
        .map_err(|_| value.error(format!("expected path for citizen {field}")))?;
    literal.parse::<Path>()
}

fn parse_non_empty_string(field: &str, value: LitStr) -> syn::Result<LitStr> {
    if value.value().trim().is_empty() {
        Err(syn::Error::new_spanned(
            &value,
            format!("non_citizen {field} cannot be empty"),
        ))
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use syn::{DeriveInput, Field, Item};

    use super::*;

    #[test]
    fn citizen_attrs_parse_symbol_and_version() {
        let input: DeriveInput = syn::parse_quote! {
            #[citizen(symbol = "example/Point", version = 1)]
            struct Point { x: i64 }
        };
        let attrs = CitizenAttrs::parse(&input.attrs).unwrap();
        assert_eq!(attrs.symbol.value(), "example/Point");
        assert_eq!(attrs.version, 1);
        assert!(attrs.example.is_none());
        assert!(attrs.fixtures.is_none());
    }

    #[test]
    fn citizen_attrs_parse_example_and_fixtures_paths() {
        let input: DeriveInput = syn::parse_quote! {
            #[citizen(
                symbol = "example/Point",
                version = 1,
                example = point_example,
                fixtures = "point_fixtures"
            )]
            struct Point { x: i64 }
        };
        let attrs = CitizenAttrs::parse(&input.attrs).unwrap();
        assert_eq!(attrs.example.unwrap().segments[0].ident, "point_example");
        assert_eq!(attrs.fixtures.unwrap().segments[0].ident, "point_fixtures");
    }

    #[test]
    fn field_attrs_parse_string_codec_path() {
        let field: Field = syn::parse_quote! {
            #[citizen(with = "point_codec")]
            value: i64
        };
        let attrs = FieldAttrs::parse(&field.attrs).unwrap();
        assert_eq!(attrs.with.unwrap().segments[0].ident, "point_codec");
    }

    #[test]
    fn non_citizen_attrs_parse_descriptor_contract() {
        let attrs = NonCitizenAttrs::parse(quote::quote!(
            reason = "runtime-owned state",
            kind = "live-handle",
            descriptor = "example/live-handle"
        ))
        .unwrap();
        assert_eq!(attrs.reason.value(), "runtime-owned state");
        assert_eq!(attrs.kind.value(), "live-handle");
        assert_eq!(attrs.descriptor.value(), "example/live-handle");
    }

    #[test]
    fn item_type_name_accepts_type_items() {
        let input: Item = syn::parse_quote! {
            struct ExampleLiveHandle;
        };
        let name = item_type_name(&input).unwrap();
        assert_eq!(name.value(), "ExampleLiveHandle");
    }
}
