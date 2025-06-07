// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

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
    pub proto_type: String,
    pub qualifier: Option<proc_macro2::TokenStream>,

    pub tag: u32,

    pub base_type_name: String,
    pub full_type_name: syn::Type,
}

/// Simple prost field annotation parsing utilities
fn parse_prost_tag_and_type(field: &syn::Field) -> (u32, String) {
    //eprintln!("🚨 DEBUG: Starting parse_prost_tag_and_type");

    // Log all attributes for debugging
    //eprintln!("🚨 DEBUG: All field attributes:");
    //for (i, attr) in field.attrs.iter().enumerate() {
    //eprintln!("🚨 DEBUG:   Attr #{}: Path: {}", i, attr.path().to_token_stream());
    //eprintln!("🚨 DEBUG:   Attr #{}: Meta: {}", i, attr.meta.to_token_stream());
    //}

    // Find the #[prost(...)] attribute
    let prost_attr = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("prost"));

    //eprintln!("🚨 DEBUG: Found prost attribute: {}", prost_attr.is_some());

    if let Some(attr) = prost_attr {
        //eprintln!("🚨 DEBUG: Prost attribute: {}", attr.to_token_stream());

        if let syn::Meta::List(meta_list) = &attr.meta {
            let tokens = &meta_list.tokens;
            //eprintln!("🚨 DEBUG: Prost meta list tokens: {}", tokens);

            // Simple parsing: extract tag number and type
            let attr_str = tokens.to_string();
            //eprintln!("🚨 DEBUG: Prost attribute string: {}", attr_str);

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

    //eprintln!("🚨 ERROR: Failed to parse protobuf tag number for field: {:?}", field.ident);
    // Return default values instead of panicking, which helps us see more of what's happening
    (0, "unknown".to_string())
}

impl FieldInfo {
    pub(crate) fn new(
        field: &syn::Field,
        type_name: &str,
        param_names: &[String],
        oneof_mapping: &OneofMapping,
    ) -> Self {
        //eprintln!("🚨 DEBUG: Starting FieldInfo::new for field in type: {}", type_name);

        field
            .ident
            .as_ref()
            .map(|ident| {
                //eprintln!("🚨 DEBUG: Processing field: {}", ident);
                let ident_str = ident.to_string();
                let field_path = format!("{}.{}", type_name, ident_str);
                //eprintln!("🚨 DEBUG: Field path: {}", field_path);
                //eprintln!("🚨 DEBUG: Checking if field_path exists in FIELD_TYPE_OVERRIDES");

                // Check if the field path exists in FIELD_TYPE_OVERRIDES
                // This is just for debugging, to see if the key exists
                // let override_exists = if let Some(entry) =
                //     otlp_model::FIELD_TYPE_OVERRIDES.get(field_path.as_str())
                // {
                //     //eprintln!("🚨 DEBUG: Found override for {}: {:?}", field_path, entry);
                //     true
                // } else {
                //     //eprintln!("🚨 DEBUG: No override found for {}", field_path);
                //     false
                // };
                //eprintln!("🚨 DEBUG: Override exists: {}", override_exists);

                let is_param = param_names.contains(&ident_str);
                //eprintln!("🚨 DEBUG: Field path: {}, is_param: {}, override_exists: {}", field_path, is_param, override_exists);

                let oneof = oneof_mapping
                    .as_ref()
                    .filter(|x| x.0 == field_path)
                    .map(|x| x.1.clone());
                //eprintln!("🚨 DEBUG: oneof mapping: {:?}", oneof);

                let is_optional = Self::is_optional(field);
                let is_repeated = Self::is_repeated(field);
                let is_message = Self::is_message(field);
                let is_primitive = Self::is_primitive(field);
                //eprintln!("🚨 DEBUG: Field properties - optional: {}, repeated: {}, message: {}, primitive: {}",
                //    is_optional, is_repeated, is_message, is_primitive);

                // Process type information
                //eprintln!("🚨 DEBUG: Extracting inner type for field: {}", ident_str);
                //eprintln!("🚨 DEBUG: Original type: {}", field.ty.to_token_stream());
                let inner_type = if is_optional || is_repeated {
                    //eprintln!("🚨 DEBUG: Field is optional/repeated, extracting inner type");
                    match Self::extract_inner_type(&field.ty) {
                        Some(inner) => {
                            //eprintln!("🚨 DEBUG: Successfully extracted inner type for field {}", ident_str);
                            inner
                        }
                        None => {
                            //eprintln!("🚨 WARNING: Failed to extract inner type for field {}, using original type as fallback", ident_str);
                            field.ty.clone() // Fallback to original type instead of panicking
                        }
                    }
                } else {
                    //eprintln!("🚨 DEBUG: Field is not optional/repeated, using original type");
                    field.ty.clone()
                };
                //eprintln!("🚨 DEBUG: Extracted inner type: {}", inner_type.to_token_stream());

                // TODO: field_type should be used
                //eprintln!("🚨 DEBUG: Checking for field type overrides for path: {}", field_path);

                // Use a safer approach with try_fold to handle errors
                let result: Result<(syn::Type, Option<syn::Type>), String> =
                    otlp_model::FIELD_TYPE_OVERRIDES
                        .get(field_path.as_str())
                        .map(|over| {
                            //eprintln!("🚨 DEBUG: Found override - datatype: {}, fieldtype: {}", over.datatype, over.fieldtype);

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
                        .unwrap_or_else(|| {
                            //eprintln!("🚨 DEBUG: No type override found, using inner_type");
                            Ok((inner_type.clone(), None))
                        });

                // Handle any errors from the parsing
                let (_field_type, as_type) = match result {
                    Ok(types) => types,
                    Err(_err) => {
                        //eprintln!("🚨 ERROR: {}", err);
                        // Fallback to inner_type on error
                        (inner_type.clone(), None)
                    }
                };

                //eprintln!("🚨 DEBUG: Final as_type: {:?}", as_type.as_ref().map(|t| t.to_token_stream().to_string()));

                // Decompose type into base name and qualifier
                //eprintln!("🚨 DEBUG: Decomposing type for field: {}", ident_str);
                let (base_type_name, qualifier) = Self::decompose_type(&inner_type);
                //eprintln!("🚨 DEBUG: Decomposed type - base_name: {}, has_qualifier: {}",
                //    base_type_name, qualifier.is_some());
                
                // For repeated fields, full_type_name should be the original Vec type, not the inner type
                let full_type_name = if is_repeated {
                    field.ty.clone()
                } else {
                    inner_type.clone()
                };

                // Parse Prost tag, _p
                //eprintln!("🚨 DEBUG: Parsing prost tag for field: {}", ident_str);

                // Inspect field attributes for better debugging
                //eprintln!("🚨 DEBUG: Field attributes:");
                //for (i, attr) in field.attrs.iter().enumerate() {
                //eprintln!("🚨 DEBUG:   Attribute #{}: Path: {:?}", i, attr.path());
                //eprintln!("🚨 DEBUG:   Attribute #{}: Meta: {:?}", i, attr.meta);
                //eprintln!("🚨 DEBUG:   Attribute #{}: Tokens: {}", i, attr.to_token_stream());
                //}

                let (tag, proto_type) = parse_prost_tag_and_type(field);
                //eprintln!("🚨 DEBUG: Parsed tag: {}, proto_type: {}", tag, proto_type);

                //eprintln!("🚨 DEBUG: Creating FieldInfo object for field: {}", ident_str);
                FieldInfo {
                    ident: ident.clone(),
                    is_param,
                    is_optional,
                    is_repeated,
                    is_message,
                    is_primitive,
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
        //eprintln!("🚨 DEBUG: related_type called with suffix: {}, base_type_name: {}, proto_type: {}, is_primitive: {}",
        //    suffix, self.base_type_name, self.proto_type, self.is_primitive);

        // For Visitor suffix, handle primitive types specially
        if suffix == "Visitor" {
            let result = self.get_primitive_visitor_trait();
            return result;
        }

        // For Visitable suffix, also handle primitive types specially
        if suffix == "Visitable" {
            let result = self.get_primitive_visitable_trait();
            //eprintln!("🚨 DEBUG: get_primitive_visitable_trait returned: {}", result);
            return result;
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
                //eprintln!("🚨 DEBUG: Primitive type {} doesn't need adapter, returning empty token stream", self.proto_type);
                // Return empty token stream - this should not be called for primitive types
                // The caller should check needs_adapter_for_field() first
                return quote::quote! { /* PRIMITIVE_NO_ADAPTER */ };
            }
        }

        // For other suffixes, use original logic
        if let Some(ref qualifier) = self.qualifier {
            let base_with_suffix = syn::Ident::new(
                &format!("{}{}", self.base_type_name, suffix),
                proc_macro2::Span::call_site(),
            );
            let result = quote::quote! { #qualifier::#base_with_suffix };
            //eprintln!("🚨 DEBUG: qualified related_type returned: {}", result);
            result
        } else {
            let type_with_suffix = syn::Ident::new(
                &format!("{}{}", self.base_type_name, suffix),
                proc_macro2::Span::call_site(),
            );
            let result = quote::quote! { #type_with_suffix };
            //eprintln!("🚨 DEBUG: unqualified related_type returned: {}", result);
            result
        }
    }

    /// Get primitive type visitor trait with generic argument
    fn get_primitive_visitor_trait(&self) -> proc_macro2::TokenStream {
        eprintln!("🚨 DEBUG: get_primitive_visitor_trait called for: {}, proto_type: {}, is_repeated: {}", 
                  self.base_type_name, self.proto_type, self.is_repeated);

        // Special handling for Vec<u8> (bytes) - check proto_type first
        if self.proto_type == "bytes" || (self.base_type_name == "Vec" && self.proto_type.contains("bytes")) {
            return quote! { crate::pdata::BytesVisitor<Argument> };
        }

        // For repeated primitive fields, use SliceVisitor with the inner primitive type
        if self.is_repeated && self.is_primitive {
            eprintln!("🚨 DEBUG: Repeated primitive field detected: {}", self.base_type_name);
            let result = match self.base_type_name.as_str() {
                "String" => quote! { crate::pdata::SliceVisitor<Argument, String> },
                "bool" => quote! { crate::pdata::SliceVisitor<Argument, bool> },
                "i32" => quote! { crate::pdata::SliceVisitor<Argument, i32> },
                "i64" => quote! { crate::pdata::SliceVisitor<Argument, i64> },
                "u32" | "u8" => quote! { crate::pdata::SliceVisitor<Argument, u32> },
                "u64" => quote! { crate::pdata::SliceVisitor<Argument, u64> },
                "f32" => quote! { crate::pdata::SliceVisitor<Argument, f32> },
                "f64" => quote! { crate::pdata::SliceVisitor<Argument, f64> },
                _ => {
                    panic!(
                        "Unknown repeated primitive type - this should not happen!\n\
                         base_type_name: '{}'\n\
                         full_type_name: {:?}\n\
                         is_primitive: {}\n\
                         is_repeated: {}\n\
                         field_ident: {:?}", 
                        self.base_type_name, 
                        self.full_type_name, 
                        self.is_primitive, 
                        self.is_repeated,
                        self.ident
                    );
                }
            };
            eprintln!("🚨 DEBUG: Returning SliceVisitor for repeated primitive: {}", result);
            return result;
        }

        let result = match self.base_type_name.as_str() {
            "String" => quote! { crate::pdata::StringVisitor<Argument> },
            "bool" => quote! { crate::pdata::BooleanVisitor<Argument> },
            "i32" => quote! { crate::pdata::I32Visitor<Argument> },
            "i64" => quote! { crate::pdata::I64Visitor<Argument> },
            "u32" | "u8" => quote! { crate::pdata::U32Visitor<Argument> },
            "u64" => quote! { crate::pdata::U64Visitor<Argument> },
            "f32" | "f64" => quote! { crate::pdata::F64Visitor<Argument> },
            "Vec" => quote! { crate::pdata::SliceVisitor<Argument, u8> }, // This should rarely be used after bytes check above
            _ => {
                //eprintln!("🚨 DEBUG: Non-primitive type: {}, generating custom visitor trait", self.base_type_name);
                // For non-primitive types, use the standard logic
                if let Some(ref qualifier) = self.qualifier {
                    let base_with_suffix = syn::Ident::new(
                        &format!("{}Visitor", self.base_type_name),
                        proc_macro2::Span::call_site(),
                    );
                    //eprintln!("🚨 DEBUG: Generating qualified visitor: {}::{}<Argument>", qualifier.to_token_stream(), base_with_suffix);
                    // Create the path properly
                    let visitor_path: syn::TypePath =
                        syn::parse_quote! { #qualifier::#base_with_suffix };
                    let argument_ident =
                        syn::Ident::new("Argument", proc_macro2::Span::call_site());
                    quote! { #visitor_path<#argument_ident> }
                } else {
                    let type_with_suffix = syn::Ident::new(
                        &format!("{}Visitor", self.base_type_name),
                        proc_macro2::Span::call_site(),
                    );
                    //eprintln!("🚨 DEBUG: Generating unqualified visitor: {}<Argument>", type_with_suffix);
                    // Create argument identifier separately
                    let argument_ident =
                        syn::Ident::new("Argument", proc_macro2::Span::call_site());
                    quote! { #type_with_suffix<#argument_ident> }
                }
            }
        };

        result
    }

    /// Get primitive type visitable trait with generic argument  
    fn get_primitive_visitable_trait(&self) -> proc_macro2::TokenStream {
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

    /// Decompose a type into base type name and qualifier
    /// Returns (base_type_name, qualifier) where qualifier is the module path
    fn decompose_type(ty: &syn::Type) -> (String, Option<proc_macro2::TokenStream>) {
        //eprintln!("🚨 DEBUG: decompose_type called with type: {}", ty.to_token_stream());

        match ty {
            syn::Type::Path(type_path) => {
                //eprintln!("🚨 DEBUG: Processing Type::Path");

                let path_str = type_path
                    .path
                    .segments
                    .iter()
                    .map(|seg| seg.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");
                //eprintln!("🚨 DEBUG: Full path string: {}", path_str);

                // Get the base type name (last segment)
                let base_type_name = type_path
                    .path
                    .segments
                    .last()
                    .map(|seg| {
                        //eprintln!("🚨 DEBUG: Last segment: {}", seg.ident);
                        seg.ident.to_string()
                    })
                    .unwrap_or_else(|| {
                        //eprintln!("🚨 DEBUG: No last segment, using full path");
                        path_str.clone()
                    });
                //eprintln!("🚨 DEBUG: Base type name: {}", base_type_name);

                // If there's only one segment, no qualifier needed
                if type_path.path.segments.len() == 1 {
                    //eprintln!("🚨 DEBUG: Only one segment, no qualifier needed");
                    return (base_type_name, None);
                }

                // Create qualifier from all but last segment
                //eprintln!("🚨 DEBUG: Creating qualifier from {} segments", type_path.path.segments.len() - 1);
                let qualifier_segments: Vec<_> = type_path
                    .path
                    .segments
                    .iter()
                    .take(type_path.path.segments.len() - 1)
                    .collect();

                if qualifier_segments.is_empty() {
                    //eprintln!("🚨 DEBUG: No qualifier segments");
                    (base_type_name, None)
                } else {
                    //eprintln!("🚨 DEBUG: Building qualifier from segments");
                    let qualifier = qualifier_segments
                        .iter()
                        .map(|seg| {
                            //eprintln!("🚨 DEBUG: Qualifier segment: {}", seg.ident);
                            &seg.ident
                        })
                        .collect::<Vec<_>>();

                    let qualifier_token = quote::quote! { #(#qualifier)::* };
                    //eprintln!("🚨 DEBUG: Created qualifier token: {}", qualifier_token);
                    (base_type_name, Some(qualifier_token))
                }
            }
            _ => {
                //eprintln!("🚨 DEBUG: Not a Type::Path, using fallback");
                // For non-path types, just use the string representation as base name
                let base_name = quote::quote! { #ty }.to_string();
                //eprintln!("🚨 DEBUG: Created fallback base name: {}", base_name);
                (base_name, None)
            }
        }
    }

    /// Extract inner type from a generic container (Option<T>, Vec<T>)
    fn extract_inner_type(ty: &syn::Type) -> Option<syn::Type> {
        //eprintln!("🚨 DEBUG: extract_inner_type called with type: {}", ty.to_token_stream());

        let result = match ty {
            syn::Type::Path(type_path) => {
                //eprintln!("🚨 DEBUG: Processing Type::Path");
                if let Some(last_segment) = type_path.path.segments.last() {
                    //eprintln!("🚨 DEBUG: Last segment: {}", last_segment.ident);

                    match &last_segment.arguments {
                        syn::PathArguments::AngleBracketed(args) => {
                            //eprintln!("🚨 DEBUG: Found AngleBracketed arguments");
                            if let Some(first_arg) = args.args.first() {
                                //eprintln!("🚨 DEBUG: First argument found: {}", first_arg.to_token_stream());

                                match first_arg {
                                    syn::GenericArgument::Type(inner_ty) => {
                                        //eprintln!("🚨 DEBUG: Extracted inner type: {}", inner_ty.to_token_stream());
                                        Some(inner_ty.clone())
                                    }
                                    _ => {
                                        //eprintln!("🚨 DEBUG: First argument is not a Type");
                                        None
                                    }
                                }
                            } else {
                                //eprintln!("🚨 DEBUG: No arguments found");
                                None
                            }
                        }
                        _ => {
                            //eprintln!("🚨 DEBUG: Not AngleBracketed arguments");
                            None
                        }
                    }
                } else {
                    //eprintln!("🚨 DEBUG: No segments in path");
                    None
                }
            }
            _ => {
                //eprintln!("🚨 DEBUG: Not a Type::Path");
                None
            }
        };

        //eprintln!("🚨 DEBUG: extract_inner_type returning: {:?}", result.as_ref().map(|t| t.to_token_stream().to_string()));
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

    // Determine if a field needs to be wrapped in an adapter (message types) vs used directly (primitives)
    // pub fn needs_adapter(&self) -> bool {
    //     // Use the parsed proto_type for fast determination
    //     match self.proto_type.as_str() {
    //         "message" | "enumeration" => true, // Complex types need adapters
    //         "string" | "int64" | "uint32" | "int32" | "uint64" | "bool" | "double" | "float"
    //         | "bytes" => false, // Primitives don't need adapters
    //         _ => {
    //             // For unknown or missing proto_type, fall back to type-based analysis
    //             // This ensures backward compatibility and handles edge cases
    //             !self.is_primitive_type() && !self.is_primitive
    //         }
    //     }
    // }

    // /// Check if this field represents a primitive type
    // fn is_primitive_type(&self) -> bool {
    //     match self.base_type_name.as_str() {
    //         "String" | "bool" | "i32" | "i64" | "u32" | "u64" | "f32" | "f64" | "u8" => true,
    //         _ => false,
    //     }
    // }
}
