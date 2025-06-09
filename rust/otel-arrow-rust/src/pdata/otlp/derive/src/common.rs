// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Common utilities for derive macro code generation to eliminate duplication.
//!
//! This module provides shared functionality used across the derive macro components
//! to apply the DRY (Don't Repeat Yourself) principle and reduce code duplication.

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use super::TokenVec;
use super::common;
use super::field_info::FieldInfo;
use otlp_model::OneofCase;

/// Generate visitor method name from type name (e.g., "LogsData" -> "visit_logs_data")
pub fn visitor_method_name(type_name: &Ident) -> Ident {
    syn::Ident::new(
        &format!("visit_{}", type_name).to_case(Case::Snake),
        type_name.span(),
    )
}

/// Generate visitable method name from type name (e.g., "LogsData" -> "accept_logs_data")
pub fn visitable_method_name(type_name: &Ident) -> Ident {
    syn::Ident::new(
        &format!("accept_{}", type_name).to_case(Case::Snake),
        type_name.span(),
    )
}

/// Generate oneof variant parameter name (e.g., "data_sum" for field "data" and case "sum")
pub fn oneof_variant_field_or_method_name(field_name: &Ident, case_name: &str) -> Ident {
    syn::Ident::new(&format!("{}_{}", field_name, case_name), field_name.span())
}

