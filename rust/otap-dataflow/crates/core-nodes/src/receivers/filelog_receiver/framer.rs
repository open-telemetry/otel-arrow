// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Incremental physical-line framing. See `docs/framer.md` for caller obligations.

use super::decoder::{
    DecodeError, DecodeEvent, DecodeStart, DecodedValue, Encoding, OnDecodeError, SourceRange,
    StreamDecoder,
};
use thiserror::Error;

// Measured to reduce allocation events; tiny lines retain spare capacity.
// See docs/framer.md, "Resource bounds".
const INITIAL_BODY_CAPACITY: usize = 64;

/// Treatment of a physical line exceeding the configured body bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OversizeBehavior {
    /// Emit all units in bounded fragments through the next LF or authorized completion.
    Split,
    /// Retain the largest safe prefix and validate/discard the remaining body.
    Truncate,
}

/// Primitive settings; receiver configuration validation remains caller-owned.
#[derive(Clone, Copy, Debug)]
pub struct LineConfig {
    /// Explicit source encoding.
    pub encoding: Encoding,
    /// Malformed-input policy, including discarded truncate tails.
    pub on_decode_error: OnDecodeError,
    /// Physical-line body bound.
    pub max_line_bytes: usize,
    /// Emitted body bound; individual lines use the minimum of both bounds.
    pub max_record_bytes: usize,
    /// Behavior when the effective bound is crossed (equality fits).
    pub oversize: OversizeBehavior,
}

/// Scan-to-LF split coordinates, independent of checkpoint encoding and identity.
///
/// Persistence maps this to `record_end_offset == 0`. Known-end multiline
/// continuation is a separate consumer responsibility.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineContinuation {
    /// Original frame start, including an initial stripped BOM.
    pub record_start_offset: u64,
    /// Index of the next fragment to emit.
    pub next_fragment_index: u32,
}

/// Framer construction boundary, authorized by the caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineStart {
    /// Fresh stream at zero with initial BOM handling.
    NewStream,
    /// Clean boundary without BOM probing, including at zero.
    ResumeAt(u64),
    /// Validated same-stream split recovery at a safe source-unit boundary.
    Continuation {
        /// Already committed source boundary; never reread the original prefix.
        offset: u64,
        /// Coordinates previously returned by a nonfinal fragment.
        continuation: LineContinuation,
    },
}

/// Why the physical line or fragment became available.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineEnding {
    /// Terminal decoded/raw LF; its bytes belong only to the frame range.
    Lf,
    /// A following body unit proved this fragment's bound was crossed.
    Split,
    /// Caller established the configured EOF-gated idle deadline.
    Idle,
    /// Caller established permanent rotation EOF; terminal-unterminated evidence.
    PermanentEof,
}

/// Explicit authority to complete an unterminated line and incomplete source unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PartialCompletion {
    /// The enabled EOF-gated idle deadline expired without intervening input.
    Idle,
    /// Rotation established permanent EOF according to the receiver contract.
    PermanentEof,
}

/// Owned payload; byte bodies preserve original source representation.
#[derive(Debug, Eq, PartialEq)]
pub enum LineBody {
    /// Decoded text, possibly containing replacement scalars.
    Text(String),
    /// Raw encoding, malformed preserve-raw content, or preserve-raw split.
    Bytes(Vec<u8>),
}

/// Split metadata used later with file identity/epoch to construct correlation IDs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineFragment {
    /// Original frame start, stable across restart.
    pub record_start_offset: u64,
    /// Zero-based index in this sequence.
    pub index: u32,
    /// Whether LF or explicit partial completion ended the sequence.
    pub is_last: bool,
}

