// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{RecordBatch, StringArray, UInt16Array, UInt8Array};
use snafu::OptionExt;


use crate::arrays::{get_u16_array, get_u8_array, MaybeDictArrayAccessor, NullableArrayAccessor};
use crate::decode::proto_bytes::{encode_field_tag, encode_varint};
use crate::encode_len_delimited_mystery_size;
use crate::error::{self, Error, Result};
use crate::otlp::attributes::store::AttributeValueType;
use crate::proto::consts::field_num::common::{KEY_VALUE_KEY, KEY_VALUE_VALUE};
use crate::proto::consts::wire_types;
use crate::schema::consts;

pub mod cbor;
pub mod decoder;
pub mod parent_id;
pub mod store;

pub(crate) struct AttributeArrays<'a> {
    pub parent_id: &'a UInt16Array,
    pub attr_type: &'a UInt8Array,
    pub key: MaybeDictArrayAccessor<'a, StringArray>,
    // TODO fill in the rest
}

impl<'a> TryFrom<&'a RecordBatch> for AttributeArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let parent_id = get_u16_array(rb, consts::PARENT_ID)?;
        let attr_type = get_u8_array(rb, consts::ATTRIBUTE_TYPE)?;
        let key = rb
            .column_by_name(consts::ATTRIBUTE_KEY)
            .with_context(|| error::ColumnNotFoundSnafu { name: consts::ATTRIBUTE_KEY })?;
        let key = MaybeDictArrayAccessor::<StringArray>::try_new(key)?;

        Ok(Self {
            parent_id,
            attr_type,
            key
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

pub fn encode_key_value(
    attr_arrays: &AttributeArrays<'_>,
    index: usize,
    result_buf: &mut Vec<u8>
) {
    if let Some(key) = attr_arrays.key.value_at(index) {
        encode_field_tag(KEY_VALUE_KEY, wire_types::LEN, result_buf);
        encode_varint(key.len() as u64, result_buf);
        result_buf.extend_from_slice(key.as_bytes());
    }

    let num_bytes = 5;
    encode_len_delimited_mystery_size!(
        KEY_VALUE_VALUE,
        num_bytes,
        encode_any_value(attr_arrays, index, result_buf),
        result_buf
    );

}

fn encode_any_value(
    attr_arrays: &AttributeArrays<'_>,
    index: usize,
    result_buf: &mut Vec<u8>
) {
    if let Some(value_type) = attr_arrays.attr_type.value_at(index) {
        // TODO nounwrap
        let value_type = AttributeValueType::try_from(value_type).unwrap();
        match value_type {
            AttributeValueType::Str => {
                // TODO
            },
            _ => {
                todo!()
            }
        }
    }
}