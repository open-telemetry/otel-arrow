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

#[cfg(any(test, feature = "test-utils"))]
use crate::capability::registry::{
    Capabilities, CapabilityRegistry, ConsumedTracker, resolve_bindings,
};
#[cfg(any(test, feature = "test-utils"))]
use otap_df_config::{CapabilityId, ExtensionId};
#[cfg(any(test, feature = "test-utils"))]
use std::collections::{HashMap, HashSet};

/// Resolves capability bindings for downstream component tests.
///
/// Production code resolves bindings as part of pipeline construction. This
/// helper exposes the same path to tests that need a real [`Capabilities`]
/// instance rather than [`Capabilities::empty`].
#[cfg(any(test, feature = "test-utils"))]
pub fn resolve_bindings_for_test(
    bindings: &HashMap<CapabilityId, ExtensionId>,
    registry: &CapabilityRegistry,
    known_extensions: &HashSet<ExtensionId>,
) -> Result<Capabilities, crate::error::Error> {
    let mut tracker = ConsumedTracker::new();
    resolve_bindings(bindings, registry, known_extensions, &mut tracker)
}
