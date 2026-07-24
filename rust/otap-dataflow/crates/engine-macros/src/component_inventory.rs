// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the `#[component_inventory]` attribute macro (RFC 0001).
//!
//! The macro annotates a security-relevant component and emits one
//! `ComponentMeta` entry into the `otap_df_engine::inventory::COMPONENT_INVENTORY`
//! distributed slice at link time, mirroring the `#[capability]` ->
//! `KNOWN_CAPABILITIES` mechanism. The annotated item is re-emitted unchanged.
//!
//! The attribute-argument grammar ([`ComponentInventoryArgs`]) is defined once
//! in the shared `otap-df-engine-inventory-syntax` crate and reused by both
//! this macro and the `cargo xtask component-inventory` scanner, so the two can
//! never disagree about what an annotation means.
//!
//! # Accepted forms
//!
//! Factory `static` (the common case) -- `id` is derived from the factory's
//! `name` (URN) field, so the author writes no `id`:
//!
//! ```rust,ignore
//! #[component_inventory(
//!     category = Receiver,
//!     description = "OTLP unary gRPC receiver on port 4317",
//!     attributes(port = "4317", protocol = "gRPC (HTTP/2)", auth = "mTLS (opt-in)"),
//! )]
//! #[distributed_slice(OTAP_RECEIVER_FACTORIES)]
//! pub static OTLP_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
//!     name: OTLP_RECEIVER_URN,
//!     // ...
//! };
//! ```
//!
//! Non-factory item (fallback) -- no `name`/URN, so an explicit URN-shaped `id`
//! is required:
//!
//! ```rust,ignore
//! #[component_inventory(
//!     id = "urn:otel:admin:http_server",
//!     category = Admin,
//!     description = "Built-in HTTP admin server",
//!     attributes(port = "8080", protocol = "HTTP", auth = "NONE"),
//! )]
//! pub struct AdminServer { /* ... */ }
//! ```

use otap_df_engine_inventory_syntax::ComponentInventoryArgs;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Attribute, Expr, ExprLit, Ident, Item, Lit};

/// Expand `#[component_inventory(...)]` on `item`.
///
/// Re-emits `item` unchanged and appends one `COMPONENT_INVENTORY` slice entry.
pub(crate) fn expand_component_inventory(args: ComponentInventoryArgs, item: Item) -> TokenStream {
    match try_expand(&args, &item) {
        Ok(ts) => ts,
        Err(err) => {
            // Emit the original item plus the error so downstream references
            // still resolve and the diagnostic points at the annotation.
            let compile_err = err.to_compile_error();
            quote! {
                #item
                #compile_err
            }
        }
    }
}

