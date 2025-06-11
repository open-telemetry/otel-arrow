// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Field analysis and metadata extraction for OTLP code generation.
//!
//! This module provides the `FieldInfo` struct and associated utilities for analyzing
//! struct fields in the context of OpenTelemetry Protocol (OTLP) code generation.
//! It handles the complex task of extracting protobuf metadata, type information,
//! and visitor pattern details from Rust struct fields.
//!
//! ## Key Responsibilities
//!
//! - **Protobuf Analysis**: Parses `#[prost(...)]` attributes to extract tag numbers,
//!   field types, and protobuf-specific characteristics (optional, repeated, message)
//! - **Type Decomposition**: Breaks down complex Rust types into base names and module
//!   qualifiers for proper trait generation and namespacing
//! - **Visitor Pattern Support**: Precomputes visitor and visitable trait information
//!   to support the visitor pattern used throughout the OTLP implementation
//! - **Field Type Overrides**: Applies custom type mappings from configuration to
//!   handle special cases like enum types
//! - **Oneof Field Handling**: Manages protobuf oneof (union) fields and their
//!   various case types
//!
//! ## Usage
//!
//! The primary entry point is `FieldInfo::new()`, which takes a `syn::Field` and
//! contextual information to produce a comprehensive `FieldInfo` instance containing
//! all metadata needed for code generation.

use convert_case::{Case, Casing};
use otlp_model::OneofCase;
use otlp_model::OneofMapping;
use quote::{ToTokens, quote};

/// Comprehensive information about a struct field for OTLP code generation.
///
/// This struct contains all the metadata needed to generate protobuf-compatible
/// Rust code from struct fields, including visitor patterns, type information,
/// and Protocol Buffer attributes.
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

    /// Whether this field represents a message type (complex nested structure) vs primitive type
    pub is_message: bool,

    /// Whether this field represents a primitive type at the protocol level, including bytes,
    /// String, integers, etc.
    pub is_primitive: bool,

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
    pub qualifier: Option<proc_macro2::TokenStream>,

    /// The protobuf field tag number for wire format serialization
    pub tag: u32,

    /// The base type name without any module qualification (e.g., "String", "MyMessage")
    pub base_type_name: String,

    /// The complete Rust type for this field, including containers like `Option<T>` or `Vec<T>`
    pub full_type_name: syn::Type,

    // Visitor pattern precomputed information for efficient code generation
    /// The visitor trait that should be implemented for this field type.
    /// Precomputed to handle special cases like primitives, repeated fields, and bytes.
    pub visitor_trait: proc_macro2::TokenStream,

    /// The visitable trait that this field type should implement.
    /// Precomputed for consistency with the corresponding visitor trait.
    pub visitable_trait: proc_macro2::TokenStream,

    /// Parameter name for the visitor in generated code (e.g., `field_name_visitor`)
    pub visitor_param_name: syn::Ident,

    /// Method name for the visit call in generated code (e.g., `visit_string`, `visit_message`)
    pub visit_method_name: syn::Ident,
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

/// Type information for primitive types in the visitor pattern.
///
/// Maps primitive Rust types to their corresponding trait names used in
/// the visitor pattern code generation (e.g., "String" -> "StringVisitor").
struct PrimitiveTypeInfo {
    trait_name: &'static str,
    rust_type: &'static str,
}

/// Mapping of base type names to their trait and Rust type information
fn get_primitive_type_info(base_type_name: &str) -> Option<PrimitiveTypeInfo> {
    match base_type_name {
        "String" => Some(PrimitiveTypeInfo {
            trait_name: "String",
            rust_type: "String",
        }),
        "bool" => Some(PrimitiveTypeInfo {
            trait_name: "Boolean",
            rust_type: "bool",
        }),
        "i32" => Some(PrimitiveTypeInfo {
            trait_name: "I32",
            rust_type: "i32",
        }),
        "i64" => Some(PrimitiveTypeInfo {
            trait_name: "I64",
            rust_type: "i64",
        }),
        "u32" | "u8" => Some(PrimitiveTypeInfo {
            trait_name: "U32",
            rust_type: "u32",
        }),
        "u64" => Some(PrimitiveTypeInfo {
            trait_name: "U64",
            rust_type: "u64",
        }),
        "f32" | "f64" => Some(PrimitiveTypeInfo {
            trait_name: "F64",
            rust_type: "f64",
        }),
        _ => None,
    }
}

