//! Proc macros for SIM citizen values.
//!
//! `#[derive(Citizen)]` generates the citizen support for a domain type from its
//! `#[citizen(...)]` attributes (symbol, version, and field options), and the
//! `#[non_citizen]` attribute marks a type as an explicit, descriptor-named
//! exemption. The generated code targets the `sim-citizen` support layer.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod attrs;
mod expand;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derives the SIM citizen support for a domain type.
///
/// Reads the type's `#[citizen(symbol = "...", version = N, ...)]` attributes
/// and generates the read constructor, constructor encoding, conformance
/// fixture, and census registration that the `sim-citizen` support layer
/// expects. Apply it as `#[derive(Citizen)]`.
#[proc_macro_derive(Citizen, attributes(citizen))]
pub fn derive_citizen(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::expand_citizen(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Marks a type as an explicit non-citizen exemption.
///
/// Applied as `#[non_citizen(reason = "...", kind = "...", descriptor = "...")]`,
/// it preserves the input item and emits an inventory row recording that the
/// type opts out of citizen conformance with a named descriptor strategy,
/// rather than being silently overlooked.
#[proc_macro_attribute]
pub fn non_citizen(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_ts = proc_macro2::TokenStream::from(item.clone());
    let input = parse_macro_input!(item as syn::Item);
    let attrs = match attrs::NonCitizenAttrs::parse(attr.into()) {
        Ok(attrs) => attrs,
        Err(err) => return err.into_compile_error().into(),
    };
    let type_name = match attrs::item_type_name(&input) {
        Ok(type_name) => type_name,
        Err(err) => return err.into_compile_error().into(),
    };
    let reason = attrs.reason;
    let kind = attrs.kind;
    let descriptor = attrs.descriptor;
    quote::quote! {
        #item_ts
        ::sim_citizen::inventory::submit! {
            ::sim_citizen::NonCitizenInfo {
                type_name: #type_name,
                crate_name: env!("CARGO_PKG_NAME"),
                reason: #reason,
                kind: #kind,
                descriptor: #descriptor,
            }
        }
    }
    .into()
}
