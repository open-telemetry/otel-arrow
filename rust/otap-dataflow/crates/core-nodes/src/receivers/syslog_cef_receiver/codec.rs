// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Independently decodable batches of original syslog and CEF messages.
//!
//! `syslog-batch` uses this big-endian framing:
//!
//! ```text
//! "SLG1" | item_count:u32 | repeated {
//!     observed_time_unix_nano:i64 | run_item_count:u32 | repeated {
//!         message_length:u32 | received_local_year:i32 | message_bytes
//!     }
//! }
//! ```
//!
//! Observed timestamps are captured when a receiver batch is sealed. Each
//! message also retains the local year at receipt, so deferred RFC 3164 decoding
//! does not infer a different year after New Year. Native batching copies these
//! frames and coalesces adjacent observed-time runs without parsing syslog.

use std::num::NonZeroU64;
use std::sync::LazyLock;

use bytes::{BufMut, Bytes, BytesMut};
use chrono::{Datelike, Local};
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata_codec::{
    BatchSettings, BatchSizer, BatchingSupport, CodecBatcherRegistration, CodecBatches, CodecError,
    CodecMetadata, CodecOperation, CodecRegistration, CodecRegistry, EncodedPdataBatch,
    PdataBatcher, PdataDecoder, PdataEncoding, RegistryError, ResolvedCodec, register_pdata_codec,
};

use super::arrow_records_encoder::ArrowRecordsBuilder;
use super::parser;

/// Stable identity of the versioned syslog/CEF batch representation.
pub const SYSLOG_CEF_ENCODING: PdataEncoding = PdataEncoding::new("syslog-batch");

const MAGIC: &[u8; 4] = b"SLG1";
const BATCH_HEADER_LEN: usize = 8;
const RUN_HEADER_LEN: usize = 12;
const FRAME_HEADER_LEN: usize = 8;
const MAX_NATIVE_LOGS: usize = 65_535;
const MAX_PREDICTED_BATCH_CAPACITY: usize = 32 * 1024;

const SYSLOG_BATCH_SETTINGS: BatchSettings = BatchSettings {
    min_size: Some(8192),
    max_size: NonZeroU64::new(MAX_NATIVE_LOGS as u64),
    sizer: BatchSizer::Items,
    max_split_fragments: None,
    max_split_overhead_bytes: None,
    max_split_fragments_per_flush: None,
};

/// A malformed syslog batch frame.
#[derive(Debug, thiserror::Error)]
#[error("{reason}")]
pub struct SyslogFramingError {
    reason: String,
}

fn framing_error(reason: impl Into<String>) -> SyslogFramingError {
    SyslogFramingError {
        reason: reason.into(),
    }
}

fn operation_error(
    operation: CodecOperation,
    source: impl std::error::Error + Send + Sync + 'static,
) -> CodecError {
    CodecError::operation(&SYSLOG_CEF_ENCODING, operation, source)
}

fn predicted_batch_capacity(max_items: usize, message_len: usize) -> usize {
    BATCH_HEADER_LEN
        .saturating_add(RUN_HEADER_LEN)
        .saturating_add(
            FRAME_HEADER_LEN
                .saturating_add(message_len)
                .saturating_mul(max_items.max(1)),
        )
        .min(MAX_PREDICTED_BATCH_CAPACITY)
}

/// Accumulates framed syslog messages without parsing them.
pub(crate) struct SyslogBatchBuilder {
    bytes: BytesMut,
    item_count: u32,
    max_items_hint: usize,
}

/// A batch sealed by the syslog codec, retaining the count established while framing.
///
/// Only the codec builder can construct this value. Its bytes and count cannot
/// be replaced independently. Cloning shares the bytes and preserves the count.
#[derive(Clone)]
pub struct SyslogBatch {
    bytes: Bytes,
    item_count: usize,
}

impl EncodedPdataBatch for SyslogBatch {
    const ENCODING: PdataEncoding = SYSLOG_CEF_ENCODING;

    fn signal_type(&self) -> SignalType {
        SignalType::Logs
    }

    fn into_parts(self) -> (Bytes, usize) {
        (self.bytes, self.item_count)
    }
}

