// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! various types & helper functions for decoding serialized protobuf data

use std::cell::Cell;
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::rc::Rc;

use crate::proto::consts::wire_types;

/// Clones the parser, sharing the underlying buffer and interior-mutability state.
/// The cloned instance will share offsets and position with the original.
impl<T: FieldRanges> Clone for ProtoBytesParser<'_, T> {
    fn clone(&self) -> Self {
        Self {
            buf: self.buf,
            pos: self.pos.clone(),
            field_ranges: self.field_ranges.clone(),
        }
    }
}
/// `FieldRanges` defines the interface used by a protobuf parser to record and retrieve
/// field offsets within a serialized message buffer.
///
/// This trait is typically implemented for specific protobuf message types and used by a
/// parser like `ProtoBytesParser`, which scans the buffer once and calls `set_field_offset`
/// as fields are encountered. Later, accessors (e.g., view structs) call `get_field_offset`
/// or `get_repeated_field_offset` to retrieve the byte ranges corresponding to particular
/// fields.
///
/// Implementations of this trait are expected to use interior mutability (e.g., `Cell`)
/// so that parsing can occur even when the parser is accessed via a shared reference.
/// This makes it possible to lazily parse fields on-demand, without requiring full
/// up-front decoding.
///
/// For best performance, implementations should try to be light weight and avoid heap allocations
/// if possible.
pub trait FieldRanges {
    /// Creates a new, empty instance of this `FieldOffsets` implementation.
    fn new() -> Self;

    /// Returns the offset of the given scalar field number, if known.
    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)>;

    /// Records the offset of a field that was encountered during parsing.
    ///
    /// Called by the parser as it scans the buffer. The implementation may
    /// choose to store only the first offset for repeated fields, or all offsets.
    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize);
}

/// helper convert an Option of `NonZeroRange` into an `Option<(usize, usize)>` to adapt internal
/// range to expected return type in `FieldOffset`
#[inline]
#[must_use]
pub fn from_option_nonzero_range_to_primitive(
    range: Option<(NonZeroUsize, NonZeroUsize)>,
) -> Option<(usize, usize)> {
    range.map(|(start, end)| (start.get(), end.get()))
}

/// helper to convert the arguments of in the `FieldOffset` function into the internal type used
/// by many of it's impls.
#[inline]
#[must_use]
pub fn to_nonzero_range(start: usize, end: usize) -> Option<(NonZeroUsize, NonZeroUsize)> {
    Some((NonZeroUsize::new(start)?, NonZeroUsize::new(end)?))
}

/// `ProtoBytesParser` is a generic struct that encapsulates the logic for iterating through
/// a buffer containing a serialized protobuf message, identifying the offsets of its fields.
/// The intention is that we'll only need to pass over the buffer once, regardless of in which
/// order the fields are accessed.
///
/// Typically, an instance of this type is embedded within a higher-level type that implements
/// one of the pdata view traits. The buffer is parsed lazily as fields are accessed via the
/// view's methods.
///
/// This type collaborates with an implementation of the `FieldRanges` trait — often specific
/// to the message being parsed — by recording the offsets of fields as they are encountered,
/// and using these offsets to return byte slices for requested fields.
///
/// # Notes
/// - Multiple `ProtoBytesParser` instances may reference the same buffer. For example, a buffer
///   containing a `LogRecord` message may be shared by both a `LogsView` and an iterator over
///   the `LogRecord`'s attributes.
/// - Parsing may need to occur even when the container of this parser is accessed via a
///   shared or immutable reference. For example, view methods like `LogDataView::severity_text()`
///   might take `&self`, but parsing requires updating offsets.
///
/// To support these patterns, this type uses interior mutability for shared position and
/// field offset state. As such this type is clearly not `Send`.
///
pub struct ProtoBytesParser<'a, T: FieldRanges> {
    /// buffer containing the serialized proto message being parsed
    buf: &'a [u8],

    /// offset within the buffer
    pos: Rc<Cell<usize>>,

    /// offsets within the buffer of fields that have been encountered as the buffer is parsed
    field_ranges: Rc<T>,
}

