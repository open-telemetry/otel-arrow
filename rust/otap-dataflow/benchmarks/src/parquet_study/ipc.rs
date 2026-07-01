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

/// Pipeline sub-step: apply the OTAP transport-optimized encoding in place
/// (delta/dictionary encodings on id and value columns, parent-id remapping).
/// This is the first thing `Producer::produce_bar` does; measuring it alone lets
/// the benchmark separate it from the Arrow IPC serialization.
pub fn transport_encode(logs: &mut OtapArrowRecords) -> StudyResult<()> {
    logs.encode_transport_optimized()?;
    Ok(())
}

/// Pipeline sub-step: serialize a logs batch to wire bytes. This runs the whole
/// encode side (transport-optimized encoding, then Arrow IPC serialization with
/// compression, then prost-encoding the `BatchArrowRecords`). The IPC
/// serialization time alone is this minus [`transport_encode`].
pub fn encode_to_bytes(mut logs: OtapArrowRecords, compressor: Compressor) -> StudyResult<Vec<u8>> {
    let mut producer = Producer::new_with_options(ProducerOptions {
        ipc_compression: compressor.ipc(),
    });
    let bar = producer.produce_bar(&mut logs)?;
    let mut buf = Vec::with_capacity(1024);
    bar.encode(&mut buf)?;
    Ok(buf)
}

/// Pipeline sub-step: deserialize wire bytes into a logs batch that is still in
/// the transport-optimized encoding (prost-decode, then `Consumer::consume_bar`,
/// then `from_record_messages`). Does not run the transport decode.
pub fn deserialize(bytes: &[u8]) -> StudyResult<OtapArrowRecords> {
    let mut bar = BatchArrowRecords::decode(bytes)?;
    let mut consumer = Consumer::default();
    let messages = consumer.consume_bar(&mut bar)?;
    Ok(OtapArrowRecords::Logs(from_record_messages::<Logs>(
        messages,
    )?))
}

/// Pipeline sub-step: reverse the transport-optimized encoding in place, leaving
/// the logical OTAP logs batch.
pub fn transport_decode(logs: &mut OtapArrowRecords) -> StudyResult<()> {
    logs.decode_transport_optimized_ids()?;
    Ok(())
}

/// Serialize the same logs batch `count` times through a single long-lived
/// [`Producer`], returning the wire size of each batch.
///
/// This models OTAP streaming: the Arrow schema is written once into the stream
/// and dictionaries are delta-encoded, so `sizes[0]` is the cold size (schema
/// plus full dictionaries plus data) while `sizes[1..]` are the steady-state
/// sizes (data plus only new dictionary entries). Sending the identical batch
/// repeatedly is a best case for dictionary amortization; real telemetry with
/// varying values falls between the cold and steady-state sizes.
pub fn stream_batch_sizes(
    logs: &OtapArrowRecords,
    compressor: Compressor,
    count: usize,
) -> StudyResult<Vec<usize>> {
    let mut producer = Producer::new_with_options(ProducerOptions {
        ipc_compression: compressor.ipc(),
    });
    let mut sizes = Vec::with_capacity(count);
    for _ in 0..count {
        let mut batch = logs.clone();
        let bar = producer.produce_bar(&mut batch)?;
        let mut buf = Vec::with_capacity(1024);
        bar.encode(&mut buf)?;
        sizes.push(buf.len());
    }
    Ok(sizes)
}

