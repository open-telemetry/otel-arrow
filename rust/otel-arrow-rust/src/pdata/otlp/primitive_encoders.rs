// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::pdata::otlp::PrecomputedSizes;

/// Encoder for boolean fields (wire type 0 - varint)
pub struct BooleanEncodedLen {
    pub tag: u32,
}

/// Encoder for string fields (wire type 2 - length-delimited)
pub struct StringEncodedLen {
    pub tag: u32,
}

/// Encoder for bytes fields (wire type 2 - length-delimited)
pub struct BytesEncodedLen {
    pub tag: u32,
}

/// Encoder for uint32 fields (wire type 0 - varint)
pub struct U32EncodedLen {
    pub tag: u32,
}

/// Encoder for uint64 fields (wire type 0 - varint)
pub struct U64EncodedLen {
    pub tag: u32,
}

/// Encoder for int32 fields (wire type 0 - varint)
pub struct I32EncodedLen {
    pub tag: u32,
}

/// Encoder for int64 fields (wire type 0 - varint)
pub struct I64EncodedLen {
    pub tag: u32,
}

/// Encoder for sint32 fields (wire type 0 - zigzag varint)
pub struct Sint32EncodedLen {
    pub tag: u32,
}

/// Encoder for sint64 fields (wire type 0 - zigzag varint)
pub struct Sint64EncodedLen {
    pub tag: u32,
}

/// Encoder for fixed32/sfixed32 fields (wire type 5 - 4 bytes)
pub struct Fixed32EncodedLen {
    pub tag: u32,
}

/// Encoder for fixed64/sfixed64 fields (wire type 1 - 8 bytes)
pub struct Fixed64EncodedLen {
    pub tag: u32,
}

/// Encoder for float fields (wire type 5 - 4 bytes)
pub struct FloatEncodedLen {
    pub tag: u32,
}

/// Encoder for double fields (wire type 1 - 8 bytes)
pub struct DoubleEncodedLen {
    pub tag: u32,
}

// ============================================================================
// GENERIC WIRE-TYPE BASED ENCODERS
// ============================================================================

/// Generic encoder for varint fields (wire type 0 - variable length)
/// Used for: bool, int32, int64, uint32, uint64, sint32, sint64, enum
pub struct VarintEncodedLen {
    pub tag: u32,
}

/// Generic encoder for length-delimited fields (wire type 2 - variable length)
/// Used for: string, bytes, embedded messages, packed repeated fields
pub struct LengthDelimitedEncodedLen {
    pub tag: u32,
}

/// Generic encoder for sint32/sint64 fields (wire type 0 - zigzag varint)
pub struct SintEncodedLen {
    pub tag: u32,
}

// ============================================================================
// VISITOR IMPLEMENTATIONS USING PROST ENCODING HELPERS
// ============================================================================

impl crate::pdata::BooleanVisitor<PrecomputedSizes> for BooleanEncodedLen {
    fn visit_bool(&mut self, mut arg: PrecomputedSizes, value: bool) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::bool::encoded_len(self.tag, &value));
        arg
    }
}

impl crate::pdata::StringVisitor<PrecomputedSizes> for StringEncodedLen {
    fn visit_string(&mut self, mut arg: PrecomputedSizes, value: &str) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::string::encoded_len(self.tag, value));
        arg
    }
}

impl crate::pdata::BytesVisitor<PrecomputedSizes> for BytesEncodedLen {
    fn visit_bytes(&mut self, mut arg: PrecomputedSizes, value: &[u8]) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::bytes::encoded_len(self.tag, value));
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

impl crate::pdata::F64Visitor<PrecomputedSizes> for FloatEncodedLen {
    fn visit_f64(&mut self, mut arg: PrecomputedSizes, value: f64) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::float::encoded_len(
            self.tag,
            &(value as f32),
        ));
        arg
    }
}

impl crate::pdata::F64Visitor<PrecomputedSizes> for DoubleEncodedLen {
    fn visit_f64(&mut self, mut arg: PrecomputedSizes, value: f64) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::double::encoded_len(self.tag, &value));
        arg
    }
}

// ============================================================================
// GENERIC WIRE-TYPE VISITOR IMPLEMENTATIONS
// ============================================================================

