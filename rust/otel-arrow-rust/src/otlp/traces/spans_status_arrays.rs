// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{
    Int32ArrayAccessor, NullableArrayAccessor, StringArrayAccessor, StructColumnAccessor,
};
use crate::error;
use crate::error::Error;
use crate::proto::opentelemetry::trace::v1::Status;
use crate::schema::consts;
use arrow::array::{Array, StructArray};

pub(crate) struct SpanStatusArrays<'a> {
    status: &'a StructArray,
    code: Option<Int32ArrayAccessor<'a>>,
    message: Option<StringArrayAccessor<'a>>,
}

impl NullableArrayAccessor for SpanStatusArrays<'_> {
    type Native = Status;

    fn value_at(&self, idx: usize) -> Option<Self::Native> {
        if !self.status.is_valid(idx) {
            return None;
        }
        let code = self.code.value_at_or_default(idx);
        let message = self.message.value_at_or_default(idx);
        Some(Status { code, message })
    }
}

impl<'a> TryFrom<&'a StructArray> for SpanStatusArrays<'a> {
    type Error = Error;

    fn try_from(status: &'a StructArray) -> error::Result<Self> {
        let column_accessor = StructColumnAccessor::new(status);
        Ok(Self {
            status,
            code: column_accessor.int32_column_op(consts::STATUS_CODE)?,
            message: column_accessor.string_column_op(consts::STATUS_MESSAGE)?,
        })
    }
}
