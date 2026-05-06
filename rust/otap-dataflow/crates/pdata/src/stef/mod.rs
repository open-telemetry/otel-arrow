// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Minimal STEF metrics codec compatible with the Splunk/Collector STEF metrics schema.
//!
//! The implementation intentionally starts with the metrics profile used by the Go Collector
//! `stefreceiver` and `stefexporter`. It writes standard STEF fixed/variable headers, column
//! frames, and the generated `otel.stef` metrics wire schema. The encoder currently supports
//! gauge and sum number data points plus simple OTLP attribute values.

use crate::OtapArrowRecords;
use crate::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor};
use crate::encode::record::{
    attributes::{AttributesRecordBatchBuilder, AttributesRecordBatchBuilderConstructorHelper},
    metrics::MetricsRecordBatchBuilder,
};
use crate::otap::Metrics;
use crate::otlp::attributes::{
    Attribute16Arrays, Attribute32Arrays, AttributeArrays, AttributeValueType,
};
use crate::otlp::common::{
    AnyValueArrays, BatchSorter, ChildIndexIter, ResourceArrays, ScopeArrays, SortedBatchCursor,
};
use crate::otlp::metrics::{MetricType, MetricsArrays, data_points::number::NumberDpArrays};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::{FieldExt, consts};
use arrow::{
    array::{
        Array, ArrayRef, ArrowPrimitiveType, BinaryArray, BooleanArray, Float64Array, Int64Array,
        RecordBatch, StringArray, TimestampNanosecondArray, UInt8Array, UInt16Array, UInt32Array,
    },
    datatypes::{Field, Schema},
    error::ArrowError,
};
use otap_df_pdata_views::views::{
    common::{AnyValueView, AttributeView, InstrumentationScopeView, Str, ValueType},
    metrics::{
        self as metrics_view, DataView, GaugeView, MetricView, MetricsView, NumberDataPointView,
        ResourceMetricsView, ScopeMetricsView, SumView,
    },
    resource::ResourceView,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

/// Root struct name used by the STEF metrics schema.
pub const METRICS_ROOT_STRUCT_NAME: &str = "Metrics";

/// Wire schema serialized by the Go-generated `otelstef.MetricsWireSchema()`.
pub const METRICS_WIRE_SCHEMA: &[u8] = &[
    0x0F, 0x06, 0x01, 0x08, 0x07, 0x03, 0x05, 0x04, 0x05, 0x05, 0x09, 0x02, 0x03, 0x02, 0x05, 0x02,
];

const HDR_SIGNATURE: &[u8; 4] = b"STEF";
const HDR_FORMAT_VERSION: u8 = 0;
const COMPRESSION_NONE: u8 = 0;
const FRAME_FLAG_NONE: u8 = 0;

const METRIC_FIELD_MASK: u64 = 0b11111111;
const RESOURCE_FIELD_MASK: u64 = 0b111;
const SCOPE_FIELD_MASK: u64 = 0b11111;
const ROOT_METRIC_FIELD: u64 = 1 << 1;
const ROOT_RESOURCE_FIELD: u64 = 1 << 2;
const ROOT_SCOPE_FIELD: u64 = 1 << 3;
const ROOT_ATTRIBUTES_FIELD: u64 = 1 << 4;
const ROOT_POINT_FIELD: u64 = 1 << 5;

/// Errors returned by the STEF metrics codec.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// The OTLP metric type is not implemented by this codec slice.
    UnsupportedMetricType(&'static str),
    /// The OTLP data point has an unsupported value shape.
    UnsupportedDataPointValue,
    /// Attribute values are limited to scalar and bytes values in this first implementation.
    UnsupportedAttributeValue(&'static str),
    /// The STEF stream header is invalid or unsupported.
    InvalidHeader(&'static str),
    /// The STEF frame is malformed.
    InvalidFrame(&'static str),
    /// The STEF stream ended before the expected bytes were available.
    UnexpectedEof,
    /// A STEF dictionary reference points outside the active dictionary.
    InvalidRefNum,
    /// The STEF stream uses a value kind outside this implementation slice.
    UnsupportedStefValue(&'static str),
    /// A STEF frame or integer exceeded the supported range.
    ValueOutOfRange(&'static str),
    /// An OTAP Arrow view could not be built from the supplied payload.
    OtapView(String),
    /// OTAP Arrow encoding failed while building a direct decoded payload.
    OtapEncode(String),
    /// A view exposed non-UTF-8 bytes for a STEF string field.
    InvalidUtf8(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedMetricType(kind) => write!(f, "unsupported STEF metric type: {kind}"),
            Self::UnsupportedDataPointValue => write!(f, "unsupported STEF data point value"),
            Self::UnsupportedAttributeValue(kind) => {
                write!(f, "unsupported STEF attribute value: {kind}")
            }
            Self::InvalidHeader(reason) => write!(f, "invalid STEF header: {reason}"),
            Self::InvalidFrame(reason) => write!(f, "invalid STEF frame: {reason}"),
            Self::UnexpectedEof => write!(f, "unexpected end of STEF stream"),
            Self::InvalidRefNum => write!(f, "invalid STEF dictionary reference"),
            Self::UnsupportedStefValue(kind) => write!(f, "unsupported STEF value: {kind}"),
            Self::ValueOutOfRange(name) => write!(f, "STEF value out of range: {name}"),
            Self::OtapView(reason) => write!(f, "OTAP view error: {reason}"),
            Self::OtapEncode(reason) => write!(f, "OTAP encode error: {reason}"),
            Self::InvalidUtf8(field) => write!(f, "invalid UTF-8 in STEF string field: {field}"),
        }
    }
}

impl std::error::Error for Error {}

/// Encodes a metrics view directly into a complete STEF metrics byte stream.
pub fn encode_metrics_view<T>(metrics: &T) -> Result<Vec<u8>, Error>
where
    T: MetricsView,
{
    encode_metrics_view_with_count(metrics).map(|(bytes, _)| bytes)
}

/// Encodes a metrics view directly into STEF and returns the encoded STEF record count.
pub fn encode_metrics_view_with_count<T>(metrics: &T) -> Result<(Vec<u8>, u64), Error>
where
    T: MetricsView,
{
    let mut encoder = MetricsStreamEncoder::default();
    encoder.encode_metrics_view(metrics)?;
    let record_count = encoder.record_count;
    let bytes = encoder.finish()?;
    Ok((bytes, record_count))
}

/// Encodes OTAP metrics Arrow records directly into a complete STEF metrics byte stream.
pub fn encode_metrics_otap(records: &OtapArrowRecords) -> Result<Vec<u8>, Error> {
    encode_metrics_otap_with_count(records).map(|(bytes, _)| bytes)
}

/// Encodes OTAP metrics Arrow records directly into STEF and returns the encoded record count.
pub fn encode_metrics_otap_with_count(records: &OtapArrowRecords) -> Result<(Vec<u8>, u64), Error> {
    let mut encoder = MetricsStreamEncoder::default();
    encoder.encode_metrics_otap(records)?;
    let record_count = encoder.record_count;
    let bytes = encoder.finish()?;
    Ok((bytes, record_count))
}

/// Decodes a complete STEF metrics byte stream directly into OTAP metrics Arrow records.
pub fn decode_metrics_otap(bytes: &[u8]) -> Result<OtapArrowRecords, Error> {
    decode_metrics_otap_with_count(bytes).map(|(records, _)| records)
}

/// Decodes a complete STEF metrics byte stream directly into OTAP records and a STEF record count.
pub fn decode_metrics_otap_with_count(bytes: &[u8]) -> Result<(OtapArrowRecords, u64), Error> {
    MetricsOtapStreamDecoder::decode(bytes)
}

struct MetricsStreamEncoder {
    record_count: u64,
    root_bits: BitWriter,
    metric: MetricEncoder,
    resource: ResourceEncoder,
    scope: ScopeEncoder,
    attributes: AttributesEncoder,
    point: PointEncoder,
}

impl Default for MetricsStreamEncoder {
    fn default() -> Self {
        let schema_url = SharedStringDict::default();
        let attribute_key = SharedStringDict::default();
        let any_value_string = SharedStringDict::default();

        Self {
            record_count: 0,
            root_bits: BitWriter::default(),
            metric: MetricEncoder::new(
                SharedStringDict::default(),
                SharedStringDict::default(),
                SharedStringDict::default(),
                attribute_key.clone(),
                any_value_string.clone(),
            ),
            resource: ResourceEncoder::new(
                schema_url.clone(),
                attribute_key.clone(),
                any_value_string.clone(),
            ),
            scope: ScopeEncoder::new(
                SharedStringDict::default(),
                SharedStringDict::default(),
                schema_url,
                attribute_key.clone(),
                any_value_string.clone(),
            ),
            attributes: AttributesEncoder::new(attribute_key, any_value_string),
            point: PointEncoder::default(),
        }
    }
}

impl MetricsStreamEncoder {
    fn encode_metrics_view<T>(&mut self, metrics: &T) -> Result<(), Error>
    where
        T: MetricsView,
    {
        for resource_metrics in metrics.resources() {
            let resource_schema_url = resource_metrics.schema_url();
            let resource = resource_metrics.resource();
            let mut wrote_resource = false;
            for scope_metrics in resource_metrics.scopes() {
                let scope_schema_url = scope_metrics.schema_url();
                let scope = scope_metrics.scope();
                let mut wrote_scope = false;
                for metric in scope_metrics.metrics() {
                    let data = metric.data().ok_or(Error::UnsupportedMetricType("empty"))?;
                    match data.value_type() {
                        metrics_view::DataType::Gauge => {
                            let gauge = data
                                .as_gauge()
                                .ok_or(Error::UnsupportedMetricType("gauge"))?;
                            let metric = ViewMetric {
                                metric: &metric,
                                r#type: 0,
                                aggregation_temporality: 0,
                                monotonic: false,
                            };
                            let wrote = self.encode_number_points_view(
                                resource_schema_url,
                                resource.as_ref(),
                                scope_schema_url,
                                scope.as_ref(),
                                &metric,
                                gauge.data_points(),
                                !wrote_resource,
                                !wrote_scope,
                            )?;
                            wrote_resource |= wrote;
                            wrote_scope |= wrote;
                        }
                        metrics_view::DataType::Sum => {
                            let sum = data.as_sum().ok_or(Error::UnsupportedMetricType("sum"))?;
                            let metric = ViewMetric {
                                metric: &metric,
                                r#type: 1,
                                aggregation_temporality: sum.aggregation_temporality() as u64,
                                monotonic: sum.is_monotonic(),
                            };
                            let wrote = self.encode_number_points_view(
                                resource_schema_url,
                                resource.as_ref(),
                                scope_schema_url,
                                scope.as_ref(),
                                &metric,
                                sum.data_points(),
                                !wrote_resource,
                                !wrote_scope,
                            )?;
                            wrote_resource |= wrote;
                            wrote_scope |= wrote;
                        }
                        metrics_view::DataType::Histogram => {
                            return Err(Error::UnsupportedMetricType("histogram"));
                        }
                        metrics_view::DataType::ExponentialHistogram => {
                            return Err(Error::UnsupportedMetricType("exponential_histogram"));
                        }
                        metrics_view::DataType::Summary => {
                            return Err(Error::UnsupportedMetricType("summary"));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn encode_number_points_view<R, S, M, P>(
        &mut self,
        resource_schema_url: Option<Str<'_>>,
        resource: Option<&R>,
        scope_schema_url: Str<'_>,
        scope: Option<&S>,
        metric: &ViewMetric<'_, M>,
        points: impl Iterator<Item = P>,
        mut encode_resource: bool,
        mut encode_scope: bool,
    ) -> Result<bool, Error>
    where
        R: ResourceView,
        S: InstrumentationScopeView,
        M: MetricView,
        P: NumberDataPointView,
    {
        let mut wrote = false;
        let mut encode_metric = true;
        for point in points {
            let point_value = match point.value() {
                Some(metrics_view::Value::Integer(value)) => StefPointValue::Int64(value),
                Some(metrics_view::Value::Double(value)) => StefPointValue::Float64(value),
                None if point.flags().no_recorded_value() => StefPointValue::None,
                None => return Err(Error::UnsupportedDataPointValue),
            };
            let mut root_mask = ROOT_ATTRIBUTES_FIELD | ROOT_POINT_FIELD;
            if encode_metric {
                root_mask |= ROOT_METRIC_FIELD;
            }
            if encode_resource {
                root_mask |= ROOT_RESOURCE_FIELD;
            }
            if encode_scope {
                root_mask |= ROOT_SCOPE_FIELD;
            }

            self.root_bits.write_bits(root_mask, 6);
            if encode_metric {
                self.metric.encode_view(metric)?;
            }
            if encode_resource {
                self.resource.encode_view(resource_schema_url, resource)?;
            }
            if encode_scope {
                self.scope.encode_view(scope_schema_url, scope)?;
            }
            self.attributes.encode_view(point.attributes())?;
            self.point.encode(&StefPoint {
                start_timestamp: point.start_time_unix_nano(),
                timestamp: point.time_unix_nano(),
                value: point_value,
            })?;
            self.record_count += 1;
            wrote = true;
            encode_metric = false;
            encode_resource = false;
            encode_scope = false;
        }
        Ok(wrote)
    }

    fn encode_metrics_otap(&mut self, records: &OtapArrowRecords) -> Result<(), Error> {
        let arrays = OtapMetricsDirectArrays::try_from(records)?;
        let mut cursors = OtapMetricsDirectCursors::new();
        cursors.init(&arrays)?;

        while !cursors.root.finished() {
            self.encode_otap_resource_metrics(&arrays, &mut cursors)?;
        }

        Ok(())
    }

    fn encode_otap_resource_metrics(
        &mut self,
        arrays: &OtapMetricsDirectArrays<'_>,
        cursors: &mut OtapMetricsDirectCursors,
    ) -> Result<(), Error> {
        let first_row = match cursors.root.curr_index() {
            Some(row) => row,
            None => return Ok(()),
        };
        let resource_id = arrays.resource.id.value_at(first_row);
        let mut wrote_resource = false;

        loop {
            self.encode_otap_scope_metrics(arrays, cursors, resource_id, &mut wrote_resource)?;

            if cursors.root.finished() {
                break;
            }

            let next_row = cursors.root.curr_index().expect("cursor not finished");
            if arrays.resource.id.value_at(next_row) != resource_id {
                break;
            }
        }

        Ok(())
    }

    fn encode_otap_scope_metrics(
        &mut self,
        arrays: &OtapMetricsDirectArrays<'_>,
        cursors: &mut OtapMetricsDirectCursors,
        resource_id: Option<u16>,
        wrote_resource: &mut bool,
    ) -> Result<(), Error> {
        let first_row = match cursors.root.curr_index() {
            Some(row) => row,
            None => return Ok(()),
        };
        let scope_id = arrays.scope.id.value_at(first_row);
        let mut wrote_scope = false;

        loop {
            let wrote = self.encode_otap_metric(arrays, cursors, !*wrote_resource, !wrote_scope)?;
            if wrote {
                *wrote_resource = true;
                wrote_scope = true;
            }

            if cursors.root.finished() {
                break;
            }

            let next_row = cursors.root.curr_index().expect("cursor not finished");
            if arrays.resource.id.value_at(next_row) != resource_id
                || arrays.scope.id.value_at(next_row) != scope_id
            {
                break;
            }
        }

        Ok(())
    }

    fn encode_otap_metric(
        &mut self,
        arrays: &OtapMetricsDirectArrays<'_>,
        cursors: &mut OtapMetricsDirectCursors,
        encode_resource: bool,
        encode_scope: bool,
    ) -> Result<bool, Error> {
        let metric_row = cursors
            .root
            .curr_index()
            .ok_or(Error::UnsupportedMetricType("empty"))?;
        let metric_type = otap_metric_type(arrays, metric_row)?;
        let metric_id = arrays
            .metrics
            .id
            .value_at(metric_row)
            .ok_or(Error::UnsupportedMetricType("empty"))?;

        let wrote = match metric_type {
            MetricType::Gauge => self.encode_otap_number_points(
                arrays,
                cursors,
                metric_row,
                metric_id,
                0,
                0,
                false,
                encode_resource,
                encode_scope,
            )?,
            MetricType::Sum => self.encode_otap_number_points(
                arrays,
                cursors,
                metric_row,
                metric_id,
                1,
                otap_aggregation_temporality(arrays, metric_row),
                otap_is_monotonic(arrays, metric_row),
                encode_resource,
                encode_scope,
            )?,
            MetricType::Empty => return Err(Error::UnsupportedMetricType("empty")),
            MetricType::Histogram => return Err(Error::UnsupportedMetricType("histogram")),
            MetricType::ExponentialHistogram => {
                return Err(Error::UnsupportedMetricType("exponential_histogram"));
            }
            MetricType::Summary => return Err(Error::UnsupportedMetricType("summary")),
        };

        cursors.root.advance();
        Ok(wrote)
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_otap_number_points(
        &mut self,
        arrays: &OtapMetricsDirectArrays<'_>,
        cursors: &mut OtapMetricsDirectCursors,
        metric_row: usize,
        metric_id: u16,
        metric_type: u64,
        aggregation_temporality: u64,
        monotonic: bool,
        mut encode_resource: bool,
        mut encode_scope: bool,
    ) -> Result<bool, Error> {
        let Some(number_dp) = arrays.number_dp.as_ref() else {
            return Ok(false);
        };

        let parent_ids = MaybeDictArrayAccessor::Native(number_dp.parent_id);
        let mut wrote = false;
        let mut encode_metric = true;

        loop {
            let dp_row = ChildIndexIter::new(metric_id, &parent_ids, &mut cursors.number_dp).next();
            let Some(dp_row) = dp_row else {
                break;
            };

            let mut root_mask = ROOT_ATTRIBUTES_FIELD | ROOT_POINT_FIELD;
            if encode_metric {
                root_mask |= ROOT_METRIC_FIELD;
            }
            if encode_resource {
                root_mask |= ROOT_RESOURCE_FIELD;
            }
            if encode_scope {
                root_mask |= ROOT_SCOPE_FIELD;
            }

            self.root_bits.write_bits(root_mask, 6);
            if encode_metric {
                self.metric.encode_otap(
                    metric_row,
                    &arrays.metrics,
                    arrays.metric_attrs.as_ref(),
                    &mut cursors.metric_attrs,
                    metric_type,
                    aggregation_temporality,
                    monotonic,
                )?;
            }
            if encode_resource {
                self.resource.encode_otap(
                    metric_row,
                    &arrays.resource,
                    arrays.resource_attrs.as_ref(),
                    &mut cursors.resource_attrs,
                )?;
            }
            if encode_scope {
                self.scope.encode_otap(
                    metric_row,
                    &arrays.scope,
                    otap_scope_schema_url(arrays, metric_row),
                    arrays.scope_attrs.as_ref(),
                    &mut cursors.scope_attrs,
                )?;
            }

            self.attributes.encode_otap_attributes(
                arrays.number_dp_attrs.as_ref(),
                number_dp.id.value_at(dp_row),
                &mut cursors.number_dp_attrs,
            )?;
            self.point.encode_otap_number_point(number_dp, dp_row)?;
            self.record_count += 1;
            wrote = true;
            encode_metric = false;
            encode_resource = false;
            encode_scope = false;
        }

        Ok(wrote)
    }

    fn finish(mut self) -> Result<Vec<u8>, Error> {
        let mut stream = Vec::with_capacity(16 * 1024);
        write_fixed_header(&mut stream);
        write_var_header_frame(&mut stream);

        if self.record_count == 0 {
            return Ok(stream);
        }

        let mut content = Vec::with_capacity(16 * 1024);
        append_uvarint(self.record_count, &mut content);

        let columns = self.collect_columns();
        write_columns(&columns, &mut content)?;
        write_frame(&mut stream, FRAME_FLAG_NONE, &content);
        Ok(stream)
    }

    fn collect_columns(&mut self) -> Column {
        let mut root = metrics_column_tree();
        root.data = self.root_bits.take_bytes();
        root.children[1] = self.metric.take_column();
        root.children[2] = self.resource.take_column();
        root.children[3] = self.scope.take_column();
        root.children[4] = self.attributes.take_column();
        root.children[5] = self.point.take_column();
        root
    }
}

struct OtapMetricsDirectArrays<'a> {
    root: &'a RecordBatch,
    metrics: MetricsArrays<'a>,
    resource: ResourceArrays<'a>,
    scope: ScopeArrays<'a>,
    resource_attrs: Option<Attribute16Arrays<'a>>,
    scope_attrs: Option<Attribute16Arrays<'a>>,
    metric_attrs: Option<Attribute16Arrays<'a>>,
    number_dp: Option<NumberDpArrays<'a>>,
    number_dp_attrs: Option<Attribute32Arrays<'a>>,
}

impl<'a> TryFrom<&'a OtapArrowRecords> for OtapMetricsDirectArrays<'a> {
    type Error = Error;

    fn try_from(records: &'a OtapArrowRecords) -> Result<Self, Self::Error> {
        let root = records
            .get(ArrowPayloadType::UnivariateMetrics)
            .or_else(|| records.get(ArrowPayloadType::MultivariateMetrics))
            .ok_or_else(|| Error::OtapView("metric record not found".to_owned()))?;

        Ok(Self {
            root,
            metrics: MetricsArrays::try_from(root).map_err(map_otap_view_error)?,
            resource: ResourceArrays::try_from(root).map_err(map_otap_view_error)?,
            scope: ScopeArrays::try_from(root).map_err(map_otap_view_error)?,
            resource_attrs: records
                .get(ArrowPayloadType::ResourceAttrs)
                .map(Attribute16Arrays::try_from)
                .transpose()
                .map_err(map_otap_view_error)?,
            scope_attrs: records
                .get(ArrowPayloadType::ScopeAttrs)
                .map(Attribute16Arrays::try_from)
                .transpose()
                .map_err(map_otap_view_error)?,
            metric_attrs: records
                .get(ArrowPayloadType::MetricAttrs)
                .map(Attribute16Arrays::try_from)
                .transpose()
                .map_err(map_otap_view_error)?,
            number_dp: records
                .get(ArrowPayloadType::NumberDataPoints)
                .map(NumberDpArrays::try_from)
                .transpose()
                .map_err(map_otap_view_error)?,
            number_dp_attrs: records
                .get(ArrowPayloadType::NumberDpAttrs)
                .map(Attribute32Arrays::try_from)
                .transpose()
                .map_err(map_otap_view_error)?,
        })
    }
}

struct OtapMetricsDirectCursors {
    batch_sorter: BatchSorter,
    root: SortedBatchCursor,
    resource_attrs: SortedBatchCursor,
    scope_attrs: SortedBatchCursor,
    metric_attrs: SortedBatchCursor,
    number_dp: SortedBatchCursor,
    number_dp_attrs: SortedBatchCursor,
}

impl OtapMetricsDirectCursors {
    fn new() -> Self {
        Self {
            batch_sorter: BatchSorter::new(),
            root: SortedBatchCursor::new(),
            resource_attrs: SortedBatchCursor::new(),
            scope_attrs: SortedBatchCursor::new(),
            metric_attrs: SortedBatchCursor::new(),
            number_dp: SortedBatchCursor::new(),
            number_dp_attrs: SortedBatchCursor::new(),
        }
    }

    fn init(&mut self, arrays: &OtapMetricsDirectArrays<'_>) -> Result<(), Error> {
        self.batch_sorter
            .init_cursor_for_root_batch(arrays.root, &mut self.root)
            .map_err(map_otap_view_error)?;

        if let Some(attrs) = arrays.resource_attrs.as_ref() {
            self.batch_sorter
                .init_cursor_for_u16_id_column(&attrs.parent_id, &mut self.resource_attrs);
        }
        if let Some(attrs) = arrays.scope_attrs.as_ref() {
            self.batch_sorter
                .init_cursor_for_u16_id_column(&attrs.parent_id, &mut self.scope_attrs);
        }
        if let Some(attrs) = arrays.metric_attrs.as_ref() {
            self.batch_sorter
                .init_cursor_for_u16_id_column(&attrs.parent_id, &mut self.metric_attrs);
        }
        if let Some(number_dp) = arrays.number_dp.as_ref() {
            let parent_ids = MaybeDictArrayAccessor::Native(number_dp.parent_id);
            self.batch_sorter
                .init_cursor_for_u16_id_column(&parent_ids, &mut self.number_dp);
        }
        if let Some(attrs) = arrays.number_dp_attrs.as_ref() {
            self.batch_sorter
                .init_cursor_for_u32_id_column(&attrs.parent_id, &mut self.number_dp_attrs);
        }

        Ok(())
    }
}

fn map_otap_view_error(error: crate::error::Error) -> Error {
    Error::OtapView(error.to_string())
}

fn otap_metric_type(arrays: &OtapMetricsDirectArrays<'_>, row: usize) -> Result<MetricType, Error> {
    let value = arrays
        .metrics
        .metric_type
        .value_at(row)
        .ok_or(Error::UnsupportedMetricType("empty"))?;
    MetricType::try_from(value).map_err(|_| Error::UnsupportedMetricType("unknown"))
}

fn otap_aggregation_temporality(arrays: &OtapMetricsDirectArrays<'_>, row: usize) -> u64 {
    match arrays
        .metrics
        .aggregation_temporality
        .as_ref()
        .and_then(|col| col.value_at(row))
    {
        Some(1) => 1,
        Some(2) => 2,
        _ => 0,
    }
}

fn otap_is_monotonic(arrays: &OtapMetricsDirectArrays<'_>, row: usize) -> bool {
    arrays.metrics.is_monotonic.value_at(row).unwrap_or(false)
}

fn otap_scope_schema_url<'a>(arrays: &'a OtapMetricsDirectArrays<'a>, row: usize) -> &'a [u8] {
    arrays
        .metrics
        .schema_url
        .as_ref()
        .and_then(|col| col.str_at(row).map(|value| value.as_bytes()))
        .unwrap_or(b"")
}

struct ViewMetric<'a, M> {
    metric: &'a M,
    r#type: u64,
    aggregation_temporality: u64,
    monotonic: bool,
}

#[derive(Clone, Copy, PartialEq)]
struct StefPoint {
    start_timestamp: u64,
    timestamp: u64,
    value: StefPointValue,
}

#[derive(Clone, Copy, PartialEq)]
enum StefPointValue {
    None,
    Int64(i64),
    Float64(f64),
}

#[derive(Default)]
struct MetricEncoder {
    bits: BitWriter,
    name: StringEncoder,
    description: StringEncoder,
    unit: StringEncoder,
    r#type: U64Encoder,
    metadata: AttributesEncoder,
    histogram_bounds: Float64ArrayEncoder,
    aggregation_temporality: U64Encoder,
    monotonic: BoolEncoder,
}

impl MetricEncoder {
    fn new(
        name: SharedStringDict,
        description: SharedStringDict,
        unit: SharedStringDict,
        attribute_key: SharedStringDict,
        any_value_string: SharedStringDict,
    ) -> Self {
        Self {
            name: StringEncoder::with_dict(name),
            description: StringEncoder::with_dict(description),
            unit: StringEncoder::with_dict(unit),
            metadata: AttributesEncoder::new(attribute_key, any_value_string),
            ..Self::default()
        }
    }

    fn encode_view<M>(&mut self, metric: &ViewMetric<'_, M>) -> Result<(), Error>
    where
        M: MetricView,
    {
        self.bits.write_bit(true);
        self.bits.write_bits(METRIC_FIELD_MASK, 8);
        self.name
            .encode_bytes(metric.metric.name(), "metric.name")?;
        self.description
            .encode_bytes(metric.metric.description(), "metric.description")?;
        self.unit
            .encode_bytes(metric.metric.unit(), "metric.unit")?;
        self.r#type.encode(metric.r#type);
        self.metadata.encode_view(metric.metric.metadata())?;
        self.histogram_bounds.encode_empty();
        self.aggregation_temporality
            .encode(metric.aggregation_temporality);
        self.monotonic.encode(metric.monotonic);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_otap(
        &mut self,
        row: usize,
        metrics: &MetricsArrays<'_>,
        metadata: Option<&Attribute16Arrays<'_>>,
        metadata_cursor: &mut SortedBatchCursor,
        metric_type: u64,
        aggregation_temporality: u64,
        monotonic: bool,
    ) -> Result<(), Error> {
        self.bits.write_bit(true);
        self.bits.write_bits(METRIC_FIELD_MASK, 8);
        self.name.encode_bytes(
            metrics
                .name
                .str_at(row)
                .map(|value| value.as_bytes())
                .unwrap_or(b""),
            "metric.name",
        )?;
        self.description.encode_bytes(
            metrics
                .description
                .as_ref()
                .and_then(|col| col.str_at(row).map(|value| value.as_bytes()))
                .unwrap_or(b""),
            "metric.description",
        )?;
        self.unit.encode_bytes(
            metrics
                .unit
                .as_ref()
                .and_then(|col| col.str_at(row).map(|value| value.as_bytes()))
                .unwrap_or(b""),
            "metric.unit",
        )?;
        self.r#type.encode(metric_type);
        self.metadata.encode_otap_attributes(
            metadata,
            metrics.id.value_at(row),
            metadata_cursor,
        )?;
        self.histogram_bounds.encode_empty();
        self.aggregation_temporality.encode(aggregation_temporality);
        self.monotonic.encode(monotonic);
        Ok(())
    }

    fn take_column(&mut self) -> Column {
        let mut column = metric_column_tree();
        column.data = self.bits.take_bytes();
        column.children[0].data = self.name.take_bytes();
        column.children[1].data = self.description.take_bytes();
        column.children[2].data = self.unit.take_bytes();
        column.children[3].data = self.r#type.take_bytes();
        column.children[4] = self.metadata.take_column();
        column.children[5] = self.histogram_bounds.take_column();
        column.children[6].data = self.aggregation_temporality.take_bytes();
        column.children[7].data = self.monotonic.take_bytes();
        column
    }
}

#[derive(Default)]
struct ResourceEncoder {
    bits: BitWriter,
    schema_url: StringEncoder,
    attributes: AttributesEncoder,
    dropped_attributes_count: U64Encoder,
}

impl ResourceEncoder {
    fn new(
        schema_url: SharedStringDict,
        attribute_key: SharedStringDict,
        any_value_string: SharedStringDict,
    ) -> Self {
        Self {
            schema_url: StringEncoder::with_dict(schema_url),
            attributes: AttributesEncoder::new(attribute_key, any_value_string),
            ..Self::default()
        }
    }

    fn encode_view<R>(
        &mut self,
        schema_url: Option<Str<'_>>,
        resource: Option<&R>,
    ) -> Result<(), Error>
    where
        R: ResourceView,
    {
        self.bits.write_bit(true);
        self.bits.write_bits(RESOURCE_FIELD_MASK, 3);
        self.schema_url
            .encode_bytes(schema_url.unwrap_or(b""), "resource.schema_url")?;
        if let Some(resource) = resource {
            self.attributes.encode_view(resource.attributes())?;
            self.dropped_attributes_count
                .encode(u64::from(resource.dropped_attributes_count()));
        } else {
            self.attributes.encode_empty();
            self.dropped_attributes_count.encode(0);
        }
        Ok(())
    }

    fn encode_otap(
        &mut self,
        row: usize,
        resource: &ResourceArrays<'_>,
        attrs: Option<&Attribute16Arrays<'_>>,
        attrs_cursor: &mut SortedBatchCursor,
    ) -> Result<(), Error> {
        self.bits.write_bit(true);
        self.bits.write_bits(RESOURCE_FIELD_MASK, 3);
        self.schema_url.encode_bytes(
            resource
                .schema_url
                .as_ref()
                .and_then(|col| col.str_at(row).map(|value| value.as_bytes()))
                .unwrap_or(b""),
            "resource.schema_url",
        )?;
        self.attributes
            .encode_otap_attributes(attrs, resource.id.value_at(row), attrs_cursor)?;
        self.dropped_attributes_count.encode(
            resource
                .dropped_attributes_count
                .value_at(row)
                .map(u64::from)
                .unwrap_or(0),
        );
        Ok(())
    }

    fn take_column(&mut self) -> Column {
        let mut column = resource_column_tree();
        column.data = self.bits.take_bytes();
        column.children[0].data = self.schema_url.take_bytes();
        column.children[1] = self.attributes.take_column();
        column.children[2].data = self.dropped_attributes_count.take_bytes();
        column
    }
}

#[derive(Default)]
struct ScopeEncoder {
    bits: BitWriter,
    name: StringEncoder,
    version: StringEncoder,
    schema_url: StringEncoder,
    attributes: AttributesEncoder,
    dropped_attributes_count: U64Encoder,
}

impl ScopeEncoder {
    fn new(
        name: SharedStringDict,
        version: SharedStringDict,
        schema_url: SharedStringDict,
        attribute_key: SharedStringDict,
        any_value_string: SharedStringDict,
    ) -> Self {
        Self {
            name: StringEncoder::with_dict(name),
            version: StringEncoder::with_dict(version),
            schema_url: StringEncoder::with_dict(schema_url),
            attributes: AttributesEncoder::new(attribute_key, any_value_string),
            ..Self::default()
        }
    }

    fn encode_view<S>(&mut self, schema_url: Str<'_>, scope: Option<&S>) -> Result<(), Error>
    where
        S: InstrumentationScopeView,
    {
        self.bits.write_bit(true);
        self.bits.write_bits(SCOPE_FIELD_MASK, 5);
        if let Some(scope) = scope {
            self.name
                .encode_bytes(scope.name().unwrap_or(b""), "scope.name")?;
            self.version
                .encode_bytes(scope.version().unwrap_or(b""), "scope.version")?;
            self.schema_url
                .encode_bytes(schema_url, "scope.schema_url")?;
            self.attributes.encode_view(scope.attributes())?;
            self.dropped_attributes_count
                .encode(u64::from(scope.dropped_attributes_count()));
        } else {
            self.name.encode("");
            self.version.encode("");
            self.schema_url
                .encode_bytes(schema_url, "scope.schema_url")?;
            self.attributes.encode_empty();
            self.dropped_attributes_count.encode(0);
        }
        Ok(())
    }

    fn encode_otap(
        &mut self,
        row: usize,
        scope: &ScopeArrays<'_>,
        schema_url: Str<'_>,
        attrs: Option<&Attribute16Arrays<'_>>,
        attrs_cursor: &mut SortedBatchCursor,
    ) -> Result<(), Error> {
        self.bits.write_bit(true);
        self.bits.write_bits(SCOPE_FIELD_MASK, 5);
        self.name.encode_bytes(
            scope
                .name
                .as_ref()
                .and_then(|col| col.str_at(row).map(|value| value.as_bytes()))
                .unwrap_or(b""),
            "scope.name",
        )?;
        self.version.encode_bytes(
            scope
                .version
                .as_ref()
                .and_then(|col| col.str_at(row).map(|value| value.as_bytes()))
                .unwrap_or(b""),
            "scope.version",
        )?;
        self.schema_url
            .encode_bytes(schema_url, "scope.schema_url")?;
        self.attributes
            .encode_otap_attributes(attrs, scope.id.value_at(row), attrs_cursor)?;
        self.dropped_attributes_count.encode(
            scope
                .dropped_attributes_count
                .value_at(row)
                .map(u64::from)
                .unwrap_or(0),
        );
        Ok(())
    }

    fn take_column(&mut self) -> Column {
        let mut column = scope_column_tree();
        column.data = self.bits.take_bytes();
        column.children[0].data = self.name.take_bytes();
        column.children[1].data = self.version.take_bytes();
        column.children[2].data = self.schema_url.take_bytes();
        column.children[3] = self.attributes.take_column();
        column.children[4].data = self.dropped_attributes_count.take_bytes();
        column
    }
}

#[derive(Default)]
struct PointEncoder {
    bits: BitWriter,
    start_timestamp: U64Encoder,
    timestamp: U64Encoder,
    value: PointValueEncoder,
    exemplars: ExemplarArrayEncoder,
    last: Option<StefPoint>,
}

impl PointEncoder {
    fn encode(&mut self, point: &StefPoint) -> Result<(), Error> {
        let last = self.last.as_ref();
        let mut mask = 0;
        if last.is_none_or(|last| last.start_timestamp != point.start_timestamp) {
            mask |= 1 << 0;
        }
        if last.is_none_or(|last| last.timestamp != point.timestamp) {
            mask |= 1 << 1;
        }
        if last.is_none_or(|last| last.value != point.value) {
            mask |= 1 << 2;
        }

        self.bits.write_bits(mask, 4);
        if mask & (1 << 0) != 0 {
            self.start_timestamp.encode(point.start_timestamp);
        }
        if mask & (1 << 1) != 0 {
            self.timestamp.encode(point.timestamp);
        }
        if mask & (1 << 2) != 0 {
            self.value.encode(&point.value);
        }
        self.last = Some(*point);
        Ok(())
    }

    fn encode_otap_number_point(
        &mut self,
        arrays: &NumberDpArrays<'_>,
        row: usize,
    ) -> Result<(), Error> {
        let start_timestamp = arrays
            .start_time_unix_nano
            .and_then(|col| col.value_at(row).map(|v| v as u64))
            .unwrap_or(0);
        let timestamp = arrays
            .time_unix_nano
            .and_then(|col| col.value_at(row).map(|v| v as u64))
            .unwrap_or(0);

        let last = self.last.as_ref();
        let mut mask = 0;
        if last.is_none_or(|last| last.start_timestamp != start_timestamp) {
            mask |= 1 << 0;
        }
        if last.is_none_or(|last| last.timestamp != timestamp) {
            mask |= 1 << 1;
        }

        if let Some(value) = arrays.double_value.and_then(|col| col.value_at(row)) {
            if last.is_none_or(|last| last.value != StefPointValue::Float64(value)) {
                mask |= 1 << 2;
            }
            self.bits.write_bits(mask, 4);
            if mask & (1 << 0) != 0 {
                self.start_timestamp.encode(start_timestamp);
            }
            if mask & (1 << 1) != 0 {
                self.timestamp.encode(timestamp);
            }
            if mask & (1 << 2) != 0 {
                self.value.encode_float64(value);
            }
            self.last = Some(StefPoint {
                start_timestamp,
                timestamp,
                value: StefPointValue::Float64(value),
            });
            return Ok(());
        }

        if let Some(value) = arrays.int_value.and_then(|col| col.value_at(row)) {
            if last.is_none_or(|last| last.value != StefPointValue::Int64(value)) {
                mask |= 1 << 2;
            }
            self.bits.write_bits(mask, 4);
            if mask & (1 << 0) != 0 {
                self.start_timestamp.encode(start_timestamp);
            }
            if mask & (1 << 1) != 0 {
                self.timestamp.encode(timestamp);
            }
            if mask & (1 << 2) != 0 {
                self.value.encode_int64(value);
            }
            self.last = Some(StefPoint {
                start_timestamp,
                timestamp,
                value: StefPointValue::Int64(value),
            });
            return Ok(());
        }

        let flags = arrays.flags.and_then(|col| col.value_at(row)).unwrap_or(0);
        if flags & 1 == 0 {
            return Err(Error::UnsupportedDataPointValue);
        }

        if last.is_none_or(|last| last.value != StefPointValue::None) {
            mask |= 1 << 2;
        }
        self.bits.write_bits(mask, 4);
        if mask & (1 << 0) != 0 {
            self.start_timestamp.encode(start_timestamp);
        }
        if mask & (1 << 1) != 0 {
            self.timestamp.encode(timestamp);
        }
        if mask & (1 << 2) != 0 {
            self.value.encode_none();
        }
        self.last = Some(StefPoint {
            start_timestamp,
            timestamp,
            value: StefPointValue::None,
        });
        Ok(())
    }

    fn take_column(&mut self) -> Column {
        let mut column = point_column_tree();
        column.data = self.bits.take_bytes();
        column.children[0].data = self.start_timestamp.take_bytes();
        column.children[1].data = self.timestamp.take_bytes();
        column.children[2] = self.value.take_column();
        column.children[3] = self.exemplars.take_column();
        column
    }
}

#[derive(Default)]
struct PointValueEncoder {
    bits: BitWriter,
    int64: I64Encoder,
    float64: Float64Encoder,
}

impl PointValueEncoder {
    fn encode(&mut self, value: &StefPointValue) {
        match value {
            StefPointValue::None => self.encode_none(),
            StefPointValue::Int64(value) => self.encode_int64(*value),
            StefPointValue::Float64(value) => self.encode_float64(*value),
        }
    }

    fn encode_none(&mut self) {
        self.bits.write_bits(0, 3);
    }

    fn encode_int64(&mut self, value: i64) {
        self.bits.write_bits(1, 3);
        self.int64.encode(value);
    }

    fn encode_float64(&mut self, value: f64) {
        self.bits.write_bits(2, 3);
        self.float64.encode(value);
    }

    fn take_column(&mut self) -> Column {
        let mut column = point_value_column_tree();
        column.data = self.bits.take_bytes();
        column.children[0].data = self.int64.take_bytes();
        column.children[1].data = self.float64.take_bytes();
        column
    }
}

#[derive(Default)]
struct AttributesEncoder {
    header: BytesWriter,
    key: StringEncoder,
    value: AnyValueEncoder,
    last: Vec<DirectAttribute>,
    changed_values: Vec<(usize, DirectAnyValue)>,
}

impl AttributesEncoder {
    fn new(attribute_key: SharedStringDict, any_value_string: SharedStringDict) -> Self {
        Self {
            key: StringEncoder::with_dict(attribute_key),
            value: AnyValueEncoder::new(any_value_string),
            ..Self::default()
        }
    }

    fn encode_empty(&mut self) {
        self.header.write_uvarint(0b1);
        self.last.clear();
        self.changed_values.clear();
    }

    fn encode_view<A>(&mut self, mut attributes: impl Iterator<Item = A>) -> Result<(), Error>
    where
        A: AttributeView,
    {
        if self.last.is_empty() || self.last.len() >= 63 {
            return self.encode_full_view(attributes);
        }

        let last_len = self.last.len();
        let mut changed = 0_u64;
        let mut seen = 0_usize;
        self.changed_values.clear();

        while let Some(attribute) = attributes.next() {
            if seen >= last_len {
                return self.encode_full_view_from_seen(seen, Some(attribute), attributes);
            }

            let last = &self.last[seen];
            if attribute.key() != last.key.as_ref().as_bytes() {
                return self.encode_full_view_from_seen(seen, Some(attribute), attributes);
            }

            if let Some(value) =
                direct_any_value_from_view_if_changed(attribute.value(), &last.value)?
            {
                changed |= 1 << seen;
                self.changed_values.push((seen, value));
            }
            seen += 1;
        }

        if seen != last_len {
            return self.encode_full_view_from_seen(seen, None, std::iter::empty::<A>());
        }

        self.header.write_uvarint(changed << 1);
        let mut changed_values = std::mem::take(&mut self.changed_values);
        for (index, value) in changed_values.drain(..) {
            self.value.encode(&value)?;
            self.last[index].value = value;
        }
        self.changed_values = changed_values;
        Ok(())
    }

    fn encode_full_view<A>(&mut self, attributes: impl Iterator<Item = A>) -> Result<(), Error>
    where
        A: AttributeView,
    {
        self.last.clear();
        for attribute in attributes {
            self.encode_full_attribute(attribute)?;
        }
        self.header
            .write_uvarint((self.last.len() as u64) << 1 | 0b1);
        Ok(())
    }

    fn encode_full_view_from_seen<A>(
        &mut self,
        seen: usize,
        current: Option<A>,
        remaining: impl Iterator<Item = A>,
    ) -> Result<(), Error>
    where
        A: AttributeView,
    {
        let old_last = std::mem::take(&mut self.last);
        let changed_values = std::mem::take(&mut self.changed_values);
        let mut changed_values = changed_values.into_iter().peekable();

        self.last.reserve(seen + usize::from(current.is_some()));
        for (index, previous) in old_last.into_iter().take(seen).enumerate() {
            let mut value = previous.value;
            if changed_values
                .peek()
                .is_some_and(|(changed_index, _)| *changed_index == index)
            {
                value = changed_values.next().expect("peeked changed value").1;
            }

            self.key.encode(previous.key.as_ref());
            self.value.encode(&value)?;
            self.last.push(DirectAttribute {
                key: previous.key,
                value,
            });
        }

        if let Some(attribute) = current {
            self.encode_full_attribute(attribute)?;
        }
        for attribute in remaining {
            self.encode_full_attribute(attribute)?;
        }

        self.header
            .write_uvarint((self.last.len() as u64) << 1 | 0b1);
        Ok(())
    }

    fn encode_full_attribute<A>(&mut self, attribute: A) -> Result<(), Error>
    where
        A: AttributeView,
    {
        let key = std::str::from_utf8(attribute.key())
            .map_err(|_| Error::InvalidUtf8("attribute.key"))?;
        self.key.encode(key);
        let value = self.value.encode_view(attribute.value())?;
        self.last.push(DirectAttribute {
            key: key.into(),
            value,
        });
        Ok(())
    }

    fn encode_otap_attributes<T>(
        &mut self,
        attrs: Option<&AttributeArrays<'_, T>>,
        parent_id: Option<T::Native>,
        cursor: &mut SortedBatchCursor,
    ) -> Result<(), Error>
    where
        T: ArrowPrimitiveType,
    {
        let (Some(attrs), Some(parent_id)) = (attrs, parent_id) else {
            self.encode_empty();
            return Ok(());
        };

        let mut rows = ChildIndexIter::new(parent_id, &attrs.parent_id, cursor);
        if self.last.is_empty() || self.last.len() >= 63 {
            return self.encode_full_otap_attributes(attrs, rows);
        }

        let last_len = self.last.len();
        let mut changed = 0_u64;
        let mut seen = 0_usize;
        self.changed_values.clear();

        while let Some(row) = rows.next() {
            let Some(key) = attrs.attr_key.str_at(row) else {
                continue;
            };
            if seen >= last_len {
                return self.encode_full_otap_attributes_from_seen(attrs, seen, Some(row), rows);
            }

            let last = &self.last[seen];
            if key.as_bytes() != last.key.as_ref().as_bytes() {
                return self.encode_full_otap_attributes_from_seen(attrs, seen, Some(row), rows);
            }

            if let Some(value) =
                direct_any_value_from_otap_if_changed(&attrs.anyval_arrays, row, &last.value)
            {
                changed |= 1 << seen;
                self.changed_values.push((seen, value));
            }
            seen += 1;
        }

        if seen != last_len {
            return self.encode_full_otap_attributes_from_seen(
                attrs,
                seen,
                None,
                std::iter::empty(),
            );
        }

        self.header.write_uvarint(changed << 1);
        let mut changed_values = std::mem::take(&mut self.changed_values);
        for (index, value) in changed_values.drain(..) {
            self.value.encode(&value)?;
            self.last[index].value = value;
        }
        self.changed_values = changed_values;
        Ok(())
    }

    fn encode_full_otap_attributes<T>(
        &mut self,
        attrs: &AttributeArrays<'_, T>,
        rows: impl Iterator<Item = usize>,
    ) -> Result<(), Error>
    where
        T: ArrowPrimitiveType,
    {
        self.last.clear();
        for row in rows {
            self.encode_full_otap_attribute(attrs, row)?;
        }
        self.header
            .write_uvarint((self.last.len() as u64) << 1 | 0b1);
        Ok(())
    }

    fn encode_full_otap_attributes_from_seen<T>(
        &mut self,
        attrs: &AttributeArrays<'_, T>,
        seen: usize,
        current: Option<usize>,
        remaining: impl Iterator<Item = usize>,
    ) -> Result<(), Error>
    where
        T: ArrowPrimitiveType,
    {
        let old_last = std::mem::take(&mut self.last);
        let changed_values = std::mem::take(&mut self.changed_values);
        let mut changed_values = changed_values.into_iter().peekable();

        self.last.reserve(seen + usize::from(current.is_some()));
        for (index, previous) in old_last.into_iter().take(seen).enumerate() {
            let mut value = previous.value;
            if changed_values
                .peek()
                .is_some_and(|(changed_index, _)| *changed_index == index)
            {
                value = changed_values.next().expect("peeked changed value").1;
            }

            self.key.encode(previous.key.as_ref());
            self.value.encode(&value)?;
            self.last.push(DirectAttribute {
                key: previous.key,
                value,
            });
        }

        if let Some(row) = current {
            self.encode_full_otap_attribute(attrs, row)?;
        }
        for row in remaining {
            self.encode_full_otap_attribute(attrs, row)?;
        }

        self.header
            .write_uvarint((self.last.len() as u64) << 1 | 0b1);
        Ok(())
    }

    fn encode_full_otap_attribute<T>(
        &mut self,
        attrs: &AttributeArrays<'_, T>,
        row: usize,
    ) -> Result<(), Error>
    where
        T: ArrowPrimitiveType,
    {
        let Some(key) = attrs.attr_key.str_at(row) else {
            return Ok(());
        };
        self.key.encode(key);
        let value = self.value.encode_otap(&attrs.anyval_arrays, row)?;
        self.last.push(DirectAttribute {
            key: key.into(),
            value,
        });
        Ok(())
    }

    fn take_column(&mut self) -> Column {
        let mut column = attributes_column_tree();
        column.data = self.header.take_bytes();
        column.children[0].data = self.key.take_bytes();
        column.children[1] = self.value.take_column();
        column
    }
}

fn direct_any_value_from_view<'v, V>(value: Option<V>) -> Result<DirectAnyValue, Error>
where
    V: AnyValueView<'v>,
{
    let Some(value) = value else {
        return Ok(DirectAnyValue::Empty);
    };
    let value = match value.value_type() {
        ValueType::Empty => DirectAnyValue::Empty,
        ValueType::String => DirectAnyValue::String(
            std::str::from_utf8(
                value
                    .as_string()
                    .ok_or(Error::UnsupportedAttributeValue("string"))?,
            )
            .map_err(|_| Error::InvalidUtf8("attribute.value.string"))?
            .into(),
        ),
        ValueType::Bool => DirectAnyValue::Bool(
            value
                .as_bool()
                .ok_or(Error::UnsupportedAttributeValue("bool"))?,
        ),
        ValueType::Int64 => DirectAnyValue::Int(
            value
                .as_int64()
                .ok_or(Error::UnsupportedAttributeValue("int"))?,
        ),
        ValueType::Double => DirectAnyValue::Double(
            value
                .as_double()
                .ok_or(Error::UnsupportedAttributeValue("double"))?,
        ),
        ValueType::Array => return Err(Error::UnsupportedAttributeValue("array")),
        ValueType::KeyValueList => return Err(Error::UnsupportedAttributeValue("kvlist")),
        ValueType::Bytes => DirectAnyValue::Bytes(
            value
                .as_bytes()
                .ok_or(Error::UnsupportedAttributeValue("bytes"))?
                .to_vec(),
        ),
    };
    Ok(value)
}

fn direct_any_value_from_otap(anyval: &AnyValueArrays<'_>, row: usize) -> DirectAnyValue {
    match otap_attribute_value_type(anyval, row) {
        AttributeValueType::Empty => DirectAnyValue::Empty,
        AttributeValueType::Str => anyval
            .attr_str
            .as_ref()
            .and_then(|accessor| accessor.str_at(row))
            .map_or(DirectAnyValue::Empty, |value| {
                DirectAnyValue::String(value.into())
            }),
        AttributeValueType::Int => anyval
            .attr_int
            .as_ref()
            .and_then(|accessor| accessor.value_at(row))
            .map_or(DirectAnyValue::Empty, DirectAnyValue::Int),
        AttributeValueType::Double => anyval
            .attr_double
            .and_then(|arr| {
                arr.is_valid(row)
                    .then(|| DirectAnyValue::Double(arr.value(row)))
            })
            .unwrap_or(DirectAnyValue::Empty),
        AttributeValueType::Bool => anyval
            .attr_bool
            .and_then(|arr| {
                arr.is_valid(row)
                    .then(|| DirectAnyValue::Bool(arr.value(row)))
            })
            .unwrap_or(DirectAnyValue::Empty),
        AttributeValueType::Bytes => anyval
            .attr_bytes
            .as_ref()
            .and_then(|accessor| accessor.slice_at(row))
            .map_or(DirectAnyValue::Empty, |value| {
                DirectAnyValue::Bytes(value.to_vec())
            }),
        AttributeValueType::Map | AttributeValueType::Slice => DirectAnyValue::Empty,
    }
}

fn direct_any_value_from_otap_if_changed(
    anyval: &AnyValueArrays<'_>,
    row: usize,
    last: &DirectAnyValue,
) -> Option<DirectAnyValue> {
    match otap_attribute_value_type(anyval, row) {
        AttributeValueType::Empty => {
            (last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty)
        }
        AttributeValueType::Str => {
            let value = anyval
                .attr_str
                .as_ref()
                .and_then(|accessor| accessor.str_at(row));
            match value {
                Some(value)
                    if matches!(
                        last,
                        DirectAnyValue::String(last) if last.as_ref() == value
                    ) =>
                {
                    None
                }
                Some(value) => Some(DirectAnyValue::String(value.into())),
                None => (last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty),
            }
        }
        AttributeValueType::Int => {
            let value = anyval
                .attr_int
                .as_ref()
                .and_then(|accessor| accessor.value_at(row));
            match value {
                Some(value) if matches!(last, DirectAnyValue::Int(last) if *last == value) => None,
                Some(value) => Some(DirectAnyValue::Int(value)),
                None => (last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty),
            }
        }
        AttributeValueType::Double => {
            let value = anyval
                .attr_double
                .and_then(|arr| arr.is_valid(row).then(|| arr.value(row)));
            match value {
                Some(value) if matches!(last, DirectAnyValue::Double(last) if *last == value) => {
                    None
                }
                Some(value) => Some(DirectAnyValue::Double(value)),
                None => (last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty),
            }
        }
        AttributeValueType::Bool => {
            let value = anyval
                .attr_bool
                .and_then(|arr| arr.is_valid(row).then(|| arr.value(row)));
            match value {
                Some(value) if matches!(last, DirectAnyValue::Bool(last) if *last == value) => None,
                Some(value) => Some(DirectAnyValue::Bool(value)),
                None => (last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty),
            }
        }
        AttributeValueType::Bytes => {
            let value = anyval
                .attr_bytes
                .as_ref()
                .and_then(|accessor| accessor.slice_at(row));
            match value {
                Some(value) if matches!(last, DirectAnyValue::Bytes(last) if last == value) => None,
                Some(value) => Some(DirectAnyValue::Bytes(value.to_vec())),
                None => (last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty),
            }
        }
        AttributeValueType::Map | AttributeValueType::Slice => {
            (last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty)
        }
    }
}

fn otap_attribute_value_type(anyval: &AnyValueArrays<'_>, row: usize) -> AttributeValueType {
    if !anyval.attr_type.is_valid(row) {
        return AttributeValueType::Empty;
    }
    AttributeValueType::try_from(anyval.attr_type.value(row)).unwrap_or(AttributeValueType::Empty)
}

fn direct_any_value_from_view_if_changed<'v, V>(
    value: Option<V>,
    last: &DirectAnyValue,
) -> Result<Option<DirectAnyValue>, Error>
where
    V: AnyValueView<'v>,
{
    let Some(value) = value else {
        return Ok((last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty));
    };

    match value.value_type() {
        ValueType::Empty => Ok((last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty)),
        ValueType::String => {
            let value = value
                .as_string()
                .ok_or(Error::UnsupportedAttributeValue("string"))?;
            let value = std::str::from_utf8(value)
                .map_err(|_| Error::InvalidUtf8("attribute.value.string"))?;
            if matches!(last, DirectAnyValue::String(last) if last.as_ref() == value) {
                Ok(None)
            } else {
                Ok(Some(DirectAnyValue::String(value.into())))
            }
        }
        ValueType::Bool => {
            let value = value
                .as_bool()
                .ok_or(Error::UnsupportedAttributeValue("bool"))?;
            Ok(
                (!matches!(last, DirectAnyValue::Bool(last) if *last == value))
                    .then_some(DirectAnyValue::Bool(value)),
            )
        }
        ValueType::Int64 => {
            let value = value
                .as_int64()
                .ok_or(Error::UnsupportedAttributeValue("int"))?;
            Ok(
                (!matches!(last, DirectAnyValue::Int(last) if *last == value))
                    .then_some(DirectAnyValue::Int(value)),
            )
        }
        ValueType::Double => {
            let value = value
                .as_double()
                .ok_or(Error::UnsupportedAttributeValue("double"))?;
            Ok(
                (!matches!(last, DirectAnyValue::Double(last) if *last == value))
                    .then_some(DirectAnyValue::Double(value)),
            )
        }
        ValueType::Array => Err(Error::UnsupportedAttributeValue("array")),
        ValueType::KeyValueList => Err(Error::UnsupportedAttributeValue("kvlist")),
        ValueType::Bytes => {
            let value = value
                .as_bytes()
                .ok_or(Error::UnsupportedAttributeValue("bytes"))?;
            if matches!(last, DirectAnyValue::Bytes(last) if last == value) {
                Ok(None)
            } else {
                Ok(Some(DirectAnyValue::Bytes(value.to_vec())))
            }
        }
    }
}

#[derive(Default)]
struct AnyValueEncoder {
    bits: BitWriter,
    string: StringEncoder,
    bool_: BoolEncoder,
    int64: I64Encoder,
    float64: Float64Encoder,
    bytes: BytesEncoder,
}

impl AnyValueEncoder {
    fn new(string_dict: SharedStringDict) -> Self {
        Self {
            string: StringEncoder::with_dict(string_dict),
            ..Self::default()
        }
    }

    fn encode(&mut self, value: &DirectAnyValue) -> Result<(), Error> {
        match value {
            DirectAnyValue::Empty => self.bits.write_bits(0, 4),
            DirectAnyValue::String(value) => {
                self.bits.write_bits(1, 4);
                self.string.encode(value.as_ref());
            }
            DirectAnyValue::Bool(value) => {
                self.bits.write_bits(2, 4);
                self.bool_.encode(*value);
            }
            DirectAnyValue::Int(value) => {
                self.bits.write_bits(3, 4);
                self.int64.encode(*value);
            }
            DirectAnyValue::Double(value) => {
                self.bits.write_bits(4, 4);
                self.float64.encode(*value);
            }
            DirectAnyValue::Bytes(value) => {
                self.bits.write_bits(7, 4);
                self.bytes.encode(value);
            }
        }
        Ok(())
    }

    fn encode_view<'v, V>(&mut self, value: Option<V>) -> Result<DirectAnyValue, Error>
    where
        V: AnyValueView<'v>,
    {
        let value = direct_any_value_from_view(value)?;
        self.encode(&value)?;
        Ok(value)
    }

    fn encode_otap(
        &mut self,
        anyval: &AnyValueArrays<'_>,
        row: usize,
    ) -> Result<DirectAnyValue, Error> {
        let value = direct_any_value_from_otap(anyval, row);
        self.encode(&value)?;
        Ok(value)
    }

    fn take_column(&mut self) -> Column {
        let mut column = any_value_column_tree();
        column.data = self.bits.take_bytes();
        column.children[0].data = self.string.take_bytes();
        column.children[1].data = self.bool_.take_bytes();
        column.children[2].data = self.int64.take_bytes();
        column.children[3].data = self.float64.take_bytes();
        column.children[6].data = self.bytes.take_bytes();
        column
    }
}

#[derive(Default)]
struct Float64ArrayEncoder {
    bits: BitWriter,
}

impl Float64ArrayEncoder {
    fn encode_empty(&mut self) {
        self.bits.write_uvarint_compact(0);
    }

    fn take_column(&mut self) -> Column {
        let mut column = array_column_tree(Column::default());
        column.data = self.bits.take_bytes();
        column
    }
}

#[derive(Default)]
struct ExemplarArrayEncoder {
    bits: BitWriter,
}

impl ExemplarArrayEncoder {
    fn take_column(&mut self) -> Column {
        let mut column = array_column_tree(Column::default());
        column.data = self.bits.take_bytes();
        column
    }
}

#[derive(Clone, Default)]
struct SharedStringDict(Rc<RefCell<StefStringDict>>);

type StefStringDict = HashMap<String, usize, ahash::RandomState>;

#[derive(Default)]
struct StringEncoder {
    bytes: BytesWriter,
    dict: SharedStringDict,
}

impl StringEncoder {
    fn with_dict(dict: SharedStringDict) -> Self {
        Self {
            bytes: BytesWriter::default(),
            dict,
        }
    }

    fn encode(&mut self, value: &str) {
        let mut dict = self.dict.0.borrow_mut();
        if let Some(ref_num) = dict.get(value) {
            self.bytes.write_varint(-(*ref_num as i64) - 1);
            return;
        }
        if value.len() > 1 {
            let ref_num = dict.len();
            let _ = dict.insert(value.to_owned(), ref_num);
        }
        drop(dict);
        self.bytes.write_varint(value.len() as i64);
        self.bytes.write_bytes(value.as_bytes());
    }

    fn encode_bytes(&mut self, value: &[u8], field: &'static str) -> Result<(), Error> {
        let value = std::str::from_utf8(value).map_err(|_| Error::InvalidUtf8(field))?;
        self.encode(value);
        Ok(())
    }

    fn take_bytes(&mut self) -> Vec<u8> {
        self.bytes.take_bytes()
    }
}

#[derive(Default)]
struct BytesEncoder {
    bytes: BytesWriter,
}

impl BytesEncoder {
    fn encode(&mut self, value: &[u8]) {
        self.bytes.write_varint(value.len() as i64);
        self.bytes.write_bytes(value);
    }

    fn take_bytes(&mut self) -> Vec<u8> {
        self.bytes.take_bytes()
    }
}

#[derive(Default)]
struct BoolEncoder {
    bits: BitWriter,
}

impl BoolEncoder {
    fn encode(&mut self, value: bool) {
        self.bits.write_bit(value);
    }

    fn take_bytes(&mut self) -> Vec<u8> {
        self.bits.take_bytes()
    }
}

#[derive(Default)]
struct U64Encoder {
    bytes: BytesWriter,
    last_value: u64,
    last_delta: u64,
}

impl U64Encoder {
    fn encode(&mut self, value: u64) {
        let delta = value.wrapping_sub(self.last_value);
        self.last_value = value;
        let delta_of_delta = delta.wrapping_sub(self.last_delta);
        self.last_delta = delta;
        self.bytes.write_varint(delta_of_delta as i64);
    }

    fn take_bytes(&mut self) -> Vec<u8> {
        self.bytes.take_bytes()
    }
}

#[derive(Default)]
struct I64Encoder {
    inner: U64Encoder,
}

impl I64Encoder {
    fn encode(&mut self, value: i64) {
        self.inner.encode(value as u64);
    }

    fn take_bytes(&mut self) -> Vec<u8> {
        self.inner.take_bytes()
    }
}

#[derive(Default)]
struct Float64Encoder {
    bits: BitWriter,
    last_value: f64,
    leading_bits: u32,
    trailing_bits: u32,
}

impl Float64Encoder {
    fn encode(&mut self, value: f64) {
        let xor_value = value.to_bits() ^ self.last_value.to_bits();
        self.last_value = value;
        if xor_value == 0 {
            self.bits.write_bit(false);
            return;
        }

        let mut leading = xor_value.leading_zeros();
        if leading >= 32 {
            leading = 31;
        }
        let trailing = xor_value.trailing_zeros();
        let significant_bits = 64 - leading - trailing;

        if leading >= self.leading_bits && trailing >= self.trailing_bits {
            let current_bit_count = 64 - self.leading_bits - self.trailing_bits;
            if 53 - self.leading_bits - self.trailing_bits <= significant_bits {
                self.bits.write_bits(0b10, 2);
                self.bits
                    .write_bits(xor_value >> self.trailing_bits, current_bit_count);
                return;
            }
        }

        self.leading_bits = leading;
        self.trailing_bits = trailing;
        let mut header = 0b11_u64;
        header = (header << 5) | u64::from(leading);
        header = (header << 6) | u64::from(significant_bits - 1);
        self.bits.write_bits(header, 13);
        self.bits
            .write_bits(xor_value >> trailing, significant_bits);
    }

    fn take_bytes(&mut self) -> Vec<u8> {
        self.bits.take_bytes()
    }
}

#[derive(Default)]
struct BytesWriter {
    bytes: Vec<u8>,
}

impl BytesWriter {
    fn write_uvarint(&mut self, value: u64) {
        append_uvarint(value, &mut self.bytes);
    }

    fn write_varint(&mut self, value: i64) {
        let encoded = ((value >> 63) ^ (value << 1)) as u64;
        self.write_uvarint(encoded);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    fn take_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.bytes)
    }
}

#[derive(Default)]
struct BitWriter {
    bytes: Vec<u8>,
    current: u8,
    used: u8,
}

impl BitWriter {
    fn write_bit(&mut self, value: bool) {
        if value {
            self.current |= 1 << (7 - self.used);
        }
        self.used += 1;
        if self.used == 8 {
            self.bytes.push(self.current);
            self.current = 0;
            self.used = 0;
        }
    }

    fn write_bits(&mut self, value: u64, bit_count: u32) {
        debug_assert!(bit_count <= 64);

        let mut remaining = bit_count;
        while remaining > 0 {
            let available = u32::from(8 - self.used);
            let take = remaining.min(available);
            let shift = remaining - take;
            let mask = (1_u64 << take) - 1;
            let bits = ((value >> shift) & mask) as u8;

            self.current |= bits << (available - take);
            self.used += take as u8;
            remaining -= take;

            if self.used == 8 {
                self.bytes.push(self.current);
                self.current = 0;
                self.used = 0;
            }
        }
    }

    fn write_uvarint_compact(&mut self, value: u64) {
        if value == 0 {
            self.write_bit(true);
        } else if value < (1 << 2) {
            self.write_bits(0b01, 2);
            self.write_bits(value, 2);
        } else if value < (1 << 5) {
            self.write_bits(0b001, 3);
            self.write_bits(value, 5);
        } else if value < (1 << 12) {
            self.write_bits(0b0001, 4);
            self.write_bits(value, 12);
        } else if value < (1 << 19) {
            self.write_bits(0b00001, 5);
            self.write_bits(value, 19);
        } else if value < (1 << 26) {
            self.write_bits(0b000001, 6);
            self.write_bits(value, 26);
        } else if value < (1_u64 << 33) {
            self.write_bits(0b0000001, 7);
            self.write_bits(value, 33);
        } else {
            self.write_bits(0b00000001, 8);
            self.write_bits(value, 48);
        }
    }

    fn take_bytes(&mut self) -> Vec<u8> {
        if self.used != 0 {
            self.bytes.push(self.current);
            self.current = 0;
            self.used = 0;
        }
        std::mem::take(&mut self.bytes)
    }
}

#[derive(Clone, Default)]
struct Column {
    data: Vec<u8>,
    children: Vec<Column>,
}

impl Column {
    fn with_children(children: Vec<Column>) -> Self {
        Self {
            data: Vec::new(),
            children,
        }
    }
}

#[derive(Default)]
struct DecodeColumn<'a> {
    data: &'a [u8],
    byte_len: usize,
    children: Vec<DecodeColumn<'a>>,
}

impl<'a> DecodeColumn<'a> {
    fn with_children(children: Vec<DecodeColumn<'a>>) -> Self {
        Self {
            data: &[],
            byte_len: 0,
            children,
        }
    }
}

fn metrics_column_tree() -> Column {
    Column::with_children(vec![
        Column::default(),
        metric_column_tree(),
        resource_column_tree(),
        scope_column_tree(),
        attributes_column_tree(),
        point_column_tree(),
    ])
}

fn metric_column_tree() -> Column {
    Column::with_children(vec![
        Column::default(),
        Column::default(),
        Column::default(),
        Column::default(),
        attributes_column_tree(),
        array_column_tree(Column::default()),
        Column::default(),
        Column::default(),
    ])
}

fn resource_column_tree() -> Column {
    Column::with_children(vec![
        Column::default(),
        attributes_column_tree(),
        Column::default(),
    ])
}

fn scope_column_tree() -> Column {
    Column::with_children(vec![
        Column::default(),
        Column::default(),
        Column::default(),
        attributes_column_tree(),
        Column::default(),
    ])
}

fn attributes_column_tree() -> Column {
    Column::with_children(vec![Column::default(), any_value_column_tree()])
}

fn any_value_column_tree() -> Column {
    Column::with_children(vec![
        Column::default(),
        Column::default(),
        Column::default(),
        Column::default(),
        Column::default(),
        Column::default(),
        Column::default(),
    ])
}

fn point_column_tree() -> Column {
    Column::with_children(vec![
        Column::default(),
        Column::default(),
        point_value_column_tree(),
        array_column_tree(Column::default()),
    ])
}

fn point_value_column_tree() -> Column {
    Column::with_children(vec![
        Column::default(),
        Column::default(),
        Column::default(),
        Column::default(),
        Column::default(),
    ])
}

fn array_column_tree(element: Column) -> Column {
    Column::with_children(vec![element])
}

fn decode_metrics_column_tree<'a>() -> DecodeColumn<'a> {
    DecodeColumn::with_children(vec![
        DecodeColumn::default(),
        decode_metric_column_tree(),
        decode_resource_column_tree(),
        decode_scope_column_tree(),
        decode_attributes_column_tree(),
        decode_point_column_tree(),
    ])
}

fn decode_metric_column_tree<'a>() -> DecodeColumn<'a> {
    DecodeColumn::with_children(vec![
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        decode_attributes_column_tree(),
        decode_array_column_tree(DecodeColumn::default()),
        DecodeColumn::default(),
        DecodeColumn::default(),
    ])
}

fn decode_resource_column_tree<'a>() -> DecodeColumn<'a> {
    DecodeColumn::with_children(vec![
        DecodeColumn::default(),
        decode_attributes_column_tree(),
        DecodeColumn::default(),
    ])
}

fn decode_scope_column_tree<'a>() -> DecodeColumn<'a> {
    DecodeColumn::with_children(vec![
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        decode_attributes_column_tree(),
        DecodeColumn::default(),
    ])
}

fn decode_attributes_column_tree<'a>() -> DecodeColumn<'a> {
    DecodeColumn::with_children(vec![
        DecodeColumn::default(),
        decode_any_value_column_tree(),
    ])
}

fn decode_any_value_column_tree<'a>() -> DecodeColumn<'a> {
    DecodeColumn::with_children(vec![
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
    ])
}

fn decode_point_column_tree<'a>() -> DecodeColumn<'a> {
    DecodeColumn::with_children(vec![
        DecodeColumn::default(),
        DecodeColumn::default(),
        decode_point_value_column_tree(),
        decode_array_column_tree(DecodeColumn::default()),
    ])
}

fn decode_point_value_column_tree<'a>() -> DecodeColumn<'a> {
    DecodeColumn::with_children(vec![
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
    ])
}

fn decode_array_column_tree(element: DecodeColumn<'_>) -> DecodeColumn<'_> {
    DecodeColumn::with_children(vec![element])
}

fn write_columns(columns: &Column, dst: &mut Vec<u8>) -> Result<(), Error> {
    let mut sizes = BitWriter::default();
    write_column_sizes(columns, &mut sizes)?;
    let size_bytes = sizes.take_bytes();
    append_uvarint(size_bytes.len() as u64, dst);
    dst.extend_from_slice(&size_bytes);
    write_column_data(columns, dst);
    Ok(())
}

fn write_column_sizes(column: &Column, sizes: &mut BitWriter) -> Result<(), Error> {
    let byte_len =
        u64::try_from(column.data.len()).map_err(|_| Error::ValueOutOfRange("column"))?;
    if byte_len >= (1_u64 << 48) {
        return Err(Error::ValueOutOfRange("column"));
    }
    sizes.write_uvarint_compact(byte_len);
    if column.data.is_empty() {
        return Ok(());
    }
    for child in &column.children {
        write_column_sizes(child, sizes)?;
    }
    Ok(())
}

fn write_column_data(column: &Column, dst: &mut Vec<u8>) {
    dst.extend_from_slice(&column.data);
    if column.data.is_empty() {
        return;
    }
    for child in &column.children {
        write_column_data(child, dst);
    }
}

fn write_fixed_header(dst: &mut Vec<u8>) {
    dst.extend_from_slice(HDR_SIGNATURE);
    append_uvarint(2, dst);
    dst.push(HDR_FORMAT_VERSION);
    dst.push(COMPRESSION_NONE);
}

fn write_var_header_frame(dst: &mut Vec<u8>) {
    let mut var_header = Vec::with_capacity(METRICS_WIRE_SCHEMA.len() + 4);
    append_uvarint(METRICS_WIRE_SCHEMA.len() as u64, &mut var_header);
    var_header.extend_from_slice(METRICS_WIRE_SCHEMA);
    append_uvarint(0, &mut var_header);
    write_frame(dst, FRAME_FLAG_NONE, &var_header);
}

fn write_frame(dst: &mut Vec<u8>, flags: u8, content: &[u8]) {
    dst.push(flags);
    append_uvarint(content.len() as u64, dst);
    dst.extend_from_slice(content);
}

fn append_uvarint(mut value: u64, dst: &mut Vec<u8>) {
    while value >= 0x80 {
        dst.push((value as u8) | 0x80);
        value >>= 7;
    }
    dst.push(value as u8);
}

#[derive(Default)]
struct MetricsOtapStreamDecoder {
    state: DirectDecoderState,
    builder: DirectOtapMetricsBuilder,
    current_resource: DirectDecResource,
    current_scope: DirectDecScope,
    current_metric: DirectDecMetric,
    current_attrs: Vec<DirectAttribute>,
    current_point: DecPoint,
}

impl MetricsOtapStreamDecoder {
    fn decode(bytes: &[u8]) -> Result<(OtapArrowRecords, u64), Error> {
        let mut input = SliceReader::new(bytes);
        read_fixed_header(&mut input)?;
        let (_, var_header) = read_frame(&mut input)?;
        read_var_header(var_header)?;

        let mut decoder = Self::default();
        let mut record_count = 0_u64;
        while !input.is_empty() {
            let (flags, frame) = read_frame(&mut input)?;
            if flags & !0b111 != 0 {
                return Err(Error::InvalidFrame("unknown frame flags"));
            }
            if flags & 0b001 != 0 {
                decoder.state.reset_dictionaries();
            }
            if flags & 0b100 != 0 {
                decoder.state.reset_codecs();
            }
            record_count = record_count.saturating_add(decoder.decode_data_frame(frame)?);
        }

        let records = decoder.builder.finish()?;
        Ok((records, record_count))
    }

    fn decode_data_frame(&mut self, frame: &[u8]) -> Result<u64, Error> {
        let mut reader = SliceReader::new(frame);
        let record_count = reader.read_uvarint()?;
        let size_len = usize::try_from(reader.read_uvarint()?)
            .map_err(|_| Error::ValueOutOfRange("column size buffer"))?;
        let size_bytes = reader.read_bytes(size_len)?;
        let mut columns = decode_metrics_column_tree();
        let mut size_reader = BitReader::new(size_bytes);
        read_decode_column_sizes(
            &mut columns,
            &mut size_reader,
            reader.remaining_len() as u64,
        )?;
        read_decode_column_data(&mut columns, &mut reader)?;
        self.builder.reserve_number_points(
            usize::try_from(record_count).map_err(|_| Error::ValueOutOfRange("record count"))?,
        );

        let mut frame_decoder = DirectMetricsFrameDecoder::new(&columns);
        for _ in 0..record_count {
            frame_decoder.decode_record(
                &mut self.builder,
                &mut self.current_resource,
                &mut self.current_scope,
                &mut self.current_metric,
                &mut self.current_attrs,
                &mut self.current_point,
                &mut self.state,
            )?;
        }
        Ok(record_count)
    }
}

struct DirectOtapMetricsBuilder {
    resource_attrs: AttributesRecordBatchBuilder<u16>,
    scope_attrs: AttributesRecordBatchBuilder<u16>,
    metric_attrs: AttributesRecordBatchBuilder<u16>,
    ndp_attrs: DirectNumberDpAttrsRecordBatchBuilder,
    metrics: MetricsRecordBatchBuilder,
    ndp: DirectNumberDataPointsRecordBatchBuilder,
    next_resource_id: u16,
    next_scope_id: u16,
    next_metric_id: u16,
    next_ndp_id: u32,
    current_resource_id: Option<u16>,
    current_scope_id: Option<u16>,
    current_metric_id: Option<u16>,
}

impl Default for DirectOtapMetricsBuilder {
    fn default() -> Self {
        Self {
            resource_attrs: AttributesRecordBatchBuilder::<u16>::new(),
            scope_attrs: AttributesRecordBatchBuilder::<u16>::new(),
            metric_attrs: AttributesRecordBatchBuilder::<u16>::new(),
            ndp_attrs: DirectNumberDpAttrsRecordBatchBuilder::default(),
            metrics: MetricsRecordBatchBuilder::new(),
            ndp: DirectNumberDataPointsRecordBatchBuilder::default(),
            next_resource_id: 0,
            next_scope_id: 0,
            next_metric_id: 0,
            next_ndp_id: 0,
            current_resource_id: None,
            current_scope_id: None,
            current_metric_id: None,
        }
    }
}

impl DirectOtapMetricsBuilder {
    fn reserve_number_points(&mut self, additional: usize) {
        self.ndp.reserve(additional);
        self.ndp_attrs.begin_frame(additional);
    }

    fn prepare_record(
        &mut self,
        modified: RootModified,
        resource: &DirectDecResource,
        scope: &DirectDecScope,
        metric: &DirectDecMetric,
    ) -> Result<u16, Error> {
        if self.current_resource_id.is_none() || modified.resource {
            self.start_resource(resource)?;
        }
        if self.current_scope_id.is_none() || modified.scope {
            self.start_scope(scope)?;
        }
        if self.current_metric_id.is_none() || modified.metric {
            self.start_metric(resource, scope, metric)?;
        }

        self.current_metric_id
            .ok_or(Error::InvalidFrame("metric id is initialized"))
    }

    fn append_number_point_row(&mut self, metric_id: u16, point_id: u32, point: &DecPoint) {
        self.ndp.append_id(point_id);
        self.ndp.append_parent_id(metric_id);
        self.ndp
            .append_start_time_unix_nano(point.start_timestamp as i64);
        self.ndp.append_time_unix_nano(point.timestamp as i64);
        match point.value {
            DecPointValue::None => {
                self.ndp.append_double_value(None);
                self.ndp.append_int_value(None);
                self.ndp.append_flags(1);
            }
            DecPointValue::Int64(value) => {
                self.ndp.append_double_value(None);
                self.ndp.append_int_value(Some(value));
                self.ndp.append_flags(0);
            }
            DecPointValue::Float64(value) => {
                self.ndp.append_double_value(Some(value));
                self.ndp.append_int_value(None);
                self.ndp.append_flags(0);
            }
        }
    }

    fn append_number_point_attrs(&mut self, point_id: u32, attrs: &[DirectAttribute]) {
        self.ndp_attrs.append_all(point_id, attrs);
    }

    fn start_resource(&mut self, resource: &DirectDecResource) -> Result<(), Error> {
        let resource_id = self.allocate_resource_id()?;
        self.current_resource_id = Some(resource_id);
        self.current_scope_id = None;
        self.current_metric_id = None;

        for kv in &resource.attributes {
            append_direct_attribute_with_parent(&mut self.resource_attrs, &resource_id, kv);
        }
        Ok(())
    }

    fn start_scope(&mut self, scope: &DirectDecScope) -> Result<(), Error> {
        let scope_id = self.allocate_scope_id()?;
        self.current_scope_id = Some(scope_id);
        self.current_metric_id = None;

        for kv in &scope.attributes {
            append_direct_attribute_with_parent(&mut self.scope_attrs, &scope_id, kv);
        }
        Ok(())
    }

    fn start_metric(
        &mut self,
        resource: &DirectDecResource,
        scope: &DirectDecScope,
        metric: &DirectDecMetric,
    ) -> Result<(), Error> {
        let metric_id = self.allocate_metric_id()?;
        let resource_id = self
            .current_resource_id
            .expect("resource id is initialized before metric");
        let scope_id = self
            .current_scope_id
            .expect("scope id is initialized before metric");

        self.metrics.append_id(metric_id);
        self.metrics.resource.append_id(Some(resource_id));
        self.metrics.resource.append_schema_url(
            (!resource.schema_url.is_empty()).then_some(resource.schema_url.as_bytes()),
        );
        self.metrics
            .resource
            .append_dropped_attributes_count(resource.dropped_attributes_count);
        self.metrics.scope.append_id(Some(scope_id));
        self.metrics
            .scope
            .append_name((!scope.name.is_empty()).then_some(scope.name.as_bytes()));
        self.metrics
            .scope
            .append_version((!scope.version.is_empty()).then_some(scope.version.as_bytes()));
        self.metrics
            .scope
            .append_dropped_attributes_count(scope.dropped_attributes_count);
        self.metrics
            .append_scope_schema_url(scope.schema_url.as_bytes());
        self.metrics.append_metric_type(match metric.r#type {
            0 => metrics_view::DataType::Gauge as u8,
            1 => metrics_view::DataType::Sum as u8,
            _ => return Err(Error::UnsupportedStefValue("metric type")),
        });
        self.metrics.append_name(metric.name.as_bytes());
        self.metrics
            .append_description(metric.description.as_bytes());
        self.metrics.append_unit(metric.unit.as_bytes());
        if metric.r#type == 1 {
            self.metrics
                .append_aggregation_temporality(Some(metric.aggregation_temporality as i32));
            self.metrics.append_is_monotonic(Some(metric.monotonic));
        } else {
            self.metrics.append_aggregation_temporality(None);
            self.metrics.append_is_monotonic(None);
        }

        for kv in &metric.metadata {
            append_direct_attribute_with_parent(&mut self.metric_attrs, &metric_id, kv);
        }

        self.current_metric_id = Some(metric_id);
        Ok(())
    }

    fn finish(mut self) -> Result<OtapArrowRecords, Error> {
        if self.next_metric_id == 0 {
            return Ok(OtapArrowRecords::Metrics(Metrics::default()));
        }

        let mut otap_batch = OtapArrowRecords::Metrics(Metrics::default());
        let pairs = [
            (
                self.metrics
                    .finish()
                    .map_err(|e| Error::OtapEncode(e.to_string()))?,
                ArrowPayloadType::UnivariateMetrics,
            ),
            (
                self.metric_attrs
                    .finish()
                    .map_err(|e| Error::OtapEncode(e.to_string()))?,
                ArrowPayloadType::MetricAttrs,
            ),
            (
                self.resource_attrs
                    .finish()
                    .map_err(|e| Error::OtapEncode(e.to_string()))?,
                ArrowPayloadType::ResourceAttrs,
            ),
            (
                self.scope_attrs
                    .finish()
                    .map_err(|e| Error::OtapEncode(e.to_string()))?,
                ArrowPayloadType::ScopeAttrs,
            ),
            (
                self.ndp
                    .finish()
                    .map_err(|e| Error::OtapEncode(e.to_string()))?,
                ArrowPayloadType::NumberDataPoints,
            ),
            (
                self.ndp_attrs
                    .finish()
                    .map_err(|e| Error::OtapEncode(e.to_string()))?,
                ArrowPayloadType::NumberDpAttrs,
            ),
        ];
        for (record_batch, payload_type) in pairs {
            if record_batch.num_rows() > 0 {
                otap_batch
                    .set(payload_type, record_batch)
                    .map_err(|e| Error::OtapEncode(e.to_string()))?;
            }
        }
        Ok(otap_batch)
    }

    fn allocate_resource_id(&mut self) -> Result<u16, Error> {
        let id = self.next_resource_id;
        self.next_resource_id = self
            .next_resource_id
            .checked_add(1)
            .ok_or(Error::ValueOutOfRange("resource id"))?;
        Ok(id)
    }

    fn allocate_scope_id(&mut self) -> Result<u16, Error> {
        let id = self.next_scope_id;
        self.next_scope_id = self
            .next_scope_id
            .checked_add(1)
            .ok_or(Error::ValueOutOfRange("scope id"))?;
        Ok(id)
    }

    fn allocate_metric_id(&mut self) -> Result<u16, Error> {
        let id = self.next_metric_id;
        self.next_metric_id = self
            .next_metric_id
            .checked_add(1)
            .ok_or(Error::ValueOutOfRange("metric id"))?;
        Ok(id)
    }

    fn allocate_number_point_id(&mut self) -> Result<u32, Error> {
        let id = self.next_ndp_id;
        self.next_ndp_id = self
            .next_ndp_id
            .checked_add(1)
            .ok_or(Error::ValueOutOfRange("number data point id"))?;
        Ok(id)
    }
}

#[derive(Default)]
struct DirectNumberDataPointsRecordBatchBuilder {
    id: Vec<u32>,
    parent_id: Vec<u16>,
    start_time_unix_nano: Vec<i64>,
    time_unix_nano: Vec<i64>,
    int_value: Vec<Option<i64>>,
    double_value: Vec<Option<f64>>,
    flags: Vec<u32>,
    has_nonzero_flags: bool,
}

impl DirectNumberDataPointsRecordBatchBuilder {
    fn reserve(&mut self, additional: usize) {
        self.id.reserve(additional);
        self.parent_id.reserve(additional);
        self.start_time_unix_nano.reserve(additional);
        self.time_unix_nano.reserve(additional);
        self.int_value.reserve(additional);
        self.double_value.reserve(additional);
        self.flags.reserve(additional);
    }

    fn append_id(&mut self, value: u32) {
        self.id.push(value);
    }

    fn append_parent_id(&mut self, value: u16) {
        self.parent_id.push(value);
    }

    fn append_start_time_unix_nano(&mut self, value: i64) {
        self.start_time_unix_nano.push(value);
    }

    fn append_time_unix_nano(&mut self, value: i64) {
        self.time_unix_nano.push(value);
    }

    fn append_int_value(&mut self, value: Option<i64>) {
        self.int_value.push(value);
    }

    fn append_double_value(&mut self, value: Option<f64>) {
        self.double_value.push(value);
    }

    fn append_flags(&mut self, value: u32) {
        self.has_nonzero_flags |= value != 0;
        self.flags.push(value);
    }

    fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
        let mut fields = Vec::with_capacity(7);
        let mut columns = Vec::with_capacity(7);

        let array = Arc::new(UInt32Array::from(std::mem::take(&mut self.id))) as ArrayRef;
        fields.push(Field::new(consts::ID, array.data_type().clone(), false).with_plain_encoding());
        columns.push(array);

        let array = Arc::new(UInt16Array::from(std::mem::take(&mut self.parent_id))) as ArrayRef;
        fields.push(
            Field::new(consts::PARENT_ID, array.data_type().clone(), false).with_plain_encoding(),
        );
        columns.push(array);

        let array = Arc::new(TimestampNanosecondArray::from(std::mem::take(
            &mut self.start_time_unix_nano,
        ))) as ArrayRef;
        fields.push(Field::new(
            consts::START_TIME_UNIX_NANO,
            array.data_type().clone(),
            true,
        ));
        columns.push(array);

        let array = Arc::new(TimestampNanosecondArray::from(std::mem::take(
            &mut self.time_unix_nano,
        ))) as ArrayRef;
        fields.push(Field::new(
            consts::TIME_UNIX_NANO,
            array.data_type().clone(),
            false,
        ));
        columns.push(array);

        let array = Arc::new(Int64Array::from(std::mem::take(&mut self.int_value))) as ArrayRef;
        fields.push(Field::new(
            consts::INT_VALUE,
            array.data_type().clone(),
            true,
        ));
        columns.push(array);

        let array =
            Arc::new(Float64Array::from(std::mem::take(&mut self.double_value))) as ArrayRef;
        fields.push(Field::new(
            consts::DOUBLE_VALUE,
            array.data_type().clone(),
            true,
        ));
        columns.push(array);

        if self.has_nonzero_flags {
            let array = Arc::new(UInt32Array::from(std::mem::take(&mut self.flags))) as ArrayRef;
            fields.push(Field::new(consts::FLAGS, array.data_type().clone(), false));
            columns.push(array);
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
    }
}

#[derive(Default)]
struct DirectNumberDpAttrsRecordBatchBuilder {
    parent_id: Vec<u32>,
    key: Vec<Rc<str>>,
    attr_type: Vec<u8>,
    str_value: Vec<Option<Rc<str>>>,
    int_value: Vec<Option<i64>>,
    double_value: Vec<Option<f64>>,
    bool_value: Vec<Option<bool>>,
    bytes_value: Vec<Option<Vec<u8>>>,
    pending_frame_points: Option<usize>,
    reserved_row_capacity: usize,
}

impl DirectNumberDpAttrsRecordBatchBuilder {
    fn begin_frame(&mut self, point_count: usize) {
        self.pending_frame_points = Some(point_count);
        self.reserve_additional_rows(point_count);
    }

    fn reserve_frame_rows_for_attr_count(&mut self, attrs_per_point: usize) {
        let Some(point_count) = self.pending_frame_points.take() else {
            return;
        };
        self.reserve_additional_rows(point_count.saturating_mul(attrs_per_point));
    }

    fn reserve_additional_rows(&mut self, additional: usize) {
        let target = self.parent_id.len().saturating_add(additional);
        if self.reserved_row_capacity < target {
            self.reserved_row_capacity = target;
        }
        reserve_vec_to(&mut self.parent_id, target);
        reserve_vec_to(&mut self.key, target);
        reserve_vec_to(&mut self.attr_type, target);
        if !self.str_value.is_empty() {
            reserve_vec_to(&mut self.str_value, target);
        }
        if !self.int_value.is_empty() {
            reserve_vec_to(&mut self.int_value, target);
        }
        if !self.double_value.is_empty() {
            reserve_vec_to(&mut self.double_value, target);
        }
        if !self.bool_value.is_empty() {
            reserve_vec_to(&mut self.bool_value, target);
        }
        if !self.bytes_value.is_empty() {
            reserve_vec_to(&mut self.bytes_value, target);
        }
    }

    fn append_all(&mut self, parent_id: u32, attrs: &[DirectAttribute]) {
        self.reserve_frame_rows_for_attr_count(attrs.len());
        for kv in attrs {
            self.append(parent_id, kv);
        }
    }

    fn append(&mut self, parent_id: u32, kv: &DirectAttribute) {
        let row_index = self.parent_id.len();
        self.parent_id.push(parent_id);
        self.key.push(kv.key.clone());

        match &kv.value {
            DirectAnyValue::Empty => {
                self.attr_type.push(AttributeValueType::Empty as u8);
                self.push_null_existing_values();
            }
            DirectAnyValue::String(value) => {
                self.attr_type.push(AttributeValueType::Str as u8);
                self.push_str_value(row_index, value.clone());
                self.push_null_int_value();
                self.push_null_double_value();
                self.push_null_bool_value();
                self.push_null_bytes_value();
            }
            DirectAnyValue::Bool(value) => {
                self.attr_type.push(AttributeValueType::Bool as u8);
                self.push_null_str_value();
                self.push_null_int_value();
                self.push_null_double_value();
                self.push_bool_value(row_index, *value);
                self.push_null_bytes_value();
            }
            DirectAnyValue::Int(value) => {
                self.attr_type.push(AttributeValueType::Int as u8);
                self.push_null_str_value();
                self.push_int_value(row_index, *value);
                self.push_null_double_value();
                self.push_null_bool_value();
                self.push_null_bytes_value();
            }
            DirectAnyValue::Double(value) => {
                self.attr_type.push(AttributeValueType::Double as u8);
                self.push_null_str_value();
                self.push_null_int_value();
                self.push_double_value(row_index, *value);
                self.push_null_bool_value();
                self.push_null_bytes_value();
            }
            DirectAnyValue::Bytes(value) => {
                self.attr_type.push(AttributeValueType::Bytes as u8);
                self.push_null_str_value();
                self.push_null_int_value();
                self.push_null_double_value();
                self.push_null_bool_value();
                self.push_bytes_value(row_index, value.clone());
            }
        }
    }

    fn push_null_existing_values(&mut self) {
        self.push_null_str_value();
        self.push_null_int_value();
        self.push_null_double_value();
        self.push_null_bool_value();
        self.push_null_bytes_value();
    }

    fn push_str_value(&mut self, row_index: usize, value: Rc<str>) {
        if self.str_value.is_empty() {
            self.str_value.resize(row_index, None);
            reserve_vec_to(&mut self.str_value, self.reserved_row_capacity);
        }
        self.str_value.push(Some(value));
    }

    fn push_null_str_value(&mut self) {
        if !self.str_value.is_empty() {
            self.str_value.push(None);
        }
    }

    fn push_int_value(&mut self, row_index: usize, value: i64) {
        if self.int_value.is_empty() {
            self.int_value.resize(row_index, None);
            reserve_vec_to(&mut self.int_value, self.reserved_row_capacity);
        }
        self.int_value.push(Some(value));
    }

    fn push_null_int_value(&mut self) {
        if !self.int_value.is_empty() {
            self.int_value.push(None);
        }
    }

    fn push_double_value(&mut self, row_index: usize, value: f64) {
        if self.double_value.is_empty() {
            self.double_value.resize(row_index, None);
            reserve_vec_to(&mut self.double_value, self.reserved_row_capacity);
        }
        self.double_value.push(Some(value));
    }

    fn push_null_double_value(&mut self) {
        if !self.double_value.is_empty() {
            self.double_value.push(None);
        }
    }

    fn push_bool_value(&mut self, row_index: usize, value: bool) {
        if self.bool_value.is_empty() {
            self.bool_value.resize(row_index, None);
            reserve_vec_to(&mut self.bool_value, self.reserved_row_capacity);
        }
        self.bool_value.push(Some(value));
    }

    fn push_null_bool_value(&mut self) {
        if !self.bool_value.is_empty() {
            self.bool_value.push(None);
        }
    }

    fn push_bytes_value(&mut self, row_index: usize, value: Vec<u8>) {
        if self.bytes_value.is_empty() {
            self.bytes_value.resize(row_index, None);
            reserve_vec_to(&mut self.bytes_value, self.reserved_row_capacity);
        }
        self.bytes_value.push(Some(value));
    }

    fn push_null_bytes_value(&mut self) {
        if !self.bytes_value.is_empty() {
            self.bytes_value.push(None);
        }
    }

    fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
        if self.parent_id.is_empty() {
            return Ok(RecordBatch::new_empty(Arc::new(Schema::empty())));
        }

        let mut fields = Vec::with_capacity(8);
        let mut columns = Vec::with_capacity(8);

        let array = Arc::new(UInt32Array::from(std::mem::take(&mut self.parent_id))) as ArrayRef;
        fields.push(
            Field::new(consts::PARENT_ID, array.data_type().clone(), false).with_plain_encoding(),
        );
        columns.push(array);

        let array = Arc::new(StringArray::from_iter_values(
            self.key.iter().map(Rc::as_ref),
        )) as ArrayRef;
        self.key.clear();
        fields.push(Field::new(
            consts::ATTRIBUTE_KEY,
            array.data_type().clone(),
            false,
        ));
        columns.push(array);

        let array = Arc::new(UInt8Array::from(std::mem::take(&mut self.attr_type))) as ArrayRef;
        fields.push(Field::new(
            consts::ATTRIBUTE_TYPE,
            array.data_type().clone(),
            false,
        ));
        columns.push(array);

        if !self.str_value.is_empty() {
            let array = Arc::new(StringArray::from_iter(
                self.str_value.iter().map(Option::as_deref),
            )) as ArrayRef;
            self.str_value.clear();
            fields.push(Field::new(
                consts::ATTRIBUTE_STR,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if !self.int_value.is_empty() {
            let array = Arc::new(Int64Array::from(std::mem::take(&mut self.int_value))) as ArrayRef;
            fields.push(Field::new(
                consts::ATTRIBUTE_INT,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if !self.double_value.is_empty() {
            let array =
                Arc::new(Float64Array::from(std::mem::take(&mut self.double_value))) as ArrayRef;
            fields.push(Field::new(
                consts::ATTRIBUTE_DOUBLE,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if !self.bool_value.is_empty() {
            let array =
                Arc::new(BooleanArray::from(std::mem::take(&mut self.bool_value))) as ArrayRef;
            fields.push(Field::new(
                consts::ATTRIBUTE_BOOL,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if !self.bytes_value.is_empty() {
            let array = Arc::new(BinaryArray::from_iter(
                self.bytes_value.iter().map(Option::as_deref),
            )) as ArrayRef;
            self.bytes_value.clear();
            fields.push(Field::new(
                consts::ATTRIBUTE_BYTES,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
    }
}

fn reserve_vec_to<T>(values: &mut Vec<T>, target_capacity: usize) {
    if values.capacity() < target_capacity {
        values.reserve(target_capacity - values.capacity());
    }
}

fn append_direct_attribute_with_parent<T>(
    attribute_rb_builder: &mut AttributesRecordBatchBuilder<T>,
    parent_id: &<<T as crate::otlp::attributes::parent_id::ParentId>::ArrayType as ArrowPrimitiveType>::Native,
    kv: &DirectAttribute,
) where
    T: crate::otlp::attributes::parent_id::ParentId + AttributesRecordBatchBuilderConstructorHelper,
{
    attribute_rb_builder.append_parent_id(parent_id);
    append_direct_attribute_value(attribute_rb_builder, kv);
}

fn append_direct_attribute_value<T>(
    attribute_rb_builder: &mut AttributesRecordBatchBuilder<T>,
    kv: &DirectAttribute,
) where
    T: crate::otlp::attributes::parent_id::ParentId + AttributesRecordBatchBuilderConstructorHelper,
{
    attribute_rb_builder.append_key(kv.key.as_bytes());
    match &kv.value {
        DirectAnyValue::Empty => attribute_rb_builder.any_values_builder.append_empty(),
        DirectAnyValue::String(value) => attribute_rb_builder
            .any_values_builder
            .append_str(value.as_bytes()),
        DirectAnyValue::Bool(value) => {
            attribute_rb_builder.any_values_builder.append_bool(*value);
        }
        DirectAnyValue::Int(value) => {
            attribute_rb_builder.any_values_builder.append_int(*value);
        }
        DirectAnyValue::Double(value) => {
            attribute_rb_builder
                .any_values_builder
                .append_double(*value);
        }
        DirectAnyValue::Bytes(value) => {
            attribute_rb_builder.any_values_builder.append_bytes(value);
        }
    }
}

#[derive(Default)]
struct DirectMetricsFrameDecoder<'a> {
    root: BitReader<'a>,
    metric: DirectMetricDecoder<'a>,
    resource: DirectResourceDecoder<'a>,
    scope: DirectScopeDecoder<'a>,
    attributes: AttributesDecoder<'a>,
    point: PointDecoder<'a>,
}

impl<'a> DirectMetricsFrameDecoder<'a> {
    fn new(columns: &'a DecodeColumn<'a>) -> Self {
        Self {
            root: BitReader::new(columns.data),
            metric: DirectMetricDecoder::new(&columns.children[1]),
            resource: DirectResourceDecoder::new(&columns.children[2]),
            scope: DirectScopeDecoder::new(&columns.children[3]),
            attributes: AttributesDecoder::new(&columns.children[4]),
            point: PointDecoder::new(&columns.children[5]),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_record(
        &mut self,
        builder: &mut DirectOtapMetricsBuilder,
        resource: &mut DirectDecResource,
        scope: &mut DirectDecScope,
        metric: &mut DirectDecMetric,
        attrs: &mut Vec<DirectAttribute>,
        point: &mut DecPoint,
        state: &mut DirectDecoderState,
    ) -> Result<(), Error> {
        let mask = self.root.read_bits(6)?;
        let modified = RootModified {
            metric: mask & (1 << 1) != 0,
            resource: mask & (1 << 2) != 0,
            scope: mask & (1 << 3) != 0,
        };

        if mask & (1 << 1) != 0 {
            self.metric.decode(metric, state)?;
        }
        if mask & (1 << 2) != 0 {
            self.resource.decode(resource, state)?;
        }
        if mask & (1 << 3) != 0 {
            self.scope.decode(scope, state)?;
        }
        let metric_id = builder.prepare_record(modified, resource, scope, metric)?;
        let point_id = builder.allocate_number_point_id()?;
        if mask & (1 << 4) != 0 {
            self.attributes.decode_direct_number_point_attrs(
                attrs,
                state,
                point_id,
                &mut builder.ndp_attrs,
            )?;
        } else {
            builder.append_number_point_attrs(point_id, attrs);
        }
        if mask & (1 << 5) != 0 {
            self.point.decode(point)?;
        }
        builder.append_number_point_row(metric_id, point_id, point);
        Ok(())
    }
}

#[derive(Clone, Copy, Default)]
struct RootModified {
    metric: bool,
    resource: bool,
    scope: bool,
}

struct DirectDecoderState {
    schema_url: DirectStringDict,
    metric_name: DirectStringDict,
    metric_description: DirectStringDict,
    metric_unit: DirectStringDict,
    scope_name: DirectStringDict,
    scope_version: DirectStringDict,
    attribute_key: DirectStringDict,
    any_value_string: DirectStringDict,
    resources: Vec<DirectDecResource>,
    scopes: Vec<DirectDecScope>,
    metrics: Vec<DirectDecMetric>,
}

impl Default for DirectDecoderState {
    fn default() -> Self {
        let mut state = Self {
            schema_url: DirectStringDict::default(),
            metric_name: DirectStringDict::default(),
            metric_description: DirectStringDict::default(),
            metric_unit: DirectStringDict::default(),
            scope_name: DirectStringDict::default(),
            scope_version: DirectStringDict::default(),
            attribute_key: DirectStringDict::default(),
            any_value_string: DirectStringDict::default(),
            resources: Vec::new(),
            scopes: Vec::new(),
            metrics: Vec::new(),
        };
        state.reset_dictionaries();
        state
    }
}

impl DirectDecoderState {
    fn reset_dictionaries(&mut self) {
        self.schema_url.reset();
        self.metric_name.reset();
        self.metric_description.reset();
        self.metric_unit.reset();
        self.scope_name.reset();
        self.scope_version.reset();
        self.attribute_key.reset();
        self.any_value_string.reset();
        self.resources.clear();
        self.resources.push(DirectDecResource::default());
        self.scopes.clear();
        self.scopes.push(DirectDecScope::default());
        self.metrics.clear();
        self.metrics.push(DirectDecMetric::default());
    }

    fn reset_codecs(&mut self) {}
}

#[derive(Clone, Default)]
struct DecPoint {
    start_timestamp: u64,
    timestamp: u64,
    value: DecPointValue,
}

#[derive(Clone, Copy, Default)]
enum DecPointValue {
    #[default]
    None,
    Int64(i64),
    Float64(f64),
}

#[derive(Clone)]
struct DirectDecResource {
    schema_url: Rc<str>,
    attributes: Vec<DirectAttribute>,
    dropped_attributes_count: u32,
}

impl Default for DirectDecResource {
    fn default() -> Self {
        Self {
            schema_url: empty_rc_str(),
            attributes: Vec::new(),
            dropped_attributes_count: 0,
        }
    }
}

#[derive(Clone)]
struct DirectDecScope {
    name: Rc<str>,
    version: Rc<str>,
    schema_url: Rc<str>,
    attributes: Vec<DirectAttribute>,
    dropped_attributes_count: u32,
}

impl Default for DirectDecScope {
    fn default() -> Self {
        Self {
            name: empty_rc_str(),
            version: empty_rc_str(),
            schema_url: empty_rc_str(),
            attributes: Vec::new(),
            dropped_attributes_count: 0,
        }
    }
}

#[derive(Clone)]
struct DirectDecMetric {
    name: Rc<str>,
    description: Rc<str>,
    unit: Rc<str>,
    r#type: u64,
    metadata: Vec<DirectAttribute>,
    aggregation_temporality: u64,
    monotonic: bool,
}

impl Default for DirectDecMetric {
    fn default() -> Self {
        Self {
            name: empty_rc_str(),
            description: empty_rc_str(),
            unit: empty_rc_str(),
            r#type: 0,
            metadata: Vec::new(),
            aggregation_temporality: 0,
            monotonic: false,
        }
    }
}

#[derive(Clone, PartialEq)]
struct DirectAttribute {
    key: Rc<str>,
    value: DirectAnyValue,
}

impl Default for DirectAttribute {
    fn default() -> Self {
        Self {
            key: empty_rc_str(),
            value: DirectAnyValue::Empty,
        }
    }
}

#[derive(Clone, Default, PartialEq)]
enum DirectAnyValue {
    #[default]
    Empty,
    String(Rc<str>),
    Bool(bool),
    Int(i64),
    Double(f64),
    Bytes(Vec<u8>),
}

fn empty_rc_str() -> Rc<str> {
    Rc::<str>::from("")
}

#[derive(Default)]
struct DirectMetricDecoder<'a> {
    bits: BitReader<'a>,
    name: BytesReader<'a>,
    description: BytesReader<'a>,
    unit: BytesReader<'a>,
    r#type: U64Decoder<'a>,
    metadata: AttributesDecoder<'a>,
    histogram_bounds: ArrayDecoder<'a>,
    aggregation_temporality: U64Decoder<'a>,
    monotonic: BoolDecoder<'a>,
}

impl<'a> DirectMetricDecoder<'a> {
    fn new(column: &'a DecodeColumn<'a>) -> Self {
        Self {
            bits: BitReader::new(column.data),
            name: BytesReader::new(column.children[0].data),
            description: BytesReader::new(column.children[1].data),
            unit: BytesReader::new(column.children[2].data),
            r#type: U64Decoder::new(column.children[3].data),
            metadata: AttributesDecoder::new(&column.children[4]),
            histogram_bounds: ArrayDecoder::new(&column.children[5]),
            aggregation_temporality: U64Decoder::new(column.children[6].data),
            monotonic: BoolDecoder::new(column.children[7].data),
        }
    }

    fn decode(
        &mut self,
        target: &mut DirectDecMetric,
        state: &mut DirectDecoderState,
    ) -> Result<(), Error> {
        if !self.bits.read_bit()? {
            let ref_num = usize::try_from(self.bits.read_uvarint_compact()?)
                .map_err(|_| Error::InvalidRefNum)?;
            let value = state
                .metrics
                .get(ref_num)
                .ok_or(Error::InvalidRefNum)?
                .clone();
            *target = value;
            return Ok(());
        }

        let mut value = target.clone();
        let mask = self.bits.read_bits(8)?;
        if mask & (1 << 0) != 0 {
            value.name = self.name.read_direct_dict_string(&mut state.metric_name)?;
        }
        if mask & (1 << 1) != 0 {
            value.description = self
                .description
                .read_direct_dict_string(&mut state.metric_description)?;
        }
        if mask & (1 << 2) != 0 {
            value.unit = self.unit.read_direct_dict_string(&mut state.metric_unit)?;
        }
        if mask & (1 << 3) != 0 {
            value.r#type = self.r#type.decode()?;
        }
        if mask & (1 << 4) != 0 {
            self.metadata.decode_direct(&mut value.metadata, state)?;
        }
        if mask & (1 << 5) != 0 {
            self.histogram_bounds.decode_empty()?;
        }
        if mask & (1 << 6) != 0 {
            value.aggregation_temporality = self.aggregation_temporality.decode()?;
        }
        if mask & (1 << 7) != 0 {
            value.monotonic = self.monotonic.decode()?;
        }
        state.metrics.push(value.clone());
        *target = value;
        Ok(())
    }
}

#[derive(Default)]
struct DirectResourceDecoder<'a> {
    bits: BitReader<'a>,
    schema_url: BytesReader<'a>,
    attributes: AttributesDecoder<'a>,
    dropped_attributes_count: U64Decoder<'a>,
}

impl<'a> DirectResourceDecoder<'a> {
    fn new(column: &'a DecodeColumn<'a>) -> Self {
        Self {
            bits: BitReader::new(column.data),
            schema_url: BytesReader::new(column.children[0].data),
            attributes: AttributesDecoder::new(&column.children[1]),
            dropped_attributes_count: U64Decoder::new(column.children[2].data),
        }
    }

    fn decode(
        &mut self,
        target: &mut DirectDecResource,
        state: &mut DirectDecoderState,
    ) -> Result<(), Error> {
        if !self.bits.read_bit()? {
            let ref_num = usize::try_from(self.bits.read_uvarint_compact()?)
                .map_err(|_| Error::InvalidRefNum)?;
            let value = state
                .resources
                .get(ref_num)
                .ok_or(Error::InvalidRefNum)?
                .clone();
            *target = value;
            return Ok(());
        }

        let mut value = target.clone();
        let mask = self.bits.read_bits(3)?;
        if mask & (1 << 0) != 0 {
            value.schema_url = self
                .schema_url
                .read_direct_dict_string(&mut state.schema_url)?;
        }
        if mask & (1 << 1) != 0 {
            self.attributes
                .decode_direct(&mut value.attributes, state)?;
        }
        if mask & (1 << 2) != 0 {
            value.dropped_attributes_count = u32::try_from(self.dropped_attributes_count.decode()?)
                .map_err(|_| Error::ValueOutOfRange("dropped_attributes_count"))?;
        }
        state.resources.push(value.clone());
        *target = value;
        Ok(())
    }
}

#[derive(Default)]
struct DirectScopeDecoder<'a> {
    bits: BitReader<'a>,
    name: BytesReader<'a>,
    version: BytesReader<'a>,
    schema_url: BytesReader<'a>,
    attributes: AttributesDecoder<'a>,
    dropped_attributes_count: U64Decoder<'a>,
}

impl<'a> DirectScopeDecoder<'a> {
    fn new(column: &'a DecodeColumn<'a>) -> Self {
        Self {
            bits: BitReader::new(column.data),
            name: BytesReader::new(column.children[0].data),
            version: BytesReader::new(column.children[1].data),
            schema_url: BytesReader::new(column.children[2].data),
            attributes: AttributesDecoder::new(&column.children[3]),
            dropped_attributes_count: U64Decoder::new(column.children[4].data),
        }
    }

    fn decode(
        &mut self,
        target: &mut DirectDecScope,
        state: &mut DirectDecoderState,
    ) -> Result<(), Error> {
        if !self.bits.read_bit()? {
            let ref_num = usize::try_from(self.bits.read_uvarint_compact()?)
                .map_err(|_| Error::InvalidRefNum)?;
            let value = state
                .scopes
                .get(ref_num)
                .ok_or(Error::InvalidRefNum)?
                .clone();
            *target = value;
            return Ok(());
        }

        let mut value = target.clone();
        let mask = self.bits.read_bits(5)?;
        if mask & (1 << 0) != 0 {
            value.name = self.name.read_direct_dict_string(&mut state.scope_name)?;
        }
        if mask & (1 << 1) != 0 {
            value.version = self
                .version
                .read_direct_dict_string(&mut state.scope_version)?;
        }
        if mask & (1 << 2) != 0 {
            value.schema_url = self
                .schema_url
                .read_direct_dict_string(&mut state.schema_url)?;
        }
        if mask & (1 << 3) != 0 {
            self.attributes
                .decode_direct(&mut value.attributes, state)?;
        }
        if mask & (1 << 4) != 0 {
            value.dropped_attributes_count = u32::try_from(self.dropped_attributes_count.decode()?)
                .map_err(|_| Error::ValueOutOfRange("dropped_attributes_count"))?;
        }
        state.scopes.push(value.clone());
        *target = value;
        Ok(())
    }
}

#[derive(Default)]
struct PointDecoder<'a> {
    bits: BitReader<'a>,
    start_timestamp: U64Decoder<'a>,
    timestamp: U64Decoder<'a>,
    value: PointValueDecoder<'a>,
    exemplars: ArrayDecoder<'a>,
}

impl<'a> PointDecoder<'a> {
    fn new(column: &'a DecodeColumn<'a>) -> Self {
        Self {
            bits: BitReader::new(column.data),
            start_timestamp: U64Decoder::new(column.children[0].data),
            timestamp: U64Decoder::new(column.children[1].data),
            value: PointValueDecoder::new(&column.children[2]),
            exemplars: ArrayDecoder::new(&column.children[3]),
        }
    }

    fn decode(&mut self, target: &mut DecPoint) -> Result<(), Error> {
        let mask = self.bits.read_bits(4)?;
        if mask & (1 << 0) != 0 {
            target.start_timestamp = self.start_timestamp.decode()?;
        }
        if mask & (1 << 1) != 0 {
            target.timestamp = self.timestamp.decode()?;
        }
        if mask & (1 << 2) != 0 {
            target.value = self.value.decode()?;
        }
        if mask & (1 << 3) != 0 {
            self.exemplars.decode_empty()?;
        }
        Ok(())
    }
}

#[derive(Default)]
struct PointValueDecoder<'a> {
    bits: BitReader<'a>,
    int64: I64Decoder<'a>,
    float64: Float64Decoder<'a>,
}

impl<'a> PointValueDecoder<'a> {
    fn new(column: &'a DecodeColumn<'a>) -> Self {
        Self {
            bits: BitReader::new(column.data),
            int64: I64Decoder::new(column.children[0].data),
            float64: Float64Decoder::new(column.children[1].data),
        }
    }

    fn decode(&mut self) -> Result<DecPointValue, Error> {
        match self.bits.read_bits(3)? {
            0 => Ok(DecPointValue::None),
            1 => Ok(DecPointValue::Int64(self.int64.decode()?)),
            2 => Ok(DecPointValue::Float64(self.float64.decode()?)),
            _ => Err(Error::UnsupportedStefValue("point value")),
        }
    }
}

#[derive(Default)]
struct AttributesDecoder<'a> {
    header: BytesReader<'a>,
    key: BytesReader<'a>,
    value: AnyValueDecoder<'a>,
}

impl<'a> AttributesDecoder<'a> {
    fn new(column: &'a DecodeColumn<'a>) -> Self {
        Self {
            header: BytesReader::new(column.data),
            key: BytesReader::new(column.children[0].data),
            value: AnyValueDecoder::new(&column.children[1]),
        }
    }

    fn decode_direct(
        &mut self,
        target: &mut Vec<DirectAttribute>,
        state: &mut DirectDecoderState,
    ) -> Result<(), Error> {
        let count_or_changed = self.header.read_uvarint()?;
        if count_or_changed == 0 {
            return Ok(());
        }

        if count_or_changed & 1 == 0 {
            let changed = count_or_changed >> 1;
            for (index, item) in target.iter_mut().enumerate() {
                if changed & (1 << index) != 0 {
                    item.value = self.value.decode_direct(state)?;
                }
            }
            return Ok(());
        }
        let count = usize::try_from(count_or_changed >> 1)
            .map_err(|_| Error::ValueOutOfRange("attributes"))?;
        target.clear();
        target.reserve(count);
        for _ in 0..count {
            let key = self.key.read_direct_dict_string(&mut state.attribute_key)?;
            let value = self.value.decode_direct(state)?;
            target.push(DirectAttribute { key, value });
        }
        Ok(())
    }

    fn decode_direct_number_point_attrs(
        &mut self,
        target: &mut Vec<DirectAttribute>,
        state: &mut DirectDecoderState,
        point_id: u32,
        attribute_rb_builder: &mut DirectNumberDpAttrsRecordBatchBuilder,
    ) -> Result<(), Error> {
        let count_or_changed = self.header.read_uvarint()?;
        if count_or_changed == 0 {
            attribute_rb_builder.append_all(point_id, target);
            return Ok(());
        }

        if count_or_changed & 1 == 0 {
            let changed = count_or_changed >> 1;
            for (index, item) in target.iter_mut().enumerate() {
                if changed & (1 << index) != 0 {
                    item.value = self.value.decode_direct(state)?;
                }
            }
            attribute_rb_builder.append_all(point_id, target);
            return Ok(());
        }

        let count = usize::try_from(count_or_changed >> 1)
            .map_err(|_| Error::ValueOutOfRange("attributes"))?;
        attribute_rb_builder.reserve_frame_rows_for_attr_count(count);
        target.clear();
        target.reserve(count);
        for _ in 0..count {
            let key = self.key.read_direct_dict_string(&mut state.attribute_key)?;
            let value = self.value.decode_direct(state)?;
            let attribute = DirectAttribute { key, value };
            attribute_rb_builder.append(point_id, &attribute);
            target.push(attribute);
        }
        Ok(())
    }
}

#[derive(Default)]
struct AnyValueDecoder<'a> {
    bits: BitReader<'a>,
    string: BytesReader<'a>,
    bool_: BoolDecoder<'a>,
    int64: I64Decoder<'a>,
    float64: Float64Decoder<'a>,
    bytes: BytesReader<'a>,
}

impl<'a> AnyValueDecoder<'a> {
    fn new(column: &'a DecodeColumn<'a>) -> Self {
        Self {
            bits: BitReader::new(column.data),
            string: BytesReader::new(column.children[0].data),
            bool_: BoolDecoder::new(column.children[1].data),
            int64: I64Decoder::new(column.children[2].data),
            float64: Float64Decoder::new(column.children[3].data),
            bytes: BytesReader::new(column.children[6].data),
        }
    }

    fn decode_direct(&mut self, state: &mut DirectDecoderState) -> Result<DirectAnyValue, Error> {
        match self.bits.read_bits(4)? {
            0 => Ok(DirectAnyValue::Empty),
            1 => Ok(DirectAnyValue::String(
                self.string
                    .read_direct_dict_string(&mut state.any_value_string)?,
            )),
            2 => Ok(DirectAnyValue::Bool(self.bool_.decode()?)),
            3 => Ok(DirectAnyValue::Int(self.int64.decode()?)),
            4 => Ok(DirectAnyValue::Double(self.float64.decode()?)),
            5 => Err(Error::UnsupportedStefValue("array attribute")),
            6 => Err(Error::UnsupportedStefValue("kvlist attribute")),
            7 => Ok(DirectAnyValue::Bytes(self.bytes.read_plain_bytes()?)),
            _ => Err(Error::UnsupportedStefValue("attribute value")),
        }
    }
}

#[derive(Default)]
struct ArrayDecoder<'a> {
    bits: BitReader<'a>,
}

impl<'a> ArrayDecoder<'a> {
    fn new(column: &'a DecodeColumn<'a>) -> Self {
        Self {
            bits: BitReader::new(column.data),
        }
    }

    fn decode_empty(&mut self) -> Result<(), Error> {
        let len = self.bits.read_uvarint_compact()?;
        if len == 0 {
            Ok(())
        } else {
            Err(Error::UnsupportedStefValue("non-empty array"))
        }
    }
}

#[derive(Default)]
struct DirectStringDict {
    values: Vec<Rc<str>>,
}

impl DirectStringDict {
    fn reset(&mut self) {
        self.values.clear();
    }

    fn decode(&mut self, value: i64, reader: &mut BytesReader<'_>) -> Result<Rc<str>, Error> {
        if value >= 0 {
            let len = usize::try_from(value).map_err(|_| Error::ValueOutOfRange("string"))?;
            let value = reader.read_shared_utf8_string(len)?;
            if len > 1 {
                self.values.push(value.clone());
            }
            return Ok(value);
        }
        let ref_num = usize::try_from(-value - 1).map_err(|_| Error::InvalidRefNum)?;
        self.values
            .get(ref_num)
            .cloned()
            .ok_or(Error::InvalidRefNum)
    }
}

#[derive(Default)]
struct BytesReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> BytesReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn read_uvarint(&mut self) -> Result<u64, Error> {
        read_uvarint_from_slice(self.bytes, &mut self.position)
    }

    fn read_varint(&mut self) -> Result<i64, Error> {
        let value = self.read_uvarint()?;
        Ok(((value >> 1) as i64) ^ (-((value & 1) as i64)))
    }

    fn read_direct_dict_string(&mut self, dict: &mut DirectStringDict) -> Result<Rc<str>, Error> {
        let value = self.read_varint()?;
        dict.decode(value, self)
    }

    fn read_plain_bytes(&mut self) -> Result<Vec<u8>, Error> {
        let len = usize::try_from(self.read_varint()?)
            .map_err(|_| Error::ValueOutOfRange("bytes length"))?;
        Ok(self.read_bytes(len)?.to_vec())
    }

    fn read_shared_utf8_string(&mut self, len: usize) -> Result<Rc<str>, Error> {
        let bytes = self.read_bytes(len)?;
        std::str::from_utf8(bytes)
            .map(Rc::<str>::from)
            .map_err(|_| Error::InvalidFrame("invalid utf8 string"))
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let end = self
            .position
            .checked_add(len)
            .ok_or(Error::ValueOutOfRange("reader position"))?;
        if end > self.bytes.len() {
            return Err(Error::UnexpectedEof);
        }
        let bytes = &self.bytes[self.position..end];
        self.position = end;
        Ok(bytes)
    }
}

#[derive(Default)]
struct U64Decoder<'a> {
    bytes: BytesReader<'a>,
    last_value: u64,
    last_delta: u64,
}

impl<'a> U64Decoder<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: BytesReader::new(bytes),
            last_value: 0,
            last_delta: 0,
        }
    }

    fn decode(&mut self) -> Result<u64, Error> {
        let delta_of_delta = self.bytes.read_varint()?;
        let delta = self.last_delta.wrapping_add(delta_of_delta as u64);
        self.last_delta = delta;
        self.last_value = self.last_value.wrapping_add(delta);
        Ok(self.last_value)
    }
}

#[derive(Default)]
struct I64Decoder<'a> {
    inner: U64Decoder<'a>,
}

impl<'a> I64Decoder<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            inner: U64Decoder::new(bytes),
        }
    }

    fn decode(&mut self) -> Result<i64, Error> {
        self.inner.decode().map(|value| value as i64)
    }
}

#[derive(Default)]
struct BoolDecoder<'a> {
    bits: BitReader<'a>,
}

impl<'a> BoolDecoder<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bits: BitReader::new(bytes),
        }
    }

    fn decode(&mut self) -> Result<bool, Error> {
        self.bits.read_bit()
    }
}

#[derive(Default)]
struct Float64Decoder<'a> {
    bits: BitReader<'a>,
    last_value: f64,
    leading_bits: u64,
    trailing_bits: u64,
}

impl<'a> Float64Decoder<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bits: BitReader::new(bytes),
            last_value: 0.0,
            leading_bits: 0,
            trailing_bits: 0,
        }
    }

    fn decode(&mut self) -> Result<f64, Error> {
        if !self.bits.read_bit()? {
            return Ok(self.last_value);
        }
        let same_window = !self.bits.read_bit()?;
        let (leading, trailing, significant_bits) = if same_window {
            (
                self.leading_bits,
                self.trailing_bits,
                64 - self.leading_bits - self.trailing_bits,
            )
        } else {
            let leading = self.bits.read_bits(5)?;
            let significant_bits = self.bits.read_bits(6)? + 1;
            let trailing = 64 - leading - significant_bits;
            self.leading_bits = leading;
            self.trailing_bits = trailing;
            (leading, trailing, significant_bits)
        };
        let _ = leading;
        let xor_value = self.bits.read_bits(significant_bits as u32)? << trailing;
        self.last_value = f64::from_bits(xor_value ^ self.last_value.to_bits());
        Ok(self.last_value)
    }
}

