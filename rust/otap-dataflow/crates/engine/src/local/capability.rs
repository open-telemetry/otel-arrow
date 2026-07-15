// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Local (!Send) capability trait re-exports.
//!
//! Each production capability's local trait variant is re-exported here
//! for convenience. Capability traits are defined by the `#[capability]`
//! proc macro in per-capability modules under
//! [`capability`](crate::capability). Test-only reference capabilities
//! live under [`crate::testing::capability`] and are intentionally not
//! re-exported here.

pub use crate::capability::bearer_token_provider::local::BearerTokenProvider;
pub use crate::capability::vendor_bundle::local::VendorBundle;
