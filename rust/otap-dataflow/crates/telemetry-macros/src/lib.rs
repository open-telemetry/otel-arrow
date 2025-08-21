// SPDX-License-Identifier: Apache-2.0

//! Procedural macros to derive `MetricSetHandler` and `AttributeSetHandler` for structs.
//!
//! Container attributes:
//!   - `#[metrics(name = "my.metrics.name")]`
//!   - `#[attributes(name = "my.attributes.name")]`
//! Field attributes:
//!   - `#[metric(name = "field.name", unit = "{unit}")]`
//!   - `#[attribute(key = "field.key")]`

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Fields, ItemStruct, LitStr};
use syn::parse_quote;

/// Derive implementation of `otap_df_telemetry::metrics::MetricSetHandler` for a struct.
///
/// Container attribute:
///   - `#[metrics(name = "my.metrics.name")]`
/// Field attributes:
///   - `#[metric(name = "field.name", unit = "{unit}")]`
#[proc_macro_derive(MetricSetHandler, attributes(metrics, metric))]
pub fn derive_metric_set_handler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let input_span = input.span();
    let struct_ident = input.ident.clone();
    let generics = input.generics.clone();

    let mut metrics_name: Option<String> = None;

    for attr in &input.attrs {
        if let Some(name) = parse_metrics_name_attr(attr) {
            metrics_name = Some(name);
        }
    }
    let metrics_name = match metrics_name {
        Some(n) => n,
        None => {
            return syn::Error::new(input.span(), "missing #[metrics(name = \"...\")] on struct")
                .to_compile_error()
                .into();
        }
    };

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return syn::Error::new(input_span, "MetricSetHandler can only be derived for structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new(input.span(), "MetricSetHandler can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    // Collect metric fields (skip non-Counter fields for now by requiring Counter<u64> type name)
    let mut metric_field_idents = Vec::new();
    let mut metric_field_units = Vec::new();
    let mut metric_field_names = Vec::new();
    let mut metric_field_briefs = Vec::new();
    let mut metric_field_instruments: Vec<proc_macro2::TokenStream> = Vec::new();

    for field in fields {
        let ident = field.ident.clone().unwrap();

        // Collect doc comments for brief (concatenate all lines)
        let mut brief_lines: Vec<String> = Vec::new();
        for attr in &field.attrs {
            if attr.meta.path().is_ident("doc") {
                if let syn::Meta::NameValue(nv) = &attr.meta {
                    if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(ls), .. }) = &nv.value {
                        let line = ls.value().trim().to_string();
                        if !line.is_empty() { brief_lines.push(line); }
                    }
                }
            }
        }
        let brief_combined = brief_lines.join(" ");

        // Find #[metric(...)]
        let mut name_attr: Option<String> = None;
        let mut unit_attr: Option<String> = None;
        for attr in &field.attrs {
            if let Some((maybe_name, u)) = parse_metric_field_attr(attr) {
                if maybe_name.is_some() { name_attr = maybe_name; }
                unit_attr = Some(u);
            }
        }

        if let Some(unit) = unit_attr {
            let derived_name = ident.to_string().replace('_', ".");
            let final_name = name_attr.unwrap_or(derived_name);

            // Validate type path and instrument kind
            let instrument_variant = match &field.ty {
                syn::Type::Path(tp) => {
                    let seg_opt = tp.path.segments.last();
                    if let Some(seg) = seg_opt {
                        let ident_ty = seg.ident.to_string();
                        // Expect generic arguments <u64>
                        let is_u64 = match &seg.arguments {
                            syn::PathArguments::AngleBracketed(ab) => {
                                if ab.args.len() != 1 { false } else {
                                    matches!(ab.args.first(), Some(syn::GenericArgument::Type(syn::Type::Path(p)) ) if p.path.is_ident("u64"))
                                }
                            },
                            _ => false
                        };
                        if !is_u64 {
                            return syn::Error::new(seg.ident.span(), "Metric field type must be one of Counter<u64>, UpDownCounter<u64>, Gauge<u64>")
                                .to_compile_error().into();
                        }
                        match ident_ty.as_str() {
                            "Counter" => quote!(otap_df_telemetry::descriptor::Instrument::Counter),
                            "UpDownCounter" => quote!(otap_df_telemetry::descriptor::Instrument::UpDownCounter),
                            "Gauge" => quote!(otap_df_telemetry::descriptor::Instrument::Gauge),
                            other => return syn::Error::new(seg.ident.span(), format!("Unsupported metric instrument type: {other}" ))
                                .to_compile_error().into(),
                        }
                    } else {
                        return syn::Error::new(field.ty.span(), "Unsupported metric field type")
                            .to_compile_error().into();
                    }
                },
                _ => return syn::Error::new(field.ty.span(), "Unsupported metric field type")
                    .to_compile_error().into(),
            };
            metric_field_idents.push(ident);
            metric_field_units.push(unit);
            metric_field_names.push(final_name);
            metric_field_briefs.push(brief_combined);
            metric_field_instruments.push(instrument_variant);
        }
    }

    let desc_ident = format_ident!("DESCRIPTOR");

    let generated = quote! {
        impl #generics otap_df_telemetry::metrics::MetricSetHandler for #struct_ident #generics {
            fn descriptor(&self) -> &'static otap_df_telemetry::descriptor::MetricsDescriptor {
                static #desc_ident: otap_df_telemetry::descriptor::MetricsDescriptor = otap_df_telemetry::descriptor::MetricsDescriptor {
                    name: #metrics_name,
                    metrics: &[
                        #( otap_df_telemetry::descriptor::MetricsField {
                            name: #metric_field_names,
                            unit: #metric_field_units,
                            brief: #metric_field_briefs,
                            instrument: #metric_field_instruments
                        } ),*
                    ],
                };
                &#desc_ident
            }
            fn snapshot_values(&self) -> ::std::vec::Vec<u64> {
                let mut out = ::std::vec::Vec::with_capacity(self.descriptor().metrics.len());
                #( out.push(self.#metric_field_idents.get()); )*
                out
            }
            fn clear_values(&mut self) {
                #( self.#metric_field_idents.reset(); )*
            }
            fn needs_flush(&self) -> bool {
                #( if self.#metric_field_idents.get() != 0 { return true; } )*
                false
            }
        }
    };

    generated.into()
}

