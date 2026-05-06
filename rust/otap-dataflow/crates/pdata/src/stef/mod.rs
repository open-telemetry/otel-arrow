// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Minimal STEF metrics codec compatible with the Splunk/Collector STEF metrics schema.
//!
//! The implementation intentionally starts with the metrics profile used by the Go Collector
//! `stefreceiver` and `stefexporter`. It writes standard STEF fixed/variable headers, column
//! frames, and the generated `otel.stef` metrics wire schema. The encoder currently supports
//! gauge and sum number data points plus simple OTLP attribute values.

use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use crate::proto::opentelemetry::common::v1::{
    AnyValue, InstrumentationScope, KeyValue, any_value as otlp_any_value,
};
use crate::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum,
    metric, number_data_point,
};
use crate::proto::opentelemetry::resource::v1::Resource;
use std::collections::HashMap;
use std::fmt;

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

const ROOT_FIELD_MASK: u64 = 0b111110;
const METRIC_FIELD_MASK: u64 = 0b11111111;
const RESOURCE_FIELD_MASK: u64 = 0b111;
const SCOPE_FIELD_MASK: u64 = 0b11111;
const POINT_FIELD_MASK: u64 = 0b1111;

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
        }
    }
}

impl std::error::Error for Error {}

/// Encodes an OTLP metrics request into a complete STEF metrics byte stream.
pub fn encode_metrics_request(request: &ExportMetricsServiceRequest) -> Result<Vec<u8>, Error> {
    let mut encoder = MetricsStreamEncoder::default();
    encoder.encode_request(request)?;
    encoder.finish()
}

/// Decodes a complete STEF metrics byte stream into an OTLP metrics request.
pub fn decode_metrics_request(bytes: &[u8]) -> Result<ExportMetricsServiceRequest, Error> {
    MetricsStreamDecoder::decode(bytes)
}

#[derive(Default)]
struct MetricsStreamEncoder {
    record_count: u64,
    root_bits: BitWriter,
    metric: MetricEncoder,
    resource: ResourceEncoder,
    scope: ScopeEncoder,
    attributes: AttributesEncoder,
    point: PointEncoder,
}

impl MetricsStreamEncoder {
    fn encode_request(&mut self, request: &ExportMetricsServiceRequest) -> Result<(), Error> {
        for resource_metrics in &request.resource_metrics {
            let resource = StefResource::from_resource_metrics(resource_metrics);
            for scope_metrics in &resource_metrics.scope_metrics {
                let scope = StefScope::from_scope_metrics(scope_metrics);
                for metric in &scope_metrics.metrics {
                    let base_metric = StefMetric::from_metric(metric)?;
                    match metric.data.as_ref() {
                        Some(metric::Data::Gauge(gauge)) => {
                            self.encode_number_points(
                                &resource,
                                &scope,
                                &base_metric,
                                &gauge.data_points,
                            )?;
                        }
                        Some(metric::Data::Sum(sum)) => {
                            let mut sum_metric = base_metric;
                            sum_metric.aggregation_temporality = sum.aggregation_temporality as u64;
                            sum_metric.monotonic = sum.is_monotonic;
                            self.encode_number_points(
                                &resource,
                                &scope,
                                &sum_metric,
                                &sum.data_points,
                            )?;
                        }
                        Some(metric::Data::Histogram(_)) => {
                            return Err(Error::UnsupportedMetricType("histogram"));
                        }
                        Some(metric::Data::ExponentialHistogram(_)) => {
                            return Err(Error::UnsupportedMetricType("exponential_histogram"));
                        }
                        Some(metric::Data::Summary(_)) => {
                            return Err(Error::UnsupportedMetricType("summary"));
                        }
                        None => return Err(Error::UnsupportedMetricType("empty")),
                    }
                }
            }
        }
        Ok(())
    }

