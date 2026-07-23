// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared parser for the `#[component_inventory(...)]` attribute grammar
//! (RFC 0001).
//!
//! This crate holds the **single** definition of the attribute-argument syntax
//! so that the two consumers cannot drift:
//!
//! - the `#[component_inventory]` proc macro (`otap-df-engine-macros`), which
//!   parses its attribute tokens and emits a `COMPONENT_INVENTORY` entry; and
//! - the `cargo xtask component-inventory` scanner, which parses the same
//!   attribute out of a `syn`-parsed source file to build the inventory
//!   baseline for threat-model drift detection.
//!
//! Because both sides parse with the *same* [`ComponentInventoryArgs`] `Parse`
//! implementation, a change to the accepted syntax automatically applies to
//! both, and neither can silently disagree about what a given annotation means.
//!
//! This is an ordinary library crate (not a proc-macro crate) so it can be a
//! dependency of both the proc-macro crate and the xtask binary.

use proc_macro2::Span;
use syn::{
    Expr, ExprLit, Ident, Lit, LitStr, Meta, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
};

/// Categories accepted by the `#[component_inventory]` macro (RFC 0001).
///
/// Kept in sync with `otap_df_engine::inventory::Category`.
pub const KNOWN_CATEGORIES: &[&str] = &[
    "Receiver",
    "Exporter",
    "Processor",
    "Extension",
    "Admin",
    "Controller",
    "Cli",
    "Subsystem",
    "Safety",
];

/// Map a `Category` identifier to its URN segment, for the URN cross-check.
///
/// Returns `None` for an unknown category. Kept in sync with
/// `otap_df_engine::inventory::Category::urn_segment`.
#[must_use]
pub fn category_urn_segment(cat: &str) -> Option<&'static str> {
    match cat {
        "Receiver" => Some("receiver"),
        "Exporter" => Some("exporter"),
        "Processor" => Some("processor"),
        "Extension" => Some("extension"),
        "Admin" => Some("admin"),
        "Controller" => Some("controller"),
        "Cli" => Some("cli"),
        "Subsystem" => Some("subsystem"),
        "Safety" => Some("safety"),
        _ => None,
    }
}

/// Parsed arguments from `#[component_inventory(...)]`.
#[derive(Debug)]
pub struct ComponentInventoryArgs {
    /// Explicit `id = "..."` (required only when the annotated item is not a
    /// factory static with a `name` field).
    pub id: Option<LitStr>,
    /// `category = <Ident>` (required). Validated against [`KNOWN_CATEGORIES`].
    pub category: Ident,
    /// Optional `description = "..."`.
    pub description: Option<LitStr>,
    /// Optional `attributes(key = "value", ...)` list, in the order written.
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

        // Validate the category identifier so a misspelling like `Reciever` is a
        // clear error rather than a silently bad entry.
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

impl ComponentInventoryArgs {
    /// The `category` identifier as a string (e.g. `"Receiver"`).
    #[must_use]
    pub fn category_str(&self) -> String {
        self.category.to_string()
    }

    /// The explicit `id` value, if one was supplied.
    #[must_use]
    pub fn id_value(&self) -> Option<String> {
        self.id.as_ref().map(LitStr::value)
    }

    /// The `description` value, if one was supplied.
    #[must_use]
    pub fn description_value(&self) -> Option<String> {
        self.description.as_ref().map(LitStr::value)
    }

    /// The `attributes(...)` pairs as owned `(key, value)` strings, in order.
    #[must_use]
    pub fn attribute_pairs(&self) -> Vec<(String, String)> {
        self.attributes
            .iter()
            .map(|(k, v)| (k.value(), v.value()))
            .collect()
    }

