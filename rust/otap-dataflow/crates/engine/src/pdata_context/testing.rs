// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Test-only builders for compiled bindings, kept out of the binding types.

use super::bindings::{ArrivalBinding, NodeBindings};
use super::reads::HeadersRead;
use otel_arrow_dfe_config::context::transport_policy::CompiledHeaderCapturePolicy;

/// Installs the all-captured transport-header read, as the compiler does for
/// an `AllCaptured` consumer declaration.
pub fn install_headers_read(bindings: &mut NodeBindings) {
    bindings.headers = Some(HeadersRead::new());
}

/// Builds the arrival binding the compiler produces for a receiver whose
/// pipeline captures transport headers with `header_capture`.
#[must_use]
pub fn arrival_binding(header_capture: Option<CompiledHeaderCapturePolicy>) -> ArrivalBinding {
    ArrivalBinding {
        header_capture,
        ..ArrivalBinding::default()
    }
}