/// One bounded physical line or fragment. No field grants Ack/checkpoint authority.
#[derive(Debug, Eq, PartialEq)]
pub struct LineFrame {
    /// Owned body, excluding the terminal LF and any stripped BOM.
    /// Charge the underlying buffer capacity, which can exceed its body length.
    pub body: LineBody,
    /// Exact body bytes for clean preserve-raw text, needed if later multiline
    /// content changes the grouped representation. Byte bodies need no shadow.
    /// Charge this buffer's capacity separately from the decoded body.
    pub source_body: Option<Vec<u8>>,
    /// Exact source bytes represented by the body.
    pub body_range: SourceRange,
    /// Contiguous source ownership, including BOM, LF and discarded tail as applicable.
    pub frame_range: SourceRange,
    /// Completion evidence for multiline and lifecycle consumers.
    pub ending: LineEnding,
    /// Present on every fragment of an oversized split line.
    pub fragment: Option<LineFragment>,
    /// Resume coordinates after this output, pending later Ack/persistence authorization.
    pub continuation: Option<LineContinuation>,
    /// Whether body bytes were intentionally discarded.
    pub truncated: bool,
    /// Discarded source body bytes, excluding LF.
    pub discarded_source_bytes: u64,
    /// Malformed units in this frame, including discarded units.
    pub malformed_units: u64,
}

/// One bounded step; the caller owns all input beyond `consumed`.
#[derive(Debug)]
#[must_use = "advance input by consumed and retain any returned frame"]
pub struct LineStep {
    /// Fresh input bytes accepted this call (at most the decoder's per-call bound).
    pub consumed: usize,
    /// At most one frame. Stop calling to apply backpressure.
    pub frame: Option<LineFrame>,
    /// True if an event was processed, even with zero consumption and no frame.
    /// Drain empty-input calls until this is false before waiting for more bytes.
    pub advanced: bool,
}

/// One bounded explicit-completion step, consuming no fresh input.
#[derive(Debug)]
#[must_use = "retain output and repeat completion until complete"]
pub struct CompletionStep {
    /// Earlier complete output or the authorized partial output.
    pub frame: Option<LineFrame>,
    /// Completion has drained all state; fresh input may now be supplied.
    pub complete: bool,
}

/// Construction and framing errors.
///
/// Only `CompletionInProgress` and `next`'s input-offset
/// `Decode(OffsetDiscontinuity)` are recoverable at runtime; other errors latch
/// in [`LineFramer::terminal_error`]. Constructor errors create no instance.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum LineError {
    /// Bounds must fit allocation arithmetic and one largest encoding unit.
    #[error("invalid physical-line bounds")]
    InvalidBounds,
    /// Scan-to-LF recovery needs positive progress/index and split behavior.
    #[error("invalid physical-line continuation")]
    InvalidContinuation,
    /// Original decoder error. `next` checks caller offsets before decoding.
    /// Internal errors latch, including `DrainRequired`: the framer drains before
    /// completion, so callers cannot trigger that sequencing error.
    #[error(transparent)]
    Decode(#[from] DecodeError),
    /// Fresh input or a different reason was supplied during explicit completion.
    #[error("finish the active partial completion before supplying more input")]
    CompletionInProgress,
    /// The successor of a nonfinal fragment cannot be represented.
    #[error("physical-line fragment index overflow")]
    FragmentIndexOverflow,
    /// Bounded payload allocation failed.
    #[error("physical-line payload allocation failed")]
    Allocation,
}

/// Failure with exact fresh-input consumption; failed-frame bytes remain unauthorized.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("{error} (consumed {consumed} source bytes in this call)")]
pub struct LineFailure {
    /// Accepted fresh input, including any decoder lookahead.
    pub consumed: usize,
    /// Failure reason.
    pub error: LineError,
}

#[derive(Debug)]
enum State {
    Line,
    Split(LineContinuation),
    Truncate,
}

/// Portable, worker-owned physical-line framer with bounded payload and inline lookahead.
#[derive(Debug)]
pub struct LineFramer {
    decoder: StreamDecoder,
    config: LineConfig,
    limit: usize,
    state: State,
    text: String,
    raw: Vec<u8>,
    body_range: SourceRange,
    frame_start: u64,
    processed_end: u64,
    malformed_units: u64,
    body_malformed: bool,
    pending: Option<DecodeEvent>,
    completing: Option<PartialCompletion>,
    terminal_error: Option<LineError>,
}

