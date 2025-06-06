// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;

use super::TokenVec;
use super::field_info::FieldInfo;
use super::message_info::MessageInfo;
use otlp_model::OneofCase;

/// Emits the adapter struct and implementation for the visitor pattern
pub fn derive(msg: &MessageInfo) -> TokenStream {
    let outer_name = &msg.outer_name;
    let adapter_name = msg.related_typename("Adapter");
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
                    &format!("{}_{}_visitor", info.ident, case.name),
                    info.ident.span(),
                );

                let visitor_type = generate_visitor_type_for_oneof_variant(&info, &case);
                visitor_params.push(quote! { mut #variant_param_name: #visitor_type });
            }
        } else {
            let field_name = &info.ident;
            // Handle raw identifiers (r#keyword) by stripping the r# prefix
            let field_name_str = field_name.to_string();
            let clean_field_name = if field_name_str.starts_with("r#") {
                &field_name_str[2..]
            } else {
                &field_name_str
            };

            let visitor_param =
                syn::Ident::new(&format!("{}_visitor", clean_field_name), field_name.span());

            let visitor_trait = generate_visitor_trait_for_field(&info);
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

/// Generates  
fn generate_visitor_call(_info: &FieldInfo) -> proc_macro2::TokenStream {
    // TODO:
}

fn generate_visitor_type_for_oneof_variant(_info: &FieldInfo, _case: &OneofCase) -> syn::Type {
    // TODO
    // if let Ok(case_type) = syn::parse_str::<syn::Type>(case.type_param) {
    // let visitor_type: syn::Type; // TODO
}

fn generate_visitor_trait_for_field(_info: &FieldInfo) -> syn::Type {
    // TODO
}
