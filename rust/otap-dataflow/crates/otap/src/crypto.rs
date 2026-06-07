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
//! | `crypto-openssl` | `rustls-openssl` | Regulated / FIPS environments      |
//! | `crypto-symcrypt`| `rustls-symcrypt`| Microsoft/SymCrypt-aligned backend |

use cfg_if::cfg_if;

/// Installs the selected rustls `CryptoProvider` as the process-wide default.
///
/// The provider is chosen at compile time via feature flags.
/// This function must be called **once**, early in `main()`, before any TLS
/// connections are established (including via `reqwest`, `tonic`, etc.).
///
/// TLS support is always compiled in, but a crypto provider is only required
/// when the process actually uses TLS/HTTPS paths. Plaintext-only pipelines can
/// run without a `crypto-*` feature. Any HTTPS exporter, TLS receiver, or
/// HTTPS proxy configuration requires one of: `crypto-ring`,
/// `crypto-aws-lc`, or `crypto-openssl`.
///
/// # Errors
///
/// Returns `Err` if a provider was already installed (non-fatal in most cases).
/// Returns `Ok(())` if installation succeeds or if no crypto feature is enabled.
pub fn install_crypto_provider() -> Result<(), String> {
    cfg_if! {
        // If you're using rustls, you must install a rustls CryptoProvider.
        if #[cfg(feature = "crypto-ring")] {
            rustls::crypto::ring::default_provider()
                .install_default()
                .map_err(|_| "crypto provider already installed (ring)".to_string())?;
        } else if #[cfg(feature = "crypto-aws-lc")] {
            rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .map_err(|_| "crypto provider already installed (aws-lc-rs)".to_string())?;
        } else if #[cfg(feature = "crypto-openssl")] {
            rustls_openssl::default_provider()
            .install_default()
            .map_err(|_| "crypto provider already installed (openssl)".to_string())?;
        } else if #[cfg(feature = "crypto-symcrypt")] {
            rustls_symcrypt::default_symcrypt_provider()
            .install_default()
            .map_err(|_| {
                "crypto provider is already installed (crypto-symcrypt)".to_string()})?;
        } else {
            otap_df_telemetry::otel_warn!(
            "crypto.no_provider",
            message = "no crypto-* feature enabled: TLS operations will fail at runtime. \
                       Enable exactly one of: crypto-ring, crypto-aws-lc, crypto-openssl, \
                       crypto-symcrypt"
            );
        }
    }
    Ok(())
}

/// Installs the crypto provider idempotently (intended for test setup).
///
/// Uses [`std::sync::Once`] so it is safe to call from every test — the actual
/// installation happens at most once per process.
pub fn ensure_crypto_provider() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        #[cfg(any(
            feature = "crypto-ring",
            feature = "crypto-aws-lc",
            feature = "crypto-symcrypt",
            feature = "crypto-openssl"
        ))]
        {
            let _ = install_crypto_provider();
        }

        #[cfg(not(any(
            feature = "crypto-ring",
            feature = "crypto-aws-lc",
            feature = "crypto-symcrypt",
            feature = "crypto-openssl"
        )))]
        {
            let _ = rustls::crypto::ring::default_provider().install_default();
        }
    });
}
