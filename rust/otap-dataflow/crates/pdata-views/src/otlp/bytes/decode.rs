// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! various types & helper functions for decoding serialized protobuf data

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::otlp::bytes::consts::wire_types;

/// `ProtoBytesParser` is a generic struct that encapsulates the logic for iterating through
/// a buffer containing a serialized protobuf message, identifying the offsets of its fields.
/// The intention is that we'll only need to pass over the buffer once, regardless of in which
/// order the fields are accessed.
///
/// Typically, an instance of this type is embedded within a higher-level type that implements
/// one of the pdata view traits. The buffer is parsed lazily as fields are accessed via the
/// view's methods.
///
/// This type collaborates with an implementation of the `FieldOffsets` trait — often specific
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
pub struct ProtoBytesParser<'a, T: FieldOffsets> {
    /// buffer containing the serialized proto message being parsed
    buf: &'a [u8],

    /// offset within the buffer
    pos: Rc<Cell<usize>>,

    /// offsets within the buffer of fields that have been encountered as the buffer is parsed
    field_offsets: Rc<RefCell<T>>,
}

enum FieldIdentifier {
    Scalar(u64),
    Repeated(u64, usize),
}

impl<'a, T> ProtoBytesParser<'a, T>
where
    T: FieldOffsets,
{
    /// Create a new instance of `ProtoBytesParser`
    #[must_use]
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            buf,
            pos: Rc::new(Cell::new(0)),
            field_offsets: Rc::new(RefCell::new(T::new())),
        }
    }

    /// Advances the parser to the specified scalar field and returns its value as a byte slice,
    /// if found. Parsing proceeds from the current position in the buffer.
    #[inline]
    #[must_use]
    pub fn advance_to_find_field(
        &self,
        field_num: u64,
        expected_wire_type: u64,
    ) -> Option<&'a [u8]> {
        self.advance_to_find_field_internal(FieldIdentifier::Scalar(field_num), expected_wire_type)
    }

    /// Advances the parser to the specified instance of a repeated field (by index) and returns
    /// its value as a byte slice, if found. Parsing continues from the current position.
    #[inline]
    #[must_use]
    pub fn advance_to_find_next_repeated(
        &self,
        field_num: u64,
        field_index: usize,
        expected_wire_type: u64,
    ) -> Option<&'a [u8]> {
        self.advance_to_find_field_internal(
            FieldIdentifier::Repeated(field_num, field_index),
            expected_wire_type,
        )
    }

    fn advance_to_find_field_internal(
        &self,
        field: FieldIdentifier,
        expected_wire_type: u64,
    ) -> Option<&'a [u8]> {
        // this loop advances parsing by one field each iteration until either the field is found
        // or the end of the buffer is reached.
        loop {
            // try to get the field offset if it is known
            let field_offset = {
                let field_offsets = self.field_offsets.borrow();
                match &field {
                    FieldIdentifier::Scalar(field_num) => {
                        field_offsets.get_field_offset(*field_num)
                    }
                    FieldIdentifier::Repeated(field_num, field_index) => {
                        field_offsets.get_repeated_field_offset(*field_num, *field_index)
                    }
                }
            };

            match field_offset {
                Some(offset) => {
                    // field offset is known, so return the slice of the buffer containing the value:
                    let (slice, _) = match expected_wire_type {
                        wire_types::VARINT => read_variant_bytes(self.buf, offset)?,
                        wire_types::FIXED64 => read_fixed64(self.buf, offset)?,
                        wire_types::LEN_DELIMITED => read_len_delim(self.buf, offset)?,
                        wire_types::FIXED_32 => read_fixed32(self.buf, offset)?,
                        _ => {
                            // invalid wire type
                            return None;
                        }
                    };
                    return Some(slice);
                }
                None => {
                    // field offset is not yet known, so advance parsing the buffer by one field...
                    let pos = self.pos.get();
                    if pos >= self.buf.len() {
                        // end of buffer reached, field not found
                        break;
                    }

                    // parse tag & advance
                    let (tag, next_pos) = read_varint(self.buf, pos)?;
                    let field = tag >> 3;
                    let wire_type = tag & 7;

                    // save the offset of the field we've encountered
                    {
                        let mut field_offsets = self.field_offsets.borrow_mut();
                        field_offsets.set_field_offset(field, next_pos);
                    }

                    // advance parsing to the start of the next field by skipping over the value
                    self.advance_past_value(wire_type, next_pos)?;
                }
            }
        }

        None
    }

    /// advance the position pointer to the position after the with the given wire type at the
    /// given position. Returns None for unknown wire_types or if the next position cannot be
    /// found (e.g. if the buffer ends before can successfully read a full varint).
    #[inline]
    fn advance_past_value(&self, wire_type: u64, pos: usize) -> Option<()> {
        let next_pos = match wire_type {
            wire_types::VARINT => {
                let (_, p) = read_varint(self.buf, pos)?;
                p
            }
            wire_types::FIXED64 => pos + 8,
            wire_types::LEN_DELIMITED => {
                let (_, p) = read_len_delim(self.buf, pos)?;
                p
            }
            wire_types::FIXED_32 => pos + 4,
            _ => return None,
        };
        self.pos.set(next_pos);
        Some(())
    }
}

/// Clones the parser, sharing the underlying buffer and interior-mutability state.
/// The cloned instance will share offsets and position with the original.
impl<T: FieldOffsets> Clone for ProtoBytesParser<'_, T> {
    fn clone(&self) -> Self {
        Self {
            buf: self.buf,
            pos: self.pos.clone(),
            field_offsets: self.field_offsets.clone(),
        }
    }
}

/// The `FieldOffsets` trait defines an interface for tracking the positions (offsets)
/// of fields within a serialized protobuf buffer.
///
/// Implementations act as lightweight offset repositories, optimized for fast access
/// during parsing. For best performance, avoid unnecessary heap allocations — prefer
/// fixed-size or stack-based storage where possible.
pub trait FieldOffsets {
    /// Creates a new, empty instance of this `FieldOffsets` implementation.
    fn new() -> Self;

    /// Returns the offset of the given scalar field number, if known.
    fn get_field_offset(&self, field_num: u64) -> Option<usize>;

    /// Returns the offset of a specific index within a repeated field, if known.
    fn get_repeated_field_offset(&self, field_num: u64, index: usize) -> Option<usize>;

    /// Records the offset of a field in the buffer.
    ///
    /// For repeated fields, this may be called multiple times, in the order that field
    /// instances are encountered during parsing.
    fn set_field_offset(&mut self, field_num: u64, offset: usize);
}

/// Decode variant at position in buffer
#[inline]
#[must_use]
pub fn read_varint(buf: &[u8], mut pos: usize) -> Option<(u64, usize)> {
    let mut out = 0u64;
    let mut shift = 0;
    while pos < buf.len() {
        let b = buf[pos];
        pos += 1;
        out |= ((b & 0x7F) as u64) << shift;
        if b & 0x80 == 0 {
            return Some((out, pos));
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }
    None
}

/// return the byte slice containing a variant
#[inline]
#[must_use]
pub fn read_variant_bytes(buf: &[u8], mut pos: usize) -> Option<(&[u8], usize)> {
    let start = pos;
    let mut shift = 0;
    while pos < buf.len() {
        if buf[pos] & 0x80 == 0 {
            return Some((&buf[start..pos + 1], pos));
        }
        pos += 1;
        shift += 7;
        if shift >= 64 {
            return None;
        }
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
