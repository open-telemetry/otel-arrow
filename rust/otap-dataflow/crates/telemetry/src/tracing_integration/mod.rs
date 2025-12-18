// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Integration between tokio-tracing and OTLP stateful encoder.
//!
//! This module bridges tokio-tracing events into our stateful OTLP encoder by implementing
//! the `LogRecordView` trait for tracing events. This enables both:
//! 1. Using tokio macros like `info!`, `warn!`, etc. in the engine
//! 2. Capturing 3rd-party libraries' tracing events through our pipeline
//!
//! The integration consists of:
//! - `TracingLogRecord`: A `LogRecordView` impl wrapping tracing::Event
//! - `OtlpTracingLayer`: A tracing subscriber layer that captures events
//! - Helper functions to extract data from tracing events

pub mod log_record;
pub mod subscriber;
pub mod otlp_bytes_formatter;
pub mod otlp_event_dispatcher;
pub mod otlp_bytes_channel;

pub use log_record::{TracingAttribute, TracingAnyValue, TracingLogRecord};
pub use subscriber::OtlpTracingLayer;
pub use otlp_bytes_formatter::{OtlpBytesFormattingLayer, FormatError};
pub use otlp_event_dispatcher::{dispatch_otlp_bytes_as_events, DispatchError};
pub use otlp_bytes_channel::{OtlpBytesChannel, OtlpBytesConsumerConfig, OtlpBytesChannelStats};
