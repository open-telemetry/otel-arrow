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
//! come from the process-level pre-encoded resource field retained by
//! [`MetricsOtlpEncoder`].
//!
//! # Instrument mapping
//!
//! | Internal instrument | OTLP representation | Start time and semantics |
//! | --- | --- | --- |
//! | `Counter` | monotonic `Sum` | Descriptor temporality; delta-window or registration start |
//! | `UpDownCounter` | non-monotonic `Sum` | Descriptor temporality; delta-window or registration start |
//! | `Gauge` | `Gauge` | Start time is zero, as required for an instantaneous value |
//! | `Mmsc` | delta, bucketless `Histogram` | Delta-window start; preserves exact min, max, sum, and count |
//! | `ExponentialHistogram` | delta `ExponentialHistogram` | Delta-window start; preserves exact exponential bucket counts |
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
//! # Encoding strategy
//!
//! Semantic preparation resolves views, collisions, coalescing, and effective
//! OTLP streams into a lightweight borrowed representation. The encoder then
//! writes the complete request directly into a `ProtoBuffer`. Nested messages
//! use `BoundedBuf::encode_len_delimited`, and the trusted pre-encoded resource
//! field shared with internal logs is copied directly into `ResourceMetrics`.
//!
//! Protobuf field order is semantically insignificant. The encoder nevertheless
//! writes scalar aggregation metadata before repeated data points to keep the
//! wire layout stable and place context before potentially long repeated fields.
//!
//! Production encoding does not construct generated OTLP messages or traverse
//! a Prost object tree. Tests decode the emitted bytes with Prost as an
//! independent compatibility oracle.
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
use otap_df_expohisto::HistogramView;
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::otlp::common::{BoundedBuf, Dropped, MAX_OTLP_SIZE_LIMIT, ProtoBuffer};
use otap_df_pdata::proto::consts::field_num::common::{
    ANY_VALUE_BOOL_VALUE, ANY_VALUE_DOUBLE_VALUE, ANY_VALUE_INT_VALUE, ANY_VALUE_KVLIST_VALUE,
    ANY_VALUE_STRING_VALUE, INSTRUMENTATION_SCOPE_ATTRIBUTES, INSTRUMENTATION_SCOPE_NAME,
    KEY_VALUE_KEY, KEY_VALUE_LIST_VALUES, KEY_VALUE_VALUE,
};
use otap_df_pdata::proto::consts::field_num::metrics::{
    EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS, EXP_HISTOGRAM_BUCKET_OFFSET, EXP_HISTOGRAM_DP_ATTRIBUTES,
    EXP_HISTOGRAM_DP_COUNT, EXP_HISTOGRAM_DP_MAX, EXP_HISTOGRAM_DP_MIN, EXP_HISTOGRAM_DP_POSITIVE,
    EXP_HISTOGRAM_DP_SCALE, EXP_HISTOGRAM_DP_START_TIME_UNIX_NANO, EXP_HISTOGRAM_DP_SUM,
    EXP_HISTOGRAM_DP_TIME_UNIX_NANO, EXP_HISTOGRAM_DP_ZERO_COUNT,
    EXPONENTIAL_HISTOGRAM_AGGREGATION_TEMPORALITY, EXPONENTIAL_HISTOGRAM_DATA_POINTS,
    GAUGE_DATA_POINTS, HISTOGRAM_AGGREGATION_TEMPORALITY, HISTOGRAM_DATA_POINTS,
    HISTOGRAM_DP_ATTRIBUTES, HISTOGRAM_DP_COUNT, HISTOGRAM_DP_MAX, HISTOGRAM_DP_MIN,
    HISTOGRAM_DP_START_TIME_UNIX_NANO, HISTOGRAM_DP_SUM, HISTOGRAM_DP_TIME_UNIX_NANO,
    METRIC_DESCRIPTION, METRIC_EXPONENTIAL_HISTOGRAM, METRIC_GAUGE, METRIC_HISTOGRAM, METRIC_NAME,
    METRIC_SUM, METRIC_UNIT, METRICS_DATA_RESOURCE_METRICS, NUMBER_DP_AS_DOUBLE, NUMBER_DP_AS_INT,
    NUMBER_DP_ATTRIBUTES, NUMBER_DP_START_TIME_UNIX_NANO, NUMBER_DP_TIME_UNIX_NANO,
    RESOURCE_METRICS_SCOPE_METRICS, SCOPE_METRICS_METRICS, SCOPE_METRICS_SCOPE,
    SUM_AGGREGATION_TEMPORALITY, SUM_DATA_POINTS, SUM_IS_MONOTONIC,
};
use otap_df_pdata::proto::consts::wire_types;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Errors produced while encoding registry metrics as OTLP.
#[derive(Debug, thiserror::Error)]
pub enum Error {
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

    /// The encoded request exceeded the protobuf buffer limit.
    #[error("internal telemetry metrics request exceeded the OTLP size limit of {limit} bytes")]
    RequestTooLarge {
        /// Maximum encoded request size accepted by the protobuf buffer.
        limit: usize,
    },
}

