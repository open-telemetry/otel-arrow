// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! In-memory Parquet read/write used by the flattened-Parquet contenders.
//!
//! The production `parquet_exporter` writes through `object_store`; for a
//! read/write cost microbenchmark we instead encode to and decode from an
//! in-memory `Vec<u8>` using the synchronous Arrow Parquet reader/writer.

use arrow::array::RecordBatch;
use arrow::compute::concat_batches;
use bytes::Bytes;
use parquet::arrow::ArrowWriter;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;

use super::StudyResult;

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
