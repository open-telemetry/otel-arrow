// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains implementation of the Views traits for OTAP Arrow RecordBatches.
//!
//! This provides zero-copy views over OTAP columnar data, abstracting away the internal
//! structure of Arrow RecordBatches. It enables direct iteration over the data using
//! a hierarchical OTLP-like interface (Resource -> Scope -> LogRecord) without exposing
//! the complexity of the raw Arrow batches or requiring conversion to intermediate formats.

mod logs;

pub use logs::OtapLogsView;
