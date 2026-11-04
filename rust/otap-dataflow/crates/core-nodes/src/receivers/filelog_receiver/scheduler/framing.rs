// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::receivers::filelog_receiver::{
    framer::{LineConfig, LineError, LineFrame, LineFramer, LineStart},
    multiline::{MultilineConfig, MultilineError, MultilineFramer, RecordFrame, RecordStart},
    multiline_pattern::{BoundaryMatcher, BoundaryPattern},
};

#[derive(Clone, Debug)]
pub(super) enum FramingConfig {
    Line(LineConfig),
    Multiline(MultilineConfig, BoundaryPattern),
}

impl FramingConfig {
    fn history_bytes(&self) -> Result<usize, Fault> {
        // Only multiline can retain a whole lookahead line beyond the emitted
        // boundary. UTF-16 source uses at most two bytes per decoded line byte.
        let line = match self {
            Self::Line(_) => 0,
            Self::Multiline(c, _) => c.line.max_line_bytes,
        };
        line.checked_mul(2)
            .and_then(|n| n.checked_add(136))
            .ok_or(Fault::Configuration)
    }

    pub(super) fn slot_bytes(&self, read_bytes: usize) -> Result<usize, Fault> {
        let payload = match self {
            Self::Line(c) => c.payload_peak_bytes().map_err(|_| Fault::Configuration)?,
            Self::Multiline(c, _) => c.payload_peak_bytes().map_err(|_| Fault::Configuration)?,
        };
        payload
            .checked_add(read_bytes)
            .and_then(|n| n.checked_add(self.history_bytes().ok()?))
            .and_then(|n| n.checked_add(size_of::<Partial>()))
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or(Fault::Configuration)
    }

    pub(super) fn create(
        &self,
        offset: u64,
        initial: bool,
        window: &Window,
        read_bytes: usize,
    ) -> Result<Partial, Fault> {
        let framer = match self {
            Self::Line(c) => Framing::Line(
                LineFramer::new(
                    *c,
                    if initial {
                        LineStart::NewStream
                    } else {
                        LineStart::ResumeAt(offset)
                    },
                )
                .map_err(|_| Fault::Configuration)?,
            ),
            Self::Multiline(c, pattern) => Framing::Multiline(
                MultilineFramer::new(
                    *c,
                    pattern.clone(),
                    if initial {
                        RecordStart::NewStream
                    } else {
                        RecordStart::ResumeAt(offset)
                    },
                )
                .map_err(|_| Fault::Configuration)?,
            ),
        };
        let history_limit = self.history_bytes()?;
        let mut history = VecDeque::with_capacity(history_limit);
        history.extend(window.bytes());
        Ok(Partial {
            framer,
            input: Vec::with_capacity(read_bytes),
            cursor: 0,
            history,
            history_end: offset,
            history_limit,
        })
    }

    pub(super) fn matcher(&self) -> Option<BoundaryMatcher> {
        match self {
            Self::Line(_) => None,
            Self::Multiline(_, p) => Some(p.matcher()),
        }
    }
}

// Inline storage is included in every conservative slot, avoiding a separate
// allocation and pointer chase for the framer on each capacity admission.
#[allow(clippy::large_enum_variant)]
pub(super) enum Framing {
    Line(LineFramer),
    Multiline(MultilineFramer),
}

#[derive(Debug)]
pub(super) enum Frame {
    Line(LineFrame),
    Multiline(RecordFrame),
}

impl Frame {
    pub(super) fn retained_capacity(&self) -> usize {
        let (body, source) = match self {
            Self::Line(f) => (&f.body, f.source_body.as_ref().map_or(0, Vec::capacity)),
            Self::Multiline(f) => (&f.body, 0),
        };
        source
            + match body {
                super::super::framer::LineBody::Text(s) => s.capacity(),
                super::super::framer::LineBody::Bytes(b) => b.capacity(),
            }
    }

    pub(super) fn range(&self) -> super::super::decoder::SourceRange {
        match self {
            Self::Line(f) => f.frame_range,
            Self::Multiline(f) => f.frame_range,
        }
    }
    pub(super) fn complete_record(&self) -> bool {
        match self {
            Self::Line(f) => f.fragment.is_none_or(|f| f.is_last),
            Self::Multiline(f) => f.fragment.is_none_or(|f| f.is_last),
        }
    }
}

#[derive(Debug)]
pub(super) enum FramingFailure {
    Line(LineError),
    Multiline(MultilineError),
}

pub(super) struct Step {
    pub consumed: usize,
    pub advanced: bool,
    pub frame: Option<Frame>,
    pub error: Option<FramingFailure>,
}

pub(super) struct Partial {
    framer: Framing,
    pub input: Vec<u8>,
    pub cursor: usize,
    history: VecDeque<u8>,
    history_end: u64,
    history_limit: usize,
}

impl Partial {
    pub(super) fn next(&mut self, input: &[u8], matcher: &mut Option<BoundaryMatcher>) -> Step {
        let step = match &mut self.framer {
            Framing::Line(f) => match f.next(f.next_expected_input_offset(), input) {
                Ok(s) => Step {
                    consumed: s.consumed,
                    advanced: s.advanced,
                    frame: s.frame.map(Frame::Line),
                    error: None,
                },
                Err(e) => Step {
                    consumed: e.consumed,
                    advanced: false,
                    frame: None,
                    error: Some(FramingFailure::Line(e.error)),
                },
            },
            Framing::Multiline(f) => match f.next(
                f.next_expected_input_offset(),
                input,
                matcher.as_mut().expect("multiline matcher"),
            ) {
                Ok(s) => Step {
                    consumed: s.consumed,
                    advanced: s.advanced,
                    frame: s.frame.map(Frame::Multiline),
                    error: None,
                },
                Err(e) => Step {
                    consumed: e.consumed,
                    advanced: false,
                    frame: None,
                    error: Some(FramingFailure::Multiline(e.error)),
                },
            },
        };
        for byte in &input[..step.consumed] {
            if self.history.len() == self.history_limit {
                let _ = self.history.pop_front();
            }
            self.history.push_back(*byte);
        }
        self.history_end += step.consumed as u64; // Decoder already checks this addition.
        step
    }

    pub(super) fn boundary(&self, offset: u64) -> Result<Window, Fault> {
        let behind = usize::try_from(
            self.history_end
                .checked_sub(offset)
                .ok_or(Fault::Accounting)?,
        )
        .map_err(|_| Fault::Accounting)?;
        let len = offset.min(64) as usize;
        let end = self
            .history
            .len()
            .checked_sub(behind)
            .ok_or(Fault::Accounting)?;
        let start = end.checked_sub(len).ok_or(Fault::Accounting)?;
        let mut result = Window {
            end: offset,
            data: [0; 64],
        };
        for (out, byte) in result.data[..len]
            .iter_mut()
            .zip(self.history.range(start..end))
        {
            *out = *byte;
        }
        Ok(result)
    }

    pub(super) fn unfinished(&self) -> bool {
        self.cursor < self.input.len()
            || match &self.framer {
                Framing::Line(f) => f.pending_source_start().is_some(),
                Framing::Multiline(f) => f.pending_source_start().is_some(),
            }
    }

    pub(super) fn used(&self) -> usize {
        size_of::<Self>()
            + self.input.capacity()
            + self.history.capacity()
            + match &self.framer {
                Framing::Line(f) => f.retained_capacity(),
                Framing::Multiline(f) => f.retained_capacity(),
            }
    }
}