impl<'a, T> ProtoBytesParser<'a, T>
where
    T: FieldRanges,
{
    /// Create a new instance of `ProtoBytesParser`
    #[must_use]
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            buf,
            pos: Rc::new(Cell::new(0)),
            field_ranges: Rc::new(T::new()),
        }
    }

    /// Advances the parser to the specified scalar field and returns its value as a byte slice,
    /// if found. Parsing proceeds from the current position in the buffer.
    #[inline]
    #[must_use]
    pub fn advance_to_find_field(&self, field_num: u64) -> Option<&'a [u8]> {
        // Check if the field offset is already cached before entering the parsing loop
        if let Some((start, end)) = self.field_ranges.get_field_range(field_num) {
            return Some(&self.buf[start..end]);
        }

        // Field offset is not yet known, so we need to parse the buffer
        // This loop advances parsing by one field each iteration until either the field is found
        // or the end of the buffer is reached.
        loop {
            let pos = self.pos.get();
            if pos >= self.buf.len() {
                // end of buffer reached, field not found
                break;
            }

            // parse tag & advance
            let (tag, next_pos) = read_varint(self.buf, pos)?;
            let field = tag >> 3;
            let wire_type = tag & 7;

            let (start, end) = field_value_range(self.buf, wire_type, next_pos)?;
            self.pos.set(end);

            // save the offset of the field we've encountered
            self.field_ranges
                .set_field_range(field, wire_type, start, end);

            // Check if this is the field we're looking for
            if field == field_num {
                return Some(&self.buf[start..end]);
            }
        }

        None
    }

    /// Advances the parser to find one of the fields specified in the `field_nums` argument.
    /// If found, it returns the the byte slice containing the value for this field and the
    /// field number as a tuple.
    #[must_use]
    pub fn advance_to_find_oneof(&self, field_nums: &[u64]) -> Option<(&'a [u8], u64)> {
        for field_num in field_nums {
            if let Some(buf) = self.advance_to_find_field(*field_num) {
                return Some((buf, *field_num));
            }
        }

        None
    }
}

/// return the range of the positions in the byte slice containing values. The range is determined
/// from the wire type.
#[inline]
pub(crate) fn field_value_range(buf: &[u8], wire_type: u64, pos: usize) -> Option<(usize, usize)> {
    let range = match wire_type {
        wire_types::VARINT => {
            // /// TODO this could maybe be read_variant bytes for faster perf
            let (_, p) = read_varint(buf, pos)?;
            (pos, p)
        }

        wire_types::LEN => {
            let (len, p) = read_varint(buf, pos)?;
            let end = p.checked_add(len as usize)?;
            (p, end)
        }
        wire_types::FIXED64 => (pos, pos + 8),
        wire_types::FIXED32 => (pos, pos + 4),
        _ => return None,
    };

    Some(range)
}

/// `RepeatedFieldProtoBytesParser` is an iterator over byte slices for some field (represented by
/// `field_num` member). It parses the protobuf serialized message as it produces slices of the
/// underlying buffer, and keeps track of the ranges for other field types it encounters.
///
/// Typically an instance of this type is embedded within an adapter iterator that yields view
/// implementations for a repeated field.
pub struct RepeatedFieldProtoBytesParser<'a, T: FieldRanges> {
    /// buffer containing the serialized proto message being parsed
    buf: &'a [u8],

    // `pos` is a shared offset representing how far parsing has progressed in the buffer
    pos: Rc<Cell<usize>>,

    /// ranges within the buffer of fields that have been encountered as the buffer is parsed
    field_ranges: Rc<T>,

    field_num: u64,
    expected_wire_type: u64,

    /// pointer to the next range (containing the serialized message) that the iterator will yield
    /// when `next` invoked
    next_range: Option<(usize, usize)>,

    values_exhausted: bool,
}

