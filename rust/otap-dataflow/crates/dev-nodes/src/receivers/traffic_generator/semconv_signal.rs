// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//!
//! The semconv signal module generates signals based on the OpenTelemetry
//! Semantic Conventions registry.
//!
//! Signals are built directly as OTAP record batches and encoded to OTLP with
//! [`super::synthetic_signal::OtlpEncoder`], so the generated OTLP has the same
//! wire layout the engine produces.
//!
//! ToDo: Add profile signal support

use crate::receivers::traffic_generator::attributes::get_attribute_name_value;
use crate::receivers::traffic_generator::fake_data::{
    SCOPE_NAME, SCOPE_VERSION, current_time, delay, gen_span_id_bytes, gen_trace_id_bytes,
};
use crate::receivers::traffic_generator::synthetic_signal::set_payloads;
use otel_arrow_dfe_pdata::arrow::datatypes::ArrowPrimitiveType;
use otel_arrow_dfe_pdata::encode::append_attribute_value;
use otel_arrow_dfe_pdata::encode::record::attributes::{
    AttributesRecordBatchBuilder, AttributesRecordBatchBuilderConstructorHelper,
};
use otel_arrow_dfe_pdata::encode::record::logs::LogsRecordBatchBuilder;
use otel_arrow_dfe_pdata::encode::record::metrics::{
    HistogramDataPointsRecordBatchBuilder, MetricsRecordBatchBuilder,
    NumberDataPointsRecordBatchBuilder,
};
use otel_arrow_dfe_pdata::encode::record::traces::{
    EventsRecordBatchBuilder, TracesRecordBatchBuilder,
};
use otel_arrow_dfe_pdata::encode::{Error, Result};
use otel_arrow_dfe_pdata::otap::{Logs, Metrics, OtapArrowRecords, Traces};
use otel_arrow_dfe_pdata::otlp::attributes::parent_id::ParentId;
use otel_arrow_dfe_pdata::otlp::metrics::MetricType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::KeyValue;
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::SeverityNumber;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::AggregationTemporality;
use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::span::SpanKind;
use otel_arrow_dfe_pdata::views::otlp::proto::common::ObjKeyValue;
use otel_arrow_dfe_pdata::views::otlp::proto::wrappers::GenericObj;
use weaver_forge::v1::registry::ResolvedRegistry;
use weaver_semconv::v1::group::{GroupType, InstrumentSpec, SpanKindSpec};

