// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared building blocks reused by more than one core exporter.

/// Consumer-side adapter over a bound `bearer_token_provider` capability.
pub(crate) mod bearer_auth;