impl<'a, T> RepeatedFieldProtoBytesParser<'a, T>
where
    T: FieldRanges,
{
    /// Create a new instance of `RepeatedFieldProtoBytesParser` with the same buffer and parser
    /// state as the passed `ProtoByteParser` that will implement iterator for the given field
    #[must_use]
    pub fn from_byte_parser(
        other: &ProtoBytesParser<'a, T>,
        field_num: u64,
        expected_wire_type: u64,
    ) -> Self {
        Self {
            buf: other.buf,
            pos: other.pos.clone(),
            field_ranges: other.field_ranges.clone(),
            field_num,
            expected_wire_type,
            next_range: None,
            values_exhausted: false,
        }
    }
}

impl<'a, T> Iterator for RepeatedFieldProtoBytesParser<'a, T>
where
    T: FieldRanges,
{
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.values_exhausted {
            return None;
        }

        // initialize first range
        while self.next_range.is_none() {
            // try to get the field offset if it is known
            let range = self.field_ranges.get_field_range(self.field_num);

            match range {
                Some(range) => self.next_range = Some(range),
                None => {
                    // advance
                    let pos = self.pos.get();
                    if pos >= self.buf.len() {
                        // end of buffer, field not found
                        return None;
                    }

                    let (tag, next_pos) = read_varint(self.buf, pos)?;
                    let field = tag >> 3;
                    let wire_type = tag & 7;

                    let (start, end) = field_value_range(self.buf, wire_type, next_pos)?;

                    // save the offset of the field we've encountered
                    self.field_ranges
                        .set_field_range(field, wire_type, start, end);

                    self.pos.set(end)
                }
            }
        }

        let mut range = self
            .next_range
            .expect("iter position should be initialized");

        // this is the return value
        let slice = &self.buf[range.0..range.1];

        // advance until until either we've found the next repeated value, or the end is reached
        loop {
            // if we're at end of buffer, stop iterating
            if range.0 >= self.buf.len() || range.1 >= self.buf.len() {
                self.values_exhausted = true;
                break;
            }

            let (tag, next_pos) = read_varint(self.buf, range.1)?;
            let field = tag >> 3;
            let wire_type = tag & 7;
            range = field_value_range(self.buf, wire_type, next_pos)?;

            if field == self.field_num && wire_type == self.expected_wire_type {
                break;
            }

            // save the offset of the field we've encountered
            self.field_ranges
                .set_field_range(field, wire_type, range.0, range.1);
        }

        // update pointers for continued parsing
        self.pos.set(range.1);
        self.next_range = Some(range);

        Some(slice)
    }
}

/// This is a helper trait for adapting iterators of primitive fields using the packed encoding
/// so that they can be used generically in [`RepeatedPrimitiveIter`]
trait PackedIter<'a> {
    type DecodedValue;

    fn new(buffer: &'a [u8]) -> Self;

    fn decode_value(slice: &[u8]) -> Option<Self::DecodedValue>;
}

/// Iterator for producing elements whose type is a repeated primitive where that primitive
/// can be encoded as a fixed64. e.g. `repeated double` or `repeated fixed64` and the field
/// is encoded using packed encoding.
///
/// Note this is for internal use only. The proto documentation states that decoders should
/// have the flexibility to accept both packed and expanded encodings. Unless we're sure that
/// the field is packed encoded, it's safer to use [`RepeatedFixed64Iter`] which automatically
/// handles both encodings.
pub struct PackedFixed64Iter<'a, V: FromFixed64> {
    buffer: &'a [u8],
    pos: usize,
    _pd: PhantomData<V>,
}

