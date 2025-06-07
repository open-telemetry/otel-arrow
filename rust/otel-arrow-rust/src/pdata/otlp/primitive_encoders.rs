// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains primitive-field encoding support for protobufs.

use crate::pdata::otlp::PrecomputedSizes;

/// Encoder for boolean fields (wire type 0 - varint)
pub struct BooleanEncodedLen {
    /// Protocol buffer tag number
    pub tag: u32,
}

/// Encoder for string fields (wire type 2 - length-delimited)
pub struct StringEncodedLen {
    /// Protocol buffer tag number
    pub tag: u32,
}

/// Encoder for bytes fields (wire type 2 - length-delimited)
pub struct BytesEncodedLen {
    /// Protocol buffer tag number
    pub tag: u32,
}

/// Encoder for uint32 fields (wire type 0 - varint)
pub struct U32EncodedLen {
    /// Protocol buffer tag number
    pub tag: u32,
}

/// Encoder for uint64 fields (wire type 0 - varint)
pub struct U64EncodedLen {
    /// Protocol buffer tag number
    pub tag: u32,
}

/// Encoder for int32 fields (wire type 0 - varint)
pub struct I32EncodedLen {
    /// Protocol buffer tag number
    pub tag: u32,
}

/// Encoder for int64 fields (wire type 0 - varint)
pub struct I64EncodedLen {
    /// Protocol buffer tag number
    pub tag: u32,
}

/// Encoder for fixed32/sfixed32 fields (wire type 5 - 4 bytes)
pub struct Fixed32EncodedLen {
    /// Protocol buffer tag number
    pub tag: u32,
}

/// Encoder for fixed64/sfixed64 fields (wire type 1 - 8 bytes)
pub struct Fixed64EncodedLen {
    /// Protocol buffer tag number
    pub tag: u32,
}

/// Encoder for double fields (wire type 1 - 8 bytes)
pub struct DoubleEncodedLen {
    /// Protocol buffer tag number
    pub tag: u32,
}

impl crate::pdata::BooleanVisitor<PrecomputedSizes> for BooleanEncodedLen {
    fn visit_bool(&mut self, mut arg: PrecomputedSizes, value: bool) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::bool::encoded_len(self.tag, &value));
        arg
    }
}

impl crate::pdata::StringVisitor<PrecomputedSizes> for StringEncodedLen {
    fn visit_string(&mut self, mut arg: PrecomputedSizes, value: &str) -> PrecomputedSizes {
        // Convert &str to String for prost API compatibility
        let string_value = value.to_string();
        arg.push_size(::prost::encoding::string::encoded_len(
            self.tag,
            &string_value,
        ));
        arg
    }
}

impl crate::pdata::BytesVisitor<PrecomputedSizes> for BytesEncodedLen {
    fn visit_bytes(&mut self, mut arg: PrecomputedSizes, value: &[u8]) -> PrecomputedSizes {
        // Convert &[u8] to Vec<u8> for prost API compatibility
        let bytes_value = value.to_vec();
        arg.push_size(::prost::encoding::bytes::encoded_len(
            self.tag,
            &bytes_value,
        ));
        arg
    }
}

impl crate::pdata::U32Visitor<PrecomputedSizes> for U32EncodedLen {
    fn visit_u32(&mut self, mut arg: PrecomputedSizes, value: u32) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::uint32::encoded_len(self.tag, &value));
        arg
    }
}

impl crate::pdata::U64Visitor<PrecomputedSizes> for U64EncodedLen {
    fn visit_u64(&mut self, mut arg: PrecomputedSizes, value: u64) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::uint64::encoded_len(self.tag, &value));
        arg
    }
}

impl crate::pdata::I32Visitor<PrecomputedSizes> for I32EncodedLen {
    fn visit_i32(&mut self, mut arg: PrecomputedSizes, value: i32) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::int32::encoded_len(self.tag, &value));
        arg
    }
}

impl crate::pdata::I64Visitor<PrecomputedSizes> for I64EncodedLen {
    fn visit_i64(&mut self, mut arg: PrecomputedSizes, value: i64) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::int64::encoded_len(self.tag, &value));
        arg
    }
}

impl crate::pdata::F64Visitor<PrecomputedSizes> for DoubleEncodedLen {
    fn visit_f64(&mut self, mut arg: PrecomputedSizes, value: f64) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::double::encoded_len(self.tag, &value));
        arg
    }
}
