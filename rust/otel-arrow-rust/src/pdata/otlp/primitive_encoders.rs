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

use crate::pdata::{
    BooleanVisitor, BytesVisitor, F64Visitor, I32Visitor, I64Visitor, SliceVisitor, StringVisitor,
    U32Visitor, U64Visitor,
};

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

/// Boolean field encoder with optional support
pub struct BooleanEncodedLen<const TAG: u32, const OPTION: bool> {}
/// String field encoder with optional support
pub struct StringEncodedLen<const TAG: u32, const OPTION: bool> {}
/// Bytes field encoder with optional support
pub struct BytesEncodedLen<const TAG: u32, const OPTION: bool> {}
/// Double field encoder with optional support
pub struct DoubleEncodedLen<const TAG: u32, const OPTION: bool> {}

/// U32 varint field encoder with optional support
pub struct U32VarintEncodedLen<const TAG: u32, const OPTION: bool> {}
/// U32 fixed field encoder with optional support
pub struct U32FixedEncodedLen<const TAG: u32, const OPTION: bool> {}
/// U64 varint field encoder with optional support
pub struct U64VarintEncodedLen<const TAG: u32, const OPTION: bool> {}
/// U64 fixed field encoder with optional support
pub struct U64FixedEncodedLen<const TAG: u32, const OPTION: bool> {}

/// I32 varint field encoder with optional support
pub struct I32VarintEncodedLen<const TAG: u32, const OPTION: bool> {}
/// I32 fixed field encoder with optional support
pub struct I32FixedEncodedLen<const TAG: u32, const OPTION: bool> {}
/// I64 varint field encoder with optional support
pub struct I64VarintEncodedLen<const TAG: u32, const OPTION: bool> {}
/// I64 fixed field encoder with optional support
pub struct I64FixedEncodedLen<const TAG: u32, const OPTION: bool> {}

/// Boolean slice encoder
pub struct SliceBooleanEncodedLen<const TAG: u32> {}
/// String slice encoder
pub struct SliceStringEncodedLen<const TAG: u32> {}
/// Bytes slice encoder
pub struct SliceBytesEncodedLen<const TAG: u32> {}
/// Double slice encoder
pub struct SliceDoubleEncodedLen<const TAG: u32> {}

/// U32 varint slice encoder
pub struct SliceU32VarintEncodedLen<const TAG: u32> {}
/// U32 fixed slice encoder
pub struct SliceU32FixedEncodedLen<const TAG: u32> {}
/// U64 varint slice encoder
pub struct SliceU64VarintEncodedLen<const TAG: u32> {}
/// U64 fixed slice encoder
pub struct SliceU64FixedEncodedLen<const TAG: u32> {}

/// I32 varint slice encoder
pub struct SliceI32VarintEncodedLen<const TAG: u32> {}
/// I32 fixed slice encoder
pub struct SliceI32FixedEncodedLen<const TAG: u32> {}
/// I64 varint slice encoder
pub struct SliceI64VarintEncodedLen<const TAG: u32> {}
/// I64 fixed slice encoder
pub struct SliceI64FixedEncodedLen<const TAG: u32> {}

// Boolean is always varint
impl<const TAG: u32, const OPTION: bool> BooleanVisitor<PrecomputedSizes>
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

// String is always length-delimited
impl<const TAG: u32, const OPTION: bool> StringVisitor<PrecomputedSizes>
    for StringEncodedLen<TAG, OPTION>
{
    fn visit_string(&mut self, arg: PrecomputedSizes, value: &str) -> PrecomputedSizes {
        let size = conditional_length_delimited_size::<TAG, OPTION>(value.len());
        push_calculated_size(arg, size)
    }
}

// Bytes is always length delimited
impl<const TAG: u32, const OPTION: bool> BytesVisitor<PrecomputedSizes>
    for BytesEncodedLen<TAG, OPTION>
{
    fn visit_bytes(&mut self, arg: PrecomputedSizes, value: &[u8]) -> PrecomputedSizes {
        let size = conditional_length_delimited_size::<TAG, OPTION>(value.len());
        push_calculated_size(arg, size)
    }
}

// Double is always fixed64
impl<const TAG: u32, const OPTION: bool> F64Visitor<PrecomputedSizes>
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

