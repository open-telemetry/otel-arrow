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
    // Handle raw identifiers by removing the "r#" prefix
    let name_str = type_name.to_string();
    let clean_name = if name_str.starts_with("r#") {
        &name_str[2..] // Remove "r#" prefix
    } else {
        &name_str
    };

    syn::Ident::new(
        &format!("accept_{}", clean_name).to_case(Case::Snake),
        type_name.span(),
    )
}

/// Generate oneof variant parameter name (e.g., "data_sum" for field "data" and case "sum")
pub fn oneof_variant_field_or_method_name(field_name: &Ident, case_name: &str) -> Ident {
    syn::Ident::new(&format!("{}_{}", field_name, case_name), field_name.span())
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
    let value_initializer = quote! {
        #oneof_ident: Some(#variant_path(#oneof_ident.into())),
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

/// Configuration for field assignment generation
#[derive(Debug)]
struct FieldAssignmentConfig {
    is_enum: bool,
    is_optional: bool,
    has_as_type: bool,
}

impl FieldAssignmentConfig {
    fn from_field_info(info: &FieldInfo) -> Self {
        Self {
            is_enum: info.proto_type == "enumeration",
            is_optional: info.is_optional,
            has_as_type: info.as_type.is_some(),
        }
    }
}

/// Generate the appropriate conversion expression based on field type
fn generate_conversion_expression(
    field_name: &Ident,
    config: &FieldAssignmentConfig,
    as_type: Option<&syn::Type>,
) -> TokenStream {
    match (config.is_enum, config.has_as_type) {
        (true, true) => {
            let as_type = as_type.unwrap();
            quote! { #field_name as #as_type }
        }
        (true, false) => quote! { #field_name.into() },
        (false, true) => {
            let as_type = as_type.unwrap();
            quote! { #field_name.into() as #as_type }
        }
        (false, false) => quote! { #field_name.into() },
    }
}

/// Generate field assignment patterns for different field types
pub fn builder_field_assignment(info: &FieldInfo) -> (TokenStream, TokenStream) {
    let field_name = &info.ident;
    let config = FieldAssignmentConfig::from_field_info(info);

    let conversion = generate_conversion_expression(field_name, &config, info.as_type.as_ref());

    match config.is_optional {
        true => (
            quote! { self.inner.#field_name = Some(#conversion); },
            quote! { #field_name: Some(#conversion), },
        ),
        false => (
            quote! { self.inner.#field_name = #conversion; },
            quote! { #field_name: #conversion, },
        ),
    }
}
