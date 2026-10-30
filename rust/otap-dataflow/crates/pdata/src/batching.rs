// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Batching results whose input provenance is independent of output measurements.

use std::ops::RangeInclusive;

/// Outputs of one ordered batching call, indexed into that call's input vector.
///
/// Each inclusive range identifies the inputs consumed to produce an output,
/// including rows removed by the format's integrity repair. Adjacent outputs
/// may share a boundary input when that input is split. Ranges must be nonempty,
/// in bounds, and ordered; overlap is allowed only at a shared boundary input.
/// Every input must occur in an output range or in `completed_inputs`, never both.
/// Codecs must not infer provenance from output byte lengths or item counts.
///
/// This describes batch-level provenance, not record order within an input.
/// Delivery contexts, completion references, and retention remain caller-owned.
pub struct BatchingResult<T> {
    /// Payloads paired with the inclusive range of original input indices consumed.
    pub batches: Vec<(T, RangeInclusive<usize>)>,
    /// Sorted, unique indices consumed successfully without producing any output.
    /// These inputs can complete immediately; errors must instead fail the call.
    pub completed_inputs: Vec<usize>,
    /// Entries emitted whole because splitting would exceed amplification budgets.
    pub budget_fallbacks: u64,
}
