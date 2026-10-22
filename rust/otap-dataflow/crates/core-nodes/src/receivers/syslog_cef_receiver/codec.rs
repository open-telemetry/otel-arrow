// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Independently decodable batches of original syslog and CEF messages.
//!
//! `syslog-batch` uses this big-endian framing:
//!
//! ```text
//! "SLG1" | item_count:u32 | repeated {
//!     observed_time_unix_nano:i64 | run_item_count:u32 | received_local_year:i32 |
//!     repeated { message_length:u32 | message_bytes
//!     }
//! }
//! ```
//!
//! Observed timestamps are captured when a receiver batch is sealed. Each
//! run also retains the local year at receipt, so deferred RFC 3164 decoding
//! does not infer a different year after New Year. The builder starts a new run
//! when the receipt year changes. Native batching copies frames and coalesces
//! adjacent runs only when both observed timestamp and receipt year match,
//! without parsing syslog contents.
//!
//! Synchronous materialization is capped at 256 messages and 256 KiB of framing
//! (16 KiB per message). Receiver admission and native batching enforce these
//! same limits until cooperative codec execution is available.

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

use super::arrow_records_encoder::ReceivedArrowRecordsBuilder;
use super::{MAX_MESSAGE_SIZE, parser};

/// Stable identity of the versioned syslog/CEF batch representation.
const SYSLOG_CEF_ENCODING: PdataEncoding = PdataEncoding::new("syslog-batch");

const MAGIC: &[u8; 4] = b"SLG1";
const BATCH_HEADER_LEN: usize = 8;
const RUN_HEADER_LEN: usize = 16;
const FRAME_HEADER_LEN: usize = 4;
// Bound one synchronous parse/build/finalize operation until cooperative codec
// execution is available. These are hard limits, not configurable batch targets.
pub(super) const MAX_BATCH_ITEMS: usize = 256;
const MAX_BATCH_BYTES: usize = 256 * 1024;
const MAX_PREDICTED_BATCH_CAPACITY: usize = 32 * 1024;

const SYSLOG_BATCH_SETTINGS: BatchSettings = BatchSettings {
    min_size: Some(100),
    max_size: NonZeroU64::new(MAX_BATCH_ITEMS as u64),
    sizer: BatchSizer::Items,
    max_split_fragments: None,
    max_split_overhead_bytes: None,
    max_split_fragments_per_flush: None,
};

/// A malformed syslog batch frame.
#[derive(Debug, thiserror::Error)]
#[error("{reason}")]
pub(super) struct SyslogFramingError {
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

fn predicted_frame_capacity(max_items: usize, message_len: usize) -> usize {
    // reserve() takes additional bytes; the first run header is already present.
    FRAME_HEADER_LEN
        .saturating_add(message_len)
        .saturating_mul(max_items.max(1))
        .min(MAX_PREDICTED_BATCH_CAPACITY - BATCH_HEADER_LEN - RUN_HEADER_LEN)
}

/// Accumulates framed syslog messages without parsing them.
pub(super) struct SyslogBatchBuilder {
    bytes: BytesMut,
    item_count: u32,
    max_items_hint: usize,
    run_year: Option<i32>,
    run_count: u32,
    run_count_offset: usize,
    // The first run is always at BATCH_HEADER_LEN. Only a receipt-year change
    // needs another offset, so the ordinary single-year batch allocates no list.
    additional_run_offsets: Vec<usize>,
}

/// A batch sealed by the syslog codec, retaining the count established while framing.
///
/// Only the codec builder can construct this value. Its bytes and count cannot
/// be replaced independently. Cloning shares the bytes and preserves the count.
#[derive(Clone)]
pub(super) struct SyslogBatch {
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
    pub(super) fn new() -> Self {
        Self::with_max_items(1)
    }

    /// Creates an empty batch using a bounded first-message capacity estimate.
    #[must_use]
    pub(super) fn with_max_items(max_items_hint: usize) -> Self {
        let mut bytes = BytesMut::with_capacity(BATCH_HEADER_LEN + RUN_HEADER_LEN);
        bytes.extend_from_slice(MAGIC);
        bytes.put_u32(0);
        Self {
            bytes,
            item_count: 0,
            max_items_hint: max_items_hint.clamp(1, MAX_BATCH_ITEMS),
            run_year: None,
            run_count: 0,
            run_count_offset: 0,
            additional_run_offsets: Vec::new(),
        }
    }

    /// Number of framed messages currently buffered.
    #[must_use]
    pub(super) const fn len(&self) -> u32 {
        self.item_count
    }

    /// Returns true when no messages have been appended.
    #[must_use]
    pub(super) const fn is_empty(&self) -> bool {
        self.item_count == 0
    }

    /// Flushes before another maximum-sized message or a new year run could exceed
    /// the hard byte limit. The caller also applies its configured item target.
    pub(super) fn should_flush(&self, max_items: usize) -> bool {
        self.len() as usize >= max_items.min(MAX_BATCH_ITEMS)
            || self.bytes.len()
                > MAX_BATCH_BYTES - MAX_MESSAGE_SIZE - FRAME_HEADER_LEN - RUN_HEADER_LEN
    }

    /// Copies a socket-backed message and captures its local receipt year.
    pub(super) fn append(&mut self, message: &[u8]) -> Result<(), SyslogFramingError> {
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
        if message.len() > MAX_MESSAGE_SIZE {
            return Err(framing_error("syslog message exceeds 16 KiB"));
        }
        let run_header_len = if self.run_year == Some(received_local_year) {
            0
        } else {
            RUN_HEADER_LEN
        };
        validate_decode_limits(
            self.item_count as usize + 1,
            self.bytes.len() + run_header_len + FRAME_HEADER_LEN + message.len(),
        )?;
        let message_len = u32::try_from(message.len())
            .map_err(|_| framing_error("syslog message length exceeds u32"))?;
        let item_count = self
            .item_count
            .checked_add(1)
            .ok_or_else(|| framing_error("syslog batch item count exceeds u32"))?;
        if self.run_year != Some(received_local_year) {
            if !self.is_empty() {
                self.bytes[self.run_count_offset..self.run_count_offset + 4]
                    .copy_from_slice(&self.run_count.to_be_bytes());
                self.additional_run_offsets.push(self.bytes.len());
            }
            self.bytes.put_i64(0);
            self.run_count_offset = self.bytes.len();
            self.bytes.put_u32(0);
            self.bytes.put_i32(received_local_year);
            self.run_count = 0;
            self.run_year = Some(received_local_year);
        }
        if self.is_empty() {
            self.bytes
                .reserve(predicted_frame_capacity(self.max_items_hint, message.len()));
        }
        self.item_count = item_count;
        // A run is no larger than the checked total item count above.
        self.run_count += 1;
        self.bytes.put_u32(message_len);
        self.bytes.extend_from_slice(message);
        Ok(())
    }

