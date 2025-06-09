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
    pub is_primitive: bool, // Includes basic types (String, u32, bool) AND bytes (Vec<u8>)
    pub is_message: bool,
    pub oneof: Option<Vec<OneofCase>>,
    pub as_type: Option<syn::Type>, // primitive type for enums
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
    pub needs_adapter: bool,
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

            // Parse tag number from "tag = \"1\"" or "tag = 1" or "tag=\"1\""
            if let Some(tag_start) = attr_str.find("tag=") {
                let tag_part = &attr_str[tag_start + 4..];
                if let Some(comma_pos) = tag_part.find(',') {
                    let tag_value = &tag_part[..comma_pos].trim().trim_matches('"');
                    tag = tag_value
                        .parse()
                        .expect(&format!("Failed to parse tag number: {}", tag_value));
                } else {
                    let tag_value = tag_part.trim().trim_matches('"');
                    tag = tag_value
                        .parse()
                        .expect(&format!("Failed to parse tag number: {}", tag_value));
                }
            } else if let Some(tag_start) = attr_str.find("tag = ") {
                let tag_part = &attr_str[tag_start + 6..];
                if let Some(comma_pos) = tag_part.find(',') {
                    let tag_value = &tag_part[..comma_pos].trim().trim_matches('"');
                    tag = tag_value
                        .parse()
                        .expect(&format!("Failed to parse tag number: {}", tag_value));
                } else {
                    let tag_value = tag_part.trim().trim_matches('"');
                    tag = tag_value
                        .parse()
                        .expect(&format!("Failed to parse tag number: {}", tag_value));
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

    // Return default values instead of panicking, which helps us see more of what's happening
    (0, "unknown".to_string())
}

impl FieldInfo {
    pub(crate)    fn new(
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

                // TODO: field_type should be used

                // Use a safer approach with try_fold to handle errors
                let result: Result<(syn::Type, Option<syn::Type>), String> =
                    otlp_model::FIELD_TYPE_OVERRIDES
                        .get(field_path.as_str())
                        .map(|over| {
                            // Parse datatype with better error handling
                            let datatype = match syn::parse_str::<syn::Type>(over.datatype) {
                                Ok(dt) => dt,
                                Err(e) => {
                                    return Err(format!(
                                        "Failed to parse datatype '{}' for field {}: {}",
                                        over.datatype, field_path, e
                                    ));
                                }
                            };

                            // Parse fieldtype with better error handling
                            let fieldtype = match syn::parse_str::<syn::Type>(over.fieldtype) {
                                Ok(ft) => Some(ft),
                                Err(e) => {
                                    return Err(format!(
                                        "Failed to parse fieldtype '{}' for field {}: {}",
                                        over.fieldtype, field_path, e
                                    ));
                                }
                            };

                            Ok((datatype, fieldtype))
                        })
                        .unwrap_or_else(|| Ok((inner_type.clone(), None)));

                // Handle any errors from the parsing
                let (enum_type, as_type) = match result {
                    Ok(types) => {
                        let (datatype, fieldtype) = types;
                        // If we have an override, store the enum type
                        if fieldtype.is_some() {
                            (Some(datatype), fieldtype)
                        } else {
                            (None, None)
                        }
                    },
                    Err(_err) => {
                        // Fallback to inner_type on error
                        (None, None)
                    }
                };

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

                // Create field info first without visitor info
                let mut field_info = FieldInfo {
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
                    visitor_trait: quote::quote! {},
                    visitable_trait: quote::quote! {},
                    visitor_param_name: syn::Ident::new("placeholder", proc_macro2::Span::call_site()),
                    visit_method_name: syn::Ident::new("placeholder", proc_macro2::Span::call_site()),
                    needs_adapter: false,
                };

                // Compute and store visitor-related information
                field_info.compute_visitor_info();
                field_info
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

        // For Adapter suffix, check if this is a primitive type that doesn't need adapters
        if suffix == "Adapter" {
            // Primitive types (including bytes/Vec<u8>) don't need adapters
            if self.is_primitive
                || self.proto_type == "bytes"
                || matches!(
                    self.proto_type.as_str(),
                    "string"
                        | "int64"
                        | "uint32"
                        | "int32"
                        | "uint64"
                        | "bool"
                        | "double"
                        | "float"
                )
            {
                // Return empty token stream - this should not be called for primitive types
                // The caller should check needs_adapter first
                return quote::quote! { /* PRIMITIVE_NO_ADAPTER */ };
            }
        }

        // For other suffixes, use original logic
        if let Some(ref qualifier) = self.qualifier {
            let base_with_suffix = syn::Ident::new(
                &format!("{}{}", self.base_type_name, suffix),
                proc_macro2::Span::call_site(),
            );
            quote::quote! { #qualifier::#base_with_suffix }
        } else {
            let type_with_suffix = syn::Ident::new(
                &format!("{}{}", self.base_type_name, suffix),
                proc_macro2::Span::call_site(),
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
        let result = match ty {
            syn::Type::Path(type_path) => {
                if let Some(last_segment) = type_path.path.segments.last() {
                    match &last_segment.arguments {
                        syn::PathArguments::AngleBracketed(args) => {
                            if let Some(first_arg) = args.args.first() {
                                match first_arg {
                                    syn::GenericArgument::Type(inner_ty) => {
                                        Some(inner_ty.clone())
                                    }
                                    _ => {
                                        None
                                    }
                                }
                            } else {
                                None
                            }
                        }
                        _ => {
                            None
                        }
                    }
                } else {
                    None
                }
            }
            _ => {
                None
            }
        };

        result
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
        // Primitive includes basic protobuf types AND bytes (Vec<u8>)
        Self::has_prost_attr(field, "bytes=\"vec\"")
            || Self::has_prost_attr(field, "string")
            || Self::has_prost_attr(field, "int64")
            || Self::has_prost_attr(field, "uint32")
            || Self::has_prost_attr(field, "int32")
            || Self::has_prost_attr(field, "uint64")
            || Self::has_prost_attr(field, "bool")
            || Self::has_prost_attr(field, "double")
            || Self::has_prost_attr(field, "float")
            // Fixed-size primitive types
            || Self::has_prost_attr(field, "fixed64")
            || Self::has_prost_attr(field, "fixed32")
            || Self::has_prost_attr(field, "sfixed64")
            || Self::has_prost_attr(field, "sfixed32")
            // Alternative protobuf type names
            || Self::has_prost_attr(field, "sint64")
            || Self::has_prost_attr(field, "sint32")
    }

    fn has_prost_attr(field: &syn::Field, value: &'static str) -> bool {
        field.attrs.iter().any(|attr| {
            attr.path().is_ident("prost") && attr.to_token_stream().to_string().contains(value)
        })
    }

    /// Compute and store all visitor-related information for this field
    fn compute_visitor_info(&mut self) {
        // Generate visitor parameter name
        let field_name_str = self.ident.to_string();
        let clean_field_name = if field_name_str.starts_with("r#") {
            &field_name_str[2..]
        } else {
            &field_name_str
        };
        self.visitor_param_name = syn::Ident::new(
            &format!("{}_visitor", clean_field_name),
            proc_macro2::Span::call_site(),
        );

        // Generate visit method name
        self.visit_method_name = self.compute_visit_method_name();

        // Determine if field needs adapter
        self.needs_adapter = self.compute_needs_adapter();

        // Generate visitor and visitable traits
        self.visitor_trait = self.compute_visitor_trait();
        self.visitable_trait = self.compute_visitable_trait();
    }

    /// Compute the visit method name for this field
    fn compute_visit_method_name(&self) -> syn::Ident {
        let method_name = if self.proto_type == "bytes" || (self.base_type_name == "Vec" && self.proto_type.contains("bytes")) {
            // CASE 1: Bytes field (Vec<u8>) - always use visit_bytes regardless of repetition
            "visit_bytes".to_string()
        } else if self.is_repeated && self.is_primitive {
            // CASE 2: Repeated primitive field (Vec<u64>, Vec<f64>, etc.) but NOT bytes
            // These should use SliceVisitor with visit_vec method
            "visit_vec".to_string()
        } else if self.is_primitive_type_direct() {
            // CASE 3: Direct primitive type (u64, f64, String, etc.)
            let suffix = self.get_primitive_method_suffix();
            format!("visit_{}", suffix)
        } else {
            // CASE 4: Message types or other non-primitive types
            let type_name = &self.base_type_name;
            format!("visit_{}", type_name.to_lowercase())
        };
        
        syn::Ident::new(&method_name, proc_macro2::Span::call_site())
    }

    /// Compute whether this field needs an adapter
    fn compute_needs_adapter(&self) -> bool {
        // Use the parsed proto_type for fast determination
        match self.proto_type.as_str() {
            "message" | "enumeration" => true, // Complex types need adapters
            "string" | "int64" | "uint32" | "int32" | "uint64" | "bool" | "double" | "float"
            | "bytes" => false, // Primitives don't need adapters
            _ => {
                // For unknown or missing proto_type, fall back to type-based analysis
                // This ensures backward compatibility and handles edge cases
                !self.is_primitive_type_direct() && !self.is_primitive
            }
        }
    }

    /// Compute the visitor trait for this field
    fn compute_visitor_trait(&self) -> proc_macro2::TokenStream {
        // Special handling for Vec<u8> (bytes) - check proto_type first
        if self.proto_type == "bytes" || (self.base_type_name == "Vec" && self.proto_type.contains("bytes")) {
            return quote! { crate::pdata::BytesVisitor<Argument> };
        }

        // For repeated primitive fields, use SliceVisitor with the inner primitive type
        if self.is_repeated && self.is_primitive {
            return match self.base_type_name.as_str() {
                "String" => quote! { crate::pdata::SliceVisitor<Argument, String> },
                "bool" => quote! { crate::pdata::SliceVisitor<Argument, bool> },
                "i32" => quote! { crate::pdata::SliceVisitor<Argument, i32> },
                "i64" => quote! { crate::pdata::SliceVisitor<Argument, i64> },
                "u32" | "u8" => quote! { crate::pdata::SliceVisitor<Argument, u32> },
                "u64" => quote! { crate::pdata::SliceVisitor<Argument, u64> },
                "f32" => quote! { crate::pdata::SliceVisitor<Argument, f32> },
                "f64" => quote! { crate::pdata::SliceVisitor<Argument, f64> },
                _ => {
                    // Unknown repeated primitive type - use the standard path logic
                    self.generate_standard_visitor_trait()
                }
            };
        }

        match self.base_type_name.as_str() {
            "String" => quote! { crate::pdata::StringVisitor<Argument> },
            "bool" => quote! { crate::pdata::BooleanVisitor<Argument> },
            "i32" => quote! { crate::pdata::I32Visitor<Argument> },
            "i64" => quote! { crate::pdata::I64Visitor<Argument> },
            "u32" | "u8" => quote! { crate::pdata::U32Visitor<Argument> },
            "u64" => quote! { crate::pdata::U64Visitor<Argument> },
            "f32" | "f64" => quote! { crate::pdata::F64Visitor<Argument> },
            "Vec" => quote! { crate::pdata::SliceVisitor<Argument, u8> }, // This should rarely be used after bytes check above
            _ => {
                // For non-primitive types, use the standard logic
                self.generate_standard_visitor_trait()
            }
        }
    }

    /// Generate standard visitor trait for non-primitive types
    fn generate_standard_visitor_trait(&self) -> proc_macro2::TokenStream {
        if let Some(ref qualifier) = self.qualifier {
            let base_with_suffix = syn::Ident::new(
                &format!("{}Visitor", self.base_type_name),
                proc_macro2::Span::call_site(),
            );
            quote! { #qualifier::#base_with_suffix<Argument> }
        } else {
            let type_with_suffix = syn::Ident::new(
                &format!("{}Visitor", self.base_type_name),
                proc_macro2::Span::call_site(),
            );
            quote! { #type_with_suffix<Argument> }
        }
    }

    /// Compute the visitable trait for this field
    fn compute_visitable_trait(&self) -> proc_macro2::TokenStream {
        // Special handling for Vec<u8> (bytes) - check proto_type first
        if self.proto_type == "bytes" || (self.base_type_name == "Vec" && self.proto_type.contains("bytes")) {
            return quote! { crate::pdata::BytesVisitable<Argument> };
        }

        // For repeated primitive fields, use SliceVisitable with the inner primitive type
        if self.is_repeated && self.is_primitive {
            return match self.base_type_name.as_str() {
                "String" => quote! { crate::pdata::SliceVisitable<Argument, String> },
                "bool" => quote! { crate::pdata::SliceVisitable<Argument, bool> },
                "i32" => quote! { crate::pdata::SliceVisitable<Argument, i32> },
                "i64" => quote! { crate::pdata::SliceVisitable<Argument, i64> },
                "u32" | "u8" => quote! { crate::pdata::SliceVisitable<Argument, u32> },
                "u64" => quote! { crate::pdata::SliceVisitable<Argument, u64> },
                "f32" => quote! { crate::pdata::SliceVisitable<Argument, f32> },
                "f64" => quote! { crate::pdata::SliceVisitable<Argument, f64> },
                _ => quote! { crate::pdata::UnknownVisitable<Argument> }
            };
        }

        match self.base_type_name.as_str() {
            "String" => quote! { crate::pdata::StringVisitable<Argument> },
            "bool" => quote! { crate::pdata::BooleanVisitable<Argument> },
            "i32" => quote! { crate::pdata::I32Visitable<Argument> },
            "i64" => quote! { crate::pdata::I64Visitable<Argument> },
            "u32" | "u8" => quote! { crate::pdata::U32Visitable<Argument> },
            "u64" => quote! { crate::pdata::U64Visitable<Argument> },
            "f32" | "f64" => quote! { crate::pdata::F64Visitable<Argument> },
            "Vec" => quote! { crate::pdata::SliceVisitable<Argument, u8> }, // This should rarely be used after bytes check above
            _ => {
                // For non-primitive types, use the standard logic
                if let Some(ref qualifier) = self.qualifier {
                    let base_with_suffix = syn::Ident::new(
                        &format!("{}Visitable", self.base_type_name),
                        proc_macro2::Span::call_site(),
                    );
                    quote! { #qualifier::#base_with_suffix<Argument> }
                } else {
                    let type_with_suffix = syn::Ident::new(
                        &format!("{}Visitable", self.base_type_name),
                        proc_macro2::Span::call_site(),
                    );
                    quote! { #type_with_suffix<Argument> }
                }
            }
        }
    }

    /// Check if this field represents a primitive type
    fn is_primitive_type_direct(&self) -> bool {
        match self.base_type_name.as_str() {
            "String" | "bool" | "i32" | "i64" | "u32" | "u64" | "f32" | "f64" | "u8" => true,
            _ => false,
        }
    }

    /// Get the method suffix for primitive types
    fn get_primitive_method_suffix(&self) -> String {
        match self.base_type_name.as_str() {
            "String" => "string".to_string(),
            "u32" => "u32".to_string(),
            "u64" => "u64".to_string(),
            "i32" => "i32".to_string(),
            "i64" => "i64".to_string(),
            "f32" => "f32".to_string(),
            "f64" => "f64".to_string(),
            "bool" => "bool".to_string(),
            name => name.to_lowercase(),
        }
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
                let visitor_name = format!("{}Visitor", Self::to_pascal_case(&case.name));
                let visitor_ident = syn::Ident::new(&visitor_name, proc_macro2::Span::call_site());
                quote! { #visitor_ident<Argument> }
            }
        }
    }

    /// Generate visitor type for a oneof variant using oneof case information
    pub fn generate_visitor_type_for_oneof_variant(case: &OneofCase) -> proc_macro2::TokenStream {
        // For oneof variants, we need to determine the type from the case
        if let Ok(case_type) = syn::parse_str::<syn::Type>(&case.type_param) {
            if Self::needs_adapter_for_type(&case_type) {
                // Generate the visitor trait name from the case type
                let type_name = Self::get_base_type_name_from_type(&case_type);
                let visitor_trait_name = format!("{}Visitor", type_name);
                
                // For Vec types that need adapters, these are typically bytes
                if type_name == "Vec" {
                    quote! { crate::pdata::BytesVisitor<Argument> }
                } else {
                    // For message types, try unqualified first, then fallback
                    let visitor_ident = syn::Ident::new(&visitor_trait_name, proc_macro2::Span::call_site());
                    quote! { #visitor_ident<Argument> }
                }
            } else {
                // For primitive types
                if Self::is_bytes_type(&case_type) {
                    quote! { crate::pdata::BytesVisitor<Argument> }
                } else {
                    let base_type = Self::get_base_type_name_from_type(&case_type);
                    match base_type.as_str() {
                        "String" => quote! { crate::pdata::StringVisitor<Argument> },
                        "bool" => quote! { crate::pdata::BooleanVisitor<Argument> },
                        "i32" => quote! { crate::pdata::I32Visitor<Argument> },
                        "i64" => quote! { crate::pdata::I64Visitor<Argument> },
                        "u32" | "u8" => quote! { crate::pdata::U32Visitor<Argument> },
                        "u64" => quote! { crate::pdata::U64Visitor<Argument> },
                        "f32" | "f64" => quote! { crate::pdata::F64Visitor<Argument> },
                        "Vec" => quote! { crate::pdata::BytesVisitor<Argument> }, // Vec in protobuf context is typically bytes
                        _ => {
                            // For message types, generate the appropriate visitor trait
                            let visitor_trait_name = format!("{}Visitor", base_type);
                            let visitor_ident = syn::Ident::new(&visitor_trait_name, proc_macro2::Span::call_site());
                            quote! { #visitor_ident<Argument> }
                        },
                    }
                }
            }
        } else {
            // If we can't parse the type, use a generic fallback
            let visitor_trait_name = format!("{}Visitor", case.name);
            let visitor_ident = syn::Ident::new(&visitor_trait_name, proc_macro2::Span::call_site());
            quote! { #visitor_ident<Argument> }
        }
    }

    /// Helper function to convert a string to PascalCase
    fn to_pascal_case(s: &str) -> String {
        s.to_case(Case::Pascal)
    }

    /// Check if a type needs an adapter (helper for oneof processing)
    fn needs_adapter_for_type(ty: &syn::Type) -> bool {
        !Self::is_primitive_type_direct_static(ty) && !Self::is_bytes_type(ty)
    }

    /// Check if a type is primitive (static helper for oneof processing)
    fn is_primitive_type_direct_static(ty: &syn::Type) -> bool {
        if let syn::Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.last() {
                matches!(
                    segment.ident.to_string().as_str(),
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
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Check if a type is a bytes type
    pub fn is_bytes_type(ty: &syn::Type) -> bool {
        if let syn::Type::Path(type_path) = ty {
            if let Some(last_segment) = type_path.path.segments.last() {
                match last_segment.ident.to_string().as_str() {
                    "Vec" => {
                        // Check if it's Vec<u8>
                        if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                            if let Some(syn::GenericArgument::Type(syn::Type::Path(inner_path))) =
                                args.args.first()
                            {
                                if let Some(inner_segment) = inner_path.path.segments.last() {
                                    return inner_segment.ident == "u8";
                                }
                            }
                        }
                        false
                    }
                    _ => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Extract the base type name from a syn::Type
    pub fn get_base_type_name_from_type(ty: &syn::Type) -> String {
        match ty {
            syn::Type::Path(type_path) => {
                if let Some(segment) = type_path.path.segments.last() {
                    segment.ident.to_string()
                } else {
                    "unknown".to_string()
                }
            }
            _ => "unknown".to_string(),
        }
    }
}
