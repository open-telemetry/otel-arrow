// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! EncodedLen visitor generation for protobuf size calculation.
//!
//! This module implements the fourth generation step of the OTLP macro system,
//! creating EncodedLen visitors that implement the visitor pattern for
//! `PrecomputedSizes` arguments. These visitors calculate the total encoded
//! size of protobuf messages using a two-pass algorithm.
//!
//! ## Generated Code Pattern
//!
//! For each message type, this generates:
//! 1. An `EncodedLen` struct containing the protobuf tag
//! 2. A visitor implementation that accumulates sizes using `PrecomputedSizes`
//! 3. Proper handling of message fields, primitive fields, and oneof variants
//!
//! ## Algorithm
//!
//! The size calculation follows the two-pass pattern described in prd.md:
//! 1. **First Pass (this module)**: Calculate sizes using visitor traversal
//! 2. **Second Pass (future)**: Encode data using precomputed sizes
//!
//! The generated visitors use `PrecomputedSizes` helper methods for:
//! - Reserving space for size calculations (`reserve()`)
//! - Getting child sizes (`get_size()`)
//! - Setting computed sizes (`set_size()`)

use proc_macro::TokenStream;
use quote::quote;

use super::TokenVec;
use super::common;
use super::field_info::FieldInfo;
use super::message_info::MessageInfo;

/// Generate the EncodedLen visitor implementation for a message type.
///
/// This creates an `EncodedLen` struct and visitor implementation that calculates
/// the total encoded size of a protobuf message using the PrecomputedSizes argument.
/// The implementation follows the visitor pattern established by the existing
/// generator steps.
pub fn derive(msg: &MessageInfo) -> TokenStream {
    let outer_name = &msg.outer_name;
    let encoded_len_name = msg.related_typename("EncodedLen");
    let visitor_name = msg.related_typename("Visitor");
    let visitable_name = msg.related_typename("Visitable");
    let visitor_method_name = common::visitor_method_name(&outer_name);

    // Generate the children_size helper method body for the visitor
    let visitor_body = generate_helper_method_body(msg);

    let expanded = quote! {
        /// EncodedLen visitor for calculating protobuf encoded size
        pub struct #encoded_len_name {
            /// Protocol buffer tag number for this message
            pub tag: u32,
        }

        impl #encoded_len_name {
            /// Create a new EncodedLen visitor with the specified tag
            pub fn new(tag: u32) -> Self {
                Self { tag }
            }

            /// Helper method to calculate the sum of direct children's encoded lengths.
            /// This method processes each child field individually via visitable interface
            /// to avoid double-counting nested descendants.
            fn children_encoded_size(
                mut arg: crate::pdata::otlp::PrecomputedSizes,
                v: impl #visitable_name<crate::pdata::otlp::PrecomputedSizes>
            ) -> (crate::pdata::otlp::PrecomputedSizes, usize) {
                let mut total = 0;
                
                #visitor_body
                
                (arg, total)
            }
        }

        impl #visitor_name<crate::pdata::otlp::PrecomputedSizes> for #encoded_len_name {
            fn #visitor_method_name(
                &mut self,
                mut arg: crate::pdata::otlp::PrecomputedSizes,
                v: impl #visitable_name<crate::pdata::otlp::PrecomputedSizes>
            ) -> crate::pdata::otlp::PrecomputedSizes {
                // Reserve space for this message's size calculation
                let my_idx = arg.len();
                arg.reserve();

                // Calculate total size of direct children using helper method
                let (updated_arg, total_child_size) = Self::children_encoded_size(arg, v);
                arg = updated_arg;

                // Calculate this message's total size
                // Formula: tag_size + length_varint_size + content_size
                let tag_size = crate::pdata::otlp::PrecomputedSizes::varint_len((self.tag << 3) as usize);

                // Store the computed size
                arg.set_size(my_idx, tag_size, total_child_size);

                arg
            }
        }
    };

    TokenStream::from(expanded)
}

