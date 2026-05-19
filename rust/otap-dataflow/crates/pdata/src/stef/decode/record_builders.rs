// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Specialized record-batch builders for decoded number data points and attributes.

use arrow::array::{
    ArrayRef, BinaryArray, BooleanArray, Float64Array, Int64Array, RecordBatch, StringArray,
    TimestampNanosecondArray, UInt8Array, UInt16Array, UInt32Array,
};
use arrow::datatypes::ArrowPrimitiveType;
use arrow::datatypes::{Field, Schema};
use arrow::error::ArrowError;
use std::sync::Arc;

use super::entities::{DecodedAnyValue, DecodedAttribute};
use crate::encode::record::attributes::{
    AttributesRecordBatchBuilder, AttributesRecordBatchBuilderConstructorHelper,
};
use crate::otlp::attributes::AttributeValueType;
use crate::schema::{FieldExt, consts};

#[derive(Default)]
pub(super) struct DirectNumberDataPointsRecordBatchBuilder {
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
    pub(super) fn reserve(&mut self, additional: usize) {
        self.id.reserve(additional);
        self.parent_id.reserve(additional);
        self.start_time_unix_nano.reserve(additional);
        self.time_unix_nano.reserve(additional);
        self.int_value.reserve(additional);
        self.double_value.reserve(additional);
        self.flags.reserve(additional);
    }

    pub(super) fn append_id(&mut self, value: u32) {
        self.id.push(value);
    }

    pub(super) fn append_parent_id(&mut self, value: u16) {
        self.parent_id.push(value);
    }

    pub(super) fn append_start_time_unix_nano(&mut self, value: i64) {
        self.start_time_unix_nano.push(value);
    }

    pub(super) fn append_time_unix_nano(&mut self, value: i64) {
        self.time_unix_nano.push(value);
    }

    pub(super) fn append_int_value(&mut self, value: Option<i64>) {
        self.int_value.push(value);
    }

    pub(super) fn append_double_value(&mut self, value: Option<f64>) {
        self.double_value.push(value);
    }

    pub(super) fn append_flags(&mut self, value: u32) {
        self.has_nonzero_flags |= value != 0;
        self.flags.push(value);
    }

    pub(super) fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
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
pub(super) struct DirectNumberDpAttrsRecordBatchBuilder {
    parent_id: Vec<u32>,
    key: Vec<Arc<str>>,
    attr_type: Vec<u8>,
    str_value: Vec<Option<Arc<str>>>,
    int_value: Vec<Option<i64>>,
    double_value: Vec<Option<f64>>,
    bool_value: Vec<Option<bool>>,
    bytes_value: Vec<Option<Arc<[u8]>>>,
    pending_frame_points: Option<usize>,
    reserved_row_capacity: usize,
    repeated_plan: DirectNumberDpAttrsAppendPlan,
}

impl DirectNumberDpAttrsRecordBatchBuilder {
    pub(super) fn begin_frame(&mut self, point_count: usize) {
        self.pending_frame_points = Some(point_count);
        self.reserve_additional_rows(point_count);
    }

    pub(super) fn reserve_frame_rows_for_attr_count(&mut self, attrs_per_point: usize) {
        let Some(point_count) = self.pending_frame_points.take() else {
            return;
        };
        self.reserve_additional_rows(point_count.saturating_mul(attrs_per_point));
    }