    /// Replaces this builder while retaining its configured item-count hint.
    #[must_use]
    pub(super) fn take(&mut self) -> Self {
        let replacement = Self::with_max_items(self.max_items_hint);
        std::mem::replace(self, replacement)
    }

    /// Discards buffered messages and releases their allocation.
    pub(super) fn discard(&mut self) {
        *self = Self::with_max_items(self.max_items_hint);
    }

    #[cfg(test)]
    pub(super) fn capacity(&self) -> usize {
        self.bytes.capacity()
    }

    /// Seals a non-empty batch, assigning the flush timestamp to every year run.
    pub(super) fn finish(mut self, observed_time: i64) -> Result<SyslogBatch, SyslogFramingError> {
        if self.is_empty() {
            return Err(framing_error("cannot seal an empty syslog batch"));
        }
        self.bytes[4..BATCH_HEADER_LEN].copy_from_slice(&self.item_count.to_be_bytes());
        self.bytes[self.run_count_offset..self.run_count_offset + 4]
            .copy_from_slice(&self.run_count.to_be_bytes());
        for offset in std::iter::once(BATCH_HEADER_LEN).chain(self.additional_run_offsets) {
            self.bytes[offset..offset + 8].copy_from_slice(&observed_time.to_be_bytes());
        }
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
    received_local_year: i32,
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
            let run_header_end = match self.cursor.checked_add(RUN_HEADER_LEN) {
                Some(end) if end <= self.bytes.len() => end,
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
            self.run_remaining = read_u32(&self.bytes[self.cursor + 8..self.cursor + 12]) as usize;
            let year = &self.bytes[self.cursor + 12..run_header_end];
            self.received_local_year = i32::from_be_bytes([year[0], year[1], year[2], year[3]]);
            self.cursor = run_header_end;
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
        let message_len = read_u32(&self.bytes[start..header_end]) as usize;
        if message_len > MAX_MESSAGE_SIZE {
            self.finished = true;
            return Some(Err(framing_error("syslog message exceeds 16 KiB")));
        }
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
            received_local_year: self.received_local_year,
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
            received_local_year: 0,
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

// Reject oversized materialization before allocating Arrow builders. Batching may
// split larger framed inputs, but neither configuration nor a budget fallback may
// emit one that the decoder cannot safely process in a single synchronous call.
fn validate_decode_limits(count: usize, bytes: usize) -> Result<(), SyslogFramingError> {
    if count > MAX_BATCH_ITEMS {
        return Err(framing_error("syslog batch exceeds 256 messages"));
    }
    if bytes > MAX_BATCH_BYTES {
        return Err(framing_error("syslog batch exceeds 256 KiB"));
    }
    Ok(())
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
        validate_decode_limits(count, bytes.len())
            .map_err(|error| operation_error(CodecOperation::Decode, error))?;
        let mut builder = ReceivedArrowRecordsBuilder::default();
        for frame in frames {
            let frame = frame.map_err(|error| operation_error(CodecOperation::Decode, error))?;
            let parsed = parser::parse(frame.message)
                .map_err(|error| framing_error(format!("invalid syslog message: {error:?}")))
                .map_err(|error| operation_error(CodecOperation::Decode, error))?;
            builder.append_syslog(parsed, frame.observed_time, frame.received_local_year);
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
        let max_items = settings.max_size.map_or(MAX_BATCH_ITEMS, |size| {
            // Configuration may lower, but never raise, the synchronous work limit.
            usize::try_from(size.get())
                .unwrap_or(MAX_BATCH_ITEMS)
                .min(MAX_BATCH_ITEMS)
        });
        // Timer flushes often contain one undersized input. Keep its buffer
        // when no split is needed, but consume every frame so malformed framing
        // still fails. Neither requested targets nor this shortcut relax the
        // hard item/byte limits or the per-message limit checked by FrameIter.
        if let [input] = inputs.as_slice()
            && input.len() <= MAX_BATCH_BYTES
        {
            let (count, mut frames) = validated_frames(input)
                .map_err(|error| operation_error(CodecOperation::Batch, error))?;
            if count <= max_items {
                frames
                    .try_for_each(|frame| frame.map(|_| ()))
                    .map_err(|error| operation_error(CodecOperation::Batch, error))?;
                return Ok(CodecBatches {
                    batches: inputs.into_iter().map(|bytes| (bytes, 0..=0)).collect(),
                    completed_inputs: Vec::new(),
                    budget_fallbacks: 0,
                });
            }
        }
        let limited_splits = settings.max_split_fragments.is_some()
            || settings.max_split_overhead_bytes.is_some()
            || settings.max_split_fragments_per_flush.is_some();
        let mut split_fanout = 0_u64;
        let mut budget_fallbacks = 0;
        let mut outputs = Vec::new();
        let mut output = RawBatchWriter::new();
        let mut first_input = 0;
        let mut last_input = 0;
        for (input_index, input) in inputs.into_iter().enumerate() {
            let (count, frames) = validated_frames(&input)
                .map_err(|error| operation_error(CodecOperation::Batch, error))?;
            // With finite budgets, avoid splitting a whole input merely to fill
            // pending space. Split oversized inputs in isolation so preflight
            // and whole-input fallback cannot partially emit the same input.
            if limited_splits
                && (count > max_items - output.len()
                    || output
                        .bytes
                        .len()
                        .saturating_add(input.len() - BATCH_HEADER_LEN)
                        > MAX_BATCH_BYTES)
            {
                if !output.is_empty() {
                    outputs.push(
                        output
                            .finish()
                            .map(|(bytes, _)| (bytes, first_input..=last_input))
                            .map_err(|error| operation_error(CodecOperation::Batch, error))?,
                    );
                    output = RawBatchWriter::new();
                }
                if count > max_items || input.len() > MAX_BATCH_BYTES {
                    let (fragments, output_bytes) = split_layout(&input, max_items)
                        .map_err(|error| operation_error(CodecOperation::Batch, error))?;
                    if exceeds_split_budget(
                        input.len(),
                        fragments,
                        output_bytes,
                        split_fanout,
                        settings,
                    ) {
                        // split_layout has validated every frame. A budget may
                        // relax the requested size, but never the hard decode caps.
                        validate_decode_limits(count, input.len())
                            .map_err(|error| operation_error(CodecOperation::Batch, error))?;
                        outputs.push((input, input_index..=input_index));
                        budget_fallbacks += 1;
                        continue;
                    }
                    // One whole output per input is mandatory; only additional
                    // outputs consume the cumulative split fan-out allowance.
                    split_fanout = split_fanout.saturating_add(fragments - 1);
                }
            }
            for frame in frames {
                let frame = frame.map_err(|error| operation_error(CodecOperation::Batch, error))?;
                if output.len() == max_items
                    || appended_len(output.bytes.len(), output.run_context, &frame)
                        > MAX_BATCH_BYTES
                {
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
            budget_fallbacks,
        })
    }
}

// Preflight both item and byte splits without allocating output buffers. Use
// the same run coalescing as RawBatchWriter so wrapper amplification is exact.
fn split_layout(input: &[u8], max_items: usize) -> Result<(u64, u64), SyslogFramingError> {
    let (_, frames) = validated_frames(input)?;
    let mut fragments = 1;
    let mut output_bytes = 0_u64;
    let mut bytes = BATCH_HEADER_LEN;
    let mut count = 0;
    let mut context = None;
    for frame in frames {
        let frame = frame?;
        if count == max_items || appended_len(bytes, context, &frame) > MAX_BATCH_BYTES {
            fragments += 1;
            output_bytes += bytes as u64;
            bytes = BATCH_HEADER_LEN;
            count = 0;
            context = None;
        }
        bytes = appended_len(bytes, context, &frame);
        context = Some((frame.observed_time, frame.received_local_year));
        count += 1;
    }
    Ok((fragments, output_bytes + bytes as u64))
}

fn appended_len(bytes: usize, context: Option<(i64, i32)>, frame: &Frame<'_>) -> usize {
    let run_header = if context == Some((frame.observed_time, frame.received_local_year)) {
        0
    } else {
        RUN_HEADER_LEN
    };
    bytes
        .saturating_add(run_header)
        .saturating_add(frame.encoded.len())
}

fn exceeds_split_budget(
    input_bytes: usize,
    fragments: u64,
    output_bytes: u64,
    split_fanout: u64,
    settings: &BatchSettings,
) -> bool {
    settings
        .max_split_fragments
        .is_some_and(|limit| fragments > limit.get())
        || settings
            .max_split_fragments_per_flush
            .is_some_and(|limit| split_fanout.saturating_add(fragments - 1) > limit.get())
        || settings
            .max_split_overhead_bytes
            .is_some_and(|limit| output_bytes.saturating_sub(input_bytes as u64) > limit.get())
}

struct RawBatchWriter {
    bytes: BytesMut,
    count: u32,
    run_count_offset: usize,
    run_count: u32,
    run_context: Option<(i64, i32)>,
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
            run_context: None,
        }
    }

    const fn len(&self) -> usize {
        self.count as usize
    }

    const fn is_empty(&self) -> bool {
        self.count == 0
    }

    fn append(&mut self, frame: Frame<'_>) -> Result<(), SyslogFramingError> {
        validate_decode_limits(
            self.count as usize + 1,
            appended_len(self.bytes.len(), self.run_context, &frame),
        )?;
        let context = (frame.observed_time, frame.received_local_year);
        if self.run_context != Some(context) {
            self.bytes.put_i64(frame.observed_time);
            self.run_count_offset = self.bytes.len();
            self.bytes.put_u32(0);
            self.bytes.put_i32(frame.received_local_year);
            self.run_count = 0;
            self.run_context = Some(context);
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
pub(super) fn resolve_syslog() -> Result<ResolvedCodec, RegistryError> {
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

    /// Opaque framed input for repeated benchmark materialization.
    ///
    /// Only the benchmark facade constructs this handle; framing internals
    /// remain private to the receiver. Cloning shares its encoded buffer.
    #[derive(Clone)]
    pub struct FramedSyslogBatch(SyslogBatch);

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
        pub fn frame(&self, messages: &[&[u8]]) -> FramedSyslogBatch {
            let mut builder = SyslogBatchBuilder::with_max_items(messages.len());
            for message in messages {
                builder
                    .append(message)
                    .expect("benchmark message must fit syslog framing");
            }
            let observed_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);
            FramedSyslogBatch(
                builder
                    .finish(observed_time)
                    .expect("benchmark batch must not be empty"),
            )
        }

        /// Admits an already framed batch without creating mutable codec state.
        #[must_use]
        fn admit_framed(&self, batch: FramedSyslogBatch) -> OtapPayload {
            self.codec
                .admit_built(batch.0)
                .expect("benchmark batch must be admitted")
        }

        /// Materializes an already framed batch through reusable codec state.
        #[must_use]
        pub fn materialize_framed(&mut self, batch: &FramedSyslogBatch) -> OtapArrowRecords {
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

    // Deliberately bypass admission limits to test hostile framing and splitting
    // older/larger encoded inputs without weakening the production builder.
    fn unchecked_batch(messages: &[&[u8]]) -> Bytes {
        let mut bytes = BytesMut::new();
        bytes.extend_from_slice(MAGIC);
        bytes.put_u32(u32::try_from(messages.len()).unwrap());
        bytes.put_i64(42);
        bytes.put_u32(u32::try_from(messages.len()).unwrap());
        bytes.put_i32(2024);
        for message in messages {
            bytes.put_u32(u32::try_from(message.len()).unwrap());
            bytes.extend_from_slice(message);
        }
        bytes.freeze()
    }

    /// Scenario: One timer-flushed input is undersized or exactly at either decode cap.
    /// Guarantees: Valid frames retain their original buffer and ownership without a split-budget fallback.
    #[test]
    fn single_input_reuses_validated_buffer_at_decode_boundaries() {
        let mut mixed = SyslogBatchBuilder::new();
        mixed.append_with_year(b"first", 2024).unwrap();
        mixed.append_with_year(b"second", 2025).unwrap();
        let mixed = mixed.finish(42).unwrap().bytes;
        let at_item_limit = batch(&vec![b"message".as_slice(); MAX_BATCH_ITEMS], 42);
        let large = vec![b'x'; MAX_MESSAGE_SIZE];
        let mut full = SyslogBatchBuilder::new();
        for _ in 0..15 {
            full.append_with_year(&large, 2024).unwrap();
        }
        let remaining = MAX_BATCH_BYTES - full.bytes.len() - FRAME_HEADER_LEN;
        full.append_with_year(&large[..remaining], 2024).unwrap();
        let at_byte_limit = full.finish(42).unwrap().bytes;
        assert_eq!(at_byte_limit.len(), MAX_BATCH_BYTES);

        for input in [mixed, at_item_limit, at_byte_limit] {
            let count = validated_item_count(&input).unwrap();
            for settings in [
                SYSLOG_BATCH_SETTINGS,
                BatchSettings {
                    max_size: NonZeroU64::new(u64::MAX),
                    max_split_fragments: NonZeroU64::new(1),
                    max_split_overhead_bytes: NonZeroU64::new(1),
                    max_split_fragments_per_flush: NonZeroU64::new(1),
                    ..SYSLOG_BATCH_SETTINGS
                },
            ] {
                let output = SyslogBatcher
                    .batch(SignalType::Logs, &settings, vec![input.clone()])
                    .unwrap();
                assert_eq!(output.batches.len(), 1);
                let (bytes, owners) = &output.batches[0];
                assert_eq!(bytes.as_ptr(), input.as_ptr());
                assert_eq!(bytes, &input);
                assert_eq!(*owners, 0..=0);
                assert_eq!(output.budget_fallbacks, 0);
                assert!(output.completed_inputs.is_empty());
                assert_eq!(
                    SyslogDecoder
                        .decode(SignalType::Logs, bytes)
                        .unwrap()
                        .num_items(),
                    count
                );
            }
        }
    }

    /// Scenario: A single fitting input has a valid outer header but malformed nested framing.
    /// Guarantees: The reuse path rejects truncated, trailing, miscounted, and oversized frames and remains reusable.
    #[test]
    fn single_input_reuse_validates_every_frame_and_recovers() {
        let valid = batch(&[b"first", b"second"], 42);
        let mut trailing = valid.to_vec();
        trailing.push(0);
        let mut invalid_run = valid.to_vec();
        invalid_run[16..20].copy_from_slice(&0_u32.to_be_bytes());
        let mut wrong_count = valid.to_vec();
        wrong_count[4..8].copy_from_slice(&3_u32.to_be_bytes());
        let too_large = vec![b'x'; MAX_MESSAGE_SIZE + 1];
        let mut batcher = SyslogBatcher;
        for invalid in [
            valid.slice(..valid.len() - 1),
            Bytes::from(trailing),
            Bytes::from(invalid_run),
            Bytes::from(wrong_count),
            unchecked_batch(&[too_large.as_slice()]),
        ] {
            assert!(validated_frames(&invalid).is_ok());
            assert!(
                batcher
                    .batch(SignalType::Logs, &SYSLOG_BATCH_SETTINGS, vec![invalid])
                    .is_err()
            );
            let recovered = batcher
                .batch(
                    SignalType::Logs,
                    &SYSLOG_BATCH_SETTINGS,
                    vec![valid.clone()],
                )
                .unwrap();
            assert_eq!(recovered.batches[0].0.as_ptr(), valid.as_ptr());
        }
    }

    /// Scenario: One input exceeds a requested item target or a hard item/byte cap.
    /// Guarantees: Single-input reuse never bypasses splitting, decode limits, message order, or input ownership.
    #[test]
    fn single_input_reuse_still_splits_oversized_batches() {
        let large = vec![b'x'; MAX_MESSAGE_SIZE];
        for (input, max_size, expected_counts) in [
            (
                batch(&[b"one", b"two", b"three", b"four"], 42),
                NonZeroU64::new(2),
                vec![2, 2],
            ),
            (
                unchecked_batch(&vec![b"message".as_slice(); MAX_BATCH_ITEMS + 1]),
                None,
                vec![256, 1],
            ),
            (
                unchecked_batch(&[large.as_slice(); 16]),
                NonZeroU64::new(u64::MAX),
                vec![15, 1],
            ),
        ] {
            let messages = |bytes: &Bytes| {
                validated_frames(bytes)
                    .unwrap()
                    .1
                    .map(|frame| frame.unwrap().message.to_vec())
                    .collect::<Vec<_>>()
            };
            let expected_messages = messages(&input);
            let settings = BatchSettings {
                min_size: Some(1),
                max_size,
                ..SYSLOG_BATCH_SETTINGS
            };
            let result = SyslogBatcher
                .batch(SignalType::Logs, &settings, vec![input])
                .unwrap();
            let mut actual_messages = Vec::new();
            let mut counts = Vec::new();
            for (bytes, owners) in result.batches {
                counts.push(validated_item_count(&bytes).unwrap());
                assert!(bytes.len() <= MAX_BATCH_BYTES);
                assert_eq!(owners, 0..=0);
                assert!(SyslogDecoder.decode(SignalType::Logs, &bytes).is_ok());
                actual_messages.extend(messages(&bytes));
            }
            assert_eq!(counts, expected_counts);
            assert_eq!(actual_messages, expected_messages);
            assert_eq!(result.budget_fallbacks, 0);
            assert!(result.completed_inputs.is_empty());
        }
    }

    /// Scenario: Admission reaches either hard bound or receives an oversized message.
    /// Guarantees: Rejected appends leave the batch untouched and exact boundaries decode.
    #[test]
    fn builder_enforces_item_byte_and_message_limits_atomically() {
        let mut builder = SyslogBatchBuilder::with_max_items(usize::MAX);
        for _ in 0..MAX_BATCH_ITEMS {
            builder.append_with_year(b"message", 2024).unwrap();
        }
        let before = builder.bytes.clone();
        assert!(builder.append_with_year(b"extra", 2025).is_err());
        assert_eq!(builder.bytes, before);
        assert_eq!(builder.len() as usize, MAX_BATCH_ITEMS);
        assert!(builder.should_flush(usize::MAX));
        assert_eq!(
            SyslogDecoder
                .decode(SignalType::Logs, &builder.finish(42).unwrap().bytes)
                .unwrap()
                .num_items(),
            MAX_BATCH_ITEMS
        );

        let message = vec![b'x'; MAX_MESSAGE_SIZE];
        let mut builder = SyslogBatchBuilder::new();
        for _ in 0..15 {
            builder.append_with_year(&message, 2024).unwrap();
        }
        assert!(builder.should_flush(usize::MAX));
        let remaining = MAX_BATCH_BYTES - builder.bytes.len() - FRAME_HEADER_LEN;
        builder
            .append_with_year(&message[..remaining], 2024)
            .unwrap();
        assert_eq!(builder.bytes.len(), MAX_BATCH_BYTES);
        let before = builder.bytes.clone();
        assert!(builder.append_with_year(b"extra", 2025).is_err());
        assert_eq!(builder.bytes, before);
        assert_eq!(
            SyslogDecoder
                .decode(SignalType::Logs, &builder.finish(42).unwrap().bytes)
                .unwrap()
                .num_items(),
            16
        );

        let mut builder = SyslogBatchBuilder::new();
        assert!(builder.append(&vec![b'x'; MAX_MESSAGE_SIZE + 1]).is_err());
        assert!(builder.is_empty());
        builder.append(b"valid after error").unwrap();
    }

    /// Scenario: Encoded input bypasses admission with excessive items, bytes, or message size.
    /// Guarantees: Decode errors recover the same identity, cached count, and buffer; reuse succeeds.
    #[test]
    fn oversized_decoding_recovers_input_and_keeps_decoder_reusable() {
        let service = otel_arrow_dfe_pdata_codec::CodecService::new().unwrap();
        let codec = resolve_syslog().unwrap();
        let large = vec![b'x'; MAX_MESSAGE_SIZE];
        let too_large = vec![b'x'; MAX_MESSAGE_SIZE + 1];
        for bytes in [
            unchecked_batch(&vec![b"message".as_slice(); MAX_BATCH_ITEMS + 1]),
            unchecked_batch(&vec![large.as_slice(); 16]),
            unchecked_batch(&[too_large.as_slice()]),
        ] {
            let mut payload: OtapPayload =
                codec.admit(SignalType::Logs, bytes.clone()).unwrap().into();
            let _ = payload.num_items();
            let count = payload.known_item_count();
            let (error, recovered) = payload.try_into_otap(&service).unwrap_err().into_parts();
            assert!(error.to_string().contains("exceeds"));
            assert_eq!(recovered.encoding(), Some(&SYSLOG_CEF_ENCODING));
            assert_eq!(recovered.known_item_count(), count);
            let original = recovered.encoded_bytes().unwrap();
            assert_eq!(original, &bytes);
            assert_eq!(original.as_ptr(), bytes.as_ptr());
        }
        let payload: OtapPayload = codec
            .admit(SignalType::Logs, batch(&[b"valid"], 42))
            .unwrap()
            .into();
        assert_eq!(payload.try_into_otap(&service).unwrap().num_items(), 1);
    }

    /// Scenario: Oversized user targets merge several admitted batches.
    /// Guarantees: Hard item and byte caps override targets without losing order or input ranges.
    #[test]
    fn native_batching_caps_user_overrides_and_preserves_input_ranges() {
        let settings = BatchSettings {
            max_size: NonZeroU64::new(u64::MAX),
            ..SYSLOG_BATCH_SETTINGS
        };
        let mut inputs = Vec::new();
        for (message, observed) in [(b"first".as_slice(), 1), (b"second", 2), (b"third", 3)] {
            inputs.push(batch(&vec![message; 200], observed));
        }
        let result = SyslogBatcher
            .batch(SignalType::Logs, &settings, inputs)
            .unwrap();
        assert_eq!(
            result
                .batches
                .iter()
                .map(|(_, r)| r.clone())
                .collect::<Vec<_>>(),
            [0..=1, 1..=2, 2..=2]
        );
        let mut observed = Vec::new();
        for (bytes, _) in result.batches {
            let (count, frames) = validated_frames(&bytes).unwrap();
            assert!(count <= MAX_BATCH_ITEMS && bytes.len() <= MAX_BATCH_BYTES);
            assert_eq!(
                SyslogDecoder
                    .decode(SignalType::Logs, &bytes)
                    .unwrap()
                    .num_items(),
                count
            );
            observed.extend(frames.map(|frame| frame.unwrap().observed_time));
        }
        assert_eq!(
            observed,
            [vec![1; 200], vec![2; 200], vec![3; 200]].concat()
        );

        let large = vec![b'x'; MAX_MESSAGE_SIZE];
        let input = batch(&[large.as_slice(); 8], 42);
        let result = SyslogBatcher
            .batch(SignalType::Logs, &settings, vec![input.clone(), input])
            .unwrap();
        assert_eq!(
            result
                .batches
                .iter()
                .map(|(_, r)| r.clone())
                .collect::<Vec<_>>(),
            [0..=1, 1..=1]
        );
        assert_eq!(
            result
                .batches
                .iter()
                .map(|(b, _)| validated_item_count(b).unwrap())
                .collect::<Vec<_>>(),
            [15, 1]
        );
        for (bytes, _) in result.batches {
            assert!(bytes.len() <= MAX_BATCH_BYTES);
            assert!(SyslogDecoder.decode(SignalType::Logs, &bytes).is_ok());
        }
    }

    /// Scenario: A byte-limited split exhausts each finite amplification budget.
    /// Guarantees: Fallback cannot bypass hard limits, and exact budgets preserve independently decodable outputs.
    #[test]
    fn byte_splits_respect_budgets_without_oversized_fallback() {
        let message = vec![b'x'; MAX_MESSAGE_SIZE];
        let input = unchecked_batch(&vec![message.as_slice(); 32]);
        let settings = BatchSettings {
            max_size: NonZeroU64::new(u64::MAX),
            ..SYSLOG_BATCH_SETTINGS
        };
        for (fragments, overhead, flush) in [
            (Some(2), None, None),
            (None, Some(47), None),
            (None, None, Some(1)),
        ] {
            let restricted = BatchSettings {
                max_split_fragments: fragments.and_then(NonZeroU64::new),
                max_split_overhead_bytes: overhead.and_then(NonZeroU64::new),
                max_split_fragments_per_flush: flush.and_then(NonZeroU64::new),
                ..settings.clone()
            };
            assert!(
                SyslogBatcher
                    .batch(SignalType::Logs, &restricted, vec![input.clone()])
                    .is_err()
            );
        }
        let exact = BatchSettings {
            max_split_fragments: NonZeroU64::new(3),
            max_split_overhead_bytes: NonZeroU64::new(48),
            max_split_fragments_per_flush: NonZeroU64::new(2),
            ..settings
        };
        let result = SyslogBatcher
            .batch(SignalType::Logs, &exact, vec![input.clone()])
            .unwrap();
        assert_eq!(result.budget_fallbacks, 0);
        assert_eq!(
            result.batches.iter().map(|(b, _)| b.len()).sum::<usize>() - input.len(),
            48
        );
        assert_eq!(
            result
                .batches
                .iter()
                .map(|(b, _)| validated_item_count(b).unwrap())
                .collect::<Vec<_>>(),
            [15, 15, 2]
        );
        for (bytes, range) in result.batches {
            assert_eq!(range, 0..=0);
            assert!(bytes.len() <= MAX_BATCH_BYTES);
            assert!(SyslogDecoder.decode(SignalType::Logs, &bytes).is_ok());
        }
    }

    /// Scenario: A timer shares one runtime thread with a maximum-sized synchronous decode.
    /// Guarantees: The measured parse/build/finalize section and timer lateness stay below 50 ms on an idle CPU.
    #[tokio::test(flavor = "current_thread")]
    #[ignore = "wall-clock check: run in release mode on an otherwise idle pinned CPU"]
    async fn bounded_decode_scheduling_latency() {
        use std::time::{Duration, Instant};

        let mut dense_cef = String::from("CEF:0|Vendor|Product|1|1|event|5|");
        for index in 0..600 {
            use std::fmt::Write;
            write!(dense_cef, " field{index}=value{index}").unwrap();
        }
        assert!(dense_cef.len() <= MAX_MESSAGE_SIZE);
        let raw = vec![b'x'; MAX_MESSAGE_SIZE];
        let small = b"<34>Oct 11 22:14:15 host tag: message";
        let cases = [
            (
                "rfc3164_item_limit",
                unchecked_batch(&vec![small.as_slice(); MAX_BATCH_ITEMS]),
            ),
            ("raw_byte_limit", unchecked_batch(&vec![raw.as_slice(); 15])),
            (
                "cef_byte_limit",
                unchecked_batch(&vec![
                    dense_cef.as_bytes();
                    (MAX_BATCH_BYTES - BATCH_HEADER_LEN - RUN_HEADER_LEN)
                        / (dense_cef.len() + FRAME_HEADER_LEN)
                ]),
            ),
        ];
        let service = otel_arrow_dfe_pdata_codec::CodecService::new().unwrap();
        let codec = resolve_syslog().unwrap();
        for (name, bytes) in cases {
            let mut max_decode = Duration::ZERO;
            let mut max_timer_lateness = Duration::ZERO;
            for _ in 0..24 {
                let payload: OtapPayload =
                    codec.admit(SignalType::Logs, bytes.clone()).unwrap().into();
                let deadline = tokio::time::Instant::now() + Duration::from_millis(1);
                let timer = async {
                    tokio::time::sleep_until(deadline).await;
                    tokio::time::Instant::now().saturating_duration_since(deadline)
                };
                let decode = async {
                    let start = Instant::now();
                    // No synthetic yield inside the operation: measure the actual
                    // synchronous codec path, including Arrow finalization.
                    let records = payload.try_into_otap(&service).unwrap();
                    let elapsed = start.elapsed();
                    let _ = std::hint::black_box(&records);
                    elapsed
                };
                // Poll the timer first so it is armed before decoding starts.
                let (lateness, elapsed) = tokio::join!(biased; timer, decode);
                max_decode = max_decode.max(elapsed);
                max_timer_lateness = max_timer_lateness.max(lateness);
            }
            eprintln!(
                "{name}: max_decode={max_decode:?}, max_timer_lateness={max_timer_lateness:?}"
            );
            assert!(
                max_decode < Duration::from_millis(50),
                "{name}: {max_decode:?}"
            );
            assert!(
                max_timer_lateness < Duration::from_millis(50),
                "{name}: {max_timer_lateness:?}"
            );
        }
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

    /// Scenario: Receipt years change in both directions within a receiver batch.
    /// Guarantees: Equal consecutive years share a run without an offset-list allocation; every year transition and observed timestamp survives sealing.
    #[test]
    fn framing_groups_consecutive_receipt_years() {
        let mut builder = SyslogBatchBuilder::new();
        for _ in 0..2 {
            builder.append_with_year(b"x", 2024).unwrap();
        }
        assert_eq!(builder.additional_run_offsets.capacity(), 0);
        builder.append_with_year(b"x", 2025).unwrap();
        builder.append_with_year(b"x", 2024).unwrap();
        let batch = builder.finish(42).unwrap();
        assert_eq!(
            batch.bytes.len(),
            BATCH_HEADER_LEN + 3 * RUN_HEADER_LEN + 4 * (FRAME_HEADER_LEN + 1)
        );
        let (count, frames) = validated_frames(&batch.bytes).unwrap();
        let frames = frames.collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(count, 4);
        assert_eq!(
            frames
                .iter()
                .map(|frame| frame.received_local_year)
                .collect::<Vec<_>>(),
            [2024, 2024, 2025, 2024]
        );
        assert!(
            frames
                .iter()
                .all(|frame| frame.observed_time == 42 && frame.message == b"x")
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
        assert!(
            predicted_capacity >= BATCH_HEADER_LEN + RUN_HEADER_LEN + (FRAME_HEADER_LEN + 72) * 100
        );
        assert_eq!(
            predicted_frame_capacity(usize::MAX, usize::MAX),
            MAX_PREDICTED_BATCH_CAPACITY - BATCH_HEADER_LEN - RUN_HEADER_LEN
        );
    }

    /// Scenario: Encoded framing is malformed or contains trailing data.
    /// Guarantees: Stateless counting and decoding reject the batch without panicking.
    #[test]
    fn malformed_framing_is_rejected() {
        let complete = batch(&[b"message"], 7);
        // Includes truncation within the run receipt year and frame headers.
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
    /// observed timestamps, and input ownership ranges even when observed timestamps match.
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
        let first = first.finish(observed).unwrap().bytes;
        assert_eq!(
            first.len(),
            BATCH_HEADER_LEN
                + 2 * RUN_HEADER_LEN
                + 3 * FRAME_HEADER_LEN
                + old.len()
                + 2 * new.len()
        );
        assert_eq!(read_u32(&first[16..20]), 1);
        let second_run = BATCH_HEADER_LEN + RUN_HEADER_LEN + FRAME_HEADER_LEN + old.len();
        assert_eq!(read_u32(&first[second_run + 8..second_run + 12]), 2);
        let inputs = vec![first, second.finish(observed).unwrap().bytes];
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
            read_u32(&bytes[BATCH_HEADER_LEN + 8..BATCH_HEADER_LEN + 12]),
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

    /// Scenario: One four-item input exceeds each finite split budget independently.
    /// Guarantees: Fallback retains the original buffer and count, while exact and wide budgets allow decoding every split.
    #[test]
    fn native_batching_honors_each_split_budget() {
        let input = batch(&[b"one", b"two", b"three", b"four"], 11);
        let base = BatchSettings {
            min_size: Some(1),
            max_size: NonZeroU64::new(1),
            ..SYSLOG_BATCH_SETTINGS
        };
        for (fragments, overhead, flush) in [
            (Some(3), None, None),
            (None, Some(71), None),
            (None, None, Some(2)),
            (Some(1), Some(1), Some(1)),
        ] {
            let settings = BatchSettings {
                max_split_fragments: fragments.and_then(NonZeroU64::new),
                max_split_overhead_bytes: overhead.and_then(NonZeroU64::new),
                max_split_fragments_per_flush: flush.and_then(NonZeroU64::new),
                ..base.clone()
            };
            let result = SyslogBatcher
                .batch(SignalType::Logs, &settings, vec![input.clone()])
                .unwrap();
            assert_eq!(result.budget_fallbacks, 1);
            assert!(result.completed_inputs.is_empty());
            assert_eq!(result.batches.len(), 1);
            assert_eq!(result.batches[0].1, 0..=0);
            assert_eq!(result.batches[0].0.as_ptr(), input.as_ptr());
            assert_eq!(result.batches[0].0, input);
            assert_eq!(
                SyslogDecoder
                    .decode(SignalType::Logs, &result.batches[0].0)
                    .unwrap()
                    .num_items(),
                4
            );
        }
        // Four single-item outputs duplicate three 24-byte wrappers. The flush
        // allowance counts the three additional outputs, excluding the whole-input floor.
        for (fragments, overhead, flush) in [(4, 72, 3), (u64::MAX, u64::MAX, u64::MAX)] {
            let settings = BatchSettings {
                max_split_fragments: NonZeroU64::new(fragments),
                max_split_overhead_bytes: NonZeroU64::new(overhead),
                max_split_fragments_per_flush: NonZeroU64::new(flush),
                ..base.clone()
            };
            let result = SyslogBatcher
                .batch(SignalType::Logs, &settings, vec![input.clone()])
                .unwrap();
            assert_eq!(result.budget_fallbacks, 0);
            assert_eq!(result.batches.len(), 4);
            assert_eq!(
                result
                    .batches
                    .iter()
                    .map(|(bytes, _)| bytes.len())
                    .sum::<usize>()
                    - input.len(),
                72
            );
            for (bytes, range) in result.batches {
                assert_eq!(range, 0..=0);
                assert_eq!(
                    SyslogDecoder
                        .decode(SignalType::Logs, &bytes)
                        .unwrap()
                        .num_items(),
                    1
                );
            }
        }
    }

    /// Scenario: Multiple oversized inputs share a flush with whole single-item inputs.
    /// Guarantees: Additional split fan-out is cumulative, whole inputs consume no allowance, and provenance and order survive fallback.
    #[test]
    fn native_batching_bounds_cumulative_split_fanout() {
        let inputs = vec![
            batch(&[b"one", b"two", b"three"], 11),
            batch(&[b"four"], 22),
            batch(&[b"five", b"six", b"seven"], 33),
            batch(&[b"eight"], 44),
        ];
        let settings = BatchSettings {
            min_size: Some(1),
            max_size: NonZeroU64::new(1),
            max_split_fragments_per_flush: NonZeroU64::new(2),
            ..SYSLOG_BATCH_SETTINGS
        };
        let result = SyslogBatcher
            .batch(SignalType::Logs, &settings, inputs.clone())
            .unwrap();
        assert_eq!(result.budget_fallbacks, 1);
        assert_eq!(
            result
                .batches
                .iter()
                .map(|(_, range)| range.clone())
                .collect::<Vec<_>>(),
            vec![0..=0, 0..=0, 0..=0, 1..=1, 2..=2, 3..=3]
        );
        assert_eq!(result.batches[4].0.as_ptr(), inputs[2].as_ptr());
        let mut messages = Vec::new();
        let mut observed = Vec::new();
        for (bytes, _) in &result.batches {
            assert!(SyslogDecoder.decode(SignalType::Logs, bytes).is_ok());
            for frame in validated_frames(bytes).unwrap().1 {
                let frame = frame.unwrap();
                messages.push(frame.message.to_vec());
                observed.push(frame.observed_time);
            }
        }
        assert_eq!(
            messages,
            [
                b"one".as_slice(),
                b"two",
                b"three",
                b"four",
                b"five",
                b"six",
                b"seven",
                b"eight"
            ]
        );
        assert_eq!(observed, [11, 11, 11, 22, 33, 33, 33, 44]);
        assert!(result.completed_inputs.is_empty());
    }

    /// Scenario: A pending partial batch precedes an input that cannot be split within budget.
    /// Guarantees: Fallback emits no partial input, and fitting inputs remain whole without consuming the split allowance.
    #[test]
    fn native_batching_fallback_preserves_pending_input_ownership() {
        let inputs = vec![
            batch(&[b"first"], 11),
            batch(&[b"second", b"third", b"fourth", b"fifth"], 22),
            batch(&[b"last"], 33),
        ];
        let mut settings = BatchSettings {
            min_size: Some(2),
            max_size: NonZeroU64::new(2),
            max_split_fragments: NonZeroU64::new(1),
            ..SYSLOG_BATCH_SETTINGS
        };
        let result = SyslogBatcher
            .batch(SignalType::Logs, &settings, inputs.clone())
            .unwrap();
        assert_eq!(result.budget_fallbacks, 1);
        assert_eq!(
            result
                .batches
                .iter()
                .map(|(_, range)| range.clone())
                .collect::<Vec<_>>(),
            vec![0..=0, 1..=1, 2..=2]
        );
        assert_eq!(result.batches[1].0, inputs[1]);
        assert_eq!(result.batches[1].0.as_ptr(), inputs[1].as_ptr());
        assert_eq!(
            result
                .batches
                .iter()
                .map(|(bytes, _)| validated_item_count(bytes).unwrap())
                .collect::<Vec<_>>(),
            vec![1, 4, 1]
        );

        settings.max_size = NonZeroU64::new(3);
        let result = SyslogBatcher
            .batch(
                SignalType::Logs,
                &settings,
                vec![
                    batch(&[b"one", b"two"], 11),
                    batch(&[b"three", b"four"], 22),
                ],
            )
            .unwrap();
        assert_eq!(result.budget_fallbacks, 0);
        assert_eq!(
            result
                .batches
                .iter()
                .map(|(bytes, range)| (validated_item_count(bytes).unwrap(), range.clone()))
                .collect::<Vec<_>>(),
            vec![(2, 0..=0), (2, 1..=1)]
        );
    }

    /// Scenario: Splitting on existing timestamp-run boundaries duplicates only batch headers.
    /// Guarantees: The overhead budget measures actual framing amplification and includes its exact boundary.
    #[test]
    fn native_batching_measures_actual_split_wrapper_overhead() {
        let mut settings = SYSLOG_BATCH_SETTINGS;
        let merged = SyslogBatcher
            .batch(
                SignalType::Logs,
                &settings,
                vec![
                    batch(&[b"one", b"two"], 11),
                    batch(&[b"three", b"four"], 22),
                ],
            )
            .unwrap();
        let input = merged.batches[0].0.clone();
        settings.min_size = Some(2);
        settings.max_size = NonZeroU64::new(2);
        for (budget, expected_batches, expected_fallbacks) in [(7, 1, 1), (8, 2, 0)] {
            settings.max_split_overhead_bytes = NonZeroU64::new(budget);
            let result = SyslogBatcher
                .batch(SignalType::Logs, &settings, vec![input.clone()])
                .unwrap();
            assert_eq!(result.batches.len(), expected_batches);
            assert_eq!(result.budget_fallbacks, expected_fallbacks);
            for (bytes, _) in &result.batches {
                assert!(SyslogDecoder.decode(SignalType::Logs, bytes).is_ok());
            }
            if expected_fallbacks == 0 {
                assert_eq!(
                    result
                        .batches
                        .iter()
                        .map(|(bytes, _)| bytes.len())
                        .sum::<usize>()
                        - input.len(),
                    8
                );
            }
        }
    }

    /// Scenario: A budget fallback encounters malformed framing or more than the hard item limit.
    /// Guarantees: Invalid or undecodable whole inputs are rejected, valid work recovers, and unrestricted splitting respects the synchronous decode limit.
    #[test]
    fn native_batching_budget_fallback_validates_framing_and_decode_limits() {
        let mut settings = BatchSettings {
            min_size: Some(1),
            max_size: NonZeroU64::new(1),
            max_split_fragments: NonZeroU64::new(1),
            ..SYSLOG_BATCH_SETTINGS
        };
        let valid = batch(&[b"one", b"two"], 11);
        let truncated = valid.slice(..valid.len() - 1);
        let mut batcher = SyslogBatcher;
        assert!(
            batcher
                .batch(SignalType::Logs, &settings, vec![truncated])
                .is_err()
        );
        let mut builder = SyslogBatchBuilder::new();
        for _ in 0..MAX_BATCH_ITEMS {
            builder.append_with_year(b"message", 2024).unwrap();
        }
        let maximum = builder.finish(11).unwrap().bytes;
        let result = batcher
            .batch(SignalType::Logs, &settings, vec![maximum.clone()])
            .unwrap();
        assert_eq!(result.budget_fallbacks, 1);
        assert_eq!(result.batches[0].0.as_ptr(), maximum.as_ptr());
        // Create one more valid frame without changing the native decoder limit.
        let mut oversized = maximum.to_vec();
        oversized[4..8].copy_from_slice(&((MAX_BATCH_ITEMS + 1) as u32).to_be_bytes());
        oversized[16..20].copy_from_slice(&((MAX_BATCH_ITEMS + 1) as u32).to_be_bytes());
        oversized.extend_from_slice(&1_u32.to_be_bytes());
        oversized.push(b'x');
        let oversized = Bytes::from(oversized);
        assert!(
            batcher
                .batch(SignalType::Logs, &settings, vec![oversized.clone()])
                .is_err()
        );
        let recovered = batcher
            .batch(SignalType::Logs, &settings, vec![valid])
            .unwrap();
        assert_eq!(recovered.budget_fallbacks, 1);
        assert_eq!(
            SyslogDecoder
                .decode(SignalType::Logs, &recovered.batches[0].0)
                .unwrap()
                .num_items(),
            2
        );

        settings.max_size = NonZeroU64::new(u64::MAX);
        settings.max_split_fragments = None;
        let split = batcher
            .batch(SignalType::Logs, &settings, vec![oversized])
            .unwrap();
        assert_eq!(split.budget_fallbacks, 0);
        assert_eq!(
            split
                .batches
                .iter()
                .map(|(bytes, _)| validated_item_count(bytes).unwrap())
                .collect::<Vec<_>>(),
            vec![MAX_BATCH_ITEMS, 1]
        );
        assert!(split.batches.iter().all(|(_, range)| range == &(0..=0)));
    }
}
