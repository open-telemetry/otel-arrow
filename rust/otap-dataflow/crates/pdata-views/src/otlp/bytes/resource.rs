// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the implementation of the pdata View traits for serialized OTLP protobuf
//! bytes for messages defined in resources.proto

use std::{cell::Cell, num::NonZeroUsize};

use crate::{
    otlp::bytes::{
        common::{KeyValueIter, RawKeyValue},
        consts::{
            field_num::resource::{RESOURCE_ATTRIBUTES, RESOURCE_DROPPED_ATTRIBUTES_COUNT},
            wire_types,
        },
        decode::{FieldOffsets, ProtoBytesParser, RepeatedFieldProtoBytesParser, read_varint},
    },
    views::resource::ResourceView,
};

/// Implementation of `ResourceView` backed by protobuf serialized `Resource` message
pub struct RawResource<'a> {
    bytes_parser: ProtoBytesParser<'a, ResourceFieldOffsets>,
}

impl<'a> RawResource<'a> {
    /// create a new instance of `RawResource`
    #[must_use]
    pub fn new(bytes_parser: ProtoBytesParser<'a, ResourceFieldOffsets>) -> Self {
        Self { bytes_parser }
    }
}

/// known field offsets for fields in buffer containing Resource message
pub struct ResourceFieldOffsets {
    dropped_attributes_count: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_attribute: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldOffsets for ResourceFieldOffsets {
    fn new() -> Self {
        Self {
            dropped_attributes_count: Cell::new(None),
            first_attribute: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            RESOURCE_ATTRIBUTES => self.first_attribute.get(),
            RESOURCE_DROPPED_ATTRIBUTES_COUNT => self.dropped_attributes_count.get(),
            _ => None,
        };

        Self::map_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        let range = match Self::to_nonzero_range(start, end) {
            Some(range) => Some(range),
            None => return,
        };

        match field_num {
            RESOURCE_DROPPED_ATTRIBUTES_COUNT => {
                if wire_type == wire_types::VARINT {
                    self.dropped_attributes_count.set(range);
                }
            }
            RESOURCE_ATTRIBUTES => {
                if self.first_attribute.get().is_none() && wire_type == wire_types::LEN {
                    self.first_attribute.set(range);
                }
            }
            _ => {
                // ignore unknown fields
            }
        }
    }
}

impl ResourceView for RawResource<'_> {
    type Attribute<'att>
        = RawKeyValue<'att>
    where
        Self: 'att;
    type AttributesIter<'att>
        = KeyValueIter<'att, ResourceFieldOffsets>
    where
        Self: 'att;

    fn attributes(&self) -> Self::AttributesIter<'_> {
        KeyValueIter::new(RepeatedFieldProtoBytesParser::from_byte_parser(
            &self.bytes_parser,
            RESOURCE_ATTRIBUTES,
            wire_types::LEN,
        ))
        // self.bytes_parser.clone(), RESOURCE_ATTRIBUTES)
    }

    fn dropped_attributes_count(&self) -> u32 {
        if let Some(slice) = self
            .bytes_parser
            .advance_to_find_field(RESOURCE_DROPPED_ATTRIBUTES_COUNT, wire_types::VARINT)
        {
            if let Some((val, _)) = read_varint(slice, 0) {
                return val as u32;
            }
        }

        // default = 0 = no attributes dropped
        0
    }
}
