// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

pub(crate) use super::super::partial_state::Partial;
use super::super::partial_state::{self, Framing};
use super::*;
use crate::receivers::filelog_receiver::{
    framer::{LineConfig, LineError, LineFrame, LineFramer, LineStart},
    multiline::{MultilineConfig, MultilineError, MultilineFramer, RecordFrame, RecordStart},
    multiline_pattern::{BoundaryMatcher, BoundaryPattern},
};

#[derive(Clone, Debug)]
pub(crate) enum FramingConfig {
    Line(LineConfig),
    Multiline(MultilineConfig, BoundaryPattern),
}

impl FramingConfig {
    fn history_bytes(&self) -> Result<usize, Fault> {
        partial_state::history_bytes(self.lookahead_bytes()).ok_or(Fault::Configuration)
    }

    fn lookahead_bytes(&self) -> usize {
        match self {
            Self::Line(_) => 0,
            Self::Multiline(c, _) => c.line.max_line_bytes,
        }
    }

    pub(crate) fn slot_bytes(&self, read_bytes: usize) -> Result<usize, Fault> {
        let payload = match self {
            Self::Line(c) => c.payload_peak_bytes().map_err(|_| Fault::Configuration)?,
            Self::Multiline(c, _) => c.payload_peak_bytes().map_err(|_| Fault::Configuration)?,
        };
        partial_state::slot_bytes(payload, read_bytes, self.lookahead_bytes())
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

#[derive(Debug)]
pub(crate) enum Frame {
    Line(LineFrame),
    Multiline(RecordFrame),
}

impl Frame {
    pub(crate) fn retained_capacity(&self) -> usize {
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

    pub(crate) fn range(&self) -> super::super::decoder::SourceRange {
        match self {
            Self::Line(f) => f.frame_range,
            Self::Multiline(f) => f.frame_range,
        }
    }
    pub(crate) fn complete_record(&self) -> bool {
        match self {
            Self::Line(f) => f.fragment.is_none_or(|f| f.is_last),
            Self::Multiline(f) => f.fragment.is_none_or(|f| f.is_last),
        }
    }
}

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
