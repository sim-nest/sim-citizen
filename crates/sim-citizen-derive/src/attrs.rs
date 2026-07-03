//! Parsing of the #[citizen(...)] attribute arguments for the Citizen derive.

use syn::{Expr, LitStr, Path, spanned::Spanned};

pub(crate) struct CitizenAttrs {
    pub(crate) symbol: LitStr,
    pub(crate) version: u32,
}

impl CitizenAttrs {
    pub(crate) fn parse(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut symbol = None;
        let mut version = None;
        for attr in attrs.iter().filter(|attr| attr.path().is_ident("citizen")) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("symbol") {
                    symbol = Some(meta.value()?.parse::<LitStr>()?);
                    Ok(())
                } else if meta.path.is_ident("version") {
                    let value = meta.value()?.parse::<Expr>()?;
                    version = Some(expr_u32(&value)?);
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
                    Ok(())
                } else if meta.path.is_ident("with") {
                    let value = meta.value()?.parse::<LitStr>()?;
                    with = Some(value.parse::<Path>()?);
                    Ok(())
                } else if meta.path.is_ident("citizen") {
                    Ok(())
                } else {
                    Err(meta.error("unsupported citizen field attribute"))
                }
            })?;
        }
        Ok(Self { with })
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

#[cfg(test)]
mod tests {
    use syn::DeriveInput;

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
    }
}
