// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Projects registry-backed metric sets into standard OTLP metrics.
//!
//! # Why this bridge exists
//!
//! The internal telemetry system records a *metric set*: one multivariate value
//! containing several metric fields that share an entity, attributes, and a
//! collection time. The current OTLP and OTAP export paths operate on standard
//! univariate metrics instead. This module is the boundary that expands each
//! field of a metric set into one OTLP [`Metric`], allowing the resulting pdata
//! to flow through either:
//!
//! - an OTLP exporter, which forwards the encoded request; or
//! - an OTAP exporter, which converts the same OTLP request into OTAP Arrow
//!   records before export.
//!
//! The registry first drains its export accumulator into an owned
//! [`MetricExportBatch`]. Consequently, this module never holds the registry
//! mutex while allocating protobuf values, encoding bytes, or waiting for a
//! downstream pipeline.
//!
//! # Input contract
//!
//! [`MetricExportBatch`] is produced only by the registry's atomic export
//! drain. Each [`MetricSetExport`] contains a static descriptor, an owned view
//! of its entity attributes, and a value vector with exactly the same length
//! and ordering as the descriptor's field vector. The metric-set macro and
//! registry maintain the value-kind/instrument pairing. Sum-like descriptors
//! must also declare a [`Temporality`]; a missing temporality is reported as
//! [`Error::MissingTemporality`] instead of emitting an ambiguous OTLP sum.
//! The encoder validates these invariants as well, so malformed batches fail
//! explicitly instead of silently truncating or substituting values.
//!
//! # Output hierarchy
//!
//! Each batch is projected into the following protobuf hierarchy:
//!
//! ```text
//! ExportMetricsServiceRequest
//! `-- ResourceMetrics                         process resource
//!     `-- ScopeMetrics                        one per metric-set schema + entity
//!         |-- InstrumentationScope
//!         |   |-- name                        metric-set descriptor name
//!         |   `-- attributes                  entity attributes
//!         `-- Metric                          one per metric-set field
//!             `-- NumberDataPoint or HistogramDataPoint
//! ```
//!
//! A [`MetricSetExport`] keeps values in the same order as
//! [`MetricsDescriptor::metrics`](crate::descriptor::MetricsDescriptor::metrics).
//! `encode_metric_set` zips those two slices, so the descriptor supplies each
//! OTLP metric's name, description, unit, instrument kind, and temporality while
//! the corresponding [`MetricValue`] supplies its data-point value.
//!
//! Entity attributes are placed on `InstrumentationScope` rather than repeated
//! on every data point. Measurement and registration attributes identify a
//! metric-set bucket and are attached to its data points. Resource attributes
//! come from the process-level `ResourceMetrics` prototype retained by
//! [`MetricsOtlpEncoder`].
//!
//! # Instrument mapping
//!
//! | Internal instrument | OTLP representation | Start time and semantics |
//! | --- | --- | --- |
//! | `Counter` | monotonic `Sum` | Descriptor temporality; delta-window or registration start |
//! | `UpDownCounter` | non-monotonic `Sum` | Descriptor temporality; delta-window or registration start |
//! | `Gauge` | `Gauge` | Start time is zero, as required for an instantaneous value |
//! | scalar `Histogram` | delta `Histogram` with one observation | Delta-window start; uses the bridge's stable explicit bounds |
//! | `Mmsc` | delta, bucketless `ExponentialHistogram` | Delta-window start; preserves exact min, max, sum, and count |
//!
//! Every point uses [`MetricExportBatch::time_unix_nano`] as its end time. A
//! dirty scalar field is emitted even when its value is zero, because zero may
//! be a meaningful gauge or cumulative transition. An empty `Mmsc` is omitted
//! because its min and max are internal sentinel values, not observations. An
//! otherwise empty batch produces no pdata.
//!
//! Aggregation and reset policy intentionally remain in the registry rather
//! than in this encoder. During an atomic drain, delta sums and histograms are
//! reset, cumulative sums and gauges retain their latest values, and the next
//! delta window begins. Multiple registered metric-set keys can still resolve
//! to the same descriptor and entity attributes. Before projection, the
//! encoder coalesces those keys into one OTLP stream identity, adding sums and
//! histograms and retaining the last gauge value. The work remains owned and
//! lock-free.
//!
//! OTLP integer data points are signed `i64`, whereas internal counters can be
//! `u64`. Values above `i64::MAX` are saturated instead of wrapping. Attribute
//! values retain their native OTLP type; unsigned attribute values follow the
//! same saturation rule, and map attributes become OTLP key-value lists.
//!
//! # Transitional design
//!
//! This univariate projection is a compatibility bridge, not the intended
//! long-term representation of internal metric sets. We plan to investigate
//! native multivariate metric-set support in OTAP so the shared structure does
//! not need to be expanded at this boundary. We may also investigate a native
//! metric-set representation in OTLP if the protocol gains suitable standard
//! support, or if an interoperable extension can be defined. Keeping the
//! projection isolated in this module makes either future path replaceable
//! without changing the hot-path metric-set API or registry aggregation model.

use crate::attributes::{AttributeSetHandler, AttributeValue};
use crate::descriptor::{Instrument, MetricsField, Temporality};
use crate::entity::EntityAttributeSet;
use crate::instrument::DistributionValue;
use crate::metrics::{MetricExportBatch, MetricSetExport, MetricValue};
use bytes::Bytes;
use otap_df_config::pipeline::telemetry::AttributeValue as ConfigAttributeValue;
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
use otap_df_pdata::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, ExponentialHistogram, Gauge, Histogram, HistogramDataPoint, Metric,
    NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, metric,
};
use prost::Message;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Errors produced while encoding registry metrics as OTLP.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The pre-encoded resource fragment was invalid.
    #[error("invalid internal telemetry resource: {0}")]
    InvalidResource(#[from] prost::DecodeError),

    /// A sum-like metric did not declare its aggregation temporality.
    #[error("sum metric '{metric}' is missing aggregation temporality")]
    MissingTemporality {
        /// Metric name from the descriptor.
        metric: &'static str,
    },

    /// A metric set's value vector did not match its descriptor's field count.
    #[error(
        "metric set '{metric_set}' contains {actual} values, but its descriptor defines {expected} fields"
    )]
    ValueCountMismatch {
        /// Metric-set descriptor name.
        metric_set: &'static str,
        /// Number of fields declared by the descriptor.
        expected: usize,
        /// Number of values supplied by the export batch.
        actual: usize,
    },

    /// A metric value did not match the kind declared by its descriptor.
    #[error("metric '{metric}' expected a {expected} value, but received {actual}")]
    ValueKindMismatch {
        /// Metric name from the descriptor.
        metric: &'static str,
        /// Value kind required by the descriptor and instrument.
        expected: &'static str,
        /// Value kind found in the export batch.
        actual: &'static str,
    },

    /// Views mapped two source fields to the same OTLP metric stream name.
    #[error(
        "instrumentation scope '{scope_name}' maps fields '{first_metric}' ('{first_name}') and '{second_metric}' ('{second_name}') to case-insensitively conflicting OTLP metric names"
    )]
    MetricNameCollision {
        /// Instrumentation scope containing both projected streams.
        scope_name: String,
        /// First source metric field that claimed the output name.
        first_metric: &'static str,
        /// Output name produced for the first field.
        first_name: String,
        /// Second source metric field that produced the collision.
        second_metric: &'static str,
        /// Output name produced for the second field.
        second_name: String,
    },
}

/// A supported subset of metric view behavior.
///
/// A metric field can match more than one view. Each matching view produces an
/// OTLP metric stream, while a field that matches no views retains its
/// descriptor-defined stream. Identical results for one source field are
/// deduplicated using a case-insensitive name, retaining the longest
/// description because descriptions are not identifying OTLP properties.
/// Mapping different source fields in the same effective scope to the same
/// case-insensitive name is rejected when at least one stream was produced by
/// a view, because their already-aggregated data cannot be merged safely.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MetricView {
    /// Selects the metric fields to which the stream overrides apply.
    pub selector: MetricViewSelector,
    /// Overrides stream metadata for matching metric fields.
    pub stream: MetricViewStream,
}

/// Exact-match selectors supported by [`MetricView`].
///
/// An omitted selector matches every value for that dimension. `scope_name`
/// matches the metric-set descriptor name, `scope_attributes` requires the
/// metric-set entity to contain every configured scalar key-value pair, and
/// `instrument_name` matches a field name within that metric set.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MetricViewSelector {
    /// Exact metric-set/instrumentation-scope name to match.
    pub scope_name: Option<String>,
    /// Exact scalar entity attributes that must all be present.
    pub scope_attributes: HashMap<String, ConfigAttributeValue>,
    /// Exact metric field/instrument name to match.
    pub instrument_name: Option<String>,
}

/// Stream metadata overrides supported by [`MetricView`].
///
/// Omitted values fall back to the corresponding metric field descriptor.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetricViewStream {
    /// OTLP metric name override.
    pub name: Option<String>,
    /// OTLP metric description override.
    pub description: Option<String>,
}

/// Reusable OTLP encoder holding the process resource prototype.
#[derive(Debug, Clone)]
pub struct MetricsOtlpEncoder {
    resource_metrics: ResourceMetrics,
    views: Vec<MetricView>,
}

/// Provenance retained while checking projected stream-name collisions.
#[derive(Clone, Copy)]
struct ProjectedStream<'a> {
    source_field: &'static MetricsField,
    output_name: &'a str,
    view_applied: bool,
}

/// Registry identity corresponding to one effective OTLP instrumentation scope.
#[derive(Clone, Hash, PartialEq, Eq)]
struct ScopeIdentity {
    name: &'static str,
    attributes: Arc<EntityAttributeSet>,
}

/// Borrowed, allocation-free case-insensitive metric-name key.
#[derive(Clone, Copy)]
struct CaseInsensitiveName<'a>(&'a str);

impl PartialEq for CaseInsensitiveName<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(other.0)
    }
}

impl Eq for CaseInsensitiveName<'_> {}

impl Hash for CaseInsensitiveName<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.len().hash(state);
        for byte in self.0.bytes() {
            byte.to_ascii_lowercase().hash(state);
        }
    }
}

/// Lightweight output metadata resolved before protobuf construction.
#[derive(Clone, Copy)]
struct ResolvedStream<'a> {
    name: &'a str,
    description: &'a str,
    view_applied: bool,
}

type ProjectedSources<'a> = SmallVec<[ProjectedStream<'a>; 1]>;
type ScopeStreams<'a> = HashMap<CaseInsensitiveName<'a>, ProjectedSources<'a>>;
type CollisionIndex<'a> = HashMap<ScopeIdentity, ScopeStreams<'a>>;

