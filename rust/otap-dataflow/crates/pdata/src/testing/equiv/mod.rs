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

use crate::proto::opentelemetry::logs::v1::LogsData;
use crate::proto::opentelemetry::metrics::v1::MetricsData;
use crate::proto::opentelemetry::trace::v1::TracesData;

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
