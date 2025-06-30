// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the implementation of the pdata View traits for serialized OTLP protobuf
//! bytes for messages defined in common.proto

use std::borrow::Cow;
use std::cell::Cell;
use std::num::NonZeroUsize;

use crate::otlp::bytes::consts::field_num::common::{
    ANY_VALUE_ARRAY_VALUE, ANY_VALUE_BOOL_VALUE, ANY_VALUE_BYES_VALUE, ANY_VALUE_DOUBLE_VALUE,
    ANY_VALUE_INT_VALUE, ANY_VALUE_KVLIST_VALUE, ANY_VALUE_STRING_VALUE, ARRAY_VALUE_VALUES,
    INSTRUMENTATION_DROPPED_ATTRIBUTES_COUNT, INSTRUMENTATION_SCOPE_ATTRIBUTES,
    INSTRUMENTATION_SCOPE_NAME, INSTRUMENTATION_SCOPE_VERSION, KEY_VALUE_KEY,
    KEY_VALUE_LIST_VALUES, KEY_VALUE_VALUE,
};
use crate::otlp::bytes::consts::wire_types;
use crate::otlp::bytes::decode::{
    FieldRanges, ProtoBytesParser, RepeatedFieldProtoBytesParser,
    from_option_nonzero_range_to_primitive, read_fixed64, read_len_delim, read_varint,
    to_nonzero_range,
};
use crate::views::common::{AnyValueView, AttributeView, InstrumentationScopeView, ValueType};

/// Implementation of `AttributeView` backed by protobuf serialized `KeyValue` message
pub struct RawKeyValue<'a> {
    // serialized message
    buf: &'a [u8],

    // the position we have reached while iterating the buffer
    pos: Cell<usize>,

    // the offsets for key & value - will initially be None, but will initialized to Some as we
    // iterate through the buffer and see the field tags for these fields.
    key_offset: Cell<Option<usize>>,
    value_offset: Cell<Option<usize>>,
}

impl<'a> RawKeyValue<'a> {
    #[inline]
    fn new(buf: &'a [u8]) -> Self {
        Self {
            buf,
            pos: Cell::new(0),
            value_offset: Cell::new(None),
            key_offset: Cell::new(None),
        }
    }

    /// advance the buffer by one field, and set the offset for the field if found
    fn advance(&self) {
        let pos = self.pos.get();
        if pos >= self.buf.len() {
            // reach end of buffer, don't advance
            return;
        }

        let (tag, next_pos) = match read_varint(self.buf, pos) {
            Some((tag, next_pos)) => (tag, next_pos),
            // invalid bytes in buffer
            None => return,
        };
        self.pos.set(next_pos);

        let field = tag >> 3;

        match field {
            KEY_VALUE_KEY => self.key_offset.set(Some(next_pos)),
            KEY_VALUE_VALUE => self.value_offset.set(Some(next_pos)),
            _ => {
                // ignore invalid field
            }
        }
    }
}

/// RawAnyValue implements `AnyValueView` backed by a byte buffer containing protobuf serialized
/// `AnyValue` message
pub struct RawAnyValue<'a> {
    buf: &'a [u8],

    // the variant, which will be determined from the field tag while parsing the buffer
    variant: Cell<Option<ValueType>>,

    // the offset in the buffer of the value. will be set to None, and will initialized to Some
    // as we parse the buffer and determine the value start.
    value_offset: Cell<Option<usize>>,
}

impl<'a> RawAnyValue<'a> {
    /// create a new instance of RawAnyValue
    #[inline]
    #[must_use]
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            buf,
            value_offset: Cell::new(None),
            variant: Cell::new(None),
        }
    }
}

/// Implementation of `InstrumentationScopeView` backed by protobuf serialized
/// `InstrumentationScope` message
pub struct RawInstrumentationScope<'a> {
    bytes_parser: ProtoBytesParser<'a, InstrumentationScopeFieldOffsets>,
}

