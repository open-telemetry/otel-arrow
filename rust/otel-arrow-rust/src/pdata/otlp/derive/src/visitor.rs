// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};

use super::TokenVec;
use super::field_info::FieldInfo;
use super::message_info::MessageInfo;

/// Emits the visitor, visitable and adapters methods.
pub fn derive(msg: &MessageInfo) -> TokenStream {
    let outer_name = &msg.outer_name;
    let visitor_name = msg.related_typename("Visitor");
    let visitable_name = msg.related_typename("Visitable");
    let visitor_method_name = syn::Ident::new(
        &format!("visit_{}", outer_name).to_case(Case::Snake),
        outer_name.span(),
    );
    let visitable_method_name = syn::Ident::new(
        &format!("accept_{}", outer_name).to_case(Case::Snake),
        outer_name.span(),
    );

    let mut visitable_args: TokenVec = Vec::new();

    for info in &msg.all_fields {
        if let Some(oneof_cases) = info.oneof.as_ref() {
            // For oneof fields, generate separate parameters for each variant
            for case in oneof_cases {
                let variant_param_name =
                    syn::Ident::new(&format!("{}_{}", info.ident, case.name), info.ident.span());

                // Parse the type_param to get the visitor trait path
                if let Ok(case_type) = syn::parse_str::<syn::Type>(&case.type_param) {
                    let visitor_type = generate_visitor_type_for_oneof_variant(&case_type);
                    visitable_args.push(quote! { #variant_param_name: impl #visitor_type });
                }
            }
            continue;
        }

        // For non-oneof fields, generate normal visitor parameter
        let param_name = &info.ident;
        let visitor_type = generate_visitor_trait_for_field(info);
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
pub fn generate_visitor_call(info: &FieldInfo) -> Option<proc_macro2::TokenStream> {
    // Oneof fields are handled separately
    if info.oneof.is_some() {
        return None; // Will be handled by generate_oneof_visitor_calls
    }

    let field_name = &info.ident;
    let field_name_str = field_name.to_string();
    let clean_field_name = if field_name_str.starts_with("r#") {
        &field_name_str[2..]
    } else {
        &field_name_str
    };

    let visitor_param = visitor_param_name(&clean_field_name);

    let visit_method = generate_visit_method_for_field(info);
    let needs_adapter = needs_adapter_for_field(info);
    let is_bytes_field = is_bytes_type(&info.full_type_name); // Vec<u8> specifically

    match (
        info.is_optional,
        info.is_repeated,
        needs_adapter,
        is_bytes_field,
    ) {
        (false, false, true, _) => {
            let adapter_name = get_adapter_name_for_field(info);
            Some(quote! {
                arg = #visitor_param.#visit_method(arg, &(#adapter_name::new(&self.data.#field_name)));
            })
        }
        (false, false, false, _) => {
            if matches!(visit_method.to_string().as_str(), "visit_string") {
                Some(quote! { arg = #visitor_param.#visit_method(arg, &self.data.#field_name); })
            } else if is_bytes_field {
                // For bytes fields (Vec<u8>), pass as slice
                Some(quote! { arg = #visitor_param.#visit_method(arg, &self.data.#field_name); })
            } else {
                Some(quote! { arg = #visitor_param.#visit_method(arg, *&self.data.#field_name); })
            }
        }
        (true, false, true, _) => {
            let adapter_name = get_adapter_name_for_field(info);
            Some(quote! {
                if let Some(f) = &self.data.#field_name {
                    arg = #visitor_param.#visit_method(arg, &(#adapter_name::new(f)));
                }
            })
        }
        (true, false, false, _) => {
            if matches!(visit_method.to_string().as_str(), "visit_string") {
                Some(quote! {
                    if let Some(f) = &self.data.#field_name {
                        arg = #visitor_param.#visit_method(arg, f);
                    }
                })
            } else if is_bytes_field {
                // For bytes fields (Vec<u8>), pass as slice
                Some(quote! {
                    if let Some(f) = &self.data.#field_name {
                        arg = #visitor_param.#visit_method(arg, f);
                    }
                })
            } else {
                Some(quote! {
                    if let Some(f) = &self.data.#field_name {
                        arg = #visitor_param.#visit_method(arg, *f);
                    }
                })
            }
        }
        (false, true, true, _) => {
            let adapter_name = get_adapter_name_for_field(info);
            Some(quote! {
                for item in &self.data.#field_name {
                    arg = #visitor_param.#visit_method(arg, &(#adapter_name::new(item)));
                }
            })
        }
        (false, true, false, true) => {
            // For bytes fields (Vec<u8>), pass as slice
            Some(quote! { arg = #visitor_param.#visit_method(arg, &self.data.#field_name); })
        }
        (false, true, false, false) => {
            if matches!(visit_method.to_string().as_str(), "visit_string") {
                Some(quote! {
                    for item in &self.data.#field_name {
                        arg = #visitor_param.#visit_method(arg, item);
                    }
                })
            } else {
                Some(quote! {
                    for item in &self.data.#field_name {
                        arg = #visitor_param.#visit_method(arg, *item);
                    }
                })
            }
        }
        (true, true, _, _) => {
            if needs_adapter {
                let adapter_name = get_adapter_name_for_field(info);
                Some(quote! {
                    if let Some(items) = &self.data.#field_name {
                        for item in items {
                            arg = #visitor_param.#visit_method(arg, &(#adapter_name::new(item)));
                        }
                    }
                })
            } else if is_bytes_field {
                Some(quote! {
                    if let Some(items) = &self.data.#field_name {
                        arg = #visitor_param.#visit_method(arg, items);
                    }
                })
            } else if matches!(visit_method.to_string().as_str(), "visit_string") {
                Some(quote! {
                    if let Some(items) = &self.data.#field_name {
                        for item in items {
                            arg = #visitor_param.#visit_method(arg, item);
                        }
                    }
                })
            } else {
                Some(quote! {
                    if let Some(items) = &self.data.#field_name {
                        for item in items {
                            arg = #visitor_param.#visit_method(arg, *item);
                        }
                    }
                })
            }
        }
    }
}

// /// Generate visitor calls for oneof fields based on their variants
// pub fn generate_oneof_visitor_calls(
//     info: &FieldInfo,
// ) -> Vec<proc_macro2::TokenStream> {
//     let mut visitor_calls = Vec::new();

//     if let Some(oneof_cases) = &info.oneof {
//         for case in oneof_cases {
//             let variant_param_name = syn::Ident::new(
//                 &format!("{}_{}_visitor", info.ident, case.name),
//                 info.ident.span(),
//             );

//             // Generate visitor call for this oneof variant
//             let field_name = &info.ident;
//             let variant_path = syn::parse_str::<syn::Expr>(case.value_variant).unwrap();

//             visitor_calls.push(quote! {
//                 if let Some(#variant_path(ref value)) = self.data.#field_name {
//                     arg = #variant_param_name.visit(arg, value);
//                 }
//             });
//         }
//     }

//     visitor_calls
// }

/// Generate visitor parameter name for a field
fn visitor_param_name(field_name: &str) -> syn::Ident {
    syn::Ident::new(
        &format!("{}_visitor", field_name),
        proc_macro2::Span::call_site(),
    )
}

/// Generate the correct visit method name for a field based on its type
fn generate_visit_method_for_field(info: &FieldInfo) -> syn::Ident {
    // DEBUG: Print all the field information to understand the logic
    eprintln!("🚨 DEBUG generate_visit_method_for_field:");
    eprintln!("  field_name: {}", info.ident);
    eprintln!("  info.is_primitive: {}", info.is_primitive);
    eprintln!("  info.is_repeated: {}", info.is_repeated);
    eprintln!("  info.full_type_name: {}", info.full_type_name.to_token_stream());
    eprintln!("  info.base_type_name: {}", info.base_type_name);
    eprintln!("  is_primitive_type(&info.full_type_name): {}", is_primitive_type(&info.full_type_name));
    eprintln!("  is_bytes_type(&info.full_type_name): {}", is_bytes_type(&info.full_type_name));

    let method_name = if info.is_primitive && is_bytes_type(&info.full_type_name) {
        // CASE 1: Primitive bytes field (Vec<u8>)
        // This checks BOTH info.is_primitive AND is_bytes_type() - redundant!
        eprintln!("  -> CASE 1: Primitive bytes");
        "visit_bytes".to_string()
    } else if is_primitive_type(&info.full_type_name) {
        // CASE 2: Direct primitive type (u64, f64, String, etc.)
        // This should handle: u64, f64, String, bool, i32, i64, u32, f32
        // BUT it only works if full_type_name is the primitive directly, not Vec<u64>
        eprintln!("  -> CASE 2: Direct primitive type");
        let suffix = get_primitive_method_suffix(&info.full_type_name);
        eprintln!("    primitive suffix: {}", suffix);
        format!("visit_{}", suffix)
    } else if info.is_repeated && info.is_primitive {
        // CASE 3: Repeated primitive field (Vec<u64>, Vec<f64>, etc.)
        // ISSUE: This branch exists because is_primitive_type(Vec<u64>) = false
        // BUT info.is_primitive should be true for the same field!
        // This suggests is_primitive_type() and info.is_primitive use different logic
        eprintln!("  -> CASE 3: Repeated primitive (this is confusing!)");
        
        if let Some(inner_type) = extract_vec_inner_type(&info.full_type_name) {
            eprintln!("    extracted inner_type: {}", inner_type.to_token_stream());
            let suffix = get_primitive_method_suffix(&inner_type);
            eprintln!("    inner primitive suffix: {}", suffix);
            format!("visit_{}", suffix)
        } else {
            // FALLBACK 3A: This shouldn't happen if the logic above is correct
            eprintln!("    -> FALLBACK 3A: Failed to extract inner type, using snake_case");
            let type_name = &info.base_type_name;
            eprintln!("    base_type_name for snake_case: {}", type_name);
            format!("visit_{}", type_name.to_case(Case::Snake))
        }
    } else {
        // CASE 4: Message types or other non-primitive types
        eprintln!("  -> CASE 4: Non-primitive (message type)");
        let type_name = &info.base_type_name;
        eprintln!("    base_type_name for snake_case: {}", type_name);
        format!("visit_{}", type_name.to_case(Case::Snake))
    };

    eprintln!("  FINAL method_name: {}", method_name);
    eprintln!("");
    
    syn::Ident::new(&method_name, proc_macro2::Span::call_site())
}

/// Extract the inner type from Vec<T> -> T
fn extract_vec_inner_type(ty: &syn::Type) -> Option<syn::Type> {
    if let syn::Type::Path(type_path) = ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            if last_segment.ident == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty.clone());
                    }
                }
            }
        }
    }
    None
}

