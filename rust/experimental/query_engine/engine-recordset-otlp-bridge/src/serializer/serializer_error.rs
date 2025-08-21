// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::str::Utf8Error;

use thiserror::Error;

use crate::serializer::ProtobufWireType;

#[derive(Error, Debug)]
pub enum SerializerError {
    #[error(
        "Unexpected tag encountered with field number '{field_number}' and wire type '{wire_type:?}' while parsing {while_parsing}"
    )]
    UnexpectedTag {
        while_parsing: &'static str,
        field_number: u32,
        wire_type: ProtobufWireType,
    },

    #[error("Unexpected wire type '{0}' encountered")]
    UnknownWireType(u32),

    #[error("Unexpected field number '{0}' encountered")]
    InvalidFieldNumber(u32),

    #[error("Unexpected end of buffer encountered")]
    UnexpectedEndOfBuffer,

    #[error("Encountered feature '{0}' which has been deprecated")]
    Deprecated(&'static str),

    #[error("Encountered UTF8 error '{0}' while parsing a string")]
    Utf8(Utf8Error),

    #[error("{0}")]
    Message(&'static str),
}