/// Derive implementation of `otap_df_telemetry::attributes::AttributeSetHandler` for a struct.
///
/// Container attribute:
///   - `#[attributes(name = "my.attributes.name")]`
/// Field attributes:
///   - `#[attribute(key = "field.key")]` (optional, defaults to field name with dots)
///   - `#[compose]` for fields that implement AttributeSetHandler
#[proc_macro_derive(AttributeSetHandler, attributes(attributes, attribute, compose))]
pub fn derive_attribute_set_handler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let input_span = input.span();
    let struct_ident = input.ident.clone();
    let generics = input.generics.clone();

    let mut attributes_name: Option<String> = None;

    for attr in &input.attrs {
        if let Some(name) = parse_attributes_name_attr(attr) {
            attributes_name = Some(name);
        }
    }
    let attributes_name = match attributes_name {
        Some(n) => n,
        None => {
            return syn::Error::new(input.span(), "missing #[attributes(name = \"...\")] on struct")
                .to_compile_error()
                .into();
        }
    };

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return syn::Error::new(input_span, "AttributeSetHandler can only be derived for structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new(input.span(), "AttributeSetHandler can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    // Determine if we're in the telemetry crate itself by looking at the crate name context
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap_or_default();
    let is_telemetry_crate = crate_name == "otap-df-telemetry";

    // Choose path prefixes based on context
    let (attr_handler_path, descriptor_path, field_path, value_type_path, attr_value_path) = if is_telemetry_crate {
        (
            quote!(crate::attributes::AttributeSetHandler),
            quote!(crate::descriptor::AttributesDescriptor),
            quote!(crate::descriptor::AttributeField),
            quote!(crate::descriptor::AttributeValueType),
            quote!(crate::attributes::AttributeValue),
        )
    } else {
        (
            quote!(::otap_df_telemetry::attributes::AttributeSetHandler),
            quote!(::otap_df_telemetry::descriptor::AttributesDescriptor),
            quote!(::otap_df_telemetry::descriptor::AttributeField),
            quote!(::otap_df_telemetry::descriptor::AttributeValueType),
            quote!(::otap_df_telemetry::attributes::AttributeValue),
        )
    };

    // Collect attribute fields and composed attribute sets
    let mut attr_field_idents = Vec::new();
    let mut attr_field_keys = Vec::new();
    let mut attr_field_descriptions = Vec::new();
    let mut attr_field_types: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut attr_iter_values: Vec<proc_macro2::TokenStream> = Vec::new();

    let mut composed_field_idents = Vec::new();

    for field in fields {
        let ident = field.ident.clone().unwrap();

        // Check if this field is marked with #[compose]
        let is_composed = field.attrs.iter().any(|attr| attr.path().is_ident("compose"));

        if is_composed {
            // This field should implement AttributeSetHandler
            composed_field_idents.push(ident);
            continue;
        }

        // Check if this field has #[attribute] annotation
        let has_attribute_attr = field.attrs.iter().any(|attr| attr.path().is_ident("attribute"));

        if !has_attribute_attr {
            // Skip fields without #[attribute] or #[compose] annotations
            continue;
        }

        // Collect doc comments for description (concatenate all lines)
        let mut desc_lines: Vec<String> = Vec::new();
        for attr in &field.attrs {
            if attr.meta.path().is_ident("doc") {
                if let syn::Meta::NameValue(nv) = &attr.meta {
                    if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(ls), .. }) = &nv.value {
                        let line = ls.value().trim().to_string();
                        if !line.is_empty() { desc_lines.push(line); }
                    }
                }
            }
        }
        let description = if desc_lines.is_empty() {
            format!("{} field", ident)
        } else {
            desc_lines.join(" ")
        };

        // Find #[attribute(key = "...")]
        let mut key_attr: Option<String> = None;
        for attr in &field.attrs {
            if let Some(key) = parse_attribute_field_attr(attr) {
                key_attr = Some(key);
            }
        }
        let derived_key = ident.to_string().replace('_', ".");
        let final_key = key_attr.unwrap_or(derived_key);

        // Determine attribute value type based on field type
        let (attr_type, iter_value) = match &field.ty {
            syn::Type::Path(tp) => {
                let seg_opt = tp.path.segments.last();
                if let Some(seg) = seg_opt {
                    let ident_ty = seg.ident.to_string();
                    match ident_ty.as_str() {
                        "String" => (
                            quote!(#value_type_path::String),
                            quote!(#attr_value_path::String(self.#ident.clone()))
                        ),
                        "u32" | "u64" | "usize" => (
                            quote!(#value_type_path::Int),
                            quote!(#attr_value_path::UInt(self.#ident as u64))
                        ),
                        "i32" | "i64" | "isize" => (
                            quote!(#value_type_path::Int),
                            quote!(#attr_value_path::Int(self.#ident as i64))
                        ),
                        "f32" | "f64" => (
                            quote!(#value_type_path::Double),
                            quote!(#attr_value_path::Double(self.#ident as f64))
                        ),
                        "bool" => (
                            quote!(#value_type_path::Boolean),
                            quote!(#attr_value_path::Boolean(self.#ident))
                        ),
                        _ => {
                            // Check if it's a generic type like Cow<'static, str>
                            if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                                if ident_ty == "Cow" && ab.args.len() == 2 {
                                    // Assume Cow<'static, str>
                                    (
                                        quote!(#value_type_path::String),
                                        quote!(#attr_value_path::String(self.#ident.to_string()))
                                    )
                                } else {
                                    return syn::Error::new(seg.ident.span(), format!("Unsupported attribute field type: {}", ident_ty))
                                        .to_compile_error().into();
                                }
                            } else {
                                return syn::Error::new(seg.ident.span(), format!("Unsupported attribute field type: {}", ident_ty))
                                    .to_compile_error().into();
                            }
                        }
                    }
                } else {
                    return syn::Error::new(field.ty.span(), "Unsupported attribute field type")
                        .to_compile_error().into();
                }
            },
            _ => return syn::Error::new(field.ty.span(), "Unsupported attribute field type")
                .to_compile_error().into(),
        };

        attr_field_idents.push(ident);
        attr_field_keys.push(final_key);
        attr_field_descriptions.push(description);
        attr_field_types.push(attr_type);
        attr_iter_values.push(iter_value);
    }

    let desc_ident = format_ident!("ATTRIBUTES_DESCRIPTOR");

    // Generate the descriptor and iterator implementation
    if composed_field_idents.is_empty() {
        // Simple case: no composition - keep the original approach
        let generated = quote! {
            impl #generics #attr_handler_path for #struct_ident #generics {
                fn descriptor(&self) -> &'static #descriptor_path {
                    static #desc_ident: #descriptor_path = #descriptor_path {
                        name: #attributes_name,
                        fields: &[
                            #( #field_path {
                                key: #attr_field_keys,
                                brief: #attr_field_descriptions,
                                r#type: #attr_field_types
                            } ),*
                        ],
                    };
                    &#desc_ident
                }

                fn iter_attributes<'a>(&'a self) -> ::std::boxed::Box<dyn ::std::iter::Iterator<Item = (&'static str, #attr_value_path)> + 'a> {
                    let fields = self.descriptor().fields;
                    let values = ::std::vec![
                        #( #attr_iter_values ),*
                    ];
                    ::std::boxed::Box::new(fields.iter().zip(values.into_iter()).map(|(field, value)| (field.key, value)))
                }
            }
        };
        generated.into()
    } else {
        // Complex case: composition - use lazy_static pattern
        let local_fields_len = attr_field_idents.len();

        let generated = quote! {
            impl #generics #attr_handler_path for #struct_ident #generics {
                fn descriptor(&self) -> &'static #descriptor_path {
                    use ::std::sync::Once;
                    static INIT: Once = Once::new();
                    static mut #desc_ident: Option<#descriptor_path> = None;

                    unsafe {
                        INIT.call_once(|| {
                            // Create a dummy instance to access composed descriptors
                            let dummy = Self::default();

                            // Calculate total field count
                            let mut total_fields = #local_fields_len;
                            #( total_fields += dummy.#composed_field_idents.descriptor().fields.len(); )*

                            // Create a vector to hold all fields
                            let mut all_fields = ::std::vec::Vec::with_capacity(total_fields);

                            // Add local fields
                            all_fields.extend_from_slice(&[
                                #( #field_path {
                                    key: #attr_field_keys,
                                    brief: #attr_field_descriptions,
                                    r#type: #attr_field_types
                                } ),*
                            ]);

                            // Add fields from composed sets
                            #( all_fields.extend_from_slice(dummy.#composed_field_idents.descriptor().fields); )*

                            // Leak the vector to get a 'static reference
                            let fields_slice: &'static [#field_path] = ::std::boxed::Box::leak(all_fields.into_boxed_slice());

                            #desc_ident = Some(#descriptor_path {
                                name: #attributes_name,
                                fields: fields_slice,
                            });
                        });

                        #desc_ident.as_ref().unwrap()
                    }
                }

                fn iter_attributes<'a>(&'a self) -> ::std::boxed::Box<dyn ::std::iter::Iterator<Item = (&'static str, #attr_value_path)> + 'a> {
                    let mut iterators: ::std::vec::Vec<::std::boxed::Box<dyn ::std::iter::Iterator<Item = (&'static str, #attr_value_path)> + 'a>> = ::std::vec::Vec::new();

                    // Add local attributes
                    if #local_fields_len > 0 {
                        let local_fields = &self.descriptor().fields[..#local_fields_len];
                        let local_values = ::std::vec![
                            #( #attr_iter_values ),*
                        ];
                        iterators.push(::std::boxed::Box::new(
                            local_fields.iter().zip(local_values.into_iter()).map(|(field, value)| (field.key, value))
                        ));
                    }

                    // Add composed attributes
                    #( iterators.push(self.#composed_field_idents.iter_attributes()); )*

                    // Chain all iterators
                    ::std::boxed::Box::new(iterators.into_iter().flatten())
                }
            }
        };
        generated.into()
    }
}

