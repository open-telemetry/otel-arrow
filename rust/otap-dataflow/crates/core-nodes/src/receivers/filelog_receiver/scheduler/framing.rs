// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::receivers::filelog_receiver::{
    framer::{LineConfig, LineError, LineFramer, LineStart},
    multiline::{MultilineConfig, MultilineError, MultilineFramer, RecordStart},
    multiline_pattern::{BoundaryMatcher, BoundaryPattern},
};

#[derive(Clone, Debug)]
pub(crate) enum FramingConfig {
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

    pub(crate) fn slot_bytes(&self, read_bytes: usize) -> Result<usize, Fault> {
        let payload = match self {
            Self::Line(c) => c.payload_peak_bytes().map_err(|_| Fault::Configuration)?,
            Self::Multiline(c, _) => c.payload_peak_bytes().map_err(|_| Fault::Configuration)?,
        };
        payload
            .checked_add(read_bytes)
            .and_then(|n| n.checked_add(self.history_bytes().ok()?))
            .and_then(|n| n.checked_add(64 + size_of::<Partial>()))
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or(Fault::Configuration)
    }

    pub(crate) fn create(
        &self,
        offset: u64,
        initial: bool,
        window: &Window,
        resume: cp::FramingResume,
    ) -> Result<Partial, Fault> {
        let framer = match self {
            Self::Line(c) => Framing::Line(
                LineFramer::new(
                    *c,
                    match resume {
                        cp::FramingResume::Clean if initial => LineStart::NewStream,
                        cp::FramingResume::Clean => LineStart::ResumeAt(offset),
                        cp::FramingResume::Continuation {
                            record_start_offset,
                            record_end_offset: 0,
                            next_fragment_index,
                        } => LineStart::Continuation {
                            offset,
                            continuation: super::super::framer::LineContinuation {
                                record_start_offset,
                                next_fragment_index,
                            },
                        },
                        _ => return Err(Fault::Configuration),
                    },
                )
                .map_err(|_| Fault::Configuration)?,
            ),
            Self::Multiline(c, pattern) => Framing::Multiline(
                MultilineFramer::new(
                    *c,
                    pattern.clone(),
                    match resume {
                        cp::FramingResume::Clean if initial => RecordStart::NewStream,
                        cp::FramingResume::Clean => RecordStart::ResumeAt(offset),
                        cp::FramingResume::Continuation {
                            record_start_offset,
                            record_end_offset,
                            next_fragment_index,
                        } => RecordStart::Continuation {
                            offset,
                            continuation: super::super::multiline::RecordContinuation {
                                record_start_offset,
                                record_end_offset,
                                next_fragment_index,
                            },
                        },
                    },
                )
                .map_err(|_| Fault::Configuration)?,
            ),
        };
        let history_limit = self.history_bytes()?;
        let mut history = VecDeque::new();
        history.extend(window.bytes());
        Ok(Partial {
            framer,
            input: Vec::new(),
            cursor: 0,
            history,
            history_end: offset,
            history_limit,
            continuing: resume != cp::FramingResume::Clean,
        })
    }

    pub(crate) fn matcher(&self) -> Option<BoundaryMatcher> {
        match self {
            Self::Line(_) => None,
            Self::Multiline(_, p) => Some(p.matcher()),
        }
    }
}

// Inline storage is included in every conservative slot, avoiding a separate
// allocation and pointer chase for the framer on each capacity admission.
#[allow(clippy::large_enum_variant)]
pub(crate) enum Framing {
    Line(LineFramer),
    Multiline(MultilineFramer),
}

pub(crate) use super::super::batch::Frame;

#[derive(Debug)]
pub(crate) enum FramingFailure {
    Line(LineError),
    Multiline(MultilineError),
}

pub(crate) struct Step {
    pub consumed: usize,
    pub advanced: bool,
    pub frame: Option<Frame>,
    pub error: Option<FramingFailure>,
}

pub(crate) struct Partial {
    framer: Framing,
    pub input: Vec<u8>,
    pub cursor: usize,
    history: VecDeque<u8>,
    history_end: u64,
    history_limit: usize,
    continuing: bool,
}

impl Partial {
    pub(crate) fn next(&mut self, input: &[u8], matcher: &mut Option<BoundaryMatcher>) -> Step {
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
        if let Some(frame) = &step.frame {
            self.continuing = !frame.complete_record();
        }
        if step.consumed != 0 {
            // At most four fresh decoder bytes per call. Reserve the declared
            // history ceiling once, only when fresh bytes actually arrive.
            if self.history.capacity() < self.history_limit {
                self.history
                    .reserve_exact(self.history_limit - self.history.len());
            }
            let excess = (self.history.len() + step.consumed).saturating_sub(self.history_limit);
            drop(self.history.drain(..excess));
            self.history.extend(&input[..step.consumed]);
        }
        self.history_end += step.consumed as u64; // Decoder already checks this addition.
        step
    }

    pub(crate) fn boundary(&self, offset: u64) -> Result<Window, Fault> {
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

    pub(crate) fn unfinished(&self) -> bool {
        self.continuing
            || self.cursor < self.input.len()
            || match &self.framer {
                Framing::Line(f) => f.pending_source_start().is_some(),
                Framing::Multiline(f) => f.pending_source_start().is_some(),
            }
    }

    pub(crate) fn used(&self) -> usize {
        size_of::<Self>()
            + self.input.capacity()
            + self.history.capacity()
            + match &self.framer {
                Framing::Line(f) => f.retained_capacity(),
                Framing::Multiline(f) => f.retained_capacity(),
            }
    }
}