/// Determine if a field needs to be wrapped in an adapter (message types) vs used directly (primitives)
pub fn needs_adapter_for_field(info: &FieldInfo) -> bool {
    // For repeated fields, check the inner type, not the Vec type
    let type_to_check = if info.is_repeated {
        // For repeated fields, we need to check the element type
        // The base_type_name should contain the element type for repeated fields
        // We can reconstruct the inner type from base_type_name and qualifier
        if let Some(ref qualifier) = info.qualifier {
            let base_ident = syn::Ident::new(&info.base_type_name, proc_macro2::Span::call_site());
            syn::parse_quote! { #qualifier::#base_ident }
        } else {
            let base_ident = syn::Ident::new(&info.base_type_name, proc_macro2::Span::call_site());
            syn::parse_quote! { #base_ident }
        }
    } else {
        info.full_type_name.clone()
    };
    
    info.is_message || !is_primitive_type(&type_to_check)
}

/// Determine if a type is a primitive type that doesn't need an adapter
fn is_primitive_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            match last_segment.ident.to_string().as_str() {
                "String" | "u32" | "u64" | "i32" | "i64" | "f32" | "f64" | "bool" => true,
                "Vec" => {
                    // Check if it's Vec<u8> (bytes)
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        if let Some(syn::GenericArgument::Type(syn::Type::Path(inner_path))) =
                            args.args.first()
                        {
                            if let Some(inner_segment) = inner_path.path.segments.last() {
                                return inner_segment.ident == "u8";
                            }
                        }
                    }
                    false
                }
                _ => false,
            }
        } else {
            false
        }
    } else {
        false
    }
}

