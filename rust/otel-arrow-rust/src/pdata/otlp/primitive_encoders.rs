// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains primitive-field encoding support for protobufs.
//!
//! - EncodedLen impls for primitive visitors.
//! - Accumulate helper visitable
//!
//! See https://protobuf.dev/programming-guides/encoding/ for details
//! about the protobuf encoding.

use crate::pdata::otlp::PrecomputedSizes;

/// Wire type constants for protobuf encoding
const WIRE_TYPE_VARINT: u32 = 0; // Varint (int32, int64, uint32, uint64, sint32, sint64, bool)
const WIRE_TYPE_FIXED64: u32 = 1; // 64-bit (fixed64, sfixed64, double)
const WIRE_TYPE_LENGTH_DELIMITED: u32 = 2; // Length-delimited (string, bytes, embedded messages)
const WIRE_TYPE_FIXED32: u32 = 5; // 32-bit (fixed32, sfixed32, float)
const WIRE_TYPE_DONTCARE: u32 = 0; // In situations where we are computing size

/// Calculate the size of a varint encoded value
fn varint_size32(mut value: u32) -> usize {
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
fn varint_size64(mut value: u64) -> usize {
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
fn varint_usize(mut value: usize) -> usize {
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
fn signed_varint_size32(value: i32) -> usize {
    let zigzag = ((value << 1) ^ (value >> 31)) as u32;
    varint_size32(zigzag)
}

/// Calculate the size of a signed varint64 (zigzag encoded)
fn signed_varint_size64(value: i64) -> usize {
    let zigzag = ((value << 1) ^ (value >> 63)) as u64;
    varint_size64(zigzag)
}

/// Helper function to calculate encoded size for non-zero primitive
/// values This is not conditional, callers have to test the
/// appropriate equality function for default().
fn calculate_primitive_size<const TAG: u32, const WIRE: u32>(value_size: usize) -> usize {
    let tag_size = varint_size32(TAG << 3 | WIRE);
    tag_size + value_size
}

/// Helper function to calculate encoded size for length-delimited fields (strings, bytes)
/// This is conditional, for optional fields.
/// pub(crate) because called from generated code.
pub(crate) fn conditional_length_delimited_size<const TAG: u32, const OPTION: bool>(
    byte_length: usize,
) -> usize {
    if OPTION && byte_length == 0 {
        0
    } else {
        calculate_primitive_size::<TAG, WIRE_TYPE_LENGTH_DELIMITED>(
            varint_usize(byte_length) + byte_length,
        )
    }
}

/// Helper function to add calculated encoded size to PrecomputedSizes
fn push_calculated_size(mut arg: PrecomputedSizes, size: usize) -> PrecomputedSizes {
    arg.push_size(size);
    arg
}

/// Helper function to process slices with varint values, where wire type does not matter.
fn process_integer_slice<const TAG: u32, T, F>(
    mut arg: PrecomputedSizes,
    slice: &[T],
    size_fn: F,
) -> PrecomputedSizes
where
    F: Fn(T) -> usize,
    T: Copy,
{
    let mut size = 0;
    for value in slice {
        size += calculate_primitive_size::<TAG, WIRE_TYPE_DONTCARE>(size_fn(*value));
    }
    arg.push_size(size);
    arg
}

/// Helper function to process slices with length-delimited values
fn process_length_delimited_slice<const TAG: u32, T, F>(
    mut arg: PrecomputedSizes,
    slice: &[T],
    len_fn: F,
) -> PrecomputedSizes
where
    F: Fn(&T) -> usize,
{
    let mut size = 0;
    for value in slice {
        size += conditional_length_delimited_size::<TAG, false>(len_fn(value));
    }
    arg.push_size(size);
    arg
}

/// Encoder for boolean fields (wire type 0 - varint)
pub struct BooleanEncodedLen<const TAG: u32, const OPTION: bool> {}

/// Encoder for string fields (wire type 2 - length-delimited)
pub struct StringEncodedLen<const TAG: u32, const OPTION: bool> {}

/// Encoder for bytes fields (wire type 2 - length-delimited)
pub struct BytesEncodedLen<const TAG: u32, const OPTION: bool> {}

/// Encoder for uint32 fields (wire type 0 - varint)
pub struct U32EncodedLen<const TAG: u32, const OPTION: bool> {}

/// Encoder for uint64 fields (wire type 0 - varint)
pub struct U64EncodedLen<const TAG: u32, const OPTION: bool> {}

/// Encoder for int32 fields (wire type 0 - varint)
pub struct I32EncodedLen<const TAG: u32, const OPTION: bool> {}

/// Encoder for int64 fields (wire type 0 - varint)
pub struct I64EncodedLen<const TAG: u32, const OPTION: bool> {}

/// Encoder for fixed32/sfixed32 fields (wire type 5 - 4 bytes)
pub struct Fixed32EncodedLen<const TAG: u32, const OPTION: bool> {}

/// Encoder for fixed64/sfixed64 fields (wire type 1 - 8 bytes)
pub struct Fixed64EncodedLen<const TAG: u32, const OPTION: bool> {}

/// Encoder for sint32 fields (wire type 0 - varint with zigzag encoding)
pub struct Sint32EncodedLen<const TAG: u32, const OPTION: bool> {}

/// Encoder for sfixed64 fields (wire type 1 - 8 bytes, signed)
pub struct Sfixed64EncodedLen<const TAG: u32, const OPTION: bool> {}

/// Encoder for double fields (wire type 1 - 8 bytes)
pub struct DoubleEncodedLen<const TAG: u32, const OPTION: bool> {}

// Slice visitor types for repeated primitive fields

/// Slice visitor for repeated u32 fields in encoded length computation
pub struct SliceU32EncodedLen<const TAG: u32> {}

/// Slice visitor for repeated u64 fields in encoded length computation  
pub struct SliceU64EncodedLen<const TAG: u32> {}

/// Slice visitor for repeated i32 fields in encoded length computation
pub struct SliceI32EncodedLen<const TAG: u32> {}

/// Slice visitor for repeated i64 fields in encoded length computation
pub struct SliceI64EncodedLen<const TAG: u32> {}

/// Slice visitor for repeated f64 fields in encoded length computation
pub struct SliceDoubleEncodedLen<const TAG: u32> {}

/// Slice visitor for repeated f32 fields in encoded length computation
pub struct SliceFixed32EncodedLen<const TAG: u32> {}

/// Slice visitor for repeated fixed64 fields in encoded length computation
pub struct SliceFixed64EncodedLen<const TAG: u32> {}

/// Slice visitor for repeated bool fields in encoded length computation
pub struct SliceBooleanEncodedLen<const TAG: u32> {}

/// Slice visitor for repeated string fields in encoded length computation
pub struct SliceStringEncodedLen<const TAG: u32> {}

/// Slice visitor for repeated bytes fields in encoded length computation
pub struct SliceBytesEncodedLen<const TAG: u32> {}

impl<const TAG: u32, const OPTION: bool> crate::pdata::BooleanVisitor<PrecomputedSizes>
    for BooleanEncodedLen<TAG, OPTION>
{
    fn visit_bool(&mut self, arg: PrecomputedSizes, value: bool) -> PrecomputedSizes {
        // Boolean is encoded as varint: 1 byte for value (0 or 1)
        let size = if OPTION && !value {
            0
        } else {
            calculate_primitive_size::<TAG, WIRE_TYPE_VARINT>(1)
        };
        push_calculated_size(arg, size)
    }
}

impl<const TAG: u32, const OPTION: bool> crate::pdata::StringVisitor<PrecomputedSizes>
    for StringEncodedLen<TAG, OPTION>
{
    fn visit_string(&mut self, arg: PrecomputedSizes, value: &str) -> PrecomputedSizes {
        let size = conditional_length_delimited_size::<TAG, OPTION>(value.len());
        push_calculated_size(arg, size)
    }
}

impl<const TAG: u32, const OPTION: bool> crate::pdata::BytesVisitor<PrecomputedSizes>
    for BytesEncodedLen<TAG, OPTION>
{
    fn visit_bytes(&mut self, arg: PrecomputedSizes, value: &[u8]) -> PrecomputedSizes {
        let size = conditional_length_delimited_size::<TAG, OPTION>(value.len());
        push_calculated_size(arg, size)
    }
}

impl<const TAG: u32, const OPTION: bool> crate::pdata::U32Visitor<PrecomputedSizes>
    for U32EncodedLen<TAG, OPTION>
{
    fn visit_u32(&mut self, arg: PrecomputedSizes, value: u32) -> PrecomputedSizes {
        let size = if OPTION && value == 0 {
            0
        } else {
            calculate_primitive_size::<TAG, WIRE_TYPE_VARINT>(varint_size32(value))
        };
        push_calculated_size(arg, size)
    }
}

impl<const TAG: u32, const OPTION: bool> crate::pdata::U64Visitor<PrecomputedSizes>
    for U64EncodedLen<TAG, OPTION>
{
    fn visit_u64(&mut self, arg: PrecomputedSizes, value: u64) -> PrecomputedSizes {
        let size = if OPTION && value == 0 {
            0
        } else {
            calculate_primitive_size::<TAG, WIRE_TYPE_VARINT>(varint_size64(value))
        };
        push_calculated_size(arg, size)
    }
}

impl<const TAG: u32, const OPTION: bool> crate::pdata::U64Visitor<PrecomputedSizes>
    for Fixed64EncodedLen<TAG, OPTION>
{
    fn visit_u64(&mut self, arg: PrecomputedSizes, value: u64) -> PrecomputedSizes {
        let size = if OPTION && value == 0 {
            0
        } else {
            calculate_primitive_size::<TAG, WIRE_TYPE_FIXED64>(8)
        };
        push_calculated_size(arg, size)
    }
}

impl<const TAG: u32, const OPTION: bool> crate::pdata::U32Visitor<PrecomputedSizes>
    for Fixed32EncodedLen<TAG, OPTION>
{
    fn visit_u32(&mut self, arg: PrecomputedSizes, value: u32) -> PrecomputedSizes {
        let size = if OPTION && value == 0 {
            0
        } else {
            calculate_primitive_size::<TAG, WIRE_TYPE_FIXED32>(4)
        };
        push_calculated_size(arg, size)
    }
}

impl<const TAG: u32, const OPTION: bool> crate::pdata::I32Visitor<PrecomputedSizes>
    for Sint32EncodedLen<TAG, OPTION>
{
    fn visit_i32(&mut self, arg: PrecomputedSizes, value: i32) -> PrecomputedSizes {
        let size = if OPTION && value == 0 {
            0
        } else {
            calculate_primitive_size::<TAG, WIRE_TYPE_VARINT>(signed_varint_size32(value))
        };
        push_calculated_size(arg, size)
    }
}

impl<const TAG: u32, const OPTION: bool> crate::pdata::I32Visitor<PrecomputedSizes>
    for I32EncodedLen<TAG, OPTION>
{
    fn visit_i32(&mut self, arg: PrecomputedSizes, value: i32) -> PrecomputedSizes {
        let size = if OPTION && value == 0 {
            0
        } else {
            calculate_primitive_size::<TAG, WIRE_TYPE_VARINT>(signed_varint_size32(value))
        };
        push_calculated_size(arg, size)
    }
}

impl<const TAG: u32, const OPTION: bool> crate::pdata::I64Visitor<PrecomputedSizes>
    for I64EncodedLen<TAG, OPTION>
{
    fn visit_i64(&mut self, arg: PrecomputedSizes, value: i64) -> PrecomputedSizes {
        let size = if OPTION && value == 0 {
            0
        } else {
            calculate_primitive_size::<TAG, WIRE_TYPE_VARINT>(signed_varint_size64(value))
        };
        push_calculated_size(arg, size)
    }
}

impl<const TAG: u32, const OPTION: bool> crate::pdata::F64Visitor<PrecomputedSizes>
    for DoubleEncodedLen<TAG, OPTION>
{
    fn visit_f64(&mut self, arg: PrecomputedSizes, value: f64) -> PrecomputedSizes {
        let size = if OPTION && value == 0.0 {
            0
        } else {
            calculate_primitive_size::<TAG, WIRE_TYPE_FIXED64>(8)
        };
        push_calculated_size(arg, size)
    }
}

impl<const TAG: u32, const OPTION: bool> crate::pdata::I64Visitor<PrecomputedSizes>
    for Sfixed64EncodedLen<TAG, OPTION>
{
    fn visit_i64(&mut self, arg: PrecomputedSizes, value: i64) -> PrecomputedSizes {
        let size = if OPTION && value == 0 {
            0
        } else {
            calculate_primitive_size::<TAG, WIRE_TYPE_FIXED64>(8)
        };
        push_calculated_size(arg, size)
    }
}

// Implementations of SliceVisitor for the slice types
impl<const TAG: u32> crate::pdata::SliceVisitor<PrecomputedSizes, u32> for SliceU32EncodedLen<TAG> {
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[u32]) -> PrecomputedSizes {
        process_integer_slice::<TAG, u32, _>(arg, slice, varint_size32)
    }
}

impl<const TAG: u32> crate::pdata::SliceVisitor<PrecomputedSizes, u64> for SliceU64EncodedLen<TAG> {
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[u64]) -> PrecomputedSizes {
        process_integer_slice::<TAG, u64, _>(arg, slice, varint_size64)
    }
}

impl<const TAG: u32> crate::pdata::SliceVisitor<PrecomputedSizes, i32> for SliceI32EncodedLen<TAG> {
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[i32]) -> PrecomputedSizes {
        process_integer_slice::<TAG, i32, _>(arg, slice, signed_varint_size32)
    }
}

