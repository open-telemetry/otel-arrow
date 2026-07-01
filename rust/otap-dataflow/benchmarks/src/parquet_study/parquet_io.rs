// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! In-memory Parquet read/write used by the flattened-Parquet contenders.
//!
//! The production `parquet_exporter` writes through `object_store`; for a
//! read/write cost microbenchmark we instead encode to and decode from an
//! in-memory `Vec<u8>` using the synchronous Arrow Parquet reader/writer.

use arrow::array::{Array, ArrayRef, RecordBatch};
use arrow::compute::{cast, concat_batches};
use arrow::datatypes::{DataType, Field, Schema};
use bytes::Bytes;
use parquet::arrow::ArrowWriter;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;

use std::sync::Arc;

use super::StudyResult;

/// Convert a flat OTAP batch into the form the Parquet writer can serialize,
/// transforming only the columns arrow-rs cannot handle and copying the rest.
///
/// Two column encodings do not survive Parquet in arrow-rs 58.3, so exactly
/// those are materialized here:
///
/// - `RunEndEncoded` columns (the run-end resource/scope attributes) cannot be
///   written; they are cast to their plain `List` value type.
/// - `Dictionary(_, FixedSizeBinary)` columns (`trace_id`, `span_id`) cannot be
///   read back, because the reader rebuilds the values through an offset buffer
///   and then asserts the `FixedSizeBinaryArray` has one buffer; they are cast to
///   plain `FixedSizeBinary`.
///
/// Every other column, including dictionary-encoded `Utf8`/`Int32` such as
/// `severity_text` and the nested `scope.name` and `body.str`, round-trips
/// through Parquet unchanged and is copied by reference. Parquet then applies its
/// own run-length and dictionary encoding on write.
pub fn to_parquet_ready(batch: &RecordBatch) -> StudyResult<RecordBatch> {
    let mut fields: Vec<Field> = Vec::with_capacity(batch.num_columns());
    let mut columns: Vec<ArrayRef> = Vec::with_capacity(batch.num_columns());
    for (field, column) in batch.schema().fields().iter().zip(batch.columns()) {
        match column.data_type() {
            DataType::RunEndEncoded(_, values) => {
                let decoded = cast(column, values.data_type())?;
                fields.push(Field::new(
                    field.name(),
                    decoded.data_type().clone(),
                    field.is_nullable(),
                ));
                columns.push(decoded);
            }
            DataType::Dictionary(_, value_type)
                if matches!(value_type.as_ref(), DataType::FixedSizeBinary(_)) =>
            {
                let decoded = cast(column, value_type)?;
                fields.push(Field::new(
                    field.name(),
                    decoded.data_type().clone(),
                    field.is_nullable(),
                ));
                columns.push(decoded);
            }
            _ => {
                fields.push(field.as_ref().clone());
                columns.push(column.clone());
            }
        }
    }
    Ok(RecordBatch::try_new(
        Arc::new(Schema::new(fields)),
        columns,
    )?)
}

/// Encode a single record batch as a Parquet file in memory.
pub fn write_parquet(batch: &RecordBatch, compression: Compression) -> StudyResult<Vec<u8>> {
    let props = WriterProperties::builder()
        .set_compression(compression)
        .build();
    let mut buf: Vec<u8> = Vec::with_capacity(4096);
    let mut writer = ArrowWriter::try_new(&mut buf, batch.schema(), Some(props))?;
    writer.write(batch)?;
    let _metadata = writer.close()?;
    Ok(buf)
}

/// Decode an in-memory Parquet file into a single record batch (concatenating
/// the reader's row-group batches).
pub fn read_parquet(bytes: &[u8]) -> StudyResult<RecordBatch> {
    let builder = ParquetRecordBatchReaderBuilder::try_new(Bytes::copy_from_slice(bytes))?;
    let schema = builder.schema().clone();
    let reader = builder.build()?;
    let mut batches: Vec<RecordBatch> = Vec::new();
    for batch in reader {
        batches.push(batch?);
    }
    match batches.len() {
        0 => Ok(RecordBatch::new_empty(schema)),
        1 => Ok(batches.into_iter().next().expect("len checked")),
        _ => Ok(concat_batches(&schema, &batches)?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{DictionaryArray, FixedSizeBinaryArray};
    use arrow::datatypes::UInt16Type;

    #[test]
    fn parquet_ready_materializes_fixed_size_binary() {
        // Dictionary(UInt16, FixedSizeBinary(4)) is the shape the OTAP encoder
        // produces for trace_id/span_id and that the Parquet reader mishandles.
        let values = FixedSizeBinaryArray::try_from_iter(
            vec![b"aaaa".to_vec(), b"bbbb".to_vec(), b"cccc".to_vec()].into_iter(),
        )
        .expect("values");
        let keys = arrow::array::UInt16Array::from(vec![0u16, 1, 2, 1, 0]);
        let dict = DictionaryArray::<UInt16Type>::try_new(keys, Arc::new(values)).expect("dict");
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "trace_id",
                dict.data_type().clone(),
                false,
            )])),
            vec![Arc::new(dict)],
        )
        .expect("batch");

        let decoded = to_parquet_ready(&batch).expect("parquet-ready");
        assert_eq!(
            decoded.schema().field(0).data_type(),
            &DataType::FixedSizeBinary(4),
            "dictionary should be materialized to plain fixed-size binary"
        );

        // The plain batch round-trips through Parquet, where the dictionary form
        // would trip the arrow-rs reader.
        let bytes = write_parquet(&decoded, Compression::UNCOMPRESSED).expect("write");
        let back = read_parquet(&bytes).expect("read");
        assert_eq!(back.num_rows(), 5);
    }
}
