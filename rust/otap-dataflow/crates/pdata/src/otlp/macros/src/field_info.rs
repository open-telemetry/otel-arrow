// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Field analysis and metadata extraction for OTLP code generation.

use otap_df_pdata_otlp_model::OneofCase;
use otap_df_pdata_otlp_model::OneofMapping;
use quote::ToTokens;

/// Comprehensive information about a struct field for OTLP code generation.
///
/// This struct contains all the metadata needed to generate protobuf-compatible
/// Rust code from struct fields, type information Protocol Buffer attributes.
///
/// Note the #[allow(dead_code)] stem from the removal of a Prost
/// object visitor pattern, item counter, and OTLP bytes encoder
/// feature. TODO PR NUMBER
#[derive(Clone, Debug)]
pub struct FieldInfo {
    /// The Rust identifier for this field (e.g., `field_name`)
    pub ident: syn::Ident,

    /// Whether this field is a required parameter in the builder pattern's `new()` method.
    /// Used to distinguish between obligatory constructor parameters and optional builder methods.
    pub is_param: bool,

    /// Whether this field is wrapped in `Option<T>` (corresponds to `optional` in protobuf)
    pub is_optional: bool,

    /// Whether this field is a repeated field, typically `Vec<T>` (corresponds to `repeated` in protobuf)
    pub is_repeated: bool,

    /// Whether this field represents a primitive type at the protocol level, including bytes,
    /// String, integers, etc.
    #[allow(dead_code)]
    pub is_primitive: bool,

    /// Whether it uses a 32- or 64-bit fixed size encoding.
    #[allow(dead_code)]
    pub is_fixed: bool,

    /// For oneof fields: contains the possible cases that this field can represent.
    /// `None` for regular fields, `Some(cases)` for oneof union types.
    pub oneof: Option<Vec<OneofCase>>,

    /// For enum fields: the primitive type that the enum maps to (e.g., `i32` for enum values)
    pub as_type: Option<syn::Type>,

    /// For enum fields: the actual enum type from FIELD_TYPE_OVERRIDES configuration.
    /// This allows custom enum types to be used instead of the default protobuf mapping.
    pub enum_type: Option<syn::Type>,

    /// The protobuf field type as a string (e.g., "string", "int64", "message", "bytes")
    pub proto_type: String,

    /// Module path qualifier for the field's type (e.g., `some::module` for `some::module::TypeName`).
    /// Used to generate proper trait implementations with correct namespacing.
    #[allow(dead_code)]
    pub qualifier: Option<proc_macro2::TokenStream>,

    /// The protobuf field tag number for wire format serialization
    #[allow(dead_code)]
    pub tag: u32,

    /// The base type name without any module qualification (e.g., "String", "MyMessage")
    pub base_type_name: String,

    /// The complete Rust type for this field, including containers like `Option<T>` or `Vec<T>`
    pub full_type_name: syn::Type,
}

/// Helper function to parse protobuf tag values from prost attribute strings.
///
/// Handles different formatting patterns like `tag=123` and `tag = 123` that may
/// appear in prost field annotations. Returns the numeric tag value if found.
fn parse_tag_value(attr_str: &str) -> Option<u32> {
    // Try different tag patterns
    let patterns = [("tag=", 4), ("tag = ", 6)];

    for (pattern, offset) in &patterns {
        if let Some(result) = attr_str.find(pattern).and_then(|tag_start| {
            let tag_part = &attr_str[tag_start + offset..];
            let tag_value = if let Some(comma_pos) = tag_part.find(',') {
                &tag_part[..comma_pos]
            } else {
                tag_part
            };

            tag_value.trim().trim_matches('"').parse().ok()
        }) {
            return Some(result);
        }
    }

    None
}

/// Extracts protobuf tag number and type information from prost field attributes.
///
/// Parses `#[prost(type, tag = "number")]` annotations to extract:
/// - The tag number for wire format serialization
/// - The protobuf type (string, int64, message, etc.)
///
/// Returns (tag, proto_type) with defaults (0, "unknown") if parsing fails.
/// This graceful fallback helps with debugging by allowing more fields to be processed.
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

            // Parse tag number using helper function
            let tag = parse_tag_value(&attr_str).unwrap_or(0);

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

