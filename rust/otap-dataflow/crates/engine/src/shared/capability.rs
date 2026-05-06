// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared (Send) capability trait re-exports.
//!
//! Each capability's shared trait variant is re-exported here for convenience.
//! Capability traits are defined by the `#[capability]` proc macro in
//! per-capability modules under [`capability`](crate::capability).

pub use crate::capability::no_op_stateful::shared::NoOpStateful;
pub use crate::capability::no_op_stateless::shared::NoOpStateless;
