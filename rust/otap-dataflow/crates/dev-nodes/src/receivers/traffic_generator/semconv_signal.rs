// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//!
//! The semconv signal module generates signals based on the OpenTelemetry
//! Semantic Conventions registry.
//!
//! Signals are built directly as OTAP record batches through the shared
//! [`otap_batch`] types and encoded to OTLP with [`OtlpEncoder`], so the
//! generated OTLP has the same wire layout the engine produces.
//!
//! [`otap_batch`]: crate::receivers::traffic_generator::otap_batch
//! [`OtlpEncoder`]: crate::receivers::traffic_generator::otap_batch::OtlpEncoder
//!
//! ToDo: Add profile signal support

use super::otap_batch::{
    AttrValue, HistogramPoint, LogRecordRow, LogsBatch, MetricRow, MetricsBatch, NumberPoint,
    TracesBatch, append_attribute, append_key_value,
};
use crate::receivers::traffic_generator::attributes::get_attribute_name_value;
use crate::receivers::traffic_generator::fake_data::current_time;
use otel_arrow_dfe_pdata::encode::Result;
use otel_arrow_dfe_pdata::encode::record::attributes::AttributesRecordBatchBuilder;
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use otel_arrow_dfe_pdata::otlp::metrics::MetricType;
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::SeverityNumber;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::AggregationTemporality;
use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::span::SpanKind;
use weaver_forge::v1::registry::ResolvedRegistry;
use weaver_semconv::v1::group::{GroupType, InstrumentSpec, SpanKindSpec};

