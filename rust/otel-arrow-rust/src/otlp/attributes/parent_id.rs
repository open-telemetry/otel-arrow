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

use crate::arrays::{UInt16ArrayAccessor, UInt32ArrayAccessor};
use crate::error;
use arrow::datatypes::{UInt16Type, UInt32Type};
use std::hash::Hash;
use std::ops::{Add, AddAssign};

use super::decoder::{Attrs16ParentIdDecoder, Attrs32ParentIdDecoder, AttrsParentIdDecoder};
use arrow::array::ArrayRef;

pub trait TryNew<'a, T> {
    fn try_new(ar: &'a ArrayRef) -> error::Result<T>;
}

// Two questions stand out:
// 1. Should we document that we're using u16/u32 with overflow.
// 2. Why dictionary encode parent ID. What is the cardinality, is it not dense?
pub trait ParentId<'a>: Copy + Hash + Eq + Default + Add<Output = Self> + AddAssign {
    type ArrayType;
    type PrimitiveType;
    type AccessorType: TryNew<'a, Self::ArrayType>;

    fn new_decoder() -> AttrsParentIdDecoder<'a, Self>;
}

impl<'a> ParentId<'a> for u16 {
    type ArrayType = UInt16Type;
    type PrimitiveType = u16;
    type AccessorType = UInt16ArrayAccessor<'a>;

    fn new_decoder() -> AttrsParentIdDecoder<'a, Self> {
        Attrs16ParentIdDecoder::default()
    }
}

impl<'a> ParentId<'a> for u32 {
    type ArrayType = UInt32Type;
    type PrimitiveType = u32;
    type AccessorType = UInt32ArrayAccessor<'a>;

    fn new_decoder() -> AttrsParentIdDecoder<'a, Self> {
        Attrs32ParentIdDecoder::default()
    }
}
