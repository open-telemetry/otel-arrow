// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the Contrib nodes (receiver, exporter, processor).

/// Exporter implementations for contrib nodes.
pub mod exporters;

/// Processor implementations for contrib nodes.
pub mod processors;

/// Test support: installs the rustls crypto provider once for the entire test binary.
/// This is needed because `reqwest` with `rustls-no-provider` requires a globally
/// installed `CryptoProvider` before creating any HTTP client.
#[cfg(test)]
pub(crate) fn ensure_crypto_provider() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = otap_df_otap::crypto::install_crypto_provider();
    });
}
