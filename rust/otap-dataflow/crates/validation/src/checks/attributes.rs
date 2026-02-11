// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Attribute validation helpers.
//!
//! Given `&[OtlpProtoMessage]`, verify that certain attribute keys (or key/value
//! pairs) appear or do **not** appear within configured domains (resource,
//! scope, or the signal itself).

use super::uniform_signal_type;
use otap_df_pdata::proto::opentelemetry::{
    common::v1::{AnyValue, KeyValue, any_value},
    logs::v1::{LogsData, ResourceLogs, ScopeLogs},
    metrics::v1::{MetricsData, metric::Data},
    trace::v1::TracesData,
};
use otap_df_pdata::proto::OtlpProtoMessage;
use serde::{Deserialize, Serialize};

/// Which part of the payload to inspect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeDomain {
    /// Attributes attached to the Resource portion of the signal.
    Resource,
    /// Attributes attached to the Instrumentation Scope.
    Scope,
    /// Attributes attached to the signal payload (log record, span, datapoint).
    Signal,
}

/// Expected value for a key/value check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    /// UTF-8 string value.
    Str(String),
    /// Boolean value.
    Bool(bool),
    /// 64-bit integer value.
    Int(i64),
    /// 64-bit floating point value.
    Double(f64),
}

/// Configuration for an attribute validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeCheckConfig {
    /// Domains to inspect. Defaults to all when empty.
    #[serde(default)]
    pub domains: Vec<AttributeDomain>,
    /// Keys that must exist in at least one inspected attribute list.
    #[serde(default)]
    pub require_keys: Vec<String>,
    /// Key/value pairs that must exist in at least one inspected attribute list.
    #[serde(default)]
    pub require_key_values: Vec<ExpectedKeyValue>,
    /// Keys that must **not** be present in any inspected attribute list.
    #[serde(default)]
    pub forbid_keys: Vec<String>,
}

/// Required key/value pair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpectedKeyValue {
    /// Attribute key to match.
    pub key: String,
    /// Expected value for the key.
    pub value: AttributeValue,
}

impl AttributeCheckConfig {
    fn domains_or_default(&self) -> Vec<AttributeDomain> {
        if self.domains.is_empty() {
            vec![
                AttributeDomain::Resource,
                AttributeDomain::Scope,
                AttributeDomain::Signal,
            ]
        } else {
            self.domains.clone()
        }
    }

    /// Run the attribute validation against the provided messages.
    pub fn check(&self, messages: &[OtlpProtoMessage]) -> bool {
        if messages.is_empty() {
            return false;
        }

        // Attribute validation only makes sense on a uniform signal type.
        if uniform_signal_type(messages).is_none() {
            return false;
        }

        let domains = self.domains_or_default();
        let mut failures = Vec::new();

        for msg in messages {
            match msg {
                OtlpProtoMessage::Logs(data) => {
                    visit_logs_strict(data, &domains, self, &mut failures)
                }
                OtlpProtoMessage::Metrics(data) => {
                    visit_metrics_strict(data, &domains, self, &mut failures)
                }
                OtlpProtoMessage::Traces(data) => {
                    visit_traces_strict(data, &domains, self, &mut failures)
                }
            }
        }

        failures.is_empty()
    }
}

/// Check that attributes in the provided messages satisfy the supplied rules.
pub fn check_attributes(messages: &[OtlpProtoMessage], cfg: &AttributeCheckConfig) -> bool {
    cfg.check(messages)
}

fn display_value(value: &AttributeValue) -> String {
    match value {
        AttributeValue::Str(s) => s.clone(),
        AttributeValue::Bool(b) => b.to_string(),
        AttributeValue::Int(i) => i.to_string(),
        AttributeValue::Double(f) => f.to_string(),
    }
}

fn value_matches(expected: &AttributeValue, actual: &AnyValue) -> bool {
    match (&actual.value, expected) {
        (Some(any_value::Value::StringValue(v)), AttributeValue::Str(exp)) => v == exp,
        (Some(any_value::Value::BoolValue(v)), AttributeValue::Bool(exp)) => v == exp,
        (Some(any_value::Value::IntValue(v)), AttributeValue::Int(exp)) => v == exp,
        (Some(any_value::Value::DoubleValue(v)), AttributeValue::Double(exp)) => (*v - *exp).abs() < f64::EPSILON,
        _ => false,
    }
}

