// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Local (!Send) capability trait re-exports.
//!
//! Each capability's local trait variant is re-exported here for convenience.
//! Capability traits are defined by the `#[capability]` proc macro in
//! per-capability modules under [`capability`](crate::capability).

// TODO: Add re-exports as capabilities are defined, e.g.:
// pub use crate::capability::bearer_token_provider::local::BearerTokenProvider;
// pub use crate::capability::key_value_store::local::KeyValueStore;