#[derive(Default)]
struct BitReader<'a> {
    bytes: &'a [u8],
    bit_position: usize,
}

impl<'a> BitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            bit_position: 0,
        }
    }

    fn read_bit(&mut self) -> Result<bool, Error> {
        let byte_index = self.bit_position / 8;
        if byte_index >= self.bytes.len() {
            return Err(Error::UnexpectedEof);
        }
        let bit_index = 7 - (self.bit_position % 8);
        self.bit_position += 1;
        Ok(((self.bytes[byte_index] >> bit_index) & 1) == 1)
    }

    fn read_bits(&mut self, bit_count: u32) -> Result<u64, Error> {
        let mut value = 0;
        for _ in 0..bit_count {
            value = (value << 1) | u64::from(self.read_bit()?);
        }
        Ok(value)
    }

    fn read_uvarint_compact(&mut self) -> Result<u64, Error> {
        let mut zeros = 0;
        while !self.read_bit()? {
            zeros += 1;
            if zeros > 7 {
                return Err(Error::InvalidFrame("invalid compact integer"));
            }
        }
        let bit_count = match zeros {
            0 => return Ok(0),
            1 => 2,
            2 => 5,
            3 => 12,
            4 => 19,
            5 => 26,
            6 => 33,
            7 => 48,
            _ => return Err(Error::InvalidFrame("invalid compact integer")),
        };
        self.read_bits(bit_count)
    }
}

