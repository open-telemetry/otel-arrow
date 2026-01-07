// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Log encoding and formatting for Tokio tracing events.
//!
//! The intermediate representation is LogRecord, includes the
//! primitive fields and static references. The remaining data are
//! placed in a partial OTLP encoding.

pub mod encoder;
pub mod formatter;

use bytes::Bytes;
use encoder::DirectFieldVisitor;
use otap_df_pdata::otlp::ProtoBuffer;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::Event;
use tracing::callsite::Identifier;
use tracing::{Level, Metadata};

pub use encoder::DirectLogRecordEncoder;
pub use formatter::{ConsoleWriter, RawLayer as RawLoggingLayer};

/// A log record with structural metadata and pre-encoded body/attributes.
#[derive(Debug, Clone)]
pub struct LogRecord {
    /// Callsite identifier used to look up cached callsite info.
    pub callsite_id: Identifier,

    /// Timestamp in UNIX epoch nanoseconds.
    pub timestamp_ns: u64,

    /// Pre-encoded body and attributes in OTLP bytes.
    pub body_attrs_bytes: Bytes,
}

/// Saved callsite information. This is information that can easily be
/// populated from Metadata, for example in a `register_callsite` hook
/// for building a map by Identifier.
#[derive(Debug, Clone)]
pub struct SavedCallsite {
    /// Target (e.g., module path)
    pub target: &'static str,

    /// Event name
    pub name: &'static str,

    /// Source file
    pub file: Option<&'static str>,

    /// Source line
    pub line: Option<u32>,

    /// Severity level
    pub level: &'static Level,
}

impl SavedCallsite {
    /// Construct saved callsite information from tracing Metadata.
    #[must_use]
    pub fn new(metadata: &'static Metadata<'static>) -> Self {
        Self {
            level: metadata.level(),
            target: metadata.target(),
            name: metadata.name(),
            file: metadata.file(),
            line: metadata.line(),
        }
    }
}

impl LogRecord {
    /// Construct a log record, partially encoding its dynamic content.
    #[must_use]
    pub fn new(event: &Event<'_>) -> Self {
        let metadata = event.metadata();

        // Encode body and attributes to bytes.
        // Note! TODO: we could potentially avoid allocating for the intermediate
        // protobuf slice with work to support a fixed-size buffer and cursor
        // instead of a Vec<u8>.
        let mut buf = ProtoBuffer::with_capacity(256);
        let mut visitor = DirectFieldVisitor::new(&mut buf);
        event.record(&mut visitor);

        Self {
            callsite_id: metadata.callsite(),
            timestamp_ns: Self::get_timestamp_nanos(),
            body_attrs_bytes: buf.into_bytes(),
        }
    }

    /// Get current timestamp in UNIX epoch nanoseconds.
    fn get_timestamp_nanos() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}