/// Check if a type is a bytes type
pub fn is_bytes_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            match last_segment.ident.to_string().as_str() {
                "Vec" => {
                    // Check if it's Vec<u8>
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        if let Some(syn::GenericArgument::Type(syn::Type::Path(inner_path))) =
                            args.args.first()
                        {
                            if let Some(inner_segment) = inner_path.path.segments.last() {
                                return inner_segment.ident == "u8";
                            }
                        }
                    }
                    false
                }
                _ => false,
            }
        } else {
            false
        }
    } else {
        false
    }
}

/// Get the method suffix for primitive types
fn get_primitive_method_suffix(ty: &syn::Type) -> String {
    if let syn::Type::Path(type_path) = ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            match last_segment.ident.to_string().as_str() {
                "String" => "string".to_string(),
                "u32" => "u32".to_string(),
                "u64" => "u64".to_string(),
                "i32" => "i32".to_string(),
                "i64" => "i64".to_string(),
                "f32" => "f32".to_string(),
                "f64" => "f64".to_string(),
                "bool" => "bool".to_string(),
                name => name.to_case(Case::Snake),
            }
        } else {
            "unknown".to_string()
        }
    } else {
        "unknown".to_string()
    }
}

/// Get the adapter name for a field type
fn get_adapter_name_for_field(info: &FieldInfo) -> proc_macro2::TokenStream {
    let adapter_suffix = "Adapter";
    info.related_type(adapter_suffix)
}