impl<'a> RawInstrumentationScope<'a> {
    /// create a new instance of `RawInstrumentationScope`
    #[inline]
    #[must_use]
    pub fn new(bytes_parser: ProtoBytesParser<'a, InstrumentationScopeFieldOffsets>) -> Self {
        Self { bytes_parser }
    }
}

/// known field offsets for buffer containing InstrumentationScope message
pub struct InstrumentationScopeFieldOffsets {
    name: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    version: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    dropped_attributes_count: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_attribute: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for InstrumentationScopeFieldOffsets {
    fn new() -> Self {
        Self {
            name: Cell::new(None),
            version: Cell::new(None),
            dropped_attributes_count: Cell::new(None),
            first_attribute: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            INSTRUMENTATION_SCOPE_NAME => self.name.get(),
            INSTRUMENTATION_SCOPE_VERSION => self.version.get(),
            INSTRUMENTATION_DROPPED_ATTRIBUTES_COUNT => self.dropped_attributes_count.get(),
            INSTRUMENTATION_SCOPE_ATTRIBUTES => self.first_attribute.get(),
            _ => None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        let range = match to_nonzero_range(start, end) {
            Some(range) => Some(range),
            None => return,
        };

        match field_num {
            INSTRUMENTATION_SCOPE_NAME => {
                if wire_type == wire_types::LEN {
                    self.name.set(range);
                }
            }
            INSTRUMENTATION_SCOPE_VERSION => {
                if wire_type == wire_types::LEN {
                    self.version.set(range);
                }
            }
            INSTRUMENTATION_DROPPED_ATTRIBUTES_COUNT => {
                if wire_type == wire_types::VARINT {
                    self.dropped_attributes_count.set(range);
                }
            }
            INSTRUMENTATION_SCOPE_ATTRIBUTES => {
                if self.first_attribute.get().is_none() && wire_type == wire_types::LEN {
                    self.first_attribute.set(range);
                }
            }
            _ => {
                // ignore unknown field_num
            }
        }
    }
}

/* ───────────────────────────── ADAPTER ITERATORS ─────────────────────── */

/// Iterator of KeyValues - produces implementation of KeyValueView from the byte array which
/// contains a protobuf serialized repeated KeyValues
pub struct KeyValueIter<'a, T: FieldRanges> {
    bytes_parser: RepeatedFieldProtoBytesParser<'a, T>,
}

impl<'a, T> KeyValueIter<'a, T>
where
    T: FieldRanges,
{
    /// Create a new instance of `KeyValueIter`
    #[must_use]
    pub fn new(bytes_parser: RepeatedFieldProtoBytesParser<'a, T>) -> Self {
        Self { bytes_parser }
    }
}

impl<'a, T> Iterator for KeyValueIter<'a, T>
where
    T: FieldRanges,
{
    type Item = RawKeyValue<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.bytes_parser.next()?;
        Some(RawKeyValue::new(slice))
    }
}

/// Iterator of AnyValues - produces implementation of AnyValueView from the byte array which
/// contains a protobuf serialized repeated AnyValues
pub struct AnyValueIter<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Iterator for AnyValueIter<'a> {
    type Item = RawAnyValue<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.buf.len() {
            let (tag, next_pos) = read_varint(self.buf, self.pos)?;
            self.pos = next_pos;
            let field = tag >> 3;
            let wire_type = tag & 7;

            if field == ARRAY_VALUE_VALUES && wire_type == wire_types::LEN {
                let (slice, next_pos) = read_len_delim(self.buf, self.pos)?;
                self.pos = next_pos;
                return Some(RawAnyValue::new(slice));
            }
        }

        None
    }
}

/* ───────────────────────────── TRAIT IMPLEMENTATIONS ─────────────────── */

