// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded incremental decoding of original Filelog source bytes.
//!
//! Each call returns at most one source unit, so a caller can stop at a framing
//! boundary before decoding later input. Empty input drains buffered events,
//! never finalizes an incomplete unit. See the sibling README for the complete
//! ownership, malformed-unit grouping, and caller-authorized boundary contract.

use std::{fmt, str};
use thiserror::Error;

/// Explicit source encoding; this decoder never autodetects or switches it.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Encoding {
    /// UTF-8.
    #[default]
    Utf8,
    /// ASCII; bytes above 0x7f are malformed.
    Ascii,
    /// UTF-16 with little-endian source code units.
    Utf16Le,
    /// UTF-16 with big-endian source code units.
    Utf16Be,
    /// Uninterpreted bytes, without validation or BOM handling.
    Raw,
}

/// Decoder-side malformed-input policy, independent of YAML configuration.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OnDecodeError {
    /// Return malformed evidence and a replacement shadow scalar.
    ///
    /// The caller must retain original bytes from the beginning of the frame,
    /// including clean units, to emit the affected frame as exact bytes.
    #[default]
    PreserveRaw,
    /// Return one replacement scalar and exact evidence per malformed unit.
    Replace,
    /// Stop at the earliest malformed unit with a sticky fatal error.
    ///
    /// Quarantine, delivery ordering and durable progress remain caller work.
    Fail,
}

/// A half-open range of offsets in the source byte stream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceRange {
    /// The first source byte in the range.
    pub start: u64,
    /// The first source byte after the range.
    pub end: u64,
}

/// The exact source bytes for one decoded or malformed source unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceBytes {
    bytes: [u8; 4],
    len: u8,
}

impl SourceBytes {
    fn from_slice(source: &[u8]) -> Self {
        debug_assert!(source.len() <= 4);
        let mut bytes = [0; 4];
        bytes[..source.len()].copy_from_slice(source);
        Self {
            bytes,
            len: source.len() as u8,
        }
    }

    fn from_pair(first: &[u8], second: &[u8]) -> Self {
        debug_assert!(first.len() + second.len() <= 4);
        let mut bytes = [0; 4];
        bytes[..first.len()].copy_from_slice(first);
        bytes[first.len()..first.len() + second.len()].copy_from_slice(second);
        Self {
            bytes,
            len: (first.len() + second.len()) as u8,
        }
    }

    /// Returns the exact source bytes represented by this value.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..usize::from(self.len)]
    }
}

impl fmt::Display for SourceBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.as_slice() {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// The decoded shadow value for one source unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(variant_size_differences)]
pub enum DecodedValue {
    /// A decoded Unicode scalar.
    Scalar(char),
    /// One byte from raw mode.
    RawByte(u8),
}

/// One source-ordered decoder event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeEvent {
    /// A decoded source unit, including its exact source evidence.
    Unit {
        /// The source range occupied by the unit.
        range: SourceRange,
        /// The exact bytes occupied by the unit.
        source: SourceBytes,
        /// The decoded value, or U+FFFD for a malformed text unit.
        value: DecodedValue,
        /// Whether the source unit was malformed.
        malformed: bool,
    },
    /// A matching byte-order mark stripped at the start of a new stream.
    StrippedBom {
        /// The source range occupied by the stripped byte-order mark.
        range: SourceRange,
        /// The exact stripped bytes, still owned by the first source frame.
        source: SourceBytes,
    },
}

impl DecodeEvent {
    /// Returns the half-open original-byte range represented by this event.
    #[must_use]
    pub const fn range(self) -> SourceRange {
        match self {
            Self::Unit { range, .. } | Self::StrippedBom { range, .. } => range,
        }
    }

    /// Returns exact original bytes, including for a stripped BOM.
    #[must_use]
    pub const fn source(self) -> SourceBytes {
        match self {
            Self::Unit { source, .. } | Self::StrippedBom { source, .. } => source,
        }
    }
}

