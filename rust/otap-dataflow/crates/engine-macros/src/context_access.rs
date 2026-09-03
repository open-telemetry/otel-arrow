// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the `context_access` macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result, Token, Visibility, braced};

/// Input for `context_access!`.
pub(crate) struct ContextAccess {
    visibility: Visibility,
    name: Ident,
    fields: Vec<Ident>,
    constants: Vec<Ident>,
}

impl Parse for ContextAccess {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let visibility = input.parse()?;
        let _ = input.parse::<Token![struct]>()?;
        let name = input.parse()?;
        let content;
        braced!(content in input);
        let fields: Vec<Ident> = content
            .parse_terminated(Ident::parse, Token![,])?
            .into_iter()
            .collect();

        let mut constants = Vec::new();
        while !input.is_empty() {
            let _ = input.parse::<Token![const]>()?;
            constants.push(input.parse()?);
            let _ = input.parse::<Token![;]>()?;
        }

        if fields.is_empty() {
            return Err(syn::Error::new_spanned(
                name,
                "context_access requires at least one access field",
            ));
        }
        if constants.is_empty() {
            return Err(syn::Error::new_spanned(
                name,
                "context_access requires at least one constant",
            ));
        }
        Ok(Self {
            visibility,
            name,
            fields,
            constants,
        })
    }
}

/// Expands provider-local access constants with sequential identifiers.
pub(crate) fn expand_context_access(input: ContextAccess) -> TokenStream {
    let ContextAccess {
        visibility,
        name,
        fields,
        constants,
    } = input;
    let engine = engine_crate_path();
    let field_definitions = fields.iter().map(|field| {
        quote! { #field: #engine::context_declaration::ContextAccessId, }
    });
    let constant_definitions = constants
        .iter()
        .enumerate()
        .map(|(constant_index, constant)| {
            let field_initializers = fields.iter().enumerate().map(|(field_index, field)| {
                let id = constant_index * fields.len() + field_index;
                quote! {
                    #field: #engine::context_declaration::ContextAccessId::new(#id),
                }
            });
            quote! {
                #visibility const #constant: #name = #name {
                    #(#field_initializers)*
                };
            }
        });

    quote! {
        #[derive(Clone, Copy)]
        #visibility struct #name {
            #(#field_definitions)*
        }

        #(#constant_definitions)*
    }
}

fn engine_crate_path() -> TokenStream {
    if std::env::var("CARGO_CRATE_NAME").as_deref() == Ok("otel_arrow_dfe_engine") {
        quote! { crate }
    } else {
        quote! { ::otel_arrow_dfe_engine }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: A component declares a pair of access fields for each signal.
    /// Guarantees: generated module-level constants assign sequential IDs.
    #[test]
    fn generates_flat_constants_for_each_access_group() {
        let input = syn::parse_str(
            "struct Access { topic, partition } const TRACES_ACCESS; const METRICS_ACCESS;",
        )
        .unwrap();
        let output = expand_context_access(input).to_string();

        assert!(output.contains("const TRACES_ACCESS : Access = Access"));
        assert!(output.contains("topic : :: otel_arrow_dfe_engine :: context_declaration :: ContextAccessId :: new (0usize)"));
        assert!(output.contains("partition : :: otel_arrow_dfe_engine :: context_declaration :: ContextAccessId :: new (1usize)"));
        assert!(output.contains("const METRICS_ACCESS : Access = Access"));
        assert!(output.contains("topic : :: otel_arrow_dfe_engine :: context_declaration :: ContextAccessId :: new (2usize)"));
        assert!(output.contains("partition : :: otel_arrow_dfe_engine :: context_declaration :: ContextAccessId :: new (3usize)"));
    }
}
