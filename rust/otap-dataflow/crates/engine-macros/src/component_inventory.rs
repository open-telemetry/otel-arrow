// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the `#[component_inventory]` attribute macro (RFC 0001).
//!
//! The macro annotates a security-relevant component and emits one
//! `ComponentMeta` entry into the `otap_df_engine::inventory::COMPONENT_INVENTORY`
//! distributed slice at link time, mirroring the `#[capability]` ->
//! `KNOWN_CAPABILITIES` mechanism. The annotated item is re-emitted unchanged.
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

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Attribute, Expr, ExprLit, Ident, Item, Lit, LitStr, Meta, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
};

/// Categories accepted by the macro in Phase 1 (RFC 0001).
///
/// Kept in sync with `otap_df_engine::inventory::Category`. Only the four
/// factory categories ship in Phase 1; the non-factory categories are deferred
/// to Phase 2.
const KNOWN_CATEGORIES: &[&str] = &["Receiver", "Exporter", "Processor", "Extension"];

/// Map a `Category` identifier to its URN segment, for the URN cross-check.
fn category_urn_segment(cat: &str) -> Option<&'static str> {
    match cat {
        "Receiver" => Some("receiver"),
        "Exporter" => Some("exporter"),
        "Processor" => Some("processor"),
        "Extension" => Some("extension"),
        _ => None,
    }
}

/// Parsed arguments from `#[component_inventory(...)]`.
pub(crate) struct ComponentInventoryArgs {
    /// Explicit `id = "..."` (required only when the annotated item is not a
    /// factory static with a `name` field).
    pub id: Option<LitStr>,
    /// `category = <Ident>` (required). Validated against `KNOWN_CATEGORIES`.
    pub category: Ident,
    /// Optional `description = "..."`.
    pub description: Option<LitStr>,
    /// Optional `attributes(key = "value", ...)` list.
    pub attributes: Vec<(LitStr, LitStr)>,
}

impl Parse for ComponentInventoryArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut id: Option<LitStr> = None;
        let mut category: Option<Ident> = None;
        let mut description: Option<LitStr> = None;
        let mut attributes: Vec<(LitStr, LitStr)> = Vec::new();

        let metas = Punctuated::<Meta, Comma>::parse_terminated(input)?;
        for meta in metas {
            match meta {
                // key = value forms: id, category, description.
                Meta::NameValue(nv) => {
                    let key = nv
                        .path
                        .get_ident()
                        .map(ToString::to_string)
                        .unwrap_or_default();
                    match key.as_str() {
                        "id" => id = Some(expect_str(&nv.value, "id")?),
                        "description" => {
                            description = Some(expect_str(&nv.value, "description")?);
                        }
                        "category" => {
                            // `category = Receiver` parses as a path expression.
                            let ident = match &nv.value {
                                Expr::Path(p) => p.path.get_ident().cloned(),
                                _ => None,
                            };
                            category = Some(ident.ok_or_else(|| {
                                syn::Error::new_spanned(
                                    &nv.value,
                                    "`category` must be a bare identifier, e.g. `Receiver`",
                                )
                            })?);
                        }
                        _ => {
                            return Err(syn::Error::new_spanned(
                                nv.path,
                                "unknown `#[component_inventory]` attribute; expected \
                                 `id`, `category`, `description`, or `attributes(...)`",
                            ));
                        }
                    }
                }
                // attributes(key = "value", ...) form.
                Meta::List(list) if list.path.is_ident("attributes") => {
                    let pairs =
                        list.parse_args_with(Punctuated::<AttrPair, Comma>::parse_terminated)?;
                    for pair in pairs {
                        attributes.push((pair.key, pair.value));
                    }
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "unknown `#[component_inventory]` attribute; expected \
                         `id`, `category`, `description`, or `attributes(...)`",
                    ));
                }
            }
        }

        let category =
            category.ok_or_else(|| input.error("missing required `category = <Category>`"))?;

        // Validate the category identifier at macro time so a misspelling like
        // `Reciever` is a clear compile error rather than a silent bad entry.
        let cat_str = category.to_string();
        if !KNOWN_CATEGORIES.contains(&cat_str.as_str()) {
            return Err(syn::Error::new_spanned(
                &category,
                format!(
                    "unknown component category `{cat_str}`; expected one of: {}",
                    KNOWN_CATEGORIES.join(", ")
                ),
            ));
        }

        Ok(ComponentInventoryArgs {
            id,
            category,
            description,
            attributes,
        })
    }
}

