// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Lossless single-column positions, with explicit rather than inferred types.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

// Leave room for JSON escaping plus envelope metadata under the 16 KiB limit.
const MAX_SCALAR_TEXT_BYTES: usize = 1024;

/// Supported scalar key types. SQL NULL and lossy numeric coercion are forbidden.
///
/// String keys require database ordering identical to Rust UTF-8 byte ordering.
/// Timestamp text retains source precision; timezone-less values mean UTC.
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", content = "value", deny_unknown_fields)]
pub enum ScalarValue {
    /// Signed 64-bit integral key.
    #[serde(rename = "int64")]
    Int64(i64),
    /// Unsigned 64-bit integral key, including values greater than i64::MAX.
    #[serde(rename = "uint64")]
    UInt64(u64),
    /// UTF-8 key ordered using binary collation, not locale or case folding.
    #[serde(rename = "string")]
    String(String),
    /// Timestamp compared chronologically after normalization to UTC.
    #[serde(rename = "timestamp")]
    Timestamp(String),
}

impl ScalarValue {
    /// Validates bounded text and parseable timestamps without including values in errors.
    pub fn validate(&self) -> Result<(), CursorError> {
        match self {
            Self::String(text) | Self::Timestamp(text) if text.len() > MAX_SCALAR_TEXT_BYTES => {
                return Err(CursorError::TextTooLong);
            }
            _ => {}
        }
        if let Self::Timestamp(text) = self {
            _ = super::otap::parse_utc_timestamp(text)
                .map_err(|_| CursorError::InvalidTimestamp)?;
        }
        Ok(())
    }

    /// Returns true only when both values have the same declared type.
    #[must_use]
    pub fn same_type(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    /// Compares two valid values without coercion, rounding, or lexical timestamp ordering.
    pub fn compare(&self, other: &Self) -> Result<Ordering, CursorError> {
        self.validate()?;
        other.validate()?;
        match (self, other) {
            (Self::Int64(left), Self::Int64(right)) => Ok(left.cmp(right)),
            (Self::UInt64(left), Self::UInt64(right)) => Ok(left.cmp(right)),
            (Self::String(left), Self::String(right)) => Ok(left.cmp(right)),
            (Self::Timestamp(left), Self::Timestamp(right)) => {
                let left = super::otap::parse_utc_timestamp(left)
                    .map_err(|_| CursorError::InvalidTimestamp)?;
                let right = super::otap::parse_utc_timestamp(right)
                    .map_err(|_| CursorError::InvalidTimestamp)?;
                Ok(left.cmp(&right))
            }
            _ => Err(CursorError::TypeMismatch),
        }
    }
}

impl fmt::Debug for ScalarValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self {
            Self::Int64(_) => "Int64",
            Self::UInt64(_) => "UInt64",
            Self::String(_) => "String",
            Self::Timestamp(_) => "Timestamp",
        };
        formatter.write_fmt(format_args!("{kind}(<redacted>)"))
    }
}

/// Invalid or incompatible cursor data. Diagnostics never contain cursor values.
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum CursorError {
    /// A position cannot be interpreted as a supported timestamp.
    #[error("database cursor timestamp is not a supported UTC timestamp")]
    InvalidTimestamp,
    /// The configured mode or scalar type does not match the supplied cursor.
    #[error("database cursor mode or scalar type does not match")]
    TypeMismatch,
    /// Text would exceed the bounded scalar cursor contract.
    #[error("scalar cursor text must not exceed 1024 UTF-8 bytes")]
    TextTooLong,
}