/// Progress and at most one event produced by a decoder call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "retain the event and advance caller input by the consumed prefix"]
pub struct DecodeStep {
    /// Bytes consumed from the input slice supplied to this call.
    pub consumed: usize,
    /// The next source-ordered event, when one became available.
    pub event: Option<DecodeEvent>,
}

/// A decoder error, without a per-call consumption count.
///
/// Malformed input under `fail` and source offset overflow are terminal.
/// Discontinuity and drain-required errors are nonterminal caller errors.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DecodeError {
    /// The caller did not supply the next expected source offset.
    #[error("decoder input offset discontinuity: expected {expected}, got {actual}")]
    OffsetDiscontinuity {
        /// The next offset expected by the decoder.
        expected: u64,
        /// The offset supplied by the caller.
        actual: u64,
    },
    /// A source range could not be represented by `u64` offsets.
    #[error("source offset overflow")]
    SourceOffsetOverflow,
    /// Buffered replay or UTF-16 lookahead must be drained before completion.
    #[error("drain decoder with empty input before resolving an incomplete unit")]
    DrainRequired,
    /// The configured fail policy rejected an exact malformed source unit.
    #[error("malformed source unit at {range:?}: {source_bytes}")]
    FatalMalformed {
        /// The exact range of the malformed source unit.
        range: SourceRange,
        /// The exact bytes of the malformed source unit.
        source_bytes: SourceBytes,
    },
}

/// A failed call and its exact caller-input consumption.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("{error} (consumed {consumed} source bytes in this call)")]
pub struct DecodeFailure {
    /// Bytes consumed from this call's input, even when it fails.
    ///
    /// These bytes are not necessarily delivered, framed or checkpointable.
    pub consumed: usize,
    /// The failure reason and, for malformed input, exact source evidence.
    #[source]
    pub error: DecodeError,
}

/// A constant-state, one-event-at-a-time decoder for a single source stream.
///
/// Owns no heap allocations or input borrows. Keep this object across empty
/// reads, scheduling pauses and descriptor eviction. Construct a new object
/// only for an explicitly new stream or caller-authorized recovery position.
#[derive(Debug)]
pub struct StreamDecoder {
    policy: OnDecodeError,
    state: DecoderState,
    bom_probe: Option<BomProbe>,
    replay: ReplayBytes,
    next_input_offset: u64,
    delivered_boundary: u64,
    terminal_error: Option<DecodeError>,
}

impl StreamDecoder {
    /// Maximum fresh input bytes consumed by one call, including lookahead.
    pub const MAX_INPUT_BYTES_PER_CALL: usize = 4;

    /// Creates a decoder at `source_offset`.
    ///
    /// Byte-order-mark probing is enabled only when `new_stream_start` is
    /// true and `source_offset` is zero.
    ///
    /// At any other position, decoding starts exactly at the supplied offset:
    /// there is no alignment, rewind or search for a character boundary. The
    /// caller must establish a safe resume boundary or intentional exclusion.
    #[must_use]
    pub fn new(
        encoding: Encoding,
        policy: OnDecodeError,
        source_offset: u64,
        new_stream_start: bool,
    ) -> Self {
        let state = match encoding {
            Encoding::Utf8 => DecoderState::Utf8(Utf8State::default()),
            Encoding::Ascii => DecoderState::Ascii,
            Encoding::Utf16Le => DecoderState::Utf16(Utf16State::new(Endian::Little)),
            Encoding::Utf16Be => DecoderState::Utf16(Utf16State::new(Endian::Big)),
            Encoding::Raw => DecoderState::Raw,
        };
        let bom_probe = (new_stream_start && source_offset == 0 && encoding != Encoding::Raw)
            .then(|| BomProbe::new(source_offset, encoding));

        Self {
            policy,
            state,
            bom_probe,
            replay: ReplayBytes::default(),
            next_input_offset: source_offset,
            delivered_boundary: source_offset,
            terminal_error: None,
        }
    }

