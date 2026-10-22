// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Geneva Metrics protocol version 6 encoding and OTLP mapping.

pub mod encoder;
pub mod otlp_to_geneva;

pub use otlp_to_geneva::{Config, ScopeAttributes};
