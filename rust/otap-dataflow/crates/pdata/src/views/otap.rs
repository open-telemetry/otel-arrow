// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains implementation of the Views traits for OTAP Arrow RecordBatches.
//!
//! This provides zero-copy views over OTAP columnar data, abstracting away the internal
//! structure of Arrow RecordBatches. It enables direct iteration over the data using
//! a hierarchical OTLP-like interface (Resource -> Scope -> LogRecord) without exposing
//! the complexity of the raw Arrow batches or requiring conversion to intermediate formats.

pub(crate) mod common;
pub(crate) mod logs;
pub(crate) mod metrics;
pub(crate) mod traces;

pub use logs::OtapLogsView;
pub use metrics::OtapMetricsView;
pub use traces::OtapTracesView;
