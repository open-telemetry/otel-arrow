// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{NullableArrayAccessor, StringArrayAccessor, StructColumnAccessor};
use crate::error;
use crate::error::Error;
use crate::proto::opentelemetry::trace::v1::Status;
use crate::schema::consts;
use arrow::array::{Array, Int32Array, StructArray};

pub(crate) struct SpanStatusArrays<'a> {
    status: &'a StructArray,
    code: Option<&'a Int32Array>,
    message: Option<StringArrayAccessor<'a>>,
}

impl NullableArrayAccessor for SpanStatusArrays<'_> {
    type Native = Status;

    fn value_at(&self, idx: usize) -> Option<Self::Native> {
        if !self.status.is_valid(idx) {
            return None;
        }

        let code = self
            .code
            .map(|arr| arr.value_at_or_default(idx))
            .unwrap_or_default();
        let message = self
            .message
            .as_ref()
            .map(|arr| arr.value_at_or_default(idx))
            .unwrap_or_default();

        Some(Status { code, message })
    }
}

impl<'a> TryFrom<&'a StructArray> for SpanStatusArrays<'a> {
    type Error = Error;

    fn try_from(status: &'a StructArray) -> error::Result<Self> {
        let column_accessor = StructColumnAccessor::new(status);
        Ok(Self {
            status,
            code: column_accessor.primitive_column_op(consts::STATUS_CODE)?,
            message: column_accessor.string_column_op(consts::STATUS_MESSAGE)?,
        })
    }
}