impl From<Dropped> for Error {
    fn from(_: Dropped) -> Self {
        Self::RequestTooLarge {
            limit: MAX_OTLP_SIZE_LIMIT,
        }
    }
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

/// Reusable OTLP encoder holding the trusted pre-encoded process resource.
///
/// Constructor resource fragments must come from the internal telemetry
/// resource encoder rather than external input.
///
/// TODO: Consider an opaque resource-fragment type if this API gains external
/// producers that cannot uphold this invariant.
#[derive(Debug, Clone)]
pub struct MetricsOtlpEncoder {
    resource_fragment: Bytes,
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

/// One resolved OTLP metric stream with one or more data points.
struct PreparedMetric<'a> {
    field: &'static MetricsField,
    name: &'a str,
    description: &'a str,
    points: SmallVec<[PreparedPoint<'a>; 1]>,
}

/// One data point and the source timing/attribute context needed to encode it.
#[derive(Clone, Copy)]
struct PreparedPoint<'a> {
    value: &'a MetricValue,
    metric_set: &'a MetricSetExport,
}

/// One effective OTLP instrumentation scope.
struct PreparedScope<'a> {
    name: &'static str,
    attributes: &'a EntityAttributeSet,
    metrics: Vec<PreparedMetric<'a>>,
}

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
    #[must_use]
    pub fn new(resource_fragment: &[u8]) -> Self {
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
    #[must_use]
    pub fn new_with_views(resource_fragment: &[u8], views: Vec<MetricView>) -> Self {
        Self {
            resource_fragment: Bytes::copy_from_slice(resource_fragment),
            views,
        }
    }

    /// Encodes a registry export batch. Empty batches produce no pdata.
    pub fn encode(&self, batch: &MetricExportBatch) -> Result<Option<OtlpProtoBytes>, Error> {
        let scopes = if batch
            .metric_sets
            .iter()
            .any(|metric_set| metric_set.identity_may_repeat)
        {
            let metric_sets = coalesce_metric_sets(&batch.metric_sets)?;
            let scopes = self.prepare_metric_sets(
                metric_sets.iter().map(CoalescedMetricSet::as_metric_set),
                metric_sets.len(),
            )?;
            if scopes.is_empty() {
                return Ok(None);
            }
            let mut buffer = ProtoBuffer::with_capacity(1024);
            encode_request(
                &mut buffer,
                &self.resource_fragment,
                &scopes,
                batch.time_unix_nano,
            )?;
            return Ok(Some(OtlpProtoBytes::ExportMetricsRequest(
                buffer.into_bytes(),
            )));
        } else {
            self.prepare_metric_sets(batch.metric_sets.iter(), batch.metric_sets.len())?
        };

        if scopes.is_empty() {
            return Ok(None);
        }

        let mut buffer = ProtoBuffer::with_capacity(1024);
        encode_request(
            &mut buffer,
            &self.resource_fragment,
            &scopes,
            batch.time_unix_nano,
        )?;
        Ok(Some(OtlpProtoBytes::ExportMetricsRequest(
            buffer.into_bytes(),
        )))
    }

    fn prepare_metric_sets<'batch, 'view>(
        &'view self,
        metric_sets: impl Iterator<Item = &'batch MetricSetExport>,
        metric_set_count: usize,
    ) -> Result<Vec<PreparedScope<'batch>>, Error>
    where
        'view: 'batch,
    {
        let mut scopes = Vec::with_capacity(metric_set_count);
        let mut scope_identities = HashMap::with_capacity(metric_set_count);
        if self.views.is_empty() {
            for metric_set in metric_sets {
                if let Some(scope) = prepare_metric_set_without_views(metric_set)? {
                    append_prepared_scope(&mut scopes, &mut scope_identities, metric_set, scope);
                }
            }
        } else {
            let mut collisions = CollisionIndex::with_capacity(metric_set_count);
            for metric_set in metric_sets {
                if let Some(scope) =
                    prepare_metric_set_with_views(metric_set, &self.views, &mut collisions)?
                {
                    append_prepared_scope(&mut scopes, &mut scope_identities, metric_set, scope);
                }
            }
        }
        Ok(scopes)
    }
}

/// Combines bucket-local points into the same metric streams without merging
/// their values. Distinct data-point attributes therefore remain distinct OTLP
/// points, while independently registered producers with the same attributes
/// have already been numerically coalesced above this layer.
fn append_prepared_scope<'a>(
    scopes: &mut Vec<PreparedScope<'a>>,
    identities: &mut HashMap<(usize, usize), usize>,
    metric_set: &'a MetricSetExport,
    incoming: PreparedScope<'a>,
) {
    let identity = (
        std::ptr::from_ref(metric_set.descriptor) as usize,
        Arc::as_ptr(&metric_set.attributes) as usize,
    );
    if let Some(index) = identities.get(&identity).copied() {
        merge_prepared_scope(&mut scopes[index], incoming);
    } else {
        let index = scopes.len();
        let _ = identities.insert(identity, index);
        scopes.push(incoming);
    }
}

fn merge_prepared_scope<'a>(target: &mut PreparedScope<'a>, incoming: PreparedScope<'a>) {
    for incoming_metric in incoming.metrics {
        let target_metric = target.metrics.iter_mut().find(|target_metric| {
            // TODO: Is this compatibility checking needed? can't imagine how
            // a single SDK would reach a point of having a disagreement.
            target_metric.name == incoming_metric.name
                && target_metric.description == incoming_metric.description
                && target_metric.field.unit == incoming_metric.field.unit
                && metric_data_compatible(target_metric.field, incoming_metric.field)
        });
        if let Some(target_metric) = target_metric {
            target_metric.points.extend(incoming_metric.points);
        } else {
            target.metrics.push(incoming_metric);
        }
    }
}

fn metric_data_compatible(left: &MetricsField, right: &MetricsField) -> bool {
    left.instrument == right.instrument && left.temporality == right.temporality
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
            | Instrument::Mmsc
            | Instrument::ExponentialHistogram => current.add_in_place(incoming),
        }
    }
}

/// Prepares one metric set without paying any view-resolution bookkeeping.
fn prepare_metric_set_without_views(
    metric_set: &MetricSetExport,
) -> Result<Option<PreparedScope<'_>>, Error> {
    validate_value_count(metric_set)?;

    let mut metrics = Vec::with_capacity(metric_set.values.len());
    for (field, value) in metric_set.descriptor.metrics.iter().zip(&metric_set.values) {
        validate_value_kind(field, value)?;
        if let Some(metric) = prepare_metric(field, value, metric_set, field.name, field.brief)? {
            metrics.push(metric);
        }
    }

    if metrics.is_empty() {
        return Ok(None);
    }
    Ok(Some(build_prepared_scope(metric_set, metrics)))
}

/// Prepares one metric set after resolving views and checking stream collisions.
fn prepare_metric_set_with_views<'batch, 'view>(
    metric_set: &'batch MetricSetExport,
    views: &'view [MetricView],
    collisions: &mut CollisionIndex<'view>,
) -> Result<Option<PreparedScope<'batch>>, Error>
where
    'view: 'batch,
{
    validate_value_count(metric_set)?;

    let mut metrics = Vec::with_capacity(metric_set.values.len());
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
            if let Some(metric) =
                prepare_metric(field, value, metric_set, stream.name, stream.description)?
            {
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
    Ok(Some(build_prepared_scope(metric_set, metrics)))
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

fn build_prepared_scope<'a>(
    metric_set: &'a MetricSetExport,
    metrics: Vec<PreparedMetric<'a>>,
) -> PreparedScope<'a> {
    PreparedScope {
        name: metric_set.descriptor.name,
        attributes: &metric_set.attributes,
        metrics,
    }
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

/// Prepares one multivariate metric field as a univariate OTLP stream.
fn prepare_metric<'a>(
    field: &'static MetricsField,
    value: &'a MetricValue,
    metric_set: &'a MetricSetExport,
    name: &'a str,
    description: &'a str,
) -> Result<Option<PreparedMetric<'a>>, Error> {
    match field.instrument {
        Instrument::Counter | Instrument::UpDownCounter => {
            let _ = field
                .temporality
                .ok_or(Error::MissingTemporality { metric: field.name })?;
        }
        Instrument::Mmsc => {
            let MetricValue::Distribution(distribution) = value else {
                unreachable!("metric value kind was validated before encoding")
            };
            let DistributionValue::Basic(mmsc) = distribution else {
                unreachable!("metric value kind was validated before encoding")
            };
            if mmsc.count == 0 {
                return Ok(None);
            }
        }
        Instrument::ExponentialHistogram => {
            let MetricValue::Distribution(distribution) = value else {
                unreachable!("metric value kind was validated before encoding")
            };
            if distribution.is_empty() {
                return Ok(None);
            }
        }
        Instrument::Gauge => {}
    };

    Ok(Some(PreparedMetric {
        field,
        name,
        description,
        points: SmallVec::from_buf([PreparedPoint { value, metric_set }]),
    }))
}