/// Exact source identity that must become one OTLP instrumentation scope.
#[derive(Hash, PartialEq, Eq)]
struct MetricSetIdentity<'a> {
    descriptor: usize,
    attributes: usize,
    datapoint_attributes: &'a [(String, String)],
}

/// Avoids cloning the common case where a source identity occurs only once.
enum CoalescedMetricSet<'a> {
    Borrowed(&'a MetricSetExport),
    Owned(MetricSetExport),
}

impl CoalescedMetricSet<'_> {
    fn as_metric_set(&self) -> &MetricSetExport {
        match self {
            Self::Borrowed(metric_set) => metric_set,
            Self::Owned(metric_set) => metric_set,
        }
    }
}

impl MetricsOtlpEncoder {
    /// Creates an encoder from the resource fragment shared with internal logs.
    ///
    /// `ResourceLogs` and `ResourceMetrics` use the same field numbers for
    /// `resource` and `schema_url`, so the pre-encoded fragment is valid for
    /// either message type.
    pub fn new(resource_fragment: &[u8]) -> Result<Self, Error> {
        Self::new_with_views(resource_fragment, Vec::new())
    }

    /// Creates an encoder with metric views applied during OTLP projection.
    ///
    /// Matching follows view semantics for the supported subset: optional
    /// selectors are exact matches, all matching views produce streams, and
    /// the descriptor-defined stream is emitted only when no view matches.
    /// Views must not map different source fields in one effective
    /// instrumentation scope to the same case-insensitive output name;
    /// [`Self::encode`] reports an [`Error::MetricNameCollision`] when that
    /// occurs. This includes fields supplied by separate metric-set exports
    /// whose scope names and entity attributes are equal.
    pub fn new_with_views(resource_fragment: &[u8], views: Vec<MetricView>) -> Result<Self, Error> {
        Ok(Self {
            resource_metrics: ResourceMetrics::decode(resource_fragment)?,
            views,
        })
    }

    /// Encodes a registry export batch. Empty batches produce no pdata.
    pub fn encode(&self, batch: &MetricExportBatch) -> Result<Option<OtlpProtoBytes>, Error> {
        let scope_metrics = if batch
            .metric_sets
            .iter()
            .any(|metric_set| metric_set.identity_may_repeat)
        {
            let metric_sets = coalesce_metric_sets(&batch.metric_sets)?;
            self.encode_metric_sets(
                metric_sets.iter().map(CoalescedMetricSet::as_metric_set),
                metric_sets.len(),
                batch.time_unix_nano,
            )?
        } else {
            self.encode_metric_sets(
                batch.metric_sets.iter(),
                batch.metric_sets.len(),
                batch.time_unix_nano,
            )?
        };

        if scope_metrics.is_empty() {
            return Ok(None);
        }

        let mut resource_metrics = self.resource_metrics.clone();
        resource_metrics.scope_metrics = scope_metrics;
        let request = ExportMetricsServiceRequest::new(vec![resource_metrics]);
        Ok(Some(OtlpProtoBytes::ExportMetricsRequest(Bytes::from(
            request.encode_to_vec(),
        ))))
    }

    fn encode_metric_sets<'a>(
        &self,
        metric_sets: impl Iterator<Item = &'a MetricSetExport>,
        metric_set_count: usize,
        time_unix_nano: u64,
    ) -> Result<Vec<ScopeMetrics>, Error> {
        let mut scope_metrics = Vec::with_capacity(metric_set_count);
        let mut scope_identities = HashMap::with_capacity(metric_set_count);
        if self.views.is_empty() {
            for metric_set in metric_sets {
                if let Some(scope) = encode_metric_set_without_views(metric_set, time_unix_nano)? {
                    append_scope_metrics(
                        &mut scope_metrics,
                        &mut scope_identities,
                        metric_set,
                        scope,
                    );
                }
            }
        } else {
            let mut collisions = CollisionIndex::with_capacity(metric_set_count);
            for metric_set in metric_sets {
                if let Some(scope) = encode_metric_set_with_views(
                    metric_set,
                    time_unix_nano,
                    &self.views,
                    &mut collisions,
                )? {
                    append_scope_metrics(
                        &mut scope_metrics,
                        &mut scope_identities,
                        metric_set,
                        scope,
                    );
                }
            }
        }
        Ok(scope_metrics)
    }
}

/// Combines bucket-local points into the same metric streams without merging
/// their values. Distinct data-point attributes therefore remain distinct OTLP
/// points, while independently registered producers with the same attributes
/// have already been numerically coalesced above this layer.
fn append_scope_metrics(
    scope_metrics: &mut Vec<ScopeMetrics>,
    identities: &mut HashMap<(usize, usize), usize>,
    metric_set: &MetricSetExport,
    incoming: ScopeMetrics,
) {
    let identity = (
        std::ptr::from_ref(metric_set.descriptor) as usize,
        Arc::as_ptr(&metric_set.attributes) as usize,
    );
    if let Some(index) = identities.get(&identity).copied() {
        merge_scope_metric_points(&mut scope_metrics[index], incoming);
    } else {
        let index = scope_metrics.len();
        let _ = identities.insert(identity, index);
        scope_metrics.push(incoming);
    }
}

fn merge_scope_metric_points(target: &mut ScopeMetrics, incoming: ScopeMetrics) {
    for incoming_metric in incoming.metrics {
        let target_metric = target.metrics.iter_mut().find(|target_metric| {
            target_metric.name == incoming_metric.name
                && target_metric.description == incoming_metric.description
                && target_metric.unit == incoming_metric.unit
                && metric_data_compatible(target_metric, &incoming_metric)
        });
        if let Some(target_metric) = target_metric {
            append_metric_points(target_metric, incoming_metric);
        } else {
            target.metrics.push(incoming_metric);
        }
    }
}

fn metric_data_compatible(left: &Metric, right: &Metric) -> bool {
    match (left.data.as_ref(), right.data.as_ref()) {
        (Some(metric::Data::Gauge(_)), Some(metric::Data::Gauge(_))) => true,
        (Some(metric::Data::Sum(left)), Some(metric::Data::Sum(right))) => {
            left.aggregation_temporality == right.aggregation_temporality
                && left.is_monotonic == right.is_monotonic
        }
        (Some(metric::Data::Histogram(left)), Some(metric::Data::Histogram(right))) => {
            left.aggregation_temporality == right.aggregation_temporality
        }
        (
            Some(metric::Data::ExponentialHistogram(left)),
            Some(metric::Data::ExponentialHistogram(right)),
        ) => left.aggregation_temporality == right.aggregation_temporality,
        _ => false,
    }
}

fn append_metric_points(target: &mut Metric, incoming: Metric) {
    match (target.data.as_mut(), incoming.data) {
        (Some(metric::Data::Gauge(target)), Some(metric::Data::Gauge(mut incoming))) => {
            target.data_points.append(&mut incoming.data_points);
        }
        (Some(metric::Data::Sum(target)), Some(metric::Data::Sum(mut incoming))) => {
            target.data_points.append(&mut incoming.data_points);
        }
        (Some(metric::Data::Histogram(target)), Some(metric::Data::Histogram(mut incoming))) => {
            target.data_points.append(&mut incoming.data_points);
        }
        (
            Some(metric::Data::ExponentialHistogram(target)),
            Some(metric::Data::ExponentialHistogram(mut incoming)),
        ) => {
            target.data_points.append(&mut incoming.data_points);
        }
        _ => unreachable!("metric stream compatibility was checked before merging"),
    }
}

/// Coalesces independently registered keys that map to the same OTLP scope.
///
/// Separate keys represent separate producers. Their sum-like values and
/// histograms therefore contribute to the same aggregate regardless of
/// temporality, while gauges use deterministic last-value semantics matching
/// registry iteration order.
fn coalesce_metric_sets(
    metric_sets: &[MetricSetExport],
) -> Result<Vec<CoalescedMetricSet<'_>>, Error> {
    if metric_sets.len() <= 1 {
        if let Some(metric_set) = metric_sets.first() {
            validate_metric_set(metric_set)?;
        }
        return Ok(metric_sets
            .first()
            .map(CoalescedMetricSet::Borrowed)
            .into_iter()
            .collect());
    }

    let mut identities = HashMap::with_capacity(metric_sets.len());
    let mut coalesced = Vec::with_capacity(metric_sets.len());
    for metric_set in metric_sets {
        validate_metric_set(metric_set)?;
        let identity = MetricSetIdentity {
            descriptor: std::ptr::from_ref(metric_set.descriptor) as usize,
            // Registry entity attributes are interned, so equal attribute
            // sets in a registry batch share this allocation.
            attributes: Arc::as_ptr(&metric_set.attributes) as usize,
            datapoint_attributes: &metric_set.item_attributes,
        };
        if let Some(&index) = identities.get(&identity) {
            let target = &mut coalesced[index];
            if let CoalescedMetricSet::Borrowed(original) = target {
                *target = CoalescedMetricSet::Owned((*original).clone());
            }
            let CoalescedMetricSet::Owned(target) = target else {
                unreachable!("duplicate metric-set identity must be owned before merging")
            };
            merge_metric_set(target, metric_set);
        } else {
            let index = coalesced.len();
            let _ = identities.insert(identity, index);
            coalesced.push(CoalescedMetricSet::Borrowed(metric_set));
        }
    }
    Ok(coalesced)
}

fn validate_metric_set(metric_set: &MetricSetExport) -> Result<(), Error> {
    validate_value_count(metric_set)?;
    for (field, value) in metric_set.descriptor.metrics.iter().zip(&metric_set.values) {
        validate_value_kind(field, value)?;
    }
    Ok(())
}

fn merge_metric_set(target: &mut MetricSetExport, incoming: &MetricSetExport) {
    target.delta_start_time_unix_nano = target
        .delta_start_time_unix_nano
        .min(incoming.delta_start_time_unix_nano);
    target.cumulative_start_time_unix_nano = target
        .cumulative_start_time_unix_nano
        .min(incoming.cumulative_start_time_unix_nano);

    for ((field, current), incoming) in target
        .descriptor
        .metrics
        .iter()
        .zip(&mut target.values)
        .zip(&incoming.values)
    {
        match field.instrument {
            Instrument::Gauge => *current = incoming.clone(),
            Instrument::Counter
            | Instrument::UpDownCounter
            | Instrument::Histogram
            | Instrument::Mmsc
            | Instrument::ExponentialHistogram => current.add_in_place(incoming),
        }
    }
}

