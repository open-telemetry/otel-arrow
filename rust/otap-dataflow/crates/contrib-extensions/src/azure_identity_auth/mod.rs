// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Azure Identity Auth extension.
//!
//! Acquires and refreshes Azure access tokens via the `azure_identity` SDK and
//! exposes them to data-path nodes through the `BearerTokenProvider`
//! capability. See `docs/azure-identity-auth-extension.md` for the design.

pub mod config;
pub mod error;