struct SliceReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> SliceReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn is_empty(&self) -> bool {
        self.position >= self.bytes.len()
    }

    fn remaining_len(&self) -> usize {
        self.bytes.len().saturating_sub(self.position)
    }

    fn read_u8(&mut self) -> Result<u8, Error> {
        Ok(*self.read_bytes(1)?.first().expect("one byte"))
    }

    fn read_uvarint(&mut self) -> Result<u64, Error> {
        read_uvarint_from_slice(self.bytes, &mut self.position)
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let end = self
            .position
            .checked_add(len)
            .ok_or(Error::ValueOutOfRange("reader position"))?;
        if end > self.bytes.len() {
            return Err(Error::UnexpectedEof);
        }
        let bytes = &self.bytes[self.position..end];
        self.position = end;
        Ok(bytes)
    }
}

fn read_uvarint_from_slice(bytes: &[u8], position: &mut usize) -> Result<u64, Error> {
    let mut value = 0_u64;
    let mut shift = 0;
    loop {
        if *position >= bytes.len() {
            return Err(Error::UnexpectedEof);
        }
        let byte = bytes[*position];
        *position += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
        shift += 7;
        if shift >= 64 {
            return Err(Error::ValueOutOfRange("uvarint"));
        }
    }
}

fn read_fixed_header(input: &mut SliceReader<'_>) -> Result<(), Error> {
    if input.read_bytes(4)? != HDR_SIGNATURE {
        return Err(Error::InvalidHeader("bad signature"));
    }
    let header_size = usize::try_from(input.read_uvarint()?)
        .map_err(|_| Error::ValueOutOfRange("fixed header"))?;
    if header_size < 2 {
        return Err(Error::InvalidHeader("too short"));
    }
    let header = input.read_bytes(header_size)?;
    if header[0] & 0x0f != HDR_FORMAT_VERSION {
        return Err(Error::InvalidHeader("unsupported version"));
    }
    if header[1] & 0b11 != COMPRESSION_NONE {
        return Err(Error::InvalidHeader("compressed STEF is not implemented"));
    }
    Ok(())
}

