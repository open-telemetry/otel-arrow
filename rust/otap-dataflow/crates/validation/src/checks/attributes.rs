// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Attribute validation helpers.
//!
//! Given `&[OtlpProtoMessage]`, verify that certain attribute keys (or key/value
//! pairs) appear for each attribute list or do **not** appear within configured domains (resource,
//! scope, or the signal itself).

use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::proto::opentelemetry::common::v1::{
    AnyValue as ProtoValue, KeyValue as ProtoKeyValue, any_value::Value as ProtoAnyValue,
};
use otap_df_pdata::proto::opentelemetry::{
    logs::v1 as proto_logs, metrics::v1 as proto_metrics, trace::v1 as proto_trace,
};
use serde::{Deserialize, Serialize};

use super::{AnyValue, KeyValue};

/// Domains where attribute assertions can be applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttributeDomain {
    /// Resource attributes.
    Resource,
    /// Scope / instrumentation scope attributes.
    Scope,
    /// Signal-specific attributes (logs, spans, data points, etc.).
    Signal,
}

fn default_domains() -> Vec<AttributeDomain> {
    vec![AttributeDomain::Signal]
}

/// Declarative attribute validation.
///
/// - `require_keys`: every attribute list in the selected domains must contain
///   these keys (value is ignored).
/// - `require`: required key/value pairs that must appear in each attribute list.
/// - `forbid_keys`: keys that must **not** appear in each attribute list.
/// - `forbid`: key/value pairs that must **not** appear in each attribute list.
/// - `domains`: which domains to inspect; defaults to [`AttributeDomain::Signal`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AttributeCheck {
    /// Domains to inspect; defaults to only the signal domain.
    #[serde(default = "default_domains")]
    pub domains: Vec<AttributeDomain>,
    /// Keys that must exist in every inspected attribute list (value ignored).
    #[serde(default)]
    pub require_keys: Vec<String>,
    /// Key/value pairs that must exist in every inspected attribute list.
    #[serde(default)]
    pub require: Vec<KeyValue>,
    /// Keys that must not appear in any inspected attribute list.
    #[serde(default)]
    pub forbid_keys: Vec<String>,
    /// Key/value pairs that must not appear in any inspected attribute list.
    #[serde(default)]
    pub forbid: Vec<KeyValue>,
}

impl AttributeCheck {
    /// Run the configured checks against a list of OTLP proto messages.
    pub fn check(&self, messages: &[OtlpProtoMessage]) -> bool {
        if messages.is_empty() {
            return false;
        }

        // Iterate through messages and validate each one; short-circuit on first failure.
        messages.iter().all(|msg| self.check_message(msg))
    }

    fn check_message(&self, message: &OtlpProtoMessage) -> bool {
        let attr_lists = extract_attributes(message, &self.domains);

        // If no attributes were found for the requested domains, only succeed when
        // there are no expectations to verify.
        if attr_lists.is_empty() {
            return self.require_keys.is_empty()
                && self.require.is_empty()
                && self.forbid_keys.is_empty()
                && self.forbid.is_empty();
        }

        attr_lists
            .iter()
            .all(|attrs| validate_attr_list(attrs, self))
    }
}

/// Convenience wrapper to run an [`AttributeCheck`] against a batch of messages.
pub fn check_attributes(messages: &[OtlpProtoMessage], config: &AttributeCheck) -> bool {
    config.check(messages)
}

fn validate_attr_list(attrs: &[ProtoKeyValue], check: &AttributeCheck) -> bool {
    // Fail fast on forbidden keys.
    if attrs
        .iter()
        .any(|kv| check.forbid_keys.iter().any(|forbid| forbid == &kv.key))
    {
        return false;
    }

    // Fail fast on forbidden key/value pairs.
    if !check.forbid.is_empty()
        && attrs
            .iter()
            .any(|kv| check.forbid.iter().any(|f| proto_kv_matches(kv, f)))
    {
        return false;
    }

    // Ensure all required keys exist.
    for required in &check.require_keys {
        if !attrs.iter().any(|kv| kv.key == *required) {
            return false;
        }
    }

    // Ensure all required key/value pairs exist.
    for required in &check.require {
        if !attrs.iter().any(|kv| proto_kv_matches(kv, required)) {
            return false;
        }
    }

    true
}

