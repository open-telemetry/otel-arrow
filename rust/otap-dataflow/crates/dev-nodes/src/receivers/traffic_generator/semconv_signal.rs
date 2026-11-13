// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//!
//! The semconv signal module generates signals based on the OpenTelemetry
//! Semantic Conventions registry.
//!
//! Signals are built directly as OTAP record batches through the shared
//! [`super::otap_batch`] types and encoded to OTLP with
//! [`super::otap_batch::OtlpEncoder`], so the generated OTLP has the same wire
//! layout the engine produces.
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
    use super::super::semconv_signal_old as old;
    use super::*;
    use otel_arrow_dfe_pdata::OtlpProtoBytes;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::KeyValue;
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogsData;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{MetricsData, metric::Data};
    use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::TracesData;
    use prost::Message;
    use std::sync::OnceLock;

    fn registry() -> &'static ResolvedRegistry {
        static REGISTRY: OnceLock<ResolvedRegistry> = OnceLock::new();
        REGISTRY.get_or_init(load_semconv_registry)
    }

    fn encode(records: OtapArrowRecords) -> OtlpProtoBytes {
        OtlpEncoder::new().encode(records).expect("encode to OTLP")
    }

    /// Weaver picks attribute values at random from the registry examples,
    /// and for `any`-typed attributes even the value type is random. Two
    /// generations therefore only agree on attribute keys, so drop values.
    /// Value encoding is covered by the synthetic equivalence tests.
    fn normalize_attrs(attrs: &mut [KeyValue]) {
        for kv in attrs {
            kv.value = None;
        }
    }

    fn normalize_logs(mut logs: LogsData) -> LogsData {
        for rl in &mut logs.resource_logs {
            for sl in &mut rl.scope_logs {
                for lr in &mut sl.log_records {
                    lr.time_unix_nano = 0;
                    lr.observed_time_unix_nano = 0;
                    normalize_attrs(&mut lr.attributes);
                }
            }
        }
        logs
    }

    fn normalize_traces(mut traces: TracesData) -> TracesData {
        for rs in &mut traces.resource_spans {
            for ss in &mut rs.scope_spans {
                for span in &mut ss.spans {
                    span.end_time_unix_nano -= span.start_time_unix_nano;
                    span.start_time_unix_nano = 0;
                    span.trace_id = vec![1; 16];
                    span.span_id = vec![1; 8];
                    // OTAP has no null bitmap on the span status struct, so
                    // the encoder writes an empty `Status` where prost omits
                    // it. Both mean `Unset`.
                    if span.status == Some(Default::default()) {
                        span.status = None;
                    }
                    normalize_attrs(&mut span.attributes);
                    for event in &mut span.events {
                        event.time_unix_nano = 0;
                    }
                }
            }
        }
        traces
    }

    fn normalize_metrics(mut metrics: MetricsData) -> MetricsData {
        for rm in &mut metrics.resource_metrics {
            for sm in &mut rm.scope_metrics {
                for m in &mut sm.metrics {
                    match m.data.as_mut().expect("metric data") {
                        Data::Gauge(g) => g.data_points.iter_mut().for_each(|p| {
                            p.time_unix_nano = 0;
                            normalize_attrs(&mut p.attributes);
                        }),
                        Data::Sum(s) => s.data_points.iter_mut().for_each(|p| {
                            p.time_unix_nano = 0;
                            normalize_attrs(&mut p.attributes);
                        }),
                        Data::Histogram(h) => h.data_points.iter_mut().for_each(|p| {
                            p.time_unix_nano = 0;
                            normalize_attrs(&mut p.attributes);
                        }),
                        Data::ExponentialHistogram(_) | Data::Summary(_) => {
                            panic!("semconv does not generate this metric type")
                        }
                    }
                }
            }
        }
        metrics
    }

    #[test]
    fn logs_match_prost_implementation() {
        for count in [1, 7, 200] {
            let expected = old::semconv_otlp_logs(count, registry());
            let actual = LogsData::decode(
                encode(semconv_otap_logs(count, registry()).expect("build logs")).as_bytes(),
            )
            .expect("decode logs");
            assert_eq!(
                normalize_logs(actual),
                normalize_logs(expected),
                "count={count}"
            );
        }
    }

    #[test]
    fn traces_match_prost_implementation() {
        for count in [1, 7, 200] {
            let expected = old::semconv_otlp_traces(count, registry());
            let actual = TracesData::decode(
                encode(semconv_otap_traces(count, registry()).expect("build traces")).as_bytes(),
            )
            .expect("decode traces");
            assert_eq!(
                normalize_traces(actual),
                normalize_traces(expected),
                "count={count}"
            );
        }
    }

    #[test]
    fn metrics_match_prost_implementation() {
        for count in [1, 7, 200] {
            let expected = old::semconv_otlp_metrics(count, registry());
            let actual = MetricsData::decode(
                encode(semconv_otap_metrics(count, registry()).expect("build metrics")).as_bytes(),
            )
            .expect("decode metrics");
            assert_eq!(
                normalize_metrics(actual),
                normalize_metrics(expected),
                "count={count}"
            );
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
