// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use convert_case::{Case, Casing};
use otlp_model::OneofCase;
use otlp_model::OneofMapping;
use quote::{ToTokens, quote};

#[derive(Clone, Debug)]
pub struct FieldInfo {
    pub ident: syn::Ident,

    // is_param is used by the builder pattern, for obligatory parameters to new() vs builder options.
    pub is_param: bool,
    pub is_optional: bool,
    pub is_repeated: bool,
    pub is_message: bool,
    pub oneof: Option<Vec<OneofCase>>,
    pub as_type: Option<syn::Type>,   // primitive type for enums
    pub enum_type: Option<syn::Type>, // enum type for enums (from datatype in FIELD_TYPE_OVERRIDES)
    pub proto_type: String,
    pub qualifier: Option<proc_macro2::TokenStream>,

    pub tag: u32,

    pub base_type_name: String,
    pub full_type_name: syn::Type,

    // Visitor-related precomputed information
    pub visitor_trait: proc_macro2::TokenStream,
    pub visitable_trait: proc_macro2::TokenStream,
    pub visitor_param_name: syn::Ident,
    pub visit_method_name: syn::Ident,
}

/// Helper function to parse tag value from different formats
fn parse_tag_value(attr_str: &str, tag_start_pattern: &str, offset: usize) -> Option<u32> {
    attr_str
        .find(tag_start_pattern)
        .and_then(|tag_start| {
            let tag_part = &attr_str[tag_start + offset..];
            let tag_value = if let Some(comma_pos) = tag_part.find(',') {
                &tag_part[..comma_pos]
            } else {
                tag_part
            };
            
            tag_value
                .trim()
                .trim_matches('"')
                .parse()
                .ok()
        })
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
            let attr_str = tokens.to_string();

            // Parse tag number using helper function with multiple patterns
            let tag = parse_tag_value(&attr_str, "tag=", 4)
                .or_else(|| parse_tag_value(&attr_str, "tag = ", 6))
                .unwrap_or(0);

            // Extract first identifier as protobuf type (string, int64, message, etc.)
            let proto_type = attr_str
                .split(',')
                .next()
                .map(|first_part| first_part.trim())
                .filter(|type_part| !type_part.starts_with("tag"))
                .map(|type_part| type_part.to_string())
                .unwrap_or_else(|| "unknown".to_string());

            return (tag, proto_type);
        }
    }

    // Return default values instead of panicking, which helps us see more of what's happening
    (0, "unknown".to_string())
}

