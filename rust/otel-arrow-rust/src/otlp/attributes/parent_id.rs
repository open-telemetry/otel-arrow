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

use crate::arrays::NullableArrayAccessor;
use crate::otlp::attributes::decoder::{
    Attrs16ParentIdDecoder, Attrs32ParentIdDecoder, AttrsParentIdDecoder,
};
use arrow::array::{UInt16Array, UInt32Array};
use arrow::datatypes::DataType;
use num_enum::TryFromPrimitive;
use std::hash::Hash;
use std::ops::{Add, AddAssign};

pub trait ParentId: Copy + Hash + Eq + Default + Add<Output = Self> + AddAssign {
    type Array: NullableArrayAccessor<Native = Self> + 'static;

    fn arrow_data_type() -> DataType;

    fn new_decoder() -> AttrsParentIdDecoder<Self>;
}

impl ParentId for u16 {
    type Array = UInt16Array;

    fn arrow_data_type() -> DataType {
        DataType::UInt16
    }

    fn new_decoder() -> AttrsParentIdDecoder<Self> {
        Attrs16ParentIdDecoder::default()
    }
}

impl ParentId for u32 {
    type Array = UInt32Array;

    fn arrow_data_type() -> DataType {
        DataType::UInt32
    }

    fn new_decoder() -> AttrsParentIdDecoder<Self> {
        Attrs32ParentIdDecoder::default()
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum ParentIdEncoding {
    /// ParentIdNoEncoding stores the parent ID as is.
    ParentIdNoEncoding = 0,
    /// ParentIdDeltaEncoding stores the parent ID as a delta from the previous
    /// parent ID.
    ParentIdDeltaEncoding = 1,
    /// ParentIdDeltaGroupEncoding stores the parent ID as a delta from the
    /// previous parent ID in the same group. A group is defined by the
    /// combination Key and Value.
    ParentIdDeltaGroupEncoding = 2,
}
