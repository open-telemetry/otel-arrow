// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Transport header validation helpers.
//!
//! Each header instruction compiles to a [`TransportHeaderCheck`] that reads
//! the SUV message context through the exporter's declared context reads and
//! keeps a running verdict, so no headers are copied per message.
//!
//! For **require** checks (`require_keys`, `require_key_values`), at least one
//! SUV message must be observed and every message must carry each key (and,
//! for value checks, one matching value).
//!
//! For **deny** checks, a message without the key -- including one without
//! any transport headers -- is acceptable.
//!
//! Every check passes when its key list is empty.

use super::ValidationInstructions;
use otel_arrow_dfe_config::ContextEntryName;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_engine::pdata_context::{
    ContextDomain, ContextEntrySelector, ContextEntrySelectorForm, ContextEntryTake,
    ContextEntryTarget, ContextError, ContextNameMatching, Metadata, NodeBindings, ValueRead,
};
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

/// Exact-name transport header read declared by the validation exporter.
#[must_use]
pub(crate) fn header_selector(
    name: ContextEntryName,
    take: ContextEntryTake,
) -> ContextEntrySelector {
    ContextEntrySelector {
        target: ContextEntryTarget::Primitive {
            domain: ContextDomain::TransportHeader,
            name,
            matching: ContextNameMatching::Exact,
        },
        form: ContextEntrySelectorForm::Value,
        take,
    }
}

#[derive(Debug)]
enum Rule {
    RequireKeys(Box<[ValueRead]>),
    RequireKeyValues(Box<[(ValueRead, String)]>),
    Deny(Box<[ValueRead]>),
}

/// One transport header instruction compiled against the exporter's context reads.
#[derive(Debug)]
pub(crate) struct TransportHeaderCheck {
    rule: Rule,
    observed: usize,
    passed: bool,
}

impl TransportHeaderCheck {
    /// Compiles a header instruction; returns `None` for other instructions.
    pub(crate) fn compile(
        instruction: &ValidationInstructions,
        bindings: Option<&NodeBindings>,
    ) -> Result<Option<Self>, ConfigError> {
        let read = |name: &ContextEntryName, take| -> Result<ValueRead, ConfigError> {
            bindings
                .ok_or_else(|| ConfigError::InvalidUserConfig {
                    error: "validation exporter has no context bindings".into(),
                })?
                .value(&header_selector(name.clone(), take))
                .cloned()
        };
        let reads = |keys: &[ContextEntryName], take| {
            keys.iter()
                .map(|key| read(key, take))
                .collect::<Result<Box<[_]>, _>>()
        };
        let rule = match instruction {
            ValidationInstructions::TransportHeaderRequireKey { keys } => {
                Rule::RequireKeys(reads(keys, ContextEntryTake::Exists)?)
            }
            ValidationInstructions::TransportHeaderRequireKeyValue { pairs } => {
                Rule::RequireKeyValues(
                    pairs
                        .iter()
                        .map(|pair| {
                            Ok((read(&pair.key, ContextEntryTake::All)?, pair.value.clone()))
                        })
                        .collect::<Result<_, ConfigError>>()?,
                )
            }
            ValidationInstructions::TransportHeaderDeny { keys } => {
                Rule::Deny(reads(keys, ContextEntryTake::Exists)?)
            }
            _ => return Ok(None),
        };
        Ok(Some(Self {
            rule,
            observed: 0,
            passed: true,
        }))
    }

    /// Checks one SUV message's context; a read error fails the check.
    pub(crate) fn observe(&mut self, metadata: &Metadata) -> Result<(), ContextError> {
        self.observed += 1;
        if !self.passed {
            return Ok(());
        }
        match self.matches(metadata) {
            Ok(matched) => {
                self.passed = matched;
                Ok(())
            }
            Err(err) => {
                self.passed = false;
                Err(err)
            }
        }
    }

    fn matches(&self, metadata: &Metadata) -> Result<bool, ContextError> {
        match &self.rule {
            Rule::RequireKeys(reads) => {
                for read in reads {
                    if read.read(metadata)?.is_none() {
                        return Ok(false);
                    }
                }
            }
            Rule::RequireKeyValues(pairs) => {
                for (read, expected) in pairs {
                    let mut found = false;
                    if let Some(selected) = read.read(metadata)? {
                        selected.visit_members(|member| {
                            found |= member
                                .values()
                                .any(|value| value.bytes == expected.as_bytes());
                        });
                    }
                    if !found {
                        return Ok(false);
                    }
                }
            }
            Rule::Deny(reads) => {
                for read in reads {
                    if read.read(metadata)?.is_some() {
                        return Ok(false);
                    }
                }
            }
        }
        Ok(true)
    }

