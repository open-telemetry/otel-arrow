// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{
    MaybeDictArrayAccessor, NullableArrayAccessor, StringArrayAccessor, StructColumnAccessor,
    get_bool_array_opt, get_f64_array_opt, get_required_array, get_u8_array,
};
use crate::decode::proto_bytes::{proto_encode_field_tag, proto_encode_varint};
use crate::error::{self, Error, Result};
use crate::otlp::attributes::{AttributeArrays, encode_key_value};
use crate::proto::consts::field_num::common::{
    INSTRUMENTATION_DROPPED_ATTRIBUTES_COUNT, INSTRUMENTATION_SCOPE_ATTRIBUTES,
    INSTRUMENTATION_SCOPE_NAME, INSTRUMENTATION_SCOPE_VERSION,
};
use crate::proto::consts::field_num::resource::{
    RESOURCE_ATTRIBUTES, RESOURCE_DROPPED_ATTRIBUTES_COUNT,
};
use crate::proto::consts::wire_types;
use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, any_value::Value};
use crate::proto_encode_len_delimited_mystery_size;
use crate::schema::consts;
use arrow::array::{
    Array, BinaryArray, BooleanArray, Float64Array, Int64Array, RecordBatch, StringArray,
    StructArray, UInt8Array, UInt16Array, UInt32Array,
};
use arrow::datatypes::{DataType, Field, Fields};
use snafu::OptionExt;
use std::fmt;
use std::fmt::Write;
use std::sync::LazyLock;

pub(in crate::otlp) struct ResourceArrays<'a> {
    pub id: &'a UInt16Array,
    pub dropped_attributes_count: Option<&'a UInt32Array>,
    pub schema_url: Option<StringArrayAccessor<'a>>,
}

static RESOURCE_ARRAY_DATA_TYPE: LazyLock<DataType> = LazyLock::new(|| {
    DataType::Struct(Fields::from(vec![
        Field::new(consts::ID, DataType::UInt16, true),
        Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        Field::new(
            consts::SCHEMA_URL,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            true,
        ),
    ]))
});

impl ResourceArrays<'_> {
    fn data_type() -> &'static DataType {
        &RESOURCE_ARRAY_DATA_TYPE
    }
}

pub(crate) fn proto_encode_resource(
    index: usize,
    resource_arrays: &ResourceArrays<'_>,
    resource_attrs_arrays: Option<&AttributeArrays<'_>>,
    resource_attrs_sorted_indices: &Vec<usize>,
    resource_attrs_sorted_index: &mut usize,
    result_buf: &mut Vec<u8>,
) {
    // add attributes
    if let Some(attrs_arrays) = resource_attrs_arrays {
        if let Some(res_id) = resource_arrays.id.value_at(index) {
            let mut attrs_index_iter = ChildIndexIter {
                parent_id: res_id,
                parent_id_col: attrs_arrays.parent_id,
                sorted_indices: resource_attrs_sorted_indices,
                pos: resource_attrs_sorted_index,
            };

            while let Some(attr_index) = attrs_index_iter.next() {
                let num_bytes = 5;
                proto_encode_len_delimited_mystery_size!(
                    RESOURCE_ATTRIBUTES,
                    num_bytes,
                    encode_key_value(attrs_arrays, attr_index, result_buf),
                    result_buf
                );
            }
        }
    }

    if let Some(col) = resource_arrays.dropped_attributes_count {
        if let Some(val) = col.value_at(index) {
            proto_encode_field_tag(
                RESOURCE_DROPPED_ATTRIBUTES_COUNT,
                wire_types::VARINT,
                result_buf,
            );
            proto_encode_varint(val as u64, result_buf);
        }
    }
}

impl<'a> TryFrom<&'a RecordBatch> for ResourceArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let struct_array = get_required_array(rb, consts::RESOURCE)?;
        let struct_array = struct_array
            .as_any()
            .downcast_ref::<StructArray>()
            .with_context(|| error::ColumnDataTypeMismatchSnafu {
                name: consts::RESOURCE,
                actual: struct_array.data_type().clone(),
                expect: Self::data_type().clone(),
            })?;

        let struct_col_accessor = StructColumnAccessor::new(struct_array);

        Ok(Self {
            id: struct_col_accessor.primitive_column(consts::ID)?,
            dropped_attributes_count: struct_col_accessor
                .primitive_column_op(consts::DROPPED_ATTRIBUTES_COUNT)?,
            schema_url: struct_col_accessor.string_column_op(consts::SCHEMA_URL)?,
        })
    }
}