    pub(super) fn reserve_additional_rows(&mut self, additional: usize) {
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

    pub(super) fn append_all(&mut self, parent_id: u32, attrs: &[DecodedAttribute]) {
        self.reserve_frame_rows_for_attr_count(attrs.len());
        if self.repeated_plan.key.len() != attrs.len() {
            self.repeated_plan.rebuild(attrs);
        }
        self.append_repeated_plan(parent_id);
    }

    pub(super) fn rebuild_repeated_plan(&mut self, attrs: &[DecodedAttribute]) {
        self.repeated_plan.rebuild(attrs);
    }

    pub(super) fn repeated_plan_len(&self) -> usize {
        self.repeated_plan.key.len()
    }

    pub(super) fn update_repeated_plan_value(&mut self, index: usize, attr: &DecodedAttribute) {
        self.repeated_plan.update_value(index, attr);
    }

    pub(super) fn append(&mut self, parent_id: u32, kv: &DecodedAttribute) {
        let row_index = self.parent_id.len();
        self.parent_id.push(parent_id);
        self.key.push(kv.key.clone());

        match &kv.value {
            DecodedAnyValue::Empty => {
                self.attr_type.push(AttributeValueType::Empty as u8);
                self.push_null_existing_values();
            }
            DecodedAnyValue::String(value) => {
                self.attr_type.push(AttributeValueType::Str as u8);
                self.push_str_value(row_index, value);
                self.push_null_int_value();
                self.push_null_double_value();
                self.push_null_bool_value();
                self.push_null_bytes_value();
            }
            DecodedAnyValue::Bool(value) => {
                self.attr_type.push(AttributeValueType::Bool as u8);
                self.push_null_str_value();
                self.push_null_int_value();
                self.push_null_double_value();
                self.push_bool_value(row_index, *value);
                self.push_null_bytes_value();
            }
            DecodedAnyValue::Int(value) => {
                self.attr_type.push(AttributeValueType::Int as u8);
                self.push_null_str_value();
                self.push_int_value(row_index, *value);
                self.push_null_double_value();
                self.push_null_bool_value();
                self.push_null_bytes_value();
            }
            DecodedAnyValue::Double(value) => {
                self.attr_type.push(AttributeValueType::Double as u8);
                self.push_null_str_value();
                self.push_null_int_value();
                self.push_double_value(row_index, *value);
                self.push_null_bool_value();
                self.push_null_bytes_value();
            }
            DecodedAnyValue::Bytes(value) => {
                self.attr_type.push(AttributeValueType::Bytes as u8);
                self.push_null_str_value();
                self.push_null_int_value();
                self.push_null_double_value();
                self.push_null_bool_value();
                self.push_bytes_value(row_index, value);
            }
        }
    }

    pub(super) fn append_repeated_plan(&mut self, parent_id: u32) {
        let row_count = self.repeated_plan.key.len();
        if row_count == 0 {
            return;
        }

        let row_index = self.parent_id.len();
        self.parent_id
            .extend(std::iter::repeat_n(parent_id, row_count));
        self.key.extend_from_slice(&self.repeated_plan.key);
        self.attr_type
            .extend_from_slice(&self.repeated_plan.attr_type);
        extend_optional_values(
            &mut self.str_value,
            row_index,
            &self.repeated_plan.str_value,
            self.reserved_row_capacity,
            self.repeated_plan.has_str_value,
        );
        extend_optional_values(
            &mut self.int_value,
            row_index,
            &self.repeated_plan.int_value,
            self.reserved_row_capacity,
            self.repeated_plan.has_int_value,
        );
        extend_optional_values(
            &mut self.double_value,
            row_index,
            &self.repeated_plan.double_value,
            self.reserved_row_capacity,
            self.repeated_plan.has_double_value,
        );
        extend_optional_values(
            &mut self.bool_value,
            row_index,
            &self.repeated_plan.bool_value,
            self.reserved_row_capacity,
            self.repeated_plan.has_bool_value,
        );
        extend_optional_values(
            &mut self.bytes_value,
            row_index,
            &self.repeated_plan.bytes_value,
            self.reserved_row_capacity,
            self.repeated_plan.has_bytes_value,
        );
    }

    pub(super) fn push_null_existing_values(&mut self) {
        self.push_null_str_value();
        self.push_null_int_value();
        self.push_null_double_value();
        self.push_null_bool_value();
        self.push_null_bytes_value();
    }

    pub(super) fn push_str_value(&mut self, row_index: usize, value: &Arc<str>) {
        if self.str_value.is_empty() {
            self.str_value.resize(row_index, None);
            reserve_vec_to(&mut self.str_value, self.reserved_row_capacity);
        }
        self.str_value.push(Some(value.clone()));
    }

    pub(super) fn push_null_str_value(&mut self) {
        if !self.str_value.is_empty() {
            self.str_value.push(None);
        }
    }

    pub(super) fn push_int_value(&mut self, row_index: usize, value: i64) {
        if self.int_value.is_empty() {
            self.int_value.resize(row_index, None);
            reserve_vec_to(&mut self.int_value, self.reserved_row_capacity);
        }
        self.int_value.push(Some(value));
    }

    pub(super) fn push_null_int_value(&mut self) {
        if !self.int_value.is_empty() {
            self.int_value.push(None);
        }
    }

    pub(super) fn push_double_value(&mut self, row_index: usize, value: f64) {
        if self.double_value.is_empty() {
            self.double_value.resize(row_index, None);
            reserve_vec_to(&mut self.double_value, self.reserved_row_capacity);
        }
        self.double_value.push(Some(value));
    }

    pub(super) fn push_null_double_value(&mut self) {
        if !self.double_value.is_empty() {
            self.double_value.push(None);
        }
    }

    pub(super) fn push_bool_value(&mut self, row_index: usize, value: bool) {
        if self.bool_value.is_empty() {
            self.bool_value.resize(row_index, None);
            reserve_vec_to(&mut self.bool_value, self.reserved_row_capacity);
        }
        self.bool_value.push(Some(value));
    }

    pub(super) fn push_null_bool_value(&mut self) {
        if !self.bool_value.is_empty() {
            self.bool_value.push(None);
        }
    }

    pub(super) fn push_bytes_value(&mut self, row_index: usize, value: &Arc<[u8]>) {
        if self.bytes_value.is_empty() {
            self.bytes_value.resize(row_index, None);
            reserve_vec_to(&mut self.bytes_value, self.reserved_row_capacity);
        }
        self.bytes_value.push(Some(value.clone()));
    }

    pub(super) fn push_null_bytes_value(&mut self) {
        if !self.bytes_value.is_empty() {
            self.bytes_value.push(None);
        }
    }

    pub(super) fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
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
            self.key.iter().map(AsRef::as_ref),
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

#[derive(Default)]
pub(super) struct DirectNumberDpAttrsAppendPlan {
    key: Vec<Arc<str>>,
    attr_type: Vec<u8>,
    str_value: Vec<Option<Arc<str>>>,
    int_value: Vec<Option<i64>>,
    double_value: Vec<Option<f64>>,
    bool_value: Vec<Option<bool>>,
    bytes_value: Vec<Option<Arc<[u8]>>>,
    has_str_value: bool,
    has_int_value: bool,
    has_double_value: bool,
    has_bool_value: bool,
    has_bytes_value: bool,
}

impl DirectNumberDpAttrsAppendPlan {
    pub(super) fn rebuild(&mut self, attrs: &[DecodedAttribute]) {
        self.key.clear();
        self.attr_type.clear();
        self.str_value.clear();
        self.int_value.clear();
        self.double_value.clear();
        self.bool_value.clear();
        self.bytes_value.clear();
        self.has_str_value = false;
        self.has_int_value = false;
        self.has_double_value = false;
        self.has_bool_value = false;
        self.has_bytes_value = false;

        self.key.reserve(attrs.len());
        self.attr_type.reserve(attrs.len());
        self.str_value.reserve(attrs.len());
        self.int_value.reserve(attrs.len());
        self.double_value.reserve(attrs.len());
        self.bool_value.reserve(attrs.len());
        self.bytes_value.reserve(attrs.len());

        for kv in attrs {
            self.key.push(kv.key.clone());
            match &kv.value {
                DecodedAnyValue::Empty => {
                    self.attr_type.push(AttributeValueType::Empty as u8);
                    self.push_null_values();
                }
                DecodedAnyValue::String(value) => {
                    self.attr_type.push(AttributeValueType::Str as u8);
                    self.str_value.push(Some(value.clone()));
                    self.int_value.push(None);
                    self.double_value.push(None);
                    self.bool_value.push(None);
                    self.bytes_value.push(None);
                    self.has_str_value = true;
                }
                DecodedAnyValue::Bool(value) => {
                    self.attr_type.push(AttributeValueType::Bool as u8);
                    self.str_value.push(None);
                    self.int_value.push(None);
                    self.double_value.push(None);
                    self.bool_value.push(Some(*value));
                    self.bytes_value.push(None);
                    self.has_bool_value = true;
                }
                DecodedAnyValue::Int(value) => {
                    self.attr_type.push(AttributeValueType::Int as u8);
                    self.str_value.push(None);
                    self.int_value.push(Some(*value));
                    self.double_value.push(None);
                    self.bool_value.push(None);
                    self.bytes_value.push(None);
                    self.has_int_value = true;
                }
                DecodedAnyValue::Double(value) => {
                    self.attr_type.push(AttributeValueType::Double as u8);
                    self.str_value.push(None);
                    self.int_value.push(None);
                    self.double_value.push(Some(*value));
                    self.bool_value.push(None);
                    self.bytes_value.push(None);
                    self.has_double_value = true;
                }
                DecodedAnyValue::Bytes(value) => {
                    self.attr_type.push(AttributeValueType::Bytes as u8);
                    self.str_value.push(None);
                    self.int_value.push(None);
                    self.double_value.push(None);
                    self.bool_value.push(None);
                    self.bytes_value.push(Some(value.clone()));
                    self.has_bytes_value = true;
                }
            }
        }
    }

    pub(super) fn update_value(&mut self, index: usize, attr: &DecodedAttribute) {
        self.key[index] = attr.key.clone();
        match &attr.value {
            DecodedAnyValue::Empty => {
                self.attr_type[index] = AttributeValueType::Empty as u8;
                self.set_null_values(index);
            }
            DecodedAnyValue::String(value) => {
                self.attr_type[index] = AttributeValueType::Str as u8;
                self.str_value[index] = Some(value.clone());
                self.int_value[index] = None;
                self.double_value[index] = None;
                self.bool_value[index] = None;
                self.bytes_value[index] = None;
                self.has_str_value = true;
            }
            DecodedAnyValue::Bool(value) => {
                self.attr_type[index] = AttributeValueType::Bool as u8;
                self.str_value[index] = None;
                self.int_value[index] = None;
                self.double_value[index] = None;
                self.bool_value[index] = Some(*value);
                self.bytes_value[index] = None;
                self.has_bool_value = true;
            }
            DecodedAnyValue::Int(value) => {
                self.attr_type[index] = AttributeValueType::Int as u8;
                self.str_value[index] = None;
                self.int_value[index] = Some(*value);
                self.double_value[index] = None;
                self.bool_value[index] = None;
                self.bytes_value[index] = None;
                self.has_int_value = true;
            }
            DecodedAnyValue::Double(value) => {
                self.attr_type[index] = AttributeValueType::Double as u8;
                self.str_value[index] = None;
                self.int_value[index] = None;
                self.double_value[index] = Some(*value);
                self.bool_value[index] = None;
                self.bytes_value[index] = None;
                self.has_double_value = true;
            }
            DecodedAnyValue::Bytes(value) => {
                self.attr_type[index] = AttributeValueType::Bytes as u8;
                self.str_value[index] = None;
                self.int_value[index] = None;
                self.double_value[index] = None;
                self.bool_value[index] = None;
                self.bytes_value[index] = Some(value.clone());
                self.has_bytes_value = true;
            }
        }
    }

    pub(super) fn push_null_values(&mut self) {
        self.str_value.push(None);
        self.int_value.push(None);
        self.double_value.push(None);
        self.bool_value.push(None);
        self.bytes_value.push(None);
    }

    pub(super) fn set_null_values(&mut self, index: usize) {
        self.str_value[index] = None;
        self.int_value[index] = None;
        self.double_value[index] = None;
        self.bool_value[index] = None;
        self.bytes_value[index] = None;
    }
}

pub(super) fn reserve_vec_to<T>(values: &mut Vec<T>, target_capacity: usize) {
    if values.capacity() < target_capacity {
        values.reserve(target_capacity - values.capacity());
    }
}

pub(super) fn extend_optional_values<T: Clone>(
    values: &mut Vec<Option<T>>,
    row_index: usize,
    rows: &[Option<T>],
    target_capacity: usize,
    has_values: bool,
) {
    if rows.is_empty() {
        return;
    }
    if values.is_empty() && has_values {
        values.resize(row_index, None);
        reserve_vec_to(values, target_capacity);
    }
    if !values.is_empty() {
        values.extend(rows.iter().cloned());
    }
}

pub(super) fn append_decoded_attribute_with_parent<T>(
    attribute_rb_builder: &mut AttributesRecordBatchBuilder<T>,
    parent_id: &<<T as crate::otlp::attributes::parent_id::ParentId>::ArrayType as ArrowPrimitiveType>::Native,
    kv: &DecodedAttribute,
) where
    T: crate::otlp::attributes::parent_id::ParentId + AttributesRecordBatchBuilderConstructorHelper,
{
    attribute_rb_builder.append_parent_id(parent_id);
    append_decoded_attribute_value(attribute_rb_builder, kv);
}

pub(super) fn append_decoded_attribute_value<T>(
    attribute_rb_builder: &mut AttributesRecordBatchBuilder<T>,
    kv: &DecodedAttribute,
) where
    T: crate::otlp::attributes::parent_id::ParentId + AttributesRecordBatchBuilderConstructorHelper,
{
    attribute_rb_builder.append_key(kv.key.as_bytes());
    match &kv.value {
        DecodedAnyValue::Empty => attribute_rb_builder.any_values_builder.append_empty(),
        DecodedAnyValue::String(value) => attribute_rb_builder
            .any_values_builder
            .append_str(value.as_bytes()),
        DecodedAnyValue::Bool(value) => {
            attribute_rb_builder.any_values_builder.append_bool(*value);
        }
        DecodedAnyValue::Int(value) => {
            attribute_rb_builder.any_values_builder.append_int(*value);
        }
        DecodedAnyValue::Double(value) => {
            attribute_rb_builder
                .any_values_builder
                .append_double(*value);
        }
        DecodedAnyValue::Bytes(value) => {
            attribute_rb_builder.any_values_builder.append_bytes(value);
        }
    }
}
