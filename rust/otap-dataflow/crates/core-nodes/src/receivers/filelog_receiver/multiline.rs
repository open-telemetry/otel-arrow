// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Incremental multiline grouping. See `docs/multiline-grouping.md`.

use super::decoder::{
    DecodeError, DecodeEvent, DecodeStart, DecodedValue, Encoding, OnDecodeError, SourceRange,
    StreamDecoder,
};
use super::framer::{
    LineBody, LineConfig, LineFramer, LineStart, OversizeBehavior, PartialCompletion,
};
use super::multiline_pattern::{
    BoundaryMatcher, BoundaryPattern, PatternError, PatternInput, PatternMode,
};
use thiserror::Error;

mod buffer;
use buffer::{Body, Line};

/// Which physical lines establish a logical record boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroupingMode {
    /// Matching lines begin records; unmatched lines before a start stand alone.
    Start,
    /// Matching lines end records and are included in their bodies.
    End,
}

/// Primitive configuration; clocks, admission, and source identity are caller-owned.
#[derive(Clone, Copy, Debug)]
pub struct MultilineConfig {
    /// Encoding, decode policy, independent line/record bounds, and oversize policy.
    pub line: LineConfig,
    /// Nonzero number of physical lines allowed in a grouped record.
    pub max_lines: u32,
    /// Start or end boundary semantics.
    pub mode: GroupingMode,
}

impl MultilineConfig {
    /// Conservative requested payload peak, including old/new buffer growth.
    /// Excludes inline state, matcher scratch, allocator overhead, and transferred output.
    pub fn payload_peak_bytes(self) -> Result<usize, MultilineError> {
        let raw = self.line.encoding == Encoding::Raw;
        let source_copies = if keeps_raw(self.line) { 1usize } else { 2 };
        let record_copies = if !raw && keeps_raw(self.line) {
            2usize
        } else {
            1
        };
        let payload = self
            .line
            .max_line_bytes
            .checked_mul(source_copies)
            .and_then(|v| v.checked_add(8))
            .and_then(|v| v.checked_add(if raw { 0 } else { self.line.max_line_bytes }))
            .and_then(|v| {
                record_copies
                    .checked_mul(self.line.max_record_bytes)
                    .and_then(|r| v.checked_add(r))
            })
            .and_then(|v| v.checked_mul(4))
            .filter(|v| *v <= isize::MAX as usize)
            .ok_or(MultilineError::InvalidConfig)?;
        Ok(payload)
    }
}

/// Coordinates compatible with checkpoint framing continuation, without Ack authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordContinuation {
    /// Original record start, used with file identity and epoch for correlation.
    pub record_start_offset: u64,
    /// Known record end, or zero for an oversized physical-line scan to LF.
    pub record_end_offset: u64,
    /// Index of the next fragment.
    pub next_fragment_index: u32,
}

/// Caller-authorized same-source construction boundary.
// Keep the small recovery coordinates inline; construction needs no heap allocation.
#[allow(variant_size_differences)]
#[derive(Clone, Copy, Debug)]
pub enum RecordStart {
    /// New stream, including initial BOM processing.
    NewStream,
    /// Clean safe-unit boundary, without BOM probing.
    ResumeAt(u64),
    /// Restart after validating identity, profile, frontier, and source size.
    Continuation {
        /// Committed boundary, not the record's original start.
        offset: u64,
        /// Previously emitted continuation coordinates.
        continuation: RecordContinuation,
    },
}

/// Why a record or fragment became available.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordEnding {
    /// A subsequent matching start line begins a new record.
    StartPattern,
    /// The included final line matched the end pattern.
    EndPattern,
    /// An unmatched line in start mode was emitted independently.
    PatternNotMatched,
    /// The configured physical-line count was reached.
    LineLimit,
    /// Record bytes forced a clean boundary or a split/truncated sequence.
    ByteLimit,
    /// Buffered content precedes a newly detected oversized physical line.
    OversizeLineBoundary,
    /// An oversized physical line is emitted independently.
    OversizeLine,
    /// A recovered known-end continuation reached its stored boundary.
    Continuation,
    /// The caller authorized an EOF-gated idle completion.
    Idle,
    /// The caller confirmed permanent rotation EOF; terminal-unterminated evidence.
    PermanentEof,
}

