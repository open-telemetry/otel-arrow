// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the implementation of the pdata View traits for serialized OTLP protobuf
//! bytes for messages defined in resources.proto


use crate::{
    otlp::bytes::{
        common::{KeyValueIterV2, RawKeyValue},
        consts::{
            field_num::resource::{RESOURCE_ATTRIBUTES, RESOURCE_DROPPED_ATTRIBUTES_COUNT},
            wire_types,
        },
        decode::{read_varint, FieldOffsets, ProtoBytesParser, RepeatedFieldProtoBytesParser},
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
    first_attribute: Option<usize>
    // attributes: Vec<usize>,
}

impl FieldOffsets for ResourceFieldOffsets {
    fn new() -> Self {
        Self {
            dropped_attributes_count: None,
            first_attribute: None,
            // attributes: Vec::new(),
        }
    }

    fn get_field_offset(&self, field_num: u64) -> Option<usize> {
        match field_num {
            RESOURCE_ATTRIBUTES => self.first_attribute,
            RESOURCE_DROPPED_ATTRIBUTES_COUNT => self.dropped_attributes_count,
            _ => None
        }
        // if field_num == RESOURCE_DROPPED_ATTRIBUTES_COUNT {
        //     self.dropped_attributes_count
        // } else if field_num ==  {
        //     None
        // }
    }

    fn get_repeated_field_offset(&self, field_num: u64, index: usize) -> Option<usize> {
        panic!("shouldn't be here")
        // if field_num == RESOURCE_ATTRIBUTES {
        //     self.attributes.get(index).copied()
        // } else {
        //     None
        // }
    }

    fn set_field_offset(&mut self, field_num: u64, wire_type: u64, offset: usize) {
        match field_num {
            RESOURCE_DROPPED_ATTRIBUTES_COUNT => {
                if wire_type == wire_types::VARINT {
                    self.dropped_attributes_count = Some(offset)
                }
            }
            RESOURCE_ATTRIBUTES => {
                if self.first_attribute.is_none() &&  wire_type == wire_types::LEN {
                    self.first_attribute = Some(offset)
                    // self.attributes.push(offset)
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
        = KeyValueIterV2<'att, ResourceFieldOffsets>
    where
        Self: 'att;

    fn attributes(&self) -> Self::AttributesIter<'_> {
        KeyValueIterV2::new(
            RepeatedFieldProtoBytesParser::from_byte_parser(&self.bytes_parser,
                RESOURCE_ATTRIBUTES,
                wire_types::LEN
            )
        )
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
