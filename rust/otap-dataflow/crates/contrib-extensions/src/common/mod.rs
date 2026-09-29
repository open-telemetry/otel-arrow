// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared functions and data types for contrib extension implementations.

#[cfg(any(
    feature = "azure-identity-auth",
    feature = "oauth2-client-auth",
    feature = "flat-file-user-pass-auth"
))]
pub mod background_refresh;

#[cfg(any(feature = "azure-identity-auth", feature = "oauth2-client-auth"))]
pub mod token_refresh;