impl<'a, V> PackedIter<'a> for PackedFixed64Iter<'a, V>
where
    V: FromFixed64,
{
    type DecodedValue = V;

    fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            pos: 0,
            _pd: PhantomData,
        }
    }

    fn decode_value(slice: &[u8]) -> Option<Self::DecodedValue> {
        let slice: [u8; 8] = slice.try_into().ok()?;
        Some(V::from_fixed64_slice(slice))
    }
}

impl<'a, V> Iterator for PackedFixed64Iter<'a, V>
where
    V: FromFixed64,
{
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + 8 > self.buffer.len() {
            // we've reached end of buffer
            return None;
        }

        let slice: [u8; 8] = self.buffer[self.pos..self.pos + 8]
            .try_into()
            .expect("can convert slice of fixed size");
        self.pos += 8;
        Some(V::from_fixed64_slice(slice))
    }
}

/// Helper trait for converting an iterator of slices of ranges from a proto buffer into a value
/// that can be produced from a byte slice with a wire type of FIXED64 (e.g. double)
pub trait FromFixed64 {
    /// create new value from a slice that was proto serialized with wire type FIXED64
    fn from_fixed64_slice(slice: [u8; 8]) -> Self;
}

impl FromFixed64 for f64 {
    fn from_fixed64_slice(slice: [u8; 8]) -> Self {
        f64::from_le_bytes(slice)
    }
}

impl FromFixed64 for u64 {
    fn from_fixed64_slice(slice: [u8; 8]) -> Self {
        u64::from_le_bytes(slice)
    }
}

/// Iterator for producing elements whose type is a repeated primitive where that primitive
/// can be encoded as a varint. e.g. `repeated uint64`. and the field is encoded using packed
/// encoding.
///
/// Note this is for internal use only. The proto documentation states that decoders should
/// have the flexibility to accept both packed and expanded encodings. Unless we're sure that
/// the field is packed encoded, it's safer to use [`RepeatedFixed64Iter`] which automatically
/// handles both encodings.
pub struct PackedVarintIter<'a> {
    buffer: &'a [u8],
    pos: usize,
}

impl<'a> PackedIter<'a> for PackedVarintIter<'a> {
    type DecodedValue = u64;

    fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, pos: 0 }
    }

    fn decode_value(slice: &[u8]) -> Option<Self::DecodedValue> {
        let (val, _) = read_varint(slice, 0)?;
        Some(val)
    }
}

// Note this only produces u64 currently as that just happens to be the only type of value in the
// OTLP data model that ends up getting encoded like that. if we ever need this to be generic over
// the return type, we could easily do that (like we do for [`PackedFixed64Iter`])
impl<'a> Iterator for PackedVarintIter<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let (val, next_pos) = read_varint(self.buffer, self.pos)?;
        self.pos = next_pos;

        Some(val)
    }
}

/// Trait for [`FieldRanges`] that also contain repeated primitive fields which may be encoded
/// using either packed or expanded encoding.
///
/// The expectation here is that the implementation of the trait will also discover the type
/// of encoding that is used when `FieldRanges::set_field_range` is set because the wire type
/// is also passed to this call.
pub trait RepeatedFieldEncodings: FieldRanges {
    /// returns `true` if the field is using packed encoding or `false` if the field is using
    /// expanded encoding.
    fn is_packed(&self, field_num: u64) -> bool;
}

/// Generic iterator for repeated primitive fields that are possibly using the packed encoding
///
/// This iterator will determine the encoding, and produce the decoded field values (of type `V`)
/// from the proto buffer which contains the repeated values.
pub struct RepeatedPrimitiveIter<'a, T: RepeatedFieldEncodings, V, P> {
    buf: &'a [u8],
    field_num: u64,
    pos: Rc<Cell<usize>>,
    field_ranges: Rc<T>,

    // expected wire type for the repeated field .. e.g. if the field is type `repeated double`
    // this would be wire_types::FIXED64
    repeated_wire_type: u64,

    // this will be used to iterate over either:
    // - the repeated values (in case that the encoding is expanded)
    // - the segments containing the packed fields (in case that the encoding is packed)
    field_iter: Option<RepeatedFieldProtoBytesParser<'a, T>>,

    // this will be initialized lazily if the encoding is packed. one instance of the packed
    // encoder (type P) wil be created for each segment of the buffer containing packed values
    packed_iter: Option<P>,

    // flag for whether the field is packed or expanded. this will be initialized lazily
    // as well once the encoding is determined
    packed: bool,

    _pd: PhantomData<V>,
}