/// Correlation coordinates; projection adds durable file identity and epoch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordFragment {
    /// Original frame start, stable across restart.
    pub record_start_offset: u64,
    /// Zero-based fragment index.
    pub index: u32,
    /// This fragment reaches the deterministic record boundary.
    pub is_last: bool,
}

/// One owned output. Ranges propose progress; only delivery/checkpointing authorizes it.
#[derive(Debug, Eq, PartialEq)]
pub struct RecordFrame {
    /// Bounded text or exact source bytes; final LF is excluded.
    pub body: LineBody,
    /// Source range represented by the body, excluding BOM and discarded tail.
    pub body_range: SourceRange,
    /// Contiguous source ownership including terminal LF, BOM, and discarded tail.
    pub frame_range: SourceRange,
    /// Completion reason for bounded telemetry and lifecycle evidence.
    pub ending: RecordEnding,
    /// Present on every split fragment, including a recovered final fragment.
    pub fragment: Option<RecordFragment>,
    /// Proposed resume after a nonfinal fragment.
    pub continuation: Option<RecordContinuation>,
    /// Whether source body bytes were intentionally discarded.
    pub truncated: bool,
    /// Discarded source body bytes, excluding the terminal LF.
    pub discarded_source_bytes: u64,
    /// Malformed units in this frame, including a discarded tail.
    pub malformed_units: u64,
}

/// One incremental step with at most one output and no borrowed input retained.
#[derive(Debug)]
#[must_use = "advance by consumed and retain any output"]
pub struct RecordStep {
    /// Fresh source bytes accepted, including decoder lookahead.
    pub consumed: usize,
    /// At most one record or fragment.
    pub frame: Option<RecordFrame>,
    /// Work advanced even if no fresh input was consumed. Drain until false.
    pub advanced: bool,
}

/// One caller-authorized completion step.
#[derive(Debug)]
#[must_use = "retain output and repeat with the same reason until complete"]
pub struct RecordCompletion {
    /// Earlier or explicitly completed output.
    pub frame: Option<RecordFrame>,
    /// This completion operation finished; a recovery sequence may remain pending.
    pub complete: bool,
}

