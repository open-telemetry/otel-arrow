// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{BinaryArray, BooleanArray, Float64Array, Int64Array, RecordBatch, StringArray, UInt16Array, UInt8Array};
use snafu::OptionExt;


use crate::arrays::{get_bool_array_opt, get_f64_array_opt, get_u16_array, get_u8_array, MaybeDictArrayAccessor, NullableArrayAccessor};
use crate::decode::proto_bytes::{encode_field_tag, encode_fixed64, encode_float64, encode_varint};
use crate::encode_len_delimited_mystery_size;
use crate::error::{self, Error, Result};
use crate::otlp::attributes::store::AttributeValueType;
use crate::proto::consts::field_num::common::{ANY_VALUE_BOOL_VALUE, ANY_VALUE_BYES_VALUE, ANY_VALUE_DOUBLE_VALUE, ANY_VALUE_INT_VALUE, ANY_VALUE_STRING_VALUE, KEY_VALUE_KEY, KEY_VALUE_VALUE};
use crate::proto::consts::wire_types;
use crate::schema::consts;

pub mod cbor;
pub mod decoder;
pub mod parent_id;
pub mod store;

pub(crate) struct AttributeArrays<'a> {
    pub parent_id: &'a UInt16Array,
    pub attr_type: &'a UInt8Array,
    pub attr_key: MaybeDictArrayAccessor<'a, StringArray>,
    pub attr_str: Option<MaybeDictArrayAccessor<'a, StringArray>>,
    pub attr_int: Option<MaybeDictArrayAccessor<'a, Int64Array>>,
    pub attr_double: Option<&'a Float64Array>,
    pub attr_bool: Option<&'a BooleanArray>,
    pub attr_bytes: Option<MaybeDictArrayAccessor<'a, BinaryArray>>,
    pub attr_ser:  Option<MaybeDictArrayAccessor<'a, BinaryArray>>,
}

impl<'a> TryFrom<&'a RecordBatch> for AttributeArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let parent_id = get_u16_array(rb, consts::PARENT_ID)?;
        let attr_type = get_u8_array(rb, consts::ATTRIBUTE_TYPE)?;
        let key = rb
            .column_by_name(consts::ATTRIBUTE_KEY)
            .with_context(|| error::ColumnNotFoundSnafu { name: consts::ATTRIBUTE_KEY })?;
        let attr_key = MaybeDictArrayAccessor::<StringArray>::try_new(key)?;
        let attr_str = rb.column_by_name(consts::ATTRIBUTE_STR).map(MaybeDictArrayAccessor::<StringArray>::try_new).transpose()?;
        let attr_int = rb.column_by_name(consts::ATTRIBUTE_INT).map(MaybeDictArrayAccessor::<Int64Array>::try_new).transpose()?;
        let attr_double = get_f64_array_opt(rb, consts::ATTRIBUTE_DOUBLE)?;
        let attr_bool = get_bool_array_opt(rb, consts::ATTRIBUTE_BOOL)?;
        let attr_bytes = rb.column_by_name(consts::ATTRIBUTE_BYTES).map(MaybeDictArrayAccessor::<BinaryArray>::try_new).transpose()?;
        let attr_ser = rb.column_by_name(consts::ATTRIBUTE_BYTES).map(MaybeDictArrayAccessor::<BinaryArray>::try_new).transpose()?;

        Ok(Self {
            parent_id,
            attr_type,
            attr_key,
            attr_str,
            attr_int,
            attr_double,
            attr_bool,
            attr_bytes,
            attr_ser
        })

    }
}

pub(crate) struct AttributesIter<'a> {
    pub parent_id: u16,
    pub attr_arrays: &'a AttributeArrays<'a>,
    pub parent_id_sorted_indices: &'a Vec<usize>,
    pub pos: usize
}


impl Iterator for AttributesIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        // we've iterated to the end of the attributes array - no more attributes
        if self.pos >= self.parent_id_sorted_indices.len() {
            return None
        }
        
        let index = self.parent_id_sorted_indices[self.pos];
        
        // TODO there's a bug in here that we actually need to increment pos
        // until it's greater than or equal to parent_id? I guess in case there's
        // a parent ID w/ no attributes or something (e.g. attrs dropped).

        if Some(self.parent_id) == self.attr_arrays.parent_id.value_at(index) {
            self.pos += 1;
            Some(index)
        } else {
            None
        }
    }
}

pub(crate) fn encode_key_value(
    attr_arrays: &AttributeArrays<'_>,
    index: usize,
    result_buf: &mut Vec<u8>
) {
    if let Some(key) = attr_arrays.attr_key.value_at(index) {
        encode_field_tag(KEY_VALUE_KEY, wire_types::LEN, result_buf);
        encode_varint(key.len() as u64, result_buf);
        result_buf.extend_from_slice(key.as_bytes());
    }


    if let Some(value_type) = attr_arrays.attr_type.value_at(index) {
        // TODO nounwrap
        let value_type = AttributeValueType::try_from(value_type).unwrap();
        // TODO just guessing the max byte length for attributes is probably the biggest contributor to
        // wasting space when doing this encoding. if there's anywhere we want to optimize the mystery
        // size guess it's here
        let num_bytes = 5;
        encode_len_delimited_mystery_size!(
            KEY_VALUE_VALUE,
            num_bytes,
            encode_any_value(attr_arrays, index, value_type, result_buf),
            result_buf
        );
    }

}

fn encode_any_value(
    attr_arrays: &AttributeArrays<'_>,
    index: usize,
    value_type: AttributeValueType,
    result_buf: &mut Vec<u8>
) {
    match value_type {
        AttributeValueType::Str => {
            if let Some(attr_str) = &attr_arrays.attr_str {
                if let Some(val) = attr_str.value_at(index) {
                    encode_field_tag(ANY_VALUE_STRING_VALUE, wire_types::LEN, result_buf);
                    encode_varint(val.len() as u64, result_buf);
                    result_buf.extend_from_slice(val.as_bytes());
                }
            }
        },
        AttributeValueType::Bool => {
            if let Some(attr_bool) = &attr_arrays.attr_bool {
                if let Some(val) = attr_bool.value_at(index) {
                    encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT, result_buf);
                    encode_varint(val as u64, result_buf);
                }
            }
        },
        AttributeValueType::Int => {
            if let Some(attr_int) = &attr_arrays.attr_int {
                if let Some(val) = attr_int.value_at(index) {
                    encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT, result_buf);
                    // TODO need to handle if it's a negative integer ...
                    encode_varint(val as u64, result_buf);
                }
            }
        }
        AttributeValueType::Double => {
            if let Some(attr_double) = &attr_arrays.attr_double {
                if let Some(val) = attr_double.value_at(index) {
                    encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64, result_buf);
                    encode_float64(val, result_buf);
                }
            }
        }
        AttributeValueType::Bytes => {
            if let Some(attr_bytes) = &attr_arrays.attr_bytes {
                if let Some(val) = attr_bytes.value_at(index) {
                    encode_field_tag(ANY_VALUE_BYES_VALUE, wire_types::LEN, result_buf);
                    encode_varint(val.len() as u64, result_buf);
                    result_buf.extend_from_slice(val.as_ref());
                }
            }
        }
        
        AttributeValueType::Map | AttributeValueType::Slice => {
            // TODO need to decode from cbor and handle
            todo!()
        }
        AttributeValueType::Empty => {
            // nothing to do
        }
    }
}