/// Generate visitor parameters for all fields including oneof variants
pub fn visitor_formal_parameters(fields: &[FieldInfo]) -> TokenVec {
    let mut params = Vec::new();

    for info in fields {
        if let Some(oneof_cases) = info.oneof.as_ref() {
            // Generate separate parameters for each oneof variant
            for case in oneof_cases {
                let variant_param_name =
                    common::oneof_variant_field_or_method_name(&info.ident, &case.name);
                let visitor_type = FieldInfo::generate_visitor_type_for_oneof_case(case);
                params.push(quote! { mut #variant_param_name: impl #visitor_type });
            }
        } else {
            // Regular field parameter
            let visitor_param = &info.visitor_param_name;
            let visitor_trait = &info.visitor_trait;
            params.push(quote! { mut #visitor_param: impl #visitor_trait });
        }
    }

    params
}

/// Process fields into parameter declarations and bounds for generic types
pub fn builder_formal_parameters(
    fields: &[FieldInfo],
    type_params: &[Ident],
) -> (TokenVec, TokenVec) {
    fields
        .iter()
        .enumerate()
        .map(|(idx, info)| {
            let param_name = &info.ident;
            let type_param = &type_params[idx];
            let target_type = &info.full_type_name;

            let decl = quote! { #param_name: #type_param };
            let bound = quote! { #type_param: Into<#target_type> };

            (decl, bound)
        })
        .unzip()
}

/// Generate field arguments for constructor calls
pub fn builder_argument_list(fields: &[FieldInfo]) -> TokenVec {
    fields
        .iter()
        .map(|info| {
            let field_name = &info.ident;
            quote! { #field_name }
        })
        .collect()
}

/// Generate type parameters (T1, T2, ...) for the given number of fields
pub fn generic_type_names(count: usize) -> Vec<Ident> {
    (0..count)
        .map(|idx| crate::create_ident(&format!("T{}", idx + 1)))
        .collect()
}

/// Generate all constructors for a oneof mapping
pub fn builder_oneof_constructors<F>(
    oneof_mapping: &(String, Vec<OneofCase>),
    param_names: &[String],
    param_bounds: &[TokenStream],
    param_decls: &[TokenStream],
    param_args: &[TokenStream],
    all_field_initializers: &[TokenStream],
    type_params: &[Ident],
    create_constructor: F,
) -> Vec<TokenStream>
where
    F: Fn(String, &[TokenStream], &[TokenStream], &[TokenStream], &[TokenStream]) -> TokenStream,
{
    let (oneof_path, oneof_cases) = oneof_mapping;
    let oneof_name = oneof_path.split('.').last().unwrap();
    let oneof_idx = param_names
        .iter()
        .position(|name| name.as_str() == oneof_name)
        .unwrap();

    oneof_cases
        .iter()
        .map(|case| {
            builder_oneof_constructor(
                case,
                oneof_name,
                oneof_idx,
                param_bounds,
                param_decls,
                param_args,
                all_field_initializers,
                type_params,
                &create_constructor,
            )
        })
        .collect()
}

/// Generate constructor for a single oneof case with shared logic
pub fn builder_oneof_constructor<F>(
    case: &OneofCase,
    oneof_name: &str,
    oneof_idx: usize,
    param_bounds: &[TokenStream],
    param_decls: &[TokenStream],
    param_args: &[TokenStream],
    all_field_initializers: &[TokenStream],
    type_params: &[Ident],
    create_constructor: F,
) -> TokenStream
where
    F: Fn(String, &[TokenStream], &[TokenStream], &[TokenStream], &[TokenStream]) -> TokenStream,
{
    let case_type = syn::parse_str::<syn::Type>(&case.type_param).unwrap();
    let variant_path = syn::parse_str::<syn::Expr>(&case.value_variant).unwrap();
    let suffix = format!("_{}", case.name);
    let oneof_ident = syn::Ident::new(oneof_name, proc_macro2::Span::call_site());

    // Duplicate the param bounds and field initializers
    let mut cur_param_bounds = param_bounds.to_vec();
    let mut cur_field_initializers = all_field_initializers.to_vec();
    let type_param = &type_params[oneof_idx];

    let value_bound = quote! { #type_param: Into<#case_type> };
    let value_initializer = if let Some(extra_call) = &case.extra_call {
        let extra_call_path = syn::parse_str::<syn::Expr>(extra_call).unwrap();
        quote! {
            #oneof_ident: Some(#variant_path(#extra_call_path(#oneof_ident.into()))),
        }
    } else {
        quote! {
            #oneof_ident: Some(#variant_path(#oneof_ident.into())),
        }
    };

    // Replace the parameter with oneof-specific expansion
    cur_param_bounds[oneof_idx] = value_bound;
    cur_field_initializers[oneof_idx] = value_initializer;

    create_constructor(
        suffix,
        &cur_param_bounds,
        param_decls,
        param_args,
        &cur_field_initializers,
    )
}

/// Check if a type parameter represents a primitive type
pub fn is_primitive_type_param(type_param: &str) -> bool {
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

/// Generate visitor call for oneof fields with common match arm generation
pub fn visitor_oneof_call(info: &FieldInfo, oneof_cases: &[OneofCase]) -> Option<TokenStream> {
    if oneof_cases.is_empty() {
        return None;
    }

    let field_name = &info.ident;

    // Generate match arms for each oneof variant
    let match_arms = oneof_cases.iter().map(|case| {
        let variant_path =
            syn::parse_str::<syn::Path>(&case.value_variant).expect("Failed to parse variant path");

        let param_name = common::oneof_variant_field_or_method_name(&info.ident, &case.name);

        // Determine the visit method based on the case name - map to correct method names
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
        let visitor_call = if let Some(extra_call) = &case.extra_call {
            // Handle message adapter transformation (Xyz::new -> XyzMessageAdapter::new)
            if extra_call.contains("::new") {
                let adapter_type = extra_call.replace("::new", "MessageAdapter::new");
                let adapter_tokens = syn::parse_str::<TokenStream>(&adapter_type).unwrap();
                quote! { arg = #param_name.#visit_method(arg, &#adapter_tokens(inner)); }
            } else {
                let extra_call_tokens = syn::parse_str::<TokenStream>(extra_call).unwrap();
                quote! { arg = #param_name.#visit_method(arg, #extra_call_tokens(inner)); }
            }
        } else if is_primitive_type_param(&case.type_param) {
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
            Some(#variant_path(inner)) => {
                #visitor_call
            }
        }
    });

    Some(quote! {
        match &self.data.#field_name {
            #(#match_arms)*
            None => {}
        }
    })
}

/// Generate default initializer for a field
pub fn builder_default_initializer(info: &FieldInfo) -> TokenStream {
    let field_name = &info.ident;

    if info.is_optional {
        quote! { #field_name: None, }
    } else {
        match info.base_type_name.as_str() {
            "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" => {
                quote! { #field_name: 0, }
            }
            "f32" | "f64" => quote! { #field_name: 0.0, },
            "bool" => quote! { #field_name: false, },
            _ => quote! { #field_name: ::core::default::Default::default(), },
        }
    }
}

/// Generate field assignment patterns for different field types
pub fn builder_field_assignment(info: &FieldInfo) -> (TokenStream, TokenStream) {
    let field_name = &info.ident;

    // For enum fields, use direct casting instead of .into()
    if info.proto_type == "enumeration" {
        match (info.is_optional, &info.as_type) {
            (true, Some(as_type)) => (
                quote! { self.inner.#field_name = Some(#field_name as #as_type); },
                quote! { #field_name: Some(#field_name as #as_type), },
            ),
            (true, None) => (
                quote! { self.inner.#field_name = Some(#field_name.into()); },
                quote! { #field_name: Some(#field_name.into()), },
            ),
            (false, Some(as_type)) => (
                quote! { self.inner.#field_name = #field_name as #as_type; },
                quote! { #field_name: #field_name as #as_type, },
            ),
            (false, None) => (
                quote! { self.inner.#field_name = #field_name.into(); },
                quote! { #field_name: #field_name.into(), },
            ),
        }
    } else {
        // For non-enum fields, use the original logic with .into()
        match (info.is_optional, &info.as_type) {
            (true, Some(as_type)) => (
                quote! { self.inner.#field_name = Some(#field_name.into() as #as_type); },
                quote! { #field_name: Some(#field_name.into() as #as_type), },
            ),
            (true, None) => (
                quote! { self.inner.#field_name = Some(#field_name.into()); },
                quote! { #field_name: Some(#field_name.into()), },
            ),
            (false, Some(as_type)) => (
                quote! { self.inner.#field_name = #field_name.into() as #as_type; },
                quote! { #field_name: #field_name.into() as #as_type, },
            ),
            (false, None) => (
                quote! { self.inner.#field_name = #field_name.into(); },
                quote! { #field_name: #field_name.into(), },
            ),
        }
    }
}
