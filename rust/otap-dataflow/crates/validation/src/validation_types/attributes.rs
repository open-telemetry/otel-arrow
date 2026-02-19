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

/// enum to describe attribute value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnyValue {
    /// string type
    String(String),
    /// int type
    Int(i64),
    /// double type
    Double(f64),
    /// boolean type
    Boolean(bool),
    /// array of any type
    Array(Vec<AnyValue>),
    /// keyvalue type
    KeyValue(Vec<KeyValue>),
}

/// struct to match key value pairs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyValue {
    /// Attribute key.
    pub key: String,
    /// Attribute value.
    pub value: AnyValue,
}

impl KeyValue {
    /// create a key value pair
    #[must_use]
    pub fn new(key: String, value: AnyValue) -> Self {
        Self { key, value }
    }
}

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

/// Collect all attribute lists (as slices) from a message for the selected domains.
pub(crate) fn collect_attributes<'a>(
    message: &'a OtlpProtoMessage,
    domains: &[AttributeDomain],
) -> Vec<&'a [ProtoKeyValue]> {
    match message {
        OtlpProtoMessage::Logs(logs) => collect_log_attrs(logs, domains),
        OtlpProtoMessage::Traces(traces) => collect_span_attrs(traces, domains),
        OtlpProtoMessage::Metrics(metrics) => collect_metric_attrs(metrics, domains),
    }
}

/// Validate that none of the provided keys appear in the selected domains.
pub(crate) fn validate_deny_keys(
    message: &OtlpProtoMessage,
    domains: &[AttributeDomain],
    keys: &[String],
) -> bool {
    collect_attributes(message, domains)
        .into_iter()
        .all(|attrs| {
            keys.iter()
                .all(|deny| attrs.iter().all(|kv| deny != &kv.key))
        })
}

/// Validate that all provided keys are present in each attribute list for the selected domains.
pub(crate) fn validate_require_keys(
    message: &OtlpProtoMessage,
    domains: &[AttributeDomain],
    keys: &[String],
) -> bool {
    collect_attributes(message, domains)
        .into_iter()
        .all(|attrs| keys.iter().all(|req| attrs.iter().any(|kv| req == &kv.key)))
}

/// Validate that all provided key/value pairs are present in each attribute list for the selected domains.
pub(crate) fn validate_require_key_values(
    message: &OtlpProtoMessage,
    domains: &[AttributeDomain],
    pairs: &[KeyValue],
) -> bool {
    let pairs_proto_kv: Vec<ProtoKeyValue> = pairs.iter().filter_map(keyvalue_to_proto).collect();

    collect_attributes(message, domains)
        .into_iter()
        .all(|attrs| {
            pairs_proto_kv.iter().all(|req| {
                attrs
                    .iter()
                    .any(|kv| kv.key == req.key && kv.value == req.value)
            })
        })
}

/// Validate that no attribute list contains duplicate keys.
pub(crate) fn validate_no_duplicate_keys(message: &OtlpProtoMessage) -> bool {
    let domains = vec![
        AttributeDomain::Resource,
        AttributeDomain::Scope,
        AttributeDomain::Signal,
    ];
    collect_attributes(message, &domains)
        .into_iter()
        .all(|attrs| {
            let mut seen = std::collections::HashSet::new();
            attrs.iter().all(|kv| seen.insert(&kv.key))
        })
}

/// helper to extract attributes from logs
fn collect_log_attrs<'a>(
    logs: &'a proto_logs::LogsData,
    domains: &[AttributeDomain],
) -> Vec<&'a [ProtoKeyValue]> {
    let mut out = Vec::new();
    let include_resource = domains.contains(&AttributeDomain::Resource);
    let include_scope = domains.contains(&AttributeDomain::Scope);
    let include_signal = domains.contains(&AttributeDomain::Signal);

    for resource_logs in &logs.resource_logs {
        if include_resource {
            if let Some(resource) = resource_logs.resource.as_ref() {
                out.push(resource.attributes.as_slice());
            }
        }
        for scope_logs in &resource_logs.scope_logs {
            if include_scope {
                if let Some(scope) = scope_logs.scope.as_ref() {
                    out.push(scope.attributes.as_slice());
                }
            }
            if include_signal {
                for record in &scope_logs.log_records {
                    out.push(record.attributes.as_slice());
                }
            }
        }
    }
    out
}

/// helper to extract attributes from spans
fn collect_span_attrs<'a>(
    traces: &'a proto_trace::TracesData,
    domains: &[AttributeDomain],
) -> Vec<&'a [ProtoKeyValue]> {
    let mut out = Vec::new();
    let include_resource = domains.contains(&AttributeDomain::Resource);
    let include_scope = domains.contains(&AttributeDomain::Scope);
    let include_signal = domains.contains(&AttributeDomain::Signal);

    for resource_spans in &traces.resource_spans {
        if include_resource {
            if let Some(resource) = resource_spans.resource.as_ref() {
                out.push(resource.attributes.as_slice());
            }
        }
        for scope_spans in &resource_spans.scope_spans {
            if include_scope {
                if let Some(scope) = scope_spans.scope.as_ref() {
                    out.push(scope.attributes.as_slice());
                }
            }
            if include_signal {
                for span in &scope_spans.spans {
                    out.push(span.attributes.as_slice());
                }
            }
        }
    }
    out
}