/// Get all attribute lists from a message for the selected domains.
fn extract_attributes(
    message: &OtlpProtoMessage,
    domains: &[AttributeDomain],
) -> Vec<Vec<ProtoKeyValue>> {
    match message {
        OtlpProtoMessage::Logs(logs) => extract_log_attributes(logs, domains),
        OtlpProtoMessage::Traces(traces) => extract_span_attributes(traces, domains),
        OtlpProtoMessage::Metrics(metrics) => extract_metrics_attributes(metrics, domains),
    }
}

fn extract_log_attributes(
    logs: &proto_logs::LogsData,
    domains: &[AttributeDomain],
) -> Vec<Vec<ProtoKeyValue>> {
    let mut out = Vec::new();

    let include_resource = domains.contains(&AttributeDomain::Resource);
    let include_scope = domains.contains(&AttributeDomain::Scope);
    let include_signal = domains.contains(&AttributeDomain::Signal);

    for resource_logs in &logs.resource_logs {
        if include_resource {
            if let Some(resource) = resource_logs.resource.as_ref() {
                out.push(resource.attributes.clone());
            }
        }

        for scope_logs in &resource_logs.scope_logs {
            if include_scope {
                if let Some(scope) = scope_logs.scope.as_ref() {
                    out.push(scope.attributes.clone());
                }
            }

            if include_signal {
                for record in &scope_logs.log_records {
                    out.push(record.attributes.clone());
                }
            }
        }
    }

    out
}

fn extract_span_attributes(
    traces: &proto_trace::TracesData,
    domains: &[AttributeDomain],
) -> Vec<Vec<ProtoKeyValue>> {
    let mut out = Vec::new();

    let include_resource = domains.contains(&AttributeDomain::Resource);
    let include_scope = domains.contains(&AttributeDomain::Scope);
    let include_signal = domains.contains(&AttributeDomain::Signal);

    for resource_spans in &traces.resource_spans {
        if include_resource {
            if let Some(resource) = resource_spans.resource.as_ref() {
                out.push(resource.attributes.clone());
            }
        }

        for scope_spans in &resource_spans.scope_spans {
            if include_scope {
                if let Some(scope) = scope_spans.scope.as_ref() {
                    out.push(scope.attributes.clone());
                }
            }

            if include_signal {
                for span in &scope_spans.spans {
                    out.push(span.attributes.clone());
                    // Span events and links carry their own attributes.
                    for event in &span.events {
                        out.push(event.attributes.clone());
                    }
                    for link in &span.links {
                        out.push(link.attributes.clone());
                    }
                }
            }
        }
    }

    out
}

