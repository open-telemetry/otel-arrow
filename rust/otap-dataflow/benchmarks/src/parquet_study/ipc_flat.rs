// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! In-memory Arrow IPC read/write for a single flat record batch.
//!
//! This is the transport counterpart to [`parquet_io`](super::parquet_io). It
//! serializes an already-flattened OTAP-flat record batch as a compressed Arrow
//! IPC stream, which is how an "OTAP-flat" wire format would move between two
//! services. Unlike Parquet, Arrow IPC can serialize `RunEndEncoded` and
//! dictionary columns, so the compact resource/scope layouts survive on the
//! wire.
//!
//! Note this is plain Arrow IPC of one batch, not the OTAP [`Producer`] path,
//! which applies transport-optimized delta and dictionary encoding to the four
//! normalized batches. The [`ipc`](super::ipc) contender measures that OTAP
//! standard path; this measures the flat alternative.
//!
//! [`Producer`]: otap_df_pdata::encode::producer::Producer

use arrow::array::RecordBatch;
use arrow::compute::concat_batches;
use arrow_ipc::reader::StreamReader;
use arrow_ipc::writer::{IpcWriteOptions, StreamWriter};

use super::{Compressor, StudyResult};

/// Serialize a single record batch as a compressed Arrow IPC stream in memory.
pub fn write_ipc(batch: &RecordBatch, compressor: Compressor) -> StudyResult<Vec<u8>> {
    let options = IpcWriteOptions::default().try_with_compression(compressor.ipc())?;
    let mut buf: Vec<u8> = Vec::with_capacity(4096);
    {
        let mut writer = StreamWriter::try_new_with_options(&mut buf, &batch.schema(), options)?;
        writer.write(batch)?;
        writer.finish()?;
    }
    Ok(buf)
}

/// Decode an in-memory Arrow IPC stream into a single record batch.
pub fn read_ipc(bytes: &[u8]) -> StudyResult<RecordBatch> {
    let reader = StreamReader::try_new(bytes, None)?;
    let schema = reader.schema();
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
    use crate::parquet_study::datagen::{LogsGenParams, gen_logs_otap};
    use crate::parquet_study::otap_flat::{Layout, flatten};

    #[test]
    fn ipc_flat_round_trips_every_layout() {
        let params = LogsGenParams {
            num_resources: 3,
            num_scopes: 2,
            num_logs: 6,
        };
        let (otap, _) = gen_logs_otap(&params);

        for layout in [
            Layout::Materialized,
            Layout::RunEndEncoded,
            Layout::Dictionary,
        ] {
            let flat = flatten(&otap, layout).expect("flatten");
            for compressor in [Compressor::Zstd, Compressor::Lz4, Compressor::None] {
                let bytes = write_ipc(&flat, compressor).expect("write ipc");
                assert!(!bytes.is_empty());
                let back = read_ipc(&bytes).expect("read ipc");
                assert_eq!(back.num_rows(), flat.num_rows(), "{}", layout.name());
                assert_eq!(back.schema(), flat.schema(), "{}", layout.name());
            }
        }
    }
}