impl<'a, T, V, P> RepeatedPrimitiveIter<'a, T, V, P>
where
    T: RepeatedFieldEncodings,
{
    fn from_byte_parser_inner(
        other: &ProtoBytesParser<'a, T>,
        field_num: u64,
        repeated_wire_type: u64,
    ) -> Self {
        Self {
            buf: other.buf,
            field_num,
            pos: other.pos.clone(),
            field_ranges: other.field_ranges.clone(),
            repeated_wire_type,
            _pd: PhantomData,

            // following fields will be initialized lazily once encoding determined
            packed: Default::default(),
            field_iter: None,
            packed_iter: None,
        }
    }
}

impl<'a, T, V, P> Iterator for RepeatedPrimitiveIter<'a, T, V, P>
where
    P: PackedIter<'a, DecodedValue = V> + Iterator<Item = V>,
    T: RepeatedFieldEncodings,
{
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        // initialize the inner iterator
        while self.field_iter.is_none() {
            let range = self.field_ranges.get_field_range(self.field_num);
            match range {
                Some(range) => {
                    self.packed = self.field_ranges.is_packed(self.field_num);
                    self.field_iter = Some(RepeatedFieldProtoBytesParser {
                        buf: self.buf,
                        pos: self.pos.clone(),
                        field_ranges: self.field_ranges.clone(),
                        field_num: self.field_num,
                        next_range: Some(range),
                        values_exhausted: false,
                        expected_wire_type: if self.packed {
                            wire_types::LEN
                        } else {
                            self.repeated_wire_type
                        },
                    });
                }
                None => {
                    // advance
                    let pos = self.pos.get();
                    if pos >= self.buf.len() {
                        // end of buffer, field not found
                        return None;
                    }

                    let (tag, next_pos) = read_varint(self.buf, pos)?;
                    let field = tag >> 3;
                    let wire_type = tag & 7;

                    let (start, end) = field_value_range(self.buf, wire_type, next_pos)?;

                    // save the offset of the field we've encountered
                    self.field_ranges
                        .set_field_range(field, wire_type, start, end);

                    self.pos.set(end)
                }
            }
        }

        // safety: this will have been initialized already
        let field_iter = self.field_iter.as_mut().expect("field iter initialized");

        if self.packed {
            if self.packed_iter.is_none() {
                let next_packed_slice = field_iter.next()?;
                self.packed_iter = Some(P::new(next_packed_slice));
            }

            let packed_iter = self.packed_iter.as_mut().expect("packed iter initialized");
            match packed_iter.next() {
                Some(val) => Some(val),
                None => {
                    // try to initialize a new packed iter for the next range
                    let next_packed_slice = field_iter.next()?;
                    let mut next_packed_iter = P::new(next_packed_slice);
                    let result = next_packed_iter.next()?;
                    self.packed_iter = Some(next_packed_iter);
                    Some(result)
                }
            }
        } else {
            field_iter.next().and_then(P::decode_value)
        }
    }
}

/// This iterator can be used for repeated primitives whose types are are encoded as `fixed64`.
/// For example it can be used for field types such as `repeated double` or `repeated fixed64`
pub type RepeatedFixed64Iter<'a, T, V> = RepeatedPrimitiveIter<'a, T, V, PackedFixed64Iter<'a, V>>;