fn read_frame<'a>(input: &mut SliceReader<'a>) -> Result<(u8, &'a [u8]), Error> {
    let flags = input.read_u8()?;
    let len =
        usize::try_from(input.read_uvarint()?).map_err(|_| Error::ValueOutOfRange("frame"))?;
    let content = input.read_bytes(len)?;
    Ok((flags, content))
}

fn read_var_header(bytes: &[u8]) -> Result<(), Error> {
    let mut reader = SliceReader::new(bytes);
    let schema_len =
        usize::try_from(reader.read_uvarint()?).map_err(|_| Error::ValueOutOfRange("schema"))?;
    let _schema = reader.read_bytes(schema_len)?;
    let user_data_count = reader.read_uvarint()?;
    for _ in 0..user_data_count {
        let key_len = usize::try_from(reader.read_uvarint()?)
            .map_err(|_| Error::ValueOutOfRange("user data key"))?;
        let _ = reader.read_bytes(key_len)?;
        let value_len = usize::try_from(reader.read_uvarint()?)
            .map_err(|_| Error::ValueOutOfRange("user data value"))?;
        let _ = reader.read_bytes(value_len)?;
    }
    Ok(())
}

fn read_decode_column_sizes(
    column: &mut DecodeColumn<'_>,
    sizes: &mut BitReader<'_>,
    mut read_limit: u64,
) -> Result<(), Error> {
    read_decode_column_sizes_inner(column, sizes, &mut read_limit)
}

