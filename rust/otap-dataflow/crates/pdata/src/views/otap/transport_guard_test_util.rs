// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::array::RecordBatch;
use arrow::datatypes::Schema;

use crate::error::Error;
use crate::otap::OtapArrowRecords;
use crate::otap::transform::transport_optimize::Encoding;
use crate::otap::transform::util::{access_column, replace_column, update_field_encoding_metadata};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

pub(crate) fn set_column_encoding(
    records: &mut OtapArrowRecords,
    payload_type: ArrowPayloadType,
    column: &str,
    encoding: Encoding,
) {
    let batch = records
        .get(payload_type)
        .unwrap_or_else(|| panic!("missing {payload_type:?} batch"))
        .clone();
    let encoded_batch = with_column_encoding(&batch, column, encoding);
    records.set(payload_type, encoded_batch).unwrap();
}

pub(crate) fn with_column_encoding(
    batch: &RecordBatch,
    column: &str,
    encoding: Encoding,
) -> RecordBatch {
    let schema = batch.schema();
    let mut columns = batch.columns().to_vec();
    let encoded_column = access_column(column, schema.as_ref(), &columns)
        .unwrap_or_else(|| panic!("missing `{column}` column"));

    let mut fields = schema.fields().to_vec();
    replace_column(
        column,
        Some(encoding),
        schema.as_ref(),
        &mut columns,
        encoded_column,
    );
    update_field_encoding_metadata(column, Some(encoding), &mut fields);

    RecordBatch::try_new(Arc::new(Schema::new(fields)), columns).unwrap()
}

pub(crate) fn assert_transport_error<T>(
    result: Result<T, Error>,
    expected_payload_type: ArrowPayloadType,
    expected_column: &str,
) {
    match result {
        Err(Error::TransportOptimizedIdsNotDecoded {
            payload_type,
            column,
        }) => {
            assert_eq!(payload_type, expected_payload_type);
            assert_eq!(column, expected_column);
        }
        Err(err) => panic!("unexpected error: {err:?}"),
        Ok(_) => panic!("expected transport-optimized ID guard error"),
    }
}
