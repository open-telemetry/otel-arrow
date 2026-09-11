// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Transport header validation helpers.
//!
//! Given SUV messages with optional [`TransportHeaders`], verify that certain
//! header keys or key/value pairs are present (or absent) on every message.
//!
//! For **require** checks (`require_keys`, `require_key_values`), every
//! message must carry transport headers (`Some`). A single `None` entry
//! (a signal that arrived without headers) causes immediate failure.
//!
//! For **deny** checks, `None` entries are acceptable -- a signal without
//! headers cannot contain a forbidden key.

use otel_arrow_dfe_config::ContextEntryName;
use otel_arrow_dfe_config::transport_headers::TransportHeaders;
use serde::{Deserialize, Serialize};

/// A key/value pair for transport header assertions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransportHeaderKeyValue {
    /// Header key (stored/logical name).
    pub key: ContextEntryName,
    /// Expected header value (UTF-8 text).
    pub value: String,
}

impl TransportHeaderKeyValue {
    /// Create a new transport header key/value pair.
    #[must_use]
    pub fn new(key: ContextEntryName, value: impl Into<String>) -> Self {
        Self {
            key,
            value: value.into(),
        }
    }

    /// Validates the header name before creating the pair.
    pub fn try_new<K>(key: K, value: impl Into<String>) -> Result<Self, K::Error>
    where
        K: TryInto<ContextEntryName>,
    {
        Ok(Self::new(key.try_into()?, value))
    }
}

/// Validate that **every** SUV message has transport headers and that each
/// set of headers contains all specified keys.
///
/// Returns `false` when:
/// - `suv` is empty (no messages to validate),
/// - any entry is `None` (a signal arrived without transport headers), or
/// - any `Some(TransportHeaders)` is missing a required key.
///
/// Returns `true` when `keys` is empty (nothing to check).
#[must_use]
pub fn validate_transport_header_require_keys(
    suv: &[Option<TransportHeaders>],
    keys: &[ContextEntryName],
) -> bool {
    if keys.is_empty() {
        return true;
    }
    if suv.is_empty() {
        return false;
    }

    for entry in suv {
        let headers = match entry {
            Some(h) => h,
            None => return false,
        };
        for key in keys {
            // check that key exists
            if headers.find_by_name(key.as_ref()).next().is_none() {
                return false;
            }
        }
    }

    true
}

/// Validate that **every** SUV message has transport headers and that each
/// set of headers contains all specified key/value pairs.
///
/// Values are compared as UTF-8 text (case-sensitive).
///
/// Returns `false` when:
/// - `suv` is empty (no messages to validate),
/// - any entry is `None` (a signal arrived without transport headers), or
/// - any `Some(TransportHeaders)` is missing a required key or has a
///   mismatched value.
///
/// Returns `true` when `pairs` is empty (nothing to check).
#[must_use]
pub fn validate_transport_header_require_key_values(
    suv: &[Option<TransportHeaders>],
    pairs: &[TransportHeaderKeyValue],
) -> bool {
    if pairs.is_empty() {
        return true;
    }
    if suv.is_empty() {
        return false;
    }

    for entry in suv {
        let headers = match entry {
            Some(h) => h,
            None => return false,
        };
        'pairs: for pair in pairs {
            for header in headers.find_by_name(pair.key.as_ref()) {
                match header.value_as_str() {
                    Some(value_str) if value_str == pair.value => continue 'pairs,
                    _ => continue,
                }
            }
            return false;
        }
    }

    true
}

