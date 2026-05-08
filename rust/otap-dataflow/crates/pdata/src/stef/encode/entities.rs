// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Encoders for the logical STEF record entities: metric, resource, scope, and point.
//!
//! These encoders track the last emitted value for each entity because STEF stores only
//! modified fields per record. The high-level stream encoder decides which entities are
//! present in a record; this module decides which fields inside those entities changed.

use super::super::wire::{
    BoolEncoder, ExemplarArrayEncoder, Float64ArrayEncoder, Float64Encoder, I64Encoder,
    METRIC_FIELD_MASK, RESOURCE_FIELD_MASK, SCOPE_FIELD_MASK, StringEncoder, U64Encoder,
    metric_column_tree, point_column_tree, point_value_column_tree, resource_column_tree,
    scope_column_tree,
};
use super::attributes::AttributesEncoder;
use super::*;

pub(super) struct ViewMetric<'a, M> {
    pub(super) metric: &'a M,
    pub(super) r#type: u64,
    pub(super) aggregation_temporality: u64,
    pub(super) monotonic: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct StefPoint {
    pub(super) start_timestamp: u64,
    pub(super) timestamp: u64,
    pub(super) value: StefPointValue,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum StefPointValue {
    None,
    Int64(i64),
    Float64(f64),
}

#[derive(Default)]
pub(super) struct MetricEncoder {
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
    pub(super) fn new(
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

    pub(super) fn encode_view<M>(&mut self, metric: &ViewMetric<'_, M>) -> Result<(), Error>
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
    pub(super) fn encode_otap(
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

    pub(super) fn take_column(&mut self) -> Column {
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
pub(super) struct ResourceEncoder {
    bits: BitWriter,
    schema_url: StringEncoder,
    attributes: AttributesEncoder,
    dropped_attributes_count: U64Encoder,
}

impl ResourceEncoder {
    pub(super) fn new(
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

    pub(super) fn encode_view<R>(
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

    pub(super) fn encode_otap(
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

    pub(super) fn take_column(&mut self) -> Column {
        let mut column = resource_column_tree();
        column.data = self.bits.take_bytes();
        column.children[0].data = self.schema_url.take_bytes();
        column.children[1] = self.attributes.take_column();
        column.children[2].data = self.dropped_attributes_count.take_bytes();
        column
    }
}

#[derive(Default)]
pub(super) struct ScopeEncoder {
    bits: BitWriter,
    name: StringEncoder,
    version: StringEncoder,
    schema_url: StringEncoder,
    attributes: AttributesEncoder,
    dropped_attributes_count: U64Encoder,
}

impl ScopeEncoder {
    pub(super) fn new(
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

    pub(super) fn encode_view<S>(
        &mut self,
        schema_url: Str<'_>,
        scope: Option<&S>,
    ) -> Result<(), Error>
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

    pub(super) fn encode_otap(
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

    pub(super) fn take_column(&mut self) -> Column {
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
pub(super) struct PointEncoder {
    bits: BitWriter,
    start_timestamp: U64Encoder,
    timestamp: U64Encoder,
    value: PointValueEncoder,
    exemplars: ExemplarArrayEncoder,
    last: Option<StefPoint>,
}

impl PointEncoder {
    pub(super) fn encode(&mut self, point: &StefPoint) -> Result<(), Error> {
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

    pub(super) fn encode_otap_number_point(
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

    pub(super) fn take_column(&mut self) -> Column {
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
pub(super) struct PointValueEncoder {
    bits: BitWriter,
    int64: I64Encoder,
    float64: Float64Encoder,
}

impl PointValueEncoder {
    pub(super) fn encode(&mut self, value: &StefPointValue) {
        match value {
            StefPointValue::None => self.encode_none(),
            StefPointValue::Int64(value) => self.encode_int64(*value),
            StefPointValue::Float64(value) => self.encode_float64(*value),
        }
    }

    pub(super) fn encode_none(&mut self) {
        self.bits.write_bits(0, 3);
    }

    pub(super) fn encode_int64(&mut self, value: i64) {
        self.bits.write_bits(1, 3);
        self.int64.encode(value);
    }

    pub(super) fn encode_float64(&mut self, value: f64) {
        self.bits.write_bits(2, 3);
        self.float64.encode(value);
    }

    pub(super) fn take_column(&mut self) -> Column {
        let mut column = point_value_column_tree();
        column.data = self.bits.take_bytes();
        column.children[0].data = self.int64.take_bytes();
        column.children[1].data = self.float64.take_bytes();
        column
    }
}
