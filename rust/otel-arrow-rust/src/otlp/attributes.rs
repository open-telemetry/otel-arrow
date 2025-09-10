// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{RecordBatch, StringArray, UInt16Array};
use prost::Message;
use snafu::OptionExt;

use crate::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor, get_u16_array};
use crate::error::{self, Error, Result};
use crate::otlp::attributes::store::AttributeValueType;
use crate::otlp::common::{AnyValueArrays, ProtoBuffer};
use crate::proto::consts::field_num::common::{
    ANY_VALUE_BOOL_VALUE, ANY_VALUE_BYES_VALUE, ANY_VALUE_DOUBLE_VALUE, ANY_VALUE_INT_VALUE,
    ANY_VALUE_STRING_VALUE, KEY_VALUE_KEY, KEY_VALUE_VALUE,
};
use crate::proto::consts::wire_types;
use crate::proto_encode_len_delimited_unknown_size;
use crate::schema::consts;

pub mod cbor;
pub mod decoder;
pub mod parent_id;
pub mod store;

pub(crate) struct AttributeArrays<'a> {
    pub parent_id: &'a UInt16Array,
    pub attr_key: MaybeDictArrayAccessor<'a, StringArray>,
    pub anyval_arrays: AnyValueArrays<'a>,
}

impl<'a> TryFrom<&'a RecordBatch> for AttributeArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let parent_id = get_u16_array(rb, consts::PARENT_ID)?;

        let key = rb.column_by_name(consts::ATTRIBUTE_KEY).with_context(|| {
            error::ColumnNotFoundSnafu {
                name: consts::ATTRIBUTE_KEY,
            }
        })?;
        let attr_key = MaybeDictArrayAccessor::<StringArray>::try_new(key)?;

        let anyval_arrays = AnyValueArrays::try_from(rb)?;

        Ok(Self {
            parent_id,
            attr_key,
            anyval_arrays,
        })
    }
}

pub(crate) fn encode_key_value(
    attr_arrays: &AttributeArrays<'_>,
    index: usize,
    result_buf: &mut ProtoBuffer,
) {
    if let Some(key) = attr_arrays.attr_key.value_at(index) {
        result_buf.encode_field_tag(KEY_VALUE_KEY, wire_types::LEN);
        result_buf.encode_varint(key.len() as u64);
        result_buf.extend_from_slice(key.as_bytes());
    }

    if let Some(value_type) = attr_arrays.anyval_arrays.attr_type.value_at(index) {
        if let Ok(value_type) = AttributeValueType::try_from(value_type) {
            // TODO just guessing the max byte length for attributes is probably the biggest contributor to
            // wasting space when doing this encoding. if there's anywhere we want to optimize the mystery
            // size guess it's here
            proto_encode_len_delimited_unknown_size!(
                KEY_VALUE_VALUE,
                encode_any_value(&attr_arrays.anyval_arrays, index, value_type, result_buf),
                result_buf
            );
        }
    }
}

pub(crate) fn encode_any_value(
    attr_arrays: &AnyValueArrays<'_>,
    index: usize,
    value_type: AttributeValueType,
    result_buf: &mut ProtoBuffer,
) {
    match value_type {
        AttributeValueType::Str => {
            if let Some(attr_str) = &attr_arrays.attr_str {
                if let Some(val) = attr_str.value_at(index) {
                    result_buf.encode_field_tag(ANY_VALUE_STRING_VALUE, wire_types::LEN);
                    result_buf.encode_varint(val.len() as u64);
                    result_buf.extend_from_slice(val.as_bytes());
                }
            }
        }
        AttributeValueType::Bool => {
            if let Some(attr_bool) = &attr_arrays.attr_bool {
                if let Some(val) = attr_bool.value_at(index) {
                    result_buf.encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT);
                    result_buf.encode_varint(val as u64)
                }
            }
        }
        AttributeValueType::Int => {
            if let Some(attr_int) = &attr_arrays.attr_int {
                if let Some(val) = attr_int.value_at(index) {
                    result_buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT);
                    // TODO need to handle if it's a negative integer ...
                    result_buf.encode_varint(val as u64);
                }
            }
        }
        AttributeValueType::Double => {
            if let Some(attr_double) = &attr_arrays.attr_double {
                if let Some(val) = attr_double.value_at(index) {
                    result_buf.encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64);
                    result_buf.extend_from_slice(&val.to_le_bytes());
                }
            }
        }
        AttributeValueType::Bytes => {
            if let Some(attr_bytes) = &attr_arrays.attr_bytes {
                if let Some(val) = attr_bytes.value_at(index) {
                    result_buf.encode_field_tag(ANY_VALUE_BYES_VALUE, wire_types::LEN);
                    result_buf.encode_varint(val.len() as u64);
                    result_buf.extend_from_slice(val.as_ref());
                }
            }
        }

        AttributeValueType::Map | AttributeValueType::Slice => {
            // TODO need to to decode from cbor to proto directly
            // for now, doing something inefficient
            if let Some(any_val) = attr_arrays.value_at(index) {
                let any_val = any_val.unwrap();
                let mut bytes = vec![];
                any_val.encode(&mut bytes).unwrap();
                result_buf.extend_from_slice(&bytes);
            }
        }
        AttributeValueType::Empty => {
            // nothing to do
        }
    }
}