/// Generate the helper method body that calculates encoded sizes of direct children.
///
/// This generates code to be used in the children_encoded_size method on the visitor.
/// It processes each field individually using visitable method calls instead of direct field access.
fn generate_helper_method_body(msg: &MessageInfo) -> proc_macro2::TokenStream {
    let mut field_processing = Vec::new();

    for info in &msg.all_fields {
        if let Some(oneof_cases) = info.oneof.as_ref() {
            // Process each oneof variant individually
            for case in oneof_cases {
                let field_name = &info.ident;
                
                // Generate the visitable method name for this field
                let visitable_method = common::visitable_method_name(&field_name);

                let visitor_instantiation = if case.is_primitive {
                    generate_primitive_visitor_instantiation(case, &info.tag)
                } else {
                    generate_message_visitor_instantiation(case, &info.tag)
                };

                // For oneof variants, call the visitable method which will handle the pattern matching
                field_processing.push(quote! {
                    // Process oneof field: #field_name
                    let idx = arg.len();
                    arg.reserve();
                    let mut visitor = #visitor_instantiation;
                    arg = v.#visitable_method(arg, &mut visitor);
                    total += arg.get_size(idx);
                });
            }
        } else {
            // Process regular field
            let field_name = &info.ident;
            
            // Generate the visitable method name for this field
            let visitable_method = common::visitable_method_name(&field_name);

            let visitor_instantiation = if info.is_primitive {
                generate_primitive_visitor_for_field(info)
            } else {
                generate_message_visitor_for_field(info)
            };

            // Call the visitable method for this field
            field_processing.push(quote! {
                // Process field: #field_name
                let idx = arg.len();
                arg.reserve();
                let mut visitor = #visitor_instantiation;
                arg = v.#visitable_method(arg, &mut visitor);
                total += arg.get_size(idx);
            });
        }
    }

    quote! {
        #(#field_processing)*
    }
}

/// Generate child visitor instantiations for each field.
///
/// Creates the appropriate visitor instances based on field types:
/// - Primitive fields use primitive-specific encoders (BooleanEncodedLen, StringEncodedLen, etc.)
/// - Message fields use recursive EncodedLen visitors
/// - Oneof fields generate separate visitors for each variant
fn generate_child_visitor_instantiations(fields: &[FieldInfo]) -> TokenVec {
    let mut visitors = Vec::new();

    for info in fields {
        if let Some(oneof_cases) = info.oneof.as_ref() {
            // Generate separate visitors for each oneof variant
            for case in oneof_cases {
                let variant_param_name =
                    common::oneof_variant_field_or_method_name(&info.ident, &case.name);

                let visitor_instantiation = if case.is_primitive {
                    generate_primitive_visitor_instantiation(case, &info.tag)
                } else {
                    generate_message_visitor_instantiation(case, &info.tag)
                };

                visitors.push(quote! {
                    let mut #variant_param_name = #visitor_instantiation;
                });
            }
        } else {
            // Regular field visitor
            let visitor_param = &info.visitor_param_name;

            let visitor_instantiation = if info.is_primitive {
                generate_primitive_visitor_for_field(info)
            } else {
                generate_message_visitor_for_field(info)
            };

            visitors.push(quote! {
                let mut #visitor_param = #visitor_instantiation;
            });
        }
    }

    visitors
}