pub(in crate::otlp) struct ScopeArrays<'a> {
    pub name: Option<StringArrayAccessor<'a>>,
    pub version: Option<StringArrayAccessor<'a>>,
    pub dropped_attributes_count: Option<&'a UInt32Array>,
    pub id: Option<&'a UInt16Array>,
}

pub static SCOPE_ARRAY_DATA_TYPE: LazyLock<DataType> = LazyLock::new(|| {
    DataType::Struct(Fields::from(vec![
        Field::new(
            consts::NAME,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            true,
        ),
        Field::new(consts::VERSION, DataType::Utf8, true),
        Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        Field::new(consts::ID, DataType::UInt16, true),
    ]))
});

impl ScopeArrays<'_> {
    fn data_type() -> &'static DataType {
        &SCOPE_ARRAY_DATA_TYPE
    }

    pub fn create_instrumentation_scope(&self, idx: usize) -> InstrumentationScope {
        InstrumentationScope {
            name: self.name.value_at(idx).unwrap_or_default(),
            version: self.version.value_at_or_default(idx),
            dropped_attributes_count: self.dropped_attributes_count.value_at_or_default(idx),
            attributes: vec![],
        }
    }
}

impl<'a> TryFrom<&'a RecordBatch> for ScopeArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let struct_array = get_required_array(rb, consts::SCOPE)?;
        let scope_array = struct_array
            .as_any()
            .downcast_ref::<StructArray>()
            .with_context(|| error::ColumnDataTypeMismatchSnafu {
                name: consts::RESOURCE,
                actual: struct_array.data_type().clone(),
                expect: Self::data_type().clone(),
            })?;
        let struct_col_accessor = StructColumnAccessor::new(scope_array);

        Ok(Self {
            name: struct_col_accessor.string_column_op(consts::NAME)?,
            version: struct_col_accessor.string_column_op(consts::VERSION)?,
            dropped_attributes_count: struct_col_accessor
                .primitive_column_op(consts::DROPPED_ATTRIBUTES_COUNT)?,
            id: struct_col_accessor.primitive_column_op(consts::ID)?,
        })
    }
}

pub(crate) fn proto_encode_instrumentation_scope(
    index: usize,
    scope_arrays: &ScopeArrays<'_>,
    scope_attrs_arrays: Option<&AttributeArrays<'_>>,
    scope_attrs_sorted_indices: &Vec<usize>,
    scope_attrs_sorted_index: &mut usize,
    result_buf: &mut Vec<u8>,
) {
    if let Some(col) = &scope_arrays.name {
        if let Some(val) = col.value_at(index) {
            proto_encode_field_tag(INSTRUMENTATION_SCOPE_NAME, wire_types::LEN, result_buf);
            proto_encode_varint(val.len() as u64, result_buf);
            result_buf.extend_from_slice(val.as_bytes());
        }
    }

    if let Some(col) = &scope_arrays.version {
        if let Some(val) = col.value_at(index) {
            proto_encode_field_tag(INSTRUMENTATION_SCOPE_VERSION, wire_types::LEN, result_buf);
            proto_encode_varint(val.len() as u64, result_buf);
            result_buf.extend_from_slice(val.as_bytes());
        }
    }

    if let Some(attr_arrays) = scope_attrs_arrays {
        if let Some(scope_id) = scope_arrays.id.value_at(index) {
            let mut attrs_index_iter = ChildIndexIter {
                parent_id: scope_id,
                parent_id_col: attr_arrays.parent_id,
                sorted_indices: scope_attrs_sorted_indices,
                pos: scope_attrs_sorted_index,
            };

            while let Some(attr_index) = attrs_index_iter.next() {
                let num_bytes = 5;
                proto_encode_len_delimited_mystery_size!(
                    INSTRUMENTATION_SCOPE_ATTRIBUTES,
                    num_bytes,
                    encode_key_value(attr_arrays, attr_index, result_buf),
                    result_buf
                );
            }
        }
    }

    if let Some(col) = scope_arrays.dropped_attributes_count {
        if let Some(val) = col.value_at(index) {
            proto_encode_field_tag(
                INSTRUMENTATION_DROPPED_ATTRIBUTES_COUNT,
                wire_types::VARINT,
                result_buf,
            );
            proto_encode_varint(val as u64, result_buf);
        }
    }
}

