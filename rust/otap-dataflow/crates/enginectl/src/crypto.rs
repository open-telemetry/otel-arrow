// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Crypto-provider bootstrap for the admin SDK client.

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

        #[cfg(not(any(
            feature = "crypto-ring",
            feature = "crypto-aws-lc",
            feature = "crypto-openssl"
        )))]
        {
            Err(
                "admin TLS support requires one of the crypto features: crypto-ring, crypto-aws-lc, or crypto-openssl"
                    .to_string(),
            )
        }
    })
    .clone()
    .map_err(CliError::config)
}