impl AttributeView for RawKeyValue<'_> {
    type Val<'val>
        = RawAnyValue<'val>
    where
        Self: 'val;

    fn key(&self) -> crate::views::common::Str<'_> {
        loop {
            if let Some(offset) = self.key_offset.get() {
                let (slice, _) = match read_len_delim(self.buf, offset) {
                    Some((slice, next_pos)) => (slice, next_pos),
                    // break if cannot read length of key
                    None => break,
                };

                match std::str::from_utf8(slice).ok() {
                    Some(str) => return Cow::Borrowed(str),
                    // break for invalid strings
                    None => break,
                }
            } else if self.pos.get() >= self.buf.len() {
                break;
            } else {
                self.advance();
            }
        }

        // return empty string when cannot read key
        Cow::Borrowed("")
    }

    fn value(&self) -> Option<Self::Val<'_>> {
        loop {
            if let Some(offset) = self.value_offset.get() {
                let (slice, _) = match read_len_delim(self.buf, offset) {
                    Some((slice, next_pos)) => (slice, next_pos),
                    None => break,
                };

                return Some(RawAnyValue {
                    buf: slice,
                    value_offset: Cell::new(None),
                    variant: Cell::new(None),
                });
            } else if self.pos.get() >= self.buf.len() {
                break;
            } else {
                self.advance();
            }
        }

        None
    }
}

impl<'a> AnyValueView<'a> for RawAnyValue<'a> {
    type KeyValue = RawKeyValue<'a>;

    type KeyValueIter<'kv>
        = KeyValueIter<'a, KeyValuesListFieldOffsets>
    where
        Self: 'kv;

    type ArrayIter<'att>
        = AnyValueIter<'a>
    where
        Self: 'att;

    #[inline]
    fn value_type(&self) -> ValueType {
        match self.variant.get() {
            Some(variant_type) => variant_type,
            None => {
                let variant_type = match read_varint(self.buf, 0) {
                    Some((tag, pos)) => {
                        let field = tag >> 3;
                        self.value_offset.set(Some(pos));

                        match field {
                            ANY_VALUE_STRING_VALUE => ValueType::String,
                            ANY_VALUE_BOOL_VALUE => ValueType::Bool,
                            ANY_VALUE_INT_VALUE => ValueType::Int64,
                            ANY_VALUE_DOUBLE_VALUE => ValueType::Double,
                            ANY_VALUE_ARRAY_VALUE => ValueType::Array,
                            ANY_VALUE_KVLIST_VALUE => ValueType::KeyValueList,
                            ANY_VALUE_BYES_VALUE => ValueType::Bytes,
                            _ => {
                                // treat unknown types as an empty value
                                ValueType::Empty
                            }
                        }
                    }
                    None => ValueType::Empty,
                };

                self.variant.set(Some(variant_type));
                variant_type
            }
        }
    }

    fn as_string(&self) -> Option<crate::views::common::Str<'_>> {
        if self.value_type() == ValueType::String {
            // safety: this value should have been initialized in the call to self.value_type
            let value_offset = self
                .value_offset
                .get()
                .expect("expect to have been initialized");
            let (slice, _) = read_len_delim(self.buf, value_offset)?;
            std::str::from_utf8(slice).ok().map(Cow::Borrowed)
        } else {
            None
        }
    }

    fn as_bool(&self) -> Option<bool> {
        if self.value_type() == ValueType::Bool {
            // safety: this value should have been initialized in the call to self.value_type
            let value_offset = self
                .value_offset
                .get()
                .expect("expect to have been initialized");

            // bools are encoded as varint where 1 == true and 0 == false
            let (val, _) = read_varint(self.buf, value_offset)?;
            Some(val == 1)
        } else {
            None
        }
    }

    fn as_bytes(&self) -> Option<&[u8]> {
        if self.value_type() == ValueType::Bytes {
            // safety: this value should have been initialized in the call to self.value_type
            let value_offset = self
                .value_offset
                .get()
                .expect("expect to have been initialized");
            let (slice, _) = read_len_delim(self.buf, value_offset)?;
            Some(slice)
        } else {
            None
        }
    }

    fn as_double(&self) -> Option<f64> {
        if self.value_type() == ValueType::Double {
            // safety: this value should have been initialized in the call to self.value_type
            let value_offset = self
                .value_offset
                .get()
                .expect("expect to have been initialized");
            let (slice, _) = read_fixed64(self.buf, value_offset)?;
            let byte_arr: [u8; 8] = slice.try_into().ok()?;
            Some(f64::from_le_bytes(byte_arr))
        } else {
            None
        }
    }

    fn as_int64(&self) -> Option<i64> {
        if self.value_type() == ValueType::Int64 {
            // safety: this value should have been initialized in the call to self.value_type
            let value_offset = self
                .value_offset
                .get()
                .expect("expect to have been initialized");
            let (val, _) = read_varint(self.buf, value_offset)?;
            Some(val as i64)
        } else {
            None
        }
    }

    fn as_array(&self) -> Option<Self::ArrayIter<'_>> {
        if self.value_type() == ValueType::Array {
            // safety: this value should have been initialized in the call to self.value_type
            let value_offset = self
                .value_offset
                .get()
                .expect("expect to have been initialized");
            let (slice, _) = read_len_delim(self.buf, value_offset)?;
            Some(AnyValueIter { buf: slice, pos: 0 })
        } else {
            None
        }
    }

    fn as_kvlist(&self) -> Option<Self::KeyValueIter<'_>> {
        if self.value_type() == ValueType::KeyValueList {
            // safety: this value should have been initialized in the call to self.value_type
            let value_offset = self
                .value_offset
                .get()
                .expect("expect to have been initialized");
            let (slice, _) = read_len_delim(self.buf, value_offset)?;
            Some(KeyValueIter::new(
                RepeatedFieldProtoBytesParser::from_byte_parser(
                    &ProtoBytesParser::new(slice),
                    KEY_VALUE_LIST_VALUES,
                    wire_types::LEN,
                ),
            ))
        } else {
            None
        }
    }
}

