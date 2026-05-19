// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Decoders for logical STEF record entities.
//!
//! STEF records are sparse: a record carries only the fields that changed from previous
//! records. These decoders maintain that rolling state and expose a complete logical record
//! to the OTAP builder after each decoded row.

use super::super::Error;
use super::super::wire::{
    ArrayDecoder, BitReader, BoolDecoder, BytesReader, DecodeColumn, DirectStringDict,
    Float64Decoder, Float64DecoderState, I64Decoder, U64Decoder, U64DecoderState,
};
use super::builder::DirectOtapMetricsBuilder;
use super::record_builders::DirectNumberDpAttrsRecordBatchBuilder;
use std::sync::Arc;

#[derive(Default)]
pub(super) struct DirectMetricsFrameDecoder<'a> {
    root: BitReader<'a>,
    metric: DirectMetricDecoder<'a>,
    resource: DirectResourceDecoder<'a>,
    scope: DirectScopeDecoder<'a>,
    attributes: AttributesDecoder<'a>,
    point: PointDecoder<'a>,
}

impl<'a> DirectMetricsFrameDecoder<'a> {
    pub(super) fn new(columns: &DecodeColumn<'a>, codecs: &DirectDecoderCodecs) -> Self {
        Self {
            root: BitReader::new(columns.data),
            metric: DirectMetricDecoder::new(&columns.children[1], &codecs.metric),
            resource: DirectResourceDecoder::new(&columns.children[2], &codecs.resource),
            scope: DirectScopeDecoder::new(&columns.children[3], &codecs.scope),
            attributes: AttributesDecoder::new(&columns.children[4], &codecs.attributes),
            point: PointDecoder::new(&columns.children[5], &codecs.point),
        }
    }

