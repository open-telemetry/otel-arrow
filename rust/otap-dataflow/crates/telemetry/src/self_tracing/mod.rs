// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! tokio-tracing support for directly encoding and formatting OTLP bytes.

pub mod compact_formatter;
pub mod direct_encoder;
pub mod log_record;
pub mod subscriber;

// Compact formatter exports (recommended for minimal fmt::layer() alternative)
pub use compact_formatter::{
    CachedCallsite, CallsiteCache, CompactFormatterLayer, CompactLogRecord, OutputTarget,
    SimpleWriter, format_log_record,
};

// Direct encoder exports (for zero-allocation OTLP encoding)
pub use direct_encoder::{
    DirectFieldVisitor, DirectLogRecordEncoder, LengthPlaceholder, ProtoBuffer,
    StatefulDirectEncoder, encode_len_placeholder, encode_resource_bytes_from_attrs,
    patch_len_placeholder,
};

// Legacy View-based exports (for compatibility)
pub use log_record::{TracingAnyValue, TracingAttribute, TracingLogRecord};
pub use subscriber::OtlpTracingLayer;