/// Mapping for primitive types to their corresponding trait names
fn get_primitive_trait_mapping(base_type_name: &str, trait_suffix: &str) -> Option<proc_macro2::TokenStream> {
    let trait_name = match base_type_name {
        "String" => format!("String{}", trait_suffix),
        "bool" => format!("Boolean{}", trait_suffix),
        "i32" => format!("I32{}", trait_suffix),
        "i64" => format!("I64{}", trait_suffix),
        "u32" | "u8" => format!("U32{}", trait_suffix),
        "u64" => format!("U64{}", trait_suffix),
        "f32" | "f64" => format!("F64{}", trait_suffix),
        _ => return None,
    };
    
    let trait_ident = syn::Ident::new(&trait_name, proc_macro2::Span::call_site());
    Some(quote! { crate::pdata::#trait_ident<Argument> })
}

/// Mapping for repeated primitive types to their corresponding slice trait names
fn get_repeated_primitive_trait_mapping(base_type_name: &str, trait_suffix: &str) -> Option<proc_macro2::TokenStream> {
    let slice_trait_name = format!("Slice{}", trait_suffix);
    let slice_trait_ident = syn::Ident::new(&slice_trait_name, proc_macro2::Span::call_site());
    
    match base_type_name {
        "String" => Some(quote! { crate::pdata::#slice_trait_ident<Argument, String> }),
        "bool" => Some(quote! { crate::pdata::#slice_trait_ident<Argument, bool> }),
        "i32" => Some(quote! { crate::pdata::#slice_trait_ident<Argument, i32> }),
        "i64" => Some(quote! { crate::pdata::#slice_trait_ident<Argument, i64> }),
        "u32" | "u8" => Some(quote! { crate::pdata::#slice_trait_ident<Argument, u32> }),
        "u64" => Some(quote! { crate::pdata::#slice_trait_ident<Argument, u64> }),
        "f32" => Some(quote! { crate::pdata::#slice_trait_ident<Argument, f32> }),
        "f64" => Some(quote! { crate::pdata::#slice_trait_ident<Argument, f64> }),
        _ => None,
    }
}

/// Generate standard trait for non-primitive types
fn generate_standard_trait(
    base_type_name: &str,
    qualifier: &Option<proc_macro2::TokenStream>,
    trait_suffix: &str,
) -> proc_macro2::TokenStream {
    let needs_argument = trait_suffix == "Visitor" || trait_suffix == "Visitable";
    
    if let Some(ref qualifier) = qualifier {
        let base_with_suffix = syn::Ident::new(
            &format!("{}{}", base_type_name, trait_suffix),
            proc_macro2::Span::call_site(),
        );
        if needs_argument {
            quote! { #qualifier::#base_with_suffix<Argument> }
        } else {
            quote! { #qualifier::#base_with_suffix }
        }
    } else {
        let type_with_suffix = syn::Ident::new(
            &format!("{}{}", base_type_name, trait_suffix),
            proc_macro2::Span::call_site(),
        );
        if needs_argument {
            quote! { #type_with_suffix<Argument> }
        } else {
            quote! { #type_with_suffix }
        }
    }
}

impl FieldInfo {
    /// Parse field type overrides and return (enum_type, as_type)
    fn parse_field_type_overrides(
        field_path: &str,
        _inner_type: &syn::Type,
    ) -> (Option<syn::Type>, Option<syn::Type>) {
        otlp_model::FIELD_TYPE_OVERRIDES
            .get(field_path)
            .and_then(|over| {
                // Parse datatype and fieldtype with proper error handling
                let datatype = syn::parse_str::<syn::Type>(over.datatype).ok()?;
                let fieldtype = syn::parse_str::<syn::Type>(over.fieldtype).ok();
                
                // If we have an override, store the enum type
                if fieldtype.is_some() {
                    Some((Some(datatype), fieldtype))
                } else {
                    Some((None, None))
                }
            })
            .unwrap_or((None, None))
    }

    /// Compute visitor-related information for a field
    fn compute_visitor_info(
        field_ident: &syn::Ident,
        proto_type: &str,
        is_repeated: bool,
        is_primitive: bool,
        base_type_name: &str,
        qualifier: &Option<proc_macro2::TokenStream>,
    ) -> (
        syn::Ident,
        syn::Ident,
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
    ) {
        let field_name_str = field_ident.to_string();
        let clean_field_name = if field_name_str.starts_with("r#") {
            &field_name_str[2..]
        } else {
            &field_name_str
        };

        let visitor_param_name = syn::Ident::new(
            &format!("{}_visitor", clean_field_name),
            proc_macro2::Span::call_site(),
        );

        let visit_method_name = Self::compute_visit_method_name(
            proto_type,
            is_repeated,
            is_primitive,
            base_type_name,
        );

        let visitor_trait = Self::compute_visitor_trait(
            proto_type,
            is_repeated,
            is_primitive,
            base_type_name,
            qualifier,
        );

        let visitable_trait = Self::compute_visitable_trait(
            proto_type,
            is_repeated,
            is_primitive,
            base_type_name,
            qualifier,
        );

        (
            visitor_param_name,
            visit_method_name,
            visitor_trait,
            visitable_trait,
        )
    }
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
                let is_primitive = Self::is_primitive(field);

                // Process type information
                let inner_type = if is_optional || is_repeated {
                    match Self::extract_inner_type(&field.ty) {
                        Some(inner) => inner,
                        None => field.ty.clone(), // Fallback to original type instead of panicking
                    }
                } else {
                    field.ty.clone()
                };

                // Parse field type overrides
                let (enum_type, as_type) = Self::parse_field_type_overrides(&field_path, &inner_type);

                // Decompose type into base name and qualifier
                let (base_type_name, qualifier) = Self::decompose_type(&inner_type);

                // For repeated fields, full_type_name should be the original Vec type, not the inner type
                let full_type_name = if is_repeated {
                    field.ty.clone()
                } else {
                    inner_type.clone()
                };

                // Parse Prost tag
                let (tag, proto_type) = parse_prost_tag_and_type(field);

                // Compute visitor information
                let (visitor_param_name, visit_method_name, visitor_trait, visitable_trait) =
                    Self::compute_visitor_info(
                        ident,
                        &proto_type,
                        is_repeated,
                        is_primitive,
                        &base_type_name,
                        &qualifier,
                    );

                // Create complete field info
                FieldInfo {
                    ident: ident.clone(),
                    is_param,
                    is_optional,
                    is_repeated,
                    is_message,
                    oneof: oneof.clone(),
                    as_type,
                    enum_type,
                    proto_type,
                    tag,
                    base_type_name,
                    full_type_name,
                    qualifier,
                    visitor_trait,
                    visitable_trait,
                    visitor_param_name,
                    visit_method_name,
                }
            })
            .expect("has field name")
    }

    pub fn related_type(&self, suffix: &str) -> proc_macro2::TokenStream {
        // For Visitor suffix, use precomputed visitor trait
        if suffix == "Visitor" {
            return self.visitor_trait.clone();
        }

        // For Visitable suffix, use precomputed visitable trait
        if suffix == "Visitable" {
            return self.visitable_trait.clone();
        }

        // For other suffixes, use standard trait generation logic
        generate_standard_trait(&self.base_type_name, &self.qualifier, suffix)
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

                    let qualifier_token = quote::quote! { #(#qualifier)::* };
                    (base_type_name, Some(qualifier_token))
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
                .and_then(|segment| match &segment.arguments {
                    syn::PathArguments::AngleBracketed(args) => args
                        .args
                        .first()
                        .and_then(|arg| match arg {
                            syn::GenericArgument::Type(inner_ty) => Some(inner_ty.clone()),
                            _ => None,
                        }),
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

    fn is_primitive(field: &syn::Field) -> bool {
        const PRIMITIVE_PATTERNS: &[&str] = &[
            "bytes=\"vec\"",
            "string",
            "int64",
            "uint32",
            "int32",
            "uint64",
            "bool",
            "double",
            "float",
            "fixed64",
            "fixed32",
            "sfixed64",
            "sfixed32",
            "sint64",
            "sint32",
            "enumeration=",
        ];

        PRIMITIVE_PATTERNS
            .iter()
            .any(|pattern| Self::has_prost_attr(field, pattern))
    }

    fn has_prost_attr(field: &syn::Field, value: &'static str) -> bool {
        field.attrs.iter().any(|attr| {
            attr.path().is_ident("prost") && attr.to_token_stream().to_string().contains(value)
        })
    }

    /// Compute the visit method name for this field
    fn compute_visit_method_name(
        proto_type: &str,
        is_repeated: bool,
        is_primitive: bool,
        base_type_name: &str,
    ) -> syn::Ident {
        let method_name = if proto_type.contains("bytes") {
            "visit_bytes".to_string()
        } else if is_repeated && is_primitive {
            "visit_slice".to_string()
        } else if is_primitive {
            // For non-bytes, non-repeated primitives, use the base type name in lowercase
            format!("visit_{}", base_type_name.to_lowercase())
        } else {
            let type_name = base_type_name;
            format!("visit_{}", type_name.to_case(convert_case::Case::Snake))
        };

        syn::Ident::new(&method_name, proc_macro2::Span::call_site())
    }

    /// Compute the visitor trait for this field
    fn compute_visitor_trait(
        proto_type: &str,
        is_repeated: bool,
        is_primitive: bool,
        base_type_name: &str,
        qualifier: &Option<proc_macro2::TokenStream>,
    ) -> proc_macro2::TokenStream {
        if proto_type.contains("bytes") {
            return quote! { crate::pdata::BytesVisitor<Argument> };
        }

        // For repeated primitive fields, use SliceVisitor with the inner primitive type
        if is_repeated && is_primitive {
            return get_repeated_primitive_trait_mapping(base_type_name, "Visitor")
                .unwrap_or_else(|| {
                    // Unknown repeated primitive type - use the standard path logic
                    generate_standard_trait(base_type_name, qualifier, "Visitor")
                });
        }

        // For direct types (both primitives and non-primitives), use type-specific visitors
        get_primitive_trait_mapping(base_type_name, "Visitor")
            .unwrap_or_else(|| {
                // For non-primitive types, use the standard logic
                generate_standard_trait(base_type_name, qualifier, "Visitor")
            })
    }

    /// Compute the visitable trait for this field
    fn compute_visitable_trait(
        proto_type: &str,
        is_repeated: bool,
        is_primitive: bool,
        base_type_name: &str,
        qualifier: &Option<proc_macro2::TokenStream>,
    ) -> proc_macro2::TokenStream {
        if proto_type.contains("bytes") {
            return quote! { crate::pdata::BytesVisitable<Argument> };
        }

        // For repeated primitive fields, use SliceVisitable with the inner primitive type
        if is_repeated && is_primitive {
            return get_repeated_primitive_trait_mapping(base_type_name, "Visitable")
                .unwrap_or_else(|| quote! { crate::pdata::UnknownVisitable<Argument> });
        }

        // For direct types (both primitives and non-primitives), use type-specific visitables
        get_primitive_trait_mapping(base_type_name, "Visitable")
            .unwrap_or_else(|| {
                // For non-primitive types, use the standard logic
                generate_standard_trait(base_type_name, qualifier, "Visitable")
            })
    }

    /// Generate visitor trait for a oneof case based on its case name
    pub fn generate_visitor_type_for_oneof_case(case: &OneofCase) -> proc_macro2::TokenStream {
        match case.name.as_ref() {
            // Primitive types
            "string" => quote! { crate::pdata::StringVisitor<Argument> },
            "bool" => quote! { crate::pdata::BooleanVisitor<Argument> },
            "int" => quote! { crate::pdata::I64Visitor<Argument> },
            "double" => quote! { crate::pdata::F64Visitor<Argument> },
            "bytes" => quote! { crate::pdata::BytesVisitor<Argument> },

            // Complex message types - use case name to generate visitor
            "kvlist" => quote! { KeyValueListVisitor<Argument> },
            "array" => quote! { ArrayValueVisitor<Argument> },

            // Standard message types - convert to PascalCase and append Visitor
            _ => {
                let visitor_name = format!("{}Visitor", case.name.to_case(Case::Pascal));
                let visitor_ident = syn::Ident::new(&visitor_name, proc_macro2::Span::call_site());
                quote! { #visitor_ident<Argument> }
            }
        }
    }
}
