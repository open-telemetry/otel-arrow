// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Log encoding and formatting for Tokio tracing events.
//!
//! The intermediate representation is LogRecord, includes the
//! primitive fields and static references. The remaining data are
//! placed in a partial OTLP encoding.

pub mod encoder;
pub mod formatter;
pub mod raw_log;

use bytes::Bytes;
use encoder::DirectFieldVisitor;
use otap_df_pdata::otlp::ProtoBuffer;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::callsite::Identifier;
use tracing::{Event, Level, Metadata};

pub use encoder::DirectLogRecordEncoder;
pub use formatter::{ConsoleWriter, RawLoggingLayer};

/// Optional key identifying the producing component.
/// TODO: This is re-exported, instead rename the underlying type.
pub type ProducerKey = crate::registry::MetricsKey;

/// A log record with structural metadata and pre-encoded body/attributes.
#[derive(Debug, Clone)]
pub struct LogRecord {
    /// Callsite identifier used to look up cached callsite info.
    pub callsite_id: Identifier,

    /// Timestamp in UNIX epoch nanoseconds.
    pub timestamp_ns: u64,

    /// Pre-encoded body and attributes in OTLP bytes.
    pub body_attrs_bytes: Bytes,

    /// Optional key identifying the producing component (for first-party logs).
    /// None for third-party logs from libraries.
    pub producer_key: Option<ProducerKey>,
}

/// Saved callsite information. This is information that can easily be
/// populated from Metadata, for example in a `register_callsite` hook
/// for building a map by Identifier.
#[derive(Debug, Clone)]
pub struct SavedCallsite {
    /// Tracing metadata.
    metadata: &'static Metadata<'static>,
}

impl SavedCallsite {
    /// Construct saved callsite information from tracing Metadata.
    #[must_use]
    pub fn new(metadata: &'static Metadata<'static>) -> Self {
        Self { metadata }
    }

    /// The level.
    #[must_use]
    pub fn level(&self) -> &Level {
        self.metadata.level()
    }

    /// The filename.
    #[must_use]
    pub fn file(&self) -> Option<&'static str> {
        self.metadata.file()
    }

    /// The line number.
    #[must_use]
    pub fn line(&self) -> Option<u32> {
        self.metadata.line()
    }

    /// The target (e.g., module).
    #[must_use]
    pub fn target(&self) -> &'static str {
        self.metadata.target()
    }

    /// The event name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.metadata.name()
    }
}

impl LogRecord {
    /// Construct a log record, partially encoding its dynamic content.
    #[must_use]
    pub fn new(event: &Event<'_>, producer_key: Option<ProducerKey>) -> Self {
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
            producer_key,
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

/// Write a LogRecord to stdout or stderr (based on level).
///
/// ERROR and WARN go to stderr, others go to stdout.
/// This is the same routing logic used by RawLoggingLayer.
pub fn raw_print(record: &LogRecord, callsite: &SavedCallsite) {
    ConsoleWriter::no_color().raw_print(record, callsite)
}