/// Expands one metric set without paying any view-resolution bookkeeping.
fn encode_metric_set_without_views(
    metric_set: &MetricSetExport,
    time_unix_nano: u64,
) -> Result<Option<ScopeMetrics>, Error> {
    validate_value_count(metric_set)?;

    let mut metrics = Vec::with_capacity(metric_set.values.len());
    let datapoint_attributes = encode_datapoint_attributes(metric_set);
    for (field, value) in metric_set.descriptor.metrics.iter().zip(&metric_set.values) {
        validate_value_kind(field, value)?;
        if let Some(metric) = encode_metric(
            field,
            value,
            metric_set,
            time_unix_nano,
            field.name,
            field.brief,
            &datapoint_attributes,
        )? {
            metrics.push(metric);
        }
    }

    if metrics.is_empty() {
        return Ok(None);
    }
    Ok(Some(build_scope_metrics(metric_set, metrics)))
}

/// Expands one metric set after resolving views and checking stream collisions.
fn encode_metric_set_with_views<'a>(
    metric_set: &MetricSetExport,
    time_unix_nano: u64,
    views: &'a [MetricView],
    collisions: &mut CollisionIndex<'a>,
) -> Result<Option<ScopeMetrics>, Error> {
    validate_value_count(metric_set)?;

    let mut metrics = Vec::with_capacity(metric_set.values.len());
    let datapoint_attributes = encode_datapoint_attributes(metric_set);
    // Scope selectors are invariant across all fields in this metric set, so
    // evaluate them once before resolving the per-instrument selectors.
    let scope_views = views
        .iter()
        .filter(|view| view_matches_scope(view, metric_set))
        .collect::<SmallVec<[&MetricView; 4]>>();
    let scope_streams = collisions
        .entry(ScopeIdentity {
            name: metric_set.descriptor.name,
            attributes: metric_set.attributes.clone(),
        })
        .or_insert_with(|| HashMap::with_capacity(metric_set.values.len()));
    for (field, value) in metric_set.descriptor.metrics.iter().zip(&metric_set.values) {
        validate_value_kind(field, value)?;
        for stream in resolve_views(field, &scope_views) {
            if let Some(metric) = encode_metric(
                field,
                value,
                metric_set,
                time_unix_nano,
                stream.name,
                stream.description,
                &datapoint_attributes,
            )? {
                register_projected_stream(
                    scope_streams,
                    metric_set.descriptor.name,
                    field,
                    stream,
                )?;
                metrics.push(metric);
            }
        }
    }

    if metrics.is_empty() {
        return Ok(None);
    }
    Ok(Some(build_scope_metrics(metric_set, metrics)))
}

/// Matches the dimensions that are common to every field in a metric set.
fn view_matches_scope(view: &MetricView, metric_set: &MetricSetExport) -> bool {
    view.selector
        .scope_name
        .as_deref()
        .is_none_or(|selector| selector == metric_set.descriptor.name)
        && view
            .selector
            .scope_attributes
            .iter()
            .all(|(expected_key, expected_value)| {
                metric_set
                    .attributes
                    .iter_attributes()
                    .any(|(actual_key, actual_value)| {
                        actual_key == expected_key
                            && scope_attribute_value_matches(expected_value, actual_value)
                    })
            })
}

/// Compares one configured scalar value with its internal metric-set value.
fn scope_attribute_value_matches(expected: &ConfigAttributeValue, actual: &AttributeValue) -> bool {
    match (expected, actual) {
        (ConfigAttributeValue::String(expected), AttributeValue::String(actual)) => {
            expected == actual
        }
        (ConfigAttributeValue::Bool(expected), AttributeValue::Boolean(actual)) => {
            expected == actual
        }
        (ConfigAttributeValue::I64(expected), AttributeValue::Int(actual)) => expected == actual,
        (ConfigAttributeValue::I64(expected), AttributeValue::UInt(actual)) => {
            u64::try_from(*expected).is_ok_and(|expected| expected == *actual)
        }
        (ConfigAttributeValue::F64(expected), AttributeValue::Double(actual)) => expected == actual,
        // Array selectors are rejected by the receiver configuration. Maps
        // also have no scalar configuration representation.
        _ => false,
    }
}

fn validate_value_count(metric_set: &MetricSetExport) -> Result<(), Error> {
    let expected = metric_set.descriptor.metrics.len();
    let actual = metric_set.values.len();
    if expected == actual {
        Ok(())
    } else {
        Err(Error::ValueCountMismatch {
            metric_set: metric_set.descriptor.name,
            expected,
            actual,
        })
    }
}

fn build_scope_metrics(metric_set: &MetricSetExport, metrics: Vec<Metric>) -> ScopeMetrics {
    let attributes: Vec<KeyValue> = metric_set
        .attributes
        .iter_attributes()
        .map(|(key, value)| KeyValue::new(key, encode_attribute_value(value)))
        .collect();
    let scope = InstrumentationScope::build()
        .name(metric_set.descriptor.name)
        .attributes(attributes)
        .finish();
    ScopeMetrics::new(scope, metrics)
}

fn encode_datapoint_attributes(metric_set: &MetricSetExport) -> Vec<KeyValue> {
    metric_set
        .item_attributes
        .iter()
        .map(|(key, value)| KeyValue::new(key.clone(), AnyValue::new_string(value.clone())))
        .collect()
}

/// Adds one stream to the collision index for its effective scope.
fn register_projected_stream<'a>(
    scope_streams: &mut ScopeStreams<'a>,
    scope_name: &'static str,
    source_field: &'static MetricsField,
    stream: ResolvedStream<'a>,
) -> Result<(), Error> {
    let sources = scope_streams
        .entry(CaseInsensitiveName(stream.name))
        .or_default();

    if let Some(previous) = sources.iter().find(|previous| {
        !std::ptr::eq(previous.source_field, source_field)
            && (previous.view_applied || stream.view_applied)
    }) {
        return Err(Error::MetricNameCollision {
            scope_name: scope_name.to_owned(),
            first_metric: previous.source_field.name,
            first_name: previous.output_name.to_owned(),
            second_metric: source_field.name,
            second_name: stream.name.to_owned(),
        });
    }

    sources.push(ProjectedStream {
        source_field,
        output_name: stream.name,
        view_applied: stream.view_applied,
    });
    Ok(())
}