fn check_attr_set(
    attrs: &[KeyValue],
    cfg: &AttributeCheckConfig,
    label: &str,
    failures: &mut Vec<String>,
) {
    for key in &cfg.require_keys {
        if !attrs.iter().any(|kv| kv.key == *key) {
            failures.push(format!("{label} missing key {key}"));
        }
    }

    for kv_req in &cfg.require_key_values {
        let found = attrs.iter().any(|kv| {
            kv.key == kv_req.key
                && kv
                    .value
                    .as_ref()
                    .map(|v| value_matches(&kv_req.value, v))
                    .unwrap_or(false)
        });
        if !found {
            failures.push(format!(
                "{label} missing key/value {}={}",
                kv_req.key,
                display_value(&kv_req.value)
            ));
        }
    }

    for forbid in &cfg.forbid_keys {
        if attrs.iter().any(|kv| kv.key == *forbid) {
            failures.push(format!("{label} contains forbidden key {forbid}"));
        }
    }
}

fn visit_logs_strict(
    data: &LogsData,
    domains: &[AttributeDomain],
    cfg: &AttributeCheckConfig,
    failures: &mut Vec<String>,
) {
    let want_resource = domains.contains(&AttributeDomain::Resource);
    let want_scope = domains.contains(&AttributeDomain::Scope);
    let want_signal = domains.contains(&AttributeDomain::Signal);

    for (r_idx, ResourceLogs { resource, scope_logs, .. }) in data.resource_logs.iter().enumerate() {
        if want_resource {
            if let Some(res) = resource {
                check_attr_set(&res.attributes, cfg, &format!("resource[{r_idx}]"), failures);
            }
        }

        for (s_idx, ScopeLogs { scope, log_records, .. }) in scope_logs.iter().enumerate() {
            if want_scope {
                if let Some(scope) = scope {
                    check_attr_set(&scope.attributes, cfg, &format!("scope[{r_idx}:{s_idx}]"), failures);
                }
            }
            if want_signal {
                for (l_idx, log) in log_records.iter().enumerate() {
                    check_attr_set(&log.attributes, cfg, &format!("log[{r_idx}:{s_idx}:{l_idx}]"), failures);
                }
            }
        }
    }
}

fn visit_metrics_strict(
    data: &MetricsData,
    domains: &[AttributeDomain],
    cfg: &AttributeCheckConfig,
    failures: &mut Vec<String>,
) {
    let want_resource = domains.contains(&AttributeDomain::Resource);
    let want_scope = domains.contains(&AttributeDomain::Scope);
    let want_signal = domains.contains(&AttributeDomain::Signal);

    for (r_idx, rm) in data.resource_metrics.iter().enumerate() {
        if want_resource {
            if let Some(resource) = &rm.resource {
                check_attr_set(&resource.attributes, cfg, &format!("resource[{r_idx}]"), failures);
            }
        }
        for (s_idx, sm) in rm.scope_metrics.iter().enumerate() {
            if want_scope {
                if let Some(scope) = &sm.scope {
                    check_attr_set(&scope.attributes, cfg, &format!("scope[{r_idx}:{s_idx}]"), failures);
                }
            }
            if want_signal {
                for (m_idx, metric) in sm.metrics.iter().enumerate() {
                    match &metric.data {
                        Some(Data::Gauge(gauge)) => {
                            for (d_idx, dp) in gauge.data_points.iter().enumerate() {
                                check_attr_set(
                                    &dp.attributes,
                                    cfg,
                                    &format!("gauge_dp[{r_idx}:{s_idx}:{m_idx}:{d_idx}]"),
                                    failures,
                                );
                            }
                        }
                        Some(Data::Sum(sum)) => {
                            for (d_idx, dp) in sum.data_points.iter().enumerate() {
                                check_attr_set(
                                    &dp.attributes,
                                    cfg,
                                    &format!("sum_dp[{r_idx}:{s_idx}:{m_idx}:{d_idx}]"),
                                    failures,
                                );
                            }
                        }
                        Some(Data::Histogram(histogram)) => {
                            for (d_idx, dp) in histogram.data_points.iter().enumerate() {
                                check_attr_set(
                                    &dp.attributes,
                                    cfg,
                                    &format!("hist_dp[{r_idx}:{s_idx}:{m_idx}:{d_idx}]"),
                                    failures,
                                );
                            }
                        }
                        Some(Data::ExponentialHistogram(exp)) => {
                            for (d_idx, dp) in exp.data_points.iter().enumerate() {
                                check_attr_set(
                                    &dp.attributes,
                                    cfg,
                                    &format!("exp_hist_dp[{r_idx}:{s_idx}:{m_idx}:{d_idx}]"),
                                    failures,
                                );
                            }
                        }
                        Some(Data::Summary(summary)) => {
                            for (d_idx, dp) in summary.data_points.iter().enumerate() {
                                check_attr_set(
                                    &dp.attributes,
                                    cfg,
                                    &format!("summary_dp[{r_idx}:{s_idx}:{m_idx}:{d_idx}]"),
                                    failures,
                                );
                            }
                        }
                        None => {}
                    }
                }
            }
        }
    }
}

