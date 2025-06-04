// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use std::collections::HashMap;
use syn::{DeriveInput, parse_macro_input};

#[derive(Clone, Debug)]
struct FieldInfo {
    ident: syn::Ident,
    is_param: bool,
    is_optional: bool,
    is_repeated: bool,
    is_oneof: bool,
    field_type: syn::Type,
    as_type: Option<syn::Type>,
    tag: u32,
    prost_type: String,
}

type TokenVec = Vec<proc_macro2::TokenStream>;
type OneofMapping<'a> = Option<(&'a &'a str, &'a Vec<otlp_model::OneofCase>)>;

/// Identifier generation utilities for consistent naming patterns
mod ident_utils {

    /// Create identifier with call_site span for generated code
    pub fn create_ident(name: &str) -> syn::Ident {
        syn::Ident::new(name, proc_macro2::Span::call_site())
    }

    /// Create identifier with span from another identifier
    pub fn create_ident_with_span(name: &str, span_from: &syn::Ident) -> syn::Ident {
        syn::Ident::new(name, span_from.span())
    }

    /// Generate builder name for a given type
    pub fn builder_name(type_name: &syn::Ident) -> syn::Ident {
        create_ident_with_span(&format!("{}Builder", type_name), type_name)
    }

    /// Generate visitor name for a given type
    pub fn visitor_name(type_name: &syn::Ident) -> syn::Ident {
        create_ident_with_span(&format!("{}Visitor", type_name), type_name)
    }

    /// Generate visitable name for a given type
    pub fn visitable_name(type_name: &syn::Ident) -> syn::Ident {
        create_ident_with_span(&format!("{}Visitable", type_name), type_name)
    }

    /// Generate adapter name for a given type
    pub fn adapter_name(type_name: &syn::Ident) -> syn::Ident {
        create_ident_with_span(&format!("{}MessageAdapter", type_name), type_name)
    }

    /// Generate visitor parameter name for a field
    pub fn visitor_param_name(field_name: &str) -> syn::Ident {
        create_ident(&format!("{}_visitor", field_name))
    }
}

/// Common type utilities for procedural macro generation
mod type_utils {
    use super::*;

    /// Extract the last segment identifier from a type path
    pub fn get_type_ident(ty: &syn::Type) -> Option<&syn::Ident> {
        match ty {
            syn::Type::Path(type_path) => type_path.path.segments.last().map(|seg| &seg.ident),
            _ => None,
        }
    }

    /// Check if a type is a specific container type (Option, Vec, etc.)
    pub fn is_container_type(ty: &syn::Type, container_name: &str) -> bool {
        get_type_ident(ty).map_or(false, |ident| ident == container_name)
    }

    /// Extract inner type from a generic container (Option<T>, Vec<T>)
    pub fn extract_inner_type(ty: &syn::Type) -> Option<syn::Type> {
        match ty {
            syn::Type::Path(type_path) => type_path
                .path
                .segments
                .last()
                .and_then(|seg| match &seg.arguments {
                    syn::PathArguments::AngleBracketed(args) => args.args.first(),
                    _ => None,
                })
                .and_then(|arg| match arg {
                    syn::GenericArgument::Type(inner_ty) => Some(inner_ty.clone()),
                    _ => None,
                }),
            _ => None,
        }
    }