fn encode_request(
    buffer: &mut ProtoBuffer,
    resource_fragment: &[u8],
    scopes: &[PreparedScope<'_>],
    time_unix_nano: u64,
) -> Result<(), Error> {
    buffer.encode_len_delimited(METRICS_DATA_RESOURCE_METRICS, |buffer| {
        buffer.extend_from_slice(resource_fragment)?;
        for scope in scopes {
            buffer.encode_len_delimited(RESOURCE_METRICS_SCOPE_METRICS, |buffer| {
                encode_scope(buffer, scope, time_unix_nano)
            })?;
        }
        Ok(())
    })
}

fn encode_scope(
    buffer: &mut ProtoBuffer,
    scope: &PreparedScope<'_>,
    time_unix_nano: u64,
) -> Result<(), Error> {
    buffer.encode_len_delimited(SCOPE_METRICS_SCOPE, |buffer| {
        encode_string(buffer, INSTRUMENTATION_SCOPE_NAME, scope.name)?;
        for (key, value) in scope.attributes.iter_attributes() {
            encode_key_value(buffer, INSTRUMENTATION_SCOPE_ATTRIBUTES, key, value)?;
        }
        Ok::<(), Error>(())
    })?;

    for metric in &scope.metrics {
        buffer.encode_len_delimited(SCOPE_METRICS_METRICS, |buffer| {
            encode_metric(buffer, metric, time_unix_nano)
        })?;
    }
    Ok::<(), Error>(())
}

fn encode_metric(
    buffer: &mut ProtoBuffer,
    metric: &PreparedMetric<'_>,
    time_unix_nano: u64,
) -> Result<(), Error> {
    encode_string(buffer, METRIC_NAME, metric.name)?;
    encode_string(buffer, METRIC_DESCRIPTION, metric.description)?;
    encode_string(buffer, METRIC_UNIT, metric.field.unit)?;

    // Sum and histogram messages intentionally place scalar aggregation metadata
    // before repeated data points; protobuf decoding itself is order-independent.
    match metric.field.instrument {
        Instrument::Gauge => {
            buffer.encode_len_delimited(METRIC_GAUGE, |buffer| {
                for point in &metric.points {
                    buffer.encode_len_delimited(GAUGE_DATA_POINTS, |buffer| {
                        encode_number_data_point(buffer, point, 0, time_unix_nano)
                    })?;
                }
                Ok::<(), Error>(())
            })?;
        }
        Instrument::Counter | Instrument::UpDownCounter => {
            let temporality = metric.field.temporality.ok_or(Error::MissingTemporality {
                metric: metric.field.name,
            })?;
            buffer.encode_len_delimited(METRIC_SUM, |buffer| {
                buffer.encode_field_tag(SUM_AGGREGATION_TEMPORALITY, wire_types::VARINT)?;
                buffer.encode_varint(encode_temporality(temporality))?;
                if matches!(metric.field.instrument, Instrument::Counter) {
                    buffer.encode_field_tag(SUM_IS_MONOTONIC, wire_types::VARINT)?;
                    buffer.encode_varint(1)?;
                }
                for point in &metric.points {
                    let start_time_unix_nano = match temporality {
                        Temporality::Delta => point.metric_set.delta_start_time_unix_nano,
                        Temporality::Cumulative => point.metric_set.cumulative_start_time_unix_nano,
                    };
                    buffer.encode_len_delimited(SUM_DATA_POINTS, |buffer| {
                        encode_number_data_point(
                            buffer,
                            point,
                            start_time_unix_nano,
                            time_unix_nano,
                        )
                    })?;
                }
                Ok::<(), Error>(())
            })?;
        }
        Instrument::Mmsc => {
            buffer.encode_len_delimited(METRIC_HISTOGRAM, |buffer| {
                buffer.encode_field_tag(HISTOGRAM_AGGREGATION_TEMPORALITY, wire_types::VARINT)?;
                buffer.encode_varint(encode_temporality(Temporality::Delta))?;
                for point in &metric.points {
                    buffer.encode_len_delimited(HISTOGRAM_DATA_POINTS, |buffer| {
                        encode_mmsc_data_point(buffer, point, time_unix_nano)
                    })?;
                }
                Ok::<(), Error>(())
            })?;
        }
        Instrument::ExponentialHistogram => {
            buffer.encode_len_delimited(METRIC_EXPONENTIAL_HISTOGRAM, |buffer| {
                buffer.encode_field_tag(
                    EXPONENTIAL_HISTOGRAM_AGGREGATION_TEMPORALITY,
                    wire_types::VARINT,
                )?;
                buffer.encode_varint(encode_temporality(Temporality::Delta))?;
                for point in &metric.points {
                    buffer.encode_len_delimited(EXPONENTIAL_HISTOGRAM_DATA_POINTS, |buffer| {
                        encode_exponential_histogram_data_point(buffer, point, time_unix_nano)
                    })?;
                }
                Ok::<(), Error>(())
            })?;
        }
    }
    Ok(())
}

fn encode_number_data_point(
    buffer: &mut ProtoBuffer,
    point: &PreparedPoint<'_>,
    start_time_unix_nano: u64,
    time_unix_nano: u64,
) -> Result<(), Error> {
    encode_datapoint_attributes(buffer, point, NUMBER_DP_ATTRIBUTES)?;
    encode_fixed64_if_nonzero(buffer, NUMBER_DP_START_TIME_UNIX_NANO, start_time_unix_nano)?;
    encode_fixed64_if_nonzero(buffer, NUMBER_DP_TIME_UNIX_NANO, time_unix_nano)?;
    match point.value {
        MetricValue::U64(value) => {
            encode_fixed64(buffer, NUMBER_DP_AS_INT, saturating_i64(*value) as u64)?
        }
        MetricValue::F64(value) => encode_double(buffer, NUMBER_DP_AS_DOUBLE, *value)?,
        MetricValue::Distribution(_) => {
            unreachable!("metric value kind was validated before encoding")
        }
    }
    Ok(())
}

fn encode_mmsc_data_point(
    buffer: &mut ProtoBuffer,
    point: &PreparedPoint<'_>,
    time_unix_nano: u64,
) -> Result<(), Error> {
    let MetricValue::Distribution(DistributionValue::Basic(mmsc)) = point.value else {
        unreachable!("metric value kind was validated before encoding")
    };
    encode_datapoint_attributes(buffer, point, HISTOGRAM_DP_ATTRIBUTES)?;
    encode_fixed64_if_nonzero(
        buffer,
        HISTOGRAM_DP_START_TIME_UNIX_NANO,
        point.metric_set.delta_start_time_unix_nano,
    )?;
    encode_fixed64_if_nonzero(buffer, HISTOGRAM_DP_TIME_UNIX_NANO, time_unix_nano)?;
    encode_fixed64_if_nonzero(buffer, HISTOGRAM_DP_COUNT, mmsc.count)?;
    if let Some(sum) = super::exphist::otlp_histogram_sum(mmsc.count, mmsc.sum, mmsc.min) {
        encode_double(buffer, HISTOGRAM_DP_SUM, sum)?;
    }
    encode_double(buffer, HISTOGRAM_DP_MIN, mmsc.min)?;
    encode_double(buffer, HISTOGRAM_DP_MAX, mmsc.max)
}

fn encode_exponential_histogram_data_point(
    buffer: &mut ProtoBuffer,
    point: &PreparedPoint<'_>,
    time_unix_nano: u64,
) -> Result<(), Error> {
    let MetricValue::Distribution(distribution) = point.value else {
        unreachable!("metric value kind was validated before encoding")
    };
    match distribution {
        DistributionValue::Basic(_) => {
            unreachable!("basic MMSC distributions use explicit-boundary histograms")
        }
        DistributionValue::Normal(histogram) => {
            encode_exponential_histogram_view(buffer, &histogram.view(), point, time_unix_nano)
        }
        DistributionValue::Detailed(histogram) => {
            encode_exponential_histogram_view(buffer, &histogram.view(), point, time_unix_nano)
        }
    }
}

fn encode_exponential_histogram_view<const N: usize>(
    buffer: &mut ProtoBuffer,
    view: &HistogramView<'_, N>,
    point: &PreparedPoint<'_>,
    time_unix_nano: u64,
) -> Result<(), Error> {
    let stats = view.stats();
    let positive = view.positive();
    encode_datapoint_attributes(buffer, point, EXP_HISTOGRAM_DP_ATTRIBUTES)?;
    encode_fixed64_if_nonzero(
        buffer,
        EXP_HISTOGRAM_DP_START_TIME_UNIX_NANO,
        point.metric_set.delta_start_time_unix_nano,
    )?;
    encode_fixed64_if_nonzero(buffer, EXP_HISTOGRAM_DP_TIME_UNIX_NANO, time_unix_nano)?;
    encode_fixed64_if_nonzero(buffer, EXP_HISTOGRAM_DP_COUNT, stats.count)?;
    if let Some(sum) = super::exphist::otlp_histogram_sum(stats.count, stats.sum, stats.min) {
        encode_double(buffer, EXP_HISTOGRAM_DP_SUM, sum)?;
    }
    if view.scale() != 0 {
        buffer.encode_field_tag(EXP_HISTOGRAM_DP_SCALE, wire_types::VARINT)?;
        buffer.encode_sint32(view.scale())?;
    }
    let mut positive_total = 0_u64;
    if !positive.is_empty() {
        buffer.encode_len_delimited(EXP_HISTOGRAM_DP_POSITIVE, |buffer| {
            if positive.offset() != 0 {
                buffer.encode_field_tag(EXP_HISTOGRAM_BUCKET_OFFSET, wire_types::VARINT)?;
                buffer.encode_sint32(positive.offset())?;
            }
            buffer.encode_len_delimited(EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS, |buffer| {
                for count in positive.iter() {
                    positive_total = positive_total.saturating_add(count);
                    buffer.encode_varint(count)?;
                }
                Ok::<(), Error>(())
            })
        })?;
    }
    let zero_count = stats.count.saturating_sub(positive_total);
    if zero_count != 0 {
        encode_fixed64(buffer, EXP_HISTOGRAM_DP_ZERO_COUNT, zero_count)?;
    }
    encode_double(buffer, EXP_HISTOGRAM_DP_MIN, stats.min)?;
    encode_double(buffer, EXP_HISTOGRAM_DP_MAX, stats.max)
}

fn encode_datapoint_attributes(
    buffer: &mut ProtoBuffer,
    point: &PreparedPoint<'_>,
    field_number: u64,
) -> Result<(), Error> {
    for (key, value) in &point.metric_set.item_attributes {
        buffer.encode_len_delimited(field_number, |buffer| {
            encode_string(buffer, KEY_VALUE_KEY, key)?;
            buffer.encode_len_delimited(KEY_VALUE_VALUE, |buffer| -> Result<(), Error> {
                buffer.encode_string(ANY_VALUE_STRING_VALUE, value)?;
                Ok(())
            })
        })?;
    }
    Ok(())
}

fn encode_key_value(
    buffer: &mut ProtoBuffer,
    outer_field: u64,
    key: &str,
    value: &AttributeValue,
) -> Result<(), Error> {
    buffer.encode_len_delimited(outer_field, |buffer| {
        encode_string(buffer, KEY_VALUE_KEY, key)?;
        buffer.encode_len_delimited(KEY_VALUE_VALUE, |buffer| {
            encode_attribute_value(buffer, value)
        })
    })
}

fn encode_attribute_value(buffer: &mut ProtoBuffer, value: &AttributeValue) -> Result<(), Error> {
    match value {
        AttributeValue::String(value) => {
            buffer.encode_string(ANY_VALUE_STRING_VALUE, value)?;
        }
        AttributeValue::Int(value) => {
            buffer.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT)?;
            buffer.encode_varint(*value as u64)?;
        }
        AttributeValue::UInt(value) => {
            buffer.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT)?;
            buffer.encode_varint(saturating_i64(*value) as u64)?;
        }
        AttributeValue::Double(value) => {
            encode_double(buffer, ANY_VALUE_DOUBLE_VALUE, *value)?;
        }
        AttributeValue::Boolean(value) => {
            buffer.encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT)?;
            buffer.encode_varint(u64::from(*value))?;
        }
        AttributeValue::Map(values) => {
            buffer.encode_len_delimited(ANY_VALUE_KVLIST_VALUE, |buffer| {
                for (key, value) in values {
                    encode_key_value(buffer, KEY_VALUE_LIST_VALUES, key, value)?;
                }
                Ok::<(), Error>(())
            })?;
        }
    }
    Ok(())
}