// U32Varint
impl<const TAG: u32, const OPTION: bool> U32Visitor<PrecomputedSizes>
    for U32VarintEncodedLen<TAG, OPTION>
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

// U32Fixed
impl<const TAG: u32, const OPTION: bool> U32Visitor<PrecomputedSizes>
    for U32FixedEncodedLen<TAG, OPTION>
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

// U64Varint
impl<const TAG: u32, const OPTION: bool> U64Visitor<PrecomputedSizes>
    for U64VarintEncodedLen<TAG, OPTION>
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

// U64Fixed
impl<const TAG: u32, const OPTION: bool> U64Visitor<PrecomputedSizes>
    for U64FixedEncodedLen<TAG, OPTION>
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

// I32Varint
impl<const TAG: u32, const OPTION: bool> I32Visitor<PrecomputedSizes>
    for I32VarintEncodedLen<TAG, OPTION>
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

// I64Varint
impl<const TAG: u32, const OPTION: bool> I64Visitor<PrecomputedSizes>
    for I64VarintEncodedLen<TAG, OPTION>
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

// I64Fixed
impl<const TAG: u32, const OPTION: bool> I64Visitor<PrecomputedSizes>
    for I64FixedEncodedLen<TAG, OPTION>
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

// I32Fixed
impl<const TAG: u32, const OPTION: bool> I32Visitor<PrecomputedSizes>
    for I32FixedEncodedLen<TAG, OPTION>
{
    fn visit_i32(&mut self, arg: PrecomputedSizes, value: i32) -> PrecomputedSizes {
        let size = if OPTION && value == 0 {
            0
        } else {
            calculate_primitive_size::<TAG, WIRE_TYPE_FIXED64>(4)
        };
        push_calculated_size(arg, size)
    }
}

// Implementations of SliceVisitor for the slice types
impl<const TAG: u32> SliceVisitor<PrecomputedSizes, u32> for SliceU32VarintEncodedLen<TAG> {
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[u32]) -> PrecomputedSizes {
        process_integer_slice::<TAG, u32, _>(arg, slice, varint_size32)
    }
}

impl<const TAG: u32> SliceVisitor<PrecomputedSizes, u64> for SliceU64VarintEncodedLen<TAG> {
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[u64]) -> PrecomputedSizes {
        process_integer_slice::<TAG, u64, _>(arg, slice, varint_size64)
    }
}

impl<const TAG: u32> SliceVisitor<PrecomputedSizes, i32> for SliceI32VarintEncodedLen<TAG> {
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[i32]) -> PrecomputedSizes {
        process_integer_slice::<TAG, i32, _>(arg, slice, signed_varint_size32)
    }
}

impl<const TAG: u32> SliceVisitor<PrecomputedSizes, i64> for SliceI64VarintEncodedLen<TAG> {
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[i64]) -> PrecomputedSizes {
        process_integer_slice::<TAG, i64, _>(arg, slice, signed_varint_size64)
    }
}

impl<const TAG: u32> SliceVisitor<PrecomputedSizes, f64> for SliceDoubleEncodedLen<TAG> {
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[f64]) -> PrecomputedSizes {
        process_integer_slice::<TAG, f64, _>(arg, slice, |_| 8)
    }
}

impl<const TAG: u32> SliceVisitor<PrecomputedSizes, bool> for SliceBooleanEncodedLen<TAG> {
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[bool]) -> PrecomputedSizes {
        process_integer_slice::<TAG, bool, _>(arg, slice, |_| 1)
    }
}

impl<const TAG: u32> SliceVisitor<PrecomputedSizes, String> for SliceStringEncodedLen<TAG> {
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[String]) -> PrecomputedSizes {
        process_length_delimited_slice::<TAG, String, _>(arg, slice, |value| value.len())
    }
}

impl<const TAG: u32> SliceVisitor<PrecomputedSizes, Vec<u8>> for SliceBytesEncodedLen<TAG> {
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[Vec<u8>]) -> PrecomputedSizes {
        process_length_delimited_slice::<TAG, Vec<u8>, _>(arg, slice, |value| value.len())
    }
}

// xx

impl<const TAG: u32> SliceVisitor<PrecomputedSizes, u32> for SliceU32FixedEncodedLen<TAG> {
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[u32]) -> PrecomputedSizes {
        process_integer_slice::<TAG, u32, _>(arg, slice, |_| 4)
    }
}