impl<const TAG: u32> crate::pdata::SliceVisitor<PrecomputedSizes, i64> for SliceI64EncodedLen<TAG> {
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[i64]) -> PrecomputedSizes {
        process_integer_slice::<TAG, i64, _>(arg, slice, signed_varint_size64)
    }
}

impl<const TAG: u32> crate::pdata::SliceVisitor<PrecomputedSizes, f64>
    for SliceDoubleEncodedLen<TAG>
{
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[f64]) -> PrecomputedSizes {
        process_integer_slice::<TAG, f64, _>(arg, slice, |_| 8)
    }
}

impl<const TAG: u32> crate::pdata::SliceVisitor<PrecomputedSizes, f32>
    for SliceFixed32EncodedLen<TAG>
{
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[f32]) -> PrecomputedSizes {
        process_integer_slice::<TAG, f32, _>(arg, slice, |_| 4)
    }
}

impl<const TAG: u32> crate::pdata::SliceVisitor<PrecomputedSizes, bool>
    for SliceBooleanEncodedLen<TAG>
{
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[bool]) -> PrecomputedSizes {
        process_integer_slice::<TAG, bool, _>(arg, slice, |_| 1)
    }
}

impl<const TAG: u32> crate::pdata::SliceVisitor<PrecomputedSizes, String>
    for SliceStringEncodedLen<TAG>
{
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[String]) -> PrecomputedSizes {
        process_length_delimited_slice::<TAG, String, _>(arg, slice, |value| value.len())
    }
}

impl<const TAG: u32> crate::pdata::SliceVisitor<PrecomputedSizes, Vec<u8>>
    for SliceBytesEncodedLen<TAG>
{
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[Vec<u8>]) -> PrecomputedSizes {
        process_length_delimited_slice::<TAG, Vec<u8>, _>(arg, slice, |value| value.len())
    }
}

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
    fn visit_slice(&mut self, mut arg: PrecomputedSizes, value: &[Primitive]) -> PrecomputedSizes {
        arg = self.inner.visit_slice(arg, value);
        self.total += arg.last();
        arg
    }
}
