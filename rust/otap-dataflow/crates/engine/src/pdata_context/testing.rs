// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Test-only builders for compiled bindings, kept out of the binding types.

use super::bindings::NodeBindings;
use super::reads::HeadersRead;

/// Installs the all-captured transport-header read, as the compiler does for
/// an `AllCaptured` consumer declaration.
pub fn install_headers_read(bindings: &mut NodeBindings) {
    bindings.headers = Some(HeadersRead::new());
}
