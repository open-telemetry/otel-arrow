// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;

use super::TokenVec;
use super::field_info::FieldInfo;
use super::message_info::MessageInfo;

/// Emits the adapter struct and implementation for the visitor pattern
pub fn derive(msg: &MessageInfo) -> TokenStream {
    let outer_name = &msg.outer_name;
    let adapter_name = msg.related_typename("MessageAdapter");
    let visitable_name = msg.related_typename("Visitable");

    // Generate the method name based on the outer type name
    // Convert CamelCase to snake_case (e.g., LogsData -> logs_data)
    let visitable_method_name = syn::Ident::new(
        &format!("accept_{}", outer_name.to_string().to_case(Case::Snake)),
        outer_name.span(),
    );

    // Generate visitor calls for each field
    let visitor_calls: TokenVec = msg.all_fields.iter().map(generate_visitor_call).collect();

    // Generate visitor parameters for the visitable trait method
    let mut visitor_params: TokenVec = Vec::new();

    for info in &msg.all_fields {
        if let Some(oneof_cases) = info.oneof.as_ref() {
            for case in oneof_cases {
                let variant_param_name = syn::Ident::new(
                    &format!("{}_{}", info.ident, case.name),
                    info.ident.span(),
                );

                let visitor_type_tokens = FieldInfo::generate_visitor_type_for_oneof_variant(&case);
                // Parse the token stream as a type for use in function signatures
                let visitor_type: syn::Type = syn::parse2(visitor_type_tokens)
                    .expect("Failed to parse visitor type tokens as syn::Type");
                visitor_params.push(quote! { mut #variant_param_name: impl #visitor_type });
            }
        } else {
            let visitor_param = &info.visitor_param_name;
            let visitor_trait = &info.visitor_trait;
            visitor_params.push(quote! { mut #visitor_param: impl #visitor_trait });
        }
    }

    let expanded = quote! {
        /// Message adapter for presenting OTLP message objects as visitable.
        pub struct #adapter_name<'a> {
            data: &'a #outer_name,
        }

        impl<'a> #adapter_name<'a> {
            /// Create a new message adapter
            pub fn new(data: &'a #outer_name) -> Self {
                Self { data }
            }
        }

        impl<'a, Argument> #visitable_name<Argument> for &#adapter_name<'a> {
            /// Visits a field of the associated type, passing child-visitors for the traversal.
            fn #visitable_method_name(&self, mut arg: Argument, #(#visitor_params),*) -> Argument {
                #(#visitor_calls)*
                arg
            }
        }
    };

    TokenStream::from(expanded)
}

/// Generates visitor call for a field
fn generate_visitor_call(info: &FieldInfo) -> proc_macro2::TokenStream {
    if let Some(call) = super::visitor::generate_visitor_call(info) {
        call
    } else {
        quote::quote! {}
    }
}


