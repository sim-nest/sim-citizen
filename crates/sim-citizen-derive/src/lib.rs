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
/// Applied as `#[non_citizen]`, it returns the item unchanged and records at the
/// source that the type opts out of citizen conformance with a named descriptor
/// strategy, rather than being silently overlooked.
#[proc_macro_attribute]
pub fn non_citizen(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
