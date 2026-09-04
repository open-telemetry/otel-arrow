// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Database-neutral values and result metadata.

use std::fmt;
use std::mem::size_of;

/// A database scalar normalized without losing source precision.
#[derive(Clone, PartialEq)]
pub enum CellValue {
    /// SQL `NULL`.
    Null,
    /// Boolean.
    Bool(bool),
    /// Signed integer.
    Int64(i64),
    /// Unsigned integer.
    UInt64(u64),
    /// Exact decimal text preserving source precision.
    Decimal(String),
    /// Finite IEEE-754 value.
    Float64(f64),
    /// UTF-8 text.
    String(String),
    /// Binary bytes.
    Bytes(Vec<u8>),
    /// Calendar date text.
    Date(String),
    /// Timestamp without source timezone.
    Timestamp(String),
    /// Timestamp with source timezone.
    TimestampTz(String),
    /// Database interval text.
    Interval(String),
    /// Valid JSON text.
    Json(String),
    /// UUID text.
    Uuid(String),
}

impl CellValue {
    /// Returns the normalized payload size used for bounded-memory accounting.
    #[must_use]
    pub fn normalized_size(&self) -> u64 {
        match self {
            Self::Null | Self::Bool(_) => 1,
            Self::Int64(_) | Self::UInt64(_) | Self::Float64(_) => 8,
            Self::Decimal(value)
            | Self::String(value)
            | Self::Date(value)
            | Self::Timestamp(value)
            | Self::TimestampTz(value)
            | Self::Interval(value)
            | Self::Json(value)
            | Self::Uuid(value) => value.len() as u64,
            Self::Bytes(value) => value.len() as u64,
        }
    }
}

impl fmt::Debug for CellValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => formatter.write_str("Null"),
            Self::Bool(_) => formatter.write_str("Bool(<redacted>)"),
            Self::Int64(_) => formatter.write_str("Int64(<redacted>)"),
            Self::UInt64(_) => formatter.write_str("UInt64(<redacted>)"),
            Self::Float64(_) => formatter.write_str("Float64(<redacted>)"),
            Self::Decimal(value) => redacted_text(formatter, "Decimal", value),
            Self::String(value) => redacted_text(formatter, "String", value),
            Self::Bytes(value) => formatter
                .debug_tuple("Bytes")
                .field(&format_args!("<redacted:{} bytes>", value.len()))
                .finish(),
            Self::Date(value) => redacted_text(formatter, "Date", value),
            Self::Timestamp(value) => redacted_text(formatter, "Timestamp", value),
            Self::TimestampTz(value) => redacted_text(formatter, "TimestampTz", value),
            Self::Interval(value) => redacted_text(formatter, "Interval", value),
            Self::Json(value) => redacted_text(formatter, "Json", value),
            Self::Uuid(value) => redacted_text(formatter, "Uuid", value),
        }
    }
}

fn redacted_text(formatter: &mut fmt::Formatter<'_>, name: &str, value: &str) -> fmt::Result {
    formatter
        .debug_tuple(name)
        .field(&format_args!("<redacted:{} bytes>", value.len()))
        .finish()
}

/// Metadata shared by every row in a database result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColumnMetadata {
    /// Result-set column name.
    pub name: String,
    /// Stable adapter-reported database type name.
    pub source_type: String,
    /// Whether the database reports this column as nullable.
    pub nullable: bool,
}

/// One database-neutral row whose values correspond to result metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    /// Ordered values matching the result columns.
    pub values: Vec<CellValue>,
}

impl Row {
    /// Returns conservative normalized storage used for bounded-memory accounting.
    #[must_use]
    pub fn normalized_size(&self) -> u64 {
        let allocated = size_of::<Self>()
            .saturating_add(self.values.len().saturating_mul(size_of::<CellValue>()))
            as u64;
        self.values.iter().fold(allocated, |size, value| {
            size.saturating_add(value.normalized_size())
        })
    }
}
