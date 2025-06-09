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

    let mut visitable_args: TokenVec = Vec::new(); // For oneof fields, generate separate parameters for each variant
    for info in &msg.all_fields {
        if let Some(oneof_cases) = info.oneof.as_ref() {
            for case in oneof_cases {
                let variant_param_name =
                    syn::Ident::new(&format!("{}_{}", info.ident, case.name), info.ident.span());

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
/// This function has been redesigned based on analysis showing only 7 specific cases
/// actually occur in practice (B, C, D, E, H, I, K from the original design).
pub fn generate_visitor_call(info: &FieldInfo) -> Option<proc_macro2::TokenStream> {
    // Handle oneof fields separately
    if let Some(oneof_cases) = info.oneof.as_ref() {
        return generate_oneof_visitor_call(info, oneof_cases);
    }

    let field_name = &info.ident;
    let visitor_param = &info.visitor_param_name;
    let visit_method = &info.visit_method_name;

    if info.is_message {
        let adapter_name = info.related_type("MessageAdapter");

        if info.is_optional {
            Some(quote! {
                if let Some(f) = &self.data.#field_name {
                    arg = #visitor_param.#visit_method(arg, &(#adapter_name::new(f)));
                }
            })
        } else if info.is_repeated {
            Some(quote! {
                for item in &self.data.#field_name {
                    arg = #visitor_param.#visit_method(arg, &(#adapter_name::new(item)));
                }
            })
        } else {
            Some(quote! {
                arg = #visitor_param.#visit_method(arg, &(#adapter_name::new(&self.data.#field_name)));
            })
        }
    } else if info.is_repeated {
        if info.is_primitive {
            Some(quote! { arg = #visitor_param.#visit_method(arg, &self.data.#field_name); })
        } else {
            Some(quote! {
                for item in &self.data.#field_name {
                    arg = #visitor_param.#visit_method(arg, item);
                }
            })
        }
    } else if info.is_optional {
        if visit_method.to_string() == "visit_string" || visit_method.to_string() == "visit_bytes" {
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
    } else {
        if visit_method.to_string() == "visit_string" || visit_method.to_string() == "visit_bytes" {
            Some(quote! { arg = #visitor_param.#visit_method(arg, &self.data.#field_name); })
        } else {
            Some(quote! { arg = #visitor_param.#visit_method(arg, *&self.data.#field_name); })
        }
    }
}

/// Generate visitor call for oneof fields with proper variant matching
pub fn generate_oneof_visitor_call(
    info: &FieldInfo,
    oneof_cases: &[OneofCase],
) -> Option<proc_macro2::TokenStream> {
    let field_name = &info.ident;

    if oneof_cases.is_empty() {
        return None;
    }

    // Generate match arms for each oneof variant
    let match_arms = oneof_cases.iter().map(|case| {
        // Use the full value_variant path for the match pattern
        let variant_path = syn::parse_str::<syn::Path>(case.value_variant)
            .unwrap_or_else(|_| panic!("Invalid variant path: {}", case.value_variant));

        let param_name =
            syn::Ident::new(&format!("{}_{}", field_name, case.name), field_name.span());

        // Determine the visit method based on the case name and type - map to correct method names
        let visit_method = match case.name {
            "string" => syn::Ident::new("visit_string", field_name.span()),
            "bool" => syn::Ident::new("visit_bool", field_name.span()),
            "int" => syn::Ident::new("visit_i64", field_name.span()),
            "double" => syn::Ident::new("visit_f64", field_name.span()),
            "bytes" => syn::Ident::new("visit_bytes", field_name.span()),
            "kvlist" => syn::Ident::new("visit_key_value_list", field_name.span()),
            "array" => syn::Ident::new("visit_array_value", field_name.span()),
            name => syn::Ident::new(&format!("visit_{}", name), field_name.span()),
        };

        // Generate the visitor call based on the type and extra_call
        let visitor_call = if let Some(extra_call) = case.extra_call {
            // Transform Xyz::new to XyzMessageAdapter::new for visitor generation
            let adapter_constructor = if extra_call.ends_with("::new") {
                let base_type = &extra_call[..extra_call.len() - 5]; // Remove "::new"
                format!("{}MessageAdapter::new", base_type)
            } else {
                panic!("Unsupported extra_call format for visitor: {}", extra_call)
            };

            let constructor = syn::parse_str::<syn::Path>(&adapter_constructor)
                .unwrap_or_else(|_| panic!("Invalid adapter constructor: {}", adapter_constructor));
            quote! {
                let adapter = #constructor(inner);
                arg = #param_name.#visit_method(arg, &adapter);
            }
        } else if is_primitive_type_param(case.type_param) {
            // For primitive types, handle references correctly
            let value_arg = match case.type_param {
                "::prost::alloc::string::String" => quote! { inner.as_str() },
                "Vec<u8>" => quote! { inner.as_slice() },
                _ => quote! { *inner }, // For basic types like i64, f64, bool
            };
            quote! {
                arg = #param_name.#visit_method(arg, #value_arg);
            }
        } else {
            // For message types, create an adapter using TypeNameMessageAdapter::new pattern
            let adapter_name = format!("{}MessageAdapter", case.type_param);
            let adapter_constructor = syn::parse_str::<syn::Path>(&adapter_name)
                .unwrap_or_else(|_| panic!("Invalid adapter constructor: {}", adapter_name));
            quote! {
                let adapter = #adapter_constructor::new(inner);
                arg = #param_name.#visit_method(arg, &adapter);
            }
        };

        quote! {
            Some(#variant_path(ref inner)) => {
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
    matches!(
        type_param,
        "bool"
            | "i32"
            | "i64"
            | "u32"
            | "u64"
            | "f32"
            | "f64"
            | "::prost::alloc::string::String"
            | "Vec<u8>"
    )
}
