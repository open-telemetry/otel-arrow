// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use quote::quote;

use super::TokenVec;
use super::common;
use super::field_info::FieldInfo;
use super::message_info::MessageInfo;

/// Emits the visitor, visitable and adapters methods.
pub fn derive(msg: &MessageInfo) -> TokenStream {
    let outer_name = &msg.outer_name;
    let visitor_name = msg.related_typename("Visitor");
    let visitable_name = msg.related_typename("Visitable");
    let visitor_method_name = common::visitor_method_name(&outer_name);
    let visitable_method_name = common::visitable_method_name(&outer_name);

    let mut visitable_args: TokenVec = Vec::new(); // For oneof fields, generate separate parameters for each variant
    for info in &msg.all_fields {
        if let Some(oneof_cases) = info.oneof.as_ref() {
            for case in oneof_cases {
                let variant_param_name =
                    common::oneof_variant_field_or_method_name(&info.ident, &case.name);

                // Use the centralized visitor type generation
                let visitor_type = FieldInfo::generate_visitor_type_for_oneof_case(case);
                visitable_args.push(quote! { #variant_param_name: impl #visitor_type });
            }
            continue;
        }

        // For non-oneof fields, use the precomputed visitor trait from FieldInfo
        let param_name = &info.ident;
        let visitor_type = &info.visitor_trait;
        visitable_args.push(quote! { #param_name: impl #visitor_type });
    }

    let expanded = quote! {
        pub trait #visitor_name<Argument> {
            fn #visitor_method_name(&mut self, arg: Argument, v: impl #visitable_name<Argument>) -> Argument;
        }

        pub trait #visitable_name<Argument> {
            fn #visitable_method_name(&self, arg: Argument, #(#visitable_args),*) -> Argument;
        }

        impl<Argument> #visitor_name<Argument> for crate::pdata::NoopVisitor {
            fn #visitor_method_name(&mut self, arg: Argument, _v: impl #visitable_name<Argument>) -> Argument {
                // NoopVisitor threads the argument through unchanged.
                arg
            }
        }
    };

    TokenStream::from(expanded)
}

/// Generate visitor call for a field with proper handling for different field types
///
/// This function has been redesigned to use the DRY principle with centralized utilities
/// from the common module, eliminating repetitive patterns.
pub fn generate_visitor_call(info: &FieldInfo) -> Option<proc_macro2::TokenStream> {
    // Handle oneof fields separately using centralized utility
    if let Some(oneof_cases) = info.oneof.as_ref() {
        return common::visitor_oneof_call(info, oneof_cases);
    }

    let field_name = &info.ident;
    let visitor_param = &info.visitor_param_name;
    let visit_method = &info.visit_method_name;

    // Use the centralized visitor call pattern generator
    let category = common::FieldCategory::from_field_info(info);
    Some(common::generate_visitor_call_pattern(
        category,
        field_name,
        visitor_param,
        visit_method,
        info,
    ))
}
