// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{RecordBatch, StringArray, UInt16Array, UInt8Array};
use snafu::OptionExt;


use crate::arrays::{get_u16_array, get_u8_array, MaybeDictArrayAccessor};
use crate::error::{self, Error, Result};
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
        let parent_id = get_u16_array(rb, consts::ID)?;
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

struct AttributesIter<'a> {
    parent_id: u16,
    attr_arrays: &'a AttributeArrays<'a>,
    parent_id_sorted_indices: &'a Vec<usize>,
    parent_id_sorted_index: usize
}

impl Iterator for AttributesIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}