// display implementation to use for debug processor
impl fmt::Display for AnyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(value) = &self.value {
            match value {
                Value::StringValue(string) => {
                    write!(f, "{string}")?;
                }
                Value::BoolValue(bool) => {
                    write!(f, "{bool}")?;
                }
                Value::IntValue(int) => {
                    write!(f, "{int}")?;
                }
                Value::DoubleValue(double) => {
                    write!(f, "{double}")?;
                }
                Value::ArrayValue(array) => {
                    let values = &array.values;
                    write!(f, "{values:?}")?;
                }
                Value::KvlistValue(kvlist) => {
                    let mut kv_string = String::new();
                    for kv in kvlist.values.iter() {
                        if let Some(value) = &kv.value {
                            _ = write!(
                                &mut kv_string,
                                "{key}={value} ",
                                key = kv.key,
                                value = value
                            );
                        }
                    }
                    write!(f, "{kv_string}")?;
                }
                Value::BytesValue(bytes) => {
                    if let Ok(byte_string) = String::from_utf8(bytes.to_vec()) {
                        write!(f, "{byte_string}")?;
                    }
                    write!(f, "")?;
                }
            }
        } else {
            write!(f, "")?;
        }
        Ok(())
    }
}

pub(crate) struct AnyValueArrays<'a> {
    pub attr_type: &'a UInt8Array,
    // TODO should these just be StringArrayAccessor, etc.
    pub attr_str: Option<MaybeDictArrayAccessor<'a, StringArray>>,
    pub attr_int: Option<MaybeDictArrayAccessor<'a, Int64Array>>,
    pub attr_double: Option<&'a Float64Array>,
    pub attr_bool: Option<&'a BooleanArray>,
    pub attr_bytes: Option<MaybeDictArrayAccessor<'a, BinaryArray>>,
    pub attr_ser: Option<MaybeDictArrayAccessor<'a, BinaryArray>>,
}

impl<'a> TryFrom<&'a RecordBatch> for AnyValueArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let attr_type = get_u8_array(rb, consts::ATTRIBUTE_TYPE)?;
        let attr_str = rb
            .column_by_name(consts::ATTRIBUTE_STR)
            .map(MaybeDictArrayAccessor::<StringArray>::try_new)
            .transpose()?;
        let attr_int = rb
            .column_by_name(consts::ATTRIBUTE_INT)
            .map(MaybeDictArrayAccessor::<Int64Array>::try_new)
            .transpose()?;
        let attr_double = get_f64_array_opt(rb, consts::ATTRIBUTE_DOUBLE)?;
        let attr_bool = get_bool_array_opt(rb, consts::ATTRIBUTE_BOOL)?;
        let attr_bytes = rb
            .column_by_name(consts::ATTRIBUTE_BYTES)
            .map(MaybeDictArrayAccessor::<BinaryArray>::try_new)
            .transpose()?;
        let attr_ser = rb
            .column_by_name(consts::ATTRIBUTE_BYTES)
            .map(MaybeDictArrayAccessor::<BinaryArray>::try_new)
            .transpose()?;

        Ok(Self {
            attr_type,
            attr_str,
            attr_int,
            attr_double,
            attr_bool,
            attr_bytes,
            attr_ser,
        })
    }
}

/// TODO coment on what this is doing
pub(crate) struct ChildIndexIter<'a> {
    pub parent_id: u16,
    pub parent_id_col: &'a UInt16Array,
    pub sorted_indices: &'a Vec<usize>,
    pub pos: &'a mut usize,
}

impl Iterator for ChildIndexIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        // we've iterated to the end of the attributes array - no more attributes
        if *self.pos >= self.sorted_indices.len() {
            return None;
        }

        let index = self.sorted_indices[*self.pos];

        // TODO there's a bug in here that we actually need to increment pos
        // until it's greater than or equal to parent_id? I guess in case there's
        // a parent ID w/ no attributes or something (e.g. attrs dropped).

        if Some(self.parent_id) == self.parent_id_col.value_at(index) {
            *self.pos += 1;
            Some(index)
        } else {
            None
        }
    }
}