impl Default for SyslogBatchBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SyslogBatchBuilder {
    /// Creates an empty batch with space for its header.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::with_max_items(1)
    }

    /// Creates an empty batch using a bounded first-message capacity estimate.
    #[must_use]
    pub(crate) fn with_max_items(max_items_hint: usize) -> Self {
        let mut bytes = BytesMut::with_capacity(BATCH_HEADER_LEN + RUN_HEADER_LEN);
        bytes.extend_from_slice(MAGIC);
        bytes.put_u32(0);
        bytes.put_i64(0);
        bytes.put_u32(0);
        Self {
            bytes,
            item_count: 0,
            max_items_hint: max_items_hint.max(1),
        }
    }

    /// Number of framed messages currently buffered.
    #[must_use]
    pub(crate) const fn len(&self) -> u32 {
        self.item_count
    }

    /// Returns true when no messages have been appended.
    #[must_use]
    pub(crate) const fn is_empty(&self) -> bool {
        self.item_count == 0
    }

    /// Copies a socket-backed message and captures its local receipt year.
    pub(crate) fn append(&mut self, message: &[u8]) -> Result<(), SyslogFramingError> {
        self.append_with_year(message, Local::now().year())
    }

    // Keep the clock at the admission boundary; decoding must never infer the
    // year from its own clock or the later batch-seal timestamp. A per-message
    // year also handles receiver batches spanning midnight on New Year.
    fn append_with_year(
        &mut self,
        message: &[u8],
        received_local_year: i32,
    ) -> Result<(), SyslogFramingError> {
        let message_len = u32::try_from(message.len())
            .map_err(|_| framing_error("syslog message length exceeds u32"))?;
        if self.is_empty() {
            let predicted_capacity = predicted_batch_capacity(self.max_items_hint, message.len());
            self.bytes
                .reserve(predicted_capacity.saturating_sub(self.bytes.len()));
        }
        self.item_count = self
            .item_count
            .checked_add(1)
            .ok_or_else(|| framing_error("syslog batch item count exceeds u32"))?;
        self.bytes.put_u32(message_len);
        self.bytes.put_i32(received_local_year);
        self.bytes.extend_from_slice(message);
        Ok(())
    }

    /// Replaces this builder while retaining its configured item-count hint.
    #[must_use]
    pub(crate) fn take(&mut self) -> Self {
        let replacement = Self::with_max_items(self.max_items_hint);
        std::mem::replace(self, replacement)
    }

    /// Discards buffered messages and releases their allocation.
    pub(crate) fn discard(&mut self) {
        *self = Self::with_max_items(self.max_items_hint);
    }

    #[cfg(test)]
    pub(crate) fn capacity(&self) -> usize {
        self.bytes.capacity()
    }

    /// Seals a non-empty batch with one timestamp run for the flush.
    pub(crate) fn finish(mut self, observed_time: i64) -> Result<SyslogBatch, SyslogFramingError> {
        if self.is_empty() {
            return Err(framing_error("cannot seal an empty syslog batch"));
        }
        self.bytes[4..BATCH_HEADER_LEN].copy_from_slice(&self.item_count.to_be_bytes());
        self.bytes[BATCH_HEADER_LEN..BATCH_HEADER_LEN + 8]
            .copy_from_slice(&observed_time.to_be_bytes());
        self.bytes[BATCH_HEADER_LEN + 8..BATCH_HEADER_LEN + RUN_HEADER_LEN]
            .copy_from_slice(&self.item_count.to_be_bytes());
        let item_count = self.item_count as usize;
        Ok(SyslogBatch {
            bytes: self.bytes.freeze(),
            item_count,
        })
    }
}

#[derive(Clone, Copy)]
struct Frame<'a> {
    observed_time: i64,
    received_local_year: i32,
    message: &'a [u8],
    encoded: &'a [u8],
}

struct FrameIter<'a> {
    bytes: &'a [u8],
    cursor: usize,
    remaining: usize,
    run_remaining: usize,
    observed_time: i64,
    finished: bool,
}

impl<'a> Iterator for FrameIter<'a> {
    type Item = Result<Frame<'a>, SyslogFramingError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        if self.remaining == 0 {
            self.finished = true;
            return (self.cursor != self.bytes.len())
                .then(|| Err(framing_error("trailing bytes after syslog batch frames")));
        }
        if self.run_remaining == 0 {
            let run_end = match self.cursor.checked_add(RUN_HEADER_LEN) {
                Some(run_end) if run_end <= self.bytes.len() => run_end,
                Some(_) => {
                    self.finished = true;
                    return Some(Err(framing_error("truncated syslog timestamp run")));
                }
                None => {
                    self.finished = true;
                    return Some(Err(framing_error("syslog timestamp run overflow")));
                }
            };
            self.observed_time = read_i64(&self.bytes[self.cursor..self.cursor + 8]);
            self.run_remaining = read_u32(&self.bytes[self.cursor + 8..run_end]) as usize;
            self.cursor = run_end;
            if self.run_remaining == 0 || self.run_remaining > self.remaining {
                self.finished = true;
                return Some(Err(framing_error(
                    "invalid syslog timestamp run item count",
                )));
            }
        }
        let start = self.cursor;
        let header_end = match start.checked_add(FRAME_HEADER_LEN) {
            Some(header_end) if header_end <= self.bytes.len() => header_end,
            Some(_) => {
                self.finished = true;
                return Some(Err(framing_error("truncated syslog frame header")));
            }
            None => {
                self.finished = true;
                return Some(Err(framing_error("syslog frame header overflow")));
            }
        };
        let message_len = read_u32(&self.bytes[start..start + 4]) as usize;
        let year_bytes = &self.bytes[start + 4..header_end];
        let received_local_year =
            i32::from_be_bytes([year_bytes[0], year_bytes[1], year_bytes[2], year_bytes[3]]);
        let message_start = header_end;
        let end = match message_start.checked_add(message_len) {
            Some(end) if end <= self.bytes.len() => end,
            Some(_) => {
                self.finished = true;
                return Some(Err(framing_error("truncated syslog frame body")));
            }
            None => {
                self.finished = true;
                return Some(Err(framing_error("syslog frame length overflow")));
            }
        };
        self.cursor = end;
        self.remaining -= 1;
        self.run_remaining -= 1;
        Some(Ok(Frame {
            observed_time: self.observed_time,
            received_local_year,
            message: &self.bytes[message_start..end],
            encoded: &self.bytes[start..end],
        }))
    }
}

