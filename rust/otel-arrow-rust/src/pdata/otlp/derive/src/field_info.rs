// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otlp_model::OneofCase;
use otlp_model::OneofMapping;
use quote::ToTokens;

#[derive(Clone, Debug)]
pub struct FieldInfo {
    pub ident: syn::Ident,

    // is_param is used by the builder pattern, for obligatory parameters to new() vs builder options.
    pub is_param: bool,
    pub is_optional: bool,
    pub is_repeated: bool,
    pub is_bytes: bool,
    pub is_message: bool,
    pub oneof: Option<Vec<OneofCase>>,
    pub as_type: Option<syn::Type>, // primitive type for enums
    pub proto_type: String,
    pub qualifier: Option<proc_macro2::TokenStream>,

    pub tag: u32,

    pub base_type_name: String,
    pub full_type_name: syn::Type,
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
            let mut proto_type = "unknown".to_string();

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
                    proto_type = type_part.to_string();
                }
            }

            return (tag, proto_type);
        }
    }

    panic!("did not parse protobuf tag number");
}

impl FieldInfo {
    pub(crate) fn new(
        field: &syn::Field,
        type_name: &str,
        param_names: &[String],
        oneof_mapping: &OneofMapping,
    ) -> Self {
        field
            .ident
            .as_ref()
            .map(|ident| {
                let ident_str = ident.to_string();
                let field_path = format!("{}.{}", type_name, ident_str);
                let is_param = param_names.contains(&ident_str);

                let oneof = oneof_mapping
                    .as_ref()
                    .filter(|x| x.0 == field_path)
                    .map(|x| x.1.clone());

                let is_optional = Self::is_optional(field);
                let is_repeated = Self::is_repeated(field);
                let is_message = Self::is_message(field);
                let is_bytes = Self::is_bytes(field);

                // Process type information
                let inner_type = if is_optional || is_repeated {
                    Self::extract_inner_type(&field.ty).expect("must have inner")
                } else {
                    field.ty.clone()
                };

                // TODO: field_type should be used
                let (_field_type, as_type) = otlp_model::FIELD_TYPE_OVERRIDES
                    .get(field_path.as_str())
                    .map(|over| {
                        (
                            syn::parse_str::<syn::Type>(over.datatype).unwrap(),
                            Some(syn::parse_str::<syn::Type>(over.fieldtype).unwrap()),
                        )
                    })
                    .unwrap_or_else(|| (inner_type.clone(), None));

                // Decompose type into base name and qualifier
                let (base_type_name, qualifier) = Self::decompose_type(&inner_type);
                let full_type_name = inner_type;

                // Parse Prost tag, _p
                let (tag, proto_type) = parse_prost_tag_and_type(field);

                FieldInfo {
                    ident: ident.clone(),
                    is_param,
                    is_optional,
                    is_repeated,
                    is_message,
                    is_bytes,
                    oneof,
                    as_type,
                    proto_type,
                    tag,
                    base_type_name,
                    full_type_name,
                    qualifier,
                }
            })
            .expect("has field name")
    }

    pub fn related_type(&self, suffix: &str) -> proc_macro2::TokenStream {
        if let Some(ref qualifier) = self.qualifier {
            let base_with_suffix = syn::Ident::new(
                &format!("{}{}", self.base_type_name, suffix),
                proc_macro2::Span::call_site()
            );
            quote::quote! { #qualifier::#base_with_suffix }
        } else {
            let type_with_suffix = syn::Ident::new(
                &format!("{}{}", self.base_type_name, suffix),
                proc_macro2::Span::call_site()
            );
            quote::quote! { #type_with_suffix }
        }
    }

    /// Decompose a type into base type name and qualifier
    /// Returns (base_type_name, qualifier) where qualifier is the module path
    fn decompose_type(ty: &syn::Type) -> (String, Option<proc_macro2::TokenStream>) {
        match ty {
            syn::Type::Path(type_path) => {
                let path_str = type_path
                    .path
                    .segments
                    .iter()
                    .map(|seg| seg.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");
                
                // Get the base type name (last segment)
                let base_type_name = type_path
                    .path
                    .segments
                    .last()
                    .map(|seg| seg.ident.to_string())
                    .unwrap_or_else(|| path_str.clone());

                // If there's only one segment, no qualifier needed
                if type_path.path.segments.len() == 1 {
                    return (base_type_name, None);
                }

                // Create qualifier from all but last segment
                let qualifier_segments: Vec<_> = type_path
                    .path
                    .segments
                    .iter()
                    .take(type_path.path.segments.len() - 1)
                    .collect();

                if qualifier_segments.is_empty() {
                    (base_type_name, None)
                } else {
                    let qualifier = qualifier_segments
                        .iter()
                        .map(|seg| &seg.ident)
                        .collect::<Vec<_>>();
                    (base_type_name, Some(quote::quote! { #(#qualifier)::* }))
                }
            }
            _ => {
                // For non-path types, just use the string representation as base name
                let base_name = quote::quote! { #ty }.to_string();
                (base_name, None)
            }
        }
    }

    /// Extract inner type from a generic container (Option<T>, Vec<T>)
    fn extract_inner_type(ty: &syn::Type) -> Option<syn::Type> {
        match ty {
            syn::Type::Path(type_path) => type_path
                .path
                .segments
                .last()
                // @@@ Note: not sure if both branches below are used?
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

    fn is_optional(field: &syn::Field) -> bool {
        Self::has_prost_attr(field, "optional")
    }

    fn is_repeated(field: &syn::Field) -> bool {
        Self::has_prost_attr(field, "repeated")
    }

    fn is_message(field: &syn::Field) -> bool {
        Self::has_prost_attr(field, "message")
    }

    fn is_bytes(field: &syn::Field) -> bool {
        Self::has_prost_attr(field, "bytes=\"vec\"")
    }

    fn has_prost_attr(field: &syn::Field, value: &'static str) -> bool {
        field.attrs.iter().any(|attr| {
            attr.path().is_ident("prost") && attr.to_token_stream().to_string().contains(value)
        })
    }
}