impl Codec for IpcCodec {
    fn name(&self) -> &'static str {
        "ipc"
    }

    fn write(&self, logs: OtapArrowRecords) -> StudyResult<Vec<u8>> {
        encode_to_bytes(logs, self.compressor)
    }

    fn read(&self, bytes: &[u8]) -> StudyResult<OtapArrowRecords> {
        let mut logs = deserialize(bytes)?;
        transport_decode(&mut logs)?;
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

    #[test]
    fn streaming_amortizes_schema_and_dictionaries() {
        let params = LogsGenParams {
            num_resources: 1,
            num_scopes: 1,
            num_logs: 2000,
        };
        let (otap, _) = gen_logs_otap(&params);

        for compressor in Compressor::IPC {
            let sizes = stream_batch_sizes(&otap, compressor, 5).expect("stream sizes");
            // The steady-state batch (2nd onward) omits the schema header and
            // re-sends only new dictionary entries, so it is smaller than the
            // cold first batch.
            assert!(
                sizes[1] < sizes[0],
                "{compressor:?}: steady {} not smaller than cold {}",
                sizes[1],
                sizes[0]
            );
            // The drop is one-time: with identical batches, every steady-state
            // batch is the same size. Arrow IPC does not compress frames against
            // each other, so frame N is not smaller than frame 2 despite carrying
            // identical data.
            assert!(
                sizes[2..].iter().all(|&s| s == sizes[1]),
                "{compressor:?}: steady-state not flat: {:?}",
                sizes
            );
        }
    }

    /// Produce `count` batches through one long-lived producer with IPC
    /// compression disabled, concatenate the wire bytes, and compress the whole
    /// stream once with zstd at `level`. This models the size a stream-level
    /// (cross-batch) compressor could reach, whereas Arrow IPC compresses each
    /// batch independently and cannot exploit redundancy across batches.
    fn stream_whole_zstd(logs: &OtapArrowRecords, count: usize, level: i32) -> usize {
        let mut producer = Producer::new_with_options(ProducerOptions {
            ipc_compression: None,
        });
        let mut stream = Vec::new();
        for _ in 0..count {
            let mut batch = logs.clone();
            let bar = producer.produce_bar(&mut batch).expect("produce");
            bar.encode(&mut stream).expect("encode");
        }
        zstd::stream::encode_all(&stream[..], level)
            .expect("zstd")
            .len()
    }

    /// Arrow IPC compresses each batch independently, so it never exploits the
    /// large redundancy across the near-identical steady-state batches. This test
    /// shows two things about that unexploited redundancy:
    ///
    /// 1. A whole-stream compressor at default effort recovers almost none of it,
    ///    because each uncompressed batch (~2.4 MB at 10k logs) is larger than the
    ///    match window, so an extra near-duplicate batch still costs about as much
    ///    as one Arrow IPC batch.
    /// 2. A whole-stream compressor with a large window and long-distance matching
    ///    does find it, collapsing each extra near-duplicate batch to a tiny
    ///    fraction of an Arrow IPC batch.
    ///
    /// The identical-batch case here is a best case; real telemetry batches differ
    /// substantially, so the recoverable cross-batch redundancy is far smaller.
    #[test]
    fn cross_batch_redundancy_needs_large_window() {
        let params = LogsGenParams {
            num_resources: 1,
            num_scopes: 1,
            num_logs: 10_000,
        };
        let (otap, _) = gen_logs_otap(&params);

        let count = 8;
        // Arrow IPC steady-state batch size (per-batch independent compression).
        let warm = stream_batch_sizes(&otap, Compressor::Zstd, count).expect("sizes")[1];

        // Default effort: window smaller than one uncompressed batch, so an extra
        // near-duplicate batch costs about the same as an Arrow IPC batch.
        let low_1 = stream_whole_zstd(&otap, 1, 3);
        let low_n = stream_whole_zstd(&otap, count, 3);
        let low_marginal = (low_n - low_1) / (count - 1);
        assert!(
            low_marginal * 2 > warm,
            "default-effort cross-batch unexpectedly cheap: {low_marginal} vs warm {warm}"
        );

        // High effort: large window plus long-distance matching finds the
        // cross-batch redundancy and collapses each extra batch to near zero.
        let high_1 = stream_whole_zstd(&otap, 1, 19);
        let high_n = stream_whole_zstd(&otap, count, 19);
        let high_marginal = (high_n - high_1) / (count - 1);
        assert!(
            high_marginal * 10 < warm,
            "high-effort cross-batch did not collapse: {high_marginal} vs warm {warm}"
        );
    }
}