    /// Decodes until one event is available or more input is needed.
    ///
    /// Advances by at most [`Self::MAX_INPUT_BYTES_PER_CALL`] fresh bytes and
    /// replays at most three already-owned BOM bytes. A following byte/code unit
    /// may be inspected to classify an earlier malformed unit, but its event
    /// cannot overtake the earlier unit.
    ///
    /// Advance the caller's slice and offset by `consumed` on both success and
    /// failure. The remaining slice stays caller-owned. Consumed input can
    /// precede delivery; drain empty-input calls before an incomplete-boundary
    /// decision. A nonempty call either consumes input, returns an event, or
    /// fails; an empty no-event result requests more input, not terminal policy.
    ///
    /// After a terminal error, all decoding/completion calls repeat that error
    /// with zero consumption and no state change. Getters remain usable.
    #[inline]
    pub fn next(&mut self, input_offset: u64, input: &[u8]) -> Result<DecodeStep, DecodeFailure> {
        if let Some(error) = self.terminal_error {
            return Err(DecodeFailure { consumed: 0, error });
        }
        if input_offset != self.next_input_offset {
            return Err(DecodeFailure {
                consumed: 0,
                error: DecodeError::OffsetDiscontinuity {
                    expected: self.next_input_offset,
                    actual: input_offset,
                },
            });
        }

        // Avoid cursor/replay dispatch for complete one-byte units. Pending
        // prefixes always use the general path, even when the next byte is ASCII.
        if let Some(&byte) = input
            .first()
            .filter(|_| self.bom_probe.is_none() && self.replay.pos == self.replay.len)
        {
            let value = match &self.state {
                DecoderState::Raw => Some(DecodedValue::RawByte(byte)),
                DecoderState::Ascii if byte.is_ascii() => {
                    Some(DecodedValue::Scalar(char::from(byte)))
                }
                DecoderState::Utf8(state) if state.len == 0 && byte.is_ascii() => {
                    Some(DecodedValue::Scalar(char::from(byte)))
                }
                _ => None,
            };
            if let Some(value) = value {
                let range =
                    one_byte_range(input_offset).map_err(|error| self.record_failure(error, 0))?;
                self.next_input_offset = range.end;
                self.delivered_boundary = range.end;
                return Ok(DecodeStep {
                    consumed: 1,
                    event: Some(DecodeEvent::Unit {
                        range,
                        source: SourceBytes::from_slice(&[byte]),
                        value,
                        malformed: false,
                    }),
                });
            }
        }

        let mut consumed = 0;
        match self.next_inner(input, &mut consumed) {
            Ok(event) => {
                if let Some(event) = event {
                    debug_assert_eq!(event.range().start, self.delivered_boundary);
                    self.delivered_boundary = event.range().end;
                }
                debug_assert!(consumed <= Self::MAX_INPUT_BYTES_PER_CALL);
                Ok(DecodeStep { consumed, event })
            }
            Err(error) => Err(self.record_failure(error, consumed)),
        }
    }

    /// Returns the source offset expected for the next caller-owned input byte.
    #[must_use]
    pub const fn next_expected_input_offset(&self) -> u64 {
        self.next_input_offset
    }

    /// Returns the highest source boundary represented by an event already
    /// returned to the caller.
    ///
    /// The decoder can consume bytes before returning all events they
    /// produce. To determine whether all consumed input is encoding-complete,
    /// drain [`Self::next`] with empty input, then inspect
    /// [`Self::pending_source_start`]. Neither frontier grants checkpoint
    /// permission, even when they are equal.
    #[must_use]
    pub const fn highest_delivered_source_boundary(&self) -> u64 {
        self.delivered_boundary
    }

    /// Returns the start of source bytes consumed but not yet represented by
    /// a returned event.
    ///
    /// Pending bytes can be either an incomplete encoding unit or a complete
    /// internally buffered event. Drain empty-input events before deciding
    /// that more source bytes are required.
    ///
    /// After terminal failure, this also covers the failed unit and any
    /// consumed lookahead, neither of which has been successfully delivered.
    #[must_use]
    pub const fn pending_source_start(&self) -> Option<u64> {
        if self.delivered_boundary < self.next_input_offset {
            Some(self.delivered_boundary)
        } else {
            None
        }
    }

