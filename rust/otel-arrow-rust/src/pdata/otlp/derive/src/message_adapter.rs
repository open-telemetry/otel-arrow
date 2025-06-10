// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use quote::quote;

use super::TokenVec;
use super::common;
use super::field_info::FieldInfo;
use super::message_info::MessageInfo;

/// Emits the adapter struct and implementation for the visitor pattern
pub fn derive(msg: &MessageInfo) -> TokenStream {
    let outer_name = &msg.outer_name;
    let adapter_name = msg.related_typename("MessageAdapter");
    let visitable_name = msg.related_typename("Visitable");

    // Generate the method name based on the outer type name
    let visitable_method_name = common::visitable_method_name(&outer_name);

    // Generate visitor calls for each field
    let visitor_calls: TokenVec = msg.all_fields.iter().map(generate_visitor_call).collect();

    // Generate visitor parameters for all fields including oneof variants
    let visitor_params = common::visitor_formal_parameters(&msg.all_fields);

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
            fn #visitable_method_name(&mut self, mut arg: Argument, #(#visitor_params),*) -> Argument {
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
