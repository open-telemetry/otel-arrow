// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;

mod builder;
mod common;
mod encoded_len;
mod field_info;
mod message_adapter;
mod message_info;
mod visitor;

use message_info::MessageInfo;

type TokenVec = Vec<proc_macro2::TokenStream>;

/// Attribute macro for associating the OTLP protocol buffer fully
/// qualified type name.
#[proc_macro_attribute]
pub fn qualified(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_str: String = args.to_string().trim_matches('"').into();

    // Parse input directly without round-trip conversion
    let mut input_ast = syn::parse_macro_input!(input as syn::DeriveInput);

    // Create a special doc comment that will store the qualified name
    let qualified_attr = syn::parse_quote! {
        #[doc(hidden, otlp_qualified_name = #args_str)]
    };

    // Add the attribute directly
    input_ast.attrs.push(qualified_attr);

    // Return the modified struct definition
    quote::quote!(#input_ast).into()
}

/// Derives the OTLP Message trait implementation for protocol buffer
/// message types. This enables additional OTLP-specific functionality
/// beyond what prost::Message provides.
#[proc_macro_derive(Message)]
pub fn derive_otlp_message(input: TokenStream) -> TokenStream {
    MessageInfo::new(input, |message_info| {
        let mut tokens = TokenStream::new();

        tokens.extend(builder::derive(&message_info));
        tokens.extend(visitor::derive(&message_info));
        tokens.extend(message_adapter::derive(&message_info));
        tokens.extend(encoded_len::derive(&message_info));

        tokens
    })
}

/// Create identifier with call_site span for generated code
// TODO belongs somewhere else
fn create_ident(name: &str) -> syn::Ident {
    syn::Ident::new(name, proc_macro2::Span::call_site())
}