/// known offset of KeyValuesList
pub struct KeyValuesListFieldOffsets {
    first_offset: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for KeyValuesListFieldOffsets {
    fn new() -> Self {
        Self {
            first_offset: Cell::new(None),
        }
    }

    fn get_field_range(&self, _field_tag: u64) -> Option<(usize, usize)> {
        self.first_offset
            .get()
            .map(|(start, end)| (start.get(), end.get()))
    }

    fn set_field_range(&self, field_tag: u64, wire_type: u64, start: usize, end: usize) {
        let range = match to_nonzero_range(start, end) {
            Some(range) => Some(range),
            None => return,
        };
        if field_tag == KEY_VALUE_LIST_VALUES
            && wire_type == wire_types::LEN
            && self.first_offset.get().is_none()
        {
            self.first_offset.set(range);
        }
    }
}
impl InstrumentationScopeView for RawInstrumentationScope<'_> {
    type Attribute<'att>
        = RawKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att, InstrumentationScopeFieldOffsets>
    where
        Self: 'att;

    fn name(&self) -> Option<crate::views::common::Str<'_>> {
        let slice = self
            .bytes_parser
            .advance_to_find_field(INSTRUMENTATION_SCOPE_NAME)?;
        std::str::from_utf8(slice).ok().map(Cow::Borrowed)
    }

    fn version(&self) -> Option<crate::views::common::Str<'_>> {
        let slice = self
            .bytes_parser
            .advance_to_find_field(INSTRUMENTATION_SCOPE_VERSION)?;
        std::str::from_utf8(slice).ok().map(Cow::Borrowed)
    }

    fn dropped_attributes_count(&self) -> u32 {
        if let Some(slice) = self
            .bytes_parser
            .advance_to_find_field(INSTRUMENTATION_DROPPED_ATTRIBUTES_COUNT)
        {
            if let Some((val, _)) = read_varint(slice, 0) {
                return val as u32;
            }
        }

        // default = 0 = no dropped attributes
        0
    }

    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(RepeatedFieldProtoBytesParser::from_byte_parser(
            &self.bytes_parser,
            INSTRUMENTATION_SCOPE_ATTRIBUTES,
            wire_types::LEN,
        ))
    }
}
