// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Azure auth utilities
#[cfg(feature = "azure")]
pub mod azure;

/// AWS auth utilities
#[cfg(feature = "aws")]
pub mod aws;

/// Redacted string type for sensitive values.
pub mod opaque_string;