    /// Cross-check a *literal* URN against the declared category.
    ///
    /// Returns an error only when the URN's middle segment is known and
    /// disagrees with the category's [`category_urn_segment`]. When the URN is
    /// not a literal (the common `const`-path factory case) the caller passes
    /// `None` and no check runs here.
    pub fn check_urn_category(&self, literal_urn: Option<&str>) -> syn::Result<()> {
        let Some(seg) = category_urn_segment(&self.category_str()) else {
            return Ok(());
        };
        let Some(urn) = literal_urn else {
            return Ok(());
        };
        if let Some(mid) = urn.split(':').nth(2) {
            if mid != seg {
                return Err(syn::Error::new_spanned(
                    &self.category,
                    format!(
                        "category `{}` (URN segment `{seg}`) does not match the \
                         component URN `{urn}` (segment `{mid}`)",
                        self.category
                    ),
                ));
            }
        }
        Ok(())
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

/// Convenience: emit a `Span::call_site()`-spanned error (used by callers that
/// need to raise their own diagnostics with the shared error type).
#[must_use]
pub fn call_site_error(msg: &str) -> syn::Error {
    syn::Error::new(Span::call_site(), msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a full annotation with id, category, description and attributes
    /// is parsed.
    /// Guarantees: every field is captured, attribute order is preserved, and
    /// the accessor helpers return owned values.
    #[test]
    fn parses_full_annotation() {
        let args: ComponentInventoryArgs = syn::parse_str(
            r#"id = "urn:otel:receiver:otlp", category = Receiver, description = "OTLP", attributes(port = "4317", auth = "mTLS")"#,
        )
        .expect("parse");
        assert_eq!(args.id_value(), Some("urn:otel:receiver:otlp".to_string()));
        assert_eq!(args.category_str(), "Receiver");
        assert_eq!(args.description_value(), Some("OTLP".to_string()));
        assert_eq!(
            args.attribute_pairs(),
            vec![
                ("port".to_string(), "4317".to_string()),
                ("auth".to_string(), "mTLS".to_string()),
            ]
        );
    }

    /// Scenario: a misspelled category is parsed.
    /// Guarantees: parsing fails with an "unknown component category" error.
    #[test]
    fn rejects_unknown_category() {
        let err = syn::parse_str::<ComponentInventoryArgs>("category = Reciever")
            .expect_err("should error");
        assert!(err.to_string().contains("unknown component category"));
    }

    /// Scenario: the required category is omitted.
    /// Guarantees: parsing fails with a "missing required `category`" error.
    #[test]
    fn rejects_missing_category() {
        let err = syn::parse_str::<ComponentInventoryArgs>(r#"description = "x""#)
            .expect_err("should error");
        assert!(err.to_string().contains("missing required `category"));
    }

    /// Scenario: an unrecognized key is supplied.
    /// Guarantees: parsing fails with an "unknown `#[component_inventory]`
    /// attribute" error.
    #[test]
    fn rejects_unknown_key() {
        let err = syn::parse_str::<ComponentInventoryArgs>(r#"category = Receiver, bogus = "x""#)
            .expect_err("should error");
        assert!(
            err.to_string()
                .contains("unknown `#[component_inventory]` attribute")
        );
    }

    /// Scenario: a literal URN whose segment disagrees with the category is
    /// cross-checked.
    /// Guarantees: `check_urn_category` returns an error naming both segments.
    #[test]
    fn urn_category_mismatch_detected() {
        let args: ComponentInventoryArgs =
            syn::parse_str("id = \"urn:otel:exporter:otlp\", category = Receiver").expect("parse");
        let err = args
            .check_urn_category(Some("urn:otel:exporter:otlp"))
            .expect_err("should mismatch");
        assert!(err.to_string().contains("does not match the component URN"));
    }

    /// Scenario: a literal URN whose segment matches the category is
    /// cross-checked, and the const-path case (None) is cross-checked.
    /// Guarantees: both return `Ok` (no false positives; const path is skipped).
    #[test]
    fn urn_category_match_and_const_path_ok() {
        let args: ComponentInventoryArgs =
            syn::parse_str("id = \"urn:otel:receiver:otlp\", category = Receiver").expect("parse");
        assert!(
            args.check_urn_category(Some("urn:otel:receiver:otlp"))
                .is_ok()
        );
        assert!(args.check_urn_category(None).is_ok());
    }

    /// Scenario: `category_urn_segment` is queried for known and unknown names.
    /// Guarantees: known categories map to their lowercase segment; unknown
    /// returns `None`.
    #[test]
    fn category_segment_mapping() {
        assert_eq!(category_urn_segment("Receiver"), Some("receiver"));
        assert_eq!(category_urn_segment("Safety"), Some("safety"));
        assert_eq!(category_urn_segment("Nope"), None);
    }
}
