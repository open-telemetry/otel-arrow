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
    let message_adapter_name = msg.related_typename("MessageAdapter");
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

        impl #outer_name {
            /// Calculate the encoded size using the OTLP visitor pattern.
            /// This method is generated for testing purposes to compare against
            /// the prost-generated encoded_len method.
            #[cfg(test)]
            pub fn pdata_size(&self) -> usize {
                let mut sizes = crate::pdata::otlp::PrecomputedSizes::default();
                let (_, size) = #encoded_len_name::children_encoded_size(
                    sizes,
                    &#message_adapter_name::new(self),
                );
                size
            }
        }
    };

    TokenStream::from(expanded)
}

/// Generate the helper method body that calculates encoded sizes of direct children.
///
/// This generates code to be used in the children_encoded_size method on the visitor.
/// It creates visitor instances for each field and calls the main visitable method once.
fn generate_helper_method_body(msg: &MessageInfo) -> proc_macro2::TokenStream {
    let mut visitor_instantiations = Vec::new();
    let mut visitor_args = Vec::new();

    // Generate visitor instances for each field
    for info in &msg.all_fields {
        if let Some(oneof_cases) = info.oneof.as_ref() {
            // Process each oneof variant individually
            for case in oneof_cases {
                let variant_param_name =
                    common::oneof_variant_field_or_method_name(&info.ident, &case.name);

                let visitor_instantiation = if case.is_primitive {
                    generate_primitive_visitor_instantiation(case, &info.tag)
                } else {
                    generate_message_visitor_instantiation(case, &info.tag)
                };

                visitor_instantiations.push(quote! {
                    let #variant_param_name = #visitor_instantiation;
                });
                visitor_args.push(quote! { #variant_param_name });
            }
        } else {
            // Process regular field
            let field_name = &info.ident;

            let visitor_instantiation = if info.is_primitive {
                generate_primitive_visitor_for_field(info)
            } else {
                generate_message_visitor_for_field(info)
            };

            visitor_instantiations.push(quote! {
                let #field_name = #visitor_instantiation;
            });
            visitor_args.push(quote! { #field_name });
        }
    }

    // Generate the main visitable method call
    let outer_name = &msg.outer_name;
    let visitable_method_name = common::visitable_method_name(&outer_name);

    quote! {
        // Create visitor instances for each field
        #(#visitor_instantiations)*

        // Call the main visitable method with all visitors
        // Note: We don't call reserve() here because:
        // 1. Child visitors reserve space for themselves
        // 2. The caller of children_encoded_size reserves for itself
        let start_len = arg.len();
        arg = v.#visitable_method_name(arg, #(#visitor_args),*);
        
        // Sum up all the sizes that were added by child visitors
        for i in start_len..arg.len() {
            total += arg.get_size(i);
        }
    }
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