impl LineFramer {
    /// Validates bounds and recovery coordinates; does not validate source identity.
    pub fn new(config: LineConfig, start: LineStart) -> Result<Self, LineError> {
        let minimum = match (config.encoding, config.on_decode_error) {
            (Encoding::Raw, _) | (Encoding::Ascii, OnDecodeError::Fail) => 1,
            (Encoding::Ascii, _) => 3,
            _ => 4,
        };
        let limit = config.max_line_bytes.min(config.max_record_bytes);
        // Two shadows plus their simultaneous old/new growth allocations must fit.
        if limit < minimum
            || config.max_line_bytes > isize::MAX as usize
            || config.max_record_bytes > isize::MAX as usize
            || limit.checked_mul(4).is_none()
        {
            return Err(LineError::InvalidBounds);
        }
        let (decode_start, offset, state) = match start {
            LineStart::NewStream => (DecodeStart::NewStream, 0, State::Line),
            LineStart::ResumeAt(offset) => (DecodeStart::ResumeAt(offset), offset, State::Line),
            LineStart::Continuation {
                offset,
                continuation,
            } => {
                if config.oversize != OversizeBehavior::Split
                    || continuation.record_start_offset >= offset
                    || continuation.next_fragment_index == 0
                {
                    return Err(LineError::InvalidContinuation);
                }
                (
                    DecodeStart::ResumeAt(offset),
                    offset,
                    State::Split(continuation),
                )
            }
        };
        Ok(Self {
            decoder: StreamDecoder::new(config.encoding, config.on_decode_error, decode_start),
            config,
            limit,
            state,
            text: String::new(),
            raw: Vec::new(),
            body_range: SourceRange {
                start: offset,
                end: offset,
            },
            frame_start: offset,
            processed_end: offset,
            malformed_units: 0,
            body_malformed: false,
            pending: None,
            completing: None,
            terminal_error: None,
        })
    }

    /// Processes at most one decoded unit or retained overflow unit.
    ///
    /// Empty input drains complete pending work, never completes an incomplete
    /// line. Both success and failure report exact fresh-byte consumption.
    pub fn next(&mut self, input_offset: u64, input: &[u8]) -> Result<LineStep, LineFailure> {
        if let Some(error) = self.terminal_error {
            return Err(LineFailure { consumed: 0, error });
        }
        if self.completing.is_some() {
            return Err(LineFailure {
                consumed: 0,
                error: LineError::CompletionInProgress,
            });
        }
        if input_offset != self.next_expected_input_offset() {
            return Err(LineFailure {
                consumed: 0,
                error: LineError::Decode(DecodeError::OffsetDiscontinuity {
                    expected: self.next_expected_input_offset(),
                    actual: input_offset,
                }),
            });
        }
        self.advance(input)
    }

    /// Completes a caller-authorized boundary after all input is supplied.
    ///
    /// Repeat the same reason until `complete`, retaining every frame. Each call
    /// handles at most one event; LF/split output precedes later incomplete units.
    /// The caller checks the clock and EOF.
    pub fn complete_partial(
        &mut self,
        reason: PartialCompletion,
    ) -> Result<CompletionStep, LineFailure> {
        if let Some(error) = self.terminal_error {
            return Err(LineFailure { consumed: 0, error });
        }
        if self.completing.is_some_and(|active| active != reason) {
            return Err(LineFailure {
                consumed: 0,
                error: LineError::CompletionInProgress,
            });
        }
        self.completing = Some(reason);
        let step = self.advance(&[])?;
        if step.advanced {
            return Ok(CompletionStep {
                frame: step.frame,
                complete: false,
            });
        }
        let event = self
            .decoder
            .finish_incomplete_unit()
            .map_err(|failure| self.failure(LineError::Decode(failure.error), 0))?;
        if let Some(event) = event {
            let frame = self
                .process(event)
                .map_err(|error| self.failure(error, 0))?;
            return Ok(CompletionStep {
                frame,
                complete: false,
            });
        }
        let frame = if self.processed_end > self.frame_start {
            let ending = match reason {
                PartialCompletion::Idle => LineEnding::Idle,
                PartialCompletion::PermanentEof => LineEnding::PermanentEof,
            };
            Some(
                self.emit(self.processed_end, self.processed_end, ending)
                    .map_err(|error| self.failure(error, 0))?,
            )
        } else {
            None
        };
        self.completing = None;
        Ok(CompletionStep {
            frame,
            complete: true,
        })
    }

