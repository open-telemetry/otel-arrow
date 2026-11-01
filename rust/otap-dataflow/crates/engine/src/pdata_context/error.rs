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
    /// A value supplied to a binding does not match the binding's compiled context.
    #[error("context value does not match this node's compiled context bindings")]
    ContextMismatch,
}
