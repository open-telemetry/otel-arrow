// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP equivalence checking for testing round-trips through OTAP encoding.
//!
//! This module provides functions to check semantic equivalence of OTLP payloads,
//! even when the structure has been reorganized (e.g., resource/scope splits or
//! merges during batching operations).
//!
//! The approach flattens the OTLP hierarchy into individual items (log records,
//! spans, metric data points) that combine all their context (resource, scope)
//! into a single comparable key.

mod canonical;
mod logs;
mod metrics;
mod traces;

pub use logs::assert_logs_equivalent;
pub use metrics::assert_metrics_equivalent;
pub use traces::assert_traces_equivalent;
