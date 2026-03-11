// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Cryptographic provider initialization for rustls.
//!
//! This module centralizes the installation of the process-wide rustls
//! [`CryptoProvider`](rustls::crypto::CryptoProvider) based on compile-time
//! feature flags.  Exactly one of the `crypto-*` features must be enabled:
//!
//! | Feature          | Backend          | Use-case                           |
//! |------------------|------------------|------------------------------------|
//! | `crypto-ring`    | `ring`           | Default, backward-compatible       |
//! | `crypto-aws-lc`  | `aws-lc-rs`      | AWS environments, broader algos    |
//! | `crypto-openssl` | `rustls-openssl`  | Regulated / FIPS environments      |

/// Installs the selected rustls `CryptoProvider` as the process-wide default.
///
/// The provider is chosen at compile time via feature flags.
/// This function must be called **once**, early in `main()`, before any TLS
/// connections are established (including via `reqwest`, `tonic`, etc.).
///
/// # Errors
///
/// Returns `Err` if a provider was already installed (non-fatal in most cases).
/// Returns `Ok(())` if installation succeeds or if no crypto feature is enabled.
pub fn install_crypto_provider() -> Result<(), String> {
    // Priority order when multiple features are enabled (e.g. --all-features):
    // ring > aws-lc-rs > openssl.
    #[cfg(feature = "crypto-ring")]
    {
        rustls::crypto::ring::default_provider()
            .install_default()
            .map_err(|_| "crypto provider already installed (ring)".to_string())?;
    }

    #[cfg(all(feature = "crypto-aws-lc", not(feature = "crypto-ring")))]
    {
        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .map_err(|_| "crypto provider already installed (aws-lc-rs)".to_string())?;
    }

    #[cfg(all(
        feature = "crypto-openssl",
        not(feature = "crypto-ring"),
        not(feature = "crypto-aws-lc")
    ))]
    {
        rustls_openssl::default_provider()
            .install_default()
            .map_err(|_| "crypto provider already installed (openssl)".to_string())?;
    }

    Ok(())
}
