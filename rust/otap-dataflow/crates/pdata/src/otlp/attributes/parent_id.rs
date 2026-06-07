// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::error::{Error, Result};
use crate::otlp::attributes::decoder::{
    Attrs16ParentIdDecoder, Attrs32ParentIdDecoder, AttrsParentIdDecoder,
};
use crate::schema::consts;
use arrow::array::{ArrowPrimitiveType, PrimitiveArray, RecordBatch};
use arrow::datatypes::{UInt16Type, UInt32Type};
use std::hash::Hash;
use std::ops::{Add, AddAssign};

#[allow(missing_docs)]
pub trait ParentId: Copy + Hash + Eq + Default + Add<Output = Self> + AddAssign
where
    <Self as ParentId>::ArrayType: ArrowPrimitiveType,
{
    #[allow(missing_docs)]
    type ArrayType;

    #[allow(missing_docs)]
    fn new_decoder() -> AttrsParentIdDecoder<Self>;

    /// Get the parent id columns from the record batch, downcast to the correct type
    fn get_parent_id_column(
        record_batch: &RecordBatch,
    ) -> Result<&PrimitiveArray<Self::ArrayType>> {
        let parent_id_arr = record_batch
            .column_by_name(consts::PARENT_ID)
            .ok_or_else(|| Error::ColumnNotFound {
                name: consts::PARENT_ID.into(),
            })?;
        let parent_id_arr = parent_id_arr
            .as_any()
            .downcast_ref::<PrimitiveArray<Self::ArrayType>>()
            .ok_or_else(|| Error::ColumnDataTypeMismatch {
                name: consts::PARENT_ID.into(),
                expect: Self::ArrayType::DATA_TYPE,
                actual: parent_id_arr.data_type().clone(),
            })?;

        Ok(parent_id_arr)
    }
}

impl ParentId for u16 {
    type ArrayType = UInt16Type;

    fn new_decoder() -> AttrsParentIdDecoder<Self> {
        Attrs16ParentIdDecoder::default()
    }
}

impl ParentId for u32 {
    type ArrayType = UInt32Type;

    fn new_decoder() -> AttrsParentIdDecoder<Self> {
        Attrs32ParentIdDecoder::default()
    }
}
