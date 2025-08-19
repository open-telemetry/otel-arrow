// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::error::{self, Result};
use crate::otlp::attributes::decoder::{
    Attrs16ParentIdDecoder, Attrs32ParentIdDecoder, AttrsParentIdDecoder,
};
use crate::schema::consts;
use arrow::array::{ArrowPrimitiveType, PrimitiveArray, RecordBatch};
use arrow::datatypes::{UInt16Type, UInt32Type};
use snafu::OptionExt;
use std::hash::Hash;
use std::ops::{Add, AddAssign};

pub trait ParentId: Copy + Hash + Eq + Default + Add<Output = Self> + AddAssign
where
    <Self as ParentId>::ArrayType: ArrowPrimitiveType,
{
    type ArrayType;

    fn new_decoder() -> AttrsParentIdDecoder<Self>;

    /// Get the parent id columns from the record batch, downcast to the correct type
    fn get_parent_id_column(
        record_batch: &RecordBatch,
    ) -> Result<&PrimitiveArray<Self::ArrayType>> {
        let parent_id_arr =
            record_batch
                .column_by_name(consts::PARENT_ID)
                .context(error::ColumnNotFoundSnafu {
                    name: consts::PARENT_ID,
                })?;
        let parent_id_arr = parent_id_arr
            .as_any()
            .downcast_ref::<PrimitiveArray<Self::ArrayType>>()
            .with_context(|| error::ColumnDataTypeMismatchSnafu {
                name: consts::PARENT_ID,
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