/// Extract the base type name from a syn::Type
pub fn get_base_type_name(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                segment.ident.to_string()
            } else {
                "unknown".to_string()
            }
        }
        _ => "unknown".to_string(),
    }
}

/// Generate visitor type for a oneof variant using simplified logic
fn generate_visitor_type_for_oneof_variant(case_type: &syn::Type) -> proc_macro2::TokenStream {
    // Handle Vec<u8> as bytes
    if is_bytes_type(case_type) {
        return quote! { crate::pdata::BytesVisitor<Argument> };
    }

    // Parse the type to get base name and qualifier
    if let syn::Type::Path(type_path) = case_type {
        let segments = &type_path.path.segments;

        if let Some(last_segment) = segments.last() {
            let base_type_name = last_segment.ident.to_string();

            // Handle specific known cases
            if base_type_name == "ArrayValue" {
                return quote! { super::ArrayValueVisitor<Argument> };
            }
            if base_type_name == "KeyValueList" {
                return quote! { super::KeyValueListVisitor<Argument> };
            }

            // Handle primitive types
            let visitor_trait = match base_type_name.as_str() {
                "String" => quote! { crate::pdata::StringVisitor<Argument> },
                "bool" => quote! { crate::pdata::BooleanVisitor<Argument> },
                "i32" => quote! { crate::pdata::I32Visitor<Argument> },
                "i64" => quote! { crate::pdata::I64Visitor<Argument> },
                "u32" | "u8" => quote! { crate::pdata::U32Visitor<Argument> },
                "u64" => quote! { crate::pdata::U64Visitor<Argument> },
                "f32" => quote! { crate::pdata::F32Visitor<Argument> },
                "f64" => quote! { crate::pdata::F64Visitor<Argument> },
                "Vec" => quote! { crate::pdata::VecVisitor<Argument> },
                _ => {
                    // For message types, construct visitor name with qualifier
                    let visitor_name = format!("{}Visitor", base_type_name);
                    let visitor_ident =
                        syn::Ident::new(&visitor_name, proc_macro2::Span::call_site());

                    if segments.len() > 1 {
                        // Has qualifier - reconstruct the path without the last segment
                        let mut qualifier_segments = segments.clone();
                        qualifier_segments.pop(); // Remove the last segment (base type)

                        let qualifier: proc_macro2::TokenStream = qualifier_segments
                            .iter()
                            .map(|seg| &seg.ident)
                            .collect::<Vec<_>>()
                            .into_iter()
                            .map(|ident| quote! { #ident })
                            .collect::<Vec<_>>()
                            .into_iter()
                            .reduce(|acc, segment| quote! { #acc::#segment })
                            .unwrap_or_else(|| quote! {});

                        quote! { #qualifier::#visitor_ident<Argument> }
                    } else {
                        quote! { #visitor_ident<Argument> }
                    }
                }
            };

            return visitor_trait;
        }
    }

    // Fallback for unknown types - should not reach here in normal cases
    eprintln!("🚨 WARNING: Falling back to UnknownVisitor for type: {:?}", case_type);
    quote! { crate::pdata::UnknownVisitor<Argument> }
}

// /// Decompose a type into base type name and qualifier, similar to FieldInfo::decompose_type
// /// but simplified for oneof variant processing
// fn decompose_type_for_oneof(ty: &syn::Type) -> (String, Option<proc_macro2::TokenStream>) {
//     match ty {
//         syn::Type::Path(type_path) => {
//             // Get the base type name (last segment)
//             let base_type_name = type_path
//                 .path
//                 .segments
//                 .last()
//                 .map(|seg| seg.ident.to_string())
//                 .unwrap_or_else(|| "Unknown".to_string());

//             // If there's only one segment, no qualifier needed
//             if type_path.path.segments.len() == 1 {
//                 return (base_type_name, None);
//             }

//             // Create qualifier from all but last segment
//             let qualifier_segments: Vec<_> = type_path
//                 .path
//                 .segments
//                 .iter()
//                 .take(type_path.path.segments.len() - 1)
//                 .collect();

//             if qualifier_segments.is_empty() {
//                 (base_type_name, None)
//             } else {
//                 let qualifier = qualifier_segments
//                     .iter()
//                     .map(|seg| &seg.ident)
//                     .collect::<Vec<_>>();

//                 let qualifier_token = quote::quote! { #(#qualifier)::* };
//                 (base_type_name, Some(qualifier_token))
//             }
//         }
//         _ => {
//             // For non-path types, just use the string representation as base name
//             let base_name = quote::quote! { #ty }.to_string();
//             (base_name, None)
//         }
//     }
// }

/// Generate visitor trait for a field based on its type  
fn generate_visitor_trait_for_field(info: &FieldInfo) -> proc_macro2::TokenStream {
    // Use the centralized logic from FieldInfo
    info.related_type("Visitor")
}

// /// Generate visitor trait for a given type
// fn generate_visitor_trait_for_type(ty: &syn::Type) -> proc_macro2::TokenStream {
//     if let syn::Type::Path(type_path) = ty {
//         if let Some(segment) = type_path.path.segments.last() {
//             let type_name = segment.ident.to_string();

//             if is_primitive_type(ty) {
//                 get_primitive_visitor_trait(&type_name)
//             } else {
//                 // For message types, generate visitor trait
//                 let visitor_name = format!("{}Visitor", type_name);
//                 let visitor_ident = syn::Ident::new(&visitor_name, segment.ident.span());
//                 quote! { #visitor_ident<Argument> }
//             }
//         } else {
//             quote! { UnknownVisitor<Argument> }
//         }
//     } else {
//         quote! { UnknownVisitor<Argument> }
//     }
// }

// /// Generate visitor trait for a given type with proper path qualification
// fn generate_visitor_trait_for_type_with_path(ty: &syn::Type) -> proc_macro2::TokenStream {
//     if let syn::Type::Path(type_path) = ty {
//         if let Some(segment) = type_path.path.segments.last() {
//             let type_name = segment.ident.to_string();

//             if is_primitive_type(ty) {
//                 get_primitive_visitor_trait(&type_name)
//             } else {
//                 // For message types, generate visitor trait with full path
//                 let visitor_name = format!("{}Visitor", type_name);
//                 let visitor_ident = syn::Ident::new(&visitor_name, segment.ident.span());

//                 // If there's more than one segment, include the qualifier
//                 if type_path.path.segments.len() > 1 {
//                     let qualifier_segments: Vec<_> = type_path
//                         .path
//                         .segments
//                         .iter()
//                         .take(type_path.path.segments.len() - 1)
//                         .map(|seg| &seg.ident)
//                         .collect();

//                     quote! { #(#qualifier_segments)::*::#visitor_ident<Argument> }
//                 } else {
//                     quote! { #visitor_ident<Argument> }
//                 }
//             }
//         } else {
//             quote! { UnknownVisitor<Argument> }
//         }
//     } else {
//         quote! { UnknownVisitor<Argument> }
//     }
// }

// /// Get primitive type visitor trait with generic argument
// fn get_primitive_visitor_trait(type_name: &str) -> proc_macro2::TokenStream {
//     match type_name {
//         "String" => quote! { crate::pdata::StringVisitor<Argument> },
//         "bool" => quote! { crate::pdata::BooleanVisitor<Argument> },
//         "i32" => quote! { crate::pdata::I32Visitor<Argument> },
//         "i64" => quote! { crate::pdata::I64Visitor<Argument> },
//         "u32" | "u8" => quote! { crate::pdata::U32Visitor<Argument> },
//         "u64" => quote! { crate::pdata::U64Visitor<Argument> },
//         "f32" => quote! { crate::pdata::F32Visitor<Argument> },
//         "f64" => quote! { crate::pdata::F64Visitor<Argument> },
//         _ => quote! { UnknownVisitor<Argument> },
//     }
// }