/// Generates an OTAP traces batch with `signal_count` spans, cycling through
/// the span groups defined in the registry.
pub fn semconv_otap_traces(
    signal_count: usize,
    registry: &ResolvedRegistry,
) -> Result<OtapArrowRecords> {
    let groups: Vec<_> = registry
        .groups
        .iter()
        .filter(|g| g.r#type == GroupType::Span)
        .collect();
    if signal_count == 0 || groups.is_empty() {
        return Ok(OtapArrowRecords::Traces(Traces::default()));
    }
    let span_count = u16::try_from(signal_count).map_err(|_| Error::U16OverflowError)?;

    let mut resource_attrs = AttributesRecordBatchBuilder::<u16>::new();
    append_resource_attributes(&mut resource_attrs);

    let mut spans = TracesRecordBatchBuilder::new();
    let mut span_attrs = AttributesRecordBatchBuilder::<u16>::new();
    let mut events = EventsRecordBatchBuilder::new();
    let mut next_event_id: u32 = 0;

    spans.resource.append_id_n(0, signal_count);
    spans.resource.append_schema_url_n(None, signal_count);
    spans
        .resource
        .append_dropped_attributes_count_n(0, signal_count);
    spans.scope.append_id_n(0, signal_count);
    spans
        .scope
        .append_name_n(Some(SCOPE_NAME.as_bytes()), signal_count);
    spans
        .scope
        .append_version_n(Some(SCOPE_VERSION.as_bytes()), signal_count);
    spans
        .scope
        .append_dropped_attributes_count_n(0, signal_count);
    spans.append_schema_url_n(None, signal_count);

    for span_id in 0..span_count {
        let group = groups[span_id as usize % groups.len()];
        let start_time = current_time();
        let end_time = start_time + delay();

        spans.append_id(Some(span_id));
        spans.append_start_time_unix_nano(start_time as i64);
        spans.append_duration_time_unix_nano(end_time as i64 - start_time as i64);
        spans.append_trace_id(gen_trace_id_bytes())?;
        spans.append_span_id(gen_span_id_bytes())?;
        spans.append_trace_state(None);
        spans.append_parent_span_id(None)?;
        spans.append_flags(None);
        spans.append_name(group.id.as_bytes());
        spans.append_kind(Some(otel_span_kind(group.span_kind.as_ref()) as i32));
        spans.append_dropped_attributes_count(Some(0));
        spans.append_dropped_events_count(Some(0));
        spans.append_dropped_links_count(Some(0));
        spans.status.append_code(None);
        spans.status.append_status_message(None);

        for attribute in &group.attributes {
            append_key_value(
                &mut span_attrs,
                span_id,
                &get_attribute_name_value(attribute),
            )?;
        }

        for event_name in &group.events {
            events.append_id(Some(next_event_id));
            events.append_parent_id(span_id);
            events.append_time_unix_nano(Some(current_time() as i64));
            events.append_name(event_name.as_bytes());
            events.append_dropped_attributes_count(Some(0));
            next_event_id = next_event_id
                .checked_add(1)
                .ok_or(Error::U32OverflowError)?;
        }
    }

    let mut records = OtapArrowRecords::Traces(Traces::default());
    set_payloads(
        &mut records,
        [
            (spans.finish()?, ArrowPayloadType::Spans),
            (events.finish()?, ArrowPayloadType::SpanEvents),
            (span_attrs.finish()?, ArrowPayloadType::SpanAttrs),
            (resource_attrs.finish()?, ArrowPayloadType::ResourceAttrs),
        ],
    )?;
    Ok(records)
}

/// Generates an OTAP logs batch with `signal_count` records, cycling through
/// the event groups (structured logs) defined in the registry.
pub fn semconv_otap_logs(
    signal_count: usize,
    registry: &ResolvedRegistry,
) -> Result<OtapArrowRecords> {
    let groups: Vec<_> = registry
        .groups
        .iter()
        .filter(|g| g.r#type == GroupType::Event)
        .collect();
    if signal_count == 0 || groups.is_empty() {
        return Ok(OtapArrowRecords::Logs(Logs::default()));
    }
    let log_count = u16::try_from(signal_count).map_err(|_| Error::U16OverflowError)?;

    let mut resource_attrs = AttributesRecordBatchBuilder::<u16>::new();
    append_resource_attributes(&mut resource_attrs);

    let mut logs = LogsRecordBatchBuilder::new();
    let mut log_attrs = AttributesRecordBatchBuilder::<u16>::new();

    logs.resource.append_id_n(0, signal_count);
    logs.resource.append_schema_url_n(None, signal_count);
    logs.resource
        .append_dropped_attributes_count_n(0, signal_count);
    logs.scope.append_id_n(0, signal_count);
    logs.scope
        .append_name_n(Some(SCOPE_NAME.as_bytes()), signal_count);
    logs.scope
        .append_version_n(Some(SCOPE_VERSION.as_bytes()), signal_count);
    logs.scope
        .append_dropped_attributes_count_n(0, signal_count);
    logs.append_schema_url_n(None, signal_count);
    logs.append_dropped_attributes_count_n(0, signal_count);
    logs.append_flags_n(None, signal_count);

    // Reused across records so rendering each body does not allocate.
    let mut body = String::new();
    for log_id in 0..log_count {
        let group = groups[log_id as usize % groups.len()];
        let timestamp = current_time() as i64;

        logs.append_time_unix_nano(timestamp);
        logs.append_observed_time_unix_nano(timestamp);
        logs.append_severity_number(Some(SeverityNumber::Info as i32));
        logs.append_severity_text(None);
        logs.append_event_name(
            group
                .name
                .as_deref()
                .filter(|name| !name.is_empty())
                .map(str::as_bytes),
        );
        logs.append_trace_id(None)?;
        logs.append_span_id(None)?;

        body.clear();
        if let Some(group_body) = &group.body {
            use std::fmt::Write;
            write!(body, "{group_body}").expect("writing to a String cannot fail");
        }
        logs.body.append_str(body.as_bytes());

        // Log ids only exist to link attributes, so records without
        // attributes get a null id.
        if group.attributes.is_empty() {
            logs.append_id(None);
            continue;
        }

        logs.append_id(Some(log_id));
        for attribute in &group.attributes {
            append_key_value(&mut log_attrs, log_id, &get_attribute_name_value(attribute))?;
        }
    }

    let mut records = OtapArrowRecords::Logs(Logs::default());
    set_payloads(
        &mut records,
        [
            (logs.finish()?, ArrowPayloadType::Logs),
            (log_attrs.finish()?, ArrowPayloadType::LogAttrs),
            (resource_attrs.finish()?, ArrowPayloadType::ResourceAttrs),
        ],
    )?;
    Ok(records)
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
    let groups: Vec<_> = registry
        .groups
        .iter()
        .filter(|g| g.r#type == GroupType::Metric)
        .collect();
    if signal_count == 0 || groups.is_empty() {
        return Ok(OtapArrowRecords::Metrics(Metrics::default()));
    }
    _ = u16::try_from(signal_count).map_err(|_| Error::U16OverflowError)?;

    let mut resource_attrs = AttributesRecordBatchBuilder::<u16>::new();
    append_resource_attributes(&mut resource_attrs);

    let mut metrics = MetricsRecordBatchBuilder::new();
    let mut ndp = NumberDataPointsRecordBatchBuilder::new();
    let mut ndp_attrs = AttributesRecordBatchBuilder::<u32>::new();
    let mut hdp = HistogramDataPointsRecordBatchBuilder::new();
    let mut hdp_attrs = AttributesRecordBatchBuilder::<u32>::new();
    let mut metric_id: u16 = 0;
    let mut next_ndp_id: u32 = 0;
    let mut next_hdp_id: u32 = 0;

    for i in 0..signal_count {
        let group = groups[i % groups.len()];
        let Some(instrument) = &group.instrument else {
            continue;
        };

        let (metric_type, temporality, monotonic) = match instrument {
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

        metrics.append_id(metric_id);
        metrics.append_metric_type(metric_type as u8);
        metrics.append_name(group.metric_name.as_deref().unwrap_or("").as_bytes());
        metrics.append_description(group.brief.as_bytes());
        metrics.append_unit(group.unit.as_deref().unwrap_or("").as_bytes());
        metrics.append_aggregation_temporality(temporality.map(|t| t as i32));
        metrics.append_is_monotonic(monotonic);

        if metric_type == MetricType::Histogram {
            let id = next_hdp_id;
            next_hdp_id += 1;
            hdp.append_id(id);
            hdp.append_parent_id(metric_id);
            hdp.append_start_time_unix_nano(0);
            hdp.append_time_unix_nano(current_time() as i64);
            hdp.append_count(0);
            hdp.append_bucket_counts(std::iter::empty());
            hdp.append_explicit_bounds(std::iter::empty());
            hdp.append_sum(None);
            hdp.append_flags(0);
            hdp.append_min(None);
            hdp.append_max(None);
            for attribute in &group.attributes {
                append_key_value(&mut hdp_attrs, id, &get_attribute_name_value(attribute))?;
            }
        } else {
            let id = next_ndp_id;
            next_ndp_id += 1;
            ndp.append_id(id);
            ndp.append_parent_id(metric_id);
            ndp.append_start_time_unix_nano(None);
            ndp.append_time_unix_nano(current_time() as i64);
            ndp.append_double_value(Some(1.0));
            ndp.append_int_value(None);
            ndp.append_flags(0);
            for attribute in &group.attributes {
                append_key_value(&mut ndp_attrs, id, &get_attribute_name_value(attribute))?;
            }
        }

        metric_id += 1;
    }

    let metric_count = metric_id as usize;
    if metric_count == 0 {
        return Ok(OtapArrowRecords::Metrics(Metrics::default()));
    }
    metrics.resource.append_id_n(0, metric_count);
    metrics.resource.append_schema_url_n(None, metric_count);
    metrics
        .resource
        .append_dropped_attributes_count_n(0, metric_count);
    metrics.scope.append_id_n(0, metric_count);
    metrics
        .scope
        .append_name_n(Some(SCOPE_NAME.as_bytes()), metric_count);
    metrics
        .scope
        .append_version_n(Some(SCOPE_VERSION.as_bytes()), metric_count);
    metrics
        .scope
        .append_dropped_attributes_count_n(0, metric_count);
    metrics.append_scope_schema_url_n(b"", metric_count);

    let mut records = OtapArrowRecords::Metrics(Metrics::default());
    set_payloads(
        &mut records,
        [
            (metrics.finish()?, ArrowPayloadType::UnivariateMetrics),
            (resource_attrs.finish()?, ArrowPayloadType::ResourceAttrs),
            (ndp.finish()?, ArrowPayloadType::NumberDataPoints),
            (ndp_attrs.finish()?, ArrowPayloadType::NumberDpAttrs),
            (hdp.finish()?, ArrowPayloadType::HistogramDataPoints),
            (hdp_attrs.finish()?, ArrowPayloadType::HistogramDpAttrs),
        ],
    )?;
    Ok(records)
}

/// Resource attributes shared by all semconv batches.
fn append_resource_attributes(builder: &mut AttributesRecordBatchBuilder<u16>) {
    builder.append_parent_id(&0);
    builder.append_key(b"traffic_generator");
    builder.any_values_builder.append_str(b"v1");
}

/// Append one generated attribute, of any value type, to an attributes builder.
fn append_key_value<T>(
    builder: &mut AttributesRecordBatchBuilder<T>,
    parent_id: <T::ArrayType as ArrowPrimitiveType>::Native,
    kv: &KeyValue,
) -> Result<()>
where
    T: ParentId + AttributesRecordBatchBuilderConstructorHelper,
{
    builder.append_parent_id(&parent_id);
    let view = ObjKeyValue::new(&kv.key, kv.value.as_ref().map(|inner| GenericObj { inner }));
    append_attribute_value(builder, &view)
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
    use super::super::semconv_signal_old as old;
    use super::super::synthetic_signal::OtlpEncoder;
    use super::*;
    use otel_arrow_dfe_pdata::OtlpProtoBytes;
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
