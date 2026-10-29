// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::CompiledHeaderPropagationPolicy;
use otel_arrow_dfe_config::context::authorized_identity::AuthorizedIdentityPolicy;
use otel_arrow_dfe_config::context::transport_policy::CompiledHeaderCapturePolicy;

/// Immutable runtime context policies compiled for one node.
///
/// Compiler declaration-validation state is kept separately. Policy types stay
/// private as node-facing binding operations are introduced incrementally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeBindings {
    /// Receiver header capture policy compiled for the engine requirements.
    pub(super) header_capture: Option<CompiledHeaderCapturePolicy>,
    /// Exporter header propagation policy resolved from node or pipeline config.
    pub(super) header_propagation: Option<CompiledHeaderPropagationPolicy>,
    /// Receiver authorized identity claim projection policy.
    pub(super) authorized_identity_capture: Option<AuthorizedIdentityPolicy>,
}

impl NodeBindings {
    pub(super) fn is_empty(&self) -> bool {
        self.header_capture.is_none()
            && self.header_propagation.is_none()
            && self.authorized_identity_capture.is_none()
    }
}
