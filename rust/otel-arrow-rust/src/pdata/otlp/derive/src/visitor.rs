// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use quote::quote;

use super::common;
use super::field_info::FieldInfo;
use super::message_info::MessageInfo;
use super::TokenVec;

/// Emits the visitor, visitable and adapters methods.
pub fn derive(msg: &MessageInfo) -> TokenStream {
    let outer_name = &msg.outer_name;
    let visitor_name = msg.related_typename("Visitor");
    let visitable_name = msg.related_typename("Visitable");
    let visitor_method_name = common::visitor_method_name(&outer_name);
    let visitable_method_name = common::visitable_method_name(&outer_name);

    let mut visitable_args: TokenVec = Vec::new();
    
    for info in &msg.all_fields {
        if let Some(oneof_cases) = info.oneof.as_ref() {
            // For oneof fields, generate separate parameters for each variant
            for case in oneof_cases {
                let variant_param_name =
                    common::oneof_variant_field_or_method_name(&info.ident, &case.name);

                // Generate visitor trait directly without using FieldInfo methods
                let visitor_type = if case.is_primitive {
                    match case.type_param {
                        "bool" => quote! { crate::pdata::BooleanVisitor },
                        "::prost::alloc::string::String" => quote! { crate::pdata::StringVisitor },
                        "Vec<u8>" => quote! { crate::pdata::BytesVisitor },
                        "u32" => quote! { crate::pdata::U32Visitor },
                        "u64" => quote! { crate::pdata::U64Visitor },
                        "i32" => quote! { crate::pdata::I32Visitor },
                        "i64" => quote! { crate::pdata::I64Visitor },
                        "f64" => quote! { crate::pdata::F64Visitor },
                        "f32" => quote! { crate::pdata::F32Visitor },
                        _ => quote! { crate::pdata::I32Visitor }, // Default for enums
                    }
                } else {
                    // For message oneof variants, use type name + Visitor
                    let type_name = case.type_param.split("::").last().unwrap_or(case.type_param);
                    let trait_name = format!("{}Visitor", type_name);
                    let trait_ident = syn::Ident::new(&trait_name, proc_macro2::Span::call_site());
                    quote! { #trait_ident }
                };

                visitable_args
                    .push(quote! { #variant_param_name: impl #visitor_type<Argument> });
            }
            continue;
        }

        // For non-oneof fields, generate visitor trait directly
        let param_name = &info.ident;
        
        let visitor_type = if info.is_primitive {
            // Handle bytes specially
            if info.proto_type.contains("bytes") {
                if info.is_repeated {
                    quote! { crate::pdata::SliceBytesVisitor }
                } else {
                    quote! { crate::pdata::BytesVisitor }
                }
            } else if info.is_repeated {
                // For repeated primitives
                match info.base_type_name.as_str() {
                    "bool" => quote! { crate::pdata::SliceBooleanVisitor },
                    "String" | "string" => quote! { crate::pdata::SliceStringVisitor },
                    "u32" => quote! { crate::pdata::SliceU32Visitor },
                    "u64" => quote! { crate::pdata::SliceU64Visitor },
                    "i32" => quote! { crate::pdata::SliceI32Visitor },
                    "i64" => quote! { crate::pdata::SliceI64Visitor },
                    "f64" => quote! { crate::pdata::SliceDoubleVisitor },
                    "f32" => quote! { crate::pdata::SliceFixed32Visitor },
                    _ => quote! { crate::pdata::SliceI32Visitor },
                }
            } else {
                // For non-repeated primitives  
                match info.base_type_name.as_str() {
                    "bool" => quote! { crate::pdata::BooleanVisitor },
                    "String" | "string" => quote! { crate::pdata::StringVisitor },
                    "u32" => quote! { crate::pdata::U32Visitor },
                    "u64" => quote! { crate::pdata::U64Visitor },
                    "i32" => quote! { crate::pdata::I32Visitor },
                    "i64" => quote! { crate::pdata::I64Visitor },
                    "f64" => quote! { crate::pdata::F64Visitor },
                    "f32" => quote! { crate::pdata::F32Visitor },
                    _ => quote! { crate::pdata::I32Visitor },
                }
            }
        } else {
            // For message fields, use base type name + Visitor
            let trait_name = format!("{}Visitor", info.base_type_name);
            let trait_ident = syn::Ident::new(&trait_name, proc_macro2::Span::call_site());
            
            if let Some(qualifier) = &info.qualifier {
                quote! { #qualifier::#trait_ident }
            } else {
                quote! { #trait_ident }
            }
        };

        visitable_args.push(quote! { #param_name: impl #visitor_type<Argument> });
    }

    // Generate the visitable implementation body
    let visitable_impl_body = generate_visitable_implementation_body(msg);

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

        impl<Argument> #visitable_name<Argument> for #outer_name {
            fn #visitable_method_name(&self, mut arg: Argument, #(#visitable_args),*) -> Argument {
                #visitable_impl_body
            }
        }
    };

    TokenStream::from(expanded)
}

/// Generate the implementation body for the visitable trait.
/// This processes each field by calling the appropriate visitor method.
fn generate_visitable_implementation_body(msg: &MessageInfo) -> proc_macro2::TokenStream {
    let mut field_calls = Vec::new();

    for info in &msg.all_fields {
        if let Some(oneof_cases) = info.oneof.as_ref() {
            // For oneof fields, we need to handle each variant
            for case in oneof_cases {
                let field_name = &info.ident;
                let variant_param_name =
                    common::oneof_variant_field_or_method_name(&info.ident, &case.name);
                let value_variant = case.value_variant;

                // Parse the value_variant to get the enum type and variant name
                let variant_path = syn::parse_str::<syn::Path>(value_variant).unwrap_or_else(|e| {
                    panic!(
                        "Failed to parse variant path: {} (error: {:?})",
                        value_variant, e
                    );
                });

                if case.is_primitive {
                    // For primitive oneof variants, call the visitor method directly
                    let visit_method = match case.type_param {
                        "bool" => syn::Ident::new("visit_bool", proc_macro2::Span::call_site()),
                        "::prost::alloc::string::String" => {
                            syn::Ident::new("visit_string", proc_macro2::Span::call_site())
                        }
                        "Vec<u8>" => syn::Ident::new("visit_bytes", proc_macro2::Span::call_site()),
                        "u32" => syn::Ident::new("visit_u32", proc_macro2::Span::call_site()),
                        "u64" => syn::Ident::new("visit_u64", proc_macro2::Span::call_site()),
                        "i32" => syn::Ident::new("visit_i32", proc_macro2::Span::call_site()),
                        "i64" => syn::Ident::new("visit_i64", proc_macro2::Span::call_site()),
                        "f64" => syn::Ident::new("visit_f64", proc_macro2::Span::call_site()),
                        "f32" => syn::Ident::new("visit_f32", proc_macro2::Span::call_site()),
                        _ => syn::Ident::new("visit_i32", proc_macro2::Span::call_site()),
                    };

                    field_calls.push(quote! {
                        if let Some(#variant_path(ref value)) = self.#field_name {
                            arg = #variant_param_name.#visit_method(arg, value);
                        }
                    });
                } else {
                    // For message oneof variants, call the visitor method
                    let visitor_method = match case
                        .type_param
                        .split("::")
                        .last()
                        .unwrap_or(case.type_param)
                    {
                        t => {
                            let base_type_ident =
                                syn::Ident::new(t, proc_macro2::Span::call_site());
                            common::visitor_method_name(&base_type_ident)
                        }
                    };

                    field_calls.push(quote! {
                        if let Some(#variant_path(ref value)) = self.#field_name {
                            arg = #variant_param_name.#visitor_method(arg, value);
                        }
                    });
                }
            }
        } else {
            // For regular fields
            let field_name = &info.ident;
            let visitor_param = &info.ident; // Use field name as parameter name, not visitor_param_name

            if info.is_primitive {
                let visit_method = &info.visit_method_name;

                if info.is_repeated {
                    field_calls.push(quote! {
                        arg = #visitor_param.visit_slice(arg, &self.#field_name);
                    });
                } else if info.is_optional {
                    field_calls.push(quote! {
                        if let Some(ref value) = self.#field_name {
                            arg = #visitor_param.#visit_method(arg, value);
                        }
                    });
                } else {
                    field_calls.push(quote! {
                        arg = #visitor_param.#visit_method(arg, &self.#field_name);
                    });
                }
            } else {
                // For message fields, call the visitor method
                let visitor_method = &info.visit_method_name;

                if info.is_repeated {
                    field_calls.push(quote! {
                        for item in &self.#field_name {
                            arg = #visitor_param.#visitor_method(arg, item);
                        }
                    });
                } else if info.is_optional {
                    field_calls.push(quote! {
                        if let Some(ref value) = self.#field_name {
                            arg = #visitor_param.#visitor_method(arg, value);
                        }
                    });
                } else {
                    field_calls.push(quote! {
                        arg = #visitor_param.#visitor_method(arg, &self.#field_name);
                    });
                }
            }
        }
    }

    quote! {
        #(#field_calls)*
        arg
    }
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