impl<const TAG: u32> SliceVisitor<PrecomputedSizes, u64> for SliceU64FixedEncodedLen<TAG> {
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[u64]) -> PrecomputedSizes {
        process_integer_slice::<TAG, u64, _>(arg, slice, |_| 8)
    }
}

impl<const TAG: u32> SliceVisitor<PrecomputedSizes, i32> for SliceI32FixedEncodedLen<TAG> {
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[i32]) -> PrecomputedSizes {
        process_integer_slice::<TAG, i32, _>(arg, slice, |_| 4)
    }
}

impl<const TAG: u32> SliceVisitor<PrecomputedSizes, i64> for SliceI64FixedEncodedLen<TAG> {
    fn visit_slice(&mut self, arg: PrecomputedSizes, slice: &[i64]) -> PrecomputedSizes {
        process_integer_slice::<TAG, i64, _>(arg, slice, |_| 8)
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

impl<V: StringVisitor<PrecomputedSizes>> StringVisitor<PrecomputedSizes> for &mut Accumulate<V> {
    fn visit_string(&mut self, mut arg: PrecomputedSizes, value: &str) -> PrecomputedSizes {
        arg = self.inner.visit_string(arg, value);
        self.total += arg.last();
        arg
    }
}

impl<V: BytesVisitor<PrecomputedSizes>> BytesVisitor<PrecomputedSizes> for &mut Accumulate<V> {
    fn visit_bytes(&mut self, mut arg: PrecomputedSizes, value: &[u8]) -> PrecomputedSizes {
        arg = self.inner.visit_bytes(arg, value);
        self.total += arg.last();
        arg
    }
}

impl<V: BooleanVisitor<PrecomputedSizes>> BooleanVisitor<PrecomputedSizes> for &mut Accumulate<V> {
    fn visit_bool(&mut self, mut arg: PrecomputedSizes, value: bool) -> PrecomputedSizes {
        arg = self.inner.visit_bool(arg, value);
        self.total += arg.last();
        arg
    }
}

impl<V: F64Visitor<PrecomputedSizes>> F64Visitor<PrecomputedSizes> for &mut Accumulate<V> {
    fn visit_f64(&mut self, mut arg: PrecomputedSizes, value: f64) -> PrecomputedSizes {
        arg = self.inner.visit_f64(arg, value);
        self.total += arg.last();
        arg
    }
}

impl<V: I32Visitor<PrecomputedSizes>> I32Visitor<PrecomputedSizes> for &mut Accumulate<V> {
    fn visit_i32(&mut self, mut arg: PrecomputedSizes, value: i32) -> PrecomputedSizes {
        arg = self.inner.visit_i32(arg, value);
        self.total += arg.last();
        arg
    }
}

impl<V: I64Visitor<PrecomputedSizes>> I64Visitor<PrecomputedSizes> for &mut Accumulate<V> {
    fn visit_i64(&mut self, mut arg: PrecomputedSizes, value: i64) -> PrecomputedSizes {
        arg = self.inner.visit_i64(arg, value);
        self.total += arg.last();
        arg
    }
}

impl<V: U32Visitor<PrecomputedSizes>> U32Visitor<PrecomputedSizes> for &mut Accumulate<V> {
    fn visit_u32(&mut self, mut arg: PrecomputedSizes, value: u32) -> PrecomputedSizes {
        arg = self.inner.visit_u32(arg, value);
        self.total += arg.last();
        arg
    }
}

impl<V: U64Visitor<PrecomputedSizes>> U64Visitor<PrecomputedSizes> for &mut Accumulate<V> {
    fn visit_u64(&mut self, mut arg: PrecomputedSizes, value: u64) -> PrecomputedSizes {
        arg = self.inner.visit_u64(arg, value);
        self.total += arg.last();
        arg
    }
}

impl<V: SliceVisitor<PrecomputedSizes, Primitive>, Primitive>
    SliceVisitor<PrecomputedSizes, Primitive> for &mut Accumulate<V>
{
    fn visit_slice(&mut self, mut arg: PrecomputedSizes, value: &[Primitive]) -> PrecomputedSizes {
        arg = self.inner.visit_slice(arg, value);
        self.total += arg.last();
        arg
    }
}