    /// Next fresh-input position, which may exceed any returned frame's end.
    #[must_use]
    pub const fn next_expected_input_offset(&self) -> u64 {
        self.decoder.next_expected_input_offset()
    }

    /// First source byte not yet owned by returned output, including failed input.
    /// This is volatile framing state, not an applied or durable frontier.
    #[must_use]
    pub fn pending_source_start(&self) -> Option<u64> {
        (self.frame_start < self.next_expected_input_offset()).then_some(self.frame_start)
    }

    /// Retained text/raw heap capacity, excluding returned output. Includes reusable
    /// text storage after byte-body emission, even with no pending source bytes.
    /// Inline state occupies `size_of::<LineFramer>()`.
    #[must_use]
    pub fn retained_capacity(&self) -> usize {
        self.text.capacity() + self.raw.capacity()
    }

    /// Sticky terminal error, if any; reconstruction requires caller authorization.
    #[must_use]
    pub const fn terminal_error(&self) -> Option<LineError> {
        self.terminal_error
    }

    fn advance(&mut self, input: &[u8]) -> Result<LineStep, LineFailure> {
        let (consumed, event) = if let Some(event) = self.pending.take() {
            (0, Some(event))
        } else {
            let step = self
                .decoder
                .next(self.next_expected_input_offset(), input)
                .map_err(|failure| {
                    self.failure(LineError::Decode(failure.error), failure.consumed)
                })?;
            (step.consumed, step.event)
        };
        let frame = match event {
            Some(event) => self
                .process(event)
                .map_err(|error| self.failure(error, consumed))?,
            None => None,
        };
        Ok(LineStep {
            consumed,
            frame,
            advanced: consumed != 0 || event.is_some(),
        })
    }

    fn process(&mut self, event: DecodeEvent) -> Result<Option<LineFrame>, LineError> {
        let DecodeEvent::Unit {
            range,
            source,
            value,
            malformed,
        } = event
        else {
            self.body_range = SourceRange {
                start: event.range().end,
                end: event.range().end,
            };
            self.processed_end = event.range().end;
            return Ok(None);
        };
        if matches!(
            value,
            DecodedValue::Scalar('\n') | DecodedValue::RawByte(b'\n')
        ) {
            return self.emit(range.end, range.start, LineEnding::Lf).map(Some);
        }
        if matches!(self.state, State::Truncate) {
            self.processed_end = range.end;
            // Each malformed unit owns at least one distinct byte of a u64 range.
            self.malformed_units += u64::from(malformed);
            return Ok(None);
        }
        let mut encoded = [0; 4];
        let text = match value {
            DecodedValue::Scalar(c) => c.encode_utf8(&mut encoded),
            DecodedValue::RawByte(_) => "",
        };
        let keep_raw = self.keeps_raw();
        let fits = text.len() <= self.limit - self.text.len()
            && (!keep_raw || source.as_slice().len() <= self.limit - self.raw.len());
        if !fits {
            match self.config.oversize {
                OversizeBehavior::Split => {
                    if matches!(self.state, State::Line) {
                        self.state = State::Split(LineContinuation {
                            record_start_offset: self.frame_start,
                            next_fragment_index: 0,
                        });
                    }
                    // Decoder consumption/delivery may be ahead of this frame boundary.
                    // Keep exactly one unit; never replay it through the live decoder.
                    self.pending = Some(event);
                    return self
                        .emit(range.start, range.start, LineEnding::Split)
                        .map(Some);
                }
                OversizeBehavior::Truncate => {
                    self.state = State::Truncate;
                    self.processed_end = range.end;
                    self.malformed_units += u64::from(malformed);
                    return Ok(None);
                }
            }
        }
        if !text.is_empty() {
            let additional = growth(
                self.text.len(),
                self.text.capacity(),
                text.len(),
                self.limit,
            );
            self.text
                .try_reserve_exact(additional)
                .map_err(|_| LineError::Allocation)?;
            self.text.push_str(text);
        }
        if keep_raw {
            let additional = growth(
                self.raw.len(),
                self.raw.capacity(),
                source.as_slice().len(),
                self.limit,
            );
            self.raw
                .try_reserve_exact(additional)
                .map_err(|_| LineError::Allocation)?;
            self.raw.extend_from_slice(source.as_slice());
        }
        self.body_range.end = range.end;
        self.processed_end = range.end;
        self.malformed_units += u64::from(malformed);
        self.body_malformed |= malformed;
        Ok(None)
    }