    pub(super) fn save_codecs(&self, codecs: &mut DirectDecoderCodecs) {
        self.metric.save_codecs(&mut codecs.metric);
        self.resource.save_codecs(&mut codecs.resource);
        self.scope.save_codecs(&mut codecs.scope);
        self.attributes.save_codecs(&mut codecs.attributes);
        self.point.save_codecs(&mut codecs.point);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn decode_record(
        &mut self,
        builder: &mut DirectOtapMetricsBuilder,
        resource: &mut DirectDecResource,
        scope: &mut DirectDecScope,
        metric: &mut DirectDecMetric,
        attrs: &mut Vec<DecodedAttribute>,
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
pub(super) struct RootModified {
    pub(super) metric: bool,
    pub(super) resource: bool,
    pub(super) scope: bool,
}

#[derive(Clone, Default)]
pub(super) struct DirectDecoderCodecs {
    metric: MetricDecoderCodecs,
    resource: ResourceDecoderCodecs,
    scope: ScopeDecoderCodecs,
    attributes: AttributesDecoderCodecs,
    point: PointDecoderCodecs,
}

#[derive(Clone, Default)]
struct MetricDecoderCodecs {
    r#type: U64DecoderState,
    metadata: AttributesDecoderCodecs,
    aggregation_temporality: U64DecoderState,
}

#[derive(Clone, Default)]
struct ResourceDecoderCodecs {
    attributes: AttributesDecoderCodecs,
    dropped_attributes_count: U64DecoderState,
}

#[derive(Clone, Default)]
struct ScopeDecoderCodecs {
    attributes: AttributesDecoderCodecs,
    dropped_attributes_count: U64DecoderState,
}

#[derive(Clone, Default)]
struct PointDecoderCodecs {
    start_timestamp: U64DecoderState,
    timestamp: U64DecoderState,
    value: PointValueDecoderCodecs,
}

#[derive(Clone, Default)]
struct PointValueDecoderCodecs {
    int64: U64DecoderState,
    float64: Float64DecoderState,
}

#[derive(Clone, Default)]
struct AttributesDecoderCodecs {
    value: AnyValueDecoderCodecs,
}

#[derive(Clone, Default)]
struct AnyValueDecoderCodecs {
    int64: U64DecoderState,
    float64: Float64DecoderState,
}

pub(super) struct DirectDecoderState {
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
    pub(super) codecs: DirectDecoderCodecs,
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
            codecs: DirectDecoderCodecs::default(),
        };
        state.reset_dictionaries();
        state
    }
}

impl DirectDecoderState {
    pub(super) fn reset_dictionaries(&mut self) {
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

    pub(super) fn reset_codecs(&mut self) {
        self.codecs = DirectDecoderCodecs::default();
    }
}

#[derive(Clone, Default)]
pub(super) struct DecPoint {
    pub(super) start_timestamp: u64,
    pub(super) timestamp: u64,
    pub(super) value: DecPointValue,
}

#[derive(Clone, Copy, Default)]
pub(super) enum DecPointValue {
    #[default]
    None,
    Int64(i64),
    Float64(f64),
}

#[derive(Clone)]
pub(super) struct DirectDecResource {
    pub(super) schema_url: Arc<str>,
    pub(super) attributes: Vec<DecodedAttribute>,
    pub(super) dropped_attributes_count: u32,
}

impl Default for DirectDecResource {
    fn default() -> Self {
        Self {
            schema_url: Arc::from(""),
            attributes: Vec::new(),
            dropped_attributes_count: 0,
        }
    }
}

#[derive(Clone)]
pub(super) struct DirectDecScope {
    pub(super) name: Arc<str>,
    pub(super) version: Arc<str>,
    pub(super) schema_url: Arc<str>,
    pub(super) attributes: Vec<DecodedAttribute>,
    pub(super) dropped_attributes_count: u32,
}

impl Default for DirectDecScope {
    fn default() -> Self {
        Self {
            name: Arc::from(""),
            version: Arc::from(""),
            schema_url: Arc::from(""),
            attributes: Vec::new(),
            dropped_attributes_count: 0,
        }
    }
}

#[derive(Clone)]
pub(super) struct DirectDecMetric {
    pub(super) name: Arc<str>,
    pub(super) description: Arc<str>,
    pub(super) unit: Arc<str>,
    pub(super) r#type: u64,
    pub(super) metadata: Vec<DecodedAttribute>,
    pub(super) aggregation_temporality: u64,
    pub(super) monotonic: bool,
}

impl Default for DirectDecMetric {
    fn default() -> Self {
        Self {
            name: Arc::from(""),
            description: Arc::from(""),
            unit: Arc::from(""),
            r#type: 0,
            metadata: Vec::new(),
            aggregation_temporality: 0,
            monotonic: false,
        }
    }
}

#[derive(Clone, PartialEq)]
pub(super) struct DecodedAttribute {
    pub(super) key: Arc<str>,
    pub(super) value: DecodedAnyValue,
}

impl Default for DecodedAttribute {
    fn default() -> Self {
        Self {
            key: Arc::from(""),
            value: DecodedAnyValue::Empty,
        }
    }
}

#[derive(Clone, Default, PartialEq)]
pub(super) enum DecodedAnyValue {
    #[default]
    Empty,
    String(Arc<str>),
    Bool(bool),
    Int(i64),
    Double(f64),
    Bytes(Arc<[u8]>),
}

#[derive(Default)]
pub(super) struct DirectMetricDecoder<'a> {
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
    fn new(column: &DecodeColumn<'a>, codecs: &MetricDecoderCodecs) -> Self {
        Self {
            bits: BitReader::new(column.data),
            name: BytesReader::new(column.children[0].data),
            description: BytesReader::new(column.children[1].data),
            unit: BytesReader::new(column.children[2].data),
            r#type: U64Decoder::with_state(column.children[3].data, codecs.r#type),
            metadata: AttributesDecoder::new(&column.children[4], &codecs.metadata),
            histogram_bounds: ArrayDecoder::new(&column.children[5]),
            aggregation_temporality: U64Decoder::with_state(
                column.children[6].data,
                codecs.aggregation_temporality,
            ),
            monotonic: BoolDecoder::new(column.children[7].data),
        }
    }

    fn save_codecs(&self, codecs: &mut MetricDecoderCodecs) {
        codecs.r#type = self.r#type.state();
        self.metadata.save_codecs(&mut codecs.metadata);
        codecs.aggregation_temporality = self.aggregation_temporality.state();
    }