/// Configuration, caller-contract, or terminal framing failure.
#[derive(Clone, Debug, Error)]
pub enum MultilineError {
    /// Invalid bounds, mode, or recovery coordinates.
    #[error("invalid multiline configuration or continuation")]
    InvalidConfig,
    /// The supplied worker cache belongs to another compiled pattern.
    #[error("multiline matcher does not belong to the configured pattern")]
    WrongMatcher,
    /// Caller must finish the active completion with its original reason.
    #[error("multiline completion is already in progress")]
    CompletionInProgress,
    /// Original source decoder error, including offset discontinuity.
    #[error(transparent)]
    Decode(#[from] DecodeError),
    /// Pattern search failed; never interpreted as a nonmatch.
    #[error(transparent)]
    Pattern(#[from] PatternError),
    /// A nonfinal fragment cannot represent its successor index.
    #[error("multiline fragment index overflow")]
    FragmentIndexOverflow,
    /// Bounded buffer allocation failed.
    #[error("multiline buffer allocation failed")]
    Allocation,
    /// Source bytes do not agree with the stored continuation boundary.
    #[error("multiline continuation boundary is inconsistent with source units")]
    ContinuationBoundary,
}

/// A failure can consume input; earlier returned frames remain caller-owned.
#[derive(Clone, Debug, Error)]
#[error("{error} (consumed {consumed} source bytes)")]
pub struct MultilineFailure {
    /// Fresh source bytes accepted by this failing call.
    pub consumed: usize,
    /// Failure reason.
    pub error: MultilineError,
}

#[derive(Clone, Copy, Debug)]
enum LineEnd {
    Lf,
    Partial(PartialCompletion),
}

impl LineEnd {
    fn reason(self) -> Option<RecordEnding> {
        match self {
            Self::Lf => None,
            Self::Partial(PartialCompletion::Idle) => Some(RecordEnding::Idle),
            Self::Partial(PartialCompletion::PermanentEof) => Some(RecordEnding::PermanentEof),
        }
    }
}

#[derive(Debug)]
struct Replay {
    decoder: StreamDecoder,
    position: usize,
    finish: Option<RecordEnding>,
    end: Option<LineEnd>,
}

/// Portable per-reader grouping state. Supply a worker-owned cache to each step.
#[derive(Debug)]
pub struct MultilineFramer {
    config: MultilineConfig,
    pattern: BoundaryPattern,
    decoder: StreamDecoder,
    initial_bom: bool,
    line: Line,
    body: Body,
    lines: u32,
    separator: Option<DecodeEvent>,
    ready: Option<LineEnd>,
    line_match: Option<bool>,
    oversize: bool,
    replay: Option<Replay>,
    pending: Option<DecodeEvent>,
    replay_pending: Option<DecodeEvent>,
    sequence: Option<RecordContinuation>,
    recovering: bool,
    completing: Option<PartialCompletion>,
    terminal: Option<MultilineError>,
}

impl MultilineFramer {
    /// Validates primitive configuration. Caller validates source identity/size and funds memory.
    pub fn new(
        config: MultilineConfig,
        pattern: BoundaryPattern,
        start: RecordStart,
    ) -> Result<Self, MultilineError> {
        let _ = LineFramer::new(config.line, LineStart::ResumeAt(0))
            .map_err(|_| MultilineError::InvalidConfig)?;
        let _ = config.payload_peak_bytes()?;
        if config.max_lines == 0 {
            return Err(MultilineError::InvalidConfig);
        }
        let mode = if config.line.encoding == Encoding::Raw {
            PatternMode::Raw
        } else {
            PatternMode::Text
        };
        if pattern.mode() != mode {
            return Err(MultilineError::InvalidConfig);
        }
        let (offset, decoder_start, sequence, recovering) = match start {
            RecordStart::NewStream => (0, DecodeStart::NewStream, None, false),
            RecordStart::ResumeAt(offset) => (offset, DecodeStart::ResumeAt(offset), None, false),
            RecordStart::Continuation {
                offset,
                continuation: c,
            } => {
                if config.line.oversize != OversizeBehavior::Split
                    || c.record_start_offset >= offset
                    || c.next_fragment_index == 0
                    || (c.record_end_offset != 0 && offset >= c.record_end_offset)
                {
                    return Err(MultilineError::InvalidConfig);
                }
                (offset, DecodeStart::ResumeAt(offset), Some(c), true)
            }
        };
        Ok(Self {
            config,
            pattern,
            decoder: StreamDecoder::new(
                config.line.encoding,
                config.line.on_decode_error,
                decoder_start,
            ),
            initial_bom: matches!(start, RecordStart::NewStream),
            line: Line::new(offset),
            body: Body::new(offset),
            lines: 0,
            separator: None,
            ready: None,
            line_match: None,
            oversize: false,
            replay: None,
            pending: None,
            replay_pending: None,
            sequence,
            recovering,
            completing: None,
            terminal: None,
        })
    }

    /// Accepts one decoded/replayed unit, or makes one bounded state transition.
    pub fn next(
        &mut self,
        offset: u64,
        input: &[u8],
        matcher: &mut BoundaryMatcher,
    ) -> Result<RecordStep, MultilineFailure> {
        self.check(matcher)?;
        if self.completing.is_some() {
            return Err(self.contract(MultilineError::CompletionInProgress));
        }
        if offset != self.next_expected_input_offset() {
            return Err(
                self.contract(MultilineError::Decode(DecodeError::OffsetDiscontinuity {
                    expected: self.next_expected_input_offset(),
                    actual: offset,
                })),
            );
        }
        self.advance(input, matcher)
    }

    /// Caller establishes idle/permanent-EOF eligibility; this method reads no clock.
    pub fn complete_partial(
        &mut self,
        reason: PartialCompletion,
        matcher: &mut BoundaryMatcher,
    ) -> Result<RecordCompletion, MultilineFailure> {
        self.check(matcher)?;
        if self.completing.is_some_and(|r| r != reason) {
            return Err(self.contract(MultilineError::CompletionInProgress));
        }
        self.completing = Some(reason);
        let step = self.advance(&[], matcher)?;
        if step.advanced {
            return Ok(RecordCompletion {
                frame: step.frame,
                complete: false,
            });
        }
        // A known end cannot be shortened by idle/EOF authority. Source-size
        // validation and truncation policy belong to the caller before recovery.
        if self.recovering && self.sequence.is_some_and(|c| c.record_end_offset != 0) {
            self.completing = None;
            return Ok(RecordCompletion {
                frame: None,
                complete: true,
            });
        }
        if let Some(event) = self
            .decoder
            .finish_incomplete_unit()
            .map_err(|f| self.fail(f.error.into(), 0))?
        {
            let frame = self.process_source(event).map_err(|e| self.fail(e, 0))?;
            return Ok(RecordCompletion {
                frame,
                complete: false,
            });
        }
        if !self.oversize && !self.recovering && !self.line.source.is_empty() {
            self.ready = Some(LineEnd::Partial(reason));
            return Ok(RecordCompletion {
                frame: None,
                complete: false,
            });
        }
        let frame = if self.body.has_progress() {
            let ending = LineEnd::Partial(reason).reason().expect("partial reason");
            let frame = self.emit(ending, true).map_err(|e| self.fail(e, 0))?;
            self.line.reset(self.body.frame_start);
            Some(frame)
        } else {
            None
        };
        self.completing = None;
        Ok(RecordCompletion {
            frame,
            complete: true,
        })
    }

    /// Next fresh-input position; may include decoder lookahead beyond output.
    #[must_use]
    pub const fn next_expected_input_offset(&self) -> u64 {
        self.decoder.next_expected_input_offset()
    }

    /// Earliest consumed source byte not yet owned by output, not a committed offset.
    #[must_use]
    pub fn pending_source_start(&self) -> Option<u64> {
        let start = if self.body.has_progress()
            || self.oversize
            || self.recovering
            || self.replay.is_some()
        {
            self.body.frame_start
        } else {
            self.line.start
        };
        (start < self.next_expected_input_offset()).then_some(start)
    }

    /// Owned buffer capacities; add inline storage and worker matcher scratch separately.
    #[must_use]
    pub fn retained_capacity(&self) -> usize {
        self.line.capacity() + self.body.capacity()
    }

    /// Sticky runtime error. Offset, matcher, and completion contract errors do not latch.
    #[must_use]
    pub fn terminal_error(&self) -> Option<&MultilineError> {
        self.terminal.as_ref()
    }

    fn check(&self, matcher: &BoundaryMatcher) -> Result<(), MultilineFailure> {
        if let Some(error) = &self.terminal {
            return Err(self.contract(error.clone()));
        }
        if !matcher.is_for(&self.pattern) {
            return Err(self.contract(MultilineError::WrongMatcher));
        }
        Ok(())
    }
    fn contract(&self, error: MultilineError) -> MultilineFailure {
        MultilineFailure { consumed: 0, error }
    }
    fn fail(&mut self, error: MultilineError, consumed: usize) -> MultilineFailure {
        self.terminal = Some(error.clone());
        MultilineFailure { consumed, error }
    }

    fn advance(
        &mut self,
        input: &[u8],
        matcher: &mut BoundaryMatcher,
    ) -> Result<RecordStep, MultilineFailure> {
        let mut consumed = 0;
        let result = (|| {
            if self.replay.is_some() {
                return self.replay_step().map(|frame| (frame, true));
            }
            if self.ready.is_some() || (self.oversize && !self.line.source.is_empty()) {
                return self.prepare(matcher).map(|frame| (frame, true));
            }
            let event = if let Some(event) = self.pending.take() {
                Some(event)
            } else {
                let input = if let Some(end) = self
                    .sequence
                    .filter(|_| self.recovering)
                    .map(|c| c.record_end_offset)
                    .filter(|e| *e != 0)
                {
                    let remaining = end
                        .checked_sub(self.next_expected_input_offset())
                        .ok_or(MultilineError::ContinuationBoundary)?;
                    &input[..input
                        .len()
                        .min(usize::try_from(remaining).unwrap_or(usize::MAX))]
                } else {
                    input
                };
                let step = self
                    .decoder
                    .next(self.next_expected_input_offset(), input)
                    .map_err(|f| {
                        consumed = f.consumed;
                        MultilineError::Decode(f.error)
                    })?;
                consumed = step.consumed;
                step.event
            };
            if let Some(event) = event {
                return self.process_source(event).map(|frame| (frame, true));
            }
            if self.recovering
                && self.sequence.is_some_and(|c| {
                    c.record_end_offset != 0
                        && self.next_expected_input_offset() == c.record_end_offset
                })
            {
                if let Some(event) = self
                    .decoder
                    .finish_incomplete_unit()
                    .map_err(|f| MultilineError::Decode(f.error))?
                {
                    return self.process_source(event).map(|frame| (frame, true));
                }
                return Err(MultilineError::ContinuationBoundary);
            }
            Ok((None, consumed != 0))
        })();
        match result {
            Ok((frame, advanced)) => Ok(RecordStep {
                consumed,
                frame,
                advanced,
            }),
            Err(error) => Err(self.fail(error, consumed)),
        }
    }

    fn process_source(
        &mut self,
        event: DecodeEvent,
    ) -> Result<Option<RecordFrame>, MultilineError> {
        if self.oversize || self.recovering {
            return self.stream(event);
        }
        if !self.line.fits(event, self.config.line) {
            self.oversize = true;
            self.pending = Some(event);
            return Ok(None);
        }
        self.line.push(event, self.config.line)?;
        if is_lf(event) {
            self.ready = Some(LineEnd::Lf);
        }
        Ok(None)
    }

    fn prepare(
        &mut self,
        matcher: &mut BoundaryMatcher,
    ) -> Result<Option<RecordFrame>, MultilineError> {
        if self.oversize {
            if self.body.has_progress() {
                return self
                    .emit(RecordEnding::OversizeLineBoundary, true)
                    .map(Some);
            }
            // emit resets ordinary grouping state, but does not consume this line.
            self.start_sequence(0);
            self.start_replay(None, None);
            return Ok(None);
        }
        let end = self.ready.expect("ready line");
        let separator = self.separator.map_or(0, |e| {
            if self.config.line.encoding == Encoding::Raw {
                e.source().as_slice().len()
            } else {
                1
            }
        });
        let text_size = self.body.text.len() + separator + self.line.text.len();
        let raw_separator = self.separator.map_or(0, |e| e.source().as_slice().len());
        let raw_size = self.body.raw.len() + raw_separator + self.line.body_bytes;
        let overflow = text_size > self.config.line.max_record_bytes
            || (keeps_raw(self.config.line) && raw_size > self.config.line.max_record_bytes);
        let finish = if overflow {
            self.start_sequence(self.line.end);
            Some(end.reason().unwrap_or(RecordEnding::ByteLimit))
        } else {
            let input = if self.config.line.encoding == Encoding::Raw {
                PatternInput::Raw(&self.line.source)
            } else {
                PatternInput::Text(&self.line.text)
            };
            // Raw matching excludes the terminal LF, like decoded text matching.
            let input = if self.config.line.encoding == Encoding::Raw && matches!(end, LineEnd::Lf)
            {
                PatternInput::Raw(&self.line.source[..self.line.source.len() - 1])
            } else {
                input
            };
            let matched = match self.line_match {
                Some(matched) => matched,
                None => {
                    let matched = matcher.is_match(input)?;
                    self.line_match = Some(matched);
                    matched
                }
            };
            if self.config.mode == GroupingMode::Start && matched && self.body.has_progress() {
                return self.emit(RecordEnding::StartPattern, true).map(Some);
            }
            let finish = match (self.config.mode, matched, self.body.has_progress()) {
                (GroupingMode::Start, false, false) => Some(RecordEnding::PatternNotMatched),
                (GroupingMode::End, true, _) => Some(RecordEnding::EndPattern),
                _ => None,
            };
            end.reason().or(finish)
        };
        self.start_replay(Some(end), finish);
        Ok(None)
    }

    fn start_sequence(&mut self, end: u64) {
        if self.config.line.oversize == OversizeBehavior::Split && self.sequence.is_none() {
            let start = if self.body.has_progress() {
                self.body.frame_start
            } else {
                self.line.start
            };
            self.sequence = Some(RecordContinuation {
                record_start_offset: start,
                record_end_offset: end,
                next_fragment_index: 0,
            });
        }
    }

    fn start_replay(&mut self, end: Option<LineEnd>, finish: Option<RecordEnding>) {
        let start = if self.initial_bom && self.line.start == 0 {
            DecodeStart::NewStream
        } else {
            DecodeStart::ResumeAt(self.line.start)
        };
        self.replay = Some(Replay {
            decoder: StreamDecoder::new(
                self.config.line.encoding,
                self.config.line.on_decode_error,
                start,
            ),
            position: 0,
            finish,
            end,
        });
        self.ready = None;
        self.line_match = None;
    }

    fn replay_step(&mut self) -> Result<Option<RecordFrame>, MultilineError> {
        if let Some(separator) = self.separator.take() {
            if let Some(frame) = self.append(separator)? {
                self.separator = Some(separator);
                return Ok(Some(frame));
            }
            return Ok(None);
        }
        let event = if let Some(event) = self.replay_pending.take() {
            Some(event)
        } else {
            let replay = self.replay.as_mut().expect("active replay");
            let step = replay
                .decoder
                .next(
                    replay.decoder.next_expected_input_offset(),
                    &self.line.source[replay.position..],
                )
                .map_err(|f| MultilineError::Decode(f.error))?;
            replay.position += step.consumed;
            if step.event.is_some() {
                step.event
            } else if step.consumed != 0 {
                return Ok(None);
            } else {
                replay
                    .decoder
                    .finish_incomplete_unit()
                    .map_err(|f| MultilineError::Decode(f.error))?
            }
        };
        if let Some(event) = event {
            if is_lf(event) {
                self.body.frame_end = event.range().end;
                self.separator = Some(event);
                return self.finish_line();
            }
            if let Some(frame) = self.append(event)? {
                self.replay_pending = Some(event);
                return Ok(Some(frame));
            }
            return Ok(None);
        }
        let partial = self.replay.as_ref().expect("active replay").end.is_some();
        if partial {
            self.finish_line()
        } else {
            let end = self.line.end;
            self.line.reset(end);
            self.replay = None;
            Ok(None)
        }
    }

    fn finish_line(&mut self) -> Result<Option<RecordFrame>, MultilineError> {
        let replay = self.replay.take().expect("active replay");
        self.lines += 1;
        let exact = self.body.size(self.config.line) == self.config.line.max_record_bytes;
        let finish = replay.finish.or({
            if exact {
                Some(RecordEnding::ByteLimit)
            } else if self.lines == self.config.max_lines {
                Some(RecordEnding::LineLimit)
            } else {
                None
            }
        });
        let end = self.line.end;
        self.line.reset(end);
        if let Some(reason) = finish {
            self.emit(reason, true).map(Some)
        } else {
            Ok(None)
        }
    }

    fn stream(&mut self, event: DecodeEvent) -> Result<Option<RecordFrame>, MultilineError> {
        let known_end = self.sequence.map_or(0, |c| c.record_end_offset);
        if known_end != 0 && event.range().end > known_end {
            return Err(MultilineError::ContinuationBoundary);
        }
        let last = if known_end != 0 {
            event.range().end == known_end
        } else {
            is_lf(event)
        };
        if last && is_lf(event) {
            self.body.frame_end = event.range().end;
        } else if let Some(frame) = self.append(event)? {
            self.pending = Some(event);
            return Ok(Some(frame));
        }
        if last {
            let reason = if self.recovering {
                RecordEnding::Continuation
            } else {
                RecordEnding::OversizeLine
            };
            let frame = self.emit(reason, true)?;
            self.line.reset(event.range().end);
            Ok(Some(frame))
        } else {
            Ok(None)
        }
    }

    fn body_config(&self) -> LineConfig {
        let mut config = self.config.line;
        if self.oversize
            || (self.recovering && self.sequence.is_some_and(|c| c.record_end_offset == 0))
        {
            config.max_record_bytes = config.max_record_bytes.min(config.max_line_bytes);
        }
        config
    }

    fn append(&mut self, event: DecodeEvent) -> Result<Option<RecordFrame>, MultilineError> {
        let config = self.body_config();
        if self.body.fits(event, config) || self.body.truncated {
            self.body.push(event, config)?;
            return Ok(None);
        }
        match self.config.line.oversize {
            OversizeBehavior::Truncate => {
                self.body.truncated = true;
                self.body.push(event, config)?;
                Ok(None)
            }
            OversizeBehavior::Split => {
                if self.sequence.is_none() {
                    return Err(MultilineError::ContinuationBoundary);
                }
                // A retained LF becomes body data only when another line joins.
                // If it overflows, the next fragment must own that separator;
                // committing through its end here would skip it on restart.
                self.body.frame_end = event.range().start;
                self.emit(RecordEnding::ByteLimit, false).map(Some)
            }
        }
    }

    fn emit(&mut self, ending: RecordEnding, last: bool) -> Result<RecordFrame, MultilineError> {
        let fragment = self.sequence.map(|c| RecordFragment {
            record_start_offset: c.record_start_offset,
            index: c.next_fragment_index,
            is_last: last,
        });
        let continuation = if last {
            None
        } else {
            let mut c = self.sequence.expect("split sequence");
            c.next_fragment_index = c
                .next_fragment_index
                .checked_add(1)
                .ok_or(MultilineError::FragmentIndexOverflow)?;
            Some(c)
        };
        let frame = self
            .body
            .take(self.config.line, ending, fragment, continuation);
        self.sequence = continuation;
        if last {
            self.lines = 0;
            self.separator = None;
            self.recovering = false;
            // A detected oversize line still awaits replay when earlier content emits.
            if ending != RecordEnding::OversizeLineBoundary {
                self.oversize = false;
            }
        }
        Ok(frame)
    }
}

fn keeps_raw(config: LineConfig) -> bool {
    config.encoding == Encoding::Raw || config.on_decode_error == OnDecodeError::PreserveRaw
}
fn is_lf(event: DecodeEvent) -> bool {
    matches!(
        event,
        DecodeEvent::Unit {
            value: DecodedValue::Scalar('\n') | DecodedValue::RawByte(b'\n'),
            ..
        }
    )
}

#[cfg(test)]
mod tests;