    fn keeps_raw(&self) -> bool {
        self.config.encoding == Encoding::Raw
            || self.config.on_decode_error == OnDecodeError::PreserveRaw
    }

    fn emit(
        &mut self,
        end: u64,
        body_end: u64,
        ending: LineEnding,
    ) -> Result<LineFrame, LineError> {
        let is_last = ending != LineEnding::Split;
        let (fragment, continuation) = if let State::Split(coordinates) = self.state {
            let continuation = if is_last {
                None
            } else {
                Some(LineContinuation {
                    record_start_offset: coordinates.record_start_offset,
                    next_fragment_index: coordinates
                        .next_fragment_index
                        .checked_add(1)
                        .ok_or(LineError::FragmentIndexOverflow)?,
                })
            };
            (
                Some(LineFragment {
                    record_start_offset: coordinates.record_start_offset,
                    index: coordinates.next_fragment_index,
                    is_last,
                }),
                continuation,
            )
        } else {
            (None, None)
        };
        let bytes = self.config.encoding == Encoding::Raw
            || (self.config.on_decode_error == OnDecodeError::PreserveRaw
                && (self.body_malformed || fragment.is_some()));
        let (body, source_body) = if bytes {
            self.text.clear();
            (LineBody::Bytes(std::mem::take(&mut self.raw)), None)
        } else {
            let shadow = self.keeps_raw().then(|| std::mem::take(&mut self.raw));
            (LineBody::Text(std::mem::take(&mut self.text)), shadow)
        };
        let frame = LineFrame {
            body,
            source_body,
            body_range: self.body_range,
            frame_range: SourceRange {
                start: self.frame_start,
                end,
            },
            ending,
            fragment,
            continuation,
            truncated: matches!(self.state, State::Truncate),
            discarded_source_bytes: body_end - self.body_range.end,
            malformed_units: self.malformed_units,
        };
        self.frame_start = end;
        self.processed_end = end;
        self.body_range = SourceRange { start: end, end };
        self.malformed_units = 0;
        self.body_malformed = false;
        self.state = continuation.map_or(State::Line, State::Split);
        Ok(frame)
    }

    fn failure(&mut self, error: LineError, consumed: usize) -> LineFailure {
        self.terminal_error = Some(error);
        LineFailure { consumed, error }
    }
}

// Geometric growth capped at the configured bound, including small raw limits.
// Return an addition relative to length, as required by try_reserve_exact.
fn growth(len: usize, capacity: usize, additional: usize, limit: usize) -> usize {
    if additional <= capacity - len {
        return 0;
    }
    let target = (len + additional).max(
        capacity
            .saturating_mul(2)
            .max(INITIAL_BODY_CAPACITY)
            .min(limit),
    );
    target - len
}

#[cfg(test)]
mod tests;
