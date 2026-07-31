// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tenant context validation helpers.
//!
//! Given SUV messages with an optional packed tenant context, verify that
//! certain tenant keys or key/value pairs are present (or absent) on every
//! message.
//!
//! Keys are named as the engine's `tenant_tokens` section declares them, not
//! as they appeared on the wire. Only keys declared with `retain: true` carry
//! their bytes past the receiver, so only those can be read back here; a key
//! that is undeclared or not retained reads as absent.
//!
//! For **require** checks, every message must carry the value. A single
//! message without it causes immediate failure.
//!
//! For **deny** checks, a message with no tenant context at all is acceptable
//! -- it cannot contain a forbidden key.

use otap_df_config::tenant::compiled::{TenantTokenRegistry, TenantView};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// What the validation exporter captured about tenant context, and the
/// registry needed to read it.
///
/// The packed context is positional: a key's slot is assigned by the compiled
/// registry, so the bytes cannot be interpreted without it. Without a registry
/// every key reads as absent.
pub struct TenantCapture<'a> {
    /// The engine's compiled tenant tokens, if the engine declared any.
    pub registry: Option<&'a Arc<TenantTokenRegistry>>,
    /// One entry per SUV message, in arrival order.
    pub contexts: &'a [Option<Arc<[u64]>>],
}

impl TenantCapture<'_> {
    /// Reads one key's retained value out of one message's context.
    ///
    /// Returns `None` when the message carried no tenant context, the engine
    /// declared no tenant tokens, the key is undeclared, or the key resolved
    /// to no value on this request.
    fn value(&self, index: usize, key: &str) -> Option<&[u8]> {
        let registry = self.registry?;
        let words = self.contexts.get(index)?.as_ref()?;
        let key = registry.key_id(key)?;
        registry.retained_value(&TenantView::new(words.as_ref()), key)
    }
}

/// A key/value pair for tenant context assertions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TenantKeyValue {
    /// Tenant token key name, as `tenant_tokens` declares it.
    pub key: String,
    /// Expected value (UTF-8 text).
    pub value: String,
}

