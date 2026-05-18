// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Test-only reference capabilities.
//!
//! Reference capabilities used by integration tests (and downstream
//! tests via the public `testing` surface) to exercise the
//! extension/capability wiring. Real production capabilities live
//! under [`crate::capability`] and are re-exported through
//! [`local::capability`](crate::local::capability) /
//! [`shared::capability`](crate::shared::capability).

pub mod no_op_stateful;
pub mod no_op_stateless;