    fn encode_number_points(
        &mut self,
        resource: &StefResource<'_>,
        scope: &StefScope<'_>,
        metric: &StefMetric<'_>,
        points: &[NumberDataPoint],
    ) -> Result<(), Error> {
        for point in points {
            let point_value = match point.value {
                Some(number_data_point::Value::AsInt(value)) => StefPointValue::Int64(value),
                Some(number_data_point::Value::AsDouble(value)) => StefPointValue::Float64(value),
                None if point.flags & 0x01 == 0x01 => StefPointValue::None,
                None => return Err(Error::UnsupportedDataPointValue),
            };
            self.root_bits.write_bits(ROOT_FIELD_MASK, 6);
            self.metric.encode(metric)?;
            self.resource.encode(resource)?;
            self.scope.encode(scope)?;
            self.attributes.encode(&point.attributes)?;
            self.point.encode(&StefPoint {
                start_timestamp: point.start_time_unix_nano,
                timestamp: point.time_unix_nano,
                value: point_value,
            })?;
            self.record_count += 1;
        }
        Ok(())
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

#[derive(Clone, Copy)]
struct StefResource<'a> {
    schema_url: &'a str,
    attributes: &'a [KeyValue],
    dropped_attributes_count: u64,
}

impl<'a> StefResource<'a> {
    fn from_resource_metrics(resource_metrics: &'a ResourceMetrics) -> Self {
        let resource = resource_metrics.resource.as_ref();
        Self {
            schema_url: resource_metrics.schema_url.as_str(),
            attributes: resource.map_or(&[], |resource| resource.attributes.as_slice()),
            dropped_attributes_count: resource
                .map(|resource| u64::from(resource.dropped_attributes_count))
                .unwrap_or(0),
        }
    }
}

#[derive(Clone, Copy)]
struct StefScope<'a> {
    name: &'a str,
    version: &'a str,
    schema_url: &'a str,
    attributes: &'a [KeyValue],
    dropped_attributes_count: u64,
}

impl<'a> StefScope<'a> {
    fn from_scope_metrics(scope_metrics: &'a ScopeMetrics) -> Self {
        let scope = scope_metrics.scope.as_ref();
        Self {
            name: scope.map_or("", |scope| scope.name.as_str()),
            version: scope.map_or("", |scope| scope.version.as_str()),
            schema_url: scope_metrics.schema_url.as_str(),
            attributes: scope.map_or(&[], |scope| scope.attributes.as_slice()),
            dropped_attributes_count: scope
                .map(|scope| u64::from(scope.dropped_attributes_count))
                .unwrap_or(0),
        }
    }
}

#[derive(Clone)]
struct StefMetric<'a> {
    name: &'a str,
    description: &'a str,
    unit: &'a str,
    r#type: u64,
    metadata: &'a [KeyValue],
    aggregation_temporality: u64,
    monotonic: bool,
}

impl<'a> StefMetric<'a> {
    fn from_metric(metric: &'a Metric) -> Result<Self, Error> {
        let r#type = match metric.data.as_ref() {
            Some(metric::Data::Gauge(_)) => 0,
            Some(metric::Data::Sum(_)) => 1,
            Some(metric::Data::Histogram(_)) => 2,
            Some(metric::Data::ExponentialHistogram(_)) => 3,
            Some(metric::Data::Summary(_)) => 4,
            None => return Err(Error::UnsupportedMetricType("empty")),
        };
        Ok(Self {
            name: metric.name.as_str(),
            description: metric.description.as_str(),
            unit: metric.unit.as_str(),
            r#type,
            metadata: &metric.metadata,
            aggregation_temporality: 0,
            monotonic: false,
        })
    }
}

