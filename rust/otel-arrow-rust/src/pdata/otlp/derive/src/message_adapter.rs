// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;

use super::TokenVec;
use super::field_info::FieldInfo;
use super::message_info::MessageInfo;
use super::visitor::{get_base_type_name, is_bytes_type, needs_adapter_for_field};
use otlp_model::OneofCase;

/// Emits the adapter struct and implementation for the visitor pattern
pub fn derive(msg: &MessageInfo) -> TokenStream {
    //eprintln!("🚨 DEBUG: Starting message_adapter::derive for: {}", msg.outer_name);

    let outer_name = &msg.outer_name;
    let adapter_name = msg.related_typename("Adapter");
    let visitable_name = msg.related_typename("Visitable");

    //eprintln!("🚨 DEBUG: Generated names - adapter: {}, visitable: {}", adapter_name, visitable_name);

    // Generate the method name based on the outer type name
    // Convert CamelCase to snake_case (e.g., LogsData -> logs_data)
    let visitable_method_name = syn::Ident::new(
        &format!("accept_{}", outer_name.to_string().to_case(Case::Snake)),
        outer_name.span(),
    );

    //eprintln!("🚨 DEBUG: Generated visitable method name: {}", visitable_method_name);

    // Generate visitor calls for each field
    //eprintln!("🚨 DEBUG: About to generate visitor calls for {} fields", msg.all_fields.len());
    let visitor_calls: TokenVec = msg.all_fields.iter().map(generate_visitor_call).collect();
    //eprintln!("🚨 DEBUG: Generated {} visitor calls", visitor_calls.len());

    // Generate visitor parameters for the visitable trait method
    //eprintln!("🚨 DEBUG: About to generate visitor parameters");
    let mut visitor_params: TokenVec = Vec::new();

    for info in &msg.all_fields {
        //eprintln!("🚨 DEBUG: Processing field for visitor params: {}", info.ident);

        if let Some(oneof_cases) = info.oneof.as_ref() {
            //eprintln!("🚨 DEBUG: Field {} has oneof with {} cases", info.ident, oneof_cases.len());
            for case in oneof_cases {
                let variant_param_name = syn::Ident::new(
                    &format!("{}_{}", info.ident, case.name),
                    info.ident.span(),
                );

                let visitor_type = generate_visitor_type_for_oneof_variant(&info, &case);
                visitor_params.push(quote! { mut #variant_param_name: impl #visitor_type });
            }
        } else {
            //eprintln!("🚨 DEBUG: Field {} is not oneof, generating regular visitor param", info.ident);
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

            //eprintln!("🚨 DEBUG: About to generate visitor trait for field: {}", info.ident);
            let visitor_trait = generate_visitor_trait_for_field(&info);
            //eprintln!("🚨 DEBUG: Generated visitor trait for field: {}", info.ident);
            visitor_params.push(quote! { mut #visitor_param: impl #visitor_trait });
        }
    }

    //eprintln!("🚨 DEBUG: Generated {} visitor parameters", visitor_params.len());
    //eprintln!("🚨 DEBUG: About to generate final quote! block");

    // Debug individual components before combining
    //eprintln!("🚨 DEBUG: adapter_name: {}", adapter_name);
    //eprintln!("🚨 DEBUG: outer_name: {}", outer_name);
    //eprintln!("🚨 DEBUG: visitable_name: {}", visitable_name);
    //eprintln!("🚨 DEBUG: visitable_method_name: {}", visitable_method_name);
    //eprintln!("🚨 DEBUG: visitor_params count: {}", visitor_params.len());
    //eprintln!("🚨 DEBUG: visitor_calls count: {}", visitor_calls.len());

    //eprintln!("🚨 DEBUG: About to execute quote! macro");
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

    //eprintln!("🚨 DEBUG: quote! macro completed successfully");

    //eprintln!("🚨 DEBUG: Successfully generated quote! block");
    //eprintln!("🚨 DEBUG: About to return TokenStream");

    TokenStream::from(expanded)
}

/// Generates visitor call for a field
fn generate_visitor_call(info: &FieldInfo) -> proc_macro2::TokenStream {
    //eprintln!("🚨 DEBUG: Generating visitor call for field: {}", info.ident);
    if let Some(call) = super::visitor::generate_visitor_call(info) {
        //eprintln!("🚨 DEBUG: Successfully generated visitor call for field: {}", info.ident);
        call
    } else {
        //eprintln!("🚨 DEBUG: No visitor call generated for field: {}, using empty block", info.ident);
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
            
            // For Vec types that need adapters, these are typically bytes
            if type_name == "Vec" {
                syn::parse_quote! { crate::pdata::BytesVisitor<Argument> }
            } else if let Ok(visitor_trait) = syn::parse_str::<syn::Type>(&visitor_trait_name) {
                syn::parse_quote! { #visitor_trait<Argument> }
            } else {
                // For message types, try unqualified first, then fallback
                let visitor_ident = syn::Ident::new(&visitor_trait_name, proc_macro2::Span::call_site());
                syn::parse_quote! { #visitor_ident<Argument> }
            }
        } else {
            // For primitive types
            if is_bytes_type(&case_type) {
                syn::parse_quote! { crate::pdata::BytesVisitor<Argument> }
            } else {
                let base_type = get_base_type_name(&case_type);
                match base_type.as_str() {
                    "String" => syn::parse_quote! { crate::pdata::StringVisitor<Argument> },
                    "bool" => syn::parse_quote! { crate::pdata::BooleanVisitor<Argument> },
                    "i32" => syn::parse_quote! { crate::pdata::I32Visitor<Argument> },
                    "i64" => syn::parse_quote! { crate::pdata::I64Visitor<Argument> },
                    "u32" | "u8" => syn::parse_quote! { crate::pdata::U32Visitor<Argument> },
                    "u64" => syn::parse_quote! { crate::pdata::U64Visitor<Argument> },
                    "f32" | "f64" => syn::parse_quote! { crate::pdata::F64Visitor<Argument> },
                    "Vec" => syn::parse_quote! { crate::pdata::BytesVisitor<Argument> }, // Vec in protobuf context is typically bytes
                    _ => {
                        // For message types, generate the appropriate visitor trait
                        let visitor_trait_name = format!("{}Visitor", base_type);
                        let visitor_ident = syn::Ident::new(&visitor_trait_name, proc_macro2::Span::call_site());
                        syn::parse_quote! { #visitor_ident<Argument> }
                    },
                }
            }
        }
    } else {
        // If we can't parse the type, panic as this should not happen
        panic!("Failed to parse type for visitor generation");
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
                "String"
                    | "bool"
                    | "i8"
                    | "i16"
                    | "i32"
                    | "i64"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
                    | "f32"
                    | "f64"
            )
        } else {
            false
        }
    } else {
        false
    }
}

fn generate_visitor_trait_for_field(info: &FieldInfo) -> syn::Type {
    //eprintln!("🚨 DEBUG: generate_visitor_trait_for_field called for field: {}", info.ident);

    // Check if this field needs an adapter (complex type) or is primitive
    if needs_adapter_for_field(info) {
        //eprintln!("🚨 DEBUG: Field {} needs adapter, generating visitor trait", info.ident);
        // For complex types, use the Visitor trait - note: related_type already includes <Argument>
        let visitor_trait = info.related_type("Visitor");

        // Parse the visitor trait directly as a type (it already includes <Argument>)
        match syn::parse2::<syn::Type>(visitor_trait.clone()) {
            Ok(parsed_type) => {
                //eprintln!("🚨 DEBUG: Successfully parsed visitor trait as Type: {}", quote! { #parsed_type });
                parsed_type
            }
            Err(_e) => {
                //eprintln!("🚨 DEBUG: Failed to parse visitor trait '{}' as Type: {}", visitor_trait, e);
                // This should not happen, panic for debugging
                panic!("Failed to parse visitor trait: {}", visitor_trait);
            }
        }
    } else {
        eprintln!("🚨 DEBUG: Field {} is primitive, generating primitive visitor", info.ident);
        
        // Check if this is a repeated primitive field first
        if info.is_repeated {
            eprintln!("🚨 DEBUG: Field {} is repeated primitive, using SliceVisitor", info.ident);
            // For repeated primitive types, use SliceVisitor
            match info.base_type_name.as_str() {
                "String" => syn::parse_quote! { crate::pdata::SliceVisitor<Argument, String> },
                "bool" => syn::parse_quote! { crate::pdata::SliceVisitor<Argument, bool> },
                "i32" => syn::parse_quote! { crate::pdata::SliceVisitor<Argument, i32> },
                "i64" => syn::parse_quote! { crate::pdata::SliceVisitor<Argument, i64> },
                "u32" | "u8" => syn::parse_quote! { crate::pdata::SliceVisitor<Argument, u32> },
                "u64" => syn::parse_quote! { crate::pdata::SliceVisitor<Argument, u64> },
                "f32" => syn::parse_quote! { crate::pdata::SliceVisitor<Argument, f32> },
                "f64" => syn::parse_quote! { crate::pdata::SliceVisitor<Argument, f64> },
                _ => {
                    panic!("Unknown repeated primitive type for visitor generation: {}", info.base_type_name);
                }
            }
        } else {
            // For non-repeated primitive types, determine the appropriate visitor trait
            eprintln!("🚨 DEBUG: Using info.base_type_name: '{}'", info.base_type_name);
            match info.base_type_name.as_str() {
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
                        panic!("Unknown primitive type for visitor generation: {}", info.base_type_name);
                    }
                }
            }
        }
    }
}
