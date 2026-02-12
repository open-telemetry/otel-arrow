// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Attribute validation helpers.
//!
//! Given `&[OtlpProtoMessage]`, verify that certain attribute keys (or key/value
//! pairs) appear for each attribute list or do **not** appear within configured domains (resource,
//! scope, or the signal itself).

use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::proto::opentelemetry::common::v1 as proto_common;
use otap_df_pdata::proto::opentelemetry::common::v1::{
    AnyValue as ProtoValue, KeyValue as ProtoKeyValue, any_value::Value as ProtoAnyValue,
};
use otap_df_pdata::proto::opentelemetry::{
    logs::v1 as proto_logs, metrics::v1 as proto_metrics, trace::v1 as proto_trace,
};
use serde::{Deserialize, Serialize};

use otap_df_pdata::otap::filter::{AnyValue, KeyValue};

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
}

impl AttributeCheck {
    /// Run the configured checks against a list of OTLP proto messages.
    pub fn check(&self, messages: &[OtlpProtoMessage]) -> bool {
        if messages.is_empty() {
            return false;
        }

        messages.iter().all(|msg| self.check_message(msg))
    }

    fn check_message(&self, message: &OtlpProtoMessage) -> bool {
        let attr_lists = self.extract_attributes(message);

        if attr_lists.is_empty() {
            return self.require_keys.is_empty()
                && self.require.is_empty()
                && self.forbid_keys.is_empty();
        }

        attr_lists
            .iter()
            .all(|attrs| self.validate_attr_list(attrs))
    }

    fn validate_attr_list(&self, attrs: &[ProtoKeyValue]) -> bool {
        // Pre-convert required key/value pairs to Proto form once per list to reduce repeat work.
        let required_proto: Vec<ProtoKeyValue> = self
            .require
            .iter()
            .filter_map(|kv| keyvalue_to_proto(kv))
            .collect();

        // Fail fast on forbidden keys.
        if attrs
            .iter()
            .any(|kv| self.forbid_keys.iter().any(|forbid| forbid == &kv.key))
        {
            return false;
        }

        // Ensure all required keys exist.
        for required in &self.require_keys {
            if !attrs.iter().any(|kv| kv.key == *required) {
                return false;
            }
        }

        // Ensure all required key/value pairs exist.
        for req in &required_proto {
            if !attrs
                .iter()
                .any(|kv| kv.key == req.key && kv.value == req.value)
            {
                return false;
            }
        }

        true
    }

    /// Get all attribute lists from a message for the selected domains.
    fn extract_attributes(&self, message: &OtlpProtoMessage) -> Vec<Vec<ProtoKeyValue>> {
        match message {
            OtlpProtoMessage::Logs(logs) => self.extract_log_attributes(logs),
            OtlpProtoMessage::Traces(traces) => self.extract_span_attributes(traces),
            OtlpProtoMessage::Metrics(metrics) => self.extract_metrics_attributes(metrics),
        }
    }

    fn extract_log_attributes(&self, logs: &proto_logs::LogsData) -> Vec<Vec<ProtoKeyValue>> {
        let mut out = Vec::new();

        let include_resource = self.domains.contains(&AttributeDomain::Resource);
        let include_scope = self.domains.contains(&AttributeDomain::Scope);
        let include_signal = self.domains.contains(&AttributeDomain::Signal);

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

    fn extract_span_attributes(&self, traces: &proto_trace::TracesData) -> Vec<Vec<ProtoKeyValue>> {
        let mut out = Vec::new();

        let include_resource = self.domains.contains(&AttributeDomain::Resource);
        let include_scope = self.domains.contains(&AttributeDomain::Scope);
        let include_signal = self.domains.contains(&AttributeDomain::Signal);

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
        &self,
        metrics: &proto_metrics::MetricsData,
    ) -> Vec<Vec<ProtoKeyValue>> {
        use proto_metrics::metric::Data as MetricData;

        let mut out = Vec::new();
        let include_resource = self.domains.contains(&AttributeDomain::Resource);
        let include_scope = self.domains.contains(&AttributeDomain::Scope);
        let include_signal = self.domains.contains(&AttributeDomain::Signal);

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
                                    }
                                }
                                MetricData::Sum(s) => {
                                    for dp in &s.data_points {
                                        out.push(dp.attributes.clone());
                                    }
                                }
                                MetricData::Histogram(h) => {
                                    for dp in &h.data_points {
                                        out.push(dp.attributes.clone());
                                    }
                                }
                                MetricData::ExponentialHistogram(eh) => {
                                    for dp in &eh.data_points {
                                        out.push(dp.attributes.clone());
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
}

fn keyvalue_to_proto(kv: &KeyValue) -> Option<ProtoKeyValue> {
    anyvalue_to_proto(&kv.value).map(|val| ProtoKeyValue {
        key: kv.key.clone(),
        value: Some(val),
    })
}

fn anyvalue_to_proto(val: &AnyValue) -> Option<ProtoValue> {
    let proto = match val {
        AnyValue::String(s) => ProtoAnyValue::StringValue(s.clone()),
        AnyValue::Int(i) => ProtoAnyValue::IntValue(*i),
        AnyValue::Double(d) => ProtoAnyValue::DoubleValue(*d),
        AnyValue::Boolean(b) => ProtoAnyValue::BoolValue(*b),
        AnyValue::Array(values) => ProtoAnyValue::ArrayValue(proto_common::ArrayValue {
            values: values
                .iter()
                .filter_map(anyvalue_to_proto)
                .collect::<Vec<_>>(),
        }),
        AnyValue::KeyValue(kvs) => ProtoAnyValue::KvlistValue(proto_common::KeyValueList {
            values: kvs.iter().filter_map(keyvalue_to_proto).collect::<Vec<_>>(),
        }),
    };

    Some(ProtoValue { value: Some(proto) })
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
