// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains primitive-field encoding support for protobufs.

use crate::pdata::otlp::PrecomputedSizes;

/// Calculate the size of a varint encoded value
fn varint_size(mut value: u32) -> usize {
    if value == 0 {
        return 1;
    }
    let mut size = 0;
    while value > 0 {
        size += 1;
        value >>= 7;
    }
    size
}

/// Calculate the size of a varint64 encoded value
fn varint64_size(mut value: u64) -> usize {
    if value == 0 {
        return 1;
    }
    let mut size = 0;
    while value > 0 {
        size += 1;
        value >>= 7;
    }
    size
}

/// Calculate the size of a signed varint (zigzag encoded)
fn signed_varint_size(value: i32) -> usize {
    let zigzag = ((value << 1) ^ (value >> 31)) as u32;
    varint_size(zigzag)
}

/// Calculate the size of a signed varint64 (zigzag encoded)
fn signed_varint64_size(value: i64) -> usize {
    let zigzag = ((value << 1) ^ (value >> 63)) as u64;
    varint64_size(zigzag)
}

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

// Slice visitor types for repeated primitive fields

/// Slice visitor for repeated u32 fields in encoded length computation
pub struct SliceU32EncodedLen {
    /// The protobuf field tag
    pub tag: u32,
}

/// Slice visitor for repeated u64 fields in encoded length computation  
pub struct SliceU64EncodedLen {
    /// The protobuf field tag
    pub tag: u32,
}

/// Slice visitor for repeated i32 fields in encoded length computation
pub struct SliceI32EncodedLen {
    /// The protobuf field tag
    pub tag: u32,
}

/// Slice visitor for repeated i64 fields in encoded length computation
pub struct SliceI64EncodedLen {
    /// The protobuf field tag
    pub tag: u32,
}

/// Slice visitor for repeated f64 fields in encoded length computation
pub struct SliceDoubleEncodedLen {
    /// The protobuf field tag
    pub tag: u32,
}

/// Slice visitor for repeated f32 fields in encoded length computation
pub struct SliceFixed32EncodedLen {
    /// The protobuf field tag
    pub tag: u32,
}

/// Slice visitor for repeated fixed64 fields in encoded length computation
pub struct SliceFixed64EncodedLen {
    /// The protobuf field tag
    pub tag: u32,
}

/// Slice visitor for repeated bool fields in encoded length computation
pub struct SliceBooleanEncodedLen {
    /// The protobuf field tag
    pub tag: u32,
}

/// Slice visitor for repeated string fields in encoded length computation
pub struct SliceStringEncodedLen {
    /// The protobuf field tag
    pub tag: u32,
}

/// Slice visitor for repeated bytes fields in encoded length computation
pub struct SliceBytesEncodedLen {
    /// The protobuf field tag
    pub tag: u32,
}

impl crate::pdata::BooleanVisitor<PrecomputedSizes> for BooleanEncodedLen {
    fn visit_bool(&mut self, mut arg: PrecomputedSizes, value: bool) -> PrecomputedSizes {
        // Boolean is encoded as varint: 1 byte for value (0 or 1)
        let total = if value {
            let tag_size = varint_size(self.tag << 3); // wire_type = 0 for varint
            let value_size = 1; // booleans are always 1 byte
            tag_size + value_size
        } else {
            0
        };
        arg.push_size(total);
        arg
    }
}

impl crate::pdata::StringVisitor<PrecomputedSizes> for StringEncodedLen {
    fn visit_string(&mut self, mut arg: PrecomputedSizes, value: &str) -> PrecomputedSizes {
        // String is wire_type = 2 (length-delimited)
        let total = if value.len() != 0 {
            let tag_size = varint_size(self.tag << 3 | 2); // wire_type = 2
            let byte_len = value.len();
            let length_size = varint_size(byte_len as u32);
            tag_size + length_size + byte_len
        } else {
            0
        };
        arg.push_size(total);
        arg
    }
}

