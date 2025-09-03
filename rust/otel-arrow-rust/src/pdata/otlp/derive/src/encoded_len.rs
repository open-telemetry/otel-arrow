// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! EncodedLen visitor generation for protobuf size calculation.
//!
//! This module implements EncodedLen visitors that implement the
//! visitor pattern for `PrecomputedSizes` arguments. These visitors
//! calculate the exact encoded size of protobuf (sub-)messages.

use proc_macro::TokenStream;
use quote::quote;

use super::common;
use super::field_info::FieldInfo;
use super::message_info::MessageInfo;

/// Generate the EncodedLen visitor implementation for a message type.
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
        pub struct #encoded_len_name<const TAG: u32, const OPTION: bool> {}

        impl<const TAG: u32, const OPTION: bool> #encoded_len_name<TAG, OPTION> {
            /// Create a new EncodedLen visitor.
            pub fn new() -> Self {
                Self { }
            }

            /// Calculate the sum of direct children's encoded lengths.
            fn visit_children(
                &mut self,
                mut arg: crate::pdata::otlp::PrecomputedSizes,
                mut v: impl #visitable_name<crate::pdata::otlp::PrecomputedSizes>
            ) -> (crate::pdata::otlp::PrecomputedSizes, usize) {
                let mut total = 0;

                #visitor_body

                (arg, total)
            }
        }

        impl<const TAG: u32, const OPTION: bool> #visitor_name<crate::pdata::otlp::PrecomputedSizes> for #encoded_len_name<TAG, OPTION> {
            fn #visitor_method_name(
                &mut self,
                mut arg: crate::pdata::otlp::PrecomputedSizes,
                mut v: impl #visitable_name<crate::pdata::otlp::PrecomputedSizes>
            ) -> crate::pdata::otlp::PrecomputedSizes {
                let idx = arg.position();
                arg.reserve();

                let (mut arg, total_child_size) = self.visit_children(arg, v);

                arg.set_size(idx, crate::pdata::otlp::encoders::conditional_length_delimited_size::<TAG, OPTION>(total_child_size));
                arg
            }
        }

        impl #outer_name {
            /// Calculate the encoded size using the existing visitor pattern, but only for
        /// test because this throws away the precomputed sizes.
            #[cfg(test)]
            pub fn pdata_size(&self) -> usize {
        let (_, total) = self.precompute_sizes(crate::pdata::otlp::PrecomputedSizes::default());
        total
            }

            /// Calculate the precomputed sizing using an input to allow re-use.
            pub fn precompute_sizes(&self, mut input: crate::pdata::otlp::PrecomputedSizes) -> (crate::pdata::otlp::PrecomputedSizes, usize) {
        input.clear();
        // Note that the <TAG, OPTION> passed here are irrelevant because
        // top-level tags are not encoded. TODO: Reduce confusion on this point.
                let mut visitor = #encoded_len_name::<0, false> {};
                visitor.visit_children(input, self)
            }
        }
    };

    // Combine the main EncodedLen implementation with the Accumulate visitor implementation
    let mut result = TokenStream::from(expanded);
    result.extend(derive_accumulate_visitor(msg));
    result
}