    /// Returns the sticky terminal error, if decoding has stopped.
    #[must_use]
    pub const fn terminal_error(&self) -> Option<DecodeError> {
        self.terminal_error
    }

    /// Applies the configured decode policy to the one incomplete source
    /// unit at a caller-established eligible idle or permanent-EOF boundary.
    ///
    /// The caller must first establish eligibility, process all earlier input
    /// and drain [`Self::next`] with empty input until no event remains.
    /// Undrained BOM replay or a queued UTF-16 code unit returns
    /// [`DecodeError::DrainRequired`] without changing state. A queued high
    /// surrogate also needs draining, even if it then becomes an incomplete tail.
    ///
    /// Ordinary live EOF, scheduling pauses and descriptor closure must not call
    /// this operation. Under preserve/replace it returns one malformed event
    /// owning the entire incomplete range, including a high surrogate plus an
    /// odd byte of its unfinished pair. Later bytes start a fresh source unit;
    /// they cannot complete the resolved tail. BOM detection is not re-enabled.
    ///
    /// No pending unit returns `None`. Fail policy latches the exact fatal
    /// range without advancing the delivered boundary. This method consumes
    /// no new input and never grants framing or checkpoint permission.
    pub fn finish_incomplete_unit(&mut self) -> Result<Option<DecodeEvent>, DecodeFailure> {
        if let Some(error) = self.terminal_error {
            return Err(DecodeFailure { consumed: 0, error });
        }
        if self.replay.pos != self.replay.len
            || matches!(&self.state, DecoderState::Utf16(state) if state.queued.is_some())
        {
            return Err(DecodeFailure {
                consumed: 0,
                error: DecodeError::DrainRequired,
            });
        }
        self.finish_incomplete_inner()
            .map_err(|error| self.record_failure(error, 0))
    }

    #[cold]
    fn record_failure(&mut self, error: DecodeError, consumed: usize) -> DecodeFailure {
        if matches!(
            error,
            DecodeError::FatalMalformed { .. } | DecodeError::SourceOffsetOverflow
        ) {
            self.terminal_error = Some(error);
        }
        DecodeFailure { consumed, error }
    }

    fn finish_incomplete_inner(&mut self) -> Result<Option<DecodeEvent>, DecodeError> {
        let pending = if let Some(probe) = self.bom_probe.as_ref() {
            (!probe.as_slice().is_empty()).then(|| {
                (
                    SourceRange {
                        start: probe.start,
                        end: self.next_input_offset,
                    },
                    SourceBytes::from_slice(probe.as_slice()),
                )
            })
        } else {
            match &self.state {
                DecoderState::Utf8(state) if state.len != 0 => Some((
                    SourceRange {
                        start: state.start,
                        end: self.next_input_offset,
                    },
                    SourceBytes::from_slice(&state.bytes[..usize::from(state.len)]),
                )),
                DecoderState::Utf16(state) => match (state.high, state.odd) {
                    (Some(high), Some((_, odd))) => Some((
                        SourceRange {
                            start: high.start,
                            end: self.next_input_offset,
                        },
                        SourceBytes::from_pair(&high.bytes, &[odd]),
                    )),
                    (Some(high), None) => Some((
                        SourceRange {
                            start: high.start,
                            end: high.end,
                        },
                        SourceBytes::from_slice(&high.bytes),
                    )),
                    (None, Some((start, odd))) => Some((
                        SourceRange {
                            start,
                            end: self.next_input_offset,
                        },
                        SourceBytes::from_slice(&[odd]),
                    )),
                    (None, None) => None,
                },
                DecoderState::Utf8(_) | DecoderState::Ascii | DecoderState::Raw => None,
            }
        };
        let Some((range, source)) = pending else {
            return Ok(None);
        };
        let event = malformed_event(self.policy, range, source)?;
        self.bom_probe = None;
        match &mut self.state {
            DecoderState::Utf8(state) => state.len = 0,
            DecoderState::Utf16(state) => {
                state.odd = None;
                state.high = None;
                state.queued = None;
            }
            DecoderState::Ascii | DecoderState::Raw => {}
        }
        self.delivered_boundary = range.end;
        Ok(Some(event))
    }

