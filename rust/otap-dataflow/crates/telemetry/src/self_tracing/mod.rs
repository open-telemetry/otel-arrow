// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! tokio-tracing support for directly encoding and formatting OTLP bytes.

pub mod direct_encoder;
pub mod log_record;
pub mod otlp_bytes_formatter;
pub mod subscriber;

// New direct encoder exports (preferred for zero-allocation encoding)
pub use direct_encoder::{
    DirectFieldVisitor, DirectLogRecordEncoder, LengthPlaceholder, ProtoBuffer,
    StatefulDirectEncoder, encode_len_placeholder, encode_resource_bytes_from_attrs,
    patch_len_placeholder,
};

// Legacy View-based exports (for compatibility)
pub use log_record::{TracingAnyValue, TracingAttribute, TracingLogRecord};
pub use otlp_bytes_formatter::{FormatError, OtlpBytesFormattingLayer};
pub use subscriber::OtlpTracingLayer;