/// helper to extract attributes from metrics
fn collect_metric_attrs<'a>(
    metrics: &'a proto_metrics::MetricsData,
    domains: &[AttributeDomain],
) -> Vec<&'a [ProtoKeyValue]> {
    use proto_metrics::metric::Data as MetricData;

    let mut out = Vec::new();
    let include_resource = domains.contains(&AttributeDomain::Resource);
    let include_scope = domains.contains(&AttributeDomain::Scope);
    let include_signal = domains.contains(&AttributeDomain::Signal);

    for resource_metrics in &metrics.resource_metrics {
        if include_resource {
            if let Some(resource) = resource_metrics.resource.as_ref() {
                out.push(resource.attributes.as_slice());
            }
        }
        for scope_metrics in &resource_metrics.scope_metrics {
            if include_scope {
                if let Some(scope) = scope_metrics.scope.as_ref() {
                    out.push(scope.attributes.as_slice());
                }
            }
            if include_signal {
                for metric in &scope_metrics.metrics {
                    if let Some(data) = &metric.data {
                        match data {
                            MetricData::Gauge(g) => {
                                for dp in &g.data_points {
                                    out.push(dp.attributes.as_slice());
                                }
                            }
                            MetricData::Sum(s) => {
                                for dp in &s.data_points {
                                    out.push(dp.attributes.as_slice());
                                }
                            }
                            MetricData::Histogram(h) => {
                                for dp in &h.data_points {
                                    out.push(dp.attributes.as_slice());
                                }
                            }
                            MetricData::ExponentialHistogram(eh) => {
                                for dp in &eh.data_points {
                                    out.push(dp.attributes.as_slice());
                                }
                            }
                            MetricData::Summary(s) => {
                                for dp in &s.data_points {
                                    out.push(dp.attributes.as_slice());
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

/// helper to convert KeyValue type to ProtoKeyValue
#[allow(dead_code)]
fn keyvalue_to_proto(kv: &KeyValue) -> Option<ProtoKeyValue> {
    anyvalue_to_proto(&kv.value).map(|val| ProtoKeyValue {
        key: kv.key.clone(),
        value: Some(val),
    })
}

/// helper to convert AnyValue type to ProtoValue
#[allow(dead_code)]
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
    use otap_df_pdata::proto::opentelemetry::logs::v1::{LogsData, ResourceLogs, ScopeLogs};
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;

    #[test]
    fn collect_log_attributes_includes_all_domains() {
        use otap_df_pdata::proto::opentelemetry::common::v1::InstrumentationScope;
        use otap_df_pdata::proto::opentelemetry::logs::v1::LogRecord;

        fn proto_kv(key: &str, val: &str) -> ProtoKeyValue {
            ProtoKeyValue {
                key: key.into(),
                value: Some(ProtoValue {
                    value: Some(ProtoAnyValue::StringValue(val.into())),
                }),
            }
        }

        let logs = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![proto_kv("res", "r")],
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        attributes: vec![proto_kv("scope", "s")],
                        ..Default::default()
                    }),
                    log_records: vec![LogRecord {
                        attributes: vec![proto_kv("sig", "l")],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let msg = OtlpProtoMessage::Logs(logs);

        let attrs = collect_attributes(&msg, &[AttributeDomain::Resource]);
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0][0].key, "res");

        let attrs = collect_attributes(&msg, &[AttributeDomain::Scope]);
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0][0].key, "scope");

        let attrs = collect_attributes(&msg, &[AttributeDomain::Signal]);
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0][0].key, "sig");

        let attrs = collect_attributes(
            &msg,
            &[
                AttributeDomain::Resource,
                AttributeDomain::Scope,
                AttributeDomain::Signal,
            ],
        );
        let keys: Vec<_> = attrs
            .iter()
            .flat_map(|slice| slice.iter().map(|kv| kv.key.as_str()))
            .collect();
        assert!(keys.contains(&"res"));
        assert!(keys.contains(&"scope"));
        assert!(keys.contains(&"sig"));
    }

    #[test]
    fn require_keys_must_be_present_in_each_domain() {
        use otap_df_pdata::proto::opentelemetry::common::v1::InstrumentationScope;
        use otap_df_pdata::proto::opentelemetry::logs::v1::LogRecord;
        fn proto_kv(key: &str, val: &str) -> ProtoKeyValue {
            ProtoKeyValue {
                key: key.into(),
                value: Some(ProtoValue {
                    value: Some(ProtoAnyValue::StringValue(val.into())),
                }),
            }
        }

        let mut logs = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![proto_kv("shared", "r")],
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        attributes: vec![proto_kv("shared", "s")],
                        ..Default::default()
                    }),
                    log_records: vec![LogRecord {
                        attributes: vec![proto_kv("shared", "l")],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let msg = OtlpProtoMessage::Logs(logs.clone());
        assert!(validate_require_keys(
            &msg,
            &[
                AttributeDomain::Resource,
                AttributeDomain::Scope,
                AttributeDomain::Signal
            ],
            &[String::from("shared")]
        ));

        // Remove the attribute from the scope to force a failure.
        logs.resource_logs[0].scope_logs[0]
            .scope
            .as_mut()
            .unwrap()
            .attributes
            .clear();
        let missing_scope = OtlpProtoMessage::Logs(logs);
        assert!(!validate_require_keys(
            &missing_scope,
            &[
                AttributeDomain::Resource,
                AttributeDomain::Scope,
                AttributeDomain::Signal
            ],
            &[String::from("shared")]
        ));
    }

    #[test]
    fn deny_keys_blocks_presence() {
        use otap_df_pdata::proto::opentelemetry::logs::v1::LogRecord;
        fn proto_kv(key: &str, val: &str) -> ProtoKeyValue {
            ProtoKeyValue {
                key: key.into(),
                value: Some(ProtoValue {
                    value: Some(ProtoAnyValue::StringValue(val.into())),
                }),
            }
        }
        let logs = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        attributes: vec![proto_kv("forbidden", "v")],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let msg = OtlpProtoMessage::Logs(logs);
        assert!(!validate_deny_keys(
            &msg,
            &[AttributeDomain::Signal],
            &[String::from("forbidden")]
        ));
    }
}