/// One `key = "value"` pair inside `attributes(...)`.
struct AttrPair {
    key: LitStr,
    value: LitStr,
}

impl Parse for AttrPair {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        // Accept either `port = "4317"` (bare ident key) or `"port" = "4317"`.
        let key = if input.peek(LitStr) {
            input.parse::<LitStr>()?
        } else {
            let ident = input.parse::<Ident>()?;
            LitStr::new(&ident.to_string(), ident.span())
        };
        let _eq: Token![=] = input.parse()?;
        let value = input.parse::<LitStr>()?;
        Ok(AttrPair { key, value })
    }
}

/// Extract a string literal from a `key = <expr>` value.
fn expect_str(expr: &Expr, key: &str) -> syn::Result<LitStr> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(s), ..
    }) = expr
    {
        Ok(s.clone())
    } else {
        Err(syn::Error::new_spanned(
            expr,
            format!("`{key}` must be a string literal"),
        ))
    }
}

/// Expand `#[component_inventory(...)]` on `item`.
///
/// Re-emits `item` unchanged and appends one `COMPONENT_INVENTORY` slice entry.
pub(crate) fn expand_component_inventory(args: ComponentInventoryArgs, item: Item) -> TokenStream {
    match try_expand(args, &item) {
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

fn try_expand(args: ComponentInventoryArgs, item: &Item) -> syn::Result<TokenStream> {
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
    // cross-check with resolved values in a later phase.
    if let Some(seg) = category_urn_segment(&args.category.to_string()) {
        if let Some(urn) = literal_urn(&args, &name_field_expr) {
            if let Some(mid) = urn.split(':').nth(2) {
                if mid != seg {
                    return Err(syn::Error::new_spanned(
                        &args.category,
                        format!(
                            "category `{}` (URN segment `{seg}`) does not match the \
                             component URN `{urn}` (segment `{mid}`)",
                            args.category
                        ),
                    ));
                }
            }
        }
    }

    let category = &args.category;
    let description_expr: TokenStream = match &args.description {
        Some(d) => quote! { ::core::option::Option::Some(#d) },
        None => quote! { ::core::option::Option::None },
    };
    let attr_keys = args.attributes.iter().map(|(k, _)| k);
    let attr_vals = args.attributes.iter().map(|(_, v)| v);

    // Unique static name for the generated entry, derived from the item ident.
    let entry_ident = format_ident!("_COMPONENT_META_{}", item_ident);

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
        #[::linkme::distributed_slice(::otap_df_engine::inventory::COMPONENT_INVENTORY)]
        #[linkme(crate = ::linkme)]
        static #entry_ident: ::otap_df_engine::inventory::ComponentMeta =
            ::otap_df_engine::inventory::ComponentMeta {
                id: #id_expr,
                category: ::otap_df_engine::inventory::Category::#category,
                description: #description_expr,
                file: ::core::file!(),
                line: ::core::line!(),
                attributes: &[ #( (#attr_keys, #attr_vals) ),* ],
            };
    })
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

    /// Parse only the args, returning the error text on failure.
    fn parse_args_err(args_src: &str) -> Option<String> {
        syn::parse_str::<ComponentInventoryArgs>(args_src)
            .err()
            .map(|e| e.to_string())
    }

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

    #[test]
    fn non_factory_without_id_is_compile_error() {
        // A struct has no `name` field, so omitting `id` must produce an error.
        let out = expand(r#"category = Extension"#, r#"pub struct Foo;"#);
        assert!(out.contains("struct Foo")); // item still emitted
        assert!(out.contains("compile_error"));
        assert!(out.contains("requires an explicit"));
    }

    #[test]
    fn unknown_category_is_rejected() {
        let err = parse_args_err(r#"category = Reciever"#).expect("should error");
        assert!(err.contains("unknown component category"));
        assert!(err.contains("Reciever"));
    }

    #[test]
    fn missing_category_is_rejected() {
        let err = parse_args_err(r#"description = "x""#).expect("should error");
        assert!(err.contains("missing required `category"));
    }

    #[test]
    fn unknown_key_is_rejected() {
        let err = parse_args_err(r#"category = Receiver, bogus = "x""#).expect("should error");
        assert!(err.contains("unknown `#[component_inventory]` attribute"));
    }

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

    #[test]
    fn literal_urn_category_match_ok() {
        let out = expand(
            r#"id = "urn:otel:receiver:otlp", category = Receiver"#,
            r#"pub struct Foo;"#,
        );
        assert!(!out.contains("compile_error"));
        assert!(out.contains(r#"id : "urn:otel:receiver:otlp""#));
    }

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
}
