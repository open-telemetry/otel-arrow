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
use otel_arrow_dfe_pdata::otlp::common::{ProtoBuffer, StackProtoBuffer};
use serde::Serialize;
use serde::ser::Serializer;
use smallvec::SmallVec;
use std::fmt;
use tracing::callsite::Identifier;
use tracing::{Event, Level, Metadata};

pub use encoder::DirectLogRecordEncoder;
pub use encoder::ScopeToBytesMap;
pub use encoder::encode_export_logs_request;
pub use formatter::{
    AnsiCode, ColorMode, ConsoleWriter, RawLoggingLayer, StyledBufWriter,
    format_log_record_to_string,
};

/// Initial heap capacity for `LogRecord::new()`'s encoding phase.
///
/// Sized to the common case so the vast majority of log events need only one
/// allocation and no `Vec` growth. Less common larger records use normal
/// geometric `Vec` growth rather than making every record allocate the 2 KiB
/// maximum. After encoding the result is converted to `Bytes` via
/// `Bytes::from(Vec<u8>)`, which is zero-copy.
pub const LOG_ARGUMENTS_ENCODE_INITIAL: usize = 256;

/// Maximum size an encoded log event's body/attributes may grow to.
///
/// Rare events that overflow `LOG_ARGUMENTS_ENCODE_INITIAL` use normal
/// geometric `Vec` growth up to this limit rather than being truncated
/// immediately. Events that still don't fit are truncated and counted via
/// `dropped_attributes_count`, same as always.
pub const LOG_ARGUMENTS_ENCODE_LIMIT: usize = 2048;

/// Fixed stack buffer size for `StackLogRecord`'s encoding phase.
///
/// Used only by the synchronous, non-escaping `raw_error!` path, which never
/// converts to owned `Bytes`, so there's no reallocation and no benefit to
/// growing past this size; oversized events are simply truncated.
pub const LOG_ARGUMENTS_ENCODE_STACK: usize = 256;

/// Buffer size for rendering a log record to text (console/raw formatting).
///
/// Must stay large enough to hold the rendered text of the largest encoded
/// record (up to `LOG_ARGUMENTS_ENCODE_LIMIT` bytes of protobuf, which can
/// render to more text than its wire size); otherwise the already-truncated
/// `[...]`-marked value could be silently re-truncated here with no marker.
pub const LOG_BUFFER_SIZE: usize = LOG_ARGUMENTS_ENCODE_LIMIT * 2;

/// A log record with structural metadata and pre-encoded body/attributes.
/// A SystemTime value for the event is presumed to be external.
#[derive(Debug, Clone)]
pub struct LogRecord {
    /// Callsite identifier used to look up cached callsite info.
    pub callsite_id: Identifier,

    /// Pre-encoded body and attributes in OTLP bytes.  These bytes
    /// can be interpreted using the otel_arrow_dfe_pdata::views::otlp::bytes::RawLogRecord
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
/// Returned by the `__log_record_impl!` macro, which is an implementation
/// detail hidden from these docs. Callers choose how to consume it:
/// - [`as_view()`](Self::as_view) for zero-copy formatting (e.g., `raw_error!`)
/// - [`into_record()`](Self::into_record) to produce an owned `LogRecord`
///   with reference-counted `Bytes` storage
pub struct StackLogRecord {
    buf: StackProtoBuffer<LOG_ARGUMENTS_ENCODE_STACK>,
    callsite_id: Identifier,
    dropped_count: u32,
}

impl StackLogRecord {
    /// Construct from an event, encoding body/attributes on the stack.
    #[must_use]
    pub fn new(event: &Event<'_>) -> Self {
        let mut buf = StackProtoBuffer::<LOG_ARGUMENTS_ENCODE_STACK>::default();
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
    /// Pre-allocates a heap buffer sized to `LOG_ARGUMENTS_ENCODE_INITIAL`
    /// (no allocation beyond this for the common case) and allows it to grow,
    /// up to `LOG_ARGUMENTS_ENCODE_LIMIT` for rare oversized events.
    /// Attributes that still don't fit are counted via
    /// `dropped_attributes_count`.
    #[must_use]
    pub fn new(event: &Event<'_>, context: LogContext) -> Self {
        Self::new_bounded::<LOG_ARGUMENTS_ENCODE_INITIAL, LOG_ARGUMENTS_ENCODE_LIMIT>(
            event, context,
        )
    }

    /// Construct a log record encoding into a heap buffer pre-allocated to
    /// `INITIAL` bytes, allowed to grow up to `LIMIT` bytes.
    ///
    /// The pre-allocation ensures the common case needs no `Vec` growth.
    /// Attributes that don't fit even after growth are counted via
    /// `dropped_attributes_count`.
    #[must_use]
    pub fn new_bounded<const INITIAL: usize, const LIMIT: usize>(
        event: &Event<'_>,
        context: LogContext,
    ) -> Self {
        let metadata = event.metadata();

        let mut buf = ProtoBuffer::with_capacity_and_limit(INITIAL, LIMIT);
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
