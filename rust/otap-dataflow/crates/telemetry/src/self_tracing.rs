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
use tracing::callsite::Identifier;
use tracing::{Event, Level, Metadata};

use crate::registry::EntityKey;

pub use encoder::DirectLogRecordEncoder;
pub use formatter::{AnsiCode, BufWriter, ConsoleWriter, LOG_BUFFER_SIZE, RawLoggingLayer};

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
    pub body_attrs_bytes: Bytes,

    /// The pipeline entity key at the time this log record was created.
    /// Used to associate the log with the correct pipeline for telemetry.
    pub pipeline_entity_key: Option<EntityKey>,

    /// The node entity key at the time this log record was created.
    /// Used to associate the log with the correct node for telemetry.
    pub node_entity_key: Option<EntityKey>,
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
    /// Entity keys are not captured; use `new_with_context` for that.
    #[must_use]
    pub fn new(event: &Event<'_>) -> Self {
        Self::new_with_context(event, None, None)
    }

    /// Construct a log record with entity context, partially encoding its dynamic content.
    #[must_use]
    pub fn new_with_context(
        event: &Event<'_>,
        pipeline_entity_key: Option<EntityKey>,
        node_entity_key: Option<EntityKey>,
    ) -> Self {
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
            body_attrs_bytes: buf.into_bytes(),
            pipeline_entity_key,
            node_entity_key,
        }
    }

    /// The callsite.
    #[must_use]
    pub fn callsite(&self) -> SavedCallsite {
        SavedCallsite::new(self.callsite_id.0.metadata())
    }

    /// The format (without timestamp).
    #[must_use]
    pub fn format_without_timestamp(&self) -> String {
        ConsoleWriter::no_color().format_log_record(None, self)
    }
}
