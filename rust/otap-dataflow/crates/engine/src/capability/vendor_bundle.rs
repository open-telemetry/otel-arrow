// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The `VendorBundle` capability.
//!
//! An agent-fed, vendor-defined attribute bag (a JSON object) the engine
//! transports untouched from the host to a data-path node. The engine never
//! interprets the contents; each consumer defines and reads its own keys.
//! Supplied by an agent-fed extension; token-only scopes do not provide it.
//!
//! The host should publish these together with the bearer token so a consumer
//! reads a consistent token/attributes snapshot.
//!
//! Like [`bearer_token_provider`](super::bearer_token_provider), the trait is
//! expanded by the `#[capability]` proc macro into `local` (!Send) and `shared`
//! (Send) variants, a `SharedAsLocal` adapter, a zero-sized registration
//! handle, and a `KNOWN_CAPABILITIES` distributed-slice entry.

use otap_df_engine_macros::capability;
use serde_json::{Map, Value};
use std::sync::Arc;

use super::error::CapabilityError;

/// Hands out an agent-fed, vendor-defined attribute bag to data-path nodes.
#[capability(
    name = "vendor_bundle",
    description = "Agent-fed vendor-defined attribute bag (opaque JSON object)"
)]
pub trait VendorBundle {
    /// The vendor-defined attributes as a shared, already-parsed JSON object.
    ///
    /// Opaque to the engine; the consumer defines its own keys. Returned as an
    /// `Arc` so a read is a refcount bump rather than a re-parse.
    ///
    /// Returns a [`CapabilityError`] when no bundle is provisioned yet or one
    /// cannot be produced, so a consumer can distinguish that from an
    /// "empty but valid" bundle (`Ok` with an empty map).
    fn attributes(&self) -> Result<Arc<Map<String, Value>>, CapabilityError>;
}
