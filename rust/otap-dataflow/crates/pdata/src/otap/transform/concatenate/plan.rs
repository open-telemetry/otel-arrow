// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Plan types shared between the ID planner and the column writers.
//!
//! The planner (see [`crate::otap::transform::reindex`]) inspects the input
//! batches without modifying them and produces, per input and per payload, a
//! [Selection] of rows that survive into the output plus an [IdRemap] for each
//! ID column. The column writers then copy every output column exactly once,
//! applying the selection and remaps on the fly.

use std::ops::Range;

use arrow::buffer::ScalarBuffer;
use arrow::datatypes::ArrowNativeType;

/// The rows of a single input record batch that survive into the output.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum Selection {
    /// Every row is kept. This is the common case.
    #[default]
    All,
    /// Only the rows in these sorted, disjoint, non-empty ranges are kept.
    Ranges(Vec<Range<usize>>),
}

impl Selection {
    /// Number of selected rows given the source length.
    #[must_use]
    pub(crate) fn count(&self, len: usize) -> usize {
        match self {
            Selection::All => len,
            Selection::Ranges(ranges) => ranges.iter().map(|r| r.len()).sum(),
        }
    }

    /// Iterate the selected ranges given the source length.
    pub(crate) fn ranges(&self, len: usize) -> impl Iterator<Item = Range<usize>> + '_ {
        let (all, ranges): (Option<Range<usize>>, &[Range<usize>]) = match self {
            Selection::All => ((len > 0).then_some(0..len), &[]),
            Selection::Ranges(ranges) => (None, ranges.as_slice()),
        };
        all.into_iter().chain(ranges.iter().cloned())
    }
}

/// How the values of one ID column of one input are transformed on the way to
/// the output.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum IdRemap<T: ArrowNativeType> {
    /// Values are copied as-is.
    Identity,
    /// `out = in.wrapping_add(delta)`. Wrapping arithmetic is intentional:
    /// null slots may hold arbitrary values that must not panic.
    Offset(T),
    /// Replacement values in source order. Indexed like the source column, or
    /// like the dictionary values array for a dictionary-encoded column.
    Replace(ScalarBuffer<T>),
}

/// An [IdRemap] for either of the two ID widths used by OTAP.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AnyRemap {
    U16(IdRemap<u16>),
    U32(IdRemap<u32>),
}

/// The ID columns that may be remapped within a payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IdCol {
    Id = 0,
    ResourceId = 1,
    ScopeId = 2,
    ParentId = 3,
}

impl IdCol {
    pub(crate) const COUNT: usize = 4;
}

/// Plan for a single input record batch of a single payload type.
#[derive(Debug, Clone, Default)]
pub(crate) struct InputPlan {
    /// Rows that survive into the output.
    pub(crate) selection: Selection,
    /// Remaps for each ID column, indexed by [IdCol]. `None` means identity.
    pub(crate) remaps: [Option<AnyRemap>; IdCol::COUNT],
}

impl InputPlan {
    /// Look up the remap for an ID column.
    #[must_use]
    pub(crate) fn remap(&self, col: IdCol) -> Option<&AnyRemap> {
        self.remaps[col as usize].as_ref()
    }
}