fn extract_metrics_attributes(
    metrics: &proto_metrics::MetricsData,
    domains: &[AttributeDomain],
) -> Vec<Vec<ProtoKeyValue>> {
    use proto_metrics::metric::Data as MetricData;

    let mut out = Vec::new();
    let include_resource = domains.contains(&AttributeDomain::Resource);
    let include_scope = domains.contains(&AttributeDomain::Scope);
    let include_signal = domains.contains(&AttributeDomain::Signal);

    for resource_metrics in &metrics.resource_metrics {
        if include_resource {
            if let Some(resource) = resource_metrics.resource.as_ref() {
                out.push(resource.attributes.clone());
            }
        }

        for scope_metrics in &resource_metrics.scope_metrics {
            if include_scope {
                if let Some(scope) = scope_metrics.scope.as_ref() {
                    out.push(scope.attributes.clone());
                }
            }

            if include_signal {
                for metric in &scope_metrics.metrics {
                    if let Some(data) = &metric.data {
                        match data {
                            MetricData::Gauge(g) => {
                                for dp in &g.data_points {
                                    out.push(dp.attributes.clone());
                                    push_exemplar_attrs(&mut out, &dp.exemplars);
                                }
                            }
                            MetricData::Sum(s) => {
                                for dp in &s.data_points {
                                    out.push(dp.attributes.clone());
                                    push_exemplar_attrs(&mut out, &dp.exemplars);
                                }
                            }
                            MetricData::Histogram(h) => {
                                for dp in &h.data_points {
                                    out.push(dp.attributes.clone());
                                    push_exemplar_attrs(&mut out, &dp.exemplars);
                                }
                            }
                            MetricData::ExponentialHistogram(eh) => {
                                for dp in &eh.data_points {
                                    out.push(dp.attributes.clone());
                                    push_exemplar_attrs(&mut out, &dp.exemplars);
                                }
                            }
                            MetricData::Summary(s) => {
                                for dp in &s.data_points {
                                    out.push(dp.attributes.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    out
}

fn push_exemplar_attrs(out: &mut Vec<Vec<ProtoKeyValue>>, exemplars: &[proto_metrics::Exemplar]) {
    for exemplar in exemplars {
        if !exemplar.filtered_attributes.is_empty() {
            out.push(exemplar.filtered_attributes.clone());
        }
    }
}

fn proto_kv_matches(proto: &ProtoKeyValue, expected: &KeyValue) -> bool {
    if proto.key != expected.key {
        return false;
    }

    match proto.value.as_ref() {
        None => false,
        Some(value) => proto_any_matches(value, &expected.value),
    }
}

fn proto_any_matches(value: &ProtoValue, expected: &AnyValue) -> bool {
    match (value.value.as_ref(), expected) {
        (Some(ProtoAnyValue::StringValue(v)), AnyValue::String(e)) => v == e,
        (Some(ProtoAnyValue::IntValue(v)), AnyValue::Int(e)) => v == e,
        (Some(ProtoAnyValue::DoubleValue(v)), AnyValue::Double(e)) => (v - e).abs() < f64::EPSILON,
        (Some(ProtoAnyValue::BoolValue(v)), AnyValue::Boolean(e)) => v == e,
        (Some(ProtoAnyValue::ArrayValue(arr)), AnyValue::Array(expected_vals)) => {
            if arr.values.len() != expected_vals.len() {
                return false;
            }
            arr.values
                .iter()
                .zip(expected_vals.iter())
                .all(|(pv, ev)| proto_any_matches(pv, ev))
        }
        (Some(ProtoAnyValue::KvlistValue(list)), AnyValue::KeyValue(expected_kvs)) => {
            expected_kvs.iter().all(|expected_kv| {
                list.values
                    .iter()
                    .any(|kv| proto_kv_matches(kv, expected_kv))
            })
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_pdata::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;

    fn kv(key: &str, val: &str) -> ProtoKeyValue {
        ProtoKeyValue {
            key: key.to_string(),
            value: Some(ProtoValue {
                value: Some(ProtoAnyValue::StringValue(val.to_string())),
            }),
        }
    }

    fn sample_log_message() -> OtlpProtoMessage {
        OtlpProtoMessage::Logs(proto_logs::LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![kv("res", "r1")],
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        attributes: vec![kv("foo", "bar")],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        })
    }

    #[test]
    fn passes_when_requirements_met() {
        let msg = sample_log_message();
        let check = AttributeCheck {
            domains: vec![AttributeDomain::Signal],
            require_keys: vec!["foo".into()],
            require: vec![KeyValue::new("foo".into(), AnyValue::String("bar".into()))],
            ..Default::default()
        };

        assert!(check.check(&[msg]));
    }

    #[test]
    fn fails_on_forbidden_key() {
        let msg = sample_log_message();
        let check = AttributeCheck {
            domains: vec![AttributeDomain::Signal],
            forbid_keys: vec!["foo".into()],
            ..Default::default()
        };

        assert!(!check.check(&[msg]));
    }

    #[test]
    fn respects_domain_selection() {
        // Resource has key res=r1, log has foo=bar. Require only resource key.
        let msg = sample_log_message();
        let check = AttributeCheck {
            domains: vec![AttributeDomain::Resource],
            require: vec![KeyValue::new("res".into(), AnyValue::String("r1".into()))],
            ..Default::default()
        };

        assert!(check.check(&[msg]));
    }
}
