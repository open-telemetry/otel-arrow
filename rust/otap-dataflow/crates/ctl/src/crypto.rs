// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Crypto-provider bootstrap for the admin SDK client.
//!
//! Rustls allows only one process-wide crypto provider to be installed, while
//! this crate supports multiple feature-selected providers. This module keeps
//! that initialization explicit, idempotent, and centralized so commands and the
//! TUI can build TLS-capable admin clients without each caller knowing which
//! provider feature was compiled.

use crate::error::CliError;
use std::sync::OnceLock;

/// Install the configured Rustls crypto provider once for the process.
pub(crate) fn ensure_crypto_provider() -> Result<(), CliError> {
    static INIT: OnceLock<Result<(), String>> = OnceLock::new();

    INIT.get_or_init(|| {
        #[cfg(feature = "crypto-openssl")]
        {
            let _ = rustls_openssl::default_provider().install_default();
            Ok(())
        }

        #[cfg(all(feature = "crypto-aws-lc", not(feature = "crypto-openssl")))]
        {
            let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
            Ok(())
        }

        #[cfg(all(
            feature = "crypto-ring",
            not(feature = "crypto-openssl"),
            not(feature = "crypto-aws-lc")
        ))]
        {
            let _ = rustls::crypto::ring::default_provider().install_default();
            Ok(())
        }

        #[cfg(all(
            feature = "crypto-symcrypt",
            not(feature = "crypto-openssl"),
            not(feature = "crypto-aws-lc"),
            not(feature = "crypto-ring")
        ))]
        {
            let _ = rustls_symcrypt::default_symcrypt_provider().install_default();
            Ok(())
        }

        #[cfg(not(any(
            feature = "crypto-ring",
            feature = "crypto-aws-lc",
            feature = "crypto-openssl",
            feature = "crypto-symcrypt"
        )))]
        {
            Err(
                "admin TLS support requires one of the crypto features: crypto-ring, crypto-aws-lc, crypto-openssl, or crypto-symcrypt"
                    .to_string(),
            )
        }
    })
    .clone()
    .map_err(CliError::config)
}