/// Mapping for primitive types to their corresponding trait names
fn get_primitive_trait_mapping(
    base_type_name: &str,
    trait_suffix: &str,
) -> Option<proc_macro2::TokenStream> {
    get_primitive_type_info(base_type_name).map(|info| {
        let trait_name = format!("{}{}", info.trait_name, trait_suffix);
        let trait_ident = syn::Ident::new(&trait_name, proc_macro2::Span::call_site());
        quote! { crate::pdata::#trait_ident<Argument> }
    })
}

/// Mapping for repeated primitive types to their corresponding slice trait names
fn get_repeated_primitive_trait_mapping(
    base_type_name: &str,
    trait_suffix: &str,
) -> Option<proc_macro2::TokenStream> {
    get_primitive_type_info(base_type_name).map(|info| {
        let slice_trait_name = format!("Slice{}", trait_suffix);
        let slice_trait_ident = syn::Ident::new(&slice_trait_name, proc_macro2::Span::call_site());
        let rust_type_ident = syn::Ident::new(info.rust_type, proc_macro2::Span::call_site());
        quote! { crate::pdata::#slice_trait_ident<Argument, #rust_type_ident> }
    })
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

        let visit_method_name =
            Self::compute_visit_method_name(proto_type, is_repeated, is_primitive, base_type_name);

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
    /// Creates a new FieldInfo instance by analyzing a struct field and its context.
    ///
    /// Performs comprehensive analysis of the field including:
    /// - Parsing prost annotations for protobuf information
    /// - Determining field characteristics (optional, repeated, message type)
    /// - Resolving type information and qualifiers
    /// - Computing visitor pattern traits and method names
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
                    is_primitive,
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

    /// Generates the appropriate trait token stream for related types with the given suffix.
    ///
    /// This method handles special cases for "Visitor" and "Visitable" suffixes by using
    /// precomputed trait information, while falling back to standard trait generation
    /// for other suffixes. This ensures consistent trait naming and proper handling
    /// of primitive types, repeated fields, and complex message types.
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

        Self::has_any_prost_attr(field, PRIMITIVE_PATTERNS)
    }

    /// Determines the appropriate visit method name for this field in the visitor pattern.
    ///
    /// Generates method names based on the protobuf type and field characteristics:
    /// - Bytes fields -> "visit_bytes"
    /// - Repeated primitives -> "visit_slice"  
    /// - Single primitives -> "visit_{type}" (e.g., "visit_string", "visit_i32")
    /// - Messages -> "visit_{snake_case_name}" (e.g., "visit_my_message")
    ///
    /// This naming convention ensures consistency across the generated visitor code.
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

    /// Unified trait computation for both Visitor and Visitable traits.
    ///
    /// Handles special cases and mappings:
    /// - Bytes fields -> BytesVisitor/BytesVisitable  
    /// - Repeated primitives -> Slice{Type}Visitor/Slice{Type}Visitable
    /// - Single primitives -> {Type}Visitor/{Type}Visitable (e.g., StringVisitor)
    /// - Messages -> {TypeName}Visitor/{TypeName}Visitable
    ///
    /// Uses precomputed mappings for primitives and falls back to standard
    /// trait generation for custom message types.
    fn compute_trait(
        proto_type: &str,
        is_repeated: bool,
        is_primitive: bool,
        base_type_name: &str,
        qualifier: &Option<proc_macro2::TokenStream>,
        trait_suffix: &str,
    ) -> proc_macro2::TokenStream {
        // Handle bytes type specially
        if proto_type.contains("bytes") {
            let bytes_trait = format!("Bytes{}", trait_suffix);
            let bytes_trait_ident = syn::Ident::new(&bytes_trait, proc_macro2::Span::call_site());
            return quote! { crate::pdata::#bytes_trait_ident<Argument> };
        }

        // For repeated primitive fields, use slice traits
        if is_repeated && is_primitive {
            return get_repeated_primitive_trait_mapping(base_type_name, trait_suffix)
                .unwrap_or_else(|| {
                    // Fallback for unknown repeated primitive types
                    if trait_suffix == "Visitable" {
                        quote! { crate::pdata::UnknownVisitable<Argument> }
                    } else {
                        generate_standard_trait(base_type_name, qualifier, trait_suffix)
                    }
                });
        }

        // For primitive types, use type-specific traits
        get_primitive_trait_mapping(base_type_name, trait_suffix).unwrap_or_else(|| {
            // For non-primitive types, use standard trait generation
            generate_standard_trait(base_type_name, qualifier, trait_suffix)
        })
    }

    /// Compute the visitor trait for this field
    fn compute_visitor_trait(
        proto_type: &str,
        is_repeated: bool,
        is_primitive: bool,
        base_type_name: &str,
        qualifier: &Option<proc_macro2::TokenStream>,
    ) -> proc_macro2::TokenStream {
        Self::compute_trait(
            proto_type,
            is_repeated,
            is_primitive,
            base_type_name,
            qualifier,
            "Visitor",
        )
    }

    /// Compute the visitable trait for this field
    fn compute_visitable_trait(
        proto_type: &str,
        is_repeated: bool,
        is_primitive: bool,
        base_type_name: &str,
        qualifier: &Option<proc_macro2::TokenStream>,
    ) -> proc_macro2::TokenStream {
        Self::compute_trait(
            proto_type,
            is_repeated,
            is_primitive,
            base_type_name,
            qualifier,
            "Visitable",
        )
    }

    /// Generates the appropriate visitor trait for a specific oneof case.
    ///
    /// Oneof fields in protobuf represent union types where a field can be one of
    /// several possible types. This method maps oneof case names to their corresponding
    /// visitor traits, handling both primitive types (which use crate::pdata traits)
    /// and complex message types (which use unqualified or standard naming).
    ///
    /// Examples:
    /// - "string" case -> `crate::pdata::StringVisitor<Argument>`
    /// - "kvlist" case -> `KeyValueListVisitor<Argument>`  
    /// - "custom" case -> `CustomVisitor<Argument>`
    pub fn generate_visitor_type_for_oneof_case(case: &OneofCase) -> proc_macro2::TokenStream {
        /// Mapping of oneof case names to their visitor traits, considering proto_type
        fn get_oneof_visitor_mapping(
            case_name: &str,
            proto_type: &str,
        ) -> Option<(&'static str, bool)> {
            match case_name {
                // Primitive types (in crate::pdata) - consider proto_type for encoding differences
                "string" => Some(("StringVisitor", true)),
                "bool" => Some(("BooleanVisitor", true)),
                "int" => {
                    // Choose visitor based on protobuf wire type
                    match proto_type {
                        "sfixed64" => Some(("I64Visitor", true)), // Will use Sfixed64EncodedLen via encoder selection
                        "int64" | _ => Some(("I64Visitor", true)), // Will use I64EncodedLen via encoder selection
                    }
                }
                "double" => Some(("F64Visitor", true)),
                "bytes" => Some(("BytesVisitor", true)),

                // Complex message types (unqualified)
                "kvlist" => Some(("KeyValueListVisitor", false)),
                "array" => Some(("ArrayValueVisitor", false)),

                _ => None,
            }
        }

        if let Some((visitor_name, use_crate_pdata)) =
            get_oneof_visitor_mapping(&case.name, &case.proto_type)
        {
            let visitor_ident = syn::Ident::new(visitor_name, proc_macro2::Span::call_site());
            if use_crate_pdata {
                quote! { crate::pdata::#visitor_ident<Argument> }
            } else {
                quote! { #visitor_ident<Argument> }
            }
        } else {
            // Standard message types - convert to PascalCase and append Visitor
            let visitor_name = format!("{}Visitor", case.name.to_case(Case::Pascal));
            let visitor_ident = syn::Ident::new(&visitor_name, proc_macro2::Span::call_site());
            quote! { #visitor_ident<Argument> }
        }
    }
}
