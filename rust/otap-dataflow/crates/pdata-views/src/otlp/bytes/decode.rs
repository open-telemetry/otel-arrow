// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! various types & helper functions for decoding serialized protobuf data

use std::cell::Cell;
use std::num::NonZeroUsize;
use std::rc::Rc;

use crate::otlp::bytes::consts::wire_types;

/// Clones the parser, sharing the underlying buffer and interior-mutability state.
/// The cloned instance will share offsets and position with the original.
impl<T: FieldRanges> Clone for ProtoBytesParser<'_, T> {
    fn clone(&self) -> Self {
        Self {
            buf: self.buf,
            pos: self.pos.clone(),
            field_offsets: self.field_offsets.clone(),
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
    field_offsets: Rc<T>,
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
            field_offsets: Rc::new(T::new()),
        }
    }

    /// Advances the parser to the specified scalar field and returns its value as a byte slice,
    /// if found. Parsing proceeds from the current position in the buffer.
    #[inline]
    #[must_use]
    pub fn advance_to_find_field(&self, field_num: u64) -> Option<&'a [u8]> {
        // Check if the field offset is already cached before entering the parsing loop
        if let Some((start, end)) = self.field_offsets.get_field_range(field_num) {
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
            self.field_offsets
                .set_field_range(field, wire_type, start, end);

            // Check if this is the field we're looking for
            if field == field_num {
                return Some(&self.buf[start..end]);
            }
        }

        None
    }
}

/// return the range of the positions in the byte slice containing values. The range is determined
/// from the wire type.
#[inline]
fn field_value_range(buf: &[u8], wire_type: u64, pos: usize) -> Option<(usize, usize)> {
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

    /// offsets within the buffer of fields that have been encountered as the buffer is parsed
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
            field_ranges: other.field_offsets.clone(),
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

/// Fields like dropped_attributes_count, dropped_events_count and dropped_links count have
/// all have this common logic where they're encoded as varints and interpreted to be zero if the
/// field is missing. This helper encapsulates that logic
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