struct StefPoint {
    start_timestamp: u64,
    timestamp: u64,
    value: StefPointValue,
}

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
    fn encode(&mut self, metric: &StefMetric<'_>) -> Result<(), Error> {
        self.bits.write_bit(true);
        self.bits.write_bits(METRIC_FIELD_MASK, 8);
        self.name.encode(metric.name);
        self.description.encode(metric.description);
        self.unit.encode(metric.unit);
        self.r#type.encode(metric.r#type);
        self.metadata.encode(metric.metadata)?;
        self.histogram_bounds.encode_empty();
        self.aggregation_temporality
            .encode(metric.aggregation_temporality);
        self.monotonic.encode(metric.monotonic);
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
    fn encode(&mut self, resource: &StefResource<'_>) -> Result<(), Error> {
        self.bits.write_bit(true);
        self.bits.write_bits(RESOURCE_FIELD_MASK, 3);
        self.schema_url.encode(resource.schema_url);
        self.attributes.encode(resource.attributes)?;
        self.dropped_attributes_count
            .encode(resource.dropped_attributes_count);
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
    fn encode(&mut self, scope: &StefScope<'_>) -> Result<(), Error> {
        self.bits.write_bit(true);
        self.bits.write_bits(SCOPE_FIELD_MASK, 5);
        self.name.encode(scope.name);
        self.version.encode(scope.version);
        self.schema_url.encode(scope.schema_url);
        self.attributes.encode(scope.attributes)?;
        self.dropped_attributes_count
            .encode(scope.dropped_attributes_count);
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
}

impl PointEncoder {
    fn encode(&mut self, point: &StefPoint) -> Result<(), Error> {
        self.bits.write_bits(POINT_FIELD_MASK, 4);
        self.start_timestamp.encode(point.start_timestamp);
        self.timestamp.encode(point.timestamp);
        self.value.encode(&point.value);
        self.exemplars.encode_empty();
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
            StefPointValue::None => self.bits.write_bits(0, 3),
            StefPointValue::Int64(value) => {
                self.bits.write_bits(1, 3);
                self.int64.encode(*value);
            }
            StefPointValue::Float64(value) => {
                self.bits.write_bits(2, 3);
                self.float64.encode(*value);
            }
        }
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
}

impl AttributesEncoder {
    fn encode(&mut self, attributes: &[KeyValue]) -> Result<(), Error> {
        self.header
            .write_uvarint((attributes.len() as u64) << 1 | 0b1);
        for attribute in attributes {
            self.key.encode(&attribute.key);
            self.value.encode(attribute.value.as_ref())?;
        }
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
    fn encode(&mut self, value: Option<&AnyValue>) -> Result<(), Error> {
        match value.and_then(|value| value.value.as_ref()) {
            None => self.bits.write_bits(0, 4),
            Some(otlp_any_value::Value::StringValue(value)) => {
                self.bits.write_bits(1, 4);
                self.string.encode(value);
            }
            Some(otlp_any_value::Value::BoolValue(value)) => {
                self.bits.write_bits(2, 4);
                self.bool_.encode(*value);
            }
            Some(otlp_any_value::Value::IntValue(value)) => {
                self.bits.write_bits(3, 4);
                self.int64.encode(*value);
            }
            Some(otlp_any_value::Value::DoubleValue(value)) => {
                self.bits.write_bits(4, 4);
                self.float64.encode(*value);
            }
            Some(otlp_any_value::Value::ArrayValue(_)) => {
                return Err(Error::UnsupportedAttributeValue("array"));
            }
            Some(otlp_any_value::Value::KvlistValue(_)) => {
                return Err(Error::UnsupportedAttributeValue("kvlist"));
            }
            Some(otlp_any_value::Value::BytesValue(value)) => {
                self.bits.write_bits(7, 4);
                self.bytes.encode(value);
            }
        }
        Ok(())
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
struct StringEncoder {
    bytes: BytesWriter,
    dict: HashMap<String, usize>,
}

impl StringEncoder {
    fn encode(&mut self, value: &str) {
        if let Some(ref_num) = self.dict.get(value) {
            self.bytes.write_varint(-(*ref_num as i64) - 1);
            return;
        }
        if value.len() > 1 {
            let _ = self.dict.insert(value.to_owned(), self.dict.len());
        }
        self.bytes.write_varint(value.len() as i64);
        self.bytes.write_bytes(value.as_bytes());
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
        for shift in (0..bit_count).rev() {
            self.write_bit(((value >> shift) & 1) == 1);
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
struct MetricsStreamDecoder {
    state: DecoderState,
    metrics: ExportMetricsServiceRequest,
    current_resource: DecResource,
    current_scope: DecScope,
    current_metric: DecMetric,
    current_attrs: Vec<KeyValue>,
    current_point: DecPoint,
    resource_index: Option<usize>,
    scope_index: Option<usize>,
    metric_index: Option<usize>,
}

impl MetricsStreamDecoder {
    fn decode(bytes: &[u8]) -> Result<ExportMetricsServiceRequest, Error> {
        let mut input = SliceReader::new(bytes);
        read_fixed_header(&mut input)?;
        let (_, var_header) = read_frame(&mut input)?;
        read_var_header(var_header)?;

        let mut decoder = Self::default();
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
            decoder.decode_data_frame(frame)?;
        }
        Ok(decoder.metrics)
    }

    fn decode_data_frame(&mut self, frame: &[u8]) -> Result<(), Error> {
        let mut reader = SliceReader::new(frame);
        let record_count = reader.read_uvarint()?;
        let size_len = usize::try_from(reader.read_uvarint()?)
            .map_err(|_| Error::ValueOutOfRange("column size buffer"))?;
        let size_bytes = reader.read_bytes(size_len)?;
        let mut columns = metrics_column_tree();
        let mut size_reader = BitReader::new(size_bytes);
        read_column_sizes(
            &mut columns,
            &mut size_reader,
            reader.remaining_len() as u64,
        )?;
        read_column_data(&mut columns, &mut reader)?;

        let mut frame_decoder = MetricsFrameDecoder::new(&columns);
        for _ in 0..record_count {
            let modified = frame_decoder.decode_record(
                &mut self.current_resource,
                &mut self.current_scope,
                &mut self.current_metric,
                &mut self.current_attrs,
                &mut self.current_point,
                &mut self.state,
            )?;
            self.append_record(modified)?;
        }
        Ok(())
    }

    fn append_record(&mut self, modified: RootModified) -> Result<(), Error> {
        if self.resource_index.is_none() || modified.resource {
            self.metrics.resource_metrics.push(ResourceMetrics {
                resource: Some(Resource {
                    attributes: self.current_resource.attributes.clone(),
                    dropped_attributes_count: self.current_resource.dropped_attributes_count,
                    entity_refs: Vec::new(),
                }),
                scope_metrics: Vec::new(),
                schema_url: self.current_resource.schema_url.clone(),
            });
            self.resource_index = Some(self.metrics.resource_metrics.len() - 1);
            self.scope_index = None;
            self.metric_index = None;
        }

        if self.scope_index.is_none() || modified.scope {
            let resource_index = self
                .resource_index
                .expect("resource index is initialized above");
            self.metrics.resource_metrics[resource_index]
                .scope_metrics
                .push(ScopeMetrics {
                    scope: Some(InstrumentationScope {
                        name: self.current_scope.name.clone(),
                        version: self.current_scope.version.clone(),
                        attributes: self.current_scope.attributes.clone(),
                        dropped_attributes_count: self.current_scope.dropped_attributes_count,
                    }),
                    metrics: Vec::new(),
                    schema_url: self.current_scope.schema_url.clone(),
                });
            self.scope_index = Some(
                self.metrics.resource_metrics[resource_index]
                    .scope_metrics
                    .len()
                    - 1,
            );
            self.metric_index = None;
        }

        if self.metric_index.is_none() || modified.metric {
            let resource_index = self
                .resource_index
                .expect("resource index is initialized above");
            let scope_index = self.scope_index.expect("scope index is initialized above");
            self.metrics.resource_metrics[resource_index].scope_metrics[scope_index]
                .metrics
                .push(self.current_metric.to_otlp()?);
            self.metric_index = Some(
                self.metrics.resource_metrics[resource_index].scope_metrics[scope_index]
                    .metrics
                    .len()
                    - 1,
            );
        }

        let resource_index = self
            .resource_index
            .expect("resource index is initialized above");
        let scope_index = self.scope_index.expect("scope index is initialized above");
        let metric_index = self
            .metric_index
            .expect("metric index is initialized above");
        let metric = &mut self.metrics.resource_metrics[resource_index].scope_metrics[scope_index]
            .metrics[metric_index];
        self.current_point
            .append_to(metric, self.current_attrs.clone())?;
        Ok(())
    }
}

#[derive(Default)]
struct MetricsFrameDecoder<'a> {
    root: BitReader<'a>,
    metric: MetricDecoder<'a>,
    resource: ResourceDecoder<'a>,
    scope: ScopeDecoder<'a>,
    attributes: AttributesDecoder<'a>,
    point: PointDecoder<'a>,
}

impl<'a> MetricsFrameDecoder<'a> {
    fn new(columns: &'a Column) -> Self {
        Self {
            root: BitReader::new(&columns.data),
            metric: MetricDecoder::new(&columns.children[1]),
            resource: ResourceDecoder::new(&columns.children[2]),
            scope: ScopeDecoder::new(&columns.children[3]),
            attributes: AttributesDecoder::new(&columns.children[4]),
            point: PointDecoder::new(&columns.children[5]),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_record(
        &mut self,
        resource: &mut DecResource,
        scope: &mut DecScope,
        metric: &mut DecMetric,
        attrs: &mut Vec<KeyValue>,
        point: &mut DecPoint,
        state: &mut DecoderState,
    ) -> Result<RootModified, Error> {
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
        if mask & (1 << 4) != 0 {
            self.attributes.decode(attrs, state)?;
        }
        if mask & (1 << 5) != 0 {
            self.point.decode(point)?;
        }
        Ok(modified)
    }
}

#[derive(Clone, Copy, Default)]
struct RootModified {
    metric: bool,
    resource: bool,
    scope: bool,
}

struct DecoderState {
    schema_url: StringDict,
    metric_name: StringDict,
    metric_description: StringDict,
    metric_unit: StringDict,
    scope_name: StringDict,
    scope_version: StringDict,
    attribute_key: StringDict,
    any_value_string: StringDict,
    resources: Vec<DecResource>,
    scopes: Vec<DecScope>,
    metrics: Vec<DecMetric>,
}

impl Default for DecoderState {
    fn default() -> Self {
        let mut state = Self {
            schema_url: StringDict::default(),
            metric_name: StringDict::default(),
            metric_description: StringDict::default(),
            metric_unit: StringDict::default(),
            scope_name: StringDict::default(),
            scope_version: StringDict::default(),
            attribute_key: StringDict::default(),
            any_value_string: StringDict::default(),
            resources: Vec::new(),
            scopes: Vec::new(),
            metrics: Vec::new(),
        };
        state.reset_dictionaries();
        state
    }
}

impl DecoderState {
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
        self.resources.push(DecResource::default());
        self.scopes.clear();
        self.scopes.push(DecScope::default());
        self.metrics.clear();
        self.metrics.push(DecMetric::default());
    }

    fn reset_codecs(&mut self) {}
}

#[derive(Clone, Default)]
struct DecResource {
    schema_url: String,
    attributes: Vec<KeyValue>,
    dropped_attributes_count: u32,
}

#[derive(Clone, Default)]
struct DecScope {
    name: String,
    version: String,
    schema_url: String,
    attributes: Vec<KeyValue>,
    dropped_attributes_count: u32,
}

#[derive(Clone, Default)]
struct DecMetric {
    name: String,
    description: String,
    unit: String,
    r#type: u64,
    metadata: Vec<KeyValue>,
    aggregation_temporality: u64,
    monotonic: bool,
}

impl DecMetric {
    fn to_otlp(&self) -> Result<Metric, Error> {
        let data = match self.r#type {
            0 => metric::Data::Gauge(Gauge {
                data_points: Vec::new(),
            }),
            1 => metric::Data::Sum(Sum {
                data_points: Vec::new(),
                aggregation_temporality: i32::try_from(self.aggregation_temporality)
                    .unwrap_or(AggregationTemporality::Unspecified as i32),
                is_monotonic: self.monotonic,
            }),
            _ => return Err(Error::UnsupportedStefValue("metric type")),
        };
        Ok(Metric {
            name: self.name.clone(),
            description: self.description.clone(),
            unit: self.unit.clone(),
            metadata: self.metadata.clone(),
            data: Some(data),
        })
    }
}

#[derive(Clone, Default)]
struct DecPoint {
    start_timestamp: u64,
    timestamp: u64,
    value: DecPointValue,
}

impl DecPoint {
    fn append_to(&self, metric: &mut Metric, attributes: Vec<KeyValue>) -> Result<(), Error> {
        let point = NumberDataPoint {
            attributes,
            start_time_unix_nano: self.start_timestamp,
            time_unix_nano: self.timestamp,
            exemplars: Vec::new(),
            flags: if matches!(self.value, DecPointValue::None) {
                1
            } else {
                0
            },
            value: match self.value {
                DecPointValue::None => None,
                DecPointValue::Int64(value) => Some(number_data_point::Value::AsInt(value)),
                DecPointValue::Float64(value) => Some(number_data_point::Value::AsDouble(value)),
            },
        };
        match metric.data.as_mut() {
            Some(metric::Data::Gauge(Gauge { data_points })) => data_points.push(point),
            Some(metric::Data::Sum(Sum { data_points, .. })) => data_points.push(point),
            _ => return Err(Error::UnsupportedStefValue("metric point target")),
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Default)]
enum DecPointValue {
    #[default]
    None,
    Int64(i64),
    Float64(f64),
}

#[derive(Default)]
struct MetricDecoder<'a> {
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

impl<'a> MetricDecoder<'a> {
    fn new(column: &'a Column) -> Self {
        Self {
            bits: BitReader::new(&column.data),
            name: BytesReader::new(&column.children[0].data),
            description: BytesReader::new(&column.children[1].data),
            unit: BytesReader::new(&column.children[2].data),
            r#type: U64Decoder::new(&column.children[3].data),
            metadata: AttributesDecoder::new(&column.children[4]),
            histogram_bounds: ArrayDecoder::new(&column.children[5]),
            aggregation_temporality: U64Decoder::new(&column.children[6].data),
            monotonic: BoolDecoder::new(&column.children[7].data),
        }
    }

    fn decode(&mut self, target: &mut DecMetric, state: &mut DecoderState) -> Result<(), Error> {
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
            value.name = self.name.read_dict_string(&mut state.metric_name)?;
        }
        if mask & (1 << 1) != 0 {
            value.description = self
                .description
                .read_dict_string(&mut state.metric_description)?;
        }
        if mask & (1 << 2) != 0 {
            value.unit = self.unit.read_dict_string(&mut state.metric_unit)?;
        }
        if mask & (1 << 3) != 0 {
            value.r#type = self.r#type.decode()?;
        }
        if mask & (1 << 4) != 0 {
            self.metadata.decode(&mut value.metadata, state)?;
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
struct ResourceDecoder<'a> {
    bits: BitReader<'a>,
    schema_url: BytesReader<'a>,
    attributes: AttributesDecoder<'a>,
    dropped_attributes_count: U64Decoder<'a>,
}

impl<'a> ResourceDecoder<'a> {
    fn new(column: &'a Column) -> Self {
        Self {
            bits: BitReader::new(&column.data),
            schema_url: BytesReader::new(&column.children[0].data),
            attributes: AttributesDecoder::new(&column.children[1]),
            dropped_attributes_count: U64Decoder::new(&column.children[2].data),
        }
    }

    fn decode(&mut self, target: &mut DecResource, state: &mut DecoderState) -> Result<(), Error> {
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
            value.schema_url = self.schema_url.read_dict_string(&mut state.schema_url)?;
        }
        if mask & (1 << 1) != 0 {
            self.attributes.decode(&mut value.attributes, state)?;
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
struct ScopeDecoder<'a> {
    bits: BitReader<'a>,
    name: BytesReader<'a>,
    version: BytesReader<'a>,
    schema_url: BytesReader<'a>,
    attributes: AttributesDecoder<'a>,
    dropped_attributes_count: U64Decoder<'a>,
}

impl<'a> ScopeDecoder<'a> {
    fn new(column: &'a Column) -> Self {
        Self {
            bits: BitReader::new(&column.data),
            name: BytesReader::new(&column.children[0].data),
            version: BytesReader::new(&column.children[1].data),
            schema_url: BytesReader::new(&column.children[2].data),
            attributes: AttributesDecoder::new(&column.children[3]),
            dropped_attributes_count: U64Decoder::new(&column.children[4].data),
        }
    }

    fn decode(&mut self, target: &mut DecScope, state: &mut DecoderState) -> Result<(), Error> {
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
            value.name = self.name.read_dict_string(&mut state.scope_name)?;
        }
        if mask & (1 << 1) != 0 {
            value.version = self.version.read_dict_string(&mut state.scope_version)?;
        }
        if mask & (1 << 2) != 0 {
            value.schema_url = self.schema_url.read_dict_string(&mut state.schema_url)?;
        }
        if mask & (1 << 3) != 0 {
            self.attributes.decode(&mut value.attributes, state)?;
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
    fn new(column: &'a Column) -> Self {
        Self {
            bits: BitReader::new(&column.data),
            start_timestamp: U64Decoder::new(&column.children[0].data),
            timestamp: U64Decoder::new(&column.children[1].data),
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
    fn new(column: &'a Column) -> Self {
        Self {
            bits: BitReader::new(&column.data),
            int64: I64Decoder::new(&column.children[0].data),
            float64: Float64Decoder::new(&column.children[1].data),
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
    fn new(column: &'a Column) -> Self {
        Self {
            header: BytesReader::new(&column.data),
            key: BytesReader::new(&column.children[0].data),
            value: AnyValueDecoder::new(&column.children[1]),
        }
    }

    fn decode(
        &mut self,
        target: &mut Vec<KeyValue>,
        state: &mut DecoderState,
    ) -> Result<(), Error> {
        let count_or_changed = self.header.read_uvarint()?;
        if count_or_changed == 0 {
            return Ok(());
        }

        if count_or_changed & 1 == 0 {
            let changed = count_or_changed >> 1;
            for (index, item) in target.iter_mut().enumerate() {
                if changed & (1 << index) != 0 {
                    item.value = Some(self.value.decode(state)?);
                }
            }
            return Ok(());
        }

        let count = usize::try_from(count_or_changed >> 1)
            .map_err(|_| Error::ValueOutOfRange("attributes"))?;
        target.clear();
        target.reserve(count);
        for _ in 0..count {
            let key = self.key.read_dict_string(&mut state.attribute_key)?;
            let value = self.value.decode(state)?;
            target.push(KeyValue {
                key,
                value: Some(value),
            });
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
    fn new(column: &'a Column) -> Self {
        Self {
            bits: BitReader::new(&column.data),
            string: BytesReader::new(&column.children[0].data),
            bool_: BoolDecoder::new(&column.children[1].data),
            int64: I64Decoder::new(&column.children[2].data),
            float64: Float64Decoder::new(&column.children[3].data),
            bytes: BytesReader::new(&column.children[6].data),
        }
    }

    fn decode(&mut self, state: &mut DecoderState) -> Result<AnyValue, Error> {
        let value = match self.bits.read_bits(4)? {
            0 => None,
            1 => Some(otlp_any_value::Value::StringValue(
                self.string.read_dict_string(&mut state.any_value_string)?,
            )),
            2 => Some(otlp_any_value::Value::BoolValue(self.bool_.decode()?)),
            3 => Some(otlp_any_value::Value::IntValue(self.int64.decode()?)),
            4 => Some(otlp_any_value::Value::DoubleValue(self.float64.decode()?)),
            5 => return Err(Error::UnsupportedStefValue("array attribute")),
            6 => return Err(Error::UnsupportedStefValue("kvlist attribute")),
            7 => Some(otlp_any_value::Value::BytesValue(
                self.bytes.read_plain_bytes()?,
            )),
            _ => return Err(Error::UnsupportedStefValue("attribute value")),
        };
        Ok(AnyValue { value })
    }
}

#[derive(Default)]
struct ArrayDecoder<'a> {
    bits: BitReader<'a>,
}

impl<'a> ArrayDecoder<'a> {
    fn new(column: &'a Column) -> Self {
        Self {
            bits: BitReader::new(&column.data),
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
struct StringDict {
    values: Vec<String>,
}

impl StringDict {
    fn reset(&mut self) {
        self.values.clear();
    }

    fn decode(&mut self, value: i64, reader: &mut BytesReader<'_>) -> Result<String, Error> {
        if value >= 0 {
            let len = usize::try_from(value).map_err(|_| Error::ValueOutOfRange("string"))?;
            let value = reader.read_utf8_string(len)?;
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

    fn read_dict_string(&mut self, dict: &mut StringDict) -> Result<String, Error> {
        let value = self.read_varint()?;
        dict.decode(value, self)
    }

    fn read_plain_bytes(&mut self) -> Result<Vec<u8>, Error> {
        let len = usize::try_from(self.read_varint()?)
            .map_err(|_| Error::ValueOutOfRange("bytes length"))?;
        Ok(self.read_bytes(len)?.to_vec())
    }

    fn read_utf8_string(&mut self, len: usize) -> Result<String, Error> {
        let bytes = self.read_bytes(len)?;
        std::str::from_utf8(bytes)
            .map(|value| value.to_owned())
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

fn read_column_sizes(
    column: &mut Column,
    sizes: &mut BitReader<'_>,
    mut read_limit: u64,
) -> Result<(), Error> {
    read_column_sizes_inner(column, sizes, &mut read_limit)
}

fn read_column_sizes_inner(
    column: &mut Column,
    sizes: &mut BitReader<'_>,
    read_limit: &mut u64,
) -> Result<(), Error> {
    let size = sizes.read_uvarint_compact()?;
    if size > *read_limit {
        return Err(Error::InvalidFrame("column exceeds frame"));
    }
    *read_limit -= size;
    column.data = vec![0; usize::try_from(size).map_err(|_| Error::ValueOutOfRange("column"))?];
    if size == 0 {
        reset_column_data(column);
        return Ok(());
    }
    for child in &mut column.children {
        read_column_sizes_inner(child, sizes, read_limit)?;
    }
    Ok(())
}

fn reset_column_data(column: &mut Column) {
    column.data.clear();
    for child in &mut column.children {
        reset_column_data(child);
    }
}

fn read_column_data(column: &mut Column, reader: &mut SliceReader<'_>) -> Result<(), Error> {
    if column.data.is_empty() {
        return Ok(());
    }
    let bytes = reader.read_bytes(column.data.len())?;
    column.data.copy_from_slice(bytes);
    for child in &mut column.children {
        read_column_data(child, reader)?;
    }
    Ok(())
}

/// Builds a default one-point gauge request for STEF tests and examples.
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn example_gauge_request() -> ExportMetricsServiceRequest {
    use crate::proto::opentelemetry::resource::v1::Resource;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::metrics::v1::{AggregationTemporality, Sum};

    #[test]
    fn encodes_metrics_stream_header_and_schema() {
        let bytes = encode_metrics_request(&example_gauge_request()).unwrap();
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
        let _ = encode_metrics_request(&request).unwrap();
    }

    #[test]
    fn roundtrips_numeric_gauge_stream() {
        let bytes = encode_metrics_request(&example_gauge_request()).unwrap();
        let decoded = decode_metrics_request(&bytes).unwrap();

        let resource_metrics = &decoded.resource_metrics[0];
        assert_eq!(
            resource_metrics.resource.as_ref().unwrap().attributes[0].key,
            "service.name"
        );
        let scope_metrics = &resource_metrics.scope_metrics[0];
        let metric = &scope_metrics.metrics[0];
        assert_eq!(metric.name, "requests");
        assert_eq!(metric.unit, "1");

        let metric::Data::Gauge(gauge) = metric.data.as_ref().unwrap() else {
            panic!("expected gauge metric");
        };
        let point = &gauge.data_points[0];
        assert_eq!(point.start_time_unix_nano, 10);
        assert_eq!(point.time_unix_nano, 20);
        assert_eq!(point.attributes[0].key, "route");
        assert_eq!(point.value, Some(number_data_point::Value::AsInt(7)));
    }
}
