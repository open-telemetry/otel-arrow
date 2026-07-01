// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! IPC baseline contender: the OTAP representation "as we have it today" -- the
//! interleaved Arrow IPC streams produced by [`Producer`] / consumed by
//! [`Consumer`], with the per-payload IPC streams optionally compressed.
//!
//! - write: [`Producer::produce_bar`] (applies transport-optimized encoding and
//!   serializes each payload's record batch to an Arrow IPC stream) then
//!   prost-encodes the resulting [`BatchArrowRecords`] to bytes.
//! - read: prost-decodes the [`BatchArrowRecords`], [`Consumer::consume_bar`]
//!   deserializes the IPC streams, [`from_record_messages`] reassembles the
//!   [`OtapArrowRecords`], and `decode_transport_optimized_ids` restores the
//!   logical (non-transport-optimized) batch so it matches the input exactly.

use otap_df_pdata::Consumer;
use otap_df_pdata::encode::producer::{Producer, ProducerOptions};
use otap_df_pdata::otap::{Logs, OtapArrowRecords, from_record_messages};
use otap_df_pdata::proto::opentelemetry::arrow::v1::BatchArrowRecords;
use prost::Message;

use super::{Codec, Compressor, StudyResult};

/// Contender that encodes OTAP logs as compressed interleaved Arrow IPC streams.
pub struct IpcCodec {
    /// Compression applied to each per-payload IPC stream.
    pub compressor: Compressor,
}

impl Codec for IpcCodec {
    fn name(&self) -> &'static str {
        "ipc"
    }

    fn write(&self, mut logs: OtapArrowRecords) -> StudyResult<Vec<u8>> {
        let mut producer = Producer::new_with_options(ProducerOptions {
            ipc_compression: self.compressor.ipc(),
        });
        let bar = producer.produce_bar(&mut logs)?;
        let mut buf = Vec::with_capacity(1024);
        bar.encode(&mut buf)?;
        Ok(buf)
    }

    fn read(&self, bytes: &[u8]) -> StudyResult<OtapArrowRecords> {
        let mut bar = BatchArrowRecords::decode(bytes)?;
        let mut consumer = Consumer::default();
        let messages = consumer.consume_bar(&mut bar)?;
        let mut logs = OtapArrowRecords::Logs(from_record_messages::<Logs>(messages)?);
        logs.decode_transport_optimized_ids()?;
        Ok(logs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parquet_study::Compressor;
    use crate::parquet_study::datagen::{LogsGenParams, gen_logs_otap};

    #[test]
    fn ipc_round_trip_preserves_structure() {
        let params = LogsGenParams {
            num_resources: 2,
            num_scopes: 2,
            num_logs: 3,
        };
        let (otap, _) = gen_logs_otap(&params);

        for compressor in Compressor::IPC {
            let codec = IpcCodec { compressor };
            let bytes = codec.write(otap.clone()).expect("write");
            let decoded = codec.read(&bytes).expect("read");
            // The IPC round-trip is lossless, though the decoded batch may
            // dictionary-encode some value columns, so compare structurally.
            crate::parquet_study::attrs::assert_logs_equivalent(
                &otap,
                &decoded,
                codec.name(),
                compressor.label(),
            );
        }
    }
}