impl TenantKeyValue {
    /// Create a new tenant key/value pair.
    #[must_use]
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

/// Validate that **every** SUV message carries a value for all `keys`.
///
/// Returns `false` when there are no messages to validate, or when any
/// message is missing any of the keys.
///
/// Returns `true` when `keys` is empty (nothing to check).
#[must_use]
pub fn validate_tenant_require_keys(capture: &TenantCapture<'_>, keys: &[String]) -> bool {
    if keys.is_empty() {
        return true;
    }
    if capture.contexts.is_empty() {
        return false;
    }

    (0..capture.contexts.len())
        .all(|index| keys.iter().all(|key| capture.value(index, key).is_some()))
}

/// Validate that **every** SUV message carries each of the given key/value
/// pairs.
///
/// Values are compared as bytes, so the comparison is exact: this is the same
/// equality the tenant matcher itself uses to decide conditions. A key holds
/// at most one value, so there is no "any of several" case to consider.
///
/// Returns `false` when there are no messages to validate, or when any
/// message is missing a key or holds a different value.
///
/// Returns `true` when `pairs` is empty (nothing to check).
#[must_use]
pub fn validate_tenant_require_key_values(
    capture: &TenantCapture<'_>,
    pairs: &[TenantKeyValue],
) -> bool {
    if pairs.is_empty() {
        return true;
    }
    if capture.contexts.is_empty() {
        return false;
    }

    (0..capture.contexts.len()).all(|index| {
        pairs
            .iter()
            .all(|pair| capture.value(index, &pair.key) == Some(pair.value.as_bytes()))
    })
}

/// Validate that no SUV message carries a value for any of `keys`.
///
/// A message with no tenant context is acceptable -- it cannot contain a
/// forbidden key.
#[must_use]
pub fn validate_tenant_deny_keys(capture: &TenantCapture<'_>, keys: &[String]) -> bool {
    if keys.is_empty() {
        return true;
    }

    (0..capture.contexts.len())
        .all(|index| keys.iter().all(|key| capture.value(index, key).is_none()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::tenant::compiled::{TenantTokenRegistryBuilder, TokenScratch};
    use otap_df_config::tenant::{Extractor, TenantTokenSpec, TenantTokens};

    const KEYS: [&str; 2] = ["tenant_id", "request_id"];

    /// A registry declaring [`KEYS`], each read from `x-<key>` and retained.
    fn registry() -> Arc<TenantTokenRegistry> {
        let mut tokens = TenantTokens::default();
        for key in KEYS {
            let _ = tokens.insert(
                key.to_owned(),
                TenantTokenSpec {
                    extractors: vec![Extractor::TransportHeader {
                        key: key.to_owned(),
                        transport_header: format!("x-{key}"),
                        retain: true,
                        bag: false,
                    }],
                },
            );
        }
        let mut builder = TenantTokenRegistryBuilder::new();
        builder.add_tokens(&tokens).expect("tokens compile");
        Arc::new(builder.build(1).expect("layout fits"))
    }

    /// One message's context, as a receiver would have packed it.
    fn context(pairs: &[(&str, &str)]) -> Option<Arc<[u64]>> {
        use otap_df_config::tenant::compiled::TokenInputs;

        let mut scratch = TokenScratch::new();
        registry().resolve(
            &mut scratch,
            TokenInputs::new(pairs.iter().map(|(k, v)| (*k, v.as_bytes()))),
        )
    }

    fn capture<'a>(
        reg: &'a Arc<TenantTokenRegistry>,
        contexts: &'a [Option<Arc<[u64]>>],
    ) -> TenantCapture<'a> {
        TenantCapture {
            registry: Some(reg),
            contexts,
        }
    }

    /// Scenario: every message carries both required keys.
    /// Guarantees: a require-keys check passes only when the value survived
    /// the whole pipeline on every message, not just the first.
    #[test]
    fn require_keys_passes_when_all_present() {
        let reg = registry();
        let ctxs = vec![
            context(&[("x-tenant_id", "acme"), ("x-request_id", "abc")]),
            context(&[("x-tenant_id", "acme"), ("x-request_id", "def")]),
        ];
        assert!(validate_tenant_require_keys(
            &capture(&reg, &ctxs),
            &["tenant_id".into(), "request_id".into()],
        ));
    }

    /// Scenario: one message is missing a required key.
    /// Guarantees: a single bad message fails the check, so a partial
    /// propagation regression cannot pass by averaging out.
    #[test]
    fn require_keys_fails_when_one_message_is_missing_a_key() {
        let reg = registry();
        let ctxs = vec![
            context(&[("x-tenant_id", "acme"), ("x-request_id", "abc")]),
            context(&[("x-tenant_id", "acme")]),
        ];
        assert!(!validate_tenant_require_keys(
            &capture(&reg, &ctxs),
            &["tenant_id".into(), "request_id".into()],
        ));
    }

    /// Scenario: no messages arrived, or none carried tenant context.
    /// Guarantees: require checks fail closed rather than passing vacuously.
    #[test]
    fn require_fails_closed_with_nothing_to_read() {
        let reg = registry();
        let empty: Vec<Option<Arc<[u64]>>> = vec![];
        assert!(!validate_tenant_require_keys(
            &capture(&reg, &empty),
            &["tenant_id".into()],
        ));

        let none = vec![None, None];
        assert!(!validate_tenant_require_keys(
            &capture(&reg, &none),
            &["tenant_id".into()],
        ));
        assert!(!validate_tenant_require_key_values(
            &capture(&reg, &none),
            &[TenantKeyValue::new("tenant_id", "acme")],
        ));
    }

    /// Scenario: the engine declared no tenant tokens at all.
    /// Guarantees: values cannot be read without the registry that assigned
    /// their slots, so require fails rather than guessing at the bytes.
    #[test]
    fn require_fails_closed_without_a_registry() {
        let ctxs = vec![context(&[("x-tenant_id", "acme")])];
        let capture = TenantCapture {
            registry: None,
            contexts: &ctxs,
        };
        assert!(!validate_tenant_require_keys(
            &capture,
            &["tenant_id".into()]
        ));
    }

    /// Scenario: the required value differs from the captured one.
    /// Guarantees: values are compared exactly, so a value that was truncated
    /// or rewritten in flight fails.
    #[test]
    fn require_key_values_compares_exactly() {
        let reg = registry();
        let ctxs = vec![context(&[("x-tenant_id", "acme")])];
        assert!(validate_tenant_require_key_values(
            &capture(&reg, &ctxs),
            &[TenantKeyValue::new("tenant_id", "acme")],
        ));
        assert!(!validate_tenant_require_key_values(
            &capture(&reg, &ctxs),
            &[TenantKeyValue::new("tenant_id", "acm")],
        ));
        assert!(!validate_tenant_require_key_values(
            &capture(&reg, &ctxs),
            &[TenantKeyValue::new("tenant_id", "ACME")],
        ));
    }

    /// Scenario: a key that must never reach the capture pipeline.
    /// Guarantees: deny fails when the key holds a value and passes when it
    /// does not, including when the message carries no context at all.
    #[test]
    fn deny_keys_tracks_presence() {
        let reg = registry();
        let present = vec![context(&[("x-tenant_id", "acme")])];
        assert!(!validate_tenant_deny_keys(
            &capture(&reg, &present),
            &["tenant_id".into()],
        ));
        assert!(validate_tenant_deny_keys(
            &capture(&reg, &present),
            &["request_id".into()],
        ));

        let absent = vec![None];
        assert!(validate_tenant_deny_keys(
            &capture(&reg, &absent),
            &["tenant_id".into()],
        ));
    }

    /// Scenario: an assertion naming a key the engine never declared.
    /// Guarantees: an undeclared key reads as absent, so deny passes and
    /// require fails -- neither silently succeeds on a typo'd key name.
    #[test]
    fn undeclared_keys_read_as_absent() {
        let reg = registry();
        let ctxs = vec![context(&[("x-tenant_id", "acme")])];
        assert!(validate_tenant_deny_keys(
            &capture(&reg, &ctxs),
            &["never_declared".into()],
        ));
        assert!(!validate_tenant_require_keys(
            &capture(&reg, &ctxs),
            &["never_declared".into()],
        ));
    }

    /// Scenario: an assertion list with no keys in it.
    /// Guarantees: an empty assertion is a no-op rather than a failure, so
    /// scenarios that configure no tenant checks still pass.
    #[test]
    fn empty_keys_always_passes() {
        let reg = registry();
        let ctxs = vec![None];
        assert!(validate_tenant_require_keys(&capture(&reg, &ctxs), &[]));
        assert!(validate_tenant_require_key_values(
            &capture(&reg, &ctxs),
            &[]
        ));
        assert!(validate_tenant_deny_keys(&capture(&reg, &ctxs), &[]));
    }
}
