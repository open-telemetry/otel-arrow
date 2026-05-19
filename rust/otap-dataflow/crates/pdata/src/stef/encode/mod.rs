// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! STEF metrics stream encoding from OTLP metric views and direct OTAP Arrow records.
//!
//! The encoder has two front doors: the OTLP view path, which avoids generated-message
//! materialization, and the direct OTAP path, which walks Arrow record batches directly.

mod attributes;
mod entities;

use self::attributes::AttributesEncoder;
use self::entities::{
    MetricEncoder, PointEncoder, ResourceEncoder, ScopeEncoder, StefPoint, StefPointValue,
    ViewMetric,
};
use crate::OtapArrowRecords;
use crate::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor};
use crate::otlp::attributes::{Attribute16Arrays, Attribute32Arrays};
use crate::otlp::common::{
    BatchSorter, ChildIndexIter, ResourceArrays, ScopeArrays, SortedBatchCursor,
};
use crate::otlp::metrics::{MetricType, MetricsArrays, data_points::number::NumberDpArrays};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use arrow::array::RecordBatch;
use otap_df_pdata_views::views::{
    common::{InstrumentationScopeView, Str},
    metrics::{
        self as metrics_view, DataView, GaugeView, MetricView, MetricsView, NumberDataPointView,
        ResourceMetricsView, ScopeMetricsView, SumView,
    },
    resource::ResourceView,
};

use super::wire::{
    BitWriter, Column, FRAME_FLAG_NONE, FrameEncoder, ROOT_ATTRIBUTES_FIELD, ROOT_METRIC_FIELD,
    ROOT_POINT_FIELD, ROOT_RESOURCE_FIELD, ROOT_SCOPE_FIELD, SharedStringDict, append_uvarint,
    metrics_column_tree, write_columns, write_fixed_header, write_var_header_frame,
};
use super::{Error, METRICS_WIRE_SCHEMA, StefCompression};

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
    encode_metrics_view_with_count_and_compression(metrics, StefCompression::None)
}

/// Encodes a metrics view directly into STEF with native STEF frame compression.
pub fn encode_metrics_view_with_count_and_compression<T>(
    metrics: &T,
    compression: StefCompression,
) -> Result<(Vec<u8>, u64), Error>
where
    T: MetricsView,
{
    let mut encoder = MetricsStreamEncoder::new(compression)?;
    encoder.encode_metrics_view(metrics)?;
    let record_count = encoder.record_count();
    let bytes = encoder.finish()?;
    Ok((bytes, record_count))
}

/// Encodes OTAP metrics Arrow records directly into a complete STEF metrics byte stream.
pub fn encode_metrics_otap(records: &OtapArrowRecords) -> Result<Vec<u8>, Error> {
    encode_metrics_otap_with_count(records).map(|(bytes, _)| bytes)
}

/// Encodes OTAP metrics Arrow records directly into STEF and returns the encoded record count.
pub fn encode_metrics_otap_with_count(records: &OtapArrowRecords) -> Result<(Vec<u8>, u64), Error> {
    encode_metrics_otap_with_count_and_compression(records, StefCompression::None)
}

/// Encodes OTAP metrics Arrow records directly into STEF with native STEF frame compression.
pub fn encode_metrics_otap_with_count_and_compression(
    records: &OtapArrowRecords,
    compression: StefCompression,
) -> Result<(Vec<u8>, u64), Error> {
    let mut encoder = MetricsStreamEncoder::new(compression)?;
    encoder.encode_metrics_otap(records)?;
    let record_count = encoder.record_count();
    let bytes = encoder.finish()?;
    Ok((bytes, record_count))
}

/// A data frame emitted by a [`MetricsStreamEncoder`].
pub struct EncodedFrame {
    /// Complete STEF frame bytes ready to send as one STEF/gRPC chunk.
    pub bytes: Vec<u8>,
    /// Records encoded in this frame.
    pub frame_record_count: u64,
    /// Cumulative records encoded in the stream after this frame.
    pub stream_record_count: u64,
}

