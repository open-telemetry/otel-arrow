// SPDX-License-Identifier: Apache-2.0

//! Procedural macros to derive `MultivariateMetrics` for metrics structs.
//!
//! Container attributes:
//!   - `#[metrics(name = "my.metrics.name")]`
//! Field attributes:
//!   - `#[metric(name = "field.name", unit = "{unit}")]`

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Fields, ItemStruct, LitStr};
use syn::parse_quote;

/// Derive implementation of `otap_df_telemetry::metrics::MultivariateMetrics` for a struct.
///
/// Container attribute:
///   - `#[metrics(name = "my.metrics.name")]`
/// Field attributes:
///   - `#[metric(name = "field.name", unit = "{unit}")]`
#[proc_macro_derive(MultivariateMetrics, attributes(metrics, metric))]
pub fn derive_multivariate_metrics(input: TokenStream) -> TokenStream {
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
                return syn::Error::new(input_span, "MultivariateMetrics can only be derived for structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new(input.span(), "MultivariateMetrics can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    // Collect metric fields (skip non-Counter fields for now by requiring Counter<u64> type name)
    let mut metric_field_idents = Vec::new();
    let mut metric_field_units = Vec::new();
    let mut metric_field_names = Vec::new();

    for field in fields {
        let ident = field.ident.clone().unwrap();

        // Find #[metric(...)]
        let mut name_attr: Option<String> = None;
        let mut unit_attr: Option<String> = None;
        for attr in &field.attrs {
            if let Some((n, u)) = parse_metric_field_attr(attr) {
                name_attr = Some(n);
                unit_attr = Some(u);
            }
        }

        if let (Some(name), Some(unit)) = (name_attr, unit_attr) {
            metric_field_idents.push(ident);
            metric_field_units.push(unit);
            metric_field_names.push(name);
        }
    }

    let desc_ident = format_ident!("DESCRIPTOR");

    let generated = quote! {
        impl #generics otap_df_telemetry::metrics::MultivariateMetrics for #struct_ident #generics {
            fn register_into(&mut self, registry: &mut otap_df_telemetry::registry::MetricsRegistry, attrs: otap_df_telemetry::attributes::NodeStaticAttrs) {
                let k = registry.insert_default::<Self>(attrs);
                self.key = Some(k);
            }

            fn descriptor(&self) -> &'static otap_df_telemetry::descriptor::MetricsDescriptor {
                // A single static descriptor per type
                static #desc_ident: otap_df_telemetry::descriptor::MetricsDescriptor = otap_df_telemetry::descriptor::MetricsDescriptor {
                    name: #metrics_name,
                    fields: &[
                        #( otap_df_telemetry::descriptor::MetricsField { name: #metric_field_names, unit: #metric_field_units, kind: otap_df_telemetry::descriptor::MetricsKind::Counter } ),*
                    ],
                };
                &#desc_ident
            }

            fn field_values(&self) -> Box<dyn Iterator<Item = (&'static otap_df_telemetry::descriptor::MetricsField, u64)> + '_> {
                let vals = [ #( self.#metric_field_idents.get() ),* ];
                Box::new(self.descriptor().fields.iter().zip(vals.into_iter()).map(|(f,v)| (f, v)))
            }

            fn merge_from_same_kind(&mut self, other: &dyn otap_df_telemetry::metrics::MultivariateMetrics) {
                let o = other.as_any().downcast_ref::<Self>().expect("type mismatch in merge_from_same_kind");
                #( self.#metric_field_idents.add(o.#metric_field_idents.get()); )*
            }

            fn aggregate_into(&self, registry: &mut otap_df_telemetry::registry::MetricsRegistryHandle) -> Result<(), otap_df_telemetry::error::Error> {
                if let Some(k) = self.key { registry.add_metrics(k, self); Ok(()) } else { Err(otap_df_telemetry::error::Error::MetricsNotRegistered { descriptor: self.descriptor() }) }
            }

            fn zero(&mut self) {
                #( self.#metric_field_idents.set(0); )*
            }

            fn has_non_zero(&self) -> bool {
                self.field_values().any(|(_, v)| v != 0)
            }

            fn as_any(&self) -> &dyn std::any::Any { self }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
        }
    };

    generated.into()
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

fn parse_metric_field_attr(attr: &Attribute) -> Option<(String, String)> {
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
    match (name, unit) { (Some(n), Some(u)) => Some((n,u)), _ => None }
}

/// Attribute macro that injects `#[repr(C, align(64))]` and wires up the MultivariateMetrics derive
/// and descriptor name via a container attribute.
///
/// Usage:
///   #[otap_df_telemetry_macros::telemetry_metrics(name = "my.metrics")]
///   pub struct MyMetrics { #[metric(name = "x", unit = "{unit}")] x: Counter<u64>, ... }
#[proc_macro_attribute]
pub fn telemetry_metrics(attr: TokenStream, item: TokenStream) -> TokenStream {
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
            return syn::Error::new(proc_macro2::Span::call_site(), "missing `name = \"...\"` in telemetry_metrics attribute")
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

    // Ensure the MultivariateMetrics derive is attached FIRST to introduce helper attributes
    let derive_attr: Attribute = parse_quote!(#[derive(otap_df_telemetry_macros::MultivariateMetrics)]);
    s.attrs.push(derive_attr);

    // Add container descriptor attribute consumed by the derive
    let metrics_attr: Attribute = parse_quote!(#[metrics(name = #metrics_name)]);
    s.attrs.push(metrics_attr);

    quote!( #s ).into()
}