impl<'a, T, V> RepeatedFixed64Iter<'a, T, V>
where
    T: RepeatedFieldEncodings,
    V: FromFixed64,
{
    /// Initialize a new instance using the [`ProtoBytesParser`] which was parsing the prot buffer
    /// that contains this repeated field
    #[must_use]
    pub fn from_byte_parser(other: &ProtoBytesParser<'a, T>, field_num: u64) -> Self {
        Self::from_byte_parser_inner(other, field_num, wire_types::FIXED64)
    }
}

/// This iterator can be used for repeated primitives whose types are are encoded as `varint`.
/// For example it can be used for field types such as `repeated uint64`
pub type RepeatedVarintIter<'a, T> = RepeatedPrimitiveIter<'a, T, u64, PackedVarintIter<'a>>;

impl<'a, T> RepeatedVarintIter<'a, T>
where
    T: RepeatedFieldEncodings,
{
    /// Initialize a new instance using the [`ProtoBytesParser`] which was parsing the prot buffer
    /// that contains this repeated field
    #[must_use]
    pub fn from_byte_parser(other: &ProtoBytesParser<'a, T>, field_num: u64) -> Self {
        Self::from_byte_parser_inner(other, field_num, wire_types::VARINT)
    }
}

/// Decode variant at position in buffer
#[inline]
#[must_use]
pub fn read_varint(buf: &[u8], mut pos: usize) -> Option<(u64, usize)> {
    let mut out = 0u64;
    let mut shift = 0u32;

    while pos < buf.len() && shift < 64 {
        let byte = buf[pos];
        pos += 1;

        out |= ((byte & 0x7F) as u64) << shift;

        if byte < 0x80 {
            return Some((out, pos));
        }

        shift += 7;
    }

    None
}

/// Decode 32 bit zigzag encoding
#[inline]
#[must_use]
pub const fn decode_sint32(val: i32) -> i32 {
    (val >> 1) ^ -(val & 1)
}

/// Decode length from byte slice and return a new slice of the buffer and the decoded length.
#[inline]
#[must_use]
pub fn read_len_delim(buf: &[u8], pos: usize) -> Option<(&[u8], usize)> {
    let (len, mut p) = read_varint(buf, pos)?;
    let end = p.checked_add(len as usize)?;
    if end > buf.len() {
        return None;
    }
    let slice = &buf[p..end];
    p = end;
    Some((slice, p))
}

/// Reads 4 bytes from the buffer at the given position, assuming a `fixed32` protobuf field.
///
/// Returns `None` if fewer than 4 bytes remain starting at `pos`.
/// On success, returns a tuple of the 4-byte slice and the updated position (`pos + 4`).
#[inline]
#[must_use]
pub fn read_fixed32(buf: &[u8], pos: usize) -> Option<(&[u8], usize)> {
    let len = 4;
    let end = pos.checked_add(len)?;
    if end > buf.len() {
        return None;
    }
    let slice = &buf[pos..end];
    Some((slice, end))
}

/// Reads 8 bytes from the buffer at the given position, assuming a `fixed64` protobuf field.
///
/// Returns `None` if fewer than 4 bytes remain starting at `pos`.
/// On success, returns a tuple of the 4-byte slice and the updated position (`pos + 8`).
#[inline]
#[must_use]
pub fn read_fixed64(buf: &[u8], pos: usize) -> Option<(&[u8], usize)> {
    let len = 8;
    let end = pos.checked_add(len)?;
    if end > buf.len() {
        return None;
    }
    let slice = &buf[pos..end];
    Some((slice, end))
}

/// Fields like dropped_attributes_count, dropped_events_count and dropped_links count all have
/// this common logic where they're encoded as varints and interpreted to be zero if the field
/// is missing. This helper encapsulates that logic
#[inline]
#[must_use]
pub fn read_dropped_count(buf: Option<&[u8]>) -> u32 {
    match buf {
        Some(slice) => match read_varint(slice, 0) {
            Some((val, _)) => val as u32,
            None => 0,
        },
        None => 0,
    }
}