    pub(super) fn decode(
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
pub(super) struct DirectResourceDecoder<'a> {
    bits: BitReader<'a>,
    schema_url: BytesReader<'a>,
    attributes: AttributesDecoder<'a>,
    dropped_attributes_count: U64Decoder<'a>,
}

impl<'a> DirectResourceDecoder<'a> {
    fn new(column: &DecodeColumn<'a>, codecs: &ResourceDecoderCodecs) -> Self {
        Self {
            bits: BitReader::new(column.data),
            schema_url: BytesReader::new(column.children[0].data),
            attributes: AttributesDecoder::new(&column.children[1], &codecs.attributes),
            dropped_attributes_count: U64Decoder::with_state(
                column.children[2].data,
                codecs.dropped_attributes_count,
            ),
        }
    }

    fn save_codecs(&self, codecs: &mut ResourceDecoderCodecs) {
        self.attributes.save_codecs(&mut codecs.attributes);
        codecs.dropped_attributes_count = self.dropped_attributes_count.state();
    }

    pub(super) fn decode(
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
pub(super) struct DirectScopeDecoder<'a> {
    bits: BitReader<'a>,
    name: BytesReader<'a>,
    version: BytesReader<'a>,
    schema_url: BytesReader<'a>,
    attributes: AttributesDecoder<'a>,
    dropped_attributes_count: U64Decoder<'a>,
}

impl<'a> DirectScopeDecoder<'a> {
    fn new(column: &DecodeColumn<'a>, codecs: &ScopeDecoderCodecs) -> Self {
        Self {
            bits: BitReader::new(column.data),
            name: BytesReader::new(column.children[0].data),
            version: BytesReader::new(column.children[1].data),
            schema_url: BytesReader::new(column.children[2].data),
            attributes: AttributesDecoder::new(&column.children[3], &codecs.attributes),
            dropped_attributes_count: U64Decoder::with_state(
                column.children[4].data,
                codecs.dropped_attributes_count,
            ),
        }
    }

    fn save_codecs(&self, codecs: &mut ScopeDecoderCodecs) {
        self.attributes.save_codecs(&mut codecs.attributes);
        codecs.dropped_attributes_count = self.dropped_attributes_count.state();
    }

    pub(super) fn decode(
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
pub(super) struct PointDecoder<'a> {
    bits: BitReader<'a>,
    start_timestamp: U64Decoder<'a>,
    timestamp: U64Decoder<'a>,
    value: PointValueDecoder<'a>,
    exemplars: ArrayDecoder<'a>,
}

impl<'a> PointDecoder<'a> {
    fn new(column: &DecodeColumn<'a>, codecs: &PointDecoderCodecs) -> Self {
        Self {
            bits: BitReader::new(column.data),
            start_timestamp: U64Decoder::with_state(
                column.children[0].data,
                codecs.start_timestamp,
            ),
            timestamp: U64Decoder::with_state(column.children[1].data, codecs.timestamp),
            value: PointValueDecoder::new(&column.children[2], &codecs.value),
            exemplars: ArrayDecoder::new(&column.children[3]),
        }
    }

    fn save_codecs(&self, codecs: &mut PointDecoderCodecs) {
        codecs.start_timestamp = self.start_timestamp.state();
        codecs.timestamp = self.timestamp.state();
        self.value.save_codecs(&mut codecs.value);
    }

    pub(super) fn decode(&mut self, target: &mut DecPoint) -> Result<(), Error> {
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
pub(super) struct PointValueDecoder<'a> {
    bits: BitReader<'a>,
    int64: I64Decoder<'a>,
    float64: Float64Decoder<'a>,
}

impl<'a> PointValueDecoder<'a> {
    fn new(column: &DecodeColumn<'a>, codecs: &PointValueDecoderCodecs) -> Self {
        Self {
            bits: BitReader::new(column.data),
            int64: I64Decoder::with_state(column.children[0].data, codecs.int64),
            float64: Float64Decoder::with_state(column.children[1].data, codecs.float64),
        }
    }

    fn save_codecs(&self, codecs: &mut PointValueDecoderCodecs) {
        codecs.int64 = self.int64.state();
        codecs.float64 = self.float64.state();
    }