    fn next_inner(
        &mut self,
        input: &[u8],
        consumed: &mut usize,
    ) -> Result<Option<DecodeEvent>, DecodeError> {
        if self.bom_probe.is_some() {
            let progress = {
                let mut cursor = ByteCursor {
                    replay: &mut self.replay,
                    next_input_offset: &mut self.next_input_offset,
                    input,
                    consumed,
                };
                process_bom_probe(&mut self.bom_probe, self.policy, &mut cursor)?
            };
            match progress {
                BomProgress::Pending => return Ok(None),
                BomProgress::Event(event) => return Ok(Some(event)),
                BomProgress::Continue => {}
            }
        }

        let mut cursor = ByteCursor {
            replay: &mut self.replay,
            next_input_offset: &mut self.next_input_offset,
            input,
            consumed,
        };
        match &mut self.state {
            DecoderState::Utf8(state) => decode_utf8(state, self.policy, &mut cursor),
            DecoderState::Ascii => decode_ascii(self.policy, &mut cursor),
            DecoderState::Utf16(state) => decode_utf16(state, self.policy, &mut cursor),
            DecoderState::Raw => decode_raw(&mut cursor),
        }
    }
}

#[derive(Debug)]
enum DecoderState {
    Utf8(Utf8State),
    Ascii,
    Utf16(Utf16State),
    Raw,
}

#[derive(Debug, Default)]
struct Utf8State {
    bytes: [u8; 4],
    len: u8,
    start: u64,
}

#[derive(Clone, Copy, Debug)]
enum Endian {
    Little,
    Big,
}

