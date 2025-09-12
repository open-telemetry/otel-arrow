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
    ANY_VALUE_ARRAY_VALUE, ANY_VALUE_BOOL_VALUE, ANY_VALUE_BYTES_VALUE, ANY_VALUE_DOUBLE_VALUE,
    ANY_VALUE_INT_VALUE, ANY_VALUE_KVLIST_VALUE, ANY_VALUE_STRING_VALUE, KEY_VALUE_KEY,
    KEY_VALUE_VALUE,
};
use crate::proto::consts::wire_types;
use crate::proto::opentelemetry::common::v1::any_value::Value;
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
) -> Result<()> {
    if let Some(key) = attr_arrays.attr_key.str_at(index) {
        result_buf.encode_field_tag(KEY_VALUE_KEY, wire_types::LEN);
        result_buf.encode_varint(key.len() as u64);
        result_buf.extend_from_slice(key.as_bytes());
    }

    if let Some(value_type) = attr_arrays.anyval_arrays.attr_type.value_at(index) {
        if let Ok(value_type) = AttributeValueType::try_from(value_type) {
            // TODO try to compute the length of the value here. This would probably be
            // straight forward for most types for all cases except map/slice, and even then
            // we could maybe guess order of magnitude by looking at the CBOR representation
            proto_encode_len_delimited_unknown_size!(
                KEY_VALUE_VALUE,
                encode_any_value(&attr_arrays.anyval_arrays, index, value_type, result_buf)?,
                result_buf
            );
        }
    }

    Ok(())
}

pub(crate) fn encode_any_value(
    attr_arrays: &AnyValueArrays<'_>,
    index: usize,
    value_type: AttributeValueType,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    match value_type {
        AttributeValueType::Str => {
            if let Some(attr_str) = &attr_arrays.attr_str {
                if let Some(val) = attr_str.str_at(index) {
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
                if let Some(val) = attr_bytes.slice_at(index) {
                    result_buf.encode_field_tag(ANY_VALUE_BYTES_VALUE, wire_types::LEN);
                    result_buf.encode_varint(val.len() as u64);
                    result_buf.extend_from_slice(val.as_ref());
                }
            }
        }

        // TODO for Map and Slice, we should be encoding directly from cbor to proto instead
        // of going through the intermediate prost struct like we're currently doing below:
        AttributeValueType::Map => {
            if let Some(any_val) = attr_arrays.value_at(index) {
                if let Some(Value::KvlistValue(kv_list)) = any_val?.value {
                    let mut bytes = vec![];
                    kv_list.encode(&mut bytes).expect("buffer has capacity");
                    result_buf.encode_field_tag(ANY_VALUE_KVLIST_VALUE, wire_types::LEN);
                    result_buf.encode_varint(bytes.len() as u64);
                    result_buf.extend_from_slice(&bytes);
                }
            }
        }
        AttributeValueType::Slice => {
            if let Some(any_val) = attr_arrays.value_at(index) {
                if let Some(Value::ArrayValue(list)) = any_val?.value {
                    let mut bytes = vec![];
                    list.encode(&mut bytes).expect("buffer has capacity");
                    result_buf.encode_field_tag(ANY_VALUE_ARRAY_VALUE, wire_types::LEN);
                    result_buf.encode_varint(bytes.len() as u64);
                    result_buf.extend_from_slice(&bytes);
                }
            }
        }
        AttributeValueType::Empty => {
            // nothing to do
        }
    }

    Ok(())
}
