// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use arrow::array::RecordBatch;

/// Wrapper for [RecordBatch].
pub struct RecordMessage {
    #[allow(unused)]
    pub(crate) batch_id: i64,
    #[allow(unused)]
    pub(crate) schema_id: String,
    pub(crate) payload_type: ArrowPayloadType,
    pub(crate) record: RecordBatch,
}