/// Generate primitive visitor instantiation for a field.
fn generate_primitive_visitor_for_field(info: &FieldInfo) -> proc_macro2::TokenStream {
    let tag = &info.tag;

    // Handle bytes fields specially - they use BytesEncodedLen regardless of base_type_name
    if info.proto_type.contains("bytes") {
        let visitor_type = if info.is_repeated {
            quote! { crate::pdata::otlp::SliceBytesEncodedLen }
        } else {
            quote! { crate::pdata::otlp::BytesEncodedLen }
        };
        return quote! { #visitor_type { tag: #tag } };
    }

    // For repeated primitive fields, use slice visitors
    if info.is_repeated {
        let visitor_type = match info.base_type_name.as_str() {
            "bool" => quote! { crate::pdata::otlp::SliceBooleanEncodedLen },
            "String" | "string" => quote! { crate::pdata::otlp::SliceStringEncodedLen },
            "u32" => quote! { crate::pdata::otlp::SliceU32EncodedLen },
            "u64" => quote! { crate::pdata::otlp::SliceU64EncodedLen },
            "i32" => quote! { crate::pdata::otlp::SliceI32EncodedLen },
            "i64" => quote! { crate::pdata::otlp::SliceI64EncodedLen },
            "f64" => quote! { crate::pdata::otlp::SliceDoubleEncodedLen },
            "f32" => quote! { crate::pdata::otlp::SliceFixed32EncodedLen },
            _ => {
                // For enums and other types, use SliceI32EncodedLen as default
                quote! { crate::pdata::otlp::SliceI32EncodedLen }
            }
        };
        quote! { #visitor_type { tag: #tag } }
    } else {
        // For non-repeated primitive fields, use individual visitors
        let visitor_type = match info.base_type_name.as_str() {
            "bool" => quote! { crate::pdata::otlp::BooleanEncodedLen },
            "String" | "string" => quote! { crate::pdata::otlp::StringEncodedLen },
            "u32" => quote! { crate::pdata::otlp::U32EncodedLen },
            "u64" => quote! { crate::pdata::otlp::U64EncodedLen },
            "i32" => quote! { crate::pdata::otlp::I32EncodedLen },
            "i64" => quote! { crate::pdata::otlp::I64EncodedLen },
            "f64" => quote! { crate::pdata::otlp::DoubleEncodedLen },
            "f32" => quote! { crate::pdata::otlp::Fixed32EncodedLen },
            _ => {
                // For enums and other types, use I32EncodedLen as default
                quote! { crate::pdata::otlp::I32EncodedLen }
            }
        };
        quote! { #visitor_type { tag: #tag } }
    }
}

/// Generate message visitor instantiation for a field.
fn generate_message_visitor_for_field(info: &FieldInfo) -> proc_macro2::TokenStream {
    let encoded_len_type = info.related_type("EncodedLen");
    let tag = &info.tag;
    quote! { #encoded_len_type::new(#tag) }
}

/// Generate primitive visitor instantiation for an oneof case.
fn generate_primitive_visitor_instantiation(
    case: &otlp_model::OneofCase,
    tag: &u32,
) -> proc_macro2::TokenStream {
    let visitor_type = match case.type_param {
        "bool" => quote! { crate::pdata::otlp::BooleanEncodedLen },
        "::prost::alloc::string::String" => quote! { crate::pdata::otlp::StringEncodedLen },
        "Vec<u8>" => quote! { crate::pdata::otlp::BytesEncodedLen },
        "u32" => quote! { crate::pdata::otlp::U32EncodedLen },
        "u64" => quote! { crate::pdata::otlp::U64EncodedLen },
        "i32" => quote! { crate::pdata::otlp::I32EncodedLen },
        "i64" => quote! { crate::pdata::otlp::I64EncodedLen },
        "f64" => quote! { crate::pdata::otlp::DoubleEncodedLen },
        "f32" => quote! { crate::pdata::otlp::Fixed32EncodedLen },
        _ => {
            // For enums and other types, use I32EncodedLen as default
            quote! { crate::pdata::otlp::I32EncodedLen }
        }
    };

    quote! { #visitor_type { tag: #tag } }
}

/// Generate message visitor instantiation for an oneof case.
fn generate_message_visitor_instantiation(
    case: &otlp_model::OneofCase,
    tag: &u32,
) -> proc_macro2::TokenStream {
    // Check if there's an extra_call that specifies the adapter type
    if let Some(extra_call) = &case.extra_call {
        // Extract the type name from the extra_call (e.g., "KeyValueList::new" -> "KeyValueList")
        let adapter_type_name = if extra_call.contains("::new") {
            extra_call.replace("::new", "")
        } else {
            extra_call.to_string()
        };

        let encoded_len_type_str = format!("{}EncodedLen", adapter_type_name);

        let encoded_len_type =
            syn::parse_str::<syn::Type>(&encoded_len_type_str).unwrap_or_else(|e| {
                panic!(
                    "Failed to parse adapter EncodedLen type: {} (error: {:?})",
                    encoded_len_type_str, e
                );
            });

        quote! { #encoded_len_type::new(#tag) }
    } else {
        // Regular message type handling
        let type_name = case.type_param;
        let encoded_len_type = if type_name.contains("::") {
            let parts: Vec<&str> = type_name.split("::").collect();
            let last_part = parts.last().unwrap();
            let type_str = format!("{}EncodedLen", last_part);
            syn::parse_str::<syn::Type>(&type_str).unwrap_or_else(|e| {
                panic!(
                    "Failed to parse EncodedLen type: {} (error: {:?})",
                    type_str, e
                );
            })
        } else {
            let type_str = format!("{}EncodedLen", type_name);
            syn::parse_str::<syn::Type>(&type_str).unwrap_or_else(|e| {
                panic!(
                    "Failed to parse EncodedLen type: {} (error: {:?})",
                    type_str, e
                );
            })
        };

        quote! { #encoded_len_type::new(#tag) }
    }
}

/// Generate the visitable call with all child visitors.
fn generate_visitable_call(
    fields: &[FieldInfo],
    visitable_method_name: &syn::Ident,
) -> proc_macro2::TokenStream {
    let mut visitor_args = Vec::new();

    for info in fields {
        if let Some(oneof_cases) = info.oneof.as_ref() {
            // Add arguments for each oneof variant
            for case in oneof_cases {
                let variant_param_name =
                    common::oneof_variant_field_or_method_name(&info.ident, &case.name);
                visitor_args.push(quote! { #variant_param_name });
            }
        } else {
            // Regular field visitor argument
            let visitor_param = &info.visitor_param_name;
            visitor_args.push(quote! { #visitor_param });
        }
    }

    quote! {
        arg = v.#visitable_method_name(arg, #(#visitor_args),*);
    }
}

/// Generate child visitor parameters for visitor method signature.
/// This is currently unused but might be needed for more complex implementations.
fn generate_child_visitor_params(fields: &[FieldInfo]) -> TokenVec {
    let mut params = Vec::new();

    for info in fields {
        if let Some(oneof_cases) = info.oneof.as_ref() {
            // Generate parameters for each oneof variant
            for case in oneof_cases {
                let variant_param_name =
                    common::oneof_variant_field_or_method_name(&info.ident, &case.name);
                let visitor_type = FieldInfo::generate_visitor_type_for_oneof_case(case);
                params.push(quote! { #variant_param_name: impl #visitor_type });
            }
        } else {
            // Regular field parameter
            let param_name = &info.ident;
            let visitor_type = &info.visitor_trait;
            params.push(quote! { #param_name: impl #visitor_type });
        }
    }

    params
}

/// Generate visitable call for a regular field.
fn generate_visitable_call_for_field(info: &FieldInfo, visitor_param: &syn::Ident) -> proc_macro2::TokenStream {
    let field_name = &info.ident;
    
    if info.is_primitive {
        // For primitive fields, call the visitor's visit method directly  
        let visit_method = &info.visit_method_name;
        if info.is_repeated {
            quote! {
                #visitor_param.visit_slice(sizes, &v.#field_name)
            }
        } else if info.is_optional {
            quote! {
                if let Some(ref value) = v.#field_name {
                    #visitor_param.#visit_method(sizes, value)
                } else {
                    sizes
                }
            }
        } else {
            quote! {
                #visitor_param.#visit_method(sizes, v.#field_name)
            }
        }
    } else {
        // For message fields, call the visitable method
        let base_type_ident = syn::Ident::new(&info.base_type_name, proc_macro2::Span::call_site());
        let visitable_method = common::visitable_method_name(&base_type_ident);
        
        if info.is_repeated {
            quote! {
                v.#field_name.iter().fold(sizes, |acc, item| {
                    item.#visitable_method(acc, &mut #visitor_param)
                })
            }
        } else if info.is_optional {
            quote! {
                if let Some(ref item) = v.#field_name {
                    item.#visitable_method(sizes, &mut #visitor_param)
                } else {
                    sizes
                }
            }
        } else {
            quote! {
                v.#field_name.#visitable_method(sizes, &mut #visitor_param)
            }
        }
    }
}

/// Generate visitable call for an oneof variant.
fn generate_visitable_call_for_oneof(
    info: &FieldInfo, 
    case: &otlp_model::OneofCase,
    visitor_param: &syn::Ident
) -> proc_macro2::TokenStream {
    let field_name = &info.ident;
    let value_variant = case.value_variant;
    
    // Parse the value_variant to get the enum type and variant name
    let variant_path = syn::parse_str::<syn::Path>(value_variant).unwrap_or_else(|e| {
        panic!("Failed to parse variant path: {} (error: {:?})", value_variant, e);
    });
    
    if case.is_primitive {
        // For primitive oneof variants, we need to determine the visit method name
        // Based on the type_param
        let visit_method = match case.type_param {
            "bool" => syn::Ident::new("visit_bool", proc_macro2::Span::call_site()),
            "::prost::alloc::string::String" => syn::Ident::new("visit_string", proc_macro2::Span::call_site()),
            "Vec<u8>" => syn::Ident::new("visit_bytes", proc_macro2::Span::call_site()),
            "u32" => syn::Ident::new("visit_u32", proc_macro2::Span::call_site()),
            "u64" => syn::Ident::new("visit_u64", proc_macro2::Span::call_site()),
            "i32" => syn::Ident::new("visit_i32", proc_macro2::Span::call_site()),
            "i64" => syn::Ident::new("visit_i64", proc_macro2::Span::call_site()),
            "f64" => syn::Ident::new("visit_f64", proc_macro2::Span::call_site()),
            "f32" => syn::Ident::new("visit_f32", proc_macro2::Span::call_site()),
            _ => syn::Ident::new("visit_i32", proc_macro2::Span::call_site()), // Default for enums
        };
        
        quote! {
            if let Some(#variant_path(ref value)) = v.#field_name {
                #visitor_param.#visit_method(sizes, value)
            } else {
                sizes
            }
        }
    } else {
        // For message oneof variants, call the visitable method
        let base_type = case.type_param.split("::").last().unwrap_or(case.type_param);
        let base_type_ident = syn::Ident::new(base_type, proc_macro2::Span::call_site());
        let visitable_method = common::visitable_method_name(&base_type_ident);
        quote! {
            if let Some(#variant_path(ref value)) = v.#field_name {
                value.#visitable_method(sizes, &mut #visitor_param)
            } else {
                sizes
            }
        }
    }
}
