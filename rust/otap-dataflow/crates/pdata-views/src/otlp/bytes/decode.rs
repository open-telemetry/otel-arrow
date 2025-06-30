// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! various types & helper functions for decoding serialized protobuf data

use std::cell::{Cell, RefCell};
use std::num::NonZeroUsize;
use std::rc::Rc;

use crate::otlp::bytes::consts::{field_num, wire_types};

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
    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)>;

    /// Records the offset of a field in the buffer.
    ///
    /// The implementation should also ignore if an offset is passed for an invalid wire type
    ///
    /// For repeated fields, this may be called multiple times, in the order that field
    /// instances are encountered during parsing.
    // TODO add comments about the interior mutability here
    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize);

    /// TODO docs
    // TODO maybe this shouldn't be on the trait
    fn map_nonzero_range_to_primitive(
        range: Option<(NonZeroUsize, NonZeroUsize)>,
    ) -> Option<(usize, usize)> {
        range.map(|(start, end)| (start.get(), end.get()))
    }

    /// TODO docs
    // TODO maybe this shouldn't be on the trait
    fn to_nonzero_range(start: usize, end: usize) -> Option<(NonZeroUsize, NonZeroUsize)> {
        let startnz = NonZeroUsize::new(start)?;
        let endnz = NonZeroUsize::new(end)?;

        Some((startnz, endnz))
    }
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
    field_offsets: Rc<T>,
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
            field_offsets: Rc::new(T::new()),
        }
    }

    /// Advances the parser to the specified scalar field and returns its value as a byte slice,
    /// if found. Parsing proceeds from the current position in the buffer.
    #[inline]
    #[must_use]
    pub fn advance_to_find_field(
        &self,
        field_num: u64,
    ) -> Option<&'a [u8]> {
        // this loop advances parsing by one field each iteration until either the field is found
        // or the end of the buffer is reached.
        loop {
            // try to get the field offset if it is known
            let range = self.field_offsets.get_field_range(field_num);

            match range {
                Some((start, end)) => {
                    let slice = &self.buf[start..end];
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

                    let (start, end) = field_value_range(self.buf, wire_type, next_pos)?;
                    self.pos.set(end);

                    // save the offset of the field we've encountered
                    self.field_offsets
                        .set_field_range(field, wire_type, start, end);
                }
            }
        }

        None
    }

}

// TODO comments
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

/// TODO comments
pub struct RepeatedFieldProtoBytesParser<'a, T: FieldOffsets> {
    // TODO this could contain the PRotoByteParser and just call into it's methods
    // might make the code cleaner
    buf: &'a [u8],
    pos: Rc<Cell<usize>>,
    field_offsets: Rc<T>,

    field_num: u64,
    expected_wire_type: u64,
    curr_range: Option<(usize, usize)>,
    // TODO there's an optimization to be made in this thing where we can bomb out early
    // if we keep track of the last index of the list so we don't always iterate right
    // to the end of the buffer
}

impl<'a, T> RepeatedFieldProtoBytesParser<'a, T>
where
    T: FieldOffsets,
{
    /// Create a new instance of `RepeatedFieldProtoBytesParser` with the same buffer and parser
    /// state as the passed `ProtoByteParser` that will implement iterator for the given field
    pub fn from_byte_parser(
        other: &ProtoBytesParser<'a, T>,
        field_num: u64,
        expected_wire_type: u64,
    ) -> Self {
        Self {
            buf: other.buf,
            pos: other.pos.clone(),
            field_offsets: other.field_offsets.clone(),

            field_num,
            expected_wire_type,
            curr_range: None,
        }
    }
}

impl<'a, T> Iterator for RepeatedFieldProtoBytesParser<'a, T>
where
    T: FieldOffsets,
{
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        while self.curr_range.is_none() {
            // try to get the field offset if it is known
            let range = self.field_offsets.get_field_range(self.field_num);

            match range {
                Some(range) => self.curr_range = Some(range),
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
                    self.field_offsets
                        .set_field_range(field, wire_type, start, end);

                    // TODO, could we avoid setting and getting cell in the loop here for better perf?
                    self.pos.set(end)
                }
            }
        }

        let mut range = self
            .curr_range
            .expect("iter position should be initialized");
        if range.0 >= self.buf.len() {
            return None;
        }
        let slice = &self.buf[range.0..range.1];

        // keep advancing until next_pos points really at the end of the buffer
        loop {
            // if we're at end of buffer, stop iterating
            if range.0 >= self.buf.len() {
                break;
            }
            let (tag, next_pos) = if range.1 < self.buf.len() {
                read_varint(self.buf, range.1)?
            } else {
                // set this as signal that end of buffer is reached
                range.0 = self.buf.len();
                break;
            };
            let field = tag >> 3;
            let wire_type = tag & 7;

            range = field_value_range(self.buf, wire_type, next_pos)?;

            if field == self.field_num && wire_type == self.expected_wire_type {
                break;
            }

            // save the offset of the field we've encountered
            self.field_offsets
                .set_field_range(field, wire_type, range.0, range.1);

        }

        self.pos.set(range.1);
        self.curr_range = Some(range);

        return Some(slice);
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
