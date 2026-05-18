// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

/// Errors returned by the STEF metrics codec.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// The OTLP metric type is not implemented by this codec slice.
    UnsupportedMetricType(&'static str),
    /// The OTLP data point has an unsupported value shape.
    UnsupportedDataPointValue,
    /// Attribute values are limited to scalar and bytes values in this first implementation.
    UnsupportedAttributeValue(&'static str),
    /// The STEF stream header is invalid or unsupported.
    InvalidHeader(&'static str),
    /// The STEF frame is malformed.
    InvalidFrame(&'static str),
    /// The STEF stream ended before the expected bytes were available.
    UnexpectedEof,
    /// A STEF dictionary reference points outside the active dictionary.
    InvalidRefNum,
    /// The STEF stream uses a value kind outside this implementation slice.
    UnsupportedStefValue(&'static str),
    /// A STEF frame or integer exceeded the supported range.
    ValueOutOfRange(&'static str),
    /// An OTAP Arrow view could not be built from the supplied payload.
    OtapView(String),
    /// OTAP Arrow encoding failed while building a direct decoded payload.
    OtapEncode(String),
    /// A view exposed non-UTF-8 bytes for a STEF string field.
    InvalidUtf8(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedMetricType(kind) => write!(f, "unsupported STEF metric type: {kind}"),
            Self::UnsupportedDataPointValue => write!(f, "unsupported STEF data point value"),
            Self::UnsupportedAttributeValue(kind) => {
                write!(f, "unsupported STEF attribute value: {kind}")
            }
            Self::InvalidHeader(reason) => write!(f, "invalid STEF header: {reason}"),
            Self::InvalidFrame(reason) => write!(f, "invalid STEF frame: {reason}"),
            Self::UnexpectedEof => write!(f, "unexpected end of STEF stream"),
            Self::InvalidRefNum => write!(f, "invalid STEF dictionary reference"),
            Self::UnsupportedStefValue(kind) => write!(f, "unsupported STEF value: {kind}"),
            Self::ValueOutOfRange(name) => write!(f, "STEF value out of range: {name}"),
            Self::OtapView(reason) => write!(f, "OTAP view error: {reason}"),
            Self::OtapEncode(reason) => write!(f, "OTAP encode error: {reason}"),
            Self::InvalidUtf8(field) => write!(f, "invalid UTF-8 in STEF string field: {field}"),
        }
    }
}

impl std::error::Error for Error {}