    pub(super) fn decode(&mut self) -> Result<DecPointValue, Error> {
        match self.bits.read_bits(3)? {
            0 => Ok(DecPointValue::None),
            1 => Ok(DecPointValue::Int64(self.int64.decode()?)),
            2 => Ok(DecPointValue::Float64(self.float64.decode()?)),
            _ => Err(Error::UnsupportedStefValue("point value")),
        }
    }
}

#[derive(Default)]
pub(super) struct AttributesDecoder<'a> {
    header: BytesReader<'a>,
    key: BytesReader<'a>,
    value: AnyValueDecoder<'a>,
}

impl<'a> AttributesDecoder<'a> {
    fn new(column: &DecodeColumn<'a>, codecs: &AttributesDecoderCodecs) -> Self {
        Self {
            header: BytesReader::new(column.data),
            key: BytesReader::new(column.children[0].data),
            value: AnyValueDecoder::new(&column.children[1], &codecs.value),
        }
    }

    fn save_codecs(&self, codecs: &mut AttributesDecoderCodecs) {
        self.value.save_codecs(&mut codecs.value);
    }

    pub(super) fn decode_direct(
        &mut self,
        target: &mut Vec<DecodedAttribute>,
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
            target.push(DecodedAttribute { key, value });
        }
        Ok(())
    }

    pub(super) fn decode_direct_number_point_attrs(
        &mut self,
        target: &mut Vec<DecodedAttribute>,
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
            let update_plan = attribute_rb_builder.repeated_plan_len() == target.len();
            for (index, item) in target.iter_mut().enumerate() {
                if changed & (1 << index) != 0 {
                    item.value = self.value.decode_direct(state)?;
                    if update_plan {
                        attribute_rb_builder.update_repeated_plan_value(index, item);
                    }
                }
            }
            if !update_plan {
                attribute_rb_builder.rebuild_repeated_plan(target);
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
            let attribute = DecodedAttribute { key, value };
            attribute_rb_builder.append(point_id, &attribute);
            target.push(attribute);
        }
        attribute_rb_builder.rebuild_repeated_plan(target);
        Ok(())
    }
}

#[derive(Default)]
pub(super) struct AnyValueDecoder<'a> {
    bits: BitReader<'a>,
    string: BytesReader<'a>,
    bool_: BoolDecoder<'a>,
    int64: I64Decoder<'a>,
    float64: Float64Decoder<'a>,
    bytes: BytesReader<'a>,
}

impl<'a> AnyValueDecoder<'a> {
    fn new(column: &DecodeColumn<'a>, codecs: &AnyValueDecoderCodecs) -> Self {
        Self {
            bits: BitReader::new(column.data),
            string: BytesReader::new(column.children[0].data),
            bool_: BoolDecoder::new(column.children[1].data),
            int64: I64Decoder::with_state(column.children[2].data, codecs.int64),
            float64: Float64Decoder::with_state(column.children[3].data, codecs.float64),
            bytes: BytesReader::new(column.children[6].data),
        }
    }

    fn save_codecs(&self, codecs: &mut AnyValueDecoderCodecs) {
        codecs.int64 = self.int64.state();
        codecs.float64 = self.float64.state();
    }

    pub(super) fn decode_direct(
        &mut self,
        state: &mut DirectDecoderState,
    ) -> Result<DecodedAnyValue, Error> {
        match self.bits.read_bits(4)? {
            0 => Ok(DecodedAnyValue::Empty),
            1 => Ok(DecodedAnyValue::String(
                self.string
                    .read_direct_dict_string(&mut state.any_value_string)?,
            )),
            2 => Ok(DecodedAnyValue::Bool(self.bool_.decode()?)),
            3 => Ok(DecodedAnyValue::Int(self.int64.decode()?)),
            4 => Ok(DecodedAnyValue::Double(self.float64.decode()?)),
            5 => Err(Error::UnsupportedStefValue("array attribute")),
            6 => Err(Error::UnsupportedStefValue("kvlist attribute")),
            7 => Ok(DecodedAnyValue::Bytes(self.bytes.read_plain_bytes()?)),
            _ => Err(Error::UnsupportedStefValue("attribute value")),
        }
    }
}
