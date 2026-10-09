// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared functions and data types for contrib extension implementations.

#[cfg(any(feature = "azure-identity-auth", feature = "azure-key-vault-sasl-auth"))]
pub mod azure_identity;

#[cfg(feature = "azure-key-vault-sasl-auth")]
pub mod azure_key_vault;

#[cfg(any(
    feature = "azure-identity-auth",
    feature = "azure-key-vault-sasl-auth",
    feature = "oauth2-client-auth",
    feature = "flat-file-api-key-auth",
    feature = "flat-file-user-pass-auth"
))]
pub mod background_refresh;

#[cfg(any(
    feature = "flat-file-api-key-auth",
    feature = "flat-file-user-pass-auth"
))]
pub mod secret_file;

#[cfg(any(feature = "azure-identity-auth", feature = "oauth2-client-auth"))]
pub mod token_refresh;
