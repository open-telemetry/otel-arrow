// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the implementation of the pdata View traits for serialized OTLP protobuf
//! bytes for messages defined in resources.proto

use crate::{
    otlp::bytes::{
        common::{KeyValueIter, RawKeyValue},
        consts::{
            field_num::resource::{RESOURCE_ATTRIBUTES, RESOURCE_DROPPED_ATTRIBUTES_COUNT},
            wire_types,
        },
        decode::{FieldOffsets, ProtoBytesParser, read_varint},
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
    dropped_attributes_count: Option<usize>,
    attributes: Vec<usize>,
}

impl FieldOffsets for ResourceFieldOffsets {
    fn new() -> Self {
        Self {
            dropped_attributes_count: None,
            attributes: Vec::new(),
        }
    }

    fn get_field_offset(&self, field_num: u64) -> Option<usize> {
        if field_num == RESOURCE_DROPPED_ATTRIBUTES_COUNT {
            self.dropped_attributes_count
        } else {
            None
        }
    }

    fn get_repeated_field_offset(&self, field_num: u64, index: usize) -> Option<usize> {
        if field_num == RESOURCE_ATTRIBUTES {
            self.attributes.get(index).copied()
        } else {
            None
        }
    }

    fn set_field_offset(&mut self, field_num: u64, wire_type: u64, offset: usize) {
        match field_num {
            RESOURCE_DROPPED_ATTRIBUTES_COUNT => {
                if wire_type == wire_types::VARINT {
                    self.dropped_attributes_count = Some(offset)
                }
            }
            RESOURCE_ATTRIBUTES => {
                if wire_type == wire_types::LEN {
                    self.attributes.push(offset)
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
        KeyValueIter::new(self.bytes_parser.clone(), RESOURCE_ATTRIBUTES)
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