fn encode_fixed64(buffer: &mut ProtoBuffer, field_number: u64, value: u64) -> Result<(), Error> {
    buffer.encode_field_tag(field_number, wire_types::FIXED64)?;
    buffer.extend_from_slice(&value.to_le_bytes())?;
    Ok(())
}

fn encode_fixed64_if_nonzero(
    buffer: &mut ProtoBuffer,
    field_number: u64,
    value: u64,
) -> Result<(), Error> {
    if value != 0 {
        encode_fixed64(buffer, field_number, value)?;
    }
    Ok(())
}

fn encode_string(buffer: &mut ProtoBuffer, field_number: u64, value: &str) -> Result<(), Error> {
    if !value.is_empty() {
        buffer.encode_string(field_number, value)?;
    }
    Ok(())
}

fn encode_double(buffer: &mut ProtoBuffer, field_number: u64, value: f64) -> Result<(), Error> {
    buffer.encode_field_tag(field_number, wire_types::FIXED64)?;
    buffer.extend_from_slice(&value.to_le_bytes())?;
    Ok(())
}

/// Validates the descriptor/value pairing before any lossy projection occurs.
fn validate_value_kind(field: &MetricsField, value: &MetricValue) -> Result<(), Error> {
    let expected = match field.instrument {
        Instrument::Mmsc => "mmsc",
        Instrument::ExponentialHistogram => "exponential histogram",
        _ => match field.value_type {
            crate::descriptor::MetricValueType::U64 => "u64",
            crate::descriptor::MetricValueType::F64 => "f64",
        },
    };
    let actual = match value {
        MetricValue::U64(_) => "u64",
        MetricValue::F64(_) => "f64",
        MetricValue::Distribution(DistributionValue::Basic(_)) => "mmsc",
        MetricValue::Distribution(
            DistributionValue::Normal(_) | DistributionValue::Detailed(_),
        ) => "exponential histogram",
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

const fn encode_temporality(temporality: Temporality) -> u64 {
    match temporality {
        Temporality::Delta => 1,
        Temporality::Cumulative => 2,
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

    /// Builds a normal-tier snapshot by recording through its instrument,
    /// which is the only way a distribution is populated.
    fn normal_distribution(observations: &[f64]) -> DistributionValue {
        let mut histogram = crate::instrument::HistogramNormal::default();
        for &value in observations {
            histogram.record(value);
        }
        histogram.get()
    }

    use super::*;
    use crate::attributes::AttributeSetHandler;
    use crate::descriptor::{
        AttributeField, AttributeValueType, AttributesDescriptor, MetricValueType,
        MetricsDescriptor,
    };
    use crate::entity::{EntityAttributeSet, EntityRegistry};
    use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::common::v1::{
        AnyValue, KeyValue, KeyValueList, any_value,
    };
    use otap_df_pdata::proto::opentelemetry::logs::v1::ResourceLogs;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{
        AggregationTemporality, Metric, NumberDataPoint, ScopeMetrics, Sum, metric,
        number_data_point,
    };
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
    use prost::Message;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    const DELTA_START: u64 = 10;
    const CUMULATIVE_START: u64 = 5;
    const COLLECTION_TIME: u64 = 20;

    /// Scenario: Direct protobuf encoding reports that its buffer limit was exceeded.
    /// Guarantees: The diagnostic includes the exact maximum accepted OTLP request size.
    #[test]
    fn request_too_large_error_reports_the_buffer_limit() {
        assert_eq!(
            Error::from(Dropped).to_string(),
            format!(
                "internal telemetry metrics request exceeded the OTLP size limit of \
                 {MAX_OTLP_SIZE_LIMIT} bytes"
            )
        );
    }

    /// Builds a basic-tier distribution value from raw Mmsc fields.
    fn mmsc_value(min: f64, max: f64, sum: f64, count: u64) -> MetricValue {
        MetricValue::from(crate::instrument::Mmsc {
            min,
            max,
            sum,
            count,
        })
    }

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
    }

    fn decode_request(encoded: OtlpProtoBytes) -> ExportMetricsServiceRequest {
        let OtlpProtoBytes::ExportMetricsRequest(bytes) = encoded else {
            panic!("encoder returned the wrong OTLP signal")
        };
        ExportMetricsServiceRequest::decode(bytes).expect("valid metrics request")
    }

    fn protobuf_fields(mut bytes: &[u8]) -> Vec<(u64, u64, &[u8])> {
        fn varint(bytes: &[u8]) -> (u64, usize) {
            let mut value = 0_u64;
            for (index, byte) in bytes.iter().copied().enumerate() {
                value |= u64::from(byte & 0x7f) << (index * 7);
                if byte & 0x80 == 0 {
                    return (value, index + 1);
                }
            }
            panic!("truncated varint")
        }

        let mut fields = Vec::new();
        while !bytes.is_empty() {
            let (key, key_len) = varint(bytes);
            bytes = &bytes[key_len..];
            let wire_type = key & 7;
            let payload_len = match wire_type {
                wire_types::VARINT => varint(bytes).1,
                wire_types::FIXED64 => 8,
                wire_types::LEN => {
                    let (len, prefix_len) = varint(bytes);
                    let len = usize::try_from(len).expect("field length fits usize");
                    let payload = &bytes[prefix_len..prefix_len + len];
                    fields.push((key >> 3, wire_type, payload));
                    bytes = &bytes[prefix_len + len..];
                    continue;
                }
                wire_types::FIXED32 => 4,
                other => panic!("unsupported wire type {other}"),
            };
            let payload = &bytes[..payload_len];
            fields.push((key >> 3, wire_type, payload));
            bytes = &bytes[payload_len..];
        }
        fields
    }

    fn message_field(bytes: &[u8], field_number: u64) -> &[u8] {
        protobuf_fields(bytes)
            .into_iter()
            .find_map(|(number, wire_type, payload)| {
                (number == field_number && wire_type == wire_types::LEN).then_some(payload)
            })
            .unwrap_or_else(|| panic!("missing message field {field_number}"))
    }

    fn metric_data_field_numbers(
        encoded: &OtlpProtoBytes,
        metric_name: &str,
        data_field: u64,
    ) -> Vec<u64> {
        let resource_metrics = message_field(encoded.as_bytes(), METRICS_DATA_RESOURCE_METRICS);
        let scope_metrics = message_field(resource_metrics, RESOURCE_METRICS_SCOPE_METRICS);
        let metric = protobuf_fields(scope_metrics)
            .into_iter()
            .filter_map(|(number, wire_type, payload)| {
                (number == SCOPE_METRICS_METRICS && wire_type == wire_types::LEN).then_some(payload)
            })
            .find(|metric| message_field(metric, METRIC_NAME) == metric_name.as_bytes())
            .unwrap_or_else(|| panic!("missing metric named {metric_name}"));
        protobuf_fields(message_field(metric, data_field))
            .into_iter()
            .map(|(number, _, _)| number)
            .collect()
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

    /// Scenario: No configured view matches a metric field.
    /// Guarantees: The descriptor-defined name, description, and unit are
    /// preserved in the emitted OTLP stream.
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
            MetricsOtlpEncoder::new_with_views(&ResourceLogs::default().encode_to_vec(), views);
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

    /// Scenario: A view selects a scope by several exact scalar attributes.
    /// Guarantees: The view applies only when every configured key, type, and
    /// value matches the metric-set entity.
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
            );
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

    /// Scenario: Several matching views produce case variants of one stream.
    /// Guarantees: One stream is emitted and retains the longest description.
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
            MetricsOtlpEncoder::new_with_views(&ResourceLogs::default().encode_to_vec(), views);
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

    /// Scenario: Views map two fields in one metric set to the same name.
    /// Guarantees: Encoding rejects the ambiguous case-insensitive collision.
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
            MetricsOtlpEncoder::new_with_views(&ResourceLogs::default().encode_to_vec(), views);
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

    /// Scenario: Views map fields from equal effective scopes to one name.
    /// Guarantees: Collision detection spans separate metric-set exports.
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
            MetricsOtlpEncoder::new_with_views(&ResourceLogs::default().encode_to_vec(), views);
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

    /// Scenario: Descriptor-defined fields already differ only by name case.
    /// Guarantees: The no-view path preserves both pre-existing streams.
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

    /// Scenario: Two MMSC snapshots with the same stream identity are coalesced
    /// before OTLP encoding.
    /// Guarantees: The merged summary is emitted as one explicit-boundary
    /// histogram point with exact statistics and no invented buckets.
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
                mmsc_value(1.0, 3.0, 4.0, 2),
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
                mmsc_value(0.0, 5.0, 8.0, 2),
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
        let Some(metric::Data::Histogram(histogram)) = mmsc.data.as_ref() else {
            panic!("expected explicit-boundary histogram metric")
        };
        let point = &histogram.data_points[0];
        assert_eq!(point.min, Some(0.0));
        assert_eq!(point.max, Some(5.0));
        assert_eq!(point.sum, Some(12.0));
        assert_eq!(point.count, 4);
        assert!(point.explicit_bounds.is_empty());
        assert!(point.bucket_counts.is_empty());
    }

    /// Scenario: A delta exponential-histogram distribution field is recorded,
    /// aggregated across two snapshots, then encoded to OTLP.
    /// Guarantees: The merged distribution exports as a single delta
    /// ExponentialHistogram data point whose count/sum/min/max reflect every
    /// recorded observation and whose delta start time is preserved.
    #[test]
    fn encodes_distribution_as_delta_exponential_histogram_point() {
        let attributes = empty_attributes();
        let first_dist = normal_distribution(&[1.0, 2.0, 4.0]);
        let first = metric_set(
            &DISTRIBUTION_ONLY_DESCRIPTOR,
            attributes.clone(),
            vec![MetricValue::from(first_dist)],
        );

        let second_dist = normal_distribution(&[8.0]);
        let second = metric_set(
            &DISTRIBUTION_ONLY_DESCRIPTOR,
            attributes,
            vec![MetricValue::from(second_dist)],
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

    /// Scenario: A direct-encoded exponential histogram uses zero timestamps,
    /// packed buckets, a negative integer attribute, and a nested map.
    /// Guarantees: Scalar defaults are absent, recursive values decode, bucket
    /// counts use one packed field, and zero plus positive counts equal count.
    #[test]
    fn direct_wire_encoding_omits_defaults_and_packs_buckets() {
        let nested = BTreeMap::from([(
            "inner".to_owned(),
            AttributeValue::Map(BTreeMap::from([(
                "delta".to_owned(),
                AttributeValue::Int(-9),
            )])),
        )]);
        let attributes = shared_attributes(
            &FULL_ATTRIBUTES_DESCRIPTOR,
            vec![
                AttributeValue::String("worker-a".to_owned()),
                AttributeValue::Int(-2),
                AttributeValue::UInt(7),
                AttributeValue::Double(0.75),
                AttributeValue::Boolean(false),
                AttributeValue::Map(nested),
            ],
        );
        let mut metric_set = metric_set(
            &DISTRIBUTION_ONLY_DESCRIPTOR,
            attributes,
            vec![MetricValue::from(normal_distribution(&[0.0, 1.0, 4.0]))],
        );
        metric_set.delta_start_time_unix_nano = 0;
        let batch = MetricExportBatch {
            time_unix_nano: 0,
            metric_sets: vec![metric_set],
        };

        let encoded = empty_resource_encoder()
            .encode(&batch)
            .expect("direct encoding succeeds")
            .expect("distribution produces a request");
        let direct_bytes = encoded.as_bytes().to_vec();
        let request = decode_request(encoded);

        let scope = only_scope(&request);
        let delta = scope
            .scope
            .as_ref()
            .expect("scope")
            .attributes
            .iter()
            .find(|attribute| attribute.key == "worker.delta")
            .and_then(|attribute| attribute.value.as_ref())
            .and_then(|value| value.value.as_ref());
        assert_eq!(delta, Some(&any_value::Value::IntValue(-2)));
        let nested_delta = scope
            .scope
            .as_ref()
            .expect("scope")
            .attributes
            .iter()
            .find(|attribute| attribute.key == "worker.labels")
            .and_then(|attribute| attribute.value.as_ref())
            .and_then(|value| value.value.as_ref())
            .and_then(|value| match value {
                any_value::Value::KvlistValue(values) => values.values.first(),
                _ => None,
            })
            .and_then(|attribute| attribute.value.as_ref())
            .and_then(|value| value.value.as_ref())
            .and_then(|value| match value {
                any_value::Value::KvlistValue(values) => values.values.first(),
                _ => None,
            })
            .and_then(|attribute| attribute.value.as_ref())
            .and_then(|value| value.value.as_ref());
        assert_eq!(nested_delta, Some(&any_value::Value::IntValue(-9)));

        let Some(metric::Data::ExponentialHistogram(histogram)) =
            metric_named(scope, "histogram.distribution").data.as_ref()
        else {
            panic!("expected exponential histogram")
        };
        let positive = histogram.data_points[0]
            .positive
            .as_ref()
            .expect("positive buckets");
        assert!(!positive.bucket_counts.is_empty());
        assert_eq!(histogram.data_points[0].zero_count, 1);
        assert_eq!(
            histogram.data_points[0].count,
            histogram.data_points[0].zero_count + positive.bucket_counts.iter().sum::<u64>()
        );

        let resource_metrics = message_field(&direct_bytes, METRICS_DATA_RESOURCE_METRICS);
        let scope_metrics = message_field(resource_metrics, RESOURCE_METRICS_SCOPE_METRICS);
        let metric = message_field(scope_metrics, SCOPE_METRICS_METRICS);
        let exponential_histogram = message_field(metric, METRIC_EXPONENTIAL_HISTOGRAM);
        let point = message_field(exponential_histogram, EXPONENTIAL_HISTOGRAM_DATA_POINTS);
        let point_fields = protobuf_fields(point);
        assert!(
            point_fields
                .iter()
                .all(|(number, _, _)| !matches!(*number, 2 | 3))
        );
        let positive = message_field(point, EXP_HISTOGRAM_DP_POSITIVE);
        let bucket_fields = protobuf_fields(positive)
            .into_iter()
            .filter(|(number, _, _)| *number == EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS)
            .collect::<Vec<_>>();
        assert_eq!(bucket_fields.len(), 1);
        assert_eq!(bucket_fields[0].1, wire_types::LEN);
    }

    /// Scenario: Two bucket-local gauge values share a scope and stream but
    /// have distinct item attributes.
    /// Guarantees: Direct encoding emits one metric with two attributed data
    /// points instead of merging values or duplicating the metric stream.
    #[test]
    fn merges_distinct_item_attribute_buckets_into_multiple_stream_points() {
        let attributes = empty_attributes();
        let mut first = metric_set(
            &F64_GAUGE_DESCRIPTOR,
            attributes.clone(),
            vec![MetricValue::F64(1.5)],
        );
        first.item_attributes = vec![("bucket".to_owned(), "first".to_owned())];
        let mut second = metric_set(
            &F64_GAUGE_DESCRIPTOR,
            attributes,
            vec![MetricValue::F64(2.5)],
        );
        second.item_attributes = vec![("bucket".to_owned(), "second".to_owned())];
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![first, second],
        };

        let request = decode_request(
            empty_resource_encoder()
                .encode(&batch)
                .expect("bucketed gauges encode")
                .expect("bucketed gauges produce a request"),
        );
        let scope = only_scope(&request);
        let [metric] = scope.metrics.as_slice() else {
            panic!("expected one merged metric stream")
        };
        let Some(metric::Data::Gauge(gauge)) = metric.data.as_ref() else {
            panic!("expected gauge data")
        };
        assert_eq!(gauge.data_points.len(), 2);
        assert_eq!(gauge.data_points[0].attributes[0].key, "bucket");
        assert_eq!(gauge.data_points[1].attributes[0].key, "bucket");
    }

    /// Scenario: Every supported instrument kind is encoded in one batch.
    /// Guarantees: Values, timing, temporality, monotonicity, saturation, and
    /// bucketless MMSC statistics retain their OTLP semantics.
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
                    mmsc_value(2.0, 9.0, 20.0, 4),
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
        assert_eq!(scope.metrics.len(), 5);

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

        let mmsc = metric_named(scope, "histogram.mmsc");
        let Some(metric::Data::Histogram(histogram)) = mmsc.data.as_ref() else {
            panic!("expected MMSC explicit-boundary histogram metric")
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
        assert!(point.explicit_bounds.is_empty());
        assert!(point.bucket_counts.is_empty());
    }

    /// Scenario: Direct encoding emits aggregation metadata and two ordered data points for sum
    /// and explicit-boundary histogram messages.
    /// Guarantees: Metadata physically precedes repeated data points, false monotonicity remains
    /// omitted, and repeated data-point order is preserved.
    #[test]
    fn encodes_sum_and_histogram_metadata_before_ordered_data_points() {
        let attributes = empty_attributes();
        let mut first = metric_set(
            &ALL_METRICS_DESCRIPTOR,
            attributes.clone(),
            vec![
                MetricValue::U64(1),
                MetricValue::U64(2),
                MetricValue::F64(-1.0),
                MetricValue::F64(3.0),
                mmsc_value(1.0, 2.0, 3.0, 2),
            ],
        );
        first.item_attributes = vec![("bucket".to_owned(), "first".to_owned())];
        let mut second = metric_set(
            &ALL_METRICS_DESCRIPTOR,
            attributes.clone(),
            vec![
                MetricValue::U64(4),
                MetricValue::U64(5),
                MetricValue::F64(-2.0),
                MetricValue::F64(6.0),
                mmsc_value(3.0, 4.0, 7.0, 2),
            ],
        );
        second.item_attributes = vec![("bucket".to_owned(), "second".to_owned())];
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![first, second],
        };

        let encoded = empty_resource_encoder()
            .encode(&batch)
            .expect("mixed metrics batch should encode")
            .expect("mixed metrics batch should produce a request");

        assert_eq!(
            metric_data_field_numbers(&encoded, "counter.delta", METRIC_SUM),
            vec![
                SUM_AGGREGATION_TEMPORALITY,
                SUM_IS_MONOTONIC,
                SUM_DATA_POINTS,
                SUM_DATA_POINTS,
            ]
        );
        assert_eq!(
            metric_data_field_numbers(&encoded, "up_down.delta", METRIC_SUM),
            vec![
                SUM_AGGREGATION_TEMPORALITY,
                SUM_DATA_POINTS,
                SUM_DATA_POINTS,
            ]
        );
        assert_eq!(
            metric_data_field_numbers(&encoded, "histogram.mmsc", METRIC_HISTOGRAM),
            vec![
                HISTOGRAM_AGGREGATION_TEMPORALITY,
                HISTOGRAM_DATA_POINTS,
                HISTOGRAM_DATA_POINTS,
            ]
        );

        let request = decode_request(encoded);
        let Some(metric::Data::Sum(sum)) = metric_named(only_scope(&request), "counter.delta")
            .data
            .as_ref()
        else {
            panic!("expected sum metric")
        };
        assert_eq!(sum.data_points.len(), 2);
        assert_eq!(
            sum.data_points[0].attributes[0].value,
            Some(AnyValue::new_string("first"))
        );
        assert_eq!(
            sum.data_points[1].attributes[0].value,
            Some(AnyValue::new_string("second"))
        );
    }

    /// Scenario: Direct encoding emits aggregation metadata and two data points for an
    /// exponential-histogram message.
    /// Guarantees: Exponential-histogram temporality physically precedes every repeated data
    /// point.
    #[test]
    fn encodes_exponential_histogram_metadata_before_data_points() {
        let attributes = empty_attributes();
        let mut first = metric_set(
            &DISTRIBUTION_ONLY_DESCRIPTOR,
            attributes.clone(),
            vec![MetricValue::from(normal_distribution(&[1.0, 2.0]))],
        );
        first.item_attributes = vec![("bucket".to_owned(), "first".to_owned())];
        let mut second = metric_set(
            &DISTRIBUTION_ONLY_DESCRIPTOR,
            attributes,
            vec![MetricValue::from(normal_distribution(&[4.0, 8.0]))],
        );
        second.item_attributes = vec![("bucket".to_owned(), "second".to_owned())];
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![first, second],
        };
        let encoded = empty_resource_encoder()
            .encode(&batch)
            .expect("exponential histogram batch should encode")
            .expect("exponential histogram batch should produce a request");

        assert_eq!(
            metric_data_field_numbers(
                &encoded,
                "histogram.distribution",
                METRIC_EXPONENTIAL_HISTOGRAM,
            ),
            vec![
                EXPONENTIAL_HISTOGRAM_AGGREGATION_TEMPORALITY,
                EXPONENTIAL_HISTOGRAM_DATA_POINTS,
                EXPONENTIAL_HISTOGRAM_DATA_POINTS,
            ]
        );
    }

    /// Scenario: A batch contains populated and empty MMSC fields and sets.
    /// Guarantees: Empty distributions are omitted without suppressing other
    /// metrics or populated scopes.
    #[test]
    fn emits_multiple_scopes_while_omitting_empty_mmsc_fields_and_sets() {
        let empty_mmsc = mmsc_value(0.0, 0.0, 0.0, 0);
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
                    vec![mmsc_value(2.0, 8.0, 10.0, 2)],
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
        assert_eq!(first.metrics.len(), 4);
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

    /// Scenario: Dirty scalar instruments contain numeric zero values.
    /// Guarantees: Their oneof values remain present rather than being omitted
    /// as protobuf defaults.
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
                    mmsc_value(0.0, 0.0, 0.0, 0),
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
        assert_eq!(scope.metrics.len(), 4);

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
    }

    /// Scenario: An MMSC summary contains a negative minimum.
    /// Guarantees: Its explicit-boundary histogram point retains count/min/max
    /// but omits the undefined OTLP histogram sum and invents no buckets.
    #[test]
    fn omits_mmsc_sum_when_the_population_contains_negative_values() {
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &MMSC_ONLY_DESCRIPTOR,
                empty_attributes(),
                vec![mmsc_value(-2.0, 8.0, 6.0, 2)],
            )],
        };
        let request = decode_request(
            empty_resource_encoder()
                .encode(&batch)
                .expect("negative MMSC values are valid")
                .expect("MMSC produces a request"),
        );
        let metric = metric_named(only_scope(&request), "histogram.empty");
        let Some(metric::Data::Histogram(histogram)) = metric.data.as_ref() else {
            panic!("expected explicit-boundary histogram metric")
        };
        let [point] = histogram.data_points.as_slice() else {
            panic!("expected one histogram data point")
        };
        assert_eq!(point.sum, None);
        assert_eq!(point.min, Some(-2.0));
        assert_eq!(point.max, Some(8.0));
        assert_eq!(point.count, 2);
        assert!(point.explicit_bounds.is_empty());
        assert!(point.bucket_counts.is_empty());
    }

    /// Scenario: An empty batch or an all-empty MMSC batch is encoded.
    /// Guarantees: Neither case emits an empty OTLP request.
    #[test]
    fn omits_empty_mmsc_and_empty_batches() {
        let encoder = empty_resource_encoder();
        let empty_mmsc = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &MMSC_ONLY_DESCRIPTOR,
                empty_attributes(),
                vec![mmsc_value(0.0, 0.0, 0.0, 0)],
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

    /// Scenario: Resource and entity attributes include every internal value
    /// type, saturation, and a map.
    /// Guarantees: The trusted resource fragment and native scope attribute
    /// values survive direct wire encoding.
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
        let encoder = MetricsOtlpEncoder::new(&fragment);

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
                vec![mmsc_value(1.0, 1.0, 1.0, 1)],
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

    /// Scenario: A metric set supplies too few or too many values.
    /// Guarantees: Encoding rejects the batch instead of truncating the
    /// descriptor/value pairing.
    #[test]
    fn rejects_metric_set_value_count_mismatches() {
        for actual in [0, 2] {
            let batch = MetricExportBatch {
                time_unix_nano: COLLECTION_TIME,
                metric_sets: vec![metric_set(
                    &MMSC_ONLY_DESCRIPTOR,
                    empty_attributes(),
                    vec![mmsc_value(1.0, 1.0, 1.0, 1); actual],
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

    /// Scenario: Metric descriptors are paired with scalar, MMSC, and
    /// exponential-histogram values of the wrong kind or tier.
    /// Guarantees: Encoding rejects every mismatch before selecting an OTLP
    /// point representation.
    #[test]
    fn rejects_metric_value_kind_mismatches() {
        let normal = normal_distribution(&[1.0]);
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
            (
                &MMSC_ONLY_DESCRIPTOR,
                MetricValue::from(normal),
                "histogram.empty",
                "mmsc",
                "exponential histogram",
            ),
            (
                &DISTRIBUTION_ONLY_DESCRIPTOR,
                mmsc_value(1.0, 1.0, 1.0, 1),
                "histogram.distribution",
                "exponential histogram",
                "mmsc",
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

    /// Scenario: A sum-like descriptor omits aggregation temporality.
    /// Guarantees: Encoding reports the missing semantic requirement.
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

    /// Scenario: Directly encoded metrics enter the OTAP conversion path.
    /// Guarantees: Resources, scopes, attributes, and metric data remain
    /// consumable after conversion to Arrow records.
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
        );
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
                    mmsc_value(2.0, 9.0, 20.0, 4),
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
