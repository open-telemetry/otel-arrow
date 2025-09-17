// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::cbor;
use crate::arrays::{
    ByteArrayAccessor, Int64ArrayAccessor, MaybeDictArrayAccessor, NullableArrayAccessor,
    StringArrayAccessor, get_bool_array_opt, get_f64_array_opt, get_u8_array,
};
use crate::error;
use crate::otlp::attributes::parent_id::ParentId;
use crate::proto::opentelemetry::common::v1::any_value::Value;
use crate::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
use crate::schema::{consts, is_parent_id_plain_encoded};
use arrow::array::{ArrowPrimitiveType, PrimitiveArray, RecordBatch};
use num_enum::TryFromPrimitive;
use snafu::{OptionExt, ResultExt};
use std::collections::HashMap;

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

trait FindOrAppendValue<V> {
    /// Finds a value with given key and returns the mutable reference to that value.
    /// Appends a new value if not found and return mutable reference to that newly created value.
    fn find_or_append(&mut self, key: &str) -> &mut V;
}

impl FindOrAppendValue<Option<AnyValue>> for Vec<KeyValue> {
    fn find_or_append(&mut self, key: &str) -> &mut Option<AnyValue> {
        // It's a workaround for https://github.com/rust-lang/rust/issues/51545
        if let Some((idx, _)) = self.iter().enumerate().find(|(_, kv)| kv.key == key) {
            return &mut self[idx].value;
        }

        self.push(KeyValue {
            key: key.to_string(),
            value: None,
        });
        &mut self.last_mut().expect("vec is not empty").value
    }
}