fn validated_frames(bytes: &[u8]) -> Result<(usize, FrameIter<'_>), SyslogFramingError> {
    if bytes.len() < BATCH_HEADER_LEN {
        return Err(framing_error("truncated syslog batch header"));
    }
    if &bytes[..4] != MAGIC {
        return Err(framing_error("invalid syslog batch magic"));
    }
    let count = read_u32(&bytes[4..BATCH_HEADER_LEN]) as usize;
    if count == 0 {
        return Err(framing_error("empty syslog batches are invalid"));
    }
    let minimum_len = count
        .checked_mul(FRAME_HEADER_LEN)
        .and_then(|len| len.checked_add(BATCH_HEADER_LEN + RUN_HEADER_LEN))
        .ok_or_else(|| framing_error("syslog batch length overflow"))?;
    if minimum_len > bytes.len() {
        return Err(framing_error("syslog batch frame count exceeds its length"));
    }

    Ok((
        count,
        FrameIter {
            bytes,
            cursor: BATCH_HEADER_LEN,
            remaining: count,
            run_remaining: 0,
            observed_time: 0,
            finished: false,
        },
    ))
}

fn validated_item_count(bytes: &[u8]) -> Result<usize, SyslogFramingError> {
    let (count, mut frames) = validated_frames(bytes)?;
    frames.try_for_each(|frame| frame.map(|_| ()))?;
    Ok(count)
}

