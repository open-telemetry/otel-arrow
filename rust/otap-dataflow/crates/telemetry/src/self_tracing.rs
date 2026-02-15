// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Log encoding and formatting for Tokio tracing events.
//!
//! The intermediate representation is LogRecord, includes the
//! primitive fields and static references. The remaining data are
//! placed in a partial OTLP encoding.

pub mod encoder;
pub mod formatter;

use crate::registry::EntityKey;
use encoder::DirectFieldVisitor;
use otap_df_pdata::otlp::ProtoBuffer;
use serde::Serialize;
use serde::ser::Serializer;
use smallvec::SmallVec;
use std::fmt;
use tracing::callsite::Identifier;
use tracing::{Event, Level, Metadata};

pub use encoder::DirectLogRecordEncoder;
pub use encoder::ScopeToBytesMap;
pub use encoder::encode_export_logs_request;
pub use encoder::encode_resource_to_bytes;
pub use formatter::{
    AnsiCode, ColorMode, ConsoleWriter, LOG_BUFFER_SIZE, RawLoggingLayer, StyledBufWriter,
    format_log_record_to_string,
};

/// A log record with structural metadata and pre-encoded body/attributes.
/// A SystemTime value for the event is presumed to be external.
#[derive(Debug, Clone)]
pub struct LogRecord {
    /// Callsite identifier used to look up cached callsite info.
    pub callsite_id: Identifier,

    /// Pre-encoded body and attributes in OTLP bytes.  These bytes
    /// can be interrpreted using the otap_df_pdata::views::otlp::bytes::RawLogRecord
    /// in practice and/or parsed by a crate::proto::opentelemetry::logs::v1::LogRecord
    /// message object for testing.
    pub body_attrs_bytes: bytes::Bytes,

    /// The context of this log record, typically pipeline and node context keys.
    pub context: LogContext,
}

/// Context for log records: entity keys that identify scope attribute
/// sets in the telemetry registry.
pub type LogContext = SmallVec<[EntityKey; 1]>;

/// A log context function typically constructs context from
/// thread-local state.
pub type LogContextFn = fn() -> LogContext;

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
    pub const fn new(metadata: &'static Metadata<'static>) -> Self {
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
    /// Construct a log record with entity context, partially encoding its dynamic content.
    #[must_use]
    pub fn new(event: &Event<'_>, context: LogContext) -> Self {
        let metadata = event.metadata();

        // Encode body and attributes to bytes.
        // TODO(#1746): we could potentially avoid allocating for the intermediate
        // protobuf slice with work to support a fixed-size buffer and cursor
        // instead of a Vec<u8>.
        let mut buf = ProtoBuffer::with_capacity(256);
        let mut visitor = DirectFieldVisitor::new(&mut buf);
        event.record(&mut visitor);

        Self {
            callsite_id: metadata.callsite(),
            body_attrs_bytes: buf.into_bytes(),
            context,
        }
    }

    /// The callsite.
    #[must_use]
    pub fn callsite(&self) -> SavedCallsite {
        SavedCallsite::new(self.callsite_id.0.metadata())
    }
}

impl fmt::Display for LogRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Note: it _should_ be possible to format directly without the
        // intermediate string except Formatter does not implement the
        // Cursor that StyledBufWriter uses.
        write!(f, "{}", format_log_record_to_string(None, self))
    }
}

impl Serialize for LogRecord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