fn visit_traces_strict(
    data: &TracesData,
    domains: &[AttributeDomain],
    cfg: &AttributeCheckConfig,
    failures: &mut Vec<String>,
) {
    let want_resource = domains.contains(&AttributeDomain::Resource);
    let want_scope = domains.contains(&AttributeDomain::Scope);
    let want_signal = domains.contains(&AttributeDomain::Signal);

    for (r_idx, rs) in data.resource_spans.iter().enumerate() {
        if want_resource {
            if let Some(resource) = &rs.resource {
                check_attr_set(&resource.attributes, cfg, &format!("resource[{r_idx}]"), failures);
            }
        }
        for (s_idx, ss) in rs.scope_spans.iter().enumerate() {
            if want_scope {
                if let Some(scope) = &ss.scope {
                    check_attr_set(&scope.attributes, cfg, &format!("scope[{r_idx}:{s_idx}]"), failures);
                }
            }
            if want_signal {
                for (sp_idx, span) in ss.spans.iter().enumerate() {
                    check_attr_set(
                        &span.attributes,
                        cfg,
                        &format!("span[{r_idx}:{s_idx}:{sp_idx}]"),
                        failures,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_pdata::proto::opentelemetry::{
        common::v1::{AnyValue, KeyValue},
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
        resource::v1::Resource,
    };

    #[test]
    fn passes_when_requirements_met() {
        let logs = OtlpProtoMessage::Logs(LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue::new("service.name", AnyValue::new_string("svc"))],
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    log_records: vec![LogRecord {
                        attributes: vec![KeyValue::new("env", AnyValue::new_string("prod"))],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        });

        let cfg = AttributeCheckConfig {
            domains: vec![AttributeDomain::Signal],
            require_keys: vec!["env".into()],
            require_key_values: vec![ExpectedKeyValue {
                key: "env".into(),
                value: AttributeValue::Str("prod".into()),
            }],
            forbid_keys: vec![],
        };

        assert!(check_attributes(&[logs], &cfg));
    }

    #[test]
    fn fails_on_missing_and_forbidden_keys() {
        let logs = OtlpProtoMessage::Logs(LogsData {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: vec![LogRecord {
                        attributes: vec![KeyValue::new("env", AnyValue::new_string("prod"))],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        });

        let cfg = AttributeCheckConfig {
            domains: vec![AttributeDomain::Signal],
            require_keys: vec!["missing".into()],
            require_key_values: vec![],
            forbid_keys: vec!["env".into()],
        };

        let result = check_attributes(&[logs], &cfg);
        assert!(!result);
    }

    #[test]
    fn fails_when_each_signal_must_have_key() {
        // Two logs, only one carries the required key
        let logs = OtlpProtoMessage::Logs(LogsData {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: vec![
                        LogRecord {
                            attributes: vec![KeyValue::new("env", AnyValue::new_string("prod"))],
                            ..Default::default()
                        },
                        LogRecord {
                            attributes: vec![],
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        });

        let cfg = AttributeCheckConfig {
            domains: vec![AttributeDomain::Signal],
            require_keys: vec!["env".into()],
            require_key_values: vec![],
            forbid_keys: vec![],
        };

        let result = check_attributes(&[logs], &cfg);
        assert!(!result);
    }
}
