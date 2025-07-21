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
            .expect(&format!(
                "Missing #[qualified(\"...\")] attribute on struct {}",
                outer_name
            ));

        // Get required parameters for this type.
        let param_names: Vec<_> = otlp_model::REQUIRED_PARAMS
            .get(type_name.as_str())
            .expect(&format!(
                "No required parameters found for OTLP type: {}",
                type_name
            ))
            .iter()
            .map(|x| x.to_string())
            .collect();

        // Check if this struct has a oneof field
        let oneof_mapping = otlp_model::ONEOF_MAPPINGS
            .iter()
            .find(|(field, _)| field.starts_with(&type_name))
            .map(|(x, y)| (x.clone(), y.clone()));

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
            panic!("Message '{}' has empty fields", type_name)
        }

        // Use a safer approach with filter_map to handle potential panic scenarios
        let fields_original: Vec<FieldInfo> = struct_fields
            .iter()
            .map(|field| {
                // Use a match pattern to safely handle field processing
                FieldInfo::new(field, &type_name, &param_names, &oneof_mapping)
            })
            .collect();

        // Partition fields into ordered parameters and remaining builder fields - safely
        let param_fields: Vec<_> = param_names
            .iter()
            .filter_map(|param_name| {
                let field = fields_original.iter().find(|info| {
                    let ident = info.ident.to_string();
                    info.is_param && ident == *param_name
                });

                match field {
                    Some(field) => Some(field.clone()),
                    None => None,
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
            self.outer_name.span(),
        )
    }
}
