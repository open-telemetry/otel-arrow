// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{ArrowPrimitiveType, PrimitiveArray, RecordBatch, StringArray};
use arrow::datatypes::{UInt16Type, UInt32Type};
use num_enum::TryFromPrimitive;
use snafu::OptionExt;

use crate::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor, get_required_array};
use crate::error::{self, Error, Result};
use crate::otlp::attributes::cbor::proto_encode_cbor_bytes;
use crate::otlp::common::{AnyValueArrays, ProtoBuffer};
use crate::proto::consts::field_num::common::{
    ANY_VALUE_BOOL_VALUE, ANY_VALUE_BYTES_VALUE, ANY_VALUE_DOUBLE_VALUE, ANY_VALUE_INT_VALUE,
    ANY_VALUE_STRING_VALUE, KEY_VALUE_KEY, KEY_VALUE_VALUE,
};
use crate::proto::consts::wire_types;
use crate::proto_encode_len_delimited_unknown_size;
use crate::schema::consts;

pub mod cbor;
pub mod decoder;
pub mod parent_id;

#[derive(Copy, Clone, Eq, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum AttributeValueType {
    Empty = 0,
    Str = 1,
    Int = 2,
    Double = 3,
    Bool = 4,
    Map = 5,
    Slice = 6,
    Bytes = 7,
}

pub(crate) type Attribute16Arrays<'a> = AttributeArrays<'a, UInt16Type>;
pub(crate) type Attribute32Arrays<'a> = AttributeArrays<'a, UInt32Type>;

pub(crate) struct AttributeArrays<'a, T: ArrowPrimitiveType> {
    pub parent_id: MaybeDictArrayAccessor<'a, PrimitiveArray<T>>,
    pub attr_key: MaybeDictArrayAccessor<'a, StringArray>,
    pub anyval_arrays: AnyValueArrays<'a>,
}

impl<'a, T> TryFrom<&'a RecordBatch> for AttributeArrays<'a, T>
where
    T: ArrowPrimitiveType,
{
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let parent_id = MaybeDictArrayAccessor::<PrimitiveArray<T>>::try_new(get_required_array(
            rb,
            consts::PARENT_ID,
        )?)?;

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

pub(crate) fn encode_key_value<T: ArrowPrimitiveType>(
    attr_arrays: &AttributeArrays<'_, T>,
    index: usize,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    if let Some(key) = attr_arrays.attr_key.str_at(index) {
        result_buf.encode_string(KEY_VALUE_KEY, key);
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
                    result_buf.encode_string(ANY_VALUE_STRING_VALUE, val);
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
                    result_buf.encode_bytes(ANY_VALUE_BYTES_VALUE, val);
                }
            }
        }
        AttributeValueType::Map | AttributeValueType::Slice => {
            if let Some(ser_bytes) = &attr_arrays.attr_ser {
                if let Some(val) = ser_bytes.slice_at(index) {
                    proto_encode_cbor_bytes(val, result_buf)?;
                }
            }
        }
        AttributeValueType::Empty => {
            // nothing to do
        }
    }

    Ok(())
}