impl crate::pdata::BytesVisitor<PrecomputedSizes> for BytesEncodedLen {
    fn visit_bytes(&mut self, mut arg: PrecomputedSizes, value: &[u8]) -> PrecomputedSizes {
        // Bytes is wire_type = 2 (length-delimited)
        let total = if value.len() != 0 {
            let tag_size = varint_size(self.tag << 3 | 2); // wire_type = 2
            let byte_len = value.len();
            let length_size = varint_size(byte_len as u32);
            let value_size = byte_len;
            tag_size + length_size + value_size
        } else {
            0
        };
        arg.push_size(total);
        arg
    }
}

impl crate::pdata::U32Visitor<PrecomputedSizes> for U32EncodedLen {
    fn visit_u32(&mut self, mut arg: PrecomputedSizes, value: u32) -> PrecomputedSizes {
        // u32 is wire_type = 0 (varint)
        let total = if value != 0 {
            let tag_size = varint_size(self.tag << 3); // wire_type = 0
            let value_size = varint_size(value);
            tag_size + value_size
        } else {
            0
        };
        arg.push_size(total);
        arg
    }
}

impl crate::pdata::U64Visitor<PrecomputedSizes> for U64EncodedLen {
    fn visit_u64(&mut self, mut arg: PrecomputedSizes, value: u64) -> PrecomputedSizes {
        // u64 is wire_type = 0 (varint)
        let total = if value != 0 {
            let tag_size = varint_size(self.tag << 3); // wire_type = 0
            let value_size = varint64_size(value);
            tag_size + value_size
        } else {
            0
        };
        arg.push_size(total);
        arg
    }
}

impl crate::pdata::I32Visitor<PrecomputedSizes> for I32EncodedLen {
    fn visit_i32(&mut self, mut arg: PrecomputedSizes, value: i32) -> PrecomputedSizes {
        // i32 is wire_type = 0 (varint with zigzag encoding)
        let total = if value != 0 {
            let tag_size = varint_size(self.tag << 3); // wire_type = 0
            let value_size = signed_varint_size(value);
            tag_size + value_size
        } else {
            0
        };
        arg.push_size(total);
        arg
    }
}

impl crate::pdata::I64Visitor<PrecomputedSizes> for I64EncodedLen {
    fn visit_i64(&mut self, mut arg: PrecomputedSizes, value: i64) -> PrecomputedSizes {
        // i64 is wire_type = 0 (varint with zigzag encoding)
        let total = if value != 0 {
            let tag_size = varint_size(self.tag << 3); // wire_type = 0
            let value_size = signed_varint64_size(value);
            tag_size + value_size
        } else {
            0
        };
        arg.push_size(total);
        arg
    }
}

impl crate::pdata::F64Visitor<PrecomputedSizes> for DoubleEncodedLen {
    fn visit_f64(&mut self, mut arg: PrecomputedSizes, value: f64) -> PrecomputedSizes {
        // f64 is wire_type = 1 (fixed64 - 8 bytes)
        let total = if value != 0.0 {
            let tag_size = varint_size(self.tag << 3 | 1); // wire_type = 1
            let value_size = 8; // f64 is always 8 bytes
            tag_size + value_size
        } else {
            0
        };
        arg.push_size(total);
        arg
    }
}

// Implementations of SliceVisitor for the slice types
impl crate::pdata::SliceVisitor<PrecomputedSizes, u32> for SliceU32EncodedLen {
    fn visit_slice(&mut self, mut arg: PrecomputedSizes, slice: &[u32]) -> PrecomputedSizes {
        for value in slice {
            let tag_size = varint_size(self.tag << 3); // wire_type = 0
            let value_size = varint_size(*value);
            arg.push_size(tag_size + value_size);
        }
        arg
    }
}

impl crate::pdata::SliceVisitor<PrecomputedSizes, u64> for SliceU64EncodedLen {
    fn visit_slice(&mut self, mut arg: PrecomputedSizes, slice: &[u64]) -> PrecomputedSizes {
        for value in slice {
            let tag_size = varint_size(self.tag << 3); // wire_type = 0
            let value_size = varint64_size(*value);
            arg.push_size(tag_size + value_size);
        }
        arg
    }
}

