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

/// Component category (RFC 0001).
///
/// This enum is the **single source of truth** for the set of accepted
/// categories and their URN segments. It lives in this leaf crate (which
/// depends only on `syn`/`proc-macro2`) so that all three consumers reference
/// the same type and cannot drift:
///
/// - `otap-df-engine` re-exports it as `otap_df_engine::inventory::Category`,
///   the runtime type stored in every `ComponentMeta`;
/// - the `#[component_inventory]` proc macro validates the `category = <Ident>`
///   argument and emits `Category::<Variant>` referencing that re-export; and
/// - the `cargo xtask component-inventory` scanner uses the same variants when
///   parsing annotations out of source.
///
/// Adding a variant here updates every consumer through the type system: the
/// parser's accepted set, the URN cross-check, and the runtime enum all derive
/// from these variants, so there is no separate list to keep in sync.
///
/// The macro accepts a bare identifier (e.g. `Receiver`) and rejects unknown
/// variants at compile time, preventing misspellings like `Reciever` from
/// silently corrupting the inventory. For factory components the macro also
/// validates the category against the URN's middle segment (e.g.
/// `urn:otel:`**`receiver`**`:otlp`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Category {
    /// A receiver: ingests telemetry into a pipeline (`urn:...:receiver:...`).
    Receiver,
    /// An exporter: emits telemetry out of a pipeline (`urn:...:exporter:...`).
    Exporter,
    /// A processor: transforms telemetry in a pipeline (`urn:...:processor:...`).
    Processor,
    /// An extension: shared, non-pipeline functionality (`urn:...:extension:...`).
    Extension,
    /// Built-in HTTP/gRPC admin server (`urn:...:admin:...`).
    Admin,
    /// Pipeline controller or OpAMP engine (`urn:...:controller:...`).
    Controller,
    /// Command line tooling (`urn:...:cli:...`).
    Cli,
    /// Core infrastructure subsystem (`urn:...:subsystem:...`).
    Subsystem,
    /// Safety guardrails such as memory limiter (`urn:...:safety:...`).
    Safety,
}

impl Category {
    /// Every category variant, in declaration order.
    ///
    /// Used to validate the `category` identifier and to build the
    /// "expected one of: ..." error, so both derive from the enum rather than a
    /// separate list.
    pub const ALL: &'static [Category] = &[
        Category::Receiver,
        Category::Exporter,
        Category::Processor,
        Category::Extension,
        Category::Admin,
        Category::Controller,
        Category::Cli,
        Category::Subsystem,
        Category::Safety,
    ];

    /// The Rust identifier for this variant (e.g. `Receiver` -> `"Receiver"`).
    #[must_use]
    pub const fn ident_str(self) -> &'static str {
        match self {
            Category::Receiver => "Receiver",
            Category::Exporter => "Exporter",
            Category::Processor => "Processor",
            Category::Extension => "Extension",
            Category::Admin => "Admin",
            Category::Controller => "Controller",
            Category::Cli => "Cli",
            Category::Subsystem => "Subsystem",
            Category::Safety => "Safety",
        }
    }

    /// The URN category segment for this variant (e.g. `Receiver` -> `"receiver"`).
    ///
    /// Used to cross-check `category` against a component's URN and by the
    /// inventory tooling.
    #[must_use]
    pub const fn urn_segment(self) -> &'static str {
        match self {
            Category::Receiver => "receiver",
            Category::Exporter => "exporter",
            Category::Processor => "processor",
            Category::Extension => "extension",
            Category::Admin => "admin",
            Category::Controller => "controller",
            Category::Cli => "cli",
            Category::Subsystem => "subsystem",
            Category::Safety => "safety",
        }
    }

    /// Parse a bare category identifier (e.g. `"Receiver"`) into a [`Category`].
    ///
    /// Returns `None` for an unknown identifier.
    #[must_use]
    pub fn from_ident_str(ident: &str) -> Option<Category> {
        Category::ALL
            .iter()
            .copied()
            .find(|cat| cat.ident_str() == ident)
    }

    /// A comma-separated list of every accepted category identifier, for use in
    /// error messages (e.g. `"Receiver, Exporter, ..."`).
    #[must_use]
    pub fn expected_list() -> String {
        Category::ALL
            .iter()
            .map(|cat| cat.ident_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl core::fmt::Display for Category {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.urn_segment())
    }
}

/// Map a `Category` identifier to its URN segment, for the URN cross-check.
///
/// Returns `None` for an unknown category. Thin wrapper over [`Category`]; the
/// mapping itself is defined once in [`Category::urn_segment`].
#[must_use]
pub fn category_urn_segment(cat: &str) -> Option<&'static str> {
    Category::from_ident_str(cat).map(Category::urn_segment)
}

/// Parsed arguments from `#[component_inventory(...)]`.
#[derive(Debug)]
pub struct ComponentInventoryArgs {
    /// Explicit `id = "..."` (required only when the annotated item is not a
    /// factory static with a `name` field).
    pub id: Option<LitStr>,
    /// `category = <Ident>` (required). Validated against [`Category`].
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
        if Category::from_ident_str(&cat_str).is_none() {
            return Err(syn::Error::new_spanned(
                &category,
                format!(
                    "unknown component category `{cat_str}`; expected one of: {}",
                    Category::expected_list()
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

    /// The declared category as a typed [`Category`].
    ///
    /// Always `Some` after a successful parse, because [`Self::parse`] rejects
    /// any identifier that is not a known category; returns `None` only if
    /// constructed by other means with an invalid identifier.
    #[must_use]
    pub fn category(&self) -> Option<Category> {
        Category::from_ident_str(&self.category_str())
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
        let Some(seg) = self.category().map(Category::urn_segment) else {
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

    /// Scenario: `Category::from_ident_str` round-trips every variant's
    /// `ident_str`, and rejects an unknown identifier.
    /// Guarantees: the parser's accepted set is exactly `Category::ALL`, so a
    /// new variant is automatically accepted and no stale string list can drift.
    #[test]
    fn from_ident_str_round_trips_all_variants() {
        for &cat in Category::ALL {
            assert_eq!(Category::from_ident_str(cat.ident_str()), Some(cat));
        }
        assert_eq!(Category::from_ident_str("Reciever"), None);
    }

    /// Scenario: every `Category` variant is asked for its `urn_segment` and
    /// `ident_str`.
    /// Guarantees: the URN segment is the lowercase of the identifier, keeping
    /// the single enum consistent with the URN cross-check for all variants.
    #[test]
    fn urn_segment_is_lowercase_ident_for_all_variants() {
        for &cat in Category::ALL {
            assert_eq!(cat.urn_segment(), cat.ident_str().to_lowercase());
        }
    }

    /// Scenario: `Category::expected_list` is rendered for an error message.
    /// Guarantees: it lists every variant identifier in declaration order, so
    /// the "expected one of" diagnostic derives from the enum, not a copy.
    #[test]
    fn expected_list_covers_all_variants_in_order() {
        assert_eq!(
            Category::expected_list(),
            "Receiver, Exporter, Processor, Extension, Admin, Controller, Cli, Subsystem, Safety"
        );
    }
}
