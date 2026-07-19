//! Code generation for the Citizen derive macro.

use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr};

use crate::attrs::{CitizenAttrs, FieldAttrs};

pub(crate) fn expand_citizen(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            input.generics,
            "Citizen derive supports non-generic structs",
        ));
    }

    let ident = input.ident;
    let attrs = CitizenAttrs::parse(&input.attrs)?;
    let fields = named_fields(&input.data)?;
    let field_count = fields.len();
    let field_names = fields
        .iter()
        .map(|field| field.ident.as_ref().expect("named field").to_string())
        .collect::<Vec<_>>();
    let field_name_literals = field_names
        .iter()
        .map(|name| LitStr::new(name, proc_macro2::Span::call_site()))
        .collect::<Vec<_>>();
    let symbol = attrs.symbol;
    let version = attrs.version;
    let version_token = format!("v{version}");
    let example = attrs.example;
    let fixtures = attrs.fixtures;

    let encodes = fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().expect("named field");
            let attrs = FieldAttrs::parse(&field.attrs)?;
            Ok(match attrs.with {
                Some(path) => quote!(#path::encode(&self.#name)),
                None => quote!(::sim_citizen::CitizenField::encode_field(&self.#name)),
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let decodes = fields
        .iter()
        .zip(field_name_literals.iter())
        .map(|(field, field_name)| {
            let name = field.ident.as_ref().expect("named field");
            let ty = &field.ty;
            let attrs = FieldAttrs::parse(&field.attrs)?;
            Ok(match attrs.with {
                Some(path) => quote! {
                    let #name: #ty = {
                        let __value = __iter.next().expect("arity checked");
                        let __expr = ::sim_citizen::value_to_expr(cx, __value, #field_name)?;
                        #path::decode(&__expr)?
                    };
                },
                None => quote! {
                    let #name: #ty = {
                        let __value = __iter.next().expect("arity checked");
                        <#ty as ::sim_citizen::CitizenField>::decode_field_value(
                            cx,
                            __value,
                            #field_name,
                        )?
                    };
                },
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let field_idents = fields
        .iter()
        .map(|field| field.ident.as_ref().expect("named field"))
        .collect::<Vec<_>>();

    let example_body = example.as_ref().map_or_else(
        || quote!(<Self as ::std::default::Default>::default()),
        |path| quote!(#path()),
    );
    let conformance_default_check = example.as_ref().map_or_else(
        || quote!(::sim_citizen::check_default_fixture::<#ident>(cx)?;),
        |path| quote!(::sim_citizen::check_fixture(cx, #path())?;),
    );
    let conformance_fixture_checks = fixtures.as_ref().map_or_else(
        || quote!(),
        |path| {
            quote! {
                for __fixture in ::core::iter::IntoIterator::into_iter(#path()) {
                    ::sim_citizen::check_fixture(cx, __fixture)?;
                }
            }
        },
    );

    Ok(quote! {
        impl ::sim_citizen::Citizen for #ident {
            fn citizen_symbol() -> ::sim_kernel::Symbol {
                ::sim_citizen::parse_symbol(#symbol)
            }

            fn citizen_version() -> u32 {
                #version
            }

            fn citizen_arity() -> usize {
                #field_count
            }

            fn citizen_fields() -> &'static [&'static str] {
                &[#(#field_name_literals),*]
            }
        }

        impl ::sim_kernel::Object for #ident {
            fn display(&self, cx: &mut ::sim_kernel::Cx) -> ::sim_kernel::Result<String> {
                match ::sim_kernel::ObjectEncode::object_encoding(self, cx)? {
                    ::sim_kernel::ObjectEncoding::Constructor { class, args } => {
                        Ok(format!("#<citizen {}:{}>", class, args.len()))
                    }
                    _ => Ok("#<citizen>".to_owned()),
                }
            }

            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }
        }

        impl ::sim_kernel::ObjectCompat for #ident {
            fn class(&self, cx: &mut ::sim_kernel::Cx) -> ::sim_kernel::Result<::sim_kernel::ClassRef> {
                let symbol = <Self as ::sim_citizen::Citizen>::citizen_symbol();
                if let Some(value) = cx.registry().class_by_symbol(&symbol) {
                    return Ok(value.clone());
                }
                cx.factory().class_stub(::sim_kernel::ClassId(0), symbol)
            }

            fn as_expr(&self, cx: &mut ::sim_kernel::Cx) -> ::sim_kernel::Result<::sim_kernel::Expr> {
                ::sim_citizen::constructor_expr(cx, self)
            }

            fn as_object_encoder(&self) -> Option<&dyn ::sim_kernel::ObjectEncode> {
                Some(self)
            }
        }

        impl ::sim_kernel::ObjectEncode for #ident {
            fn object_encoding(
                &self,
                _cx: &mut ::sim_kernel::Cx,
            ) -> ::sim_kernel::Result<::sim_kernel::ObjectEncoding> {
                Ok(::sim_kernel::ObjectEncoding::Constructor {
                    class: <Self as ::sim_citizen::Citizen>::citizen_symbol(),
                    args: vec![
                        ::sim_kernel::Expr::Symbol(::sim_kernel::Symbol::new(#version_token)),
                        #(#encodes),*
                    ],
                })
            }
        }

        impl ::sim_citizen::CitizenRuntime for #ident {
            fn citizen_info() -> ::sim_citizen::CitizenInfo {
                ::sim_citizen::CitizenInfo {
                    symbol: #symbol,
                    version: #version,
                    crate_name: env!("CARGO_PKG_NAME"),
                    arity: #field_count,
                    install: <Self as ::sim_citizen::CitizenRuntime>::install,
                    conformance: <Self as ::sim_citizen::CitizenRuntime>::conformance,
                }
            }

            fn conformance(cx: &mut ::sim_kernel::Cx) -> ::sim_kernel::Result<()> {
                #conformance_default_check
                #conformance_fixture_checks
                Ok(())
            }

            fn construct_from_values(
                cx: &mut ::sim_kernel::Cx,
                args: Vec<::sim_kernel::Value>,
            ) -> ::sim_kernel::Result<Self> {
                let expected = <Self as ::sim_citizen::Citizen>::citizen_arity() + 1;
                if args.len() != expected {
                    return Err(::sim_citizen::arity_error(
                        <Self as ::sim_citizen::Citizen>::citizen_symbol(),
                        expected,
                        args.len(),
                    ));
                }
                let mut __iter = args.into_iter();
                let __version = __iter.next().expect("arity checked");
                ::sim_citizen::decode_version(
                    cx,
                    __version,
                    <Self as ::sim_citizen::Citizen>::citizen_version(),
                    <Self as ::sim_citizen::Citizen>::citizen_symbol(),
                )?;
                #(#decodes)*
                Ok(Self {
                    #(#field_idents),*
                })
            }

            fn example() -> Self {
                #example_body
            }
        }

        const _: () = {
            ::sim_citizen::inventory::submit! {
                ::sim_citizen::CitizenInfo {
                    symbol: #symbol,
                    version: #version,
                    crate_name: env!("CARGO_PKG_NAME"),
                    arity: #field_count,
                    install: <#ident as ::sim_citizen::CitizenRuntime>::install,
                    conformance: <#ident as ::sim_citizen::CitizenRuntime>::conformance,
                }
            }
        };
    })
}

fn named_fields(
    data: &Data,
) -> syn::Result<&syn::punctuated::Punctuated<syn::Field, syn::Token![,]>> {
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => Ok(&fields.named),
            other => Err(syn::Error::new_spanned(
                other,
                "Citizen derive supports named-field structs",
            )),
        },
        _ => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Citizen derive supports structs",
        )),
    }
}