    /// Check if a type is a primitive type
    pub fn is_primitive_type(ty: &syn::Type) -> bool {
        get_type_ident(ty).map_or(false, |ident| {
            matches!(
                ident.to_string().as_str(),
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
        })
    }

    /// Check if a type is Vec<u8> (bytes)
    pub fn is_bytes_type(ty: &syn::Type) -> bool {
        if let syn::Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Vec" {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            if let syn::Type::Path(inner_path) = inner_ty {
                                if let Some(inner_segment) = inner_path.path.segments.last() {
                                    return inner_segment.ident == "u8";
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Get primitive type visitor method name
    pub fn get_primitive_visitor_method(type_name: &str) -> &'static str {
        match type_name {
            "String" => "visit_string",
            "bool" => "visit_bool",
            "i32" => "visit_i32",
            "i64" => "visit_i64",
            "u32" | "u8" => "visit_u32", // Map u8 to u32
            "u64" => "visit_u64",
            "f32" | "f64" => "visit_f64",
            _ => "visit_unknown",
        }
    }

    /// Get primitive type visitor trait with generic argument
    pub fn get_primitive_visitor_trait(type_name: &str) -> proc_macro2::TokenStream {
        match type_name {
            "String" => quote! { crate::pdata::StringVisitor<Argument> },
            "bool" => quote! { crate::pdata::BooleanVisitor<Argument> },
            "i32" => quote! { crate::pdata::I32Visitor<Argument> },
            "i64" => quote! { crate::pdata::I64Visitor<Argument> },
            "u32" | "u8" => quote! { crate::pdata::U32Visitor<Argument> },
            "u64" => quote! { crate::pdata::U64Visitor<Argument> },
            "f32" | "f64" => quote! { crate::pdata::F64Visitor<Argument> },
            _ => quote! { UnknownVisitor<Argument> },
        }
    }

    /// Strip all container wrappers to get the base type
    pub fn get_base_type(ty: &syn::Type, is_optional: bool, is_repeated: bool) -> syn::Type {
        let mut current = ty.clone();

        if is_repeated && is_container_type(&current, "Vec") {
            current = extract_inner_type(&current).unwrap_or(current);
        }

        if is_optional && is_container_type(&current, "Option") {
            current = extract_inner_type(&current).unwrap_or(current);
        }

        current
    }
}

/// Path resolution utilities for OTLP types
mod path_utils {
    use super::*;

    /// Mapping of well-known OTLP paths to their module locations
    pub fn get_path_mappings() -> HashMap<&'static str, &'static str> {
        let mut map = HashMap::new();

        // Resource types
        map.insert(
            "resource::v1::Resource",
            "crate::proto::opentelemetry::resource::v1",
        );

        // Common types
        map.insert(
            "common::v1::InstrumentationScope",
            "crate::proto::opentelemetry::common::v1",
        );
        map.insert(
            "common::v1::KeyValue",
            "crate::proto::opentelemetry::common::v1",
        );
        map.insert(
            "common::v1::AnyValue",
            "crate::proto::opentelemetry::common::v1",
        );
        map.insert(
            "common::v1::ArrayValue",
            "crate::proto::opentelemetry::common::v1",
        );
        map.insert(
            "common::v1::KeyValueList",
            "crate::proto::opentelemetry::common::v1",
        );
        map.insert(
            "common::v1::EntityRef",
            "crate::proto::opentelemetry::common::v1",
        );

        map
    }

    /// Resolve full path for adapter or visitor
    pub fn resolve_type_path(type_path: &syn::TypePath, suffix: &str) -> String {
        let path_str = type_path
            .path
            .segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");

        let mappings = get_path_mappings();

        // Check for exact matches first
        for (pattern, module_path) in &mappings {
            if path_str.contains(pattern) {
                if let Some(type_name) = pattern.split("::").last() {
                    return format!("{}::{}{}", module_path, type_name, suffix);
                }
            }
        }

        // Pattern-based matching for different modules
        match path_str.as_str() {
            path if path.contains("metrics::v1::") => {
                if let Some(type_name) = path.split("::").last() {
                    format!(
                        "crate::proto::opentelemetry::metrics::v1::{}{}",
                        type_name, suffix
                    )
                } else {
                    format!("{}{}", path, suffix)
                }
            }
            path if path.contains("logs::v1::") => {
                if let Some(type_name) = path.split("::").last() {
                    format!(
                        "crate::proto::opentelemetry::logs::v1::{}{}",
                        type_name, suffix
                    )
                } else {
                    format!("{}{}", path, suffix)
                }
            }
            path if path.contains("trace::v1::") => {
                if let Some(type_name) = path.split("::").last() {
                    format!(
                        "crate::proto::opentelemetry::trace::v1::{}{}",
                        type_name, suffix
                    )
                } else {
                    format!("{}{}", path, suffix)
                }
            }
            path if path.contains("::") => {
                let parts: Vec<&str> = path.split("::").collect();
                if parts.len() == 2 {
                    // Nested type like "span::Event"
                    format!("{}::{}{}", parts[0], parts[1], suffix)
                } else {
                    format!("{}{}", path, suffix)
                }
            }
            _ => {
                // Local type - just add suffix
                format!("{}{}", path_str, suffix)
            }
        }
    }
}

/// Field processing utilities to reduce repetitive field handling logic
mod field_utils {
    use super::*;

    /// Generate field assignment patterns for different field types
    pub fn generate_field_assignment(
        info: &FieldInfo,
    ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        let field_name = &info.ident;

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

    /// Generate default initializer for a field
    pub fn generate_default_initializer(info: &FieldInfo) -> proc_macro2::TokenStream {
        let field_name = &info.ident;

        if info.is_optional {
            quote! { #field_name: None, }
        } else {
            let type_str = info.field_type.to_token_stream().to_string();
            match type_str.as_str() {
                "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" => {
                    quote! { #field_name: 0, }
                }
                "f32" | "f64" => quote! { #field_name: 0.0, },
                "bool" => quote! { #field_name: false, },
                _ => quote! { #field_name: ::core::default::Default::default(), },
            }
        }
    }

    /// Generate visitor call for a field with proper handling for different field types
    pub fn generate_visitor_call(info: &FieldInfo) -> Option<proc_macro2::TokenStream> {
        // Oneof fields are handled separately in generate_oneof_visitor_calls
        if info.is_oneof {
            return None;
        }

        let field_name = &info.ident;
        let field_name_str = field_name.to_string();
        let clean_field_name = if field_name_str.starts_with("r#") {
            &field_name_str[2..]
        } else {
            &field_name_str
        };

        let visitor_param = ident_utils::visitor_param_name(&clean_field_name);

        let visit_method = generate_visit_method_for_field(info);
        let needs_adapter = needs_adapter_for_field(info);
        let is_bytes = type_utils::is_bytes_type(&info.field_type);

        match (info.is_optional, info.is_repeated, needs_adapter, is_bytes) {
            (false, false, true, _) => {
                let adapter_name = get_adapter_name_for_field(info);
                Some(quote! {
                    arg = #visitor_param.#visit_method(arg, &(#adapter_name::new(&self.data.#field_name)));
                })
            }
            (false, false, false, _) => {
                if matches!(visit_method.to_string().as_str(), "visit_string") {
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
                    Some(
                        quote! { 
                            if let Some(f) = &self.data.#field_name {
                                arg = #visitor_param.#visit_method(arg, f);
                            }
                        },
                    )
                } else {
                    Some(
                        quote! { 
                            if let Some(f) = &self.data.#field_name {
                                arg = #visitor_param.#visit_method(arg, *f);
                            }
                        },
                    )
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
                } else if is_bytes {
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

    /// Generate visitor calls for oneof fields based on their variants
    pub fn generate_oneof_visitor_calls(
        info: &FieldInfo,
        oneof_mapping: OneofMapping,
    ) -> Vec<proc_macro2::TokenStream> {
        let mut visitor_calls = Vec::new();

        if !info.is_oneof {
            return visitor_calls;
        }

        if let Some((oneof_name, oneof_cases)) = oneof_mapping {
            if oneof_name.ends_with(&format!(".{}", info.ident)) {
                let field_name = &info.ident;

                for case in oneof_cases {
                    let variant_param_name = syn::Ident::new(
                        &format!("{}_{}_visitor", field_name, case.name),
                        field_name.span(),
                    );

                    // For now, generate a no-op visitor call that just threads the argument
                    // This ensures we consume all the visitor parameters that were generated
                    // TODO: Implement proper oneof variant matching and visiting
                    let visitor_call = quote! {
                        // TODO: Implement oneof visitor call for #variant_param_name
                        let _ = &#variant_param_name; // Consume the parameter to avoid unused warnings
                    };

                    visitor_calls.push(visitor_call);
                }
            }
        }

        visitor_calls
    }
}

/// Prost field annotation parsing utilities for protobuf encoding/decoding
/// Oneof processing utilities to reduce repetitive oneof handling
mod oneof_utils {
    use super::*;

    /// Generate constructor for a single oneof case
    pub fn generate_oneof_constructor(
        case: &otlp_model::OneofCase,
        oneof_name: &str,
        oneof_idx: usize,
        param_bounds: &[proc_macro2::TokenStream],
        param_decls: &[proc_macro2::TokenStream],
        param_args: &[proc_macro2::TokenStream],
        all_field_initializers: &[proc_macro2::TokenStream],
        type_params: &[syn::Ident],
        create_constructor: &dyn Fn(
            String,
            &[proc_macro2::TokenStream],
            &[proc_macro2::TokenStream],
            &[proc_macro2::TokenStream],
            &[proc_macro2::TokenStream],
        ) -> proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let case_type = syn::parse_str::<syn::Type>(case.type_param).unwrap();
        let variant_path = syn::parse_str::<syn::Expr>(case.value_variant).unwrap();
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

    /// Generate all constructors for a oneof mapping
    pub fn generate_oneof_constructors(
        oneof_mapping: (&str, &Vec<otlp_model::OneofCase>),
        param_names: &[&str],
        param_bounds: &[proc_macro2::TokenStream],
        param_decls: &[proc_macro2::TokenStream],
        param_args: &[proc_macro2::TokenStream],
        all_field_initializers: &[proc_macro2::TokenStream],
        type_params: &[syn::Ident],
        create_constructor: &dyn Fn(
            String,
            &[proc_macro2::TokenStream],
            &[proc_macro2::TokenStream],
            &[proc_macro2::TokenStream],
            &[proc_macro2::TokenStream],
        ) -> proc_macro2::TokenStream,
    ) -> Vec<proc_macro2::TokenStream> {
        let (oneof_path, oneof_cases) = oneof_mapping;
        let oneof_name = oneof_path.split('.').last().unwrap();
        let oneof_idx = param_names
            .iter()
            .position(|&name| name == oneof_name)
            .unwrap();

        oneof_cases
            .iter()
            .map(|case| {
                generate_oneof_constructor(
                    case,
                    oneof_name,
                    oneof_idx,
                    param_bounds,
                    param_decls,
                    param_args,
                    all_field_initializers,
                    type_params,
                    create_constructor,
                )
            })
            .collect()
    }
}

/// Simple prost field annotation parsing utilities
fn parse_prost_tag_and_type(field: &syn::Field) -> (u32, String) {
    // Find the #[prost(...)] attribute
    let prost_attr = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("prost"));
    
    if let Some(attr) = prost_attr {
        if let syn::Meta::List(meta_list) = &attr.meta {
            let tokens = &meta_list.tokens;
            
            // Simple parsing: extract tag number and type
            let attr_str = tokens.to_string();
            let mut tag = 0u32;
            let mut prost_type = "unknown".to_string();
            
            // Parse tag number from "tag = \"1\"" or "tag = 1"
            if let Some(tag_start) = attr_str.find("tag = ") {
                let tag_part = &attr_str[tag_start + 6..];
                if let Some(comma_pos) = tag_part.find(',') {
                    let tag_value = &tag_part[..comma_pos].trim().trim_matches('"');
                    tag = tag_value.parse().unwrap_or(0);
                } else {
                    let tag_value = tag_part.trim().trim_matches('"');
                    tag = tag_value.parse().unwrap_or(0);
                }
            }
            
            // Extract first identifier as protobuf type (string, int64, message, etc.)
            let parts: Vec<&str> = attr_str.split(',').collect();
            if let Some(first_part) = parts.first() {
                let type_part = first_part.trim();
                if !type_part.starts_with("tag") {
                    prost_type = type_part.to_string();
                }
            }
            
            return (tag, prost_type);
        }
    }
    
    // Default values if parsing fails
    (0, "unknown".to_string())
}

/// Attribute macro for associating the OTLP protocol buffer fully
/// qualified type name.
#[proc_macro_attribute]
pub fn qualified(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_str: String = args.to_string().trim_matches('"').into();

    // Parse input and add the qualified attribute in a more functional way
    let input_ast = syn::parse_macro_input!(input as syn::DeriveInput)
        .into_token_stream()
        .to_string();

    // Create a special doc comment that will store the qualified name
    let qualified_attr = syn::parse_quote! {
        #[doc(hidden, otlp_qualified_name = #args_str)]
    };

    // Parse again and add the attribute
    let mut final_ast = syn::parse_str::<DeriveInput>(&input_ast).unwrap();
    final_ast.attrs.push(qualified_attr);

    // Return the modified struct definition
    quote::quote!(#final_ast).into()
}

/// Derives the OTLP Message trait implementation for protocol buffer
/// message types. This enables additional OTLP-specific functionality
/// beyond what prost::Message provides.
#[proc_macro_derive(Message)]
pub fn derive_otlp_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let outer_name = &input.ident;

    // Get the fully qualified type name from attribute
    let type_name = input
        .attrs
        .iter()
        .find_map(|attr| {
            if attr.path().is_ident("doc") {
                // Use parse_nested_meta to extract the qualified name
                let mut qualified_name = None;
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("hidden") {
                        Ok(())
                    } else if meta.path.is_ident("otlp_qualified_name") {
                        let value = meta.value()?;
                        let lit: syn::LitStr = value.parse()?;
                        qualified_name = Some(lit.value());
                        Ok(())
                    } else {
                        Ok(())
                    }
                });
                qualified_name
            } else {
                None
            }
        })
        .unwrap();

    // Get required parameters for this type.
    let param_names = otlp_model::REQUIRED_PARAMS.get(type_name.as_str()).unwrap();

    // Check if this struct has a oneof field
    let oneof_mapping = otlp_model::ONEOF_MAPPINGS
        .iter()
        .find(|(field, _)| field.starts_with(&type_name));

    // Extract all fields from the struct definition
    let struct_fields = match &input.data {
        syn::Data::Struct(data) => {
            if let syn::Fields::Named(fields) = &data.fields {
                fields.named.iter().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };

    // If there are no fields, it's either an empty message or an enum,
    // either way should not be listed, no builder is needed.
    if struct_fields.is_empty() {
        panic!("message with empty fields")
    }

    // Helper function to check if a type is Option<T>
    let is_option_type = |ty: &syn::Type| -> bool { type_utils::is_container_type(ty, "Option") };

    // Helper function to check if a type is Vec<T>
    let is_vec_type = |ty: &syn::Type| -> bool { type_utils::is_container_type(ty, "Vec") };

    // Function to check if a field is marked as optional
    let is_optional_repeated = |field: &syn::Field| {
        // Check prost attributes first
        let attr_optional = field.attrs.iter().any(|attr| {
            attr.path().is_ident("prost") && attr.to_token_stream().to_string().contains("optional")
        });
        let attr_repeated = field.attrs.iter().any(|attr| {
            attr.path().is_ident("prost") && attr.to_token_stream().to_string().contains("repeated")
        });

        // Also check the actual type structure
        let type_optional = is_option_type(&field.ty);
        let type_repeated = is_vec_type(&field.ty);

        (
            attr_optional || type_optional,
            attr_repeated || type_repeated,
        )
    };

    // Extract option inner type as a standalone function for better reuse
    let extract_option_inner_type = |ty: &syn::Type| -> Option<(syn::Type, bool)> {
        if type_utils::is_container_type(ty, "Option") {
            type_utils::extract_inner_type(ty).map(|inner| (inner, true))
        } else {
            None
        }
    };

    let fields_original: Vec<FieldInfo> = struct_fields
        .iter()
        .filter_map(|field| {
            // Early return if no identifier
            field.ident.as_ref().map(|ident| {
                let ident_str = ident.to_string();
                let field_path = format!("{}.{}", type_name, ident_str);
                let is_param = param_names.contains(&ident_str.as_str());
                let (is_optional, is_repeated) = is_optional_repeated(field);
                let is_oneof = oneof_mapping.map(|x| *x.0 == field_path).unwrap_or(false);

                // Process type information
                let (inner_type, is_optional_extraction_ok) = if is_optional {
                    extract_option_inner_type(&field.ty)
                        .unwrap_or_else(|| (field.ty.clone(), false))
                } else {
                    (field.ty.clone(), true)
                };

                // Validate optional field extraction
                if is_optional && !is_optional_extraction_ok {
                    panic!(
                        "Field '{}' is marked optional but does not have a valid Option<T> type",
                        ident
                    );
                }

                // Get type overrides if present
                let (field_type, as_type) = otlp_model::FIELD_TYPE_OVERRIDES
                    .get(field_path.as_str())
                    .map(|over| {
                        (
                            syn::parse_str::<syn::Type>(over.datatype).unwrap(),
                            Some(syn::parse_str::<syn::Type>(over.fieldtype).unwrap()),
                        )
                    })
                    .unwrap_or_else(|| (inner_type, None));

                // Parse Prost field annotation
                let (tag, prost_type) = parse_prost_tag_and_type(field);

                FieldInfo {
                    ident: ident.clone(),
                    is_param,
                    is_optional,
                    is_repeated,
                    is_oneof,
                    field_type,
                    as_type,
                    tag,
                    prost_type,
                }
            })
        })
        .collect();

    // Partition fields into ordered parameters and remaining builder fields.
    let param_fields: Vec<_> = param_names
        .iter()
        .map(|param_name| {
            fields_original
                .iter()
                .find(|info| {
                    let ident = info.ident.to_string();
                    info.is_param && ident == *param_name
                })
                .unwrap()
        })
        .cloned()
        .collect();
    let builder_fields: Vec<_> = fields_original
        .iter()
        .filter(|info| !info.is_param)
        .cloned()
        .collect();
    let all_fields: Vec<_> = param_fields
        .iter()
        .chain(builder_fields.iter())
        .cloned()
        .collect();

    let mut tokens = TokenStream::new();

    tokens.extend(derive_otlp_builders(
        outer_name,
        param_names,
        &param_fields,
        &builder_fields,
        &all_fields,
        oneof_mapping,
    ));

    tokens.extend(derive_otlp_visitors(
        outer_name,
        param_names,
        &param_fields,
        &builder_fields,
        &all_fields,
        oneof_mapping,
    ));

    tokens.extend(derive_otlp_adapters(
        outer_name,
        param_names,
        &param_fields,
        &builder_fields,
        &all_fields,
        oneof_mapping,
    ));

    tokens
}

/// Emits the builders, new(), and finish() methods.
fn derive_otlp_builders(
    outer_name: &syn::Ident,
    param_names: &Vec<&str>,
    param_fields: &[FieldInfo],
    builder_fields: &[FieldInfo],
    all_fields: &[FieldInfo],
    oneof_mapping: OneofMapping,
) -> TokenStream {
    let builder_name = ident_utils::builder_name(outer_name);

    // Generate generic type parameters names like ["T1", "T2", ...]
    let type_params: Vec<syn::Ident> = (0..all_fields.len())
        .map(|idx| ident_utils::create_ident(&format!("T{}", idx + 1)))
        .collect();

    // Generate a list of arguments to pass from build() to new().
    let param_args: TokenVec = param_fields
        .iter()
        .map(|info| {
            let field_name = &info.ident;
            quote! { #field_name }
        })
        .collect();

    // Generate parameter declarations and where bounds
    let (param_decls, param_bounds): (TokenVec, TokenVec) = param_fields
        .iter()
        .enumerate()
        .map(|(idx, info)| {
            let param_name = &info.ident;
            let type_param = &type_params[idx];
            let target_type = &info.field_type;

            let decl = quote! { #param_name: #type_param };
            let bound = quote! { #type_param: Into<#target_type> };

            (decl, bound)
        })
        .unzip();

    // Generate field assignments and initializers
    let (field_assignments, field_initializers): (TokenVec, TokenVec) = all_fields
        .iter()
        .map(field_utils::generate_field_assignment)
        .unzip();

    // Default initializers for fields
    let default_initializers: TokenVec = all_fields
        .iter()
        .map(field_utils::generate_default_initializer)
        .collect();

    // All field initializers includes parameters and defaults
    let all_field_initializers: Vec<_> = (0..all_fields.len())
        .map(|idx| {
            if idx < param_names.len() {
                field_initializers[idx].clone()
            } else {
                default_initializers[idx].clone()
            }
        })
        .collect();

    // Generate builder methods
    let builder_methods: TokenVec = all_fields
        .iter()
        .enumerate()
        .filter(|(_, info)| !info.is_oneof)
        .map(|(idx, info)| {
            let field_name = &info.ident;
            let field_type = &info.field_type;
            let value_assignment = field_assignments[idx].clone();

            quote! {
                pub fn #field_name<T: Into<#field_type>>(mut self, #field_name: T) -> Self
                {
                    #value_assignment
                    self
                }
            }
        })
        .collect();

    // When there are no builder fields, we can skip the builder struct.
    let derive_builder = !builder_fields.is_empty();

    // Function to build constructors used in oneof and normal cases.
    let create_constructor =
        |suffix: String,
         cur_param_bounds: &[proc_macro2::TokenStream],
         cur_param_decls: &[proc_macro2::TokenStream],
         cur_param_args: &[proc_macro2::TokenStream],
         cur_field_initializers: &[proc_macro2::TokenStream]| {
            let build_name = ident_utils::create_ident(&format!("build{}", suffix));
            let new_name = ident_utils::create_ident(&format!("new{}", suffix));

            let mut cons = quote! {
            pub fn #new_name<#(#cur_param_bounds),*>(#(#cur_param_decls),*) -> Self {
                        Self{
                #(#cur_field_initializers)*
                }
            }
            };
            if derive_builder {
                cons.extend(quote! {
                pub fn #build_name<#(#cur_param_bounds),*>(#(#cur_param_decls),*) -> #builder_name {
                            #builder_name{
                    inner: #outer_name::#new_name(#(#cur_param_args),*),
                            }
                }
                });
            }
            cons
        };

    // Build constructors for both regular and oneof cases.
    let all_constructors: TokenVec = match oneof_mapping {
        None => {
            vec![create_constructor(
                "".to_string(),
                &param_bounds,
                &param_decls,
                &param_args,
                &all_field_initializers,
            )]
        }
        Some((oneof_path, oneof_cases)) => oneof_utils::generate_oneof_constructors(
            (oneof_path, oneof_cases),
            param_names,
            &param_bounds,
            &param_decls,
            &param_args,
            &all_field_initializers,
            &type_params,
            &create_constructor,
        ),
    };

    // Produce expanded implementation
    let mut expanded = quote! {
            impl #outer_name {
        #(#all_constructors)*
        }
    };

    if derive_builder {
        expanded.extend(quote! {
                pub struct #builder_name {
                    inner: #outer_name,
                }

                impl #builder_name {
                    #(#builder_methods)*

                    pub fn finish(self) -> #outer_name {
                        self.inner
                    }
                }

                impl std::convert::From<#builder_name> for #outer_name {
                    fn from(builder: #builder_name) -> Self {
                        builder.finish()
                    }
                }
        });
    }

    TokenStream::from(expanded)
}

/// Emits the visitor, visitable and adapters methods.
fn derive_otlp_visitors(
    outer_name: &syn::Ident,
    _param_names: &Vec<&str>,
    _param_fields: &[FieldInfo],
    _builder_fields: &[FieldInfo],
    all_fields: &[FieldInfo],
    oneof_mapping: OneofMapping,
) -> TokenStream {
    let visitor_name = ident_utils::visitor_name(outer_name);
    let visitable_name = ident_utils::visitable_name(outer_name);
    let visitor_method_name = syn::Ident::new(
        &format!("visit_{}", outer_name).to_case(Case::Snake),
        outer_name.span(),
    );
    let visitable_method_name = syn::Ident::new(
        &format!("accept_{}", outer_name).to_case(Case::Snake),
        outer_name.span(),
    );

    let mut visitable_args: TokenVec = Vec::new();

    for info in all_fields {
        if info.is_oneof {
            // For oneof fields, generate separate parameters for each variant
            if let Some((oneof_name, oneof_cases)) = oneof_mapping {
                if oneof_name.ends_with(&format!(".{}", info.ident)) {
                    // This is the oneof field we're looking for
                    for case in oneof_cases {
                        let variant_param_name = syn::Ident::new(
                            &format!("{}_{}", info.ident, case.name),
                            info.ident.span(),
                        );

                        // Parse the type_param to get the visitor trait path
                        if let Ok(case_type) = syn::parse_str::<syn::Type>(case.type_param) {
                            let visitor_type = generate_visitor_type_for_oneof_variant(&case_type);
                            visitable_args.push(quote! { #variant_param_name: #visitor_type });
                        }
                    }
                    continue;
                }
            }
        }

        // For non-oneof fields, generate normal visitor parameter
        let param_name = &info.ident;
        let visitor_type = generate_visitor_trait_for_field(info);
        visitable_args.push(quote! { #param_name: impl #visitor_type });
    }

    let expanded = quote! {
    pub trait #visitor_name<Argument> {
        type Return;
        fn #visitor_method_name(&mut self, arg: Argument, v: impl #visitable_name<Argument>) -> Self::Return;
    }

    pub trait #visitable_name<Argument> {
        fn #visitable_method_name(&self, arg: Argument, #(#visitable_args),*) -> Argument;
    }

    impl<Argument> #visitor_name<Argument> for crate::pdata::NoopVisitor {
        type Return = Argument;
        fn #visitor_method_name(&mut self, arg: Argument, _v: impl #visitable_name<Argument>) -> Self::Return {
            arg
        }
    }
    };

    TokenStream::from(expanded)
}

/// Generate visitor type for a oneof variant
fn generate_visitor_type_for_oneof_variant(case_type: &syn::Type) -> proc_macro2::TokenStream {
    match case_type {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                match segment.ident.to_string().as_str() {
                    "Vec" => {
                        if let Some(inner_type) = type_utils::extract_inner_type(case_type) {
                            if type_utils::is_bytes_type(case_type) {
                                quote! { impl crate::pdata::BytesVisitor<Argument> }
                            } else {
                                let visitor_trait = generate_visitor_trait_for_field(&FieldInfo {
                                    ident: syn::Ident::new("temp", proc_macro2::Span::call_site()),
                                    is_param: false,
                                    is_optional: false,
                                    is_repeated: true,
                                    is_oneof: false,
                                    field_type: inner_type,
                                    as_type: None,
                                    tag: 0,
                                    prost_type: "message".to_string(),
                                });
                                quote! { impl #visitor_trait }
                            }
                        } else {
                            quote! { impl UnknownVisitor }
                        }
                    }
                    type_name if type_utils::is_primitive_type(case_type) => {
                        let visitor_trait = type_utils::get_primitive_visitor_trait(type_name);
                        quote! { impl #visitor_trait }
                    }
                    _ => {
                        // For message types, use the visitor generation utility
                        let visitor_trait = generate_visitor_trait_for_field(&FieldInfo {
                            ident: syn::Ident::new("temp", proc_macro2::Span::call_site()),
                            is_param: false,
                            is_optional: false,
                            is_repeated: false,
                            is_oneof: false,
                            field_type: case_type.clone(),
                            as_type: None,
                            tag: 0,
                            prost_type: "message".to_string(),
                        });
                        quote! { impl #visitor_trait }
                    }
                }
            } else {
                quote! { impl UnknownVisitor }
            }
        }
        _ => quote! { impl UnknownVisitor },
    }
}

impl FieldInfo {
    /// Extract the base type by stripping Option<T> and Vec<T> wrappers
    fn extract_base_type(&self) -> syn::Type {
        type_utils::get_base_type(&self.field_type, self.is_optional, self.is_repeated)
    }
}

/// Determine if a field needs to be wrapped in an adapter (message types) vs used directly (primitives)
fn needs_adapter_for_field(info: &FieldInfo) -> bool {
    // Use the parsed prost_type for fast determination - this is the main benefit!
    match info.prost_type.as_str() {
        "message" | "enumeration" => true,  // Complex types need adapters
        "string" | "int64" | "uint32" | "int32" | "uint64" | "bool" | "double" | "float" | "bytes" => false,  // Primitives don't need adapters
        _ => {
            // For unknown or missing prost_type, fall back to the original type-based analysis
            // This ensures backward compatibility and handles edge cases
            let base_type = if let Some(as_type) = &info.as_type {
                as_type
            } else if info.is_repeated {
                // For repeated fields, check the element type
                &type_utils::get_base_type(&info.field_type, false, true)
            } else {
                &info.field_type
            };
            needs_adapter_for_type(base_type)
        }
    }
}

/// Determine if a type needs an adapter wrapper
fn needs_adapter_for_type(ty: &syn::Type) -> bool {
    // Primitive types and Vec<u8> don't need adapters
    !type_utils::is_primitive_type(ty) && !type_utils::is_bytes_type(ty)
}

/// Get the adapter name for a field type
fn get_adapter_name_for_field(info: &FieldInfo) -> proc_macro2::TokenStream {
    // Use the parsed prost_type to determine if we should generate an adapter,
    // but still use the original type path resolution for proper module qualification
    match info.prost_type.as_str() {
        "message" | "enumeration" => {
            // These types need adapters - use original complex path resolution
            // to ensure proper module qualification
            let base_type = if info.is_repeated {
                info.extract_base_type()
            } else {
                info.field_type.clone()
            };

            match &base_type {
                syn::Type::Path(type_path) => {
                    if let Some(segment) = type_path.path.segments.last() {
                        let type_name = segment.ident.to_string();
                        let adapter_name = format!("{}MessageAdapter", type_name);

                        // Use the original complex path resolution for proper module qualification
                        let adapter_path = path_utils::resolve_type_path(type_path, "MessageAdapter");

                        // Parse the path and return as TokenStream
                        match syn::parse_str::<syn::Path>(&adapter_path) {
                            Ok(path) => quote! { #path },
                            Err(_) => {
                                // Fallback to simple name if parsing fails
                                let adapter_ident = syn::Ident::new(&adapter_name, segment.ident.span());
                                quote! { #adapter_ident }
                            }
                        }
                    } else {
                        quote! { UnknownMessageAdapter }
                    }
                }
                _ => quote! { UnknownMessageAdapter },
            }
        }
        // Primitive types like "string", "int64", "uint32", "bytes", "bool", "double", "float"
        // don't need adapters - this case should be handled by needs_adapter_for_field()
        _ => {
            // For primitive types or unknown types, this shouldn't be called
            // if needs_adapter_for_field() is working correctly
            quote! { PrimitiveAdapter }
        }
    }
}



/// Emits the adapter struct and implementation for the visitor pattern
fn derive_otlp_adapters(
    outer_name: &syn::Ident,
    _param_names: &Vec<&str>,
    _param_fields: &[FieldInfo],
    _builder_fields: &[FieldInfo],
    all_fields: &[FieldInfo],
    oneof_mapping: OneofMapping,
) -> TokenStream {
    let adapter_name = ident_utils::adapter_name(outer_name);
    let visitable_name = ident_utils::visitable_name(outer_name);

    // Generate the method name based on the outer type name
    // Convert CamelCase to snake_case (e.g., LogsData -> logs_data)
    let visitable_method_name = syn::Ident::new(
        &format!("accept_{}", outer_name.to_string().to_case(Case::Snake)),
        outer_name.span(),
    );

    // Generate visitor calls for each field
    let mut visitor_calls: TokenVec = all_fields
        .iter()
        .filter_map(field_utils::generate_visitor_call)
        .collect();

    // Add oneof visitor calls
    for info in all_fields {
        if info.is_oneof {
            let oneof_calls = field_utils::generate_oneof_visitor_calls(info, oneof_mapping);
            visitor_calls.extend(oneof_calls);
        }
    }

    // Generate visitor parameters for the visitable trait method
    let mut visitor_params: TokenVec = Vec::new();

    for info in all_fields {
        if info.is_oneof {
            // Generate parameters for each oneof variant (matching the trait definition)
            if let Some((oneof_name, oneof_cases)) = oneof_mapping {
                if oneof_name.ends_with(&format!(".{}", info.ident)) {
                    for case in oneof_cases {
                        let variant_param_name = syn::Ident::new(
                            &format!("{}_{}_visitor", info.ident, case.name),
                            info.ident.span(),
                        );

                        if let Ok(case_type) = syn::parse_str::<syn::Type>(case.type_param) {
                            let visitor_type = generate_visitor_type_for_oneof_variant(&case_type);
                            visitor_params.push(quote! { mut #variant_param_name: #visitor_type });
                        }
                    }
                }
            }
        } else {
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

            // Generate the appropriate visitor trait type
            let visitor_trait = generate_visitor_trait_for_field(info);
            visitor_params.push(quote! { mut #visitor_param: impl #visitor_trait });
        }
    }

    let expanded = quote! {
        /// MessageAdapter for presenting OTLP message objects as visitable.
        pub struct #adapter_name<'a> {
            data: &'a #outer_name,
        }

        impl<'a> #adapter_name<'a> {
            /// Create a new adapter
            pub fn new(data: &'a #outer_name) -> Self {
                Self { data }
            }
        }

        impl<'a, Argument> #visitable_name<Argument> for &#adapter_name<'a> {
            fn #visitable_method_name(&self, mut arg: Argument, #(#visitor_params),*) -> Argument {
                #(#visitor_calls)*
                arg
            }
        }
    };

    TokenStream::from(expanded)
}

/// Generate the correct visit method name for a field based on its type
fn generate_visit_method_for_field(info: &FieldInfo) -> syn::Ident {
    // Special handling for repeated Vec<u8> fields (bytes)
    if info.is_repeated && type_utils::is_bytes_type(&info.field_type) {
        return syn::Ident::new("visit_bytes", proc_macro2::Span::call_site());
    }

    // Check if this field has an as_type (enum field), use the underlying primitive type
    let base_type = if let Some(as_type) = &info.as_type {
        as_type.clone()
    } else if info.is_repeated {
        info.extract_base_type()
    } else {
        info.field_type.clone()
    };

    let method_name = if let Some(type_ident) = type_utils::get_type_ident(&base_type) {
        let type_name = type_ident.to_string();
        if type_utils::is_primitive_type(&base_type) {
            type_utils::get_primitive_visitor_method(&type_name).to_string()
        } else {
            // For message types, convert to snake_case (e.g., LogRecord -> visit_log_record)
            format!("visit_{}", type_name.to_case(Case::Snake))
        }
    } else {
        "visit_unknown".to_string()
    };

    syn::Ident::new(&method_name, proc_macro2::Span::call_site())
}

/// Generate visitor trait for a field based on its type  
fn generate_visitor_trait_for_field(info: &FieldInfo) -> proc_macro2::TokenStream {
    // Special handling for repeated Vec<u8> fields (bytes)
    if info.is_repeated && type_utils::is_bytes_type(&info.field_type) {
        return quote! { crate::pdata::BytesVisitor<Argument> };
    }

    // Check if this field has an as_type (enum field), use the underlying primitive type
    let base_type = if let Some(as_type) = &info.as_type {
        as_type.clone()
    } else if info.is_repeated {
        info.extract_base_type()
    } else {
        info.field_type.clone()
    };

    generate_visitor_trait_for_type(&base_type)
}

/// Generate visitor trait for a given type
fn generate_visitor_trait_for_type(ty: &syn::Type) -> proc_macro2::TokenStream {
    if let Some(type_ident) = type_utils::get_type_ident(ty) {
        let type_name = type_ident.to_string();

        if type_utils::is_primitive_type(ty) {
            let primitive_trait = type_utils::get_primitive_visitor_trait(&type_name);
            quote! { #primitive_trait }
        } else if let syn::Type::Path(type_path) = ty {
            // For message types, generate visitor trait using the path resolver
            let visitor_name = format!("{}Visitor", type_name);
            let visitor_path = resolve_visitor_trait_path_for_type(type_path, &visitor_name);

            match syn::parse_str::<syn::Path>(&visitor_path) {
                Ok(path) => quote! { #path<Argument> },
                Err(_) => {
                    let visitor_ident = syn::Ident::new(&visitor_name, type_ident.span());
                    quote! { #visitor_ident<Argument> }
                }
            }
        } else {
            quote! { UnknownVisitor<Argument> }
        }
    } else {
        quote! { UnknownVisitor<Argument> }
    }
}

/// Resolve visitor trait path for protobuf types with proper module resolution
fn resolve_visitor_trait_path_for_type(type_path: &syn::TypePath, _visitor_name: &str) -> String {
    path_utils::resolve_type_path(type_path, "Visitor")
}
