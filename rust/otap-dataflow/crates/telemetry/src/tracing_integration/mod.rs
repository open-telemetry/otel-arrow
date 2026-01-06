// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! tokio-tracing support for directly encoding and formatting OTLP bytes.

pub mod log_record;
pub mod otlp_bytes_formatter;
pub mod subscriber;

pub use log_record::{TracingAnyValue, TracingAttribute, TracingLogRecord};
pub use otlp_bytes_formatter::{FormatError, OtlpBytesFormattingLayer};
pub use subscriber::OtlpTracingLayer;