fn try_expand(args: &ComponentInventoryArgs, item: &Item) -> syn::Result<TokenStream> {
    // Derive the identity of the annotated item and (for factory statics) the
    // `name` field expression used as the id.
    let (item_ident, name_field_expr, cfg_attrs) = inspect_item(item)?;

    // Resolve the `id` expression:
    //  - explicit `id = "..."` wins;
    //  - else, for a factory static, reuse the `name` field expression (URN);
    //  - else it is an error (non-factory item with no id).
    let id_expr: TokenStream = if let Some(id) = &args.id {
        quote! { #id }
    } else if let Some(name_expr) = &name_field_expr {
        quote! { #name_expr }
    } else {
        return Err(syn::Error::new(
            Span::call_site(),
            "#[component_inventory] requires an explicit `id = \"urn:...\"` for items \
             that are not factory statics with a `name` field",
        ));
    };

    // Best-effort URN cross-check: only possible when the id is a string
    // literal visible at macro time (explicit `id`, or a `name` field written
    // as a literal). When the URN is a `const` path (the common factory case),
    // the value is not visible here; the xtask scanner performs the full
    // cross-check with resolved values.
    args.check_urn_category(literal_urn(args, &name_field_expr).as_deref())?;

    let category = &args.category;
    let description_expr: TokenStream = match &args.description {
        Some(d) => quote! { ::core::option::Option::Some(#d) },
        None => quote! { ::core::option::Option::None },
    };
    let attr_keys = args.attributes.iter().map(|(k, _)| k);
    let attr_vals = args.attributes.iter().map(|(_, v)| v);

    // Unique static name for the generated entry, derived from the item ident.
    let entry_ident = format_ident!("_COMPONENT_META_{}", item_ident);

    let engine_path = engine_crate_path();

    Ok(quote! {
        // Re-emit the annotated item unchanged.
        #item

        // Register the component in the COMPONENT_INVENTORY distributed slice at
        // link time. `#[allow(unsafe_code)]` is required because
        // `linkme::distributed_slice` emits a static with `#[link_section]`,
        // which crate lints (`-D unsafe-code`) would otherwise reject. The
        // entry inherits the annotated item's `#[cfg(...)]` so the inventory
        // reflects exactly what was compiled.
        #(#cfg_attrs)*
        #[allow(unsafe_code)]
        #[allow(non_upper_case_globals)]
        #[doc(hidden)]
        #[::linkme::distributed_slice(#engine_path::inventory::COMPONENT_INVENTORY)]
        #[linkme(crate = ::linkme)]
        static #entry_ident: #engine_path::inventory::ComponentMeta =
            #engine_path::inventory::ComponentMeta {
                id: #id_expr,
                category: #engine_path::inventory::Category::#category,
                description: #description_expr,
                file: ::core::file!(),
                line: ::core::line!(),
                attributes: &[ #( (#attr_keys, #attr_vals) ),* ],
            };
    })
}

/// Path to `otap_df_engine` crate: `crate` when compiled within `otap-df-engine`,
/// otherwise `::otap_df_engine`.
fn engine_crate_path() -> TokenStream {
    if std::env::var("CARGO_CRATE_NAME").as_deref() == Ok("otap_df_engine") {
        quote! { crate }
    } else {
        quote! { ::otap_df_engine }
    }
}

/// Inspect the annotated item, returning:
///  - the item's identifier (for naming the generated static),
///  - the `name` field's value expression, if the item is a factory `static`
///    initialized with a struct literal containing a `name` field,
///  - the item's `#[cfg(...)]` attributes (propagated to the emitted entry).
fn inspect_item(item: &Item) -> syn::Result<(Ident, Option<Expr>, Vec<Attribute>)> {
    match item {
        Item::Static(s) => {
            let name_expr = struct_field_expr(&s.expr, "name");
            let cfgs = cfg_attrs(&s.attrs);
            Ok((s.ident.clone(), name_expr, cfgs))
        }
        Item::Struct(s) => Ok((s.ident.clone(), None, cfg_attrs(&s.attrs))),
        Item::Enum(e) => Ok((e.ident.clone(), None, cfg_attrs(&e.attrs))),
        Item::Fn(f) => Ok((f.sig.ident.clone(), None, cfg_attrs(&f.attrs))),
        other => Err(syn::Error::new_spanned(
            other,
            "#[component_inventory] can only be applied to a `static`, `struct`, \
             `enum`, or `fn`",
        )),
    }
}

/// If `expr` is a struct literal (`Path { field: value, ... }`), return the
/// value expression of the field named `field`.
fn struct_field_expr(expr: &Expr, field: &str) -> Option<Expr> {
    if let Expr::Struct(s) = expr {
        for fv in &s.fields {
            if let syn::Member::Named(ident) = &fv.member {
                if ident == field {
                    return Some(fv.expr.clone());
                }
            }
        }
    }
    None
}

/// The URN string literal, if it is visible at macro time (explicit string
/// `id`, or a `name` field written as a string literal).
fn literal_urn(args: &ComponentInventoryArgs, name_field_expr: &Option<Expr>) -> Option<String> {
    if let Some(id) = &args.id {
        return Some(id.value());
    }
    if let Some(Expr::Lit(ExprLit {
        lit: Lit::Str(s), ..
    })) = name_field_expr
    {
        return Some(s.value());
    }
    None
}

/// Collect `#[cfg(...)]` attributes from an item's attribute list.
fn cfg_attrs(attrs: &[Attribute]) -> Vec<Attribute> {
    attrs
        .iter()
        .filter(|a| a.path().is_ident("cfg"))
        .cloned()
        .collect()
}

// Compile-fail behavior (unknown/missing category, missing id, URN/category
// mismatch) is covered here by asserting on the generated error text rather
// than with `trybuild` UI tests: this repo runs tests via `cargo nextest` from
// a prebuilt archive in `--offline` mode, and trybuild spawns a nested `cargo`
// build for its fixture crate that cannot resolve dependencies offline. The
// expansion/argument-parsing helpers below exercise the same error paths in a
// self-contained, environment-independent way. (Argument-parsing errors are
// additionally covered by unit tests in `otap-df-engine-inventory-syntax`.)
#[cfg(test)]
mod tests {
    use super::*;

    /// Parse `#[component_inventory(<args>)]` args + an item, expand, and
    /// return the generated token stream as a string.
    fn expand(args_src: &str, item_src: &str) -> String {
        let args: ComponentInventoryArgs = syn::parse_str(args_src).expect("parse args");
        let item: Item = syn::parse_str(item_src).expect("parse item");
        expand_component_inventory(args, item).to_string()
    }

    /// Scenario: a factory `static` with a `name` field is expanded without an
    /// explicit `id`.
    /// Guarantees: the item is re-emitted and a `COMPONENT_INVENTORY` entry is
    /// generated whose `id` is the `name` field expression, with the category,
    /// description, and attributes taken from the annotation.
    #[test]
    fn factory_static_derives_id_from_name_field() {
        let out = expand(
            r#"category = Receiver, description = "OTLP receiver", attributes(port = "4317")"#,
            r#"pub static OTLP_RECEIVER: ReceiverFactory = ReceiverFactory { name: OTLP_RECEIVER_URN };"#,
        );
        // Item is re-emitted.
        assert!(out.contains("static OTLP_RECEIVER"));
        // Entry is generated with the derived id (the name field expression).
        assert!(out.contains("_COMPONENT_META_OTLP_RECEIVER"));
        assert!(out.contains("COMPONENT_INVENTORY"));
        assert!(out.contains("id : OTLP_RECEIVER_URN"));
        assert!(out.contains("Category :: Receiver"));
        assert!(
            out.contains(r#"description : :: core :: option :: Option :: Some ("OTLP receiver")"#)
        );
        assert!(out.contains(r#"("port" , "4317")"#));
    }

    /// Scenario: a `struct` (no `name` field) is expanded with an explicit `id`.
    /// Guarantees: the entry uses the explicit `id` and a `None` description.
    #[test]
    fn non_factory_struct_requires_and_uses_explicit_id() {
        let out = expand(
            r#"id = "urn:otel:extension:foo", category = Extension"#,
            r#"pub struct Foo;"#,
        );
        assert!(out.contains("struct Foo"));
        assert!(out.contains(r#"id : "urn:otel:extension:foo""#));
        assert!(out.contains(":: core :: option :: Option :: None"));
    }

    /// Scenario: a `struct` (no `name` field) is expanded without an `id`.
    /// Guarantees: the item is still emitted but a `compile_error!` is produced
    /// telling the author to supply an explicit `id`.
    #[test]
    fn non_factory_without_id_is_compile_error() {
        // A struct has no `name` field, so omitting `id` must produce an error.
        let out = expand(r#"category = Extension"#, r#"pub struct Foo;"#);
        assert!(out.contains("struct Foo")); // item still emitted
        assert!(out.contains("compile_error"));
        assert!(out.contains("requires an explicit"));
    }

    /// Scenario: an explicit literal URN `id` whose category segment disagrees
    /// with the declared `category` (e.g. `exporter` URN + `category = Receiver`).
    /// Guarantees: expansion produces a `compile_error!` reporting the URN/category
    /// mismatch.
    #[test]
    fn literal_urn_category_mismatch_is_error() {
        // Explicit literal id whose URN segment (`exporter`) disagrees with the
        // declared category (`Receiver`).
        let out = expand(
            r#"id = "urn:otel:exporter:otlp", category = Receiver"#,
            r#"pub struct Foo;"#,
        );
        assert!(out.contains("compile_error"));
        assert!(out.contains("does not match the component URN"));
    }

    /// Scenario: an explicit literal URN `id` whose category segment matches the
    /// declared `category`.
    /// Guarantees: expansion succeeds (no `compile_error!`) and uses the id.
    #[test]
    fn literal_urn_category_match_ok() {
        let out = expand(
            r#"id = "urn:otel:receiver:otlp", category = Receiver"#,
            r#"pub struct Foo;"#,
        );
        assert!(!out.contains("compile_error"));
        assert!(out.contains(r#"id : "urn:otel:receiver:otlp""#));
    }

    /// Scenario: a factory `static` whose `name` field is a `const` path (not a
    /// string literal), with a category that would mismatch if the value were known.
    /// Guarantees: the URN cross-check is skipped (the value is invisible at macro
    /// time), so expansion succeeds and the id is the const path.
    #[test]
    fn const_path_urn_skips_cross_check() {
        // When the `name` field is a const path (not a literal), the value is
        // not visible at macro time, so no cross-check runs -- even if the
        // category would mismatch, this must expand cleanly.
        let out = expand(
            r#"category = Exporter"#,
            r#"pub static E: ExporterFactory = ExporterFactory { name: SOME_URN_CONST };"#,
        );
        assert!(!out.contains("compile_error"));
        assert!(out.contains("id : SOME_URN_CONST"));
    }

    /// Scenario: the annotated item carries a `#[cfg(feature = "...")]`.
    /// Guarantees: the emitted `COMPONENT_INVENTORY` entry inherits the same
    /// `#[cfg(...)]`, so the inventory reflects exactly what was compiled.
    #[test]
    fn cfg_attr_is_propagated_to_entry() {
        let out = expand(
            r#"category = Receiver"#,
            r#"#[cfg(feature = "etw")] pub static R: ReceiverFactory = ReceiverFactory { name: URN };"#,
        );
        // The cfg appears twice: once on the re-emitted item, once on the entry.
        let occurrences = out.matches(r#"cfg (feature = "etw")"#).count();
        assert!(occurrences >= 2, "cfg not propagated to entry: {out}");
    }

    /// Scenario: several `attributes(...)` pairs are supplied.
    /// Guarantees: all pairs are emitted into the entry's `attributes` slice in
    /// the order written.
    #[test]
    fn multiple_attributes_preserved_in_order() {
        let out = expand(
            r#"category = Receiver, attributes(port = "4317", protocol = "gRPC", auth = "mTLS")"#,
            r#"pub static R: ReceiverFactory = ReceiverFactory { name: URN };"#,
        );
        let expected =
            r#"attributes : & [("port" , "4317") , ("protocol" , "gRPC") , ("auth" , "mTLS")]"#;
        assert!(out.contains(expected), "attributes not as expected: {out}");
    }

    /// Scenario: a non-factory item is expanded with category `Admin` and an explicit URN `id`.
    /// Guarantees: expansion succeeds, category `Admin` is validated, and the `id` is used.
    #[test]
    fn non_factory_admin_category_expansion_ok() {
        let out = expand(
            r#"id = "urn:otel:admin:http_server", category = Admin"#,
            r#"pub struct AdminServer;"#,
        );
        assert!(!out.contains("compile_error"));
        assert!(out.contains(r#"id : "urn:otel:admin:http_server""#));
        assert!(out.contains("Category :: Admin"));
    }
}
