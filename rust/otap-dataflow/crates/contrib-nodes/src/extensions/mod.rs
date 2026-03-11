// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Azure Identity authentication extension.
#[cfg(feature = "azure-identity-auth-extension")]
pub mod azure_identity_auth_extension;

/// Static bearer token authentication extension.
#[cfg(feature = "bearer-auth-extension")]
pub mod bearer_auth_extension;
