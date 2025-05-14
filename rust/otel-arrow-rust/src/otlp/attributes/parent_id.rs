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

use crate::otlp::attributes::decoder::{
    Attrs16ParentIdDecoder, Attrs32ParentIdDecoder, AttrsParentIdDecoder,
};
use arrow::datatypes::{UInt16Type, UInt32Type};
use std::hash::Hash;
use std::ops::{Add, AddAssign};

pub trait ParentId: Copy + Hash + Eq + Default + Add<Output = Self> + AddAssign {
    type ArrayType;

    fn new_decoder() -> AttrsParentIdDecoder<Self>;
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
