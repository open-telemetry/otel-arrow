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
use otap_df_pdata::otlp::common::{ProtoBuffer, StackProtoBuffer};
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
    AnsiCode, ColorMode, ConsoleWriter, RawLoggingLayer, StyledBufWriter,
    format_log_record_to_string,
};

/// Inline buffer size for the encoding phase.
///
/// During encoding, `ProtoBuffer<LOG_ARGUMENTS_ENCODE_INLINE>` keeps data on the
/// stack.  After encoding the result is converted to `Bytes` for
/// cheap reference-counted storage.
pub const LOG_ARGUMENTS_ENCODE_INLINE: usize = 256;

/// Default buffer size for log formatting. Note that we truncate and
/// recognize dropped_attributes_count at the top-level of each log
/// record.
pub const LOG_BUFFER_SIZE: usize = 512;

/// A log record with structural metadata and pre-encoded body/attributes.
/// A SystemTime value for the event is presumed to be external.
#[derive(Debug, Clone)]
pub struct LogRecord {
    /// Callsite identifier used to look up cached callsite info.
    pub callsite_id: Identifier,

    /// Pre-encoded body and attributes in OTLP bytes.  These bytes
    /// can be interpreted using the otap_df_pdata::views::otlp::bytes::RawLogRecord
    /// in practice and/or parsed by a crate::proto::opentelemetry::logs::v1::LogRecord
    /// message object for testing.
    pub body_attrs_bytes: bytes::Bytes,

    /// Number of attribute fields dropped due to truncation (if any).
    pub dropped_attributes_count: u16,

    /// The context of this log record, typically pipeline and node context keys.
    pub context: LogContext,
}

/// Borrowed view of a log record for zero-copy formatting.
///
/// This is the common interface for formatting / printing log records.
/// `raw_error!` constructs one directly from a stack buffer (zero
/// allocation); `LogRecord` produces one via [`as_view()`](LogRecord::as_view).
#[derive(Debug, Clone)]
pub struct BorrowedLogRecord<'a> {
    /// Pre-encoded body and attributes in OTLP bytes.
    pub body_attrs_bytes: &'a [u8],
    /// Callsite information (level, target, name, file, line).
    pub callsite: SavedCallsite,
    /// Number of attribute fields dropped due to truncation.
    pub dropped_attributes_count: u32,
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

/// A log record encoded on the stack, not yet converted to `Bytes`.
///
/// Returned by [`__log_record_impl!`]. Callers choose how to consume it:
/// - [`as_view()`](Self::as_view) for zero-copy formatting (e.g., `raw_error!`)
/// - [`into_record()`](Self::into_record) to produce an owned `LogRecord`
///   with reference-counted `Bytes` storage
pub struct StackLogRecord {
    buf: StackProtoBuffer<LOG_ARGUMENTS_ENCODE_INLINE>,
    callsite_id: Identifier,
    dropped_count: u32,
}

impl StackLogRecord {
    /// Construct from an event, encoding body/attributes on the stack.
    #[must_use]
    pub fn new(event: &Event<'_>) -> Self {
        let mut buf = StackProtoBuffer::<LOG_ARGUMENTS_ENCODE_INLINE>::default();
        let dropped_count;
        {
            let mut visitor = DirectFieldVisitor::new(&mut buf);
            event.record(&mut visitor);
            dropped_count = visitor.dropped_count();
        }
        Self {
            buf,
            callsite_id: event.metadata().callsite(),
            dropped_count,
        }
    }

    /// Borrow as a [`BorrowedLogRecord`] for zero-copy formatting.
    #[must_use]
    pub fn as_view(&self) -> BorrowedLogRecord<'_> {
        BorrowedLogRecord {
            body_attrs_bytes: self.buf.as_ref(),
            callsite: SavedCallsite::new(self.callsite_id.0.metadata()),
            dropped_attributes_count: self.dropped_count,
        }
    }

    /// Convert into an owned [`LogRecord`], allocating `Bytes`.
    #[must_use]
    pub fn into_record(self, context: LogContext) -> LogRecord {
        LogRecord {
            dropped_attributes_count: self.dropped_count as u16,
            body_attrs_bytes: self.buf.to_bytes(),
            callsite_id: self.callsite_id,
            context,
        }
    }
}

impl LogRecord {
    /// Construct a log record with entity context, partially encoding its dynamic content.
    ///
    /// Uses stack-allocated inline storage for the protobuf buffer.
    /// Attributes that don't fit are counted via `dropped_attributes_count`.
    #[must_use]
    pub fn new(event: &Event<'_>, context: LogContext) -> Self {
        Self::new_bounded::<LOG_ARGUMENTS_ENCODE_INLINE>(event, context)
    }

    /// Construct a log record encoding into a heap buffer pre-allocated to
    /// `INLINE` bytes and bounded by `INLINE`.
    ///
    /// The pre-allocation ensures the encoder never grows the Vec on the hot
    /// path. Attributes that don't fit are counted via
    /// `dropped_attributes_count`.
    #[must_use]
    pub fn new_bounded<const INLINE: usize>(event: &Event<'_>, context: LogContext) -> Self {
        let metadata = event.metadata();

        let mut buf = ProtoBuffer::with_capacity_and_limit(INLINE, INLINE);
        let dropped_count;
        {
            let mut visitor = DirectFieldVisitor::new(&mut buf);
            event.record(&mut visitor);
            dropped_count = visitor.dropped_count();
        }

        Self {
            callsite_id: metadata.callsite(),
            dropped_attributes_count: dropped_count as u16,
            body_attrs_bytes: buf.into_bytes(),
            context,
        }
    }

    /// The callsite.
    #[must_use]
    pub fn callsite(&self) -> SavedCallsite {
        SavedCallsite::new(self.callsite_id.0.metadata())
    }

    /// Create a borrowed view for zero-copy formatting.
    #[must_use]
    pub fn as_view(&self) -> BorrowedLogRecord<'_> {
        BorrowedLogRecord {
            body_attrs_bytes: self.body_attrs_bytes.as_ref(),
            callsite: self.callsite(),
            dropped_attributes_count: self.dropped_attributes_count as u32,
        }
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