// VarintEncodedLen implementations for various types
impl crate::pdata::BooleanVisitor<PrecomputedSizes> for VarintEncodedLen {
    fn visit_bool(&mut self, mut arg: PrecomputedSizes, value: bool) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::bool::encoded_len(self.tag, &value));
        arg
    }
}

impl crate::pdata::U32Visitor<PrecomputedSizes> for VarintEncodedLen {
    fn visit_u32(&mut self, mut arg: PrecomputedSizes, value: u32) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::uint32::encoded_len(self.tag, &value));
        arg
    }
}

impl crate::pdata::U64Visitor<PrecomputedSizes> for VarintEncodedLen {
    fn visit_u64(&mut self, mut arg: PrecomputedSizes, value: u64) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::uint64::encoded_len(self.tag, &value));
        arg
    }
}

impl crate::pdata::I32Visitor<PrecomputedSizes> for VarintEncodedLen {
    fn visit_i32(&mut self, mut arg: PrecomputedSizes, value: i32) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::int32::encoded_len(self.tag, &value));
        arg
    }
}

impl crate::pdata::I64Visitor<PrecomputedSizes> for VarintEncodedLen {
    fn visit_i64(&mut self, mut arg: PrecomputedSizes, value: i64) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::int64::encoded_len(self.tag, &value));
        arg
    }
}

// LengthDelimitedEncodedLen implementations for string and bytes types
impl crate::pdata::StringVisitor<PrecomputedSizes> for LengthDelimitedEncodedLen {
    fn visit_string(&mut self, mut arg: PrecomputedSizes, value: &str) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::string::encoded_len(self.tag, value));
        arg
    }
}

impl crate::pdata::BytesVisitor<PrecomputedSizes> for LengthDelimitedEncodedLen {
    fn visit_bytes(&mut self, mut arg: PrecomputedSizes, value: &[u8]) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::bytes::encoded_len(self.tag, value));
        arg
    }
}

// SintEncodedLen implementations for zigzag encoded signed integers
impl crate::pdata::I32Visitor<PrecomputedSizes> for SintEncodedLen {
    fn visit_i32(&mut self, mut arg: PrecomputedSizes, value: i32) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::sint32::encoded_len(self.tag, &value));
        arg
    }
}

impl crate::pdata::I64Visitor<PrecomputedSizes> for SintEncodedLen {
    fn visit_i64(&mut self, mut arg: PrecomputedSizes, value: i64) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::sint64::encoded_len(self.tag, &value));
        arg
    }
}

// Fixed32EncodedLen implementations for 32-bit fixed-width types
impl crate::pdata::U32Visitor<PrecomputedSizes> for Fixed32EncodedLen {
    fn visit_u32(&mut self, mut arg: PrecomputedSizes, value: u32) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::fixed32::encoded_len(self.tag, &value));
        arg
    }
}

impl crate::pdata::I32Visitor<PrecomputedSizes> for Fixed32EncodedLen {
    fn visit_i32(&mut self, mut arg: PrecomputedSizes, value: i32) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::sfixed32::encoded_len(self.tag, &value));
        arg
    }
}

impl crate::pdata::F32Visitor<PrecomputedSizes> for Fixed32EncodedLen {
    fn visit_f32(&mut self, mut arg: PrecomputedSizes, value: f32) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::float::encoded_len(self.tag, &value));
        arg
    }
}

// Fixed64EncodedLen implementations for 64-bit fixed-width types
impl crate::pdata::U64Visitor<PrecomputedSizes> for Fixed64EncodedLen {
    fn visit_u64(&mut self, mut arg: PrecomputedSizes, value: u64) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::fixed64::encoded_len(self.tag, &value));
        arg
    }
}

impl crate::pdata::I64Visitor<PrecomputedSizes> for Fixed64EncodedLen {
    fn visit_i64(&mut self, mut arg: PrecomputedSizes, value: i64) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::sfixed64::encoded_len(self.tag, &value));
        arg
    }
}

impl crate::pdata::F64Visitor<PrecomputedSizes> for Fixed64EncodedLen {
    fn visit_f64(&mut self, mut arg: PrecomputedSizes, value: f64) -> PrecomputedSizes {
        arg.push_size(::prost::encoding::double::encoded_len(self.tag, &value));
        arg
    }
}