/// Attribute macro that injects `#[repr(C, align(64))]` and wires up the MetricSetHandler derive
/// and descriptor name via a container attribute.
/// Usage:
///   #[otap_df_telemetry_macros::metric_set(name = "my.metrics")]
///   pub struct MyMetrics { #[metric(name = "x", unit = "{unit}")] x: Counter<u64>, ... }
#[proc_macro_attribute]
pub fn metric_set(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse name argument
    let args = proc_macro2::TokenStream::from(attr);
    let mut name_val: Option<String> = None;
    if let Err(err) = syn::parse::Parser::parse2(
        |input: syn::parse::ParseStream<'_>| -> syn::Result<()> {
            while !input.is_empty() {
                let ident: syn::Ident = input.parse()?;
                let _: syn::Token![=] = input.parse()?;
                let lit: LitStr = input.parse()?;
                if ident == "name" { name_val = Some(lit.value()); }
                if input.peek(syn::Token![,]) { let _: syn::Token![,] = input.parse()?; }
            }
            Ok(())
        }, args) {
        return err.to_compile_error().into();
    }

    let metrics_name = match name_val {
        Some(n) => n,
        None => {
            return syn::Error::new(proc_macro2::Span::call_site(), "missing `name = \"...\"` in metric_set attribute")
                .to_compile_error()
                .into();
        }
    };

    // Parse the struct item
    let mut s = parse_macro_input!(item as ItemStruct);

    // Inject #[repr(C, align(64))]
    let repr_attr: Attribute = parse_quote!(#[repr(C, align(64))]);
    // Only add if not already present
    let has_repr = s.attrs.iter().any(|a| a.path().is_ident("repr"));
    if !has_repr { s.attrs.push(repr_attr); }

    // Ensure the MetricSetHandler derive is attached
    let derive_attr: Attribute = parse_quote!(#[derive(otap_df_telemetry_macros::MetricSetHandler)]);
    s.attrs.push(derive_attr);

    // Add container descriptor attribute consumed by the derive
    let metrics_attr: Attribute = parse_quote!(#[metrics(name = #metrics_name)]);
    s.attrs.push(metrics_attr);

    quote!( #s ).into()
}

/// Attribute macro that injects `#[derive(AttributeSetHandler)]` and wires up the AttributeSetHandler derive
/// and descriptor name via a container attribute.
/// Usage:
///   #[otap_df_telemetry_macros::attribute_set(name = "my.attributes")]
///   pub struct MyAttributes { #[attribute(key = "custom.key")] field: String, ... }
#[proc_macro_attribute]
pub fn attribute_set(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse name argument
    let args = proc_macro2::TokenStream::from(attr);
    let mut name_val: Option<String> = None;
    if let Err(err) = syn::parse::Parser::parse2(
        |input: syn::parse::ParseStream<'_>| -> syn::Result<()> {
            while !input.is_empty() {
                let ident: syn::Ident = input.parse()?;
                let _: syn::Token![=] = input.parse()?;
                let lit: LitStr = input.parse()?;
                if ident == "name" { name_val = Some(lit.value()); }
                if input.peek(syn::Token![,]) { let _: syn::Token![,] = input.parse()?; }
            }
            Ok(())
        }, args) {
        return err.to_compile_error().into();
    }

    let attributes_name = match name_val {
        Some(n) => n,
        None => {
            return syn::Error::new(proc_macro2::Span::call_site(), "missing `name = \"...\"` in attribute_set attribute")
                .to_compile_error()
                .into();
        }
    };

    // Parse the struct item
    let mut s = parse_macro_input!(item as ItemStruct);

    // Ensure the AttributeSetHandler derive is attached
    let derive_attr: Attribute = parse_quote!(#[derive(otap_df_telemetry_macros::AttributeSetHandler)]);
    s.attrs.push(derive_attr);

    // Add container descriptor attribute consumed by the derive
    let attributes_attr: Attribute = parse_quote!(#[attributes(name = #attributes_name)]);
    s.attrs.push(attributes_attr);

    quote!( #s ).into()
}

fn parse_metrics_name_attr(attr: &Attribute) -> Option<String> {
    if !attr.path().is_ident("metrics") { return None; }
    let mut out: Option<String> = None;
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("name") {
            let s: LitStr = meta.value()?.parse()?;
            out = Some(s.value());
        }
        Ok(())
    });
    out
}

fn parse_metric_field_attr(attr: &Attribute) -> Option<(Option<String>, String)> {
    if !attr.path().is_ident("metric") { return None; }
    let mut name: Option<String> = None;
    let mut unit: Option<String> = None;
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("name") {
            let s: LitStr = meta.value()?.parse()?;
            name = Some(s.value());
        } else if meta.path.is_ident("unit") {
            let s: LitStr = meta.value()?.parse()?;
            unit = Some(s.value());
        }
        Ok(())
    });
    match unit { Some(u) => Some((name, u)), _ => None }
}

fn parse_attributes_name_attr(attr: &Attribute) -> Option<String> {
    if !attr.path().is_ident("attributes") { return None; }
    let mut out: Option<String> = None;
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("name") {
            let s: LitStr = meta.value()?.parse()?;
            out = Some(s.value());
        }
        Ok(())
    });
    out
}

fn parse_attribute_field_attr(attr: &Attribute) -> Option<String> {
    if !attr.path().is_ident("attribute") { return None; }
    let mut out: Option<String> = None;
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("key") {
            let s: LitStr = meta.value()?.parse()?;
            out = Some(s.value());
        }
        Ok(())
    });
    out
}
