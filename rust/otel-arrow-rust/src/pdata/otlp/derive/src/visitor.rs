// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;

use super::TokenVec;
use super::field_info::FieldInfo;
use super::message_info::MessageInfo;
use otlp_model::OneofCase;

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

    let mut visitable_args: TokenVec = Vec::new();        // For oneof fields, generate separate parameters for each variant
        for info in &msg.all_fields {
            if let Some(oneof_cases) = info.oneof.as_ref() {
                for case in oneof_cases {
                    let variant_param_name = syn::Ident::new(
                        &format!("{}_{}", info.ident, case.name),
                        info.ident.span(),
                    );

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
pub fn generate_visitor_call(info: &FieldInfo) -> Option<proc_macro2::TokenStream> {
    // Handle oneof fields
    if let Some(oneof_cases) = info.oneof.as_ref() {
        return generate_oneof_visitor_call(info, oneof_cases);
    }

    let field_name = &info.ident;
    let visitor_param = &info.visitor_param_name;
    let visit_method = &info.visit_method_name;
    let needs_adapter = info.needs_adapter;
    let is_bytes_field = FieldInfo::is_bytes_type(&info.full_type_name); // Vec<u8> specifically

    match (
        info.is_optional,
        info.is_repeated,
        needs_adapter,
        is_bytes_field,
    ) {
        (false, false, true, _) => {
            let adapter_name = info.related_type("MessageAdapter");
            Some(quote! {
                arg = #visitor_param.#visit_method(arg, &(#adapter_name::new(&self.data.#field_name)));
            })
        }
        (false, false, false, _) => {
            if visit_method.to_string() == "visit_string" {
                Some(quote! { arg = #visitor_param.#visit_method(arg, &self.data.#field_name); })
            } else if is_bytes_field {
                // For bytes fields (Vec<u8>), pass as slice
                Some(quote! { arg = #visitor_param.#visit_method(arg, &self.data.#field_name); })
            } else {
                Some(quote! { arg = #visitor_param.#visit_method(arg, *&self.data.#field_name); })
            }
        }
        (true, false, true, _) => {
            let adapter_name = info.related_type("MessageAdapter");
            Some(quote! {
                if let Some(f) = &self.data.#field_name {
                    arg = #visitor_param.#visit_method(arg, &(#adapter_name::new(f)));
                }
            })
        }
        (true, false, false, _) => {
            if visit_method.to_string() == "visit_string" {
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
            let adapter_name = info.related_type("MessageAdapter");
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
            if visit_method.to_string() == "visit_vec" {
                // For repeated primitives using SliceVisitor, pass the entire slice
                Some(quote! { arg = #visitor_param.#visit_method(arg, &self.data.#field_name); })
            } else if visit_method.to_string() == "visit_string" {
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
                let adapter_name = info.related_type("MessageAdapter");
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
            } else if visit_method.to_string() == "visit_vec" {
                // For optional repeated primitives using SliceVisitor, pass the entire slice
                Some(quote! {
                    if let Some(items) = &self.data.#field_name {
                        arg = #visitor_param.#visit_method(arg, items);
                    }
                })
            } else if visit_method.to_string() == "visit_string" {
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

/// Generate visitor call for oneof fields with proper variant matching
pub fn generate_oneof_visitor_call(info: &FieldInfo, oneof_cases: &[OneofCase]) -> Option<proc_macro2::TokenStream> {
    let field_name = &info.ident;
    
    if oneof_cases.is_empty() {
        return None;
    }

    // Generate match arms for each oneof variant
    let match_arms = oneof_cases.iter().map(|case| {
        // Extract just the variant name from the full path like "metric::Data::Sum" -> "Sum"
        let variant_name = case.value_variant.split("::").last().unwrap_or(case.value_variant);
        let variant_ident = syn::Ident::new(variant_name, field_name.span());
        
        let param_name = syn::Ident::new(
            &format!("{}_{}", field_name, case.name),
            field_name.span(),
        );
        
        // Determine the visit method based on the case name and type - map to correct method names
        let visit_method = match case.name {
            "string" => syn::Ident::new("visit_string", field_name.span()),
            "bool" => syn::Ident::new("visit_bool", field_name.span()),
            "int" => syn::Ident::new("visit_i64", field_name.span()),
            "double" => syn::Ident::new("visit_f64", field_name.span()),
            "bytes" => syn::Ident::new("visit_bytes", field_name.span()),
            "kvlist" => syn::Ident::new("visit_kvlist", field_name.span()),
            "array" => syn::Ident::new("visit_array", field_name.span()),
            name => syn::Ident::new(&format!("visit_{}", name), field_name.span()),
        };
        
        // Generate the visitor call based on the type and extra_call
        let visitor_call = if let Some(extra_call) = case.extra_call {
            // For cases with extra_call, use the constructor function
            let constructor = syn::parse_str::<syn::Path>(extra_call)
                .unwrap_or_else(|_| panic!("Invalid extra_call: {}", extra_call));
            quote! {
                let adapter = #constructor(inner);
                arg = #param_name.#visit_method(arg, &adapter);
            }
        } else if is_primitive_type_param(case.type_param) {
            // For primitive types, use the value directly
            quote! {
                arg = #param_name.#visit_method(arg, inner);
            }
        } else {
            // For message types, create an adapter
            let type_ident = syn::Ident::new(case.type_param, field_name.span());
            quote! {
                let adapter = #type_ident::MessageAdapter::new(inner);
                arg = #param_name.#visit_method(arg, &adapter);
            }
        };
        
        quote! {
            Some(#field_name::#variant_ident(ref inner)) => {
                #visitor_call
            }
        }
    });

    let visitor_call = quote! {
        // Handle oneof field with variant matching
        match &self.data.#field_name {
            #(#match_arms)*
            None => {
                // No variant is set, nothing to visit
            }
        }
    };

    Some(visitor_call)
}

/// Check if a type parameter represents a primitive type
fn is_primitive_type_param(type_param: &str) -> bool {
    matches!(type_param, 
        "bool" | "i32" | "i64" | "u32" | "u64" | "f32" | "f64" | 
        "::prost::alloc::string::String" | "Vec<u8>"
    )
}