impl FieldInfo {
    /// Parse field type overrides and return (enum_type, as_type)
    fn parse_field_type_overrides(
        field_path: &str,
        _inner_type: &syn::Type,
    ) -> (Option<syn::Type>, Option<syn::Type>) {
        otap_df_pdata_otlp_model::FIELD_TYPE_OVERRIDES
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

    /// Creates a new FieldInfo instance by analyzing a struct field and its context.
    ///
    /// Performs comprehensive analysis of the field including:
    /// - Parsing prost annotations for protobuf information
    /// - Determining field characteristics (optional, repeated, message type)
    /// - Resolving type information and qualifiers
    /// - Applying field type overrides from configuration
    ///
    /// # Arguments
    /// * `field` - The syn::Field to analyze
    /// * `type_name` - Name of the containing struct/type
    /// * `param_names` - Fields that should be constructor parameters
    /// * `oneof_mapping` - Mapping for oneof union field information
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
                let is_primitive = Self::is_primitive(field);
                let is_fixed = Self::is_fixed(field);

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
                let (enum_type, as_type) =
                    Self::parse_field_type_overrides(&field_path, &inner_type);

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

                // Create complete field info
                FieldInfo {
                    ident: ident.clone(),
                    is_param,
                    is_optional,
                    is_repeated,
                    is_primitive,
                    is_fixed,
                    oneof: oneof.clone(),
                    as_type,
                    enum_type,
                    proto_type,
                    tag,
                    base_type_name,
                    full_type_name,
                    qualifier,
                }
            })
            .expect("has field name")
    }

    /// Decomposes a Rust type into its base name and module qualifier.
    ///
    /// For example:
    /// - `String` -> ("String", None)
    /// - `std::string::String` -> ("String", Some(quote! { std::string }))
    /// - `my::module::CustomType` -> ("CustomType", Some(quote! { my::module }))
    ///
    /// This separation enables proper trait generation with correct namespacing
    /// while maintaining the base type name for method and trait naming conventions.
    ///
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

    /// Extracts the inner type from generic container types like `Option<T>` or `Vec<T>`.
    ///
    /// This is essential for processing optional and repeated fields where we need
    /// to understand the actual data type being contained. For example:
    /// - `Option<String>` -> `String`
    /// - `Vec<MyMessage>` -> `MyMessage`
    ///
    /// Returns None if the type is not a recognized generic container.
    fn extract_inner_type(ty: &syn::Type) -> Option<syn::Type> {
        match ty {
            syn::Type::Path(type_path) => {
                type_path
                    .path
                    .segments
                    .last()
                    .and_then(|segment| match &segment.arguments {
                        syn::PathArguments::AngleBracketed(args) => {
                            args.args.first().and_then(|arg| match arg {
                                syn::GenericArgument::Type(inner_ty) => Some(inner_ty.clone()),
                                _ => None,
                            })
                        }
                        _ => None,
                    })
            }
            _ => None,
        }
    }

    /// Generic helper to check if a field has a prost attribute with specific patterns
    fn has_prost_attr(field: &syn::Field, value: &str) -> bool {
        field.attrs.iter().any(|attr| {
            attr.path().is_ident("prost") && attr.to_token_stream().to_string().contains(value)
        })
    }

    /// Check multiple prost attribute patterns at once
    fn has_any_prost_attr(field: &syn::Field, patterns: &[&str]) -> bool {
        patterns
            .iter()
            .any(|pattern| Self::has_prost_attr(field, pattern))
    }

    fn is_optional(field: &syn::Field) -> bool {
        Self::has_prost_attr(field, "optional")
    }

    fn is_fixed(field: &syn::Field) -> bool {
        const FIXED_PATTERNS: &[&str] = &["fixed64", "fixed32", "sfixed64", "sfixed32"];

        Self::has_any_prost_attr(field, FIXED_PATTERNS)
    }

    fn is_repeated(field: &syn::Field) -> bool {
        Self::has_prost_attr(field, "repeated")
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

        Self::has_any_prost_attr(field, PRIMITIVE_PATTERNS)
    }
}
