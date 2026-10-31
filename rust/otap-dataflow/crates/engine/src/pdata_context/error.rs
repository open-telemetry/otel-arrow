// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Runtime failures of context binding operations.
//!
//! Construction-time lookups report configuration errors instead. A receiver
//! rejects a request whose metadata cannot be formed; a processor permanently
//! nacks the input.

/// A binding operation could not form message metadata.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ContextError {
    /// A produced value carried a handle issued by a different projection binding.
    #[error("produced context entry handle was not issued by this node's projection binding")]
    ForeignProducedEntry,
}