impl Endian {
    const fn decode(self, bytes: [u8; 2]) -> u16 {
        match self {
            Self::Little => u16::from_le_bytes(bytes),
            Self::Big => u16::from_be_bytes(bytes),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Utf16Unit {
    start: u64,
    end: u64,
    bytes: [u8; 2],
    value: u16,
}

#[derive(Debug)]
struct Utf16State {
    endian: Endian,
    odd: Option<(u64, u8)>,
    high: Option<Utf16Unit>,
    queued: Option<Utf16Unit>,
}

impl Utf16State {
    const fn new(endian: Endian) -> Self {
        Self {
            endian,
            odd: None,
            high: None,
            queued: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum ByteOrigin {
    Replay,
    Input,
}

#[derive(Clone, Copy, Debug)]
struct NextByte {
    byte: u8,
    offset: u64,
    origin: ByteOrigin,
}

struct ByteCursor<'a> {
    replay: &'a mut ReplayBytes,
    next_input_offset: &'a mut u64,
    input: &'a [u8],
    consumed: &'a mut usize,
}

impl ByteCursor<'_> {
    fn peek(&self) -> Result<Option<NextByte>, DecodeError> {
        if let Some((byte, offset)) = self.replay.peek()? {
            return Ok(Some(NextByte {
                byte,
                offset,
                origin: ByteOrigin::Replay,
            }));
        }
        Ok(self
            .input
            .get(*self.consumed)
            .copied()
            .map(|byte| NextByte {
                byte,
                offset: *self.next_input_offset,
                origin: ByteOrigin::Input,
            }))
    }

    fn consume(&mut self, next: NextByte) -> Result<(), DecodeError> {
        match next.origin {
            ByteOrigin::Replay => self.replay.consume(),
            ByteOrigin::Input => {
                let next_offset = self
                    .next_input_offset
                    .checked_add(1)
                    .ok_or(DecodeError::SourceOffsetOverflow)?;
                *self.next_input_offset = next_offset;
                *self.consumed += 1;
            }
        }
        Ok(())
    }

    fn take(&mut self) -> Result<Option<NextByte>, DecodeError> {
        let Some(next) = self.peek()? else {
            return Ok(None);
        };
        self.consume(next)?;
        Ok(Some(next))
    }
}

#[derive(Debug, Default)]
struct ReplayBytes {
    bytes: [u8; 3],
    start: u64,
    len: u8,
    pos: u8,
}

impl ReplayBytes {
    fn load(&mut self, start: u64, bytes: &[u8]) {
        debug_assert!(bytes.len() <= self.bytes.len());
        debug_assert!(self.pos == self.len);
        self.bytes[..bytes.len()].copy_from_slice(bytes);
        self.start = start;
        self.len = bytes.len() as u8;
        self.pos = 0;
    }

    fn peek(&self) -> Result<Option<(u8, u64)>, DecodeError> {
        if self.pos == self.len {
            return Ok(None);
        }
        let offset = self
            .start
            .checked_add(u64::from(self.pos))
            .ok_or(DecodeError::SourceOffsetOverflow)?;
        Ok(Some((self.bytes[usize::from(self.pos)], offset)))
    }

    fn consume(&mut self) {
        debug_assert!(self.pos < self.len);
        self.pos += 1;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BomKind {
    Utf8,
    Utf16Le,
    Utf16Be,
}

impl BomKind {
    const ALL: [Self; 3] = [Self::Utf8, Self::Utf16Le, Self::Utf16Be];

    const fn bytes(self) -> &'static [u8] {
        match self {
            Self::Utf8 => &[0xef, 0xbb, 0xbf],
            Self::Utf16Le => &[0xff, 0xfe],
            Self::Utf16Be => &[0xfe, 0xff],
        }
    }

    const fn matches(self, encoding: Encoding) -> bool {
        matches!(
            (self, encoding),
            (Self::Utf8, Encoding::Utf8)
                | (Self::Utf16Le, Encoding::Utf16Le)
                | (Self::Utf16Be, Encoding::Utf16Be)
        )
    }
}

#[derive(Debug)]
struct BomProbe {
    bytes: [u8; 3],
    len: u8,
    start: u64,
    encoding: Encoding,
}

impl BomProbe {
    const fn new(start: u64, encoding: Encoding) -> Self {
        Self {
            bytes: [0; 3],
            len: 0,
            start,
            encoding,
        }
    }

    fn as_slice(&self) -> &[u8] {
        &self.bytes[..usize::from(self.len)]
    }

    fn push(&mut self, byte: u8) {
        debug_assert!(usize::from(self.len) < self.bytes.len());
        self.bytes[usize::from(self.len)] = byte;
        self.len += 1;
    }
}

enum BomProgress {
    Pending,
    Continue,
    Event(DecodeEvent),
}

fn process_bom_probe(
    probe: &mut Option<BomProbe>,
    policy: OnDecodeError,
    cursor: &mut ByteCursor<'_>,
) -> Result<BomProgress, DecodeError> {
    loop {
        let Some(next) = cursor.take()? else {
            return Ok(BomProgress::Pending);
        };
        let active = probe
            .as_mut()
            .expect("BOM processing requires an active probe");
        active.push(next.byte);

        if let Some(kind) = BomKind::ALL
            .into_iter()
            .find(|kind| kind.bytes() == active.as_slice())
        {
            let active = probe.take().expect("the active BOM probe is present");
            let range = SourceRange {
                start: active.start,
                end: *cursor.next_input_offset,
            };
            let source = SourceBytes::from_slice(active.as_slice());
            if kind.matches(active.encoding) {
                return Ok(BomProgress::Event(DecodeEvent::StrippedBom {
                    range,
                    source,
                }));
            }
            return malformed_event(policy, range, source).map(BomProgress::Event);
        }

        if BomKind::ALL
            .into_iter()
            .any(|kind| kind.bytes().starts_with(active.as_slice()))
        {
            continue;
        }

        let active = probe.take().expect("the active BOM probe is present");
        cursor.replay.load(active.start, active.as_slice());
        return Ok(BomProgress::Continue);
    }
}

fn decode_utf8(
    state: &mut Utf8State,
    policy: OnDecodeError,
    cursor: &mut ByteCursor<'_>,
) -> Result<Option<DecodeEvent>, DecodeError> {
    loop {
        let Some(next) = cursor.peek()? else {
            return Ok(None);
        };
        let pending_len = usize::from(state.len);
        let mut candidate = state.bytes;
        candidate[pending_len] = next.byte;
        let candidate_len = pending_len + 1;

        enum Decision {
            Scalar(char),
            Incomplete,
            Malformed(usize),
        }

        let decision = match str::from_utf8(&candidate[..candidate_len]) {
            Ok(valid) => Decision::Scalar(
                valid
                    .chars()
                    .next()
                    .expect("a UTF-8 candidate always contains one byte"),
            ),
            Err(error) => match error.error_len() {
                Some(len) => Decision::Malformed(len),
                None => Decision::Incomplete,
            },
        };

        match decision {
            Decision::Scalar(value) => {
                cursor.consume(next)?;
                let start = if state.len == 0 {
                    next.offset
                } else {
                    state.start
                };
                let source = SourceBytes::from_slice(&candidate[..candidate_len]);
                state.len = 0;
                let range = SourceRange {
                    start,
                    end: next
                        .offset
                        .checked_add(1)
                        .ok_or(DecodeError::SourceOffsetOverflow)?,
                };
                return Ok(Some(DecodeEvent::Unit {
                    range,
                    source,
                    value: DecodedValue::Scalar(value),
                    malformed: false,
                }));
            }
            Decision::Incomplete => {
                cursor.consume(next)?;
                if state.len == 0 {
                    state.start = next.offset;
                }
                state.bytes[pending_len] = next.byte;
                state.len += 1;
            }
            Decision::Malformed(error_len) if error_len == candidate_len => {
                cursor.consume(next)?;
                let start = if state.len == 0 {
                    next.offset
                } else {
                    state.start
                };
                state.len = 0;
                let range = SourceRange {
                    start,
                    end: next
                        .offset
                        .checked_add(1)
                        .ok_or(DecodeError::SourceOffsetOverflow)?,
                };
                let source = SourceBytes::from_slice(&candidate[..error_len]);
                return malformed_event(policy, range, source).map(Some);
            }
            Decision::Malformed(error_len) => {
                debug_assert_eq!(error_len, pending_len);
                let start = state.start;
                state.len = 0;
                let end = start
                    .checked_add(error_len as u64)
                    .ok_or(DecodeError::SourceOffsetOverflow)?;
                let range = SourceRange { start, end };
                let source = SourceBytes::from_slice(&candidate[..error_len]);
                return malformed_event(policy, range, source).map(Some);
            }
        }
    }
}

fn decode_ascii(
    policy: OnDecodeError,
    cursor: &mut ByteCursor<'_>,
) -> Result<Option<DecodeEvent>, DecodeError> {
    let Some(next) = cursor.take()? else {
        return Ok(None);
    };
    let range = one_byte_range(next.offset)?;
    let source = SourceBytes::from_slice(&[next.byte]);
    if next.byte <= 0x7f {
        Ok(Some(DecodeEvent::Unit {
            range,
            source,
            value: DecodedValue::Scalar(char::from(next.byte)),
            malformed: false,
        }))
    } else {
        malformed_event(policy, range, source).map(Some)
    }
}

fn decode_utf16(
    state: &mut Utf16State,
    policy: OnDecodeError,
    cursor: &mut ByteCursor<'_>,
) -> Result<Option<DecodeEvent>, DecodeError> {
    loop {
        let Some(unit) = next_utf16_unit(state, cursor)? else {
            return Ok(None);
        };

        if let Some(high) = state.high.take() {
            if is_low_surrogate(unit.value) {
                let high_value = u32::from(high.value - 0xd800);
                let low_value = u32::from(unit.value - 0xdc00);
                let scalar = 0x1_0000 + (high_value << 10) + low_value;
                let value =
                    char::from_u32(scalar).expect("a UTF-16 surrogate pair is a Unicode scalar");
                return Ok(Some(DecodeEvent::Unit {
                    range: SourceRange {
                        start: high.start,
                        end: unit.end,
                    },
                    source: SourceBytes::from_pair(&high.bytes, &unit.bytes),
                    value: DecodedValue::Scalar(value),
                    malformed: false,
                }));
            }

            state.queued = Some(unit);
            let range = SourceRange {
                start: high.start,
                end: high.end,
            };
            let source = SourceBytes::from_slice(&high.bytes);
            return malformed_event(policy, range, source).map(Some);
        }

        if is_high_surrogate(unit.value) {
            state.high = Some(unit);
            continue;
        }
        if is_low_surrogate(unit.value) {
            let range = SourceRange {
                start: unit.start,
                end: unit.end,
            };
            let source = SourceBytes::from_slice(&unit.bytes);
            return malformed_event(policy, range, source).map(Some);
        }

        let value =
            char::from_u32(u32::from(unit.value)).expect("a non-surrogate u16 is a Unicode scalar");
        return Ok(Some(DecodeEvent::Unit {
            range: SourceRange {
                start: unit.start,
                end: unit.end,
            },
            source: SourceBytes::from_slice(&unit.bytes),
            value: DecodedValue::Scalar(value),
            malformed: false,
        }));
    }
}

fn next_utf16_unit(
    state: &mut Utf16State,
    cursor: &mut ByteCursor<'_>,
) -> Result<Option<Utf16Unit>, DecodeError> {
    if let Some(unit) = state.queued.take() {
        return Ok(Some(unit));
    }

    if state.odd.is_none() {
        let Some(first) = cursor.take()? else {
            return Ok(None);
        };
        state.odd = Some((first.offset, first.byte));
    }

    let Some(second) = cursor.take()? else {
        return Ok(None);
    };
    let (start, first) = state
        .odd
        .take()
        .expect("a UTF-16 second byte requires a first byte");
    let bytes = [first, second.byte];
    let end = second
        .offset
        .checked_add(1)
        .ok_or(DecodeError::SourceOffsetOverflow)?;
    Ok(Some(Utf16Unit {
        start,
        end,
        bytes,
        value: state.endian.decode(bytes),
    }))
}

fn decode_raw(cursor: &mut ByteCursor<'_>) -> Result<Option<DecodeEvent>, DecodeError> {
    let Some(next) = cursor.take()? else {
        return Ok(None);
    };
    Ok(Some(DecodeEvent::Unit {
        range: one_byte_range(next.offset)?,
        source: SourceBytes::from_slice(&[next.byte]),
        value: DecodedValue::RawByte(next.byte),
        malformed: false,
    }))
}

const fn is_high_surrogate(value: u16) -> bool {
    value >= 0xd800 && value <= 0xdbff
}

const fn is_low_surrogate(value: u16) -> bool {
    value >= 0xdc00 && value <= 0xdfff
}

fn one_byte_range(start: u64) -> Result<SourceRange, DecodeError> {
    let end = start
        .checked_add(1)
        .ok_or(DecodeError::SourceOffsetOverflow)?;
    Ok(SourceRange { start, end })
}

fn malformed_event(
    policy: OnDecodeError,
    range: SourceRange,
    source: SourceBytes,
) -> Result<DecodeEvent, DecodeError> {
    match policy {
        OnDecodeError::PreserveRaw | OnDecodeError::Replace => Ok(DecodeEvent::Unit {
            range,
            source,
            value: DecodedValue::Scalar('\u{fffd}'),
            malformed: true,
        }),
        OnDecodeError::Fail => Err(DecodeError::FatalMalformed {
            range,
            source_bytes: source,
        }),
    }
}

#[cfg(test)]
#[path = "decoder/tests.rs"]
mod tests;
