// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTAP Contrib extensions.
//!
//! Each extension is gated behind an opt-in feature and registers itself into
//! the OTAP pipeline factory's extension slice via `linkme` when its feature is
//! enabled.

#[cfg(feature = "azure-identity-auth-extension")]
pub mod azure_identity_auth;