impl crate::pdata::SliceVisitor<PrecomputedSizes, i32> for SliceI32EncodedLen {
    fn visit_slice(&mut self, mut arg: PrecomputedSizes, slice: &[i32]) -> PrecomputedSizes {
        for value in slice {
            let tag_size = varint_size(self.tag << 3); // wire_type = 0
            let value_size = signed_varint_size(*value);
            arg.push_size(tag_size + value_size);
        }
        arg
    }
}

impl crate::pdata::SliceVisitor<PrecomputedSizes, i64> for SliceI64EncodedLen {
    fn visit_slice(&mut self, mut arg: PrecomputedSizes, slice: &[i64]) -> PrecomputedSizes {
        for value in slice {
            let tag_size = varint_size(self.tag << 3); // wire_type = 0
            let value_size = signed_varint64_size(*value);
            arg.push_size(tag_size + value_size);
        }
        arg
    }
}

impl crate::pdata::SliceVisitor<PrecomputedSizes, f64> for SliceDoubleEncodedLen {
    fn visit_slice(&mut self, mut arg: PrecomputedSizes, slice: &[f64]) -> PrecomputedSizes {
        for _value in slice {
            let tag_size = varint_size(self.tag << 3 | 1); // wire_type = 1
            let value_size = 8; // f64 is always 8 bytes
            arg.push_size(tag_size + value_size);
        }
        arg
    }
}

impl crate::pdata::SliceVisitor<PrecomputedSizes, f32> for SliceFixed32EncodedLen {
    fn visit_slice(&mut self, mut arg: PrecomputedSizes, slice: &[f32]) -> PrecomputedSizes {
        for _value in slice {
            let tag_size = varint_size(self.tag << 3 | 5); // wire_type = 5 (fixed32)
            let value_size = 4; // f32 is always 4 bytes
            arg.push_size(tag_size + value_size);
        }
        arg
    }
}

impl crate::pdata::SliceVisitor<PrecomputedSizes, bool> for SliceBooleanEncodedLen {
    fn visit_slice(&mut self, mut arg: PrecomputedSizes, slice: &[bool]) -> PrecomputedSizes {
        for _value in slice {
            let tag_size = varint_size(self.tag << 3); // wire_type = 0
            let value_size = 1; // bool is always 1 byte
            arg.push_size(tag_size + value_size);
        }
        arg
    }
}

impl crate::pdata::SliceVisitor<PrecomputedSizes, String> for SliceStringEncodedLen {
    fn visit_slice(&mut self, mut arg: PrecomputedSizes, slice: &[String]) -> PrecomputedSizes {
        for value in slice {
            let tag_size = varint_size(self.tag << 3 | 2); // wire_type = 2
            let byte_len = value.len();
            let length_size = varint_size(byte_len as u32);
            let value_size = byte_len;
            arg.push_size(tag_size + length_size + value_size);
        }
        arg
    }
}

impl crate::pdata::SliceVisitor<PrecomputedSizes, Vec<u8>> for SliceBytesEncodedLen {
    fn visit_slice(&mut self, mut arg: PrecomputedSizes, slice: &[Vec<u8>]) -> PrecomputedSizes {
        for value in slice {
            let tag_size = varint_size(self.tag << 3 | 2); // wire_type = 2
            let byte_len = value.len();
            let length_size = varint_size(byte_len as u32);
            let value_size = byte_len;
            arg.push_size(tag_size + length_size + value_size);
        }
        arg
    }
}

//

/// Accumulate is a wrapper that sums the sizes from a child visitor.
pub struct Accumulate<V> {
    /// The inner visitor
    pub inner: V,
    /// The children's subtotal
    pub total: usize,
}

impl<V> Accumulate<V> {
    /// Accumulate for children of V
    pub fn new(inner: V) -> Self {
        Self { inner, total: 0 }
    }
}

