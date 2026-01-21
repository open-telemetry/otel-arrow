// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Core OTAP exporter components.

/// An error-exporter returns a static error
pub mod error_exporter;

/// Implementation of a noop exporter that acts as a exporter placeholder
pub mod noop_exporter;

/// Implementation of OTAP Exporter that implements the exporter trait
pub mod otap_exporter;

/// Implementation of OTLP exporter that implements the exporter trait
pub mod otlp_exporter;

/// Parquet exporter
pub mod parquet_exporter;

/// Performance exporter for testing
pub mod perf_exporter;
