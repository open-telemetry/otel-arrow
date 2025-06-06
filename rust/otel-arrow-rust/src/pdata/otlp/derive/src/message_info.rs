// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otlp_model::OneofMapping;

use super::field_info::FieldInfo;
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

pub struct MessageInfo {
    pub outer_name: syn::Ident,
    pub param_names: Vec<String>,
    pub param_fields: Vec<FieldInfo>,
    pub builder_fields: Vec<FieldInfo>,
    pub all_fields: Vec<FieldInfo>,
    pub oneof_mapping: OneofMapping,
}

impl MessageInfo {
    pub fn new<F>(input: TokenStream, f: F) -> TokenStream
    where
        F: FnOnce(Self) -> TokenStream,
    {
        let input = parse_macro_input!(input as DeriveInput);
        let outer_name = input.ident.clone();
        
        eprintln!("🚨 DEBUG: Starting MessageInfo::new for struct: {}", outer_name);

        // Get the fully qualified type name from attribute
        eprintln!("🚨 DEBUG: Looking for qualified attribute on {}", outer_name);
        let type_name = input
            .attrs
            .iter()
            .find_map(|attr| {
                eprintln!("🚨 DEBUG: Checking attribute: {:?}", attr.path());
                if attr.path().is_ident("doc") {
                    eprintln!("🚨 DEBUG: Found doc attribute for {}", outer_name);
                    // Use parse_nested_meta to extract the qualified name
                    let mut qualified_name = None;
                    let _ = attr.parse_nested_meta(|meta| {
                        eprintln!("🚨 DEBUG: Parsing meta: {:?}", meta.path);
                        if meta.path.is_ident("hidden") {
                            Ok(())
                        } else if meta.path.is_ident("otlp_qualified_name") {
                            let value = meta.value()?;
                            let lit: syn::LitStr = value.parse()?;
                            qualified_name = Some(lit.value());
                            eprintln!("🚨 DEBUG: Found qualified name: {}", qualified_name.as_ref().unwrap());
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
            .expect(&format!("Missing #[qualified(\"...\")] attribute on struct {}", outer_name));
        

        eprintln!("🚨 DEBUG: Found type_name: {}", type_name);

        // Get required parameters for this type.
        eprintln!("🚨 DEBUG: Looking up required params for: {}", type_name);
        let param_names: Vec<_> = otlp_model::REQUIRED_PARAMS
            .get(type_name.as_str())
            .expect(&format!("No required parameters found for OTLP type: {}", type_name))
            .iter()
            .map(|x| x.to_string())
            .collect();        eprintln!("🚨 DEBUG: Found {} required params", param_names.len());

        // Check if this struct has a oneof field
        eprintln!("🚨 DEBUG: Looking up oneof mapping for: {}", type_name);
        let oneof_mapping = otlp_model::ONEOF_MAPPINGS
            .iter()
            .find(|(field, _)| field.starts_with(&type_name))
            .map(|(x, y)| (x.clone(), y.clone()));

        eprintln!("🚨 DEBUG: Found oneof mapping: {:?}", oneof_mapping.is_some());

        // Extract all fields from the struct definition
        eprintln!("🚨 DEBUG: Extracting struct fields for: {}", outer_name);
        let struct_fields = match &input.data {
            syn::Data::Struct(data) => {
                if let syn::Fields::Named(fields) = &data.fields {
                    eprintln!("🚨 DEBUG: Found {} named fields", fields.named.len());
                    fields.named.iter().collect::<Vec<_>>()
                } else {
                    eprintln!("🚨 DEBUG: No named fields found");
                    Vec::new()
                }
            }
            _ => {
                eprintln!("🚨 DEBUG: Not a struct");
                Vec::new()
            }
        };



        // If there are no fields, it's either an empty message or an enum,
        // either way should not be listed, no builder is needed.
        eprintln!("🚨 DEBUG: Checking if struct_fields is empty: {}", struct_fields.is_empty());
        if struct_fields.is_empty() {
            panic!("Message '{}' has empty fields", type_name)
        }

        eprintln!("🚨 DEBUG: About to create FieldInfo objects for {} fields", struct_fields.len());
        // Use a safer approach with filter_map to handle potential panic scenarios
        let fields_original: Vec<FieldInfo> = struct_fields
            .iter()
            .enumerate()
            .filter_map(|(i, field)| {
                eprintln!("🚨 DEBUG: Processing field {} of {}", i+1, struct_fields.len());
                
                // Use a match pattern to safely handle field processing
                match field.ident.as_ref() {
                    Some(ident) => {
                        eprintln!("🚨 DEBUG: Field has ident: {}", ident);
                        
                        // Use a safer approach - catch any potential panics
                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            FieldInfo::new(field, &type_name, &param_names, &oneof_mapping)
                        })) {
                            Ok(field_info) => Some(field_info),
                            Err(e) => {
                                // Log the error and continue with remaining fields
                                if let Some(msg) = e.downcast_ref::<String>() {
                                    eprintln!("🚨 ERROR: Failed to process field {}: {}", ident, msg);
                                } else {
                                    eprintln!("🚨 ERROR: Failed to process field {}: unknown error", ident);
                                }
                                None
                            }
                        }
                    },
                    None => {
                        eprintln!("🚨 DEBUG: Field has no identifier, skipping");
                        None
                    }
                }
            })
            .collect();

        // Partition fields into ordered parameters and remaining builder fields - safely
        let param_fields: Vec<_> = param_names
            .iter()
            .filter_map(|param_name| {
                let field = fields_original
                    .iter()
                    .find(|info| {
                        let ident = info.ident.to_string();
                        info.is_param && ident == *param_name
                    });
                
                match field {
                    Some(field) => {
                        eprintln!("🚨 DEBUG: Found param field: {}", param_name);
                        Some(field.clone())
                    },
                    None => {
                        eprintln!("🚨 ERROR: Required parameter {} not found in fields", param_name);
                        None
                    }
                }
            })
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

        f(MessageInfo {
            outer_name,
            param_names,
            param_fields,
            builder_fields,
            all_fields,
            oneof_mapping,
        })
    }

    pub fn related_typename(&self, suffix: &str) -> syn::Ident {
        syn::Ident::new(
            &format!("{}{}", self.outer_name, suffix),
            self.outer_name.span()
        )
    }
}
