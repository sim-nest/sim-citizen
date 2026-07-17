//! Proc macros for SIM citizen values.
//!
//! `#[derive(Citizen)]` generates the citizen support for a domain type from its
//! `#[citizen(...)]` attributes (symbol, version, example/fixture hooks, and
//! field options), and the
//! `#[non_citizen]` attribute marks a type as an explicit, descriptor-named
//! exemption. The generated code targets the `sim-citizen` support layer.
//!
//! `Citizen` accepts this attribute grammar:
//!
//! - Type-level `#[citizen(...)]` keys:
//!   `symbol = "namespace/Name"` and `version = N` are required.
//! - `example = path::to::fixture_fn` is optional and names a zero-argument
//!   function that returns the canonical `Self` fixture. Without it, the derive
//!   uses `Default::default()`.
//! - `fixtures = path::to::fixtures_fn` is optional and names a zero-argument
//!   function whose return value implements `IntoIterator<Item = Self>`. Each
//!   emitted fixture runs through the conformance round-trip gate.
//! - Field-level `#[citizen(with = path::to::codec)]` is optional and names a
//!   module with `encode(&FieldTy) -> ::sim_kernel::Expr` and
//!   `decode(&::sim_kernel::Expr) -> ::sim_kernel::Result<FieldTy>`.
//! - Field markers `#[citizen(list)]` and `#[citizen(citizen)]` are rejected.
//!   Use `Vec<T>`, `Option<T>`, or an explicit `with` codec instead of inert
//!   syntax.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod attrs;
mod expand;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derives the SIM citizen support for a domain type.
///
/// Reads the type's `#[citizen(...)]` attributes and generates the read
/// constructor, constructor encoding, conformance fixture, and census
/// registration that the `sim-citizen` support layer expects.
///
/// The derive requires `symbol = "..."`
/// and `version = N`, optionally accepts `example = path::to::fixture_fn`
/// and `fixtures = path::to::fixtures_fn`, and accepts
/// `#[citizen(with = path::to::codec)]` on fields whose type needs a custom
/// field codec module. Apply it as `#[derive(Citizen)]`.
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
