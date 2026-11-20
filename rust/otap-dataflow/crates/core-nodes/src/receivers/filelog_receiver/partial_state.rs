// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared partial-state layout and accounting for configuration and scheduling.

use super::{framer::LineFramer, multiline::MultilineFramer};
use std::collections::VecDeque;

// Inline storage is included in every conservative slot, avoiding a separate
// allocation and pointer chase for the framer on each capacity admission.
#[allow(clippy::large_enum_variant)]
pub(crate) enum Framing {
    Line(LineFramer),
    Multiline(MultilineFramer),
}

pub(crate) struct Partial {
    pub(crate) framer: Framing,
    pub input: Vec<u8>,
    pub cursor: usize,
    pub(crate) history: VecDeque<u8>,
    pub(crate) history_end: u64,
    pub(crate) history_limit: usize,
    pub(crate) continuing: bool,
}

/// Source history includes multiline lookahead and decoder/frontier slack.
pub(super) fn history_bytes(lookahead_bytes: usize) -> Option<usize> {
    // UTF-16 source uses at most two bytes per decoded line byte.
    lookahead_bytes.checked_mul(2)?.checked_add(136)
}

/// One complete reservation, including the continuity window and inline state.
pub(super) fn slot_bytes(
    payload: usize,
    read_bytes: usize,
    lookahead_bytes: usize,
) -> Option<usize> {
    payload
        .checked_add(read_bytes)?
        .checked_add(history_bytes(lookahead_bytes)?)?
        .checked_add(64 + size_of::<Partial>())
        .filter(|n| *n <= isize::MAX as usize)
}
