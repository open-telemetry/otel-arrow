// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP equivalence checking for testing round-trips through OTAP encoding.
//!
//! This module provides functions to check semantic equivalence of
//! OTLP payloads, even when the structure has been reorganized.
//!
//! The approach flattens the OTLP hierarchy into individual items
//! (log records, spans, metric data points) that combine all their
//! context (resource, scope) into a single comparable struct, which
//! is serialized via protobuf after a recursive-canonicalize step.
//!
//! See the corresponding Golang implementation in go/pkg/otel/assert/equiv.go

mod canonical;
mod logs;
mod metrics;
mod traces;

use crate::proto::OtlpProtoMessage;
use otap_df_config::SignalType;

use crate::proto::opentelemetry::common::v1::KeyValue;
use crate::proto::opentelemetry::logs::v1::LogsData;
use crate::proto::opentelemetry::metrics::v1::MetricsData;
use crate::proto::opentelemetry::trace::v1::TracesData;

use crate::proto::opentelemetry::metrics::v1::{Exemplar, Gauge, Histogram, Sum, metric};
use prost::Message;

use canonical::canonicalize_any_value;
use logs::assert_logs_equivalent;
use metrics::assert_metrics_equivalent;
use traces::assert_traces_equivalent;

fn otap_to_otlp_logs(msg: &OtlpProtoMessage) -> LogsData {
    match msg {
        OtlpProtoMessage::Logs(logs) => logs.clone(),
        _ => panic!("expected logs"),
    }
}

fn otap_to_otlp_metrics(msg: &OtlpProtoMessage) -> MetricsData {
    match msg {
        OtlpProtoMessage::Metrics(metrics) => metrics.clone(),
        _ => panic!("expected metrics"),
    }
}

fn otap_to_otlp_traces(msg: &OtlpProtoMessage) -> TracesData {
    match msg {
        OtlpProtoMessage::Traces(traces) => traces.clone(),
        _ => panic!("expected traces"),
    }
}

/// Asserts that two OTLP protocol message slices contain equivalent data.
/// Requires the inputs to have a single signal type.
pub fn assert_equivalent(left: &[OtlpProtoMessage], right: &[OtlpProtoMessage]) {
    let signal_type = left.first().expect("at least one input").signal_type();

    match signal_type {
        SignalType::Logs => assert_logs_equivalent(
            &left.iter().map(otap_to_otlp_logs).collect::<Vec<_>>(),
            &right.iter().map(otap_to_otlp_logs).collect::<Vec<_>>(),
        ),
        SignalType::Metrics => assert_metrics_equivalent(
            &left.iter().map(otap_to_otlp_metrics).collect::<Vec<_>>(),
            &right.iter().map(otap_to_otlp_metrics).collect::<Vec<_>>(),
        ),
        SignalType::Traces => assert_traces_equivalent(
            &left.iter().map(otap_to_otlp_traces).collect::<Vec<_>>(),
            &right.iter().map(otap_to_otlp_traces).collect::<Vec<_>>(),
        ),
    }
}

/// Asserts that two OTLP protocol message slices have equivalent **attributes only**.
/// Non-attribute fields (timestamps, names, values, etc.) are ignored.
pub fn assert_attributes_equivalent(left: &[OtlpProtoMessage], right: &[OtlpProtoMessage]) {
    let signal_type = left.first().expect("at least one input").signal_type();

    match signal_type {
        SignalType::Logs => {
            pretty_assertions::assert_eq!(
                collect_log_attributes(left),
                collect_log_attributes(right),
                "Log attributes not equivalent"
            );
        }
        SignalType::Metrics => {
            pretty_assertions::assert_eq!(
                collect_metric_attributes(left),
                collect_metric_attributes(right),
                "Metric attributes not equivalent"
            );
        }
        SignalType::Traces => {
            pretty_assertions::assert_eq!(
                collect_trace_attributes(left),
                collect_trace_attributes(right),
                "Trace attributes not equivalent"
            );
        }
    }
}

/// Returns true when attribute sets differ between `left` and `right`.
pub fn attributes_differ(left: &[OtlpProtoMessage], right: &[OtlpProtoMessage]) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        assert_attributes_equivalent(left, right)
    }))
    .is_err()
}