    /// Returns the verdict over every observed SUV message.
    #[must_use]
    pub(crate) fn passed(&self) -> bool {
        let requires_messages = match &self.rule {
            Rule::RequireKeys(reads) => !reads.is_empty(),
            Rule::RequireKeyValues(pairs) => !pairs.is_empty(),
            Rule::Deny(_) => false,
        };
        self.passed && (!requires_messages || self.observed > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation_exporter::VALIDATION_EXPORTER_FACTORY;
    use otel_arrow_dfe_config::PipelineKey;
    use otel_arrow_dfe_config::context::transport_headers::{TransportHeader, TransportHeaders};
    use otel_arrow_dfe_config::engine::OtelDataflowSpec;
    use otel_arrow_dfe_engine::PipelineFactory;

    fn context_name(raw: &str) -> ContextEntryName {
        ContextEntryName::try_from(raw).expect("valid test context entry name")
    }

    fn context_names(raw: &[&str]) -> Vec<ContextEntryName> {
        raw.iter().map(|name| context_name(name)).collect()
    }

    fn key_value(key: &str, value: &str) -> TransportHeaderKeyValue {
        TransportHeaderKeyValue::try_new(key, value).expect("valid test context entry name")
    }

    /// Compiles `instruction` as the validation exporter does and observes
    /// one message per entry, `None` meaning a message without headers.
    fn run(instruction: ValidationInstructions, suv: &[Option<TransportHeaders>]) -> bool {
        let spec: OtelDataflowSpec = serde_json::from_value(serde_json::json!({
            "version": "otel_dataflow/v1",
            "engine": {},
            "groups": {"group": {"pipelines": {"pipeline": {
                "nodes": {"exporter": {
                    "type": "urn:otel:exporter:validation",
                    "config": {"suv_input": "suv", "validations": [instruction.clone()]}
                }},
                "connections": []
            }}}}
        }))
        .expect("pipeline config");
        let mut resolved = spec.resolve();
        // This fixture does not register the implicit telemetry pipeline's nodes.
        resolved
            .pipelines
            .retain(|pipeline| pipeline.pipeline_group_id.as_ref() == "group");
        let prepared = PipelineFactory::new(
            &[],
            &[],
            std::slice::from_ref(&VALIDATION_EXPORTER_FACTORY),
            &[],
        )
        .compile_initial_context(&resolved)
        .expect("compiled validation config");
        let bindings = prepared.bindings.node_bindings(
            &PipelineKey::new("group".into(), "pipeline".into()),
            &"exporter".into(),
        );
        let mut check = TransportHeaderCheck::compile(&instruction, bindings.map(|b| &**b))
            .expect("compiled check")
            .expect("header instruction");
        for headers in suv {
            let mut metadata = Metadata::default();
            if let Some(headers) = headers {
                metadata.set_transport_headers(headers.clone());
            }
            check.observe(&metadata).expect("read succeeds");
        }
        check.passed()
    }

    fn validate_transport_header_require_keys(
        suv: &[Option<TransportHeaders>],
        keys: &[ContextEntryName],
    ) -> bool {
        run(
            ValidationInstructions::TransportHeaderRequireKey {
                keys: keys.to_vec(),
            },
            suv,
        )
    }

    fn validate_transport_header_require_key_values(
        suv: &[Option<TransportHeaders>],
        pairs: &[TransportHeaderKeyValue],
    ) -> bool {
        run(
            ValidationInstructions::TransportHeaderRequireKeyValue {
                pairs: pairs.to_vec(),
            },
            suv,
        )
    }

    fn validate_transport_header_deny_keys(
        suv: &[Option<TransportHeaders>],
        keys: &[ContextEntryName],
    ) -> bool {
        run(
            ValidationInstructions::TransportHeaderDeny {
                keys: keys.to_vec(),
            },
            suv,
        )
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

    /// Scenario: headers differ from the configured names only by case.
    /// Guarantees: require and deny checks use exact-case lookup.
    #[test]
    fn checks_use_exact_case_names() {
        let suv = vec![Some(make_headers(&[("Tenant", "acme")]))];
        assert!(validate_transport_header_require_keys(
            &suv,
            &context_names(&["Tenant"]),
        ));
        assert!(!validate_transport_header_require_keys(
            &suv,
            &context_names(&["tenant"]),
        ));
        assert!(validate_transport_header_deny_keys(
            &suv,
            &context_names(&["tenant"]),
        ));
        assert!(!validate_transport_header_require_key_values(
            &suv,
            &[key_value("tenant", "acme")],
        ));
    }

    /// Scenario: a header/value assertion uses an invalid stored name.
    /// Guarantees: construction rejects whitespace in the context entry name.
    #[test]
    fn key_value_construction_rejects_invalid_name() {
        assert!(TransportHeaderKeyValue::try_new("not valid", "value").is_err());
    }
}
