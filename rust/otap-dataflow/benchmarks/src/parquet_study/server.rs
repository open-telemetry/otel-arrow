// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Server-side CPU model for the OTAP-to-Parquet debate.
//!
//! In the target system the data ends up as flattened Parquet on the server
//! *either way*; the only question is where the OTAP -> Parquet conversion CPU
//! is spent.
//!
//! - **Option A -- client sends OTAP/IPC, server converts.** The server pays to
//!   decode the IPC, flatten to the single Parquet table, and encode Parquet.
//!   This is [`convert_ipc_to_parquet`].
//! - **Option B -- client sends precomputed Parquet.** The client already paid
//!   the conversion; the server only persists the bytes (essentially an I/O
//!   copy, ~0 CPU) or, if it must partition / index / validate, reparses the
//!   Parquet back into Arrow without going all the way to OTAP. That reparse is
//!   [`reparse_parquet`]; persisting is [`persist_bytes`].
//!
//! The server-side CPU *saved* by accepting client Parquet is therefore roughly
//! the whole of Option A minus whatever minimal handling Option B needs.

use arrow::array::RecordBatch;

use super::ipc::IpcCodec;
use super::{Codec, Compressor, StudyResult, parquet_io};

/// Option A: the server receives OTAP/IPC and produces flattened Parquet.
///
/// `parquet` selects the flattening layout (nested / map / wide) and the Parquet
/// compressor. The IPC decode auto-detects each stream's compression, so the
/// compressor used to construct the internal [`IpcCodec`] does not affect the
/// read.
pub fn convert_ipc_to_parquet(ipc_bytes: &[u8], parquet: &dyn Codec) -> StudyResult<Vec<u8>> {
    let otap = IpcCodec {
        compressor: Compressor::Zstd,
    }
    .read(ipc_bytes)?;
    parquet.write(otap)
}

/// Option B (server needs the data): reparse client-precomputed Parquet into an
/// Arrow record batch. This decodes Parquet but does not rebuild OTAP -- a
/// server that partitions or indexes the flat table works in Arrow directly.
pub fn reparse_parquet(parquet_bytes: &[u8]) -> StudyResult<RecordBatch> {
    parquet_io::read_parquet(parquet_bytes)
}

/// Option B (server only stores): persist the received bytes. Modeled as the
/// single copy the server makes handing the buffer to storage; real storage
/// cost is I/O, so this is a generous upper bound on the server's CPU.
#[must_use]
pub fn persist_bytes(parquet_bytes: &[u8]) -> usize {
    let sink: Vec<u8> = parquet_bytes.to_vec();
    sink.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parquet_study::attrs::assert_logs_equivalent;
    use crate::parquet_study::datagen::{LogsGenParams, gen_logs_otap};
    use crate::parquet_study::nested::NestedParquetCodec;
    use crate::parquet_study::{Codec, Compressor};

    #[test]
    fn server_convert_matches_direct_flatten() {
        let params = LogsGenParams {
            num_resources: 2,
            num_scopes: 2,
            num_logs: 4,
        };
        let (otap, _) = gen_logs_otap(&params);

        let ipc = IpcCodec {
            compressor: Compressor::Zstd,
        };
        let ipc_bytes = ipc.write(otap.clone()).expect("ipc write");

        let nested = NestedParquetCodec {
            compressor: Compressor::Zstd,
        };
        // Option A server output must round-trip back to an equivalent batch.
        let parquet_bytes = convert_ipc_to_parquet(&ipc_bytes, &nested).expect("convert");
        let decoded = nested.read(&parquet_bytes).expect("read back");
        assert_logs_equivalent(&otap, &decoded, "server-convert", "zstd");

        // Option B reparse yields the flat table without error.
        let flat = reparse_parquet(&parquet_bytes).expect("reparse");
        assert_eq!(flat.num_rows(), params.total_logs());
        assert!(persist_bytes(&parquet_bytes) > 0);
    }
}