const fn read_u32(bytes: &[u8]) -> u32 {
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

const fn read_i64(bytes: &[u8]) -> i64 {
    i64::from_be_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

#[derive(Default)]
struct SyslogDecoder;

impl PdataDecoder for SyslogDecoder {
    fn decode(
        &mut self,
        signal: SignalType,
        bytes: &Bytes,
    ) -> Result<OtapArrowRecords, CodecError> {
        if signal != SignalType::Logs {
            return Err(CodecError::Unsupported {
                encoding: SYSLOG_CEF_ENCODING,
                operation: CodecOperation::Decode,
                signal,
            });
        }
        let (count, frames) = validated_frames(bytes)
            .map_err(|error| operation_error(CodecOperation::Decode, error))?;
        if count > MAX_NATIVE_LOGS {
            return Err(operation_error(
                CodecOperation::Decode,
                framing_error("syslog batch exceeds native log ID capacity"),
            ));
        }
        let mut builder = ArrowRecordsBuilder::new();
        for frame in frames {
            let frame = frame.map_err(|error| operation_error(CodecOperation::Decode, error))?;
            let parsed = parser::parse(frame.message)
                .map_err(|error| framing_error(format!("invalid syslog message: {error:?}")))
                .map_err(|error| operation_error(CodecOperation::Decode, error))?;
            builder.append_syslog_with_receive_context(
                parsed,
                frame.observed_time,
                frame.received_local_year,
            );
        }
        builder
            .build()
            .map_err(|error| operation_error(CodecOperation::Decode, error))
    }
}

#[derive(Default)]
struct SyslogBatcher;

impl PdataBatcher for SyslogBatcher {
    fn batch(
        &mut self,
        signal: SignalType,
        settings: &BatchSettings,
        inputs: Vec<Bytes>,
    ) -> Result<CodecBatches, CodecError> {
        if signal != SignalType::Logs {
            return Err(CodecError::Unsupported {
                encoding: SYSLOG_CEF_ENCODING,
                operation: CodecOperation::Batch,
                signal,
            });
        }
        if settings.sizer != BatchSizer::Items {
            return Err(CodecError::invalid_batch(
                "syslog native batching supports items only",
            ));
        }
        let max_items = settings.max_size.map_or(MAX_NATIVE_LOGS, |size| {
            // Oversized limits still cap at the native log batch limit on 32-bit targets.
            usize::try_from(size.get())
                .unwrap_or(MAX_NATIVE_LOGS)
                .min(MAX_NATIVE_LOGS)
        });
        let mut outputs = Vec::new();
        let mut output = RawBatchWriter::new();
        let mut first_input = 0;
        let mut last_input = 0;
        for (input_index, input) in inputs.into_iter().enumerate() {
            let (_, frames) = validated_frames(&input)
                .map_err(|error| operation_error(CodecOperation::Batch, error))?;
            for frame in frames {
                let frame = frame.map_err(|error| operation_error(CodecOperation::Batch, error))?;
                if output.len() == max_items {
                    outputs.push(
                        output
                            .finish()
                            .map(|(bytes, _)| (bytes, first_input..=last_input))
                            .map_err(|error| operation_error(CodecOperation::Batch, error))?,
                    );
                    output = RawBatchWriter::new();
                }
                if output.is_empty() {
                    first_input = input_index;
                }
                last_input = input_index;
                output
                    .append(frame)
                    .map_err(|error| operation_error(CodecOperation::Batch, error))?;
            }
        }
        if !output.is_empty() {
            outputs.push(
                output
                    .finish()
                    .map(|(bytes, _)| (bytes, first_input..=last_input))
                    .map_err(|error| operation_error(CodecOperation::Batch, error))?,
            );
        }
        Ok(CodecBatches {
            batches: outputs,
            // Framing rejects empty inputs; every accepted input contributes to an output.
            completed_inputs: Vec::new(),
            budget_fallbacks: 0,
        })
    }
}

struct RawBatchWriter {
    bytes: BytesMut,
    count: u32,
    run_count_offset: usize,
    run_count: u32,
    observed_time: Option<i64>,
}

impl RawBatchWriter {
    fn new() -> Self {
        let mut bytes = BytesMut::with_capacity(BATCH_HEADER_LEN);
        bytes.extend_from_slice(MAGIC);
        bytes.put_u32(0);
        Self {
            bytes,
            count: 0,
            run_count_offset: 0,
            run_count: 0,
            observed_time: None,
        }
    }

    const fn len(&self) -> usize {
        self.count as usize
    }

    const fn is_empty(&self) -> bool {
        self.count == 0
    }

    fn append(&mut self, frame: Frame<'_>) -> Result<(), SyslogFramingError> {
        if self.observed_time != Some(frame.observed_time) {
            self.bytes.put_i64(frame.observed_time);
            self.run_count_offset = self.bytes.len();
            self.bytes.put_u32(0);
            self.run_count = 0;
            self.observed_time = Some(frame.observed_time);
        }
        self.count = self
            .count
            .checked_add(1)
            .ok_or_else(|| framing_error("syslog batch item count exceeds u32"))?;
        self.run_count = self
            .run_count
            .checked_add(1)
            .ok_or_else(|| framing_error("syslog timestamp run item count exceeds u32"))?;
        self.bytes[self.run_count_offset..self.run_count_offset + 4]
            .copy_from_slice(&self.run_count.to_be_bytes());
        self.bytes.extend_from_slice(frame.encoded);
        Ok(())
    }

    fn finish(mut self) -> Result<(Bytes, usize), SyslogFramingError> {
        if self.is_empty() {
            return Err(framing_error("cannot finish an empty syslog batch"));
        }
        self.bytes[4..BATCH_HEADER_LEN].copy_from_slice(&self.count.to_be_bytes());
        let count = self.count as usize;
        Ok((self.bytes.freeze(), count))
    }
}

fn count_items(signal: SignalType, bytes: &[u8]) -> Option<usize> {
    (signal == SignalType::Logs)
        .then(|| validated_item_count(bytes).ok())
        .flatten()
}

static SYSLOG_CEF_METADATA: CodecMetadata =
    CodecMetadata::new(SYSLOG_CEF_ENCODING, &[SignalType::Logs]).with_format_version("1");

static SYSLOG_BATCHING_SUPPORT: BatchingSupport = BatchingSupport {
    sizers: &[BatchSizer::Items],
    default_settings: SYSLOG_BATCH_SETTINGS,
};

register_pdata_codec!(
    SYSLOG_CEF_CODEC,
    CodecRegistration::new(&SYSLOG_CEF_METADATA)
        .with_decoder(|_| Box::<SyslogDecoder>::default())
        .with_batcher(CodecBatcherRegistration::new(
            &SYSLOG_BATCHING_SUPPORT,
            || Box::<SyslogBatcher>::default(),
        ))
        .with_item_counter(count_items),
);

/// Resolves the linked syslog codec from the validated production registry.
pub fn resolve_syslog() -> Result<ResolvedCodec, RegistryError> {
    static RESOLVED: LazyLock<Result<ResolvedCodec, RegistryError>> =
        LazyLock::new(|| CodecRegistry::global()?.resolve(&SYSLOG_CEF_ENCODING));
    RESOLVED.as_ref().copied().map_err(Clone::clone)
}

/// Helpers exposing the receiver codec path to the workspace benchmarks.
#[cfg(feature = "bench")]
pub mod bench_support {
    use chrono::Utc;
    use otel_arrow_dfe_pdata::OtapArrowRecords;
    use otel_arrow_dfe_pdata_codec::{CodecService, OtapPayload, ResolvedCodec};

    use super::*;

    /// Reusable codec state matching one pipeline runtime's effect-handler state.
    pub struct SyslogCodecBench {
        codec: ResolvedCodec,
        service: CodecService,
    }

    impl Default for SyslogCodecBench {
        fn default() -> Self {
            Self::new()
        }
    }

    impl SyslogCodecBench {
        /// Resolves the syslog codec without creating its mutable instance.
        #[must_use]
        pub fn new() -> Self {
            let codec = resolve_syslog().expect("syslog codec must be registered for benchmark");
            Self {
                codec,
                service: CodecService::new().expect("valid codec registry"),
            }
        }

        /// Runs receiver framing and lazy encoded admission for one message batch.
        #[must_use]
        pub fn admit(&self, messages: &[&[u8]]) -> OtapPayload {
            self.admit_framed(self.frame(messages))
        }

        /// Frames one receiver batch without resolving or invoking mutable codec state.
        #[must_use]
        pub fn frame(&self, messages: &[&[u8]]) -> SyslogBatch {
            let mut builder = SyslogBatchBuilder::with_max_items(messages.len());
            for message in messages {
                builder
                    .append(message)
                    .expect("benchmark message must fit syslog framing");
            }
            let observed_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);
            builder
                .finish(observed_time)
                .expect("benchmark batch must not be empty")
        }

        /// Admits an already framed batch without creating mutable codec state.
        #[must_use]
        pub fn admit_framed(&self, batch: SyslogBatch) -> OtapPayload {
            self.codec
                .admit_built(batch)
                .expect("benchmark batch must be admitted")
        }

        /// Materializes an already framed batch through reusable codec state.
        #[must_use]
        pub fn materialize_framed(&mut self, batch: &SyslogBatch) -> OtapArrowRecords {
            self.admit_framed(batch.clone())
                .try_into_otap(&self.service)
                .expect("benchmark syslog batch must decode")
        }

        /// Runs receiver admission followed by consumer-local lazy OTAP conversion.
        #[must_use]
        pub fn admit_and_materialize(&mut self, messages: &[&[u8]]) -> OtapArrowRecords {
            self.admit(messages)
                .try_into_otap(&self.service)
                .expect("benchmark syslog batch must decode")
        }
    }
}

#[cfg(test)]
mod tests {
    use arrow::array::TimestampNanosecondArray;
    use chrono::TimeZone;
    use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otel_arrow_dfe_pdata_codec::{
        CodecServiceBuilder, DecodePolicy, DecodeValidation, OtapPayload,
    };

    use super::*;

    fn batch(messages: &[&[u8]], observed_time: i64) -> Bytes {
        let mut builder = SyslogBatchBuilder::new();
        for message in messages {
            builder.append(message).expect("append test message");
        }
        builder
            .finish(observed_time)
            .expect("seal test batch")
            .bytes
    }

    /// Scenario: the syslog receiver codec is checked by the shared codec harness.
    /// Guarantees: framed syslog preserves signal and item semantics and malformed
    /// input remains recoverable across repeated decode failures.
    #[test]
    fn syslog_codec_conforms_to_registered_codec_contract() {
        let service = CodecServiceBuilder::from_global_registry()
            .expect("valid codec registry")
            .with_decode_policy(DecodePolicy::new(DecodeValidation::Strict))
            .build();
        let codec = resolve_syslog().unwrap();
        let valid = codec
            .admit(
                SignalType::Logs,
                batch(
                    &[
                        b"<34>1 2024-01-01T00:00:00Z host app - - - first",
                        b"second",
                    ],
                    42,
                ),
            )
            .expect("admit valid syslog batch");
        otel_arrow_dfe_pdata_codec::testing::assert_decode_conformance(
            &service,
            otel_arrow_dfe_pdata_codec::testing::DecodeConformanceCase {
                valid,
                malformed: Some(Bytes::from_static(b"invalid-frame")),
                signal: SignalType::Logs,
                expected_items: 2,
            },
        );
    }

    /// Scenario: A receiver seals multiple original messages into one encoded batch.
    /// Guarantees: Framing preserves byte order, item count, and the flush timestamp.
    #[test]
    fn framing_round_trip_preserves_messages_and_timestamp() {
        let bytes = batch(&[b"first", b"second"], 42);
        let (count, frames) = validated_frames(&bytes).expect("valid framed batch");
        let frames = frames.collect::<Result<Vec<_>, _>>().expect("valid frames");

        assert_eq!(count, 2);
        assert_eq!(frames[0].message, b"first");
        assert_eq!(frames[1].message, b"second");
        assert!(frames.iter().all(|frame| frame.observed_time == 42));
        assert_eq!(
            bytes.len(),
            BATCH_HEADER_LEN + RUN_HEADER_LEN + 2 * FRAME_HEADER_LEN + 11
        );
    }

    /// Scenario: The codec builder seals messages and the receiver admits its opaque result.
    /// Guarantees: The established count and buffer survive admission; framing and timestamps
    /// remain valid, an empty builder fails, and the next nonempty batch still succeeds.
    #[test]
    fn constructed_batch_preserves_count_and_buffer() {
        let codec = resolve_syslog().unwrap();
        let mut builder = SyslogBatchBuilder::new();
        assert!(builder.take().finish(42).is_err());
        builder.append(b"first").unwrap();
        builder.append(b"second").unwrap();
        let batch = builder.take().finish(42).unwrap();
        let pointer = batch.bytes.as_ptr();
        let payload = codec.admit_built(batch).unwrap();
        assert_eq!(payload.known_item_count(), Some(2));
        assert_eq!(payload.encoding(), Some(&SYSLOG_CEF_ENCODING));
        assert_eq!(payload.signal_type(), SignalType::Logs);
        let bytes = payload.encoded_bytes().unwrap();
        assert_eq!(bytes.as_ptr(), pointer);
        assert_eq!(validated_item_count(bytes).unwrap(), 2);
        let (_, frames) = validated_frames(bytes).unwrap();
        assert!(
            frames
                .map(Result::unwrap)
                .all(|frame| frame.observed_time == 42)
        );
        assert!(builder.is_empty());
    }

    /// Scenario: A receiver starts a homogeneous batch with a configured item target.
    /// Guarantees: The first message reserves enough bounded capacity to append the target
    /// number of same-sized messages without growing the buffer again.
    #[test]
    fn first_message_predicts_bounded_batch_capacity() {
        let message = [7_u8; 72];
        let mut builder = SyslogBatchBuilder::with_max_items(100);
        builder.append(&message).expect("append first message");
        let predicted_capacity = builder.capacity();

        for _ in 1..100 {
            builder.append(&message).expect("append predicted message");
        }

        assert_eq!(builder.capacity(), predicted_capacity);
        assert!(predicted_capacity >= BATCH_HEADER_LEN + (FRAME_HEADER_LEN + 72) * 100);
        assert_eq!(
            predicted_batch_capacity(usize::MAX, usize::MAX),
            MAX_PREDICTED_BATCH_CAPACITY
        );
    }

    /// Scenario: Encoded framing is malformed or contains trailing data.
    /// Guarantees: Stateless counting and decoding reject the batch without panicking.
    #[test]
    fn malformed_framing_is_rejected() {
        let complete = batch(&[b"message"], 7);
        // Includes truncation within the per-message receipt year.
        for end in 0..complete.len() {
            assert!(validated_item_count(&complete[..end]).is_err());
            assert!(
                SyslogDecoder
                    .decode(SignalType::Logs, &complete.slice(..end))
                    .is_err()
            );
        }

        let mut trailing = batch(&[b"message"], 7).to_vec();
        trailing.push(0);
        assert!(validated_item_count(&trailing).is_err());
        assert_eq!(count_items(SignalType::Logs, &trailing), None);
        assert_eq!(count_items(SignalType::Metrics, &trailing), None);
    }

    /// Scenario: A native consumer requests OTAP logs from a mixed syslog and CEF batch.
    /// Guarantees: Lazy decoding parses every frame and produces one Arrow log per item.
    #[test]
    fn decode_mixed_messages_to_logs() {
        let bytes = batch(
            &[
                b"<34>1 2024-01-15T10:30:45.123Z host app - ID1 message",
                b"<34>Oct 11 22:14:15 host app[123]: message",
                b"CEF:0|Vendor|Product|1.0|100|Event|5|src=10.0.0.1",
            ],
            123,
        );
        let records = SyslogDecoder
            .decode(SignalType::Logs, &bytes)
            .expect("decode mixed batch");
        let logs = records
            .get(ArrowPayloadType::Logs)
            .expect("logs record batch");
        let observed_times = logs
            .column_by_name("observed_time_unix_nano")
            .expect("observed time column")
            .as_any()
            .downcast_ref::<TimestampNanosecondArray>()
            .expect("timestamp nanosecond array");

        assert_eq!(logs.num_rows(), 3);
        assert!((0..3).all(|index| observed_times.value(index) == 123));
    }

    fn local_timestamp(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> i64 {
        Local
            .with_ymd_and_hms(year, month, day, hour, minute, second)
            .single()
            .unwrap()
            .timestamp_nanos_opt()
            .unwrap()
    }

    /// Scenario: Plain and CEF-wrapped RFC 3164 messages are decoded after their receipt year.
    /// Guarantees: Canonical and mixed-case timestamps use the stored year; RFC 5424, raw CEF,
    /// and observed timestamps remain unchanged regardless of the decoding clock.
    #[test]
    fn delayed_decode_preserves_receipt_year() {
        let messages: &[&[u8]] = &[
            b"<34>Dec 31 23:59:58 host tag: message",
            b"<34>Dec 31 23:59:58 host CEF:0|Security|product|1.0|100|test|10|",
            b"<34>dEc 31 23:59:58 host tag: message",
            b"<34>dEc 31 23:59:58 host CEF:0|Security|product|1.0|100|test|10|",
            b"<34>1 2024-01-01T00:00:00Z host app - - - message",
            b"CEF:0|Security|product|1.0|100|test|10|",
        ];
        // Distinct years make the regression independent of today's year.
        for year in [2020, 2021] {
            let observed = local_timestamp(year + 1, 1, 1, 0, 0, 5);
            let mut builder = SyslogBatchBuilder::new();
            for message in messages {
                builder.append_with_year(message, year).unwrap();
            }
            let batch = builder.finish(observed).unwrap();
            let records = SyslogDecoder
                .decode(SignalType::Logs, &batch.bytes)
                .unwrap();
            let logs = records.get(ArrowPayloadType::Logs).unwrap();
            let times = logs
                .column_by_name("time_unix_nano")
                .unwrap()
                .as_any()
                .downcast_ref::<TimestampNanosecondArray>()
                .unwrap();
            let observed_times = logs
                .column_by_name("observed_time_unix_nano")
                .unwrap()
                .as_any()
                .downcast_ref::<TimestampNanosecondArray>()
                .unwrap();
            let expected = local_timestamp(year, 12, 31, 23, 59, 58);
            assert_eq!(&times.values()[..4], &[expected; 4]);
            assert_eq!(times.value(4), 1_704_067_200_000_000_000);
            assert_eq!(times.value(5), 0);
            assert!(observed_times.values().iter().all(|time| *time == observed));
        }
    }

    /// Scenario: A receiver batch spans New Year and is merged and split with another batch.
    /// Guarantees: Every independently decoded output retains per-message years, order,
    /// observed timestamps, and input ownership ranges even within a shared timestamp run.
    #[test]
    fn native_batching_preserves_years_across_receiver_rollover() {
        let old = b"<34>Dec 31 23:59:58 host tag: message";
        let new = b"<34>Jan 01 00:00:01 host CEF:0|Security|product|1.0|100|test|10|";
        let observed = local_timestamp(2021, 1, 1, 0, 0, 5);
        let mut first = SyslogBatchBuilder::new();
        first.append_with_year(old, 2020).unwrap();
        first.append_with_year(new, 2021).unwrap();
        first.append_with_year(new, 2021).unwrap();
        let mut second = SyslogBatchBuilder::new();
        second.append_with_year(new, 2021).unwrap();
        let inputs = vec![
            first.finish(observed).unwrap().bytes,
            second.finish(observed).unwrap().bytes,
        ];
        let settings = BatchSettings {
            min_size: Some(2),
            max_size: NonZeroU64::new(2),
            ..SYSLOG_BATCH_SETTINGS
        };
        let output = SyslogBatcher
            .batch(SignalType::Logs, &settings, inputs)
            .unwrap();
        assert_eq!(
            output
                .batches
                .iter()
                .map(|(_, range)| range.clone())
                .collect::<Vec<_>>(),
            vec![0..=0, 0..=1]
        );
        let mut years = Vec::new();
        let mut times = Vec::new();
        for (bytes, _) in output.batches {
            let (_, frames) = validated_frames(&bytes).unwrap();
            for frame in frames {
                let frame = frame.unwrap();
                years.push(frame.received_local_year);
                assert_eq!(frame.observed_time, observed);
            }
            let records = SyslogDecoder.decode(SignalType::Logs, &bytes).unwrap();
            let logs = records.get(ArrowPayloadType::Logs).unwrap();
            let values = logs
                .column_by_name("time_unix_nano")
                .unwrap()
                .as_any()
                .downcast_ref::<TimestampNanosecondArray>()
                .unwrap();
            times.extend_from_slice(values.values());
        }
        assert_eq!(years, vec![2020, 2021, 2021, 2021]);
        let new_time = local_timestamp(2021, 1, 1, 0, 0, 1);
        assert_eq!(
            times,
            vec![
                local_timestamp(2020, 12, 31, 23, 59, 58),
                new_time,
                new_time,
                new_time
            ]
        );
    }

    /// Scenario: Flow metrics inspect an admitted syslog batch before codec creation.
    /// Guarantees: The registration supplies an exact stateless count while admission stays lazy.
    #[test]
    fn registration_counts_admitted_items_without_decode() {
        let bytes = batch(&[b"first", b"second", b"third"], 5);
        let codec = resolve_syslog().expect("resolve syslog codec");
        let mut payload: OtapPayload = codec
            .admit(SignalType::Logs, bytes)
            .expect("admit framed bytes")
            .into();

        assert_eq!(payload.known_item_count(), None);
        assert_eq!(payload.num_items(), 3);
        assert_eq!(payload.known_item_count(), Some(3));
    }

    /// Scenario: Native batching combines adjacent input batches sealed at the same time.
    /// Guarantees: Equal timestamps share one run while message order and item count are retained.
    #[test]
    fn native_batching_coalesces_adjacent_timestamp_runs() {
        let first = batch(&[b"one", b"two"], 11);
        let second = batch(&[b"three"], 11);
        let settings = BatchSettings {
            min_size: Some(4),
            max_size: NonZeroU64::new(4),
            sizer: BatchSizer::Items,
            max_split_fragments: None,
            max_split_overhead_bytes: None,
            max_split_fragments_per_flush: None,
        };

        let output = SyslogBatcher
            .batch(SignalType::Logs, &settings, vec![first, second])
            .expect("batch encoded syslog");

        assert_eq!(output.batches.len(), 1);
        let (bytes, input_range) = &output.batches[0];
        assert_eq!(*input_range, 0..=1);
        assert_eq!(validated_item_count(bytes).unwrap(), 3);
        assert_eq!(
            read_u32(&bytes[BATCH_HEADER_LEN + 8..BATCH_HEADER_LEN + RUN_HEADER_LEN]),
            3
        );
        assert_eq!(
            bytes.len(),
            BATCH_HEADER_LEN + RUN_HEADER_LEN + 3 * FRAME_HEADER_LEN + 11
        );
        let (_, frames) = validated_frames(bytes).expect("valid coalesced batch");
        let frames = frames.collect::<Result<Vec<_>, _>>().expect("valid frames");
        assert_eq!(
            frames.iter().map(|frame| frame.message).collect::<Vec<_>>(),
            vec![b"one".as_slice(), b"two".as_slice(), b"three".as_slice()]
        );
        assert!(frames.iter().all(|frame| frame.observed_time == 11));
    }

    /// Scenario: Native batching combines inputs and splits at an item limit.
    /// Guarantees: Input ranges, order, original bytes, and observed timestamps are preserved.
    #[test]
    fn native_batching_splits_by_item_count() {
        let first = batch(&[b"one", b"two"], 11);
        let second = batch(&[b"three"], 22);
        let settings = BatchSettings {
            min_size: Some(2),
            max_size: NonZeroU64::new(2),
            sizer: BatchSizer::Items,
            max_split_fragments: None,
            max_split_overhead_bytes: None,
            max_split_fragments_per_flush: None,
        };

        let output = SyslogBatcher
            .batch(SignalType::Logs, &settings, vec![first, second])
            .expect("batch encoded syslog");

        assert_eq!(output.budget_fallbacks, 0);
        assert_eq!(output.batches.len(), 2);
        assert_eq!(output.batches[0].1, 0..=0);
        assert_eq!(output.batches[1].1, 1..=1);
        assert_eq!(validated_item_count(&output.batches[0].0).unwrap(), 2);
        assert_eq!(validated_item_count(&output.batches[1].0).unwrap(), 1);
        let (_, first_frames) = validated_frames(&output.batches[0].0).expect("first output");
        let (_, second_frames) = validated_frames(&output.batches[1].0).expect("second output");
        let first_frames = first_frames
            .collect::<Result<Vec<_>, _>>()
            .expect("valid first frames");
        let second_frames = second_frames
            .collect::<Result<Vec<_>, _>>()
            .expect("valid second frames");
        assert_eq!(first_frames[0].message, b"one");
        assert_eq!(first_frames[1].message, b"two");
        assert!(first_frames.iter().all(|frame| frame.observed_time == 11));
        assert_eq!(second_frames[0].message, b"three");
        assert_eq!(second_frames[0].observed_time, 22);
    }

    /// Scenario: Splitting spans multiple syslog inputs and a limit exceeds a 32-bit size.
    /// Guarantees: Output ranges track original inputs and wide limits never wrap or truncate.
    #[test]
    fn native_batching_tracks_split_inputs_and_wide_limits() {
        let inputs = vec![
            batch(&[b"one", b"two", b"three"], 11),
            batch(&[b"four", b"five"], 22),
        ];
        let mut settings = SYSLOG_BATCH_SETTINGS;
        settings.min_size = Some(2);
        settings.max_size = NonZeroU64::new(2);
        let result = SyslogBatcher
            .batch(SignalType::Logs, &settings, inputs.clone())
            .unwrap();
        assert_eq!(
            result
                .batches
                .iter()
                .map(|(_, range)| range.clone())
                .collect::<Vec<_>>(),
            vec![0..=0, 0..=1, 1..=1]
        );
        assert!(result.completed_inputs.is_empty());
        assert_eq!(
            result
                .batches
                .iter()
                .map(|(bytes, _)| validated_item_count(bytes).unwrap())
                .collect::<Vec<_>>(),
            vec![2, 2, 1]
        );

        settings.max_size = NonZeroU64::new(u64::MAX);
        let result = SyslogBatcher
            .batch(SignalType::Logs, &settings, inputs)
            .unwrap();
        assert_eq!(result.batches.len(), 1);
        assert_eq!(result.batches[0].1, 0..=1);
        assert_eq!(validated_item_count(&result.batches[0].0).unwrap(), 5);
    }
}