/// Generate the helper method body that calculates encoded sizes of direct children.
fn generate_helper_method_body(msg: &MessageInfo) -> proc_macro2::TokenStream {
    let mut accumulate_instantiations = Vec::new();
    let mut accumulate_args = Vec::new();

    // Generate Accumulate wrapper instances for each field
    for info in &msg.all_fields {
        if let Some(oneof_cases) = info.oneof.as_ref() {
            // Process each oneof variant individually
            for case in oneof_cases {
                let variant_param_name =
                    common::oneof_variant_field_or_method_name(&info.ident, &case.name);

                let visitor_instantiation = if case.is_primitive {
                    generate_primitive_visitor_instantiation_oneof(case, &case.tag)
                } else {
                    generate_message_visitor_instantiation_oneof(case)
                };

                accumulate_instantiations.push(quote! {
                    let mut #variant_param_name = crate::pdata::otlp::Accumulate::new(
                        #visitor_instantiation,
                    );
                });
                accumulate_args.push(quote! { #variant_param_name });
            }
        } else {
            // Process regular field
            let field_name = &info.ident;

            let visitor_instantiation = if info.is_primitive {
                generate_primitive_visitor_for_field(info)
            } else {
                generate_message_visitor_for_field(info)
            };

            accumulate_instantiations.push(quote! {
                let mut #field_name = crate::pdata::otlp::Accumulate::new(
                    #visitor_instantiation,
                );
            });
            accumulate_args.push(quote! { #field_name });
        }
    }

    // Generate the main visitable method call
    let outer_name = &msg.outer_name;
    let visitable_method_name = common::visitable_method_name(&outer_name);

    quote! {
        #(#accumulate_instantiations)*

        arg = v.#visitable_method_name(arg, #(&mut #accumulate_args),*);

        #(total += #accumulate_args.total;)*
    }
}

/// Generate primitive visitor instantiation for a field.
fn generate_primitive_visitor_for_field(info: &FieldInfo) -> proc_macro2::TokenStream {
    let tag = &info.tag;

    // For repeated primitive fields, use slice visitors
    if info.is_repeated {
        let visitor_type = match info.base_type_name.as_str() {
            "bool" => quote! { crate::pdata::otlp::SliceBooleanEncodedLen },
            "String" | "string" => quote! { crate::pdata::otlp::SliceStringEncodedLen },
            "u32" => {
                if info.is_fixed {
                    quote! { crate::pdata::otlp::SliceU32FixedEncodedLen }
                } else {
                    quote! { crate::pdata::otlp::SliceU32VarintEncodedLen }
                }
            }
            "u64" => {
                if info.is_fixed {
                    quote! { crate::pdata::otlp::SliceU64FixedEncodedLen }
                } else {
                    quote! { crate::pdata::otlp::SliceU64VarintEncodedLen }
                }
            }
            "i32" => {
                if info.is_fixed {
                    quote! { crate::pdata::otlp::SliceI32FixedEncodedLen }
                } else {
                    quote! { crate::pdata::otlp::SliceI32VarintEncodedLen }
                }
            }
            "i64" => {
                if info.is_fixed {
                    quote! { crate::pdata::otlp::SliceI64FixedEncodedLen }
                } else {
                    quote! { crate::pdata::otlp::SliceI64VarintEncodedLen }
                }
            }
            "f64" => quote! { crate::pdata::otlp::SliceDoubleEncodedLen },
            _ => {
                if !info.proto_type.contains("bytes") {
                    panic!("unimplemented");
                }
                quote! { crate::pdata::otlp::SliceBytesEncodedLen }
            }
        };
        quote! { #visitor_type::<#tag> {} }
    } else if info.proto_type.contains("bytes") {
        // Handle bytes fields specially - they use BytesEncodedLen regardless of base_type_name
        if !info.is_optional {
            quote! { crate::pdata::otlp::BytesEncodedLen::<#tag, true> {} }
        } else {
            quote! { crate::pdata::otlp::BytesEncodedLen::<#tag, false> {} }
        }
    } else {
        // For non-repeated primitive fields, use individual visitors
        let visitor_type = match info.base_type_name.as_str() {
            "bool" => quote! { crate::pdata::otlp::BooleanEncodedLen },
            "String" | "string" => quote! { crate::pdata::otlp::StringEncodedLen },
            "f64" => quote! { crate::pdata::otlp::DoubleEncodedLen },
            "u32" => {
                // Choose between varint and fixed32 encoding based on proto_type
                if info.is_fixed {
                    quote! { crate::pdata::otlp::U32FixedEncodedLen }
                } else {
                    quote! { crate::pdata::otlp::U32VarintEncodedLen }
                }
            }
            "u64" => {
                // Choose between varint and fixed64 encoding based on proto_type
                if info.is_fixed {
                    quote! { crate::pdata::otlp::U64FixedEncodedLen }
                } else {
                    quote! { crate::pdata::otlp::U64VarintEncodedLen }
                }
            }
            "i32" => {
                // Choose between varint and sint32 encoding based on proto_type
                if info.is_fixed {
                    quote! { crate::pdata::otlp::I32FixedEncodedLen }
                } else {
                    quote! { crate::pdata::otlp::I32VarintEncodedLen }
                }
            }
            "i64" => {
                // Choose between varint and sfixed64 encoding based on proto_type
                if info.is_fixed {
                    quote! { crate::pdata::otlp::I64FixedEncodedLen }
                } else {
                    quote! { crate::pdata::otlp::I64VarintEncodedLen }
                }
            }
            _ => panic!("unimplemented"),
        };
        if !info.is_optional {
            quote! { #visitor_type::<#tag, true> {} }
        } else {
            quote! { #visitor_type::<#tag, false> {} }
        }
    }
}

/// Generate message visitor instantiation for a field.
fn generate_message_visitor_for_field(info: &FieldInfo) -> proc_macro2::TokenStream {
    let encoded_len_type = info.related_type("EncodedLen");
    let tag = &info.tag;
    if !info.is_optional {
        quote! { #encoded_len_type::<#tag, true> {} }
    } else {
        quote! { #encoded_len_type::<#tag, false> {} }
    }
}

/// Generate primitive visitor instantiation for an oneof case.
fn generate_primitive_visitor_instantiation_oneof(
    case: &otlp_model::OneofCase,
    tag: &u32,
) -> proc_macro2::TokenStream {
    let visitor_type = match case.type_param {
        "bool" => quote! { crate::pdata::otlp::BooleanEncodedLen },
        "::prost::alloc::string::String" => quote! { crate::pdata::otlp::StringEncodedLen },
        "Vec<u8>" => quote! { crate::pdata::otlp::BytesEncodedLen },
        "f64" => quote! { crate::pdata::otlp::DoubleEncodedLen },
        "u32" => {
            if case.proto_type.contains("fixed32") {
                quote! { crate::pdata::otlp::U32FixedEncodedLen }
            } else {
                quote! { crate::pdata::otlp::U32VarintEncodedLen }
            }
        }
        "u64" => {
            if case.proto_type.contains("fixed64") {
                quote! { crate::pdata::otlp::U64FixedEncodedLen }
            } else {
                quote! { crate::pdata::otlp::U64VarintEncodedLen }
            }
        }
        "i32" => {
            if case.proto_type.contains("sfixed32") {
                quote! { crate::pdata::otlp::I32FixedEncodedLen }
            } else {
                quote! { crate::pdata::otlp::I32VarintEncodedLen }
            }
        }
        "i64" => {
            if case.proto_type.contains("sfixed64") {
                quote! { crate::pdata::otlp::I64FixedEncodedLen }
            } else {
                quote! { crate::pdata::otlp::I64VarintEncodedLen }
            }
        }
        _ => panic!("unimplemented"),
    };

    // Oneofs are not optional
    quote! { #visitor_type::<#tag, false> {} }
}

/// Generate message visitor instantiation for an oneof case.
fn generate_message_visitor_instantiation_oneof(
    case: &otlp_model::OneofCase,
) -> proc_macro2::TokenStream {
    // Regular message type handling
    let type_name = case.type_param;
    let tag = &case.tag;
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

    quote! { #encoded_len_type::<#tag, false> {} }
}

/// Generate Accumulate visitor implementations for a message type.
/// This creates implementations that wrap a visitor and accumulate only the direct child sizes.
pub fn derive_accumulate_visitor(msg: &MessageInfo) -> TokenStream {
    let visitor_name = msg.related_typename("Visitor");
    let visitable_name = msg.related_typename("Visitable");
    let visitor_method_name = common::visitor_method_name(&msg.outer_name);

    let expanded = quote! {
        impl<V: #visitor_name<crate::pdata::otlp::PrecomputedSizes>>
            #visitor_name<crate::pdata::otlp::PrecomputedSizes> for &mut crate::pdata::otlp::Accumulate<V>
        {
            fn #visitor_method_name(
                &mut self,
                mut arg: crate::pdata::otlp::PrecomputedSizes,
                v: impl #visitable_name<crate::pdata::otlp::PrecomputedSizes>,
            ) -> crate::pdata::otlp::PrecomputedSizes {
                let idx = arg.position();
                arg = self.inner.#visitor_method_name(arg, v);
                self.total += arg.get_size(idx);
                arg
            }
        }
    };

    TokenStream::from(expanded)
}