// Implement all primitive visitor traits for Accumulate wrapper
// These delegate to the inner visitor and accumulate the size difference

impl<V: crate::pdata::StringVisitor<PrecomputedSizes>> crate::pdata::StringVisitor<PrecomputedSizes>
    for &mut Accumulate<V>
{
    fn visit_string(&mut self, mut arg: PrecomputedSizes, value: &str) -> PrecomputedSizes {
        arg = self.inner.visit_string(arg, value);
        self.total += arg.last();
        arg
    }
}

impl<V: crate::pdata::BytesVisitor<PrecomputedSizes>> crate::pdata::BytesVisitor<PrecomputedSizes>
    for &mut Accumulate<V>
{
    fn visit_bytes(&mut self, mut arg: PrecomputedSizes, value: &[u8]) -> PrecomputedSizes {
        arg = self.inner.visit_bytes(arg, value);
        self.total += arg.last();
        arg
    }
}

impl<V: crate::pdata::I32Visitor<PrecomputedSizes>> crate::pdata::I32Visitor<PrecomputedSizes>
    for &mut Accumulate<V>
{
    fn visit_i32(&mut self, mut arg: PrecomputedSizes, value: i32) -> PrecomputedSizes {
        arg = self.inner.visit_i32(arg, value);
        self.total += arg.last();
        arg
    }
}

impl<V: crate::pdata::I64Visitor<PrecomputedSizes>> crate::pdata::I64Visitor<PrecomputedSizes>
    for &mut Accumulate<V>
{
    fn visit_i64(&mut self, mut arg: PrecomputedSizes, value: i64) -> PrecomputedSizes {
        arg = self.inner.visit_i64(arg, value);
        self.total += arg.last();
        arg
    }
}

impl<V: crate::pdata::U32Visitor<PrecomputedSizes>> crate::pdata::U32Visitor<PrecomputedSizes>
    for &mut Accumulate<V>
{
    fn visit_u32(&mut self, mut arg: PrecomputedSizes, value: u32) -> PrecomputedSizes {
        arg = self.inner.visit_u32(arg, value);
        self.total += arg.last();
        arg
    }
}

impl<V: crate::pdata::U64Visitor<PrecomputedSizes>> crate::pdata::U64Visitor<PrecomputedSizes>
    for &mut Accumulate<V>
{
    fn visit_u64(&mut self, mut arg: PrecomputedSizes, value: u64) -> PrecomputedSizes {
        arg = self.inner.visit_u64(arg, value);
        self.total += arg.last();
        arg
    }
}

impl<V: crate::pdata::F64Visitor<PrecomputedSizes>> crate::pdata::F64Visitor<PrecomputedSizes>
    for &mut Accumulate<V>
{
    fn visit_f64(&mut self, mut arg: PrecomputedSizes, value: f64) -> PrecomputedSizes {
        arg = self.inner.visit_f64(arg, value);
        self.total += arg.last();
        arg
    }
}

impl<V: crate::pdata::BooleanVisitor<PrecomputedSizes>>
    crate::pdata::BooleanVisitor<PrecomputedSizes> for &mut Accumulate<V>
{
    fn visit_bool(&mut self, mut arg: PrecomputedSizes, value: bool) -> PrecomputedSizes {
        arg = self.inner.visit_bool(arg, value);
        self.total += arg.last();
        arg
    }
}

impl<V: crate::pdata::SliceVisitor<PrecomputedSizes, Primitive>, Primitive>
    crate::pdata::SliceVisitor<PrecomputedSizes, Primitive> for &mut Accumulate<V>
{
    fn visit_slice(&mut self, arg: PrecomputedSizes, _value: &[Primitive]) -> PrecomputedSizes {
        // @@@
        // arg = self.inner.visit_bytes(arg, value);
        // self.total += arg.last();
        // arg
        // let before_size = arg.total_size();
        // let result = self.inner.visit_slice(arg, value);
        // let after_size = result.total_size();
        // self.total += after_size - before_size;
        // result
        arg
    }
}
