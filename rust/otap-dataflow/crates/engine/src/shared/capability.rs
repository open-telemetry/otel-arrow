// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared (Send) capability trait re-exports.
//!
//! Each production capability's shared trait variant is re-exported
//! here for convenience. Capability traits are defined by the
//! `#[capability]` proc macro in per-capability modules under
//! [`capability`](crate::capability). Test-only reference capabilities
//! live under [`crate::testing::capability`] and are intentionally not
//! re-exported here.