/// Stateful STEF metrics stream encoder.
///
/// A single instance corresponds to one STEF byte stream. It writes the fixed and
/// variable headers once, then each call to a frame encoder method emits one data
/// frame while preserving STEF dictionaries, previous-value codecs, and native
/// compression state across frames.
pub struct MetricsStreamEncoder {
    compression: StefCompression,
    header_written: bool,
    frame_encoder: FrameEncoder,
    record_count: u64,
    frame_record_count: u64,
    root_bits: BitWriter,
    metric: MetricEncoder,
    resource: ResourceEncoder,
    scope: ScopeEncoder,
    attributes: AttributesEncoder,
    point: PointEncoder,
}

impl MetricsStreamEncoder {
    /// Creates a new STEF metrics stream encoder.
    pub fn new(compression: StefCompression) -> Result<Self, Error> {
        let schema_url = SharedStringDict::default();
        let attribute_key = SharedStringDict::default();
        let any_value_string = SharedStringDict::default();

        Ok(Self {
            compression,
            header_written: false,
            frame_encoder: FrameEncoder::new(compression)?,
            record_count: 0,
            frame_record_count: 0,
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
        })
    }

    /// Returns the cumulative number of metric records written to this stream.
    #[must_use]
    pub fn record_count(&self) -> u64 {
        self.record_count
    }

    /// Writes the STEF fixed-header chunk and variable-header frame chunk.
    ///
    /// The returned chunks should be sent before data frames. The first chunk is
    /// the fixed header and the second chunk is the variable-header frame, matching
    /// the Go STEF/gRPC writer's chunking behavior.
    pub fn stream_header_chunks(&mut self) -> Result<Option<(Vec<u8>, Vec<u8>)>, Error> {
        if self.header_written {
            return Ok(None);
        }

        let mut fixed_header = Vec::with_capacity(8);
        write_fixed_header(&mut fixed_header, self.compression);

        let mut var_header = Vec::with_capacity(METRICS_WIRE_SCHEMA.len() + 16);
        write_var_header_frame(&mut self.frame_encoder, &mut var_header)?;

        self.header_written = true;
        Ok(Some((fixed_header, var_header)))
    }

    /// Encodes a metrics view into the next STEF data frame.
    pub fn encode_metrics_view_frame<T>(&mut self, metrics: &T) -> Result<EncodedFrame, Error>
    where
        T: MetricsView,
    {
        self.encode_metrics_view(metrics)?;
        self.flush_frame()
    }

    /// Encodes direct OTAP metrics records into the next STEF data frame.
    pub fn encode_metrics_otap_frame(
        &mut self,
        records: &OtapArrowRecords,
    ) -> Result<EncodedFrame, Error> {
        self.encode_metrics_otap(records)?;
        self.flush_frame()
    }

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
            self.frame_record_count += 1;
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
            self.frame_record_count += 1;
            wrote = true;
            encode_metric = false;
            encode_resource = false;
            encode_scope = false;
        }

        Ok(wrote)
    }

    fn finish(mut self) -> Result<Vec<u8>, Error> {
        let mut stream = Vec::with_capacity(16 * 1024);
        if let Some((fixed_header, var_header)) = self.stream_header_chunks()? {
            stream.extend_from_slice(&fixed_header);
            stream.extend_from_slice(&var_header);
        }

        let frame = self.flush_frame()?;
        stream.extend_from_slice(&frame.bytes);
        Ok(stream)
    }

    fn flush_frame(&mut self) -> Result<EncodedFrame, Error> {
        let frame_record_count = self.frame_record_count;
        if frame_record_count == 0 {
            return Ok(EncodedFrame {
                bytes: Vec::new(),
                frame_record_count: 0,
                stream_record_count: self.record_count,
            });
        }

        let mut frame = Vec::with_capacity(16 * 1024);
        let mut content = Vec::with_capacity(16 * 1024);
        append_uvarint(frame_record_count, &mut content);

        let columns = self.collect_columns();
        write_columns(&columns, &mut content)?;
        self.frame_encoder
            .write_frame(&mut frame, FRAME_FLAG_NONE, &content)?;
        self.frame_record_count = 0;

        Ok(EncodedFrame {
            bytes: frame,
            frame_record_count,
            stream_record_count: self.record_count,
        })
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