/// Generates an OTAP traces batch with `signal_count` spans, cycling through
/// the span groups defined in the registry.
pub fn semconv_otap_traces(
    signal_count: usize,
    registry: &ResolvedRegistry,
) -> Result<OtapArrowRecords> {
    let mut batch = TracesBatch::new();
    let groups: Vec<_> = registry
        .groups
        .iter()
        .filter(|g| g.r#type == GroupType::Span)
        .collect();
    if signal_count == 0 || groups.is_empty() {
        return batch.finish();
    }
    append_resource_attributes(&mut batch.resource_attrs);

    for i in 0..signal_count {
        let group = groups[i % groups.len()];
        let span_id = batch.append_span(&group.id, otel_span_kind(group.span_kind.as_ref()))?;
        for attribute in &group.attributes {
            append_key_value(
                &mut batch.span_attrs,
                span_id,
                &get_attribute_name_value(attribute),
            )?;
        }
        for event_name in &group.events {
            batch.append_event(span_id, event_name)?;
        }
    }
    batch.finish()
}

/// Generates an OTAP logs batch with `signal_count` records, cycling through
/// the event groups (structured logs) defined in the registry.
pub fn semconv_otap_logs(
    signal_count: usize,
    registry: &ResolvedRegistry,
) -> Result<OtapArrowRecords> {
    let mut batch = LogsBatch::new();
    let groups: Vec<_> = registry
        .groups
        .iter()
        .filter(|g| g.r#type == GroupType::Event)
        .collect();
    if signal_count == 0 || groups.is_empty() {
        return batch.finish();
    }
    append_resource_attributes(&mut batch.resource_attrs);

    // Reused across records so rendering each body does not allocate.
    let mut body = String::new();
    for i in 0..signal_count {
        let group = groups[i % groups.len()];
        body.clear();
        if let Some(group_body) = &group.body {
            use std::fmt::Write;
            write!(body, "{group_body}").expect("writing to a String cannot fail");
        }

        let log_id = batch.append_log(LogRecordRow {
            severity_number: SeverityNumber::Info,
            severity_text: None,
            event_name: group.name.as_deref().filter(|name| !name.is_empty()),
            trace_context: false,
            body: Some(&body),
            has_attributes: !group.attributes.is_empty(),
        })?;
        for attribute in &group.attributes {
            append_key_value(
                &mut batch.log_attrs,
                log_id,
                &get_attribute_name_value(attribute),
            )?;
        }
    }
    batch.finish()
}

/// Generates an OTAP metrics batch by cycling through the metric groups
/// defined in the registry, one data point per metric.
///
/// Groups without an instrument are skipped, so the batch can contain fewer
/// than `signal_count` metrics.
pub fn semconv_otap_metrics(
    signal_count: usize,
    registry: &ResolvedRegistry,
) -> Result<OtapArrowRecords> {
    let mut batch = MetricsBatch::new();
    let groups: Vec<_> = registry
        .groups
        .iter()
        .filter(|g| g.r#type == GroupType::Metric)
        .collect();
    if signal_count == 0 || groups.is_empty() {
        return batch.finish();
    }
    append_resource_attributes(&mut batch.resource_attrs);

    for i in 0..signal_count {
        let group = groups[i % groups.len()];
        let Some(instrument) = &group.instrument else {
            continue;
        };
        let (metric_type, temporality, is_monotonic) = match instrument {
            InstrumentSpec::Counter => (
                MetricType::Sum,
                Some(AggregationTemporality::Unspecified),
                Some(true),
            ),
            InstrumentSpec::UpDownCounter => (
                MetricType::Sum,
                Some(AggregationTemporality::Unspecified),
                Some(false),
            ),
            InstrumentSpec::Gauge => (MetricType::Gauge, None, None),
            InstrumentSpec::Histogram => (
                MetricType::Histogram,
                Some(AggregationTemporality::Unspecified),
                None,
            ),
        };
        let metric_id = batch.append_metric(MetricRow {
            metric_type,
            name: group.metric_name.as_deref().unwrap_or(""),
            description: &group.brief,
            unit: group.unit.as_deref().unwrap_or(""),
            temporality,
            is_monotonic,
        })?;

        let time = current_time();
        let (attrs, id) = if metric_type == MetricType::Histogram {
            let id = batch.append_histogram_point(
                metric_id,
                HistogramPoint {
                    start_time: 0,
                    time,
                    count: 0,
                    sum: None,
                    bucket_counts: &[],
                    explicit_bounds: &[],
                    min: None,
                    max: None,
                },
            )?;
            (&mut batch.hdp_attrs, id)
        } else {
            let id = batch.append_number_point(
                metric_id,
                NumberPoint {
                    start_time: None,
                    time,
                    value: 1.0,
                },
            )?;
            (&mut batch.ndp_attrs, id)
        };
        for attribute in &group.attributes {
            append_key_value(attrs, id, &get_attribute_name_value(attribute))?;
        }
    }
    batch.finish()
}

/// Resource attributes shared by all semconv batches.
fn append_resource_attributes(builder: &mut AttributesRecordBatchBuilder<u16>) {
    append_attribute(builder, 0, "traffic_generator", AttrValue::Str("v1"));
}

/// map a SpanKindSpec to a SpanKind
#[must_use]
const fn otel_span_kind(span_kind: Option<&SpanKindSpec>) -> SpanKind {
    match span_kind {
        Some(SpanKindSpec::Client) => SpanKind::Client,
        Some(SpanKindSpec::Server) => SpanKind::Server,
        Some(SpanKindSpec::Producer) => SpanKind::Producer,
        Some(SpanKindSpec::Consumer) => SpanKind::Consumer,
        Some(SpanKindSpec::Internal) | None => SpanKind::Internal,
    }
}

#[cfg(test)]
mod tests {
    use super::super::compression_ratio_tests::load_semconv_registry;
    use super::super::otap_batch::OtlpEncoder;
    use super::*;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::KeyValue;
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogsData;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{MetricsData, metric::Data};
    use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::TracesData;
    use prost::Message;
    use std::sync::OnceLock;
    use weaver_forge::v1::registry::ResolvedGroup;

    fn registry() -> &'static ResolvedRegistry {
        static REGISTRY: OnceLock<ResolvedRegistry> = OnceLock::new();
        REGISTRY.get_or_init(load_semconv_registry)
    }

    fn groups(group_type: GroupType) -> Vec<&'static ResolvedGroup> {
        registry()
            .groups
            .iter()
            .filter(|g| g.r#type == group_type)
            .collect()
    }

    fn encode(records: OtapArrowRecords) -> Vec<u8> {
        OtlpEncoder::new()
            .encode(records)
            .expect("encode to OTLP")
            .as_bytes()
            .to_vec()
    }

    /// Weaver picks attribute values at random from the registry examples,
    /// so only the keys are deterministic.
    fn keys(attrs: &[KeyValue]) -> Vec<&str> {
        attrs.iter().map(|kv| kv.key.as_str()).collect()
    }

    fn group_keys(group: &ResolvedGroup) -> Vec<&str> {
        group.attributes.iter().map(|a| a.name.as_str()).collect()
    }

    #[test]
    fn logs_follow_event_groups() {
        let groups = groups(GroupType::Event);
        let count = groups.len() + 3;
        let logs = LogsData::decode(
            encode(semconv_otap_logs(count, registry()).expect("build logs")).as_slice(),
        )
        .expect("decode logs");

        assert_eq!(logs.resource_logs.len(), 1);
        let resource = logs.resource_logs[0].resource.as_ref().expect("resource");
        assert_eq!(keys(&resource.attributes), ["traffic_generator"]);
        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        assert_eq!(records.len(), count);
        for (i, record) in records.iter().enumerate() {
            let group = groups[i % groups.len()];
            let expected_name = group.name.as_deref().unwrap_or("");
            assert_eq!(record.event_name, expected_name, "record {i}");
            assert_eq!(keys(&record.attributes), group_keys(group), "record {i}");
            assert_eq!(record.severity_number, SeverityNumber::Info as i32);
            assert!(record.body.is_some());
        }
    }

    #[test]
    fn traces_follow_span_groups() {
        let groups = groups(GroupType::Span);
        let count = groups.len() + 3;
        let traces = TracesData::decode(
            encode(semconv_otap_traces(count, registry()).expect("build traces")).as_slice(),
        )
        .expect("decode traces");

        let spans = &traces.resource_spans[0].scope_spans[0].spans;
        assert_eq!(spans.len(), count);
        for (i, span) in spans.iter().enumerate() {
            let group = groups[i % groups.len()];
            assert_eq!(span.name, group.id, "span {i}");
            assert_eq!(
                span.kind,
                otel_span_kind(group.span_kind.as_ref()) as i32,
                "span {i}"
            );
            assert_eq!(keys(&span.attributes), group_keys(group), "span {i}");
            let events: Vec<&str> = span.events.iter().map(|e| e.name.as_str()).collect();
            assert_eq!(events, group.events, "span {i}");
            assert_eq!(span.trace_id.len(), 16);
            assert_eq!(span.span_id.len(), 8);
        }
    }

    #[test]
    fn metrics_follow_metric_groups() {
        let groups = groups(GroupType::Metric);
        let count = groups.len() + 3;
        let metrics = MetricsData::decode(
            encode(semconv_otap_metrics(count, registry()).expect("build metrics")).as_slice(),
        )
        .expect("decode metrics");

        // Groups without an instrument are skipped.
        let expected: Vec<&ResolvedGroup> = (0..count)
            .map(|i| groups[i % groups.len()])
            .filter(|g| g.instrument.is_some())
            .collect();
        let list = &metrics.resource_metrics[0].scope_metrics[0].metrics;
        assert_eq!(list.len(), expected.len());
        for (metric, group) in list.iter().zip(expected) {
            assert_eq!(metric.name, group.metric_name.as_deref().unwrap_or(""));
            assert_eq!(metric.unit, group.unit.as_deref().unwrap_or(""));
            let attrs = match (metric.data.as_ref(), group.instrument.as_ref()) {
                (Some(Data::Sum(s)), Some(InstrumentSpec::Counter)) => {
                    assert!(s.is_monotonic);
                    &s.data_points[0].attributes
                }
                (Some(Data::Sum(s)), Some(InstrumentSpec::UpDownCounter)) => {
                    assert!(!s.is_monotonic);
                    &s.data_points[0].attributes
                }
                (Some(Data::Gauge(g)), Some(InstrumentSpec::Gauge)) => &g.data_points[0].attributes,
                (Some(Data::Histogram(h)), Some(InstrumentSpec::Histogram)) => {
                    &h.data_points[0].attributes
                }
                (data, instrument) => {
                    panic!("metric {} has {data:?} for {instrument:?}", metric.name)
                }
            };
            assert_eq!(keys(attrs), group_keys(group), "metric {}", metric.name);
        }
    }

    #[test]
    fn zero_count_produces_empty_batches() {
        for records in [
            semconv_otap_logs(0, registry()).expect("logs"),
            semconv_otap_traces(0, registry()).expect("traces"),
            semconv_otap_metrics(0, registry()).expect("metrics"),
        ] {
            assert_eq!(records.num_items(), 0);
        }
    }
}