fn read_decode_column_sizes_inner(
    column: &mut DecodeColumn<'_>,
    sizes: &mut BitReader<'_>,
    read_limit: &mut u64,
) -> Result<(), Error> {
    let size = sizes.read_uvarint_compact()?;
    if size > *read_limit {
        return Err(Error::InvalidFrame("column exceeds frame"));
    }
    *read_limit -= size;
    column.byte_len = usize::try_from(size).map_err(|_| Error::ValueOutOfRange("column"))?;
    column.data = &[];
    if size == 0 {
        reset_decode_column_data(column);
        return Ok(());
    }
    for child in &mut column.children {
        read_decode_column_sizes_inner(child, sizes, read_limit)?;
    }
    Ok(())
}

fn reset_decode_column_data(column: &mut DecodeColumn<'_>) {
    column.data = &[];
    column.byte_len = 0;
    for child in &mut column.children {
        reset_decode_column_data(child);
    }
}

fn read_decode_column_data<'a>(
    column: &mut DecodeColumn<'a>,
    reader: &mut SliceReader<'a>,
) -> Result<(), Error> {
    if column.byte_len == 0 {
        return Ok(());
    }
    column.data = reader.read_bytes(column.byte_len)?;
    for child in &mut column.children {
        read_decode_column_data(child, reader)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use crate::proto::opentelemetry::common::v1::{
        AnyValue, KeyValue, any_value as otlp_any_value,
    };
    use crate::proto::opentelemetry::metrics::v1::{
        AggregationTemporality, Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum,
        metric, number_data_point,
    };
    use crate::proto::opentelemetry::resource::v1::Resource;
    use crate::views::otlp::bytes::metrics::RawMetricsData;
    use prost::Message as ProstMessage;

    fn example_gauge_request() -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_owned(),
                        value: Some(AnyValue {
                            value: Some(otlp_any_value::Value::StringValue("demo".to_owned())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: Vec::new(),
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: None,
                    metrics: vec![Metric {
                        name: "requests".to_owned(),
                        description: String::new(),
                        unit: "1".to_owned(),
                        metadata: Vec::new(),
                        data: Some(metric::Data::Gauge(Gauge {
                            data_points: vec![NumberDataPoint {
                                attributes: vec![KeyValue {
                                    key: "route".to_owned(),
                                    value: Some(AnyValue {
                                        value: Some(otlp_any_value::Value::StringValue(
                                            "/v1/metrics".to_owned(),
                                        )),
                                    }),
                                }],
                                start_time_unix_nano: 10,
                                time_unix_nano: 20,
                                exemplars: Vec::new(),
                                flags: 0,
                                value: Some(number_data_point::Value::AsInt(7)),
                            }],
                        })),
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        }
    }

    fn otlp_bytes(request: &ExportMetricsServiceRequest) -> Vec<u8> {
        let mut bytes = Vec::new();
        request.encode(&mut bytes).unwrap();
        bytes
    }

    fn string_kv(key: &str, value: &str) -> KeyValue {
        KeyValue {
            key: key.to_owned(),
            value: Some(AnyValue {
                value: Some(otlp_any_value::Value::StringValue(value.to_owned())),
            }),
        }
    }

    fn encode_request_direct(request: &ExportMetricsServiceRequest) -> (Vec<u8>, u64) {
        let bytes = otlp_bytes(request);
        let view = RawMetricsData::new(&bytes);
        encode_metrics_view_with_count(&view).unwrap()
    }

    fn assert_single_gauge_value(records: &OtapArrowRecords) {
        let view = crate::views::otap::metrics::OtapMetricsView::try_from(records).unwrap();
        let resource_metrics = view.resources().next().unwrap();
        let scope_metrics = resource_metrics.scopes().next().unwrap();
        let metric = scope_metrics.metrics().next().unwrap();
        assert_eq!(std::str::from_utf8(metric.name()).unwrap(), "requests");
        assert_eq!(std::str::from_utf8(metric.unit()).unwrap(), "1");

        let data = metric.data().unwrap();
        let gauge = data.as_gauge().unwrap();
        let point = gauge.data_points().next().unwrap();
        assert_eq!(point.start_time_unix_nano(), 10);
        assert_eq!(point.time_unix_nano(), 20);
        assert_eq!(point.value(), Some(metrics_view::Value::Integer(7)));
    }

    fn string_attrs<P>(point: &P) -> Vec<(String, String)>
    where
        P: NumberDataPointView,
    {
        point
            .attributes()
            .map(|attribute| {
                let key = std::str::from_utf8(attribute.key()).unwrap().to_owned();
                let value = attribute.value().unwrap();
                assert_eq!(value.value_type(), ValueType::String);
                let value = std::str::from_utf8(value.as_string().unwrap())
                    .unwrap()
                    .to_owned();
                (key, value)
            })
            .collect()
    }

    #[test]
    fn encodes_metrics_stream_header_and_schema() {
        let (bytes, record_count) = encode_request_direct(&example_gauge_request());
        assert_eq!(record_count, 1);
        assert!(bytes.starts_with(b"STEF"));
        assert!(
            bytes
                .windows(METRICS_WIRE_SCHEMA.len())
                .any(|window| window == METRICS_WIRE_SCHEMA)
        );
    }

    #[test]
    fn accepts_empty_sum_metric() {
        let mut request = example_gauge_request();
        request.resource_metrics[0].scope_metrics[0].metrics[0].data =
            Some(metric::Data::Sum(Sum {
                data_points: Vec::new(),
                aggregation_temporality: AggregationTemporality::Cumulative as i32,
                is_monotonic: true,
            }));
        let (bytes, record_count) = encode_request_direct(&request);
        assert_eq!(record_count, 0);
        assert!(bytes.starts_with(b"STEF"));
    }

    #[test]
    fn roundtrips_numeric_gauge_through_direct_otap() {
        let (bytes, encoded_count) = encode_request_direct(&example_gauge_request());
        assert_eq!(encoded_count, 1);

        let (records, decoded_count) = decode_metrics_otap_with_count(&bytes).unwrap();
        assert_eq!(decoded_count, 1);
        assert_eq!(records.num_items(), 1);
        assert_single_gauge_value(&records);
    }

    #[test]
    fn decodes_metrics_stream_directly_to_otap_records() {
        let (bytes, _) = encode_request_direct(&example_gauge_request());
        let (records, record_count) = decode_metrics_otap_with_count(&bytes).unwrap();
        assert_eq!(record_count, 1);
        assert_eq!(records.num_items(), 1);

        let (encoded, encoded_count) = encode_metrics_otap_with_count(&records).unwrap();
        assert_eq!(encoded_count, 1);
        let (decoded_records, decoded_count) = decode_metrics_otap_with_count(&encoded).unwrap();
        assert_eq!(decoded_count, 1);
        assert_single_gauge_value(&decoded_records);
    }

    #[test]
    fn encodes_raw_otlp_metrics_view_directly_to_stef() {
        let otlp_bytes = otlp_bytes(&example_gauge_request());
        let view = RawMetricsData::new(&otlp_bytes);
        let (encoded, record_count) = encode_metrics_view_with_count(&view).unwrap();
        assert_eq!(record_count, 1);

        let (records, decoded_count) = decode_metrics_otap_with_count(&encoded).unwrap();
        assert_eq!(decoded_count, 1);
        assert_single_gauge_value(&records);
    }

    #[test]
    fn encodes_attribute_layout_change_after_repeated_prefix() {
        let mut request = example_gauge_request();
        let metric = &mut request.resource_metrics[0].scope_metrics[0].metrics[0];
        let metric::Data::Gauge(gauge) = metric.data.as_mut().unwrap() else {
            panic!("example metric must be a gauge");
        };
        gauge.data_points = vec![
            NumberDataPoint {
                attributes: vec![string_kv("stable", "one"), string_kv("first", "old")],
                start_time_unix_nano: 10,
                time_unix_nano: 20,
                exemplars: Vec::new(),
                flags: 0,
                value: Some(number_data_point::Value::AsInt(7)),
            },
            NumberDataPoint {
                attributes: vec![string_kv("stable", "one"), string_kv("second", "new")],
                start_time_unix_nano: 10,
                time_unix_nano: 21,
                exemplars: Vec::new(),
                flags: 0,
                value: Some(number_data_point::Value::AsInt(8)),
            },
        ];

        let (encoded, encoded_count) = encode_request_direct(&request);
        assert_eq!(encoded_count, 2);

        let (records, decoded_count) = decode_metrics_otap_with_count(&encoded).unwrap();
        assert_eq!(decoded_count, 2);
        let view = crate::views::otap::metrics::OtapMetricsView::try_from(&records).unwrap();
        let resource_metrics = view.resources().next().unwrap();
        let scope_metrics = resource_metrics.scopes().next().unwrap();
        let metric = scope_metrics.metrics().next().unwrap();
        let data = metric.data().unwrap();
        let gauge = data.as_gauge().unwrap();
        let point = gauge.data_points().nth(1).unwrap();

        assert_eq!(point.value(), Some(metrics_view::Value::Integer(8)));
        assert_eq!(
            string_attrs(&point),
            vec![
                ("stable".to_owned(), "one".to_owned()),
                ("second".to_owned(), "new".to_owned()),
            ]
        );
    }
}