/// Resolves and deduplicates lightweight metadata before protobuf construction.
fn resolve_views<'a>(
    field: &'static MetricsField,
    scope_views: &[&'a MetricView],
) -> SmallVec<[ResolvedStream<'a>; 1]> {
    let mut matched = false;
    let mut streams: SmallVec<[ResolvedStream<'a>; 1]> = SmallVec::new();

    for view in scope_views.iter().copied().filter(|view| {
        view.selector
            .instrument_name
            .as_deref()
            .is_none_or(|selector| selector == field.name)
    }) {
        matched = true;
        let stream = ResolvedStream {
            name: view.stream.name.as_deref().unwrap_or(field.name),
            description: view.stream.description.as_deref().unwrap_or(field.brief),
            view_applied: true,
        };
        if let Some(existing) = streams
            .iter_mut()
            .find(|existing| existing.name.eq_ignore_ascii_case(stream.name))
        {
            if stream.description.len() > existing.description.len() {
                existing.description = stream.description;
            }
        } else {
            streams.push(stream);
        }
    }

    if !matched {
        streams.push(ResolvedStream {
            name: field.name,
            description: field.brief,
            view_applied: false,
        });
    }
    streams
}

/// Projects one multivariate metric field into its univariate OTLP data type.
fn encode_metric(
    field: &'static MetricsField,
    value: &MetricValue,
    metric_set: &MetricSetExport,
    time_unix_nano: u64,
    name: &str,
    description: &str,
    datapoint_attributes: &[KeyValue],
) -> Result<Option<Metric>, Error> {
    let data = match field.instrument {
        Instrument::Counter | Instrument::UpDownCounter => {
            let temporality = field
                .temporality
                .ok_or(Error::MissingTemporality { metric: field.name })?;
            let start_time = match temporality {
                Temporality::Delta => metric_set.delta_start_time_unix_nano,
                Temporality::Cumulative => metric_set.cumulative_start_time_unix_nano,
            };
            let point = number_data_point(value, start_time, time_unix_nano, datapoint_attributes);
            metric::Data::Sum(Sum::new(
                encode_temporality(temporality),
                matches!(field.instrument, Instrument::Counter),
                vec![point],
            ))
        }
        Instrument::Gauge => {
            let point = number_data_point(value, 0, time_unix_nano, datapoint_attributes);
            metric::Data::Gauge(Gauge::new(vec![point]))
        }
        Instrument::Histogram => {
            let point = scalar_histogram_data_point(
                value,
                metric_set.delta_start_time_unix_nano,
                time_unix_nano,
                datapoint_attributes,
            );
            metric::Data::Histogram(Histogram::new(AggregationTemporality::Delta, vec![point]))
        }
        Instrument::Mmsc => {
            let MetricValue::Mmsc(snapshot) = value else {
                unreachable!("metric value kind was validated before encoding")
            };
            if snapshot.count == 0 {
                return Ok(None);
            }
            let point = crate::metrics::exphist::mmsc_exponential_histogram_data_point(
                snapshot,
                metric_set.delta_start_time_unix_nano,
                time_unix_nano,
                datapoint_attributes,
            );
            metric::Data::ExponentialHistogram(ExponentialHistogram::new(
                AggregationTemporality::Delta,
                vec![point],
            ))
        }
        Instrument::ExponentialHistogram => {
            let MetricValue::Distribution(distribution) = value else {
                unreachable!("metric value kind was validated before encoding")
            };
            let DistributionValue::Live(live) = distribution.as_ref() else {
                unreachable!("registry distributions are always live aggregations")
            };
            if live.is_empty() {
                return Ok(None);
            }
            let point = crate::metrics::exphist::distribution_exponential_histogram_data_point(
                live,
                metric_set.delta_start_time_unix_nano,
                time_unix_nano,
                datapoint_attributes,
            );
            metric::Data::ExponentialHistogram(ExponentialHistogram::new(
                AggregationTemporality::Delta,
                vec![point],
            ))
        }
    };

    Ok(Some(Metric {
        name: name.to_owned(),
        description: description.to_owned(),
        unit: field.unit.to_owned(),
        metadata: Vec::new(),
        data: Some(data),
    }))
}

/// Validates the descriptor/value pairing before any lossy projection occurs.
fn validate_value_kind(field: &MetricsField, value: &MetricValue) -> Result<(), Error> {
    let expected = match field.instrument {
        Instrument::Mmsc => "mmsc",
        Instrument::ExponentialHistogram => "distribution",
        _ => match field.value_type {
            crate::descriptor::MetricValueType::U64 => "u64",
            crate::descriptor::MetricValueType::F64 => "f64",
        },
    };
    let actual = match value {
        MetricValue::U64(_) => "u64",
        MetricValue::F64(_) => "f64",
        MetricValue::Mmsc(_) => "mmsc",
        MetricValue::Distribution(_) => "distribution",
    };

    if expected == actual {
        Ok(())
    } else {
        Err(Error::ValueKindMismatch {
            metric: field.name,
            expected,
            actual,
        })
    }
}

/// Creates a scalar OTLP point, saturating unsigned values to OTLP's signed range.
fn number_data_point(
    value: &MetricValue,
    start_time_unix_nano: u64,
    time_unix_nano: u64,
    datapoint_attributes: &[KeyValue],
) -> NumberDataPoint {
    let builder = NumberDataPoint::build()
        .attributes(datapoint_attributes.to_vec())
        .start_time_unix_nano(start_time_unix_nano)
        .time_unix_nano(time_unix_nano);
    match value {
        MetricValue::U64(value) => builder.value_int(saturating_i64(*value)).finish(),
        MetricValue::F64(value) => builder.value_double(*value).finish(),
        MetricValue::Mmsc(_) | MetricValue::Distribution(_) => {
            unreachable!("metric value kind was validated before encoding")
        }
    }
}

/// Encodes the legacy scalar histogram form as exactly one observation.
///
/// Scalar histograms are not pre-aggregated. These stable explicit bounds keep
/// their output shape consistent across native ITS releases.
/// [`Instrument::Mmsc`] follows a separate, bucketless path above.
fn scalar_histogram_data_point(
    value: &MetricValue,
    start_time_unix_nano: u64,
    time_unix_nano: u64,
    datapoint_attributes: &[KeyValue],
) -> HistogramDataPoint {
    const DEFAULT_BOUNDS: [f64; 15] = [
        0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 2500.0, 5000.0,
        7500.0, 10000.0,
    ];

    let value = match value {
        MetricValue::U64(value) => *value as f64,
        MetricValue::F64(value) => *value,
        MetricValue::Mmsc(_) | MetricValue::Distribution(_) => {
            unreachable!("metric value kind was validated before encoding")
        }
    };
    let bucket = DEFAULT_BOUNDS
        .iter()
        .position(|bound| value <= *bound)
        .unwrap_or(DEFAULT_BOUNDS.len());
    let mut bucket_counts = vec![0; DEFAULT_BOUNDS.len() + 1];
    bucket_counts[bucket] = 1;

    let mut point = HistogramDataPoint::build()
        .attributes(datapoint_attributes.to_vec())
        .start_time_unix_nano(start_time_unix_nano)
        .time_unix_nano(time_unix_nano)
        .count(1u64)
        .min(value)
        .max(value)
        .bucket_counts(bucket_counts)
        .explicit_bounds(DEFAULT_BOUNDS.to_vec());
    if value >= 0.0 {
        point = point.sum(value);
    }
    point.finish()
}

const fn encode_temporality(temporality: Temporality) -> AggregationTemporality {
    match temporality {
        Temporality::Delta => AggregationTemporality::Delta,
        Temporality::Cumulative => AggregationTemporality::Cumulative,
    }
}

/// Preserves internal attribute types in their corresponding OTLP value forms.
fn encode_attribute_value(value: &AttributeValue) -> AnyValue {
    match value {
        AttributeValue::String(value) => AnyValue::new_string(value.clone()),
        AttributeValue::Int(value) => AnyValue::new_int(*value),
        AttributeValue::UInt(value) => AnyValue::new_int(saturating_i64(*value)),
        AttributeValue::Double(value) => AnyValue::new_double(*value),
        AttributeValue::Boolean(value) => AnyValue::new_bool(*value),
        AttributeValue::Map(values) => AnyValue::new_kvlist(
            values
                .iter()
                .map(|(key, value)| KeyValue::new(key, encode_attribute_value(value)))
                .collect::<Vec<_>>(),
        ),
    }
}

/// Converts an unsigned internal value without wrapping OTLP's signed integer.
const fn saturating_i64(value: u64) -> i64 {
    if value > i64::MAX as u64 {
        i64::MAX
    } else {
        value as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::AttributeSetHandler;
    use crate::descriptor::{
        AttributeField, AttributeValueType, AttributesDescriptor, MetricValueType,
        MetricsDescriptor,
    };
    use crate::entity::{EntityAttributeSet, EntityRegistry};
    use crate::instrument::MmscSnapshot;
    use otap_df_pdata::proto::opentelemetry::common::v1::{KeyValueList, any_value};
    use otap_df_pdata::proto::opentelemetry::logs::v1::ResourceLogs;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{metric, number_data_point};
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::views::otap::OtapMetricsView;
    use otap_df_pdata::{OtapArrowRecords, OtapPayload, TryIntoWithOptions};
    use otap_df_pdata_views::views::common::{
        AnyValueView, AttributeView, InstrumentationScopeView,
    };
    use otap_df_pdata_views::views::metrics::{
        DataView, MetricView as PdataMetricView, MetricsView, NumberDataPointView,
        ResourceMetricsView, ScopeMetricsView, SumView, Value,
    };
    use otap_df_pdata_views::views::resource::ResourceView;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    const DELTA_START: u64 = 10;
    const CUMULATIVE_START: u64 = 5;
    const COLLECTION_TIME: u64 = 20;

    static ALL_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test.scope",
        metrics: &[
            MetricsField {
                name: "counter.delta",
                unit: "{request}",
                brief: "Delta counter",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::U64,
            },
            MetricsField {
                name: "counter.cumulative",
                unit: "By",
                brief: "Cumulative counter",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Cumulative),
                value_type: MetricValueType::U64,
            },
            MetricsField {
                name: "up_down.delta",
                unit: "1",
                brief: "Delta up/down counter",
                instrument: Instrument::UpDownCounter,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::F64,
            },
            MetricsField {
                name: "gauge",
                unit: "Cel",
                brief: "Current gauge",
                instrument: Instrument::Gauge,
                temporality: None,
                value_type: MetricValueType::F64,
            },
            MetricsField {
                name: "histogram.scalar",
                unit: "ms",
                brief: "Scalar histogram",
                instrument: Instrument::Histogram,
                temporality: None,
                value_type: MetricValueType::F64,
            },
            MetricsField {
                name: "histogram.mmsc",
                unit: "ms",
                brief: "Pre-aggregated histogram",
                instrument: Instrument::Mmsc,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::F64,
            },
        ],
    };

    static MMSC_ONLY_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test.empty_mmsc",
        metrics: &[MetricsField {
            name: "histogram.empty",
            unit: "ms",
            brief: "Empty pre-aggregated histogram",
            instrument: Instrument::Mmsc,
            temporality: Some(Temporality::Delta),
            value_type: MetricValueType::F64,
        }],
    };

    static DISTRIBUTION_ONLY_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test.distribution",
        metrics: &[MetricsField {
            name: "histogram.distribution",
            unit: "ms",
            brief: "Exponential-histogram distribution",
            instrument: Instrument::ExponentialHistogram,
            temporality: Some(Temporality::Delta),
            value_type: MetricValueType::F64,
        }],
    };

    static INVALID_SUM_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test.invalid",
        metrics: &[MetricsField {
            name: "invalid.sum",
            unit: "1",
            brief: "Sum without temporality",
            instrument: Instrument::Counter,
            temporality: None,
            value_type: MetricValueType::U64,
        }],
    };

    static F64_GAUGE_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test.f64_gauge",
        metrics: &[MetricsField {
            name: "gauge.f64",
            unit: "1",
            brief: "Floating-point gauge",
            instrument: Instrument::Gauge,
            temporality: None,
            value_type: MetricValueType::F64,
        }],
    };

    static TWO_FIELD_VIEW_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test.view_collision",
        metrics: &[
            MetricsField {
                name: "gauge.first",
                unit: "1",
                brief: "First gauge",
                instrument: Instrument::Gauge,
                temporality: None,
                value_type: MetricValueType::F64,
            },
            MetricsField {
                name: "gauge.second",
                unit: "1",
                brief: "Second gauge",
                instrument: Instrument::Gauge,
                temporality: None,
                value_type: MetricValueType::F64,
            },
        ],
    };

    static NO_VIEW_COLLISION_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test.no_view_collision",
        metrics: &[
            MetricsField {
                name: "existing.name",
                unit: "1",
                brief: "First pre-existing stream",
                instrument: Instrument::Gauge,
                temporality: None,
                value_type: MetricValueType::F64,
            },
            MetricsField {
                name: "EXISTING.NAME",
                unit: "1",
                brief: "Second pre-existing stream",
                instrument: Instrument::Gauge,
                temporality: None,
                value_type: MetricValueType::F64,
            },
        ],
    };

    static SHARED_SCOPE_FIRST_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test.shared_view_collision",
        metrics: &[MetricsField {
            name: "gauge.shared_first",
            unit: "1",
            brief: "First shared-scope gauge",
            instrument: Instrument::Gauge,
            temporality: None,
            value_type: MetricValueType::F64,
        }],
    };

    static SHARED_SCOPE_SECOND_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test.shared_view_collision",
        metrics: &[MetricsField {
            name: "gauge.shared_second",
            unit: "1",
            brief: "Second shared-scope gauge",
            instrument: Instrument::Gauge,
            temporality: None,
            value_type: MetricValueType::F64,
        }],
    };

    static U64_HISTOGRAM_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test.u64_histogram",
        metrics: &[MetricsField {
            name: "histogram.u64",
            unit: "By",
            brief: "Unsigned histogram",
            instrument: Instrument::Histogram,
            temporality: Some(Temporality::Delta),
            value_type: MetricValueType::U64,
        }],
    };

    static EMPTY_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "test.empty_attributes",
        fields: &[],
    };

    static FULL_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "test.full_attributes",
        fields: &[
            AttributeField {
                key: "worker.name",
                brief: "Worker name",
                r#type: AttributeValueType::String,
            },
            AttributeField {
                key: "worker.delta",
                brief: "Signed delta",
                r#type: AttributeValueType::Int,
            },
            AttributeField {
                key: "worker.sequence",
                brief: "Unsigned sequence",
                r#type: AttributeValueType::Int,
            },
            AttributeField {
                key: "worker.load",
                brief: "Worker load",
                r#type: AttributeValueType::Double,
            },
            AttributeField {
                key: "worker.ready",
                brief: "Readiness",
                r#type: AttributeValueType::Boolean,
            },
            AttributeField {
                key: "worker.labels",
                brief: "Labels",
                r#type: AttributeValueType::Map,
            },
        ],
    };

    #[derive(Debug)]
    struct TestAttributeSet {
        descriptor: &'static AttributesDescriptor,
        values: Vec<AttributeValue>,
    }

    impl AttributeSetHandler for TestAttributeSet {
        fn descriptor(&self) -> &'static AttributesDescriptor {
            self.descriptor
        }

        fn attribute_values(&self) -> &[AttributeValue] {
            &self.values
        }
    }

    fn shared_attributes(
        descriptor: &'static AttributesDescriptor,
        values: Vec<AttributeValue>,
    ) -> Arc<EntityAttributeSet> {
        let mut entities = EntityRegistry::default();
        let key = entities
            .register(TestAttributeSet { descriptor, values })
            .key();
        entities.get_shared(key).expect("registered entity")
    }

    fn empty_attributes() -> Arc<EntityAttributeSet> {
        shared_attributes(&EMPTY_ATTRIBUTES_DESCRIPTOR, Vec::new())
    }

    fn metric_set(
        descriptor: &'static MetricsDescriptor,
        attributes: Arc<EntityAttributeSet>,
        values: Vec<MetricValue>,
    ) -> MetricSetExport {
        MetricSetExport {
            descriptor,
            attributes,
            item_attributes: Vec::new(),
            values,
            delta_start_time_unix_nano: DELTA_START,
            cumulative_start_time_unix_nano: CUMULATIVE_START,
            identity_may_repeat: true,
        }
    }

    fn empty_resource_encoder() -> MetricsOtlpEncoder {
        MetricsOtlpEncoder::new(&ResourceLogs::default().encode_to_vec())
            .expect("valid resource fragment")
    }

    fn decode_request(encoded: OtlpProtoBytes) -> ExportMetricsServiceRequest {
        let OtlpProtoBytes::ExportMetricsRequest(bytes) = encoded else {
            panic!("encoder returned the wrong OTLP signal")
        };
        ExportMetricsServiceRequest::decode(bytes).expect("valid metrics request")
    }

    fn only_scope(request: &ExportMetricsServiceRequest) -> &ScopeMetrics {
        let [resource_metrics] = request.resource_metrics.as_slice() else {
            panic!("expected one resource metrics message")
        };
        let [scope_metrics] = resource_metrics.scope_metrics.as_slice() else {
            panic!("expected one scope metrics message")
        };
        scope_metrics
    }

    fn metric_named<'a>(scope: &'a ScopeMetrics, name: &str) -> &'a Metric {
        scope
            .metrics
            .iter()
            .find(|metric| metric.name == name)
            .unwrap_or_else(|| panic!("missing metric {name}"))
    }

    fn number_point(metric: &Metric) -> (&NumberDataPoint, &Sum) {
        let Some(metric::Data::Sum(sum)) = metric.data.as_ref() else {
            panic!("expected sum metric")
        };
        let [point] = sum.data_points.as_slice() else {
            panic!("expected one number data point")
        };
        (point, sum)
    }

    #[test]
    fn unmatched_views_preserve_the_descriptor_defined_stream() {
        let views = vec![MetricView {
            selector: MetricViewSelector {
                scope_name: Some("another.scope".to_owned()),
                scope_attributes: HashMap::new(),
                instrument_name: Some("gauge.f64".to_owned()),
            },
            stream: MetricViewStream {
                name: Some("renamed.gauge".to_owned()),
                description: Some("A renamed gauge".to_owned()),
            },
        }];
        let encoder =
            MetricsOtlpEncoder::new_with_views(&ResourceLogs::default().encode_to_vec(), views)
                .expect("valid resource fragment");
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &F64_GAUGE_DESCRIPTOR,
                empty_attributes(),
                vec![MetricValue::F64(3.5)],
            )],
        };

        let request = decode_request(
            encoder
                .encode(&batch)
                .expect("encode succeeds")
                .expect("non-empty request"),
        );
        let scope = only_scope(&request);
        let [metric] = scope.metrics.as_slice() else {
            panic!("expected one default metric stream")
        };
        assert_eq!(metric.name, "gauge.f64");
        assert_eq!(metric.description, "Floating-point gauge");
        assert_eq!(metric.unit, "1");
    }

    #[test]
    fn scope_attribute_selectors_require_all_exact_scalar_matches() {
        let attributes = shared_attributes(
            &FULL_ATTRIBUTES_DESCRIPTOR,
            vec![
                AttributeValue::String("worker-a".to_owned()),
                AttributeValue::Int(-2),
                AttributeValue::UInt(7),
                AttributeValue::Double(0.75),
                AttributeValue::Boolean(true),
                AttributeValue::Map(BTreeMap::from([(
                    "region".to_owned(),
                    AttributeValue::String("west".to_owned()),
                )])),
            ],
        );
        let matching_scope_attributes = HashMap::from([
            (
                "worker.name".to_owned(),
                ConfigAttributeValue::String("worker-a".to_owned()),
            ),
            ("worker.delta".to_owned(), ConfigAttributeValue::I64(-2)),
            ("worker.sequence".to_owned(), ConfigAttributeValue::I64(7)),
            ("worker.load".to_owned(), ConfigAttributeValue::F64(0.75)),
            ("worker.ready".to_owned(), ConfigAttributeValue::Bool(true)),
        ]);
        let view_for = |scope_attributes| MetricView {
            selector: MetricViewSelector {
                scope_name: Some("test.f64_gauge".to_owned()),
                scope_attributes,
                instrument_name: Some("gauge.f64".to_owned()),
            },
            stream: MetricViewStream {
                name: Some("viewed.gauge".to_owned()),
                description: None,
            },
        };
        let encode_with = |scope_attributes| {
            let encoder = MetricsOtlpEncoder::new_with_views(
                &ResourceLogs::default().encode_to_vec(),
                vec![view_for(scope_attributes)],
            )
            .expect("valid resource fragment");
            let batch = MetricExportBatch {
                time_unix_nano: COLLECTION_TIME,
                metric_sets: vec![metric_set(
                    &F64_GAUGE_DESCRIPTOR,
                    attributes.clone(),
                    vec![MetricValue::F64(3.5)],
                )],
            };
            decode_request(
                encoder
                    .encode(&batch)
                    .expect("encode succeeds")
                    .expect("non-empty request"),
            )
        };

        let matching_request = encode_with(matching_scope_attributes.clone());
        assert_eq!(
            only_scope(&matching_request).metrics[0].name,
            "viewed.gauge"
        );

        let mut missing_attribute = matching_scope_attributes.clone();
        let _ = missing_attribute.insert(
            "worker.missing".to_owned(),
            ConfigAttributeValue::String("absent".to_owned()),
        );
        let mut wrong_value = matching_scope_attributes.clone();
        let _ = wrong_value.insert(
            "worker.name".to_owned(),
            ConfigAttributeValue::String("worker-b".to_owned()),
        );
        let mut wrong_type = matching_scope_attributes.clone();
        let _ = wrong_type.insert("worker.sequence".to_owned(), ConfigAttributeValue::F64(7.0));
        let mut negative_unsigned = matching_scope_attributes;
        let _ =
            negative_unsigned.insert("worker.sequence".to_owned(), ConfigAttributeValue::I64(-1));

        for scope_attributes in [
            missing_attribute,
            wrong_value,
            wrong_type,
            negative_unsigned,
        ] {
            let request = encode_with(scope_attributes);
            assert_eq!(only_scope(&request).metrics[0].name, "gauge.f64");
        }
    }

    #[test]
    fn matching_views_deduplicate_names_case_insensitively_and_keep_longest_description() {
        let renamed_stream = MetricViewStream {
            name: Some("viewed.gauge".to_owned()),
            description: None,
        };
        let views = vec![
            // A partial selector matches every instrument in this scope.
            MetricView {
                selector: MetricViewSelector {
                    scope_name: Some("test.f64_gauge".to_owned()),
                    scope_attributes: HashMap::new(),
                    instrument_name: None,
                },
                stream: renamed_stream.clone(),
            },
            // A different partial selector produces the same final stream.
            MetricView {
                selector: MetricViewSelector {
                    scope_name: None,
                    scope_attributes: HashMap::new(),
                    instrument_name: Some("gauge.f64".to_owned()),
                },
                stream: MetricViewStream {
                    name: Some("VIEWED.GAUGE".to_owned()),
                    description: None,
                },
            },
            // Omitting both selectors matches every metric, but this duplicate
            // result must still be emitted only once.
            MetricView {
                selector: MetricViewSelector::default(),
                stream: renamed_stream,
            },
            // Description is not part of OTLP stream identity. The longest
            // description is retained when otherwise identical streams differ.
            MetricView {
                selector: MetricViewSelector {
                    scope_name: Some("test.f64_gauge".to_owned()),
                    scope_attributes: HashMap::new(),
                    instrument_name: Some("gauge.f64".to_owned()),
                },
                stream: MetricViewStream {
                    name: Some("Viewed.Gauge".to_owned()),
                    description: Some("Viewed gauge description".to_owned()),
                },
            },
            // A shorter description after the longest one must not replace it.
            MetricView {
                selector: MetricViewSelector {
                    scope_name: Some("test.f64_gauge".to_owned()),
                    scope_attributes: HashMap::new(),
                    instrument_name: Some("gauge.f64".to_owned()),
                },
                stream: MetricViewStream {
                    name: Some("VIEWED.GAUGE".to_owned()),
                    description: Some("Short".to_owned()),
                },
            },
        ];
        let encoder =
            MetricsOtlpEncoder::new_with_views(&ResourceLogs::default().encode_to_vec(), views)
                .expect("valid resource fragment");
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &F64_GAUGE_DESCRIPTOR,
                empty_attributes(),
                vec![MetricValue::F64(3.5)],
            )],
        };

        let request = decode_request(
            encoder
                .encode(&batch)
                .expect("encode succeeds")
                .expect("non-empty request"),
        );
        let scope = only_scope(&request);
        assert_eq!(scope.scope.as_ref().expect("scope").name, "test.f64_gauge");
        assert_eq!(scope.metrics.len(), 1);

        let renamed = metric_named(scope, "viewed.gauge");
        assert_eq!(renamed.description, "Viewed gauge description");
        assert_eq!(renamed.unit, "1");
        assert!(renamed.metadata.is_empty());

        let Some(metric::Data::Gauge(gauge)) = renamed.data.as_ref() else {
            panic!("expected gauge data")
        };
        let [point] = gauge.data_points.as_slice() else {
            panic!("expected one gauge point")
        };
        assert_eq!(point.time_unix_nano, COLLECTION_TIME);
        assert_eq!(point.value, Some(number_data_point::Value::AsDouble(3.5)));
    }

    #[test]
    fn rejects_view_name_collisions_between_fields_in_one_metric_set() {
        let views = vec![
            MetricView {
                selector: MetricViewSelector {
                    scope_name: Some("test.view_collision".to_owned()),
                    scope_attributes: HashMap::new(),
                    instrument_name: Some("gauge.first".to_owned()),
                },
                stream: MetricViewStream {
                    name: Some("Process.Value".to_owned()),
                    description: None,
                },
            },
            MetricView {
                selector: MetricViewSelector {
                    scope_name: Some("test.view_collision".to_owned()),
                    scope_attributes: HashMap::new(),
                    instrument_name: Some("gauge.second".to_owned()),
                },
                stream: MetricViewStream {
                    name: Some("process.value".to_owned()),
                    description: None,
                },
            },
        ];
        let encoder =
            MetricsOtlpEncoder::new_with_views(&ResourceLogs::default().encode_to_vec(), views)
                .expect("valid resource fragment");
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &TWO_FIELD_VIEW_DESCRIPTOR,
                empty_attributes(),
                vec![MetricValue::F64(1.0), MetricValue::F64(2.0)],
            )],
        };

        let error = encoder
            .encode(&batch)
            .expect_err("different source fields cannot share a viewed output name");
        assert!(matches!(
            error,
            Error::MetricNameCollision {
                scope_name,
                first_metric: "gauge.first",
                first_name,
                second_metric: "gauge.second",
                second_name,
            } if scope_name == "test.view_collision"
                && first_name == "Process.Value"
                && second_name == "process.value"
        ));
    }

    #[test]
    fn rejects_view_name_collisions_across_metric_sets_with_the_same_scope_identity() {
        let views = vec![
            MetricView {
                selector: MetricViewSelector {
                    scope_name: Some("test.shared_view_collision".to_owned()),
                    scope_attributes: HashMap::new(),
                    instrument_name: Some("gauge.shared_first".to_owned()),
                },
                stream: MetricViewStream {
                    name: Some("shared.output".to_owned()),
                    description: None,
                },
            },
            MetricView {
                selector: MetricViewSelector {
                    scope_name: Some("test.shared_view_collision".to_owned()),
                    scope_attributes: HashMap::new(),
                    instrument_name: Some("gauge.shared_second".to_owned()),
                },
                stream: MetricViewStream {
                    name: Some("SHARED.OUTPUT".to_owned()),
                    description: None,
                },
            },
        ];
        let encoder =
            MetricsOtlpEncoder::new_with_views(&ResourceLogs::default().encode_to_vec(), views)
                .expect("valid resource fragment");
        let attributes = empty_attributes();
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![
                metric_set(
                    &SHARED_SCOPE_FIRST_DESCRIPTOR,
                    attributes.clone(),
                    vec![MetricValue::F64(1.0)],
                ),
                metric_set(
                    &SHARED_SCOPE_SECOND_DESCRIPTOR,
                    attributes,
                    vec![MetricValue::F64(2.0)],
                ),
            ],
        };

        let error = encoder
            .encode(&batch)
            .expect_err("equal scopes must be collision-checked across metric sets");
        assert!(matches!(
            error,
            Error::MetricNameCollision {
                scope_name,
                first_metric: "gauge.shared_first",
                second_metric: "gauge.shared_second",
                ..
            } if scope_name == "test.shared_view_collision"
        ));
    }

    #[test]
    fn preserves_preexisting_no_view_name_collision_behavior() {
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &NO_VIEW_COLLISION_DESCRIPTOR,
                empty_attributes(),
                vec![MetricValue::F64(1.0), MetricValue::F64(2.0)],
            )],
        };
        let request = decode_request(
            empty_resource_encoder()
                .encode(&batch)
                .expect("the no-view fast path must preserve existing streams")
                .expect("non-empty request"),
        );
        assert_eq!(only_scope(&request).metrics.len(), 2);
    }

    #[test]
    fn coalesces_duplicate_metric_set_identities_into_one_otlp_scope() {
        let attributes = empty_attributes();
        let first = metric_set(
            &ALL_METRICS_DESCRIPTOR,
            attributes.clone(),
            vec![
                MetricValue::U64(2),
                MetricValue::U64(10),
                MetricValue::F64(-1.0),
                MetricValue::F64(3.0),
                MetricValue::F64(4.0),
                MetricValue::Mmsc(MmscSnapshot {
                    min: 1.0,
                    max: 3.0,
                    sum: 4.0,
                    count: 2,
                }),
            ],
        );
        let mut second = metric_set(
            &ALL_METRICS_DESCRIPTOR,
            attributes,
            vec![
                MetricValue::U64(5),
                MetricValue::U64(20),
                MetricValue::F64(2.0),
                MetricValue::F64(8.0),
                MetricValue::F64(6.0),
                MetricValue::Mmsc(MmscSnapshot {
                    min: 0.0,
                    max: 5.0,
                    sum: 8.0,
                    count: 2,
                }),
            ],
        );
        second.delta_start_time_unix_nano = 8;
        second.cumulative_start_time_unix_nano = 3;
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![first, second],
        };

        let request = decode_request(
            empty_resource_encoder()
                .encode(&batch)
                .expect("duplicate identities should be coalesced")
                .expect("coalesced metrics should produce a request"),
        );
        let scope = only_scope(&request);

        let (delta, _) = number_point(metric_named(scope, "counter.delta"));
        assert_eq!(delta.value, Some(number_data_point::Value::AsInt(7)));
        assert_eq!(delta.start_time_unix_nano, 8);

        let (cumulative, _) = number_point(metric_named(scope, "counter.cumulative"));
        assert_eq!(cumulative.value, Some(number_data_point::Value::AsInt(30)));
        assert_eq!(cumulative.start_time_unix_nano, 3);

        let gauge = metric_named(scope, "gauge");
        let Some(metric::Data::Gauge(gauge)) = gauge.data.as_ref() else {
            panic!("expected gauge metric")
        };
        assert_eq!(
            gauge.data_points[0].value,
            Some(number_data_point::Value::AsDouble(8.0))
        );

        let mmsc = metric_named(scope, "histogram.mmsc");
        let Some(metric::Data::ExponentialHistogram(histogram)) = mmsc.data.as_ref() else {
            panic!("expected exponential histogram metric")
        };
        assert_eq!(histogram.data_points[0].min, Some(0.0));
        assert_eq!(histogram.data_points[0].max, Some(5.0));
        assert_eq!(histogram.data_points[0].sum, Some(12.0));
        assert_eq!(histogram.data_points[0].count, 4);
    }

    // Scenario: A delta exponential-histogram distribution field is recorded,
    // aggregated across two snapshots, then encoded to OTLP.
    // Guarantees: The merged distribution exports as a single delta
    // ExponentialHistogram data point whose count/sum/min/max reflect every
    // recorded observation and whose delta start time is preserved.
    #[test]
    fn encodes_distribution_as_delta_exponential_histogram_point() {
        use crate::instrument::{Distribution, DistributionValue};

        let attributes = empty_attributes();
        let mut first_dist = Distribution::normal();
        for value in [1.0_f64, 2.0, 4.0] {
            first_dist.record(value);
        }
        let first = metric_set(
            &DISTRIBUTION_ONLY_DESCRIPTOR,
            attributes.clone(),
            vec![MetricValue::Distribution(Box::new(
                DistributionValue::Live(first_dist),
            ))],
        );

        let mut second_dist = Distribution::normal();
        second_dist.record(8.0);
        let second = metric_set(
            &DISTRIBUTION_ONLY_DESCRIPTOR,
            attributes,
            vec![MetricValue::Distribution(Box::new(
                DistributionValue::Live(second_dist),
            ))],
        );

        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![first, second],
        };

        let request = decode_request(
            empty_resource_encoder()
                .encode(&batch)
                .expect("distribution batch should encode")
                .expect("distribution batch should produce a request"),
        );
        let scope = only_scope(&request);
        let metric = metric_named(scope, "histogram.distribution");
        let Some(metric::Data::ExponentialHistogram(histogram)) = metric.data.as_ref() else {
            panic!("expected exponential histogram metric")
        };
        assert_eq!(
            histogram.aggregation_temporality,
            AggregationTemporality::Delta as i32
        );
        let [point] = histogram.data_points.as_slice() else {
            panic!("expected one exponential histogram data point")
        };
        assert_eq!(point.count, 4);
        assert_eq!(point.sum, Some(15.0));
        assert_eq!(point.min, Some(1.0));
        assert_eq!(point.max, Some(8.0));
        assert_eq!(point.start_time_unix_nano, DELTA_START);
    }

    #[test]
    fn encodes_all_instrument_kinds_with_otlp_semantics() {
        let encoder = empty_resource_encoder();
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &ALL_METRICS_DESCRIPTOR,
                empty_attributes(),
                vec![
                    MetricValue::U64(7),
                    MetricValue::U64(u64::MAX),
                    MetricValue::F64(-2.5),
                    MetricValue::F64(18.25),
                    MetricValue::F64(42.0),
                    MetricValue::Mmsc(MmscSnapshot {
                        min: 2.0,
                        max: 9.0,
                        sum: 20.0,
                        count: 4,
                    }),
                ],
            )],
        };

        let request = decode_request(
            encoder
                .encode(&batch)
                .expect("encode succeeds")
                .expect("non-empty request"),
        );
        let scope = only_scope(&request);
        assert_eq!(scope.scope.as_ref().expect("scope").name, "test.scope");
        assert_eq!(scope.metrics.len(), 6);

        let delta_counter = metric_named(scope, "counter.delta");
        assert_eq!(delta_counter.description, "Delta counter");
        assert_eq!(delta_counter.unit, "{request}");
        let (point, sum) = number_point(delta_counter);
        assert_eq!(
            sum.aggregation_temporality,
            AggregationTemporality::Delta as i32
        );
        assert!(sum.is_monotonic);
        assert_eq!(point.start_time_unix_nano, DELTA_START);
        assert_eq!(point.time_unix_nano, COLLECTION_TIME);
        assert_eq!(point.value, Some(number_data_point::Value::AsInt(7)));

        let cumulative_counter = metric_named(scope, "counter.cumulative");
        let (point, sum) = number_point(cumulative_counter);
        assert_eq!(
            sum.aggregation_temporality,
            AggregationTemporality::Cumulative as i32
        );
        assert!(sum.is_monotonic);
        assert_eq!(point.start_time_unix_nano, CUMULATIVE_START);
        assert_eq!(point.time_unix_nano, COLLECTION_TIME);
        assert_eq!(point.value, Some(number_data_point::Value::AsInt(i64::MAX)));

        let up_down = metric_named(scope, "up_down.delta");
        let (point, sum) = number_point(up_down);
        assert_eq!(
            sum.aggregation_temporality,
            AggregationTemporality::Delta as i32
        );
        assert!(!sum.is_monotonic);
        assert_eq!(point.start_time_unix_nano, DELTA_START);
        assert_eq!(point.time_unix_nano, COLLECTION_TIME);
        assert_eq!(point.value, Some(number_data_point::Value::AsDouble(-2.5)));

        let gauge = metric_named(scope, "gauge");
        let Some(metric::Data::Gauge(gauge)) = gauge.data.as_ref() else {
            panic!("expected gauge metric")
        };
        let [point] = gauge.data_points.as_slice() else {
            panic!("expected one gauge point")
        };
        assert_eq!(point.start_time_unix_nano, 0);
        assert_eq!(point.time_unix_nano, COLLECTION_TIME);
        assert_eq!(point.value, Some(number_data_point::Value::AsDouble(18.25)));

        let scalar = metric_named(scope, "histogram.scalar");
        let Some(metric::Data::Histogram(histogram)) = scalar.data.as_ref() else {
            panic!("expected histogram metric")
        };
        assert_eq!(
            histogram.aggregation_temporality,
            AggregationTemporality::Delta as i32
        );
        let [point] = histogram.data_points.as_slice() else {
            panic!("expected one histogram point")
        };
        assert_eq!(point.start_time_unix_nano, DELTA_START);
        assert_eq!(point.time_unix_nano, COLLECTION_TIME);
        assert_eq!(point.count, 1);
        assert_eq!(point.sum, Some(42.0));
        assert_eq!(point.min, Some(42.0));
        assert_eq!(point.max, Some(42.0));
        assert_eq!(
            point.explicit_bounds,
            vec![
                0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 2500.0,
                5000.0, 7500.0, 10000.0,
            ]
        );
        let mut expected_buckets = vec![0; 16];
        expected_buckets[4] = 1;
        assert_eq!(point.bucket_counts, expected_buckets);

        let mmsc = metric_named(scope, "histogram.mmsc");
        let Some(metric::Data::ExponentialHistogram(histogram)) = mmsc.data.as_ref() else {
            panic!("expected MMSC exponential histogram metric")
        };
        assert_eq!(
            histogram.aggregation_temporality,
            AggregationTemporality::Delta as i32
        );
        let [point] = histogram.data_points.as_slice() else {
            panic!("expected one MMSC histogram point")
        };
        assert_eq!(point.start_time_unix_nano, DELTA_START);
        assert_eq!(point.time_unix_nano, COLLECTION_TIME);
        assert_eq!(point.count, 4);
        assert_eq!(point.sum, Some(20.0));
        assert_eq!(point.min, Some(2.0));
        assert_eq!(point.max, Some(9.0));
        assert_eq!(point.scale, 0);
        assert_eq!(point.zero_count, 0);
        assert!(point.positive.is_none());
        assert!(point.negative.is_none());
    }

    #[test]
    fn emits_multiple_scopes_while_omitting_empty_mmsc_fields_and_sets() {
        let empty_mmsc = MetricValue::Mmsc(MmscSnapshot {
            min: f64::MAX,
            max: f64::MIN,
            sum: 0.0,
            count: 0,
        });
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![
                metric_set(
                    &ALL_METRICS_DESCRIPTOR,
                    empty_attributes(),
                    vec![
                        MetricValue::U64(1),
                        MetricValue::U64(2),
                        MetricValue::F64(-1.0),
                        MetricValue::F64(3.0),
                        MetricValue::F64(4.0),
                        empty_mmsc.clone(),
                    ],
                ),
                metric_set(
                    &MMSC_ONLY_DESCRIPTOR,
                    empty_attributes(),
                    vec![empty_mmsc.clone()],
                ),
                metric_set(
                    &MMSC_ONLY_DESCRIPTOR,
                    empty_attributes(),
                    vec![MetricValue::Mmsc(MmscSnapshot {
                        min: 2.0,
                        max: 8.0,
                        sum: 10.0,
                        count: 2,
                    })],
                ),
            ],
        };

        let request = decode_request(
            empty_resource_encoder()
                .encode(&batch)
                .expect("encode succeeds")
                .expect("non-empty request"),
        );
        let [resource_metrics] = request.resource_metrics.as_slice() else {
            panic!("expected one resource metrics message")
        };
        assert_eq!(resource_metrics.scope_metrics.len(), 2);

        let first = &resource_metrics.scope_metrics[0];
        assert_eq!(first.scope.as_ref().expect("scope").name, "test.scope");
        assert_eq!(first.metrics.len(), 5);
        assert!(
            first
                .metrics
                .iter()
                .all(|metric| metric.name != "histogram.mmsc")
        );

        let second = &resource_metrics.scope_metrics[1];
        assert_eq!(
            second.scope.as_ref().expect("scope").name,
            "test.empty_mmsc"
        );
        assert_eq!(second.metrics.len(), 1);
        assert_eq!(second.metrics[0].name, "histogram.empty");
    }

    #[test]
    fn emits_meaningful_zero_scalar_values() {
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &ALL_METRICS_DESCRIPTOR,
                empty_attributes(),
                vec![
                    MetricValue::U64(0),
                    MetricValue::U64(0),
                    MetricValue::F64(0.0),
                    MetricValue::F64(0.0),
                    MetricValue::F64(0.0),
                    MetricValue::Mmsc(MmscSnapshot {
                        min: f64::MAX,
                        max: f64::MIN,
                        sum: 0.0,
                        count: 0,
                    }),
                ],
            )],
        };

        let request = decode_request(
            empty_resource_encoder()
                .encode(&batch)
                .expect("encode succeeds")
                .expect("zero scalar values must still produce a request"),
        );
        let scope = only_scope(&request);
        assert_eq!(scope.metrics.len(), 5);

        for name in ["counter.delta", "counter.cumulative", "up_down.delta"] {
            let (point, _) = number_point(metric_named(scope, name));
            assert!(matches!(
                point.value,
                Some(number_data_point::Value::AsInt(0))
                    | Some(number_data_point::Value::AsDouble(0.0))
            ));
        }

        let gauge = metric_named(scope, "gauge");
        let Some(metric::Data::Gauge(gauge)) = gauge.data.as_ref() else {
            panic!("expected gauge metric")
        };
        assert_eq!(
            gauge.data_points[0].value,
            Some(number_data_point::Value::AsDouble(0.0))
        );

        let histogram = metric_named(scope, "histogram.scalar");
        let Some(metric::Data::Histogram(histogram)) = histogram.data.as_ref() else {
            panic!("expected histogram metric")
        };
        let point = &histogram.data_points[0];
        assert_eq!(point.count, 1);
        assert_eq!(point.sum, Some(0.0));
        assert_eq!(point.bucket_counts[0], 1);
    }

    #[test]
    fn scalar_histogram_uses_closed_upper_bounds_and_handles_extremes_and_u64() {
        let cases = [
            (MetricValue::F64(f64::MIN), 0, f64::MIN),
            (MetricValue::F64(0.0), 0, 0.0),
            (MetricValue::F64(f64::EPSILON), 1, f64::EPSILON),
            (MetricValue::F64(5.0), 1, 5.0),
            (
                MetricValue::F64(5.000_000_000_000_001),
                2,
                5.000_000_000_000_001,
            ),
            (MetricValue::F64(10_000.0), 14, 10_000.0),
            (MetricValue::F64(f64::MAX), 15, f64::MAX),
            (MetricValue::U64(u64::MAX), 15, u64::MAX as f64),
        ];

        for (value, expected_bucket, expected_value) in cases {
            let point = scalar_histogram_data_point(&value, DELTA_START, COLLECTION_TIME, &[]);
            assert_eq!(point.count, 1);
            assert_eq!(
                point.sum,
                (expected_value >= 0.0).then_some(expected_value),
                "OTLP histogram sums are only defined for non-negative populations"
            );
            assert_eq!(point.min, Some(expected_value));
            assert_eq!(point.max, Some(expected_value));
            assert_eq!(point.bucket_counts.iter().sum::<u64>(), 1);
            assert_eq!(point.bucket_counts[expected_bucket], 1);
        }

        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &U64_HISTOGRAM_DESCRIPTOR,
                empty_attributes(),
                vec![MetricValue::U64(u64::MAX)],
            )],
        };
        let request = decode_request(
            empty_resource_encoder()
                .encode(&batch)
                .expect("u64 histogram value matches its descriptor")
                .expect("histogram produces a request"),
        );
        let metric = metric_named(only_scope(&request), "histogram.u64");
        let Some(metric::Data::Histogram(histogram)) = metric.data.as_ref() else {
            panic!("expected histogram metric")
        };
        assert_eq!(histogram.data_points[0].sum, Some(u64::MAX as f64));
        assert_eq!(histogram.data_points[0].bucket_counts[15], 1);
    }

    #[test]
    fn omits_mmsc_sum_when_the_population_contains_negative_values() {
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &MMSC_ONLY_DESCRIPTOR,
                empty_attributes(),
                vec![MetricValue::Mmsc(MmscSnapshot {
                    min: -2.0,
                    max: 8.0,
                    sum: 6.0,
                    count: 2,
                })],
            )],
        };
        let request = decode_request(
            empty_resource_encoder()
                .encode(&batch)
                .expect("negative MMSC values are valid")
                .expect("MMSC produces a request"),
        );
        let metric = metric_named(only_scope(&request), "histogram.empty");
        let Some(metric::Data::ExponentialHistogram(histogram)) = metric.data.as_ref() else {
            panic!("expected exponential histogram metric")
        };
        let [point] = histogram.data_points.as_slice() else {
            panic!("expected one histogram data point")
        };
        assert_eq!(point.sum, None);
        assert_eq!(point.min, Some(-2.0));
        assert_eq!(point.max, Some(8.0));
        assert_eq!(point.count, 2);
    }

    #[test]
    fn omits_empty_mmsc_and_empty_batches() {
        let encoder = empty_resource_encoder();
        let empty_mmsc = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &MMSC_ONLY_DESCRIPTOR,
                empty_attributes(),
                vec![MetricValue::Mmsc(MmscSnapshot {
                    min: f64::MAX,
                    max: f64::MIN,
                    sum: 0.0,
                    count: 0,
                })],
            )],
        };
        assert!(
            encoder
                .encode(&empty_mmsc)
                .expect("encode succeeds")
                .is_none()
        );

        let empty_batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: Vec::new(),
        };
        assert!(
            encoder
                .encode(&empty_batch)
                .expect("encode succeeds")
                .is_none()
        );
    }

    #[test]
    fn attaches_native_attributes_to_scope_and_preserves_resource() {
        let resource = Resource {
            attributes: vec![KeyValue::new(
                "service.name",
                AnyValue::new_string("telemetry-test"),
            )],
            dropped_attributes_count: 2,
            entity_refs: Vec::new(),
        };
        let fragment = ResourceLogs {
            resource: Some(resource.clone()),
            scope_logs: Vec::new(),
            schema_url: "https://resource.example/schema".to_owned(),
        }
        .encode_to_vec();
        let encoder = MetricsOtlpEncoder::new(&fragment).expect("valid log resource fragment");

        let mut labels = BTreeMap::new();
        let _ = labels.insert("overflow".to_owned(), AttributeValue::UInt(u64::MAX));
        let _ = labels.insert(
            "region".to_owned(),
            AttributeValue::String("west".to_owned()),
        );
        let attributes = shared_attributes(
            &FULL_ATTRIBUTES_DESCRIPTOR,
            vec![
                AttributeValue::String("worker-a".to_owned()),
                AttributeValue::Int(-4),
                AttributeValue::UInt(u64::MAX),
                AttributeValue::Double(0.75),
                AttributeValue::Boolean(true),
                AttributeValue::Map(labels),
            ],
        );
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &MMSC_ONLY_DESCRIPTOR,
                attributes,
                vec![MetricValue::Mmsc(MmscSnapshot {
                    min: 1.0,
                    max: 1.0,
                    sum: 1.0,
                    count: 1,
                })],
            )],
        };

        let request = decode_request(
            encoder
                .encode(&batch)
                .expect("encode succeeds")
                .expect("non-empty request"),
        );
        let [resource_metrics] = request.resource_metrics.as_slice() else {
            panic!("expected one resource metrics message")
        };
        assert_eq!(resource_metrics.resource, Some(resource));
        assert_eq!(
            resource_metrics.schema_url,
            "https://resource.example/schema"
        );

        let scope = resource_metrics.scope_metrics[0]
            .scope
            .as_ref()
            .expect("instrumentation scope");
        assert_eq!(scope.name, "test.empty_mmsc");
        assert_eq!(scope.attributes.len(), 6);
        assert_eq!(
            scope.attributes[0]
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref()),
            Some(&any_value::Value::StringValue("worker-a".to_owned()))
        );
        assert_eq!(
            scope.attributes[1]
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref()),
            Some(&any_value::Value::IntValue(-4))
        );
        assert_eq!(
            scope.attributes[2]
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref()),
            Some(&any_value::Value::IntValue(i64::MAX))
        );
        assert_eq!(
            scope.attributes[3]
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref()),
            Some(&any_value::Value::DoubleValue(0.75))
        );
        assert_eq!(
            scope.attributes[4]
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref()),
            Some(&any_value::Value::BoolValue(true))
        );
        assert_eq!(scope.attributes[5].key, "worker.labels");
        assert_eq!(
            scope.attributes[5]
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref()),
            Some(&any_value::Value::KvlistValue(KeyValueList {
                values: vec![
                    KeyValue::new("overflow", AnyValue::new_int(i64::MAX)),
                    KeyValue::new("region", AnyValue::new_string("west")),
                ],
            }))
        );
    }

    #[test]
    fn rejects_invalid_resource_fragment() {
        let error = MetricsOtlpEncoder::new(&[0x0a, 0x02, 0x08])
            .expect_err("truncated nested resource must fail");
        assert!(matches!(error, Error::InvalidResource(_)));
    }

    #[test]
    fn rejects_metric_set_value_count_mismatches() {
        for actual in [0, 2] {
            let batch = MetricExportBatch {
                time_unix_nano: COLLECTION_TIME,
                metric_sets: vec![metric_set(
                    &MMSC_ONLY_DESCRIPTOR,
                    empty_attributes(),
                    vec![
                        MetricValue::Mmsc(MmscSnapshot {
                            min: 1.0,
                            max: 1.0,
                            sum: 1.0,
                            count: 1,
                        });
                        actual
                    ],
                )],
            };

            let error = empty_resource_encoder()
                .encode(&batch)
                .expect_err("descriptor/value count mismatch must fail");
            assert!(matches!(
                error,
                Error::ValueCountMismatch {
                    metric_set: "test.empty_mmsc",
                    expected: 1,
                    actual: found,
                } if found == actual
            ));
        }
    }

    #[test]
    fn rejects_metric_value_kind_mismatches() {
        let cases = [
            (
                &INVALID_SUM_DESCRIPTOR,
                MetricValue::F64(1.0),
                "invalid.sum",
                "u64",
                "f64",
            ),
            (
                &F64_GAUGE_DESCRIPTOR,
                MetricValue::U64(1),
                "gauge.f64",
                "f64",
                "u64",
            ),
            (
                &MMSC_ONLY_DESCRIPTOR,
                MetricValue::U64(1),
                "histogram.empty",
                "mmsc",
                "u64",
            ),
        ];

        for (descriptor, value, metric, expected, actual) in cases {
            let batch = MetricExportBatch {
                time_unix_nano: COLLECTION_TIME,
                metric_sets: vec![metric_set(descriptor, empty_attributes(), vec![value])],
            };
            let error = empty_resource_encoder()
                .encode(&batch)
                .expect_err("descriptor/value kind mismatch must fail");
            assert!(matches!(
                error,
                Error::ValueKindMismatch {
                    metric: found_metric,
                    expected: found_expected,
                    actual: found_actual,
                } if found_metric == metric
                    && found_expected == expected
                    && found_actual == actual
            ));
        }
    }

    #[test]
    fn rejects_sum_without_temporality() {
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &INVALID_SUM_DESCRIPTOR,
                empty_attributes(),
                vec![MetricValue::U64(1)],
            )],
        };
        let error = empty_resource_encoder()
            .encode(&batch)
            .expect_err("missing temporality must fail");
        assert!(matches!(
            error,
            Error::MissingTemporality {
                metric: "invalid.sum"
            }
        ));
    }

    #[test]
    fn encoded_metrics_are_consumable_by_the_otap_export_path() {
        let resource = Resource {
            attributes: vec![KeyValue::new(
                "service.name",
                AnyValue::new_string("telemetry-test"),
            )],
            dropped_attributes_count: 0,
            entity_refs: Vec::new(),
        };
        let encoder = MetricsOtlpEncoder::new(
            &ResourceLogs {
                resource: Some(resource),
                scope_logs: Vec::new(),
                schema_url: "https://resource.example/schema".to_owned(),
            }
            .encode_to_vec(),
        )
        .expect("valid resource fragment");
        let mut labels = BTreeMap::new();
        let _ = labels.insert(
            "region".to_owned(),
            AttributeValue::String("west".to_owned()),
        );
        let attributes = shared_attributes(
            &FULL_ATTRIBUTES_DESCRIPTOR,
            vec![
                AttributeValue::String("worker-a".to_owned()),
                AttributeValue::Int(-4),
                AttributeValue::UInt(9),
                AttributeValue::Double(0.75),
                AttributeValue::Boolean(true),
                AttributeValue::Map(labels),
            ],
        );
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &ALL_METRICS_DESCRIPTOR,
                attributes,
                vec![
                    MetricValue::U64(7),
                    MetricValue::U64(11),
                    MetricValue::F64(-2.5),
                    MetricValue::F64(18.25),
                    MetricValue::F64(42.0),
                    MetricValue::Mmsc(MmscSnapshot {
                        min: 2.0,
                        max: 9.0,
                        sum: 20.0,
                        count: 4,
                    }),
                ],
            )],
        };
        let encoded = encoder
            .encode(&batch)
            .expect("OTLP encoding succeeds")
            .expect("batch is non-empty");
        let payload: OtapPayload = encoded.into();

        let records: OtapArrowRecords = payload
            .try_into_with_default()
            .expect("OTAP exporter can convert bridge output to Arrow records");
        assert!(matches!(records, OtapArrowRecords::Metrics(_)));

        let view = OtapMetricsView::try_from(&records).expect("valid OTAP metrics view");
        let mut resources = view.resources();
        let resource_metrics = resources.next().expect("one resource metrics group");
        assert!(resources.next().is_none());
        assert_eq!(
            resource_metrics.schema_url(),
            Some(b"https://resource.example/schema".as_slice())
        );
        {
            let resource = resource_metrics.resource().expect("resource metadata");
            let service_name = resource
                .attributes()
                .find(|attribute| attribute.key() == b"service.name")
                .expect("service.name resource attribute");
            let value = service_name.value().expect("resource attribute value");
            assert_eq!(value.as_string(), Some(b"telemetry-test".as_slice()));
        }

        let mut scopes = resource_metrics.scopes();
        let scope_metrics = scopes.next().expect("one scope metrics group");
        assert!(scopes.next().is_none());
        {
            let scope = scope_metrics.scope().expect("instrumentation scope");
            assert_eq!(scope.name(), Some(b"test.scope".as_slice()));
            let scope_attributes = scope.attributes().collect::<Vec<_>>();
            assert_eq!(scope_attributes.len(), 6);
            let worker_name = scope_attributes
                .iter()
                .find(|attribute| attribute.key() == b"worker.name")
                .expect("worker.name scope attribute");
            let value = worker_name.value().expect("scope attribute value");
            assert_eq!(value.as_string(), Some(b"worker-a".as_slice()));
        }

        let counter = scope_metrics
            .metrics()
            .find(|metric| metric.name() == b"counter.delta")
            .expect("delta counter survives OTAP conversion");
        assert_eq!(counter.description(), b"Delta counter");
        assert_eq!(counter.unit(), b"{request}");
        let data = counter.data().expect("counter data");
        let sum = data.as_sum().expect("counter remains a sum");
        assert!(sum.is_monotonic());
        assert_eq!(
            sum.aggregation_temporality(),
            otap_df_pdata_views::views::metrics::AggregationTemporality::Delta
        );
        let mut points = sum.data_points();
        let point = points.next().expect("counter data point");
        assert!(points.next().is_none());
        assert_eq!(point.start_time_unix_nano(), DELTA_START);
        assert_eq!(point.time_unix_nano(), COLLECTION_TIME);
        assert_eq!(point.value(), Some(Value::Integer(7)));
    }
}