fn collect_log_attributes(msgs: &[OtlpProtoMessage]) -> Vec<Vec<u8>> {
    let mut encoded = Vec::new();

    for msg in msgs {
        let logs = otap_to_otlp_logs(msg);
        for resource_logs in &logs.resource_logs {
            push_kvs(
                &mut encoded,
                &resource_logs.resource.as_ref().map(|r| &r.attributes),
            );

            for scope_logs in &resource_logs.scope_logs {
                push_kvs(
                    &mut encoded,
                    &scope_logs.scope.as_ref().map(|s| &s.attributes),
                );

                for log_record in &scope_logs.log_records {
                    push_kvs(&mut encoded, &Some(&log_record.attributes));
                }
            }
        }
    }

    encoded.sort();
    encoded
}

fn collect_metric_attributes(msgs: &[OtlpProtoMessage]) -> Vec<Vec<u8>> {
    let mut encoded = Vec::new();

    for msg in msgs {
        let metrics = otap_to_otlp_metrics(msg);
        for resource_metrics in &metrics.resource_metrics {
            push_kvs(
                &mut encoded,
                &resource_metrics.resource.as_ref().map(|r| &r.attributes),
            );

            for scope_metrics in &resource_metrics.scope_metrics {
                push_kvs(
                    &mut encoded,
                    &scope_metrics.scope.as_ref().map(|s| &s.attributes),
                );

                for metric in &scope_metrics.metrics {
                    push_kvs(&mut encoded, &Some(&metric.metadata));

                    if let Some(data) = &metric.data {
                        match data {
                            metric::Data::Gauge(Gauge { data_points }) => {
                                for dp in data_points {
                                    push_kvs(&mut encoded, &Some(&dp.attributes));
                                    push_exemplar_attrs(&mut encoded, &dp.exemplars);
                                }
                            }
                            metric::Data::Sum(Sum { data_points, .. }) => {
                                for dp in data_points {
                                    push_kvs(&mut encoded, &Some(&dp.attributes));
                                    push_exemplar_attrs(&mut encoded, &dp.exemplars);
                                }
                            }
                            metric::Data::Histogram(Histogram { data_points, .. }) => {
                                for dp in data_points {
                                    push_kvs(&mut encoded, &Some(&dp.attributes));
                                    push_exemplar_attrs(&mut encoded, &dp.exemplars);
                                }
                            }
                            metric::Data::ExponentialHistogram(data) => {
                                for dp in &data.data_points {
                                    push_kvs(&mut encoded, &Some(&dp.attributes));
                                    push_exemplar_attrs(&mut encoded, &dp.exemplars);
                                }
                            }
                            metric::Data::Summary(data) => {
                                for dp in &data.data_points {
                                    push_kvs(&mut encoded, &Some(&dp.attributes));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    encoded.sort();
    encoded
}

fn collect_trace_attributes(msgs: &[OtlpProtoMessage]) -> Vec<Vec<u8>> {
    let mut encoded = Vec::new();

    for msg in msgs {
        let traces = otap_to_otlp_traces(msg);
        for resource_spans in &traces.resource_spans {
            push_kvs(
                &mut encoded,
                &resource_spans.resource.as_ref().map(|r| &r.attributes),
            );

            for scope_spans in &resource_spans.scope_spans {
                push_kvs(
                    &mut encoded,
                    &scope_spans.scope.as_ref().map(|s| &s.attributes),
                );

                for span in &scope_spans.spans {
                    push_kvs(&mut encoded, &Some(&span.attributes));

                    for event in &span.events {
                        push_kvs(&mut encoded, &Some(&event.attributes));
                    }

                    for link in &span.links {
                        push_kvs(&mut encoded, &Some(&link.attributes));
                    }
                }
            }
        }
    }

    encoded.sort();
    encoded
}

fn push_kvs(target: &mut Vec<Vec<u8>>, maybe_kvs: &Option<&Vec<KeyValue>>) {
    if let Some(kvs) = maybe_kvs {
        for kv in kvs.iter().cloned() {
            let mut kv = kv;
            if let Some(value) = &mut kv.value {
                canonicalize_any_value(value);
            }
            let mut buf = Vec::new();
            kv.encode(&mut buf).expect("encode KeyValue");
            target.push(buf);
        }
    }
}

fn push_exemplar_attrs(target: &mut Vec<Vec<u8>>, exemplars: &[Exemplar]) {
    for exemplar in exemplars {
        push_kvs(target, &Some(&exemplar.filtered_attributes));
    }
}
