// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compact log formatting for tokio-tracing events.
//!
//! This module provides a lightweight formatting layer for tokio-tracing events
//! that encodes body+attributes to partial OTLP bytes, then formats them for
//! console output.

pub mod compact_formatter;
pub mod direct_encoder;

// Compact formatter exports (the primary API)
pub use compact_formatter::{
    CachedCallsite, CallsiteCache, CompactFormatterLayer, CompactLogRecord, OutputTarget,
    SimpleWriter, encode_body_and_attrs, format_log_record,
};

// Direct encoder exports (used internally, exposed for benchmarking)
pub use direct_encoder::{DirectFieldVisitor, ProtoBuffer};