/// Validate that no SUV message carrying transport headers contains any of
/// the specified keys.
///
/// `None` entries (signals without transport headers) are acceptable -- a
/// signal that never received headers cannot contain a forbidden key.
#[must_use]
pub fn validate_transport_header_deny_keys(
    suv: &[Option<TransportHeaders>],
    keys: &[ContextEntryName],
) -> bool {
    if keys.is_empty() {
        return true;
    }

    for headers in suv.iter().flatten() {
        for key in keys {
            if headers.find_by_name(key.as_ref()).next().is_some() {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::transport_headers::{TransportHeader, TransportHeaders};

    fn context_name(raw: &str) -> ContextEntryName {
        ContextEntryName::try_from(raw).expect("valid test context entry name")
    }

    fn context_names(raw: &[&str]) -> Vec<ContextEntryName> {
        raw.iter().map(|name| context_name(name)).collect()
    }

    fn key_value(key: &str, value: &str) -> TransportHeaderKeyValue {
        TransportHeaderKeyValue::try_new(key, value).expect("valid test context entry name")
    }

    fn make_headers(entries: &[(&str, &str)]) -> TransportHeaders {
        let mut headers = TransportHeaders::default();
        for (name, value) in entries {
            headers.push(TransportHeader::text(context_name(name), value.as_bytes()));
        }
        headers
    }

    #[test]
    fn require_keys_passes_when_all_present() {
        let headers = make_headers(&[("x-tenant-id", "acme"), ("x-request-id", "abc")]);
        let suv = vec![Some(headers)];
        assert!(validate_transport_header_require_keys(
            &suv,
            &context_names(&["x-tenant-id", "x-request-id"]),
        ));
    }

    #[test]
    fn require_keys_fails_when_key_missing() {
        let headers = make_headers(&[("x-tenant-id", "acme")]);
        let suv = vec![Some(headers)];
        assert!(!validate_transport_header_require_keys(
            &suv,
            &context_names(&["x-tenant-id", "x-missing"]),
        ));
    }

    #[test]
    fn require_keys_fails_when_no_messages_have_headers() {
        let suv: Vec<Option<TransportHeaders>> = vec![None, None];
        assert!(!validate_transport_header_require_keys(
            &suv,
            &context_names(&["x-tenant-id"]),
        ));
    }

    #[test]
    fn require_key_values_passes_on_match() {
        let headers = make_headers(&[("x-tenant-id", "acme")]);
        let suv = vec![Some(headers)];
        assert!(validate_transport_header_require_key_values(
            &suv,
            &[key_value("x-tenant-id", "acme")],
        ));
    }

    #[test]
    fn require_key_values_fails_on_value_mismatch() {
        let headers = make_headers(&[("x-tenant-id", "acme")]);
        let suv = vec![Some(headers)];
        assert!(!validate_transport_header_require_key_values(
            &suv,
            &[key_value("x-tenant-id", "other")],
        ));
    }

    #[test]
    fn require_key_values_fails_on_missing_key() {
        let headers = make_headers(&[("x-tenant-id", "acme")]);
        let suv = vec![Some(headers)];
        assert!(!validate_transport_header_require_key_values(
            &suv,
            &[key_value("x-missing", "value")],
        ));
    }

    #[test]
    fn deny_keys_passes_when_key_absent() {
        let headers = make_headers(&[("x-tenant-id", "acme")]);
        let suv = vec![Some(headers)];
        assert!(validate_transport_header_deny_keys(
            &suv,
            &context_names(&["x-secret"]),
        ));
    }

    #[test]
    fn deny_keys_fails_when_key_present() {
        let headers = make_headers(&[("x-tenant-id", "acme")]);
        let suv = vec![Some(headers)];
        assert!(!validate_transport_header_deny_keys(
            &suv,
            &context_names(&["x-tenant-id"]),
        ));
    }

    #[test]
    fn deny_keys_passes_on_no_headers() {
        let suv: Vec<Option<TransportHeaders>> = vec![None];
        assert!(validate_transport_header_deny_keys(
            &suv,
            &context_names(&["x-tenant-id"]),
        ));
    }

    #[test]
    fn empty_keys_always_passes() {
        let suv: Vec<Option<TransportHeaders>> = vec![None];
        assert!(validate_transport_header_require_keys(&suv, &[]));
        assert!(validate_transport_header_require_key_values(&suv, &[]));
        assert!(validate_transport_header_deny_keys(&suv, &[]));
    }

    #[test]
    fn multiple_messages_all_must_pass() {
        let h1 = make_headers(&[("x-tenant-id", "acme"), ("x-request-id", "1")]);
        let h2 = make_headers(&[("x-tenant-id", "acme")]); // missing x-request-id
        let suv = vec![Some(h1), Some(h2)];
        assert!(!validate_transport_header_require_keys(
            &suv,
            &context_names(&["x-tenant-id", "x-request-id"]),
        ));
    }

    #[test]
    fn require_keys_fails_when_one_message_has_no_headers() {
        let headers = make_headers(&[("x-tenant-id", "acme")]);
        let suv = vec![Some(headers), None];
        assert!(!validate_transport_header_require_keys(
            &suv,
            &context_names(&["x-tenant-id"]),
        ));
    }

    #[test]
    fn require_key_values_fails_when_one_message_has_no_headers() {
        let headers = make_headers(&[("x-tenant-id", "acme")]);
        let suv = vec![Some(headers), None];
        assert!(!validate_transport_header_require_key_values(
            &suv,
            &[key_value("x-tenant-id", "acme")],
        ));
    }

    #[test]
    fn require_keys_fails_on_empty_suv() {
        let suv: Vec<Option<TransportHeaders>> = vec![];
        assert!(!validate_transport_header_require_keys(
            &suv,
            &context_names(&["x-tenant-id"]),
        ));
    }

    #[test]
    fn require_key_values_fails_on_empty_suv() {
        let suv: Vec<Option<TransportHeaders>> = vec![];
        assert!(!validate_transport_header_require_key_values(
            &suv,
            &[key_value("x-tenant-id", "acme")],
        ));
    }

    #[test]
    fn deny_keys_passes_when_mixed_some_and_none() {
        let headers = make_headers(&[("x-tenant-id", "acme")]);
        let suv = vec![Some(headers), None];
        assert!(validate_transport_header_deny_keys(
            &suv,
            &context_names(&["x-secret"]),
        ));
    }

    #[test]
    fn require_key_values_matches_duplicate_key_second_value() {
        // Two headers with the same key but different values.
        // The required value matches the second entry.
        let headers = make_headers(&[("x-env", "staging"), ("x-env", "production")]);
        let suv = vec![Some(headers)];
        assert!(validate_transport_header_require_key_values(
            &suv,
            &[key_value("x-env", "production")],
        ));
    }

    #[test]
    fn require_key_values_matches_duplicate_key_first_value() {
        let headers = make_headers(&[("x-env", "staging"), ("x-env", "production")]);
        let suv = vec![Some(headers)];
        assert!(validate_transport_header_require_key_values(
            &suv,
            &[key_value("x-env", "staging")],
        ));
    }

    #[test]
    fn require_key_values_fails_when_no_duplicate_matches() {
        let headers = make_headers(&[("x-env", "staging"), ("x-env", "production")]);
        let suv = vec![Some(headers)];
        assert!(!validate_transport_header_require_key_values(
            &suv,
            &[key_value("x-env", "development")],
        ));
    }

    /// Scenario: a header/value assertion uses an unrestricted stored name.
    /// Guarantees: construction preserves the existing accepted input domain.
    #[test]
    fn key_value_construction_preserves_name() {
        let pair =
            TransportHeaderKeyValue::try_new("not normalized", "value").expect("name is accepted");
        assert_eq!(pair.key.as_str(), "not normalized");
    }
}
