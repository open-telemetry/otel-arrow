// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;

use super::TokenVec;
use super::field_info::FieldInfo;
use super::message_info::MessageInfo;
use super::visitor::{needs_adapter_for_field, is_bytes_type, get_base_type_name};
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

/// Generates visitor call for a field 
fn generate_visitor_call(info: &FieldInfo) -> proc_macro2::TokenStream {
    if let Some(call) = super::visitor::generate_visitor_call(info) {
        call
    } else {
        quote::quote! {}
    }
}

fn generate_visitor_type_for_oneof_variant(_info: &FieldInfo, case: &OneofCase) -> syn::Type {
    // For oneof variants, we need to determine the type from the case
    // This is a simplified implementation - might need refinement based on actual usage
    if let Ok(case_type) = syn::parse_str::<syn::Type>(&case.type_param) {
        if needs_adapter_for_field_type(&case_type) {
            // Generate the visitor trait name from the case type
            let type_name = get_base_type_name(&case_type);
            let visitor_trait_name = format!("{}Visitor", type_name);
            if let Ok(visitor_trait) = syn::parse_str::<syn::Type>(&visitor_trait_name) {
                syn::parse_quote! { #visitor_trait<Argument> }
            } else {
                syn::parse_quote! { crate::pdata::UnknownVisitor<Argument> }
            }
        } else {
            // For primitive types
            let base_type = get_base_type_name(&case_type);
            match base_type.as_str() {
                "String" => syn::parse_quote! { crate::pdata::StringVisitor<Argument> },
                "bool" => syn::parse_quote! { crate::pdata::BooleanVisitor<Argument> },
                "i32" => syn::parse_quote! { crate::pdata::I32Visitor<Argument> },
                "i64" => syn::parse_quote! { crate::pdata::I64Visitor<Argument> },
                "u32" | "u8" => syn::parse_quote! { crate::pdata::U32Visitor<Argument> },
                "u64" => syn::parse_quote! { crate::pdata::U64Visitor<Argument> },
                "f32" | "f64" => syn::parse_quote! { crate::pdata::F64Visitor<Argument> },
                _ => syn::parse_quote! { crate::pdata::UnknownVisitor<Argument> },
            }
        }
    } else {
        syn::parse_quote! { crate::pdata::UnknownVisitor<Argument> }
    }
}

/// Check if a type needs an adapter (helper for oneof processing)
fn needs_adapter_for_field_type(ty: &syn::Type) -> bool {
    !is_primitive_type_direct(ty) && !is_bytes_type(ty)
}

/// Check if a type is primitive (helper for oneof processing)
fn is_primitive_type_direct(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            matches!(
                segment.ident.to_string().as_str(),
                "String" | "bool" | "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64"
            )
        } else {
            false
        }
    } else {
        false
    }
}

fn generate_visitor_trait_for_field(info: &FieldInfo) -> syn::Type {
    // Check if this field needs an adapter (complex type) or is primitive
    if needs_adapter_for_field(info) {
        // For complex types, use the Visitor trait
        let visitor_trait = info.related_type("Visitor");
        syn::parse_quote! { #visitor_trait<Argument> }
    } else {
        // For primitive types, determine the appropriate visitor trait
        let base_type = get_base_type_name(&info.full_type_name);
        match base_type.as_str() {
            "String" => syn::parse_quote! { crate::pdata::StringVisitor<Argument> },
            "bool" => syn::parse_quote! { crate::pdata::BooleanVisitor<Argument> },
            "i32" => syn::parse_quote! { crate::pdata::I32Visitor<Argument> },
            "i64" => syn::parse_quote! { crate::pdata::I64Visitor<Argument> },
            "u32" | "u8" => syn::parse_quote! { crate::pdata::U32Visitor<Argument> },
            "u64" => syn::parse_quote! { crate::pdata::U64Visitor<Argument> },
            "f32" | "f64" => syn::parse_quote! { crate::pdata::F64Visitor<Argument> },
            _ => {
                if is_bytes_type(&info.full_type_name) {
                    syn::parse_quote! { crate::pdata::BytesVisitor<Argument> }
                } else {
                    syn::parse_quote! { crate::pdata::UnknownVisitor<Argument> }
                }
            }
        }
    }
}
