// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared functions and data types for contrib extension implementations.

#[cfg(any(
    feature = "azure-identity-auth-extension",
    feature = "oauth2-client-auth-extension"
))]
pub mod token_refresh